use std::{
    error::Error,
    sync::{Arc, Mutex}, any::Any,
};

use gdb::{
    async_target::{GDBAsyncNotifier, GDBAsyncStub},
    connection::Connection,
    target::Target,
};
use mips_emulator::{cpu::{CpuExternalHandler, EmulatorInterface}};

use super::{debug_target::{MipsDebugger, MipsTargetInterface, Breakpoint}, handlers::ExternalHandler};

//------------------------------------------------------------------------

pub type CreateTarget<T> = Box<dyn FnOnce() -> Result<T, Box<dyn Error>> + Send>;
pub type CreateConnection<C> = Box<dyn FnOnce() -> Result<C, Box<dyn Error>> + Send>;
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
pub struct ConnectionInfo{
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
                match lock{
                    Ok(lock) => {
                        Ok(ConnectionInfo { 
                            connection_str_repr: lock.connection_string_repr(), 
                            packets_sent: lock.packets_sent(), 
                            packets_receved: lock.packets_receved(),
                            bytes_sent: lock.bytes_sent(), 
                            bytes_receved: lock.bytes_receved(),
                        })
                    },
                    Err(err) => {
                        Err(err.to_string().into())
                    },
                }
            }else{
                Err("No connection present".into())
            }
        }

        fn try_get_connection_info(&mut self) -> Result<Option<ConnectionInfo>, Box<dyn Error>> {
            if let Some(stub) = &mut self.stub {
                let lock = stub.gdb.try_lock();
                match lock{
                    Ok(lock) => {
                        Ok(Some(ConnectionInfo { 
                            connection_str_repr: lock.connection_string_repr(), 
                            packets_sent: lock.packets_sent(), 
                            packets_receved: lock.packets_receved(),
                            bytes_sent: lock.bytes_sent(), 
                            bytes_receved: lock.bytes_receved(),
                        }))
                    },
                    Err(std::sync::TryLockError::WouldBlock)=> {
                        Ok(None)
                    }
                    Err(err) => {
                        Err(err.to_string().into())
                    },
                }
            }else{
                Err("No connection present".into())
            }
        }
    }

    let internal = Arc::new(Mutex::new(DThread::<C, T> {
        state: State::Disconnected,
        stub: None,
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
            internal.lock().unwrap().state = State::Connecting;
            let c = (builder.create_connetion)()?;
            let t = (builder.create_target)()?;
            let stub = gdb::stub::GDBStub::new(t, c);
            let (stub, notifier) = gdb::async_target::create_async_stub(stub);
            (builder.attach)(notifier)?;
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

//------------------------------------------------------------------------

pub struct DisconnectedDebuggerConnection {}

impl<T: Target> DebuggerConnection<T> for DisconnectedDebuggerConnection {
    fn state(&self) -> State {
        State::Disconnected
    }

    fn target<'a>(&mut self, _accessor: Box<dyn FnOnce(&mut T) + 'a>) {}

    fn try_target<'a>(&mut self, _accessor: Box<dyn FnOnce(&mut T) + 'a>) {}

    fn get_connection_info(&mut self) -> Result<ConnectionInfo, Box<dyn Error>> {
        Err("No Connection".into())
    }

    fn try_get_connection_info(&mut self) -> Result<Option<ConnectionInfo>, Box<dyn Error>> {
        Err("No Connection".into())
    }

}

pub trait MipsDebuggerConnection{
    fn state(&self) -> State;

    fn get_breakpoints(&mut self) -> Vec<Breakpoint>;
    fn try_get_breakpoints(&mut self)  -> Vec<Breakpoint>;

    fn get_connection_info(&mut self) -> Result<ConnectionInfo, Box<dyn Error>>;
    fn try_get_connection_info(&mut self)  -> Result<Option<ConnectionInfo>, Box<dyn Error>>;
}

impl<T> MipsDebuggerConnection for T where
    T: DebuggerConnection<MipsTargetInterface<ExternalHandler>>{
    fn get_breakpoints(&mut self) -> Vec<Breakpoint> {
        let mut breakpoints = Vec::new();
        self.target(Box::new(|target|{
            breakpoints = target.breakpoints().to_vec()
        }));
        breakpoints
    }

    fn try_get_breakpoints(&mut self)  -> Vec<Breakpoint> {
        let mut breakpoints = Vec::new();
        self.try_target(Box::new(|target|{
            breakpoints = target.breakpoints().to_vec()
        }));
        breakpoints
    }

    fn state(&self) -> State {
        DebuggerConnection::state(self)
    }

    fn get_connection_info(&mut self) -> Result<ConnectionInfo, Box<dyn Error>> {
        DebuggerConnection::get_connection_info(self)
    }

    fn try_get_connection_info(&mut self)  -> Result<Option<ConnectionInfo>, Box<dyn Error>> {
        DebuggerConnection::try_get_connection_info(self)
    }
}