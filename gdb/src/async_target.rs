use std::sync::{Arc, Mutex};

use crate::{
    connection::Connection,
    stub::{DisconnectReason, GDBError, GDBStub, StopReason},
    target::Target, signal::Signal,
};

pub struct GDBAsyncNotifier<C: Connection, T: Target> {
    gdb: Arc<Mutex<GDBStub<C, T>>>,
}

impl<C: Connection, T: Target> GDBAsyncNotifier<C, T> {
    pub fn on_target_stop(&mut self) {
        let mut stub = self.gdb.lock().unwrap();
        if stub.is_target_running_or_inturrupt() {
            stub.target_stop(crate::stub::StopReason::Signal(Signal::SIGTSTP));
        }
    }

    pub fn target_stop_signal(&mut self, reason: StopReason){
        let mut stub = self.gdb.lock().unwrap();
        stub.target_stop(reason);
        
    }
}

pub struct GDBAsyncStub<C: Connection, T: Target> {
    gdb: Arc<Mutex<GDBStub<C, T>>>,
}

impl<C: Connection, T: Target> GDBAsyncStub<C, T> {
    pub fn run_blocking(self) -> Result<DisconnectReason, GDBError<C>> {
        
        loop {
            let mut lock = self.gdb.lock().map_err(|_|GDBError::ExternalError)?;
            while {
                if let Some(reason) = lock.check_non_blocking()?{
                    return Ok(reason)
                }
                lock.has_data_to_read()
            }{}
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
