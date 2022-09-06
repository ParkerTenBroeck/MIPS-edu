use std::net::TcpListener;

use gdb::{
    async_target::{self, GDBAsyncNotifier},
    connection::Connection,
    stub::{GDBStub},
    target::Target, signal::Signal,
};
use mips_emulator::{
    cpu::{CpuExternalHandler, EmulatorInterface, MipsCpu},
    memory::page_pool::MemoryDefaultAccess,
};

fn main() {
    struct Logger {}
    impl log::Log for Logger {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            println!("{}", record.args());
        }

        fn flush(&self) {}
    }
    _ = log::set_logger(&Logger {});
    log::set_max_level(log::LevelFilter::Trace);
    log::info!("asdasdasd");

    let mut cpu = MipsCpu::<ExternalHandler>::new_interface(ExternalHandler { debugger: None });

    let target = TargetInterface {
        emulator: cpu.clone(),
    };
    let (connection, _addr) = TcpListener::bind("localhost:1234")
        .unwrap()
        .accept()
        .unwrap();
    let stub = GDBStub::new(target, connection);
    let (stub, notifier) = async_target::create_async_stub(stub);

    cpu.cpu_mut(|cpu| unsafe {
        cpu.raw_handler().debugger = Some(Box::new(notifier));
        cpu.raw_mem().set_u32_alligned_be(0, 0xFFBBCCDD);
        cpu.raw_mem().set_u32_alligned_be(4, 0xFFBBCCDD);
        cpu.raw_mem().set_u32_alligned_be(8, 0xFFBBCCDD);
        cpu.raw_mem().set_u32_alligned_be(12, 0xFFBBCCDD);
    });

    // let debugger = DebugServer::new(cpu);
    // let _ = debugger.start_debug_server();
    let handle = std::thread::spawn(|| {
        log::info!("{:?}", stub.run_blocking());
    });
    _ = handle.join();
}

struct TargetInterface {
    emulator: EmulatorInterface<ExternalHandler>,
}

impl Target for TargetInterface {
    fn inturrupt(&mut self) -> Result<(), ()> {
        self.emulator.stop().map_err(|_| ())
    }

    fn step_at(&mut self, addr: Option<u32>) {
        if let Some(addr) = addr {
            self.emulator.cpu_mut(|cpu| cpu.set_pc(addr));
        }
        _ = self.emulator.step_new_thread();
    }

    fn continue_at(&mut self, addr: Option<u32>) {
        if let Some(addr) = addr {
            self.emulator.cpu_mut(|cpu| cpu.set_pc(addr));
        }
        _ = self.emulator.start_new_thread();
    }
}

trait Debugger: Sync + Send + 'static {
    fn on_start(&mut self);
    fn on_stop(&mut self);
    fn on_illegal_opcode(&mut self);
}

struct ExternalHandler {
    pub debugger: Option<Box<dyn Debugger>>,
}

impl<C: Connection + Sync + Send + 'static, T: Target + Sync + Send + 'static> Debugger
    for GDBAsyncNotifier<C, T>
{
    fn on_start(&mut self) {
        //todo!()
    }

    fn on_stop(&mut self) {
        self.on_target_stop()
    }

    fn on_illegal_opcode(&mut self) {
        self.target_stop_signal(gdb::stub::StopReason::Signal(Signal::SIGILL));
    }
}

unsafe impl CpuExternalHandler for ExternalHandler {
    fn arithmetic_error(&mut self, cpu: &mut MipsCpu<Self>, _error_id: u32) {
        cpu.stop();
    }

    fn memory_error(&mut self, cpu: &mut MipsCpu<Self>, _error_id: u32) {cpu.stop();}

    fn invalid_opcode(&mut self, cpu: &mut MipsCpu<Self>) {
        if let Some(debugger) = &mut self.debugger {
            debugger.on_illegal_opcode();
        }
        cpu.stop();
    }

    fn system_call(&mut self, cpu: &mut MipsCpu<Self>, _call_id: u32) {cpu.stop();}

    fn system_call_error(
        &mut self,
        _cpu: &mut MipsCpu<Self>,
        _call_id: u32,
        _error_id: u32,
        _message: &str,
    ) {
    }

    fn cpu_start(&mut self) {}

    fn cpu_stop(&mut self) {
        if let Some(debugger) = &mut self.debugger {
            debugger.on_stop();
        }
    }
}


pub mod test{
    use gdb::DebugServer;
    use mips_emulator::cpu::{MipsCpu, DefaultExternalHandler};

    #[test]
    pub fn test(){

        let mut cpu =
            MipsCpu::<DefaultExternalHandler>::new_interface(DefaultExternalHandler::default());
    
            let debugger = DebugServer::new(cpu);
            let _ = debugger.start_debug_server();
        }
}