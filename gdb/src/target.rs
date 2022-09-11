use std::fmt::Debug;

pub trait Target {
    type Error: Debug;

    fn inturrupt(&mut self) -> Result<(), Self::Error>;
    fn step_at(&mut self, addr: Option<u32>);
    fn continue_at(&mut self, addr: Option<u32>);
    fn write_memory(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error>;
    fn read_memory(&mut self, addr: u32, len: u32) -> Result<Vec<u8>, Self::Error>;
    fn read_registers(&mut self) -> Result<[u32; 38], Self::Error>;
    fn read_register(&mut self, reg: u8) -> Result<u32, Self::Error>;
    fn write_register(&mut self, reg: u8, data: u32) -> Result<(), Self::Error>;
    fn write_registers(&mut self, data: [u32; 38]) -> Result<(), Self::Error>;

    fn sw_breakpoint_hit(&mut self);
    fn insert_software_breakpoint(&mut self, kind: u8, addr: u32) -> Result<(), Self::Error>;
    fn remove_software_breakpoint(&mut self, kind: u8, addr: u32) -> Result<(), Self::Error>;
}
