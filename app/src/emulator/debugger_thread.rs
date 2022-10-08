use std::{
    any::Any,
    error::Error,
    sync::{Arc, Mutex},
};

use gdb::{
    async_target::{GDBAsyncNotifier, GDBAsyncStub},
    connection::Connection,
    target::Target,
};
use mips_emulator::cpu::{CpuExternalHandler, EmulatorInterface};

use super::debug_target::{MipsDebugger, MipsTargetInterface};

//------------------------------------------------------------------------

pub type CreateTarget<T> = Box<dyn FnOnce() -> Result<T, Box<dyn Error>> + Send>;
pub type CreateConnection<C> = Box<dyn FnMut() -> Result<Option<C>, Box<dyn Error>> + Send>;
pub type AttachDebugger<C, T> =
    Box<dyn FnOnce(GDBAsyncNotifier<C, T>) -> Result<(), Box<dyn Error>> + Send>;
pub struct DebuggerBuilder<C: Connection + Send + Sync + 'static, T: Target + Send + Sync + 'static>
{
    pub create_target: CreateTarget<T>,
    pub create_connetion: CreateConnection<C>,
    pub attach: AttachDebugger<C, T>,
}

//------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum State {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub connection_str_repr: Option<String>,
    pub packets_sent: usize,
    pub packets_receved: usize,
    pub bytes_sent: usize,
    pub bytes_receved: usize,
}

pub trait DebuggerConnection<T: Target>: Any {
    fn state(&self) -> State;
    fn target<'a>(&mut self, accessor: Box<dyn FnOnce(&mut T) + 'a>);
    fn try_target<'a>(&mut self, accessor: Box<dyn FnOnce(&mut T) + 'a>);
    fn get_connection_info(&mut self) -> Result<ConnectionInfo, Box<dyn Error>>;
    fn try_get_connection_info(&mut self) -> Result<Option<ConnectionInfo>, Box<dyn Error>>;
    fn force_close(&mut self);
}

pub fn mips_emulator_debugger_builder<
    T: CpuExternalHandler,
    C: Connection + Sync + Send + 'static,
>(
    mut interface: EmulatorInterface<T>,
    create_connetion: CreateConnection<C>,
) -> DebuggerBuilder<C, MipsTargetInterface<T>> {
    let inter = interface.clone();
    DebuggerBuilder {
        create_target: Box::new(move || {
            let target = crate::emulator::debug_target::MipsTargetInterface::new(inter);
            Ok(target)
        }),
        attach: Box::new(move |notifier| {
            interface.cpu_mut(|cpu| {
                cpu.attach_debugger(MipsDebugger::new(notifier));
            });
            Ok(())
        }),
        create_connetion,
    }
}

//------------------------------------------------------------------------

pub type DebuggerInfo<T> = Arc<Mutex<dyn DebuggerConnection<T>>>;

pub fn start<
    C: Connection + std::fmt::Debug + Send + Sync + 'static,
    T: Target + std::fmt::Debug + Sync + Send + 'static,
