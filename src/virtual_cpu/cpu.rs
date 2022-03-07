use virtual_cpu::memory::Memory;

pub struct MipsCpu{
    pc: u32,
    reg: [u32; 32],
    lo: u32,
    hi: u32,
    running: bool,
    finished: bool,
    pub mem: Memory,
}

macro_rules! jump_immediate_address{
    ($expr:expr) => {
        $expr & 0b00000011111111111111111111111111
    }
}

macro_rules! jump_immediate_offset{
    ($expr:expr) => {
        (($expr as i32) << 6) >> 4
    }
}

impl MipsCpu{
    pub fn new() -> Self{
        MipsCpu{
            pc: 0,
            reg: [0;32],
            lo: 0,
            hi: 0,
            running: false,
            finished: true,
            mem: Memory::new(),
            
        }
    }

    pub fn stop(&mut self){
        self.running = false;
    }

    pub fn reset(&mut self){
        self.pc = 0;
        self.reg = [0;32];
        self.lo = 0;
        self.hi = 0;
    }

    #[allow(arithmetic_overflow)]
    pub fn start(&mut self){
        if self.running || !self.finished {return;}

        self.running = true;
        self.finished = false;

        while{
            let op = self.mem.get_u32(self.pc);

            //prevent overflow
            self.pc = self.pc.wrapping_add(4);

            match op >> 26{
                0 => {
                    match op & 0b111111{
                        _ => {}
                    }
                }
                //Jump formatted instruction
                0b000010 => {//jump
                    self.pc = (self.pc as i32 + jump_immediate_offset!(op)) as u32;
                }
                0b000011 => {//jal

                }
                0b011010 => {//trap

                }
                _ => {

                }
            }
            if self.pc == 0 {
                self.stop();
            }
            self.running //do while self.running
        }{}
        self.finished = true;
    }
}