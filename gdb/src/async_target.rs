use std::sync::{Arc, Mutex, MutexGuard};

use crate::{
    connection::Connection,
    signal::Signal,
    stub::{DisconnectReason, GDBError, GDBStub, StopReason},
    target::Target,
};

pub struct GDBAsyncNotifier<C: Connection, T: Target> {
    pub gdb: Arc<Mutex<GDBStub<C, T>>>,
}

impl<C: Connection, T: Target> GDBAsyncNotifier<C, T> {
    pub fn on_target_stop(&self) {
        self.send_stub_stop_signal(crate::stub::StopReason::Signal(Signal::SIGTRAP))
    }

    pub fn target_stop_signal(&self, reason: StopReason) {
        self.send_stub_stop_signal(reason);
    }

    pub fn detach(&self){
        let mut stub = self.gdb.lock().unwrap();
        stub.detach_and_kill();
    }

    fn send_stub_stop_signal(&self, reason: StopReason) {
        let mut stub = self.gdb.lock().unwrap();
        if stub.is_target_running_or_inturrupt() {
            stub.target_stop(reason);
        }
    }
}

pub struct GDBAsyncStub<C: Connection, T: Target> {
    gdb: Arc<Mutex<GDBStub<C, T>>>,
}

impl<C: Connection, T: Target> GDBAsyncStub<C, T> {
    pub fn run_blocking(self) -> Result<DisconnectReason, GDBError<C, T>> {
        loop {
            let mut lock = self.gdb.lock().map_err(|_| GDBError::ExternalError)?;
            while {
                if let Some(reason) = lock.check_non_blocking()? {
                    return Ok(reason);
                }
                lock.has_data_to_read()
            } {}
            drop(lock);

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

pub fn create_async_stub<C: Connection, T: Target>(
    stub: GDBStub<C, T>,
) -> (GDBAsyncStub<C, T>, GDBAsyncNotifier<C, T>) {
    let shared = Arc::new(Mutex::new(stub));
    (
        GDBAsyncStub {
            gdb: shared.clone(),
        },
        GDBAsyncNotifier { gdb: shared },
    )
}