>(
    builder: DebuggerBuilder<C, T>,
) -> Result<DebuggerInfo<T>, Box<dyn Error>> {
    struct DThread<C: Connection + Send + Sync + 'static, T: Target + Send + Sync + 'static> {
        state: State,
        force_close: bool,
        stub: Option<GDBAsyncStub<C, T>>,
    }

    impl<C: Connection + Send + Sync + 'static, T: Target + Send + Sync + 'static>
        DebuggerConnection<T> for DThread<C, T>
    {
        fn state(&self) -> State {
            self.state
        }

        fn target<'a>(&mut self, accessor: Box<dyn FnOnce(&mut T) + 'a>) {
            if let Some(stub) = &mut self.stub {
                let lock = stub.gdb.lock();
                if let Ok(mut lock) = lock {
                    accessor(&mut lock.target);
                }
            }
        }

        fn try_target<'a>(&mut self, accessor: Box<dyn FnOnce(&mut T) + 'a>) {
            if let Some(stub) = &mut self.stub {
                let lock = stub.gdb.try_lock();
                if let Ok(mut lock) = lock {
                    accessor(&mut lock.target);
                }
            }
        }

        fn get_connection_info(&mut self) -> Result<ConnectionInfo, Box<dyn Error>> {
            if let Some(stub) = &mut self.stub {
                let lock = stub.gdb.try_lock();
                match lock {
                    Ok(lock) => Ok(ConnectionInfo {
                        connection_str_repr: lock.connection_string_repr(),
                        packets_sent: lock.packets_sent(),
                        packets_receved: lock.packets_receved(),
                        bytes_sent: lock.bytes_sent(),
                        bytes_receved: lock.bytes_receved(),
                    }),
                    Err(err) => Err(err.to_string().into()),
                }
            } else {
                Err("No connection present".into())
            }
        }

        fn try_get_connection_info(&mut self) -> Result<Option<ConnectionInfo>, Box<dyn Error>> {
            if let Some(stub) = &mut self.stub {
                let lock = stub.gdb.try_lock();
                match lock {
                    Ok(lock) => Ok(Some(ConnectionInfo {
                        connection_str_repr: lock.connection_string_repr(),
                        packets_sent: lock.packets_sent(),
                        packets_receved: lock.packets_receved(),
                        bytes_sent: lock.bytes_sent(),
                        bytes_receved: lock.bytes_receved(),
                    })),
                    Err(std::sync::TryLockError::WouldBlock) => Ok(None),
                    Err(err) => Err(err.to_string().into()),
                }
            } else {
                Err("No connection present".into())
            }
        }

        fn force_close(&mut self) {
            self.force_close = true;
            if let Some(stub) = &self.stub {
                if let Ok(mut stub) = stub.gdb.try_lock() {
                    stub.disconnect(gdb::stub::DisconnectReason::Kill)
                }
            }
        }
    }

    let internal = Arc::new(Mutex::new(DThread::<C, T> {
        state: State::Disconnected,
        stub: None,
        force_close: false,
    }));

    let c = internal.clone();

    let res = crate::platform::thread::start_thread(move || {
        fn create_stub<
            C: Connection + std::fmt::Debug + Send + Sync + 'static,
            T: Target + Send + Sync + 'static,
        >(
            internal: &Arc<Mutex<DThread<C, T>>>,
            builder: DebuggerBuilder<C, T>,
        ) -> Result<GDBAsyncStub<C, T>, Box<dyn Error>> {
            let DebuggerBuilder {
                create_target,
                mut create_connetion,
                attach,
            } = builder;

            internal.lock().unwrap().state = State::Connecting;
            let c = loop {
                if let Some(con) = (create_connetion)()? {
                    break con;
                }
                if internal.lock().unwrap().force_close {
                    internal.lock().unwrap().state = State::Disconnected;
                    return Err("Forced to disconnect by user".into());
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            };

            let t = (create_target)()?;

            let stub = gdb::stub::GDBStub::new(t, c);
            let (stub, notifier) = gdb::async_target::create_async_stub(stub);
            (attach)(notifier)?;
            internal.lock().unwrap().state = State::Connected;
            internal.lock().unwrap().stub = Some(stub.clone());
            Ok(stub)
        }

        match create_stub(&c, builder) {
            Ok(stub) => {
                log::trace!("Starting debugger in seperate thread");
                log::info!("{:?}", stub.run_blocking());
            }
            Err(err) => {
                log::trace!("Error while creating stub: {}", err);
            }
        }
        c.lock().unwrap().state = State::Disconnected;
        c.lock().unwrap().stub = None;
    });
    if let Err(err) = res {
        log::trace!("Error creating debugger thread: {}", err);
        return Err(err);
    }
    Ok(internal)
}
