use std::sync::{Arc, Mutex};

use crate::{
    connection::Connection,
    signal::Signal,
    stub::{DisconnectReason, GDBError, GDBState, GDBStub, StopReason},
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

    pub fn on_target_detach(&self) {
        self.gdb
            .lock()
            .unwrap()
            .disconnect(DisconnectReason::TargetDisconnected)
    }

    fn send_stub_stop_signal(&self, reason: StopReason) {
        let mut stub = self.gdb.lock().unwrap();
        if stub.is_target_running_or_inturrupt() && stub.target_stop(reason).is_err() {
            stub.detach_target_and_disconnect(DisconnectReason::Error);
        }
    }
}

pub struct GDBAsyncStub<C: Connection, T: Target> {
    pub gdb: Arc<Mutex<GDBStub<C, T>>>,
}

impl<C: Connection, T: Target> Clone for GDBAsyncStub<C, T> {
    fn clone(&self) -> Self {
        Self {
            gdb: self.gdb.clone(),
        }
    }
}

impl<C: Connection, T: Target> GDBAsyncStub<C, T> {
    pub fn run_blocking(mut self) -> Result<DisconnectReason, GDBError<C, T>> {
        loop {
            match self.run_non_blocking() {
                Ok(None) => {}
                Ok(Some(disconnect_reason)) => {
                    self.gdb
                        .lock()
                        .unwrap()
                        .detach_target_and_disconnect(disconnect_reason);
                    return Ok(disconnect_reason);
                }
                Err(err) => {
                    self.gdb
                        .lock()
                        .unwrap()
                        .detach_target_and_disconnect(DisconnectReason::Error);
                    return Err(err);
                }
            };

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    fn run_non_blocking(&mut self) -> Result<Option<DisconnectReason>, GDBError<C, T>> {
        let mut lock = self.gdb.lock().map_err(|_| GDBError::ExternalError)?;
        while {
            if let Some(reason) = lock.check_non_blocking()? {
                return Ok(Some(reason));
            }
            lock.has_data_to_read() && matches!(lock.state(), GDBState::Idle | GDBState::Running)
        } {}
        drop(lock);
        Ok(None)
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
