use gdb::{async_target::GDBAsyncNotifier, connection::Connection, target::Target};
use mips_emulator::{
    cpu::{CpuExternalHandler, EmulatorInterface},
    memory::{page_pool::MemoryDefaultAccess, single_cached_memory::SingleCachedMemory},
};

#[derive(Debug)]
pub enum TargetError {
    MemoryWriteError,
    MemoryReadError,
    InvalidRegister(u8),
    UnsupportedBreakpointKind,
    InvalidBreakpointAddress(u32),
    BreakpointDoesntExist(u32),
    BreakpointAlreadyExists,
}

struct Breakpoint {
    addr: u32,
    old_data: u32,
}

pub struct TargetInterface<T: CpuExternalHandler> {
    pub emulator: EmulatorInterface<T>,
    breakpoints: Vec<Breakpoint>,
}

impl<T: CpuExternalHandler> TargetInterface<T> {
    pub fn new(emulator: EmulatorInterface<T>) -> Self {
        Self {
            emulator,
            breakpoints: Default::default(),
        }
    }
}

impl<T: CpuExternalHandler> std::fmt::Debug for TargetInterface<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetInterface").finish()
    }
}

impl<T: CpuExternalHandler> Target for TargetInterface<T> {
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
            for i in 0..len {
                vec.push(mem.get_u8_be(addr.wrapping_add(i)));
            }
            Ok(vec)
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

    fn insert_software_breakpoint(&mut self, kind: u8, addr: u32) -> Result<(), Self::Error> {
        if kind == 4 {
            if self.breakpoints.iter().any(|val| val.addr == addr) {
                return Ok(()); //Err(TargetError::BreakpointAlreadyExists)
            }
            if addr & 0b11 == 0 {
                self.emulator.cpu_mut(|cpu| {
                    let mut mem = cpu.get_mem::<SingleCachedMemory>();
                    let data = unsafe { mem.get_u32_alligned_o_be(addr) };
                    if let Some(old_data) = data {
                        self.breakpoints.push(Breakpoint { addr, old_data });
                        unsafe {
                            mem.set_u32_alligned_be(addr, 0x00000000D);
                        }
                        Ok(())
                    } else {
                        Err(TargetError::InvalidBreakpointAddress(addr))
                    }
                })
            } else {
                Err(TargetError::InvalidBreakpointAddress(addr))
            }
        } else {
            Err(TargetError::UnsupportedBreakpointKind)
        }
    }

    fn remove_software_breakpoint(&mut self, kind: u8, addr: u32) -> Result<(), Self::Error> {
        if kind == 4 {
            if let Some(breakpoint) = self.breakpoints.iter().find(|val| val.addr == addr) {
                self.emulator.cpu_mut(|cpu| {
                    let mut mem = cpu.get_mem::<SingleCachedMemory>();
                    _ = unsafe { mem.set_u32_alligned_o_be(addr, breakpoint.old_data) };
                    Ok(())
                })
            } else {
                Err(TargetError::BreakpointDoesntExist(addr))
            }
        } else {
            Err(TargetError::UnsupportedBreakpointKind)
        }
    }

    fn sw_breakpoint_hit(&mut self) {
        self.emulator.cpu_mut(|cpu| {
            let bp_addr = cpu.pc().wrapping_sub(4);
            if self.breakpoints.iter().any(|val| val.addr == bp_addr) {
                //this is a debugger breakpoint so lets move the pc back one
                cpu.set_pc(bp_addr)
            }
        })
    }
}

pub trait Debugger<T: Target>: Sync + Send + 'static {
    fn on_start(&self);
    fn on_stop(&self);
    fn on_illegal_opcode(&self);
    fn on_software_breakpoint(&self);
}

impl<C: Connection + Sync + Send + 'static, T: Target + Sync + Send + 'static> Debugger<T>
    for GDBAsyncNotifier<C, T>
{
    fn on_start(&self) {
        //todo!()
    }

    fn on_stop(&self) {
        self.on_target_stop()
    }

    fn on_illegal_opcode(&self) {
        self.on_stop();
    }

    fn on_software_breakpoint(&self) {
        self.target_stop_signal(gdb::stub::StopReason::SwBreak);
    }
}
