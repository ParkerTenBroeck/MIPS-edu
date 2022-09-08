use gdb::{async_target::GDBAsyncNotifier, connection::Connection, target::Target};
use mips_emulator::{
    cpu::EmulatorInterface,
    memory::{page_pool::{MemoryDefaultAccess}, single_cached_memory::SingleCachedMemory},
};


#[derive(Debug)]
pub enum TargetError {
    MemoryWriteError,
    MemoryReadError,
    InvalidRegister(u8),
}

pub struct TargetInterface {
    pub emulator: EmulatorInterface<super::handlers::ExternalHandler>,
}

impl std::fmt::Debug for TargetInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetInterface").finish()
    }
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
            let mut mem = cpu.get_mem::<SingleCachedMemory>();
            let mut vec = Vec::with_capacity(len as usize);
            for i in 0..len{
                vec.push(mem.get_u8_be(addr.wrapping_add(i)));
            }
            Ok(vec)
            // match cpu.raw_mem().slice_vec_or_none(
            //     addr,
            //     addr.checked_add(len).ok_or(TargetError::MemoryReadError)?,
            // ) {
            //     mips_emulator::memory::emulator_memory::TernaryOption::Option1(slice) => {
            //         Ok((*slice).to_owned())
            //     }
            //     mips_emulator::memory::emulator_memory::TernaryOption::Option2(vec) => Ok(vec),
            //     mips_emulator::memory::emulator_memory::TernaryOption::None => {
            //         Err(TargetError::MemoryReadError)
            //     }
            // }
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
                34 => self.emulator.hi(),
                35 => 0,
                36 => 0,
                37 => self.emulator.pc(),
                _ => Err(TargetError::InvalidRegister(reg))?,
            }
        })
    }

    fn write_register(&mut self, _reg: u8, _data: u32) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_registers(&mut self, _data: [u32; 38]) -> Result<(), Self::Error> {
        todo!()
    }
}

pub trait Debugger: Sync + Send + 'static {
    fn on_start(&self);
    fn on_stop(&self);
    fn on_illegal_opcode(&self);
}

impl<C: Connection + Sync + Send + 'static, T: Target + Sync + Send + 'static> Debugger
    for GDBAsyncNotifier<C, T>
{
    fn on_start(&self) {
        //todo!()
    }

    fn on_stop(&self) {
        self.on_target_stop()
    }

    fn on_illegal_opcode(&self) {
        self.on_stop();//self.target_stop_signal(gdb::stub::StopReason::Signal(Signal::SIGILL));
    }
}
