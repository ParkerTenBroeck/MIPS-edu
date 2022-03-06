use virtual_cpu::memory::Memory;

pub struct MipsCpu{
    pc: u32,
    reg: [u32; 32],
    lo: u32,
    hi: u32,
    running: bool,
    mem: Memory,
}


impl MipsCpu{
    fn new() -> Self{
        MipsCpu{
            pc: 0,
            reg: [0;32],
            lo: 0,
            hi: 0,
            running: false,
            mem: Memory::new(),
        }
    }

    fn start(&mut self){
        self.running = true;

        while{
            let op = self.mem.get_u32(self.pc);

            

            pc += 4;

            self.running //do while self.running
        }{}
    }

    fn stop(&mut self){
        self.running = false;
    }

    fn reset(&mut self){
        self.pc = 0;
        self.reg = [0;32];
        self.lo = 0;
        self.hi = 0;
    }
}