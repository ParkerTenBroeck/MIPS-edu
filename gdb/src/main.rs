use std::net::TcpListener;

use gdb::{
    async_target::{self, GDBAsyncNotifier},
    connection::Connection,
    signal::Signal,
    stub::GDBStub,
    target::Target,
};
use mips_emulator::{
    cpu::{CpuExternalHandler, EmulatorInterface, MipsCpu},
    memory::{page_pool::MemoryDefaultAccess, single_cached_memory::SingleCachedMemory},
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

impl std::fmt::Debug for TargetInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetInterface").finish()
    }
}

#[derive(Debug)]
pub enum TargetError {
    MemoryWriteError,
    MemoryReadError,
    InvalidRegister(u8),
}

impl Target for TargetInterface {
    type Error = TargetError;

    fn inturrupt(&mut self) -> Result<(), Self::Error> {
        self.emulator
            .stop()
            .map_err(|_| TargetError::MemoryWriteError)
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

    fn write_memory(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error> {
        self.emulator.cpu_mut(|cpu| {
            let mut mem = cpu.get_mem::<SingleCachedMemory>();
            for (index, byte) in data.iter().enumerate() {
                unsafe {
                    mem.set_u8_be(addr.wrapping_add(index as u32), *byte);
                }
            }
        });
        Ok(())
    }

    fn read_memory(&mut self, addr: u32, len: u32) -> Result<Vec<u8>, Self::Error> {
        self.emulator.cpu_mut(|cpu| unsafe {
            match cpu.raw_mem().slice_vec_or_none(
                addr,
                addr.checked_add(len).ok_or(TargetError::MemoryReadError)?,
            ) {
                mips_emulator::memory::emulator_memory::TernaryOption::Option1(slice) => {
                    Ok((*slice).to_owned())
                }
                mips_emulator::memory::emulator_memory::TernaryOption::Option2(vec) => Ok(vec),
                mips_emulator::memory::emulator_memory::TernaryOption::None => {
                    Err(TargetError::MemoryReadError)
                }
            }
        })
    }

    fn read_registers(&mut self) -> Result<[u32; 38], Self::Error> {
        let mut regs = [0u32; 38];
        unsafe {
            let cpu = &*self.emulator.raw_cpu();
            regs[0..32].copy_from_slice(cpu.reg());

            //regs[32] = (0x0); //sr
            regs[33] = cpu.hi();
            regs[34] = cpu.lo();
            //regs[35] = (0x0); //bad
            //regs[36] = (0x0); //cause
            regs[37] = cpu.pc();
        }
        Ok(regs)
    }

    fn read_register(&mut self, reg: u8) -> Result<u32, Self::Error> {
        Ok(unsafe {
            match reg {
                0..=31 => self.emulator.reg()[reg as usize],
                32 => 0,
                33 => self.emulator.lo(),
                34 => self.emulator.pc(),
                35 => 0,
                36 => 0,
                37 => self.emulator.pc(),
                _ => Err(TargetError::InvalidRegister(reg))?,
            }
        })
    }

    fn write_register(&mut self, reg: u8, data: u32) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_registers(&mut self, data: [u32; 38]) -> Result<(), Self::Error> {
        todo!()
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

    fn memory_error(&mut self, cpu: &mut MipsCpu<Self>, _error_id: u32) {
        cpu.stop();
    }

    fn invalid_opcode(&mut self, cpu: &mut MipsCpu<Self>) {
        if let Some(debugger) = &mut self.debugger {
            debugger.on_illegal_opcode();
        }
        cpu.stop();
    }

    fn system_call(&mut self, cpu: &mut MipsCpu<Self>, _call_id: u32) {
        cpu.stop();
    }

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

// pub mod test {
//     use gdb::DebugServer;
//     use mips_emulator::cpu::{DefaultExternalHandler, MipsCpu};

//     #[test]
//     pub fn test() {
//         let mut cpu =
//             MipsCpu::<DefaultExternalHandler>::new_interface(DefaultExternalHandler::default());

//         let debugger = DebugServer::new(cpu);
//         let _ = debugger.start_debug_server();
//     }
// }
