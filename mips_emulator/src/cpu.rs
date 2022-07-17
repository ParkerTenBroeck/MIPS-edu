use core::panic;
use std::{sync::{atomic::AtomicUsize}, time::Duration, panic::AssertUnwindSafe};

use crate::memory::{page_pool::{ PagePoolRef, PagePoolListener, PagePoolController, MemoryDefaultAccess}, emulator_memory::Memory, single_cached_memory::SingleCachedMemory};



//macros
//jump encoding
macro_rules! jump_immediate_address {
    ($expr:expr) => {
        ((($expr as u32) & 0b00000011111111111111111111111111) << 2)
    };
}

#[allow(unused)]
macro_rules! jump_immediate_offset {
    ($expr:expr) => {
        (($expr as i32) << 6) >> 4
    };
}

//immediate encoding
macro_rules! immediate_immediate_signed_extended {
    ($expr:expr) => {
        ((($expr as i32) << 16) >> 16) as u32
    };
}
macro_rules! immediate_immediate_zero_extended {
    ($expr:expr) => {
        (($expr as u32) & 0xFFFF)
    };
}

macro_rules! immediate_immediate_address {
    ($expr:expr) => {
        (($expr as i32) << 16) >> 14
    };
}

#[allow(unused)]
macro_rules! immediate_immediate_unsigned_hi {
    ($expr:expr) => {
        (($expr as u32) << 16)
    };
}

macro_rules! immediate_s {
    ($expr:expr) => {
        ((($expr as u32) >> 21) & 0b11111) as usize
    };
}

macro_rules! immediate_t {
    ($expr:expr) => {
        ((($expr as u32) >> 16) & 0b11111) as usize
    };
}

macro_rules! register_s {
    ($expr:expr) => {
        ((($expr as u32) >> 21) & 0b11111) as usize
    };
}

macro_rules! register_t {
    ($expr:expr) => {
        ((($expr as u32) >> 16) & 0b11111) as usize
    };
}

macro_rules! register_d {
    ($expr:expr) => {
        ((($expr as u32) >> 11) & 0b11111) as usize
    };
}

macro_rules! register_a {
    ($expr:expr) => {
        (($expr as u32) >> 6) & 0b11111
    };
}


//Co processor macros
macro_rules! cop1_function {
    ($expr:expr) => {
        (($expr as u32)) & 0b111111
    };
}

macro_rules! cop1_fd {
    ($expr:expr) => {
        (($expr as u32) >> 6) & 0b11111
    };
}
macro_rules! cop1_fs {
    ($expr:expr) => {
        (($expr as u32) >> 11) & 0b11111
    };
}
macro_rules! cop1_ft {
    ($expr:expr) => {
        (($expr as u32) >> 16) & 0b11111
    };
}
macro_rules! cop1_fmt {
    ($expr:expr) => {
        (($expr as u32) >> 21) & 0b11111
    };
}


//Macros


//-------------------------------------------------------- co processors
pub struct CP0{
    registers: [u32; 32]
}

impl CP0{
    pub fn new() -> Self{
        CP0 {
            registers: [0; 32],
        }
    }
}
#[repr(C)]
pub union CP1Reg {
    pub single: [f32; 32],
    pub double: [f64; 16]
}

pub struct CP1{
    registers: CP1Reg
}

impl CP1{
    pub fn new() -> Self{
        CP1 { 
            registers: CP1Reg { 
                single: [0.0; 32] 
            }  
        }
    }
}

//-------------------------------------------------------- co processors
#[repr(align(4096))]
pub struct MipsCpu {
    pub pc: u32,
    pub reg: [u32; 32],
    pub cp0: CP0,
    pub cp1: CP1,
    pub lo: u32,
    pub hi: u32,
    i_check: bool,
    running: bool,
    finished: bool,
    is_paused: bool,
    is_within_memory_event: bool,
    instructions_ran: u64,
    paused: AtomicUsize,
    inturupts: Vec<()>,
    pub mem: PagePoolRef<Memory>,
    external_handler: Box<dyn CpuExternalHandler>,
}

pub trait CpuExternalHandler: Sync + Send {
    fn arithmetic_error(&mut self, cpu: &mut MipsCpu, error_id:  u32);
    fn memory_error(&mut self, cpu: &mut MipsCpu, error_id: u32);
    fn invalid_opcode(&mut self, cpu: &mut MipsCpu);
    fn system_call(&mut self, cpu: &mut MipsCpu, call_id: u32);
    fn system_call_error(&mut self, cpu: &mut MipsCpu, call_id: u32, error_id: u32, message:  &str);
}


struct DefaultExternalHandler{

}

impl DefaultExternalHandler{
    fn opcode_address(cpu: &mut MipsCpu) -> u32{
        cpu.pc.wrapping_sub(4)
    }

    fn opcode(cpu: &mut MipsCpu) -> u32{
        cpu.mem.get_u32_alligned(cpu.pc.wrapping_sub(4))
    }
}

impl Default for DefaultExternalHandler{
    fn default() -> Self {
        Self {  }
    }
}

impl CpuExternalHandler for DefaultExternalHandler {
    fn arithmetic_error(&mut self, cpu: &mut MipsCpu, error_id:  u32) {
        log::warn!("arithmetic error {}", error_id);
        cpu.stop();
    }

    fn memory_error(&mut self, cpu: &mut MipsCpu, error_id: u32) {
        log::warn!("Memory Error: {}", error_id);
        cpu.stop();
    }

    fn invalid_opcode(&mut self, cpu: &mut MipsCpu) {            
        log::warn!("invalid opcode {:#08X} at {:#08X}", Self::opcode(cpu), Self::opcode_address(cpu));
        cpu.stop();
    }

    fn system_call(&mut self, cpu: &mut MipsCpu, call_id: u32) {
        match call_id {
            0 => cpu.stop(),
            1 => log::info!("{}", cpu.reg[4] as i32),
            4 => {
                let _address = cpu.reg[4];
            }
            5 => {
                let mut string = String::new();
                let _ = std::io::stdin().read_line(&mut string);
                match string.parse::<i32>() {
                    Ok(val) => cpu.reg[2] = val as u32,
                    Err(_) => match string.parse::<u32>() {
                        Ok(val) => cpu.reg[2] = val,
                        Err(_) => {
                            cpu.system_call_error(
                                call_id,
                                0,
                                "unable to parse integer".into(),
                            );
                        }
                    },
                }
            }
            99 => {}
            101 => match char::from_u32(cpu.reg[4]) {
                Some(val) => log::info!("{}", val),
                None => log::warn!("Invalid char{}", cpu.reg[4]),
            },
            102 => {
                let mut string = String::new();
                let _ = std::io::stdin().read_line(&mut string);
                string = string.replace("\n", "");
                string = string.replace("\r", "");
                if string.len() != 1 {
                    cpu.reg[2] = string.chars().next().unwrap() as u32;
                } else {
                    cpu.system_call_error(call_id, 0, "invalid input");
                }
            }
            105 => {
                use std::thread;
                thread::sleep(Duration::from_millis(cpu.reg[4] as u64));
            }
            106 => {
                static mut LAST: u128 = 0;
                let time =
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                let dur = unsafe{time - LAST};
                cpu.reg[4] *= 2;
                if (cpu.reg[4]  as u128 ) >= dur{
                    std::thread::sleep(std::time::Duration::from_millis((cpu.reg[4] as u64) - (dur as u64)));
                    unsafe{
                        LAST =
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                    }
                }else{
                    unsafe{
                        LAST = time;
                    }
                }
                
            }
            107 => {
                cpu.reg[2] = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    & 0xFFFFFFFFu128) as u32;
            }
            130 => {
                cpu.reg[2] = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_micros()
                    & 0xFFFFFFFFu128) as u32;
            }
            111 => {
                cpu.stop();
            }
            _ => {
                self.system_call_error(cpu, call_id, 0, "invalid system call");
            }
        }
    }

    fn system_call_error(&mut self, _cpu: &mut MipsCpu, call_id: u32, error_id: u32, message:  &str) {
        log::warn!(
            "System Call: {} Error: {} Message: {}",
            call_id,
            error_id,
            message
        );
    }
}

impl PagePoolListener for MipsCpu{
    fn lock(&mut self, _initiator: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.pause_exclude_memory_event();
        Result::Ok(())
    }

    fn unlock(&mut self, _initiator: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.resume();
        Result::Ok(())
    }
}

impl Drop for MipsCpu{
    fn drop(&mut self) {
        self.stop();
        while self.is_running(){
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

impl MipsCpu {
    #[allow(unused)]
    pub fn new() -> Self {
        let mut tmp = MipsCpu {
            instructions_ran: 0,
            pc: 0,
            reg: [0; 32],
            cp0: CP0::new(),
            cp1: CP1::new(),
            lo: 0,
            hi: 0,
            i_check: !false,
            running: false,
            finished: true,
            paused: 0.into(),
            is_paused: true,
            is_within_memory_event: false,
            mem: Memory::new(),
            external_handler: Box::new(DefaultExternalHandler::default()),
            inturupts: Vec::new(),
        };

        tmp
    }

    unsafe fn into_listener(&mut self) -> &'static mut (dyn PagePoolListener + Sync + Send){
        let test: &mut dyn PagePoolListener = self;
        std::mem::transmute(test)
    }

    #[allow(unused)]
    pub fn get_general_registers(&self) -> &[u32; 32] {
        &self.reg
    }
    #[allow(unused)]
    pub fn get_reg(&self, reg: usize) -> u32 {
        self.reg[reg]
    }
    #[allow(unused)]
    pub fn get_hi_register(&self) -> u32 {
        self.hi
    }
    #[allow(unused)]
    pub fn get_lo_register(&self) -> u32 {
        self.lo
    }
    #[allow(unused)]
    pub fn get_pc(&self) -> u32 {
        self.pc
    }
    #[allow(unused)]
    pub fn get_instructions_ran(&self) -> u64 {
        self.instructions_ran
    }
    #[allow(unused)]
    pub fn get_general_registers_mut(&mut self) -> &mut [u32; 32] {
        &mut self.reg
    }
    #[allow(unused)]
    pub fn get_hi_register_mut(&mut self) -> &mut u32 {
        &mut self.hi
    }
    #[allow(unused)]
    pub fn get_lo_register_mut(&mut self) -> &mut u32 {
        &mut self.lo
    }
    #[allow(unused)]
    pub fn get_pc_mut(&mut self) -> &mut u32 {
        &mut self.pc
    }
    #[allow(unused)]
    pub fn get_mem(&mut self) -> PagePoolRef<SingleCachedMemory>{
        //&mut self.mem
        self.get_mem_controller().lock().unwrap().add_holder(SingleCachedMemory::new())
    }
    #[allow(unused)]
    pub fn get_mem_controller(&self) -> std::sync::Arc<std::sync::Mutex<PagePoolController>>{
        match &self.mem.page_pool{
            Some(val) => {
                val.clone_page_pool_mutex()
            },
            None => panic!(),
        }
    }
    pub fn set_external_handlers(&mut self, handlers: impl CpuExternalHandler + 'static){
        self.external_handler = Box::new(handlers);
    }

    #[allow(unused)]
    pub fn is_running(&self) -> bool {
        //this is a hack(dont come for me :))
        *crate::black_box(&self.running) || !*crate::black_box(&self.finished)
    }

    pub fn paused_or_stopped(&self) -> bool{
        self.is_paused() || !self.is_running()
    }

    pub fn is_paused(&self) -> bool {
        //this is a hack(dont come for me :))
        *crate::black_box(&self.is_paused)
    }

    fn is_within_memory_event(&self) -> bool {
        *crate::black_box(&self.is_within_memory_event)
    }

    #[allow(unused)]
    pub fn stop(&mut self) {
        self.running = false;
        self.i_check = !true;
    }

    pub fn stop_and_wait(&mut self) {
        self.stop();
        while {
            self.is_running()
        }{
            //std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[allow(unused)]
    pub fn reset(&mut self) {
        self.pc = 0;
        self.reg = [0; 32];
        self.lo = 0;
        self.hi = 0;
        self.instructions_ran = 0;
    }

    #[allow(unused)]
    pub fn clear(&mut self) {
        self.reset();
        self.mem.unload_all_pages();
    }

    #[allow(unused)]
    pub fn pause(&mut self) {
        self.paused
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        while {
            //println!("CPU {:p}", self);
            self.i_check = !true;
            !self.is_paused()
        }{
            //std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[allow(unused)]
    pub fn pause_exclude_memory_event(&mut self) {
        self.paused
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        while {
            //println!("CPU {:p}", self);
            self.i_check = !true;
            !(self.is_paused() || self.is_within_memory_event())
        }{
            //std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[allow(unused)]
    pub fn resume(&mut self) {
        if self.paused.load(std::sync::atomic::Ordering::Relaxed) > 0{
            self.paused
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    #[inline(always)]
    fn get_handler(&mut self) -> &'static mut dyn CpuExternalHandler {
        let thing = self.external_handler.as_mut();
        unsafe{std::mem::transmute(thing)}
    }

    fn run_panic(&mut self){
        self.running = false;
        self.finished = true;
        self.i_check = !false;
        self.paused.store(0, std::sync::atomic::Ordering::Relaxed);
        self.is_paused = true;
        self.clear();
    }

    #[inline(always)]
    #[allow(unused)]
    fn system_call_error(&mut self, call_id: u32, error_id: u32, message: &str) {
        self.get_handler().system_call_error(self, call_id, error_id, message);
    }

    #[inline(never)]
    fn memory_error(&mut self, error_id: u32) {
        self.get_handler().memory_error(self, error_id);
    }

    #[inline(never)]
    fn arithmetic_error(&mut self, id: u32) {
        self.get_handler().arithmetic_error(self, id);
    }

    #[inline(never)]
    fn invalid_op_code(&mut self) {
        self.get_handler().invalid_opcode(self);
    }

    #[inline(never)]
    fn system_call(&mut self, call_id: u32) {
        self.get_handler().system_call(self, call_id);
    }

    #[allow(dead_code)]
    pub fn start_new_thread(&'static mut self) {
        if self.running || !self.finished {
            return;
        }

        self.running = true;
        self.finished = false;

        println!("{:p}", self);


        //stack overflows occur so we need to make a custom thread with a larger stack to account for that and it fixes the problem
        let _handle = std::thread::Builder::new().stack_size(32 * 1024 * 1024).spawn(move || {
            // some work here
            log::info!("CPU Started");
            let start = std::time::SystemTime::now();

            let result = std::panic::catch_unwind(AssertUnwindSafe(||{
                self.run();
            }));

            let since_the_epoch = std::time::SystemTime::now()
                .duration_since(start)
                .expect("Time went backwards");
            log::info!("{:?}", since_the_epoch);
            log::info!("CPU stopping");

            match result{
                Ok(_) => {
                },
                Err(err) => {
                    self.run_panic();
                    log::error!("{:?}", err);
                    //std::panic::resume_unwind(err);
                },
            }
        
        });
    }

    #[allow(dead_code)]
    pub fn step_new_thread(&'static mut self) {
        if self.running || !self.finished {
            return;
        }
        self.running = false;
        self.finished = false;
        self.i_check = !true;

        let _handle = std::thread::Builder::new().stack_size(32 * 1024 * 1024).spawn(|| {
            log::info!("CPU Step Started");
            let start = std::time::SystemTime::now();
            
            let result = std::panic::catch_unwind(AssertUnwindSafe(||{
                self.run();
            }));


            let since_the_epoch = std::time::SystemTime::now()
                .duration_since(start)
                .expect("Time went backwards");
            log::info!("{:?}", since_the_epoch);
            log::info!("CPU Step Stopping");
            
            match result{
                Ok(_) => {},
                Err(err) => {
                    self.run_panic();
                    //std::panic::resume_unwind(err);
                },
            }
        });
    }

    #[allow(dead_code)]
    pub fn start_local(&mut self) {
        if self.running || !self.finished {
            return;
        }

        self.running = true;
        self.finished = false;
        self.i_check = !true;

        log::info!("CPU Started");
        //let start = std::time::SystemTime::now();
        self.run();
        //let since_the_epoch = std::time::SystemTime::now()
        //    .duration_since(start)
        //    .expect("Time went backwards");
        //log::info!("{:?}", since_the_epoch);
        log::info!("CPU Step Stopping");
    }

    #[allow(dead_code)]
    pub fn step_local(&'static mut self) {
        if self.running || !self.finished {
            return;
        }
        self.running = false;
        self.finished = false;
        self.i_check = !true;
        
        log::info!("CPU Step Started");
        
        self.run();

        log::info!("CPU Step Stopped");
    }

    #[inline(never)]
    fn run(&mut self) {

        //let result = std::panic::catch_unwind(||{
        //TODO ensure that the memory isnt currently locked beforehand
        
        let listener = unsafe{self.into_listener()};
        self.mem.add_listener(listener);
        self.mem.add_thing(unsafe{std::mem::transmute(&mut self.is_within_memory_event)});



        self.is_paused = false;
        'main_loop: while {
            while *self.paused.get_mut() > 0 {
                self.is_paused = true;
                if !self.running {
                    break 'main_loop;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            self.is_paused = false;


            while {
                let op =
                self.mem.get_u32_alligned(self.pc);
                self.instructions_ran += 1;
                
                //prevent overflow
                self.pc = self.pc.wrapping_add(4);
                
                self.run_opcode(op);

                self.i_check
            }{}
            self.i_check = !false;

            //TODO clear paused state and check states when CPU stops running
            self.running //do while self.running
        } {}
        self.finished = true;
        self.mem.remove_listener();
        self.mem.remove_thing();
    }

    #[inline(always)]
    fn run_opcode(&mut self, op: u32){
        
        macro_rules! set_reg {
            ($reg:expr, $val:expr) => {
                unsafe {
                    *self.reg.get_unchecked_mut($reg) = $val;
                }
            };
        }
        macro_rules! get_reg {
            ($reg:expr) => {
                unsafe { *self.reg.get_unchecked($reg) }
            };
        }

        match op >> 26 {
            0 => {
                match op & 0b111111 {
                    // REGISTER formatted instructions

                    //arithmatic
                    0b100000 => {
                        //ADD
                        
                        match (self.reg[register_s!(op)] as i32).checked_add(self.reg[register_t!((op))] as i32)
                        {
                            Some(val) => {
                                self.reg[register_d!(op)] = val as u32;
                            },
                            None => {
                                self.arithmetic_error(2);
                            },
                        }
                    }
                    0b100001 => {
                        //ADDU
                        self.reg[register_d!(op)] = self.reg[register_s!(op)]
                            .wrapping_add(self.reg[register_t!((op))])
                    }
                    0b100100 => {
                        //AND
                        self.reg[register_d!(op)] =
                            self.reg[register_s!(op)] & self.reg[register_t!((op))]
                    }
                    0b011010 => {
                        //DIV
                        let t = self.reg[register_t!(op)] as i32;
                        if t != 0 {
                            let s = self.reg[register_s!(op)] as i32;
                            self.lo = (s.wrapping_div(t)) as u32;
                            self.hi = (s.wrapping_rem(t)) as u32;
                        } else {
                            //self.arithmetic_error(0);
                        }
                    }
                    0b011011 => {
                        //DIVU
                        let t = self.reg[register_t!(op)];
                        if t != 0 {
                            let s = self.reg[register_s!(op)];
                            self.lo = s.wrapping_div(t);
                            self.hi = s.wrapping_rem(t);
                        } else {
                            //self.arithmetic_error(0);
                        }
                    }
                    0b011000 => {
                        //MULT
                        let t = self.reg[register_t!(op)] as i32 as i64;
                        let s = self.reg[register_s!(op)] as i32 as i64;
                        let result = t.wrapping_mul(s);
                        self.lo = (result & 0xFFFFFFFF) as u32;
                        self.hi = (result >> 32) as u32;
                    }
                    0b011001 => {
                        //MULTU
                        let t = self.reg[register_t!(op)] as u64;
                        let s = self.reg[register_s!(op)] as u64;
                        let result = t.wrapping_mul(s);
                        self.lo = (result & 0xFFFFFFFF) as u32;
                        self.hi = (result >> 32) as u32;
                    }
                    0b100111 => {
                        //NOR
                        self.reg[register_d!(op)] =
                            !(self.reg[register_s!(op)] | self.reg[register_t!(op)]);
                    }
                    0b100101 => {
                        //OR
                        self.reg[register_d!(op)] =
                            self.reg[register_s!(op)] | self.reg[register_t!(op)];
                    }
                    0b100110 => {
                        //XOR
                        self.reg[register_d!(op)] =
                            self.reg[register_s!(op)] ^ self.reg[register_t!(op)];
                    }
                    0b000000 => {
                        //SLL
                        self.reg[register_d!(op)] =
                            self.reg[register_t!(op)] << register_a!(op);
                    }
                    0b000100 => {
                        //SLLV
                        self.reg[register_d!(op)] =
                            (self.reg[register_t!(op)]) << ( 0b11111 & self.reg[register_s!(op)]);
                    }
                    0b000011 => {
                        //SRA
                        self.reg[register_d!(op)] =
                            ((self.reg[register_t!(op)] as i32 >> register_a!(op))) as u32;
                    }
                    0b000111 => {
                        //SRAV
                        self.reg[register_d!(op)] = 
                            (self.reg[register_t!(op)] as i32 >>  ( 0b11111 & self.reg[register_s!(op)]))
                            as u32;
                    }
                    0b000010 => {
                        //SRL
                        self.reg[register_d!(op)] =
                            (self.reg[register_t!(op)] >> register_a!(op)) as u32;
                    }
                    0b000110 => {
                        //SRLV
                        self.reg[register_d!(op)] =
                            (self.reg[register_t!(op)] >> ( 0b11111 & self.reg[register_s!(op)])) as u32;
                    }
                    0b100010 => {
                        //SUB
                        self.reg[register_d!(op)] = (self.reg[register_s!(op)] as i32
                            - self.reg[register_t!(op)] as i32)
                            as u32;
                    }
                    0b100011 => {
                        //SUBU
                        self.reg[register_d!(op)] =
                            self.reg[register_s!(op)].wrapping_sub(self.reg[register_t!(op)]);
                    }

                    //comparason
                    0b101010 => {
                        //SLT
                        self.reg[register_d!(op)] = {
                            if (self.reg[register_s!(op)] as i32)
                                < (self.reg[register_t!(op)] as i32)
                            {
                                1
                            } else {
                                0
                            }
                        }
                    }
                    0b101011 => {
                        //SLTU
                        self.reg[register_d!(op)] = {
                            if self.reg[register_s!(op)] < self.reg[register_t!(op)] {
                                1
                            } else {
                                0
                            }
                        }
                    }

                    //jump
                    0b001001 => {
                        //JALR
                        self.reg[31] = self.pc;
                        self.pc = self.reg[register_s!(op)];
                    }
                    0b001000 => {
                        //JR
                        self.pc = self.reg[register_s!(op)];
                    }

                    //data movement
                    0b010000 => {
                        //MFHI
                        self.reg[register_d!(op)] = self.hi;
                    }
                    0b010010 => {
                        //MFLO
                        self.reg[register_d!(op)] = self.lo;
                    }
                    0b010001 => {
                        //MTHI
                        self.hi = self.reg[register_s!(op)];
                    }
                    0b010011 => {
                        //MTLO
                        self.lo = self.reg[register_s!(op)];
                    }

                    //special
                    0b001100 => {
                        //syscall
                        self.system_call((op >> 6) & 0b11111111111111111111)
                    }
                    0b001101 => {
                        //break
                        self.system_call((op >> 6) & 0b11111111111111111111)
                    }
                    0b110100 => {
                        //TEQ
                        if self.reg[register_s!(op)] == self.reg[register_t!(op)]{
                            self.system_call((op >> 6) & 0b1111111111)
                        }
                    }
                    0b110000 => {
                        //TGE
                        if self.reg[register_s!(op)] as i32 >= self.reg[register_t!(op)] as i32{
                            self.system_call((op >> 6) & 0b1111111111)
                        }
                    }
                    0b110001 => {
                        //TGEU
                        if self.reg[register_s!(op)] >= self.reg[register_t!(op)]{
                            self.system_call((op >> 6) & 0b1111111111)
                        }
                    }
                    0b110010 => {
                        //TIT
                        if (self.reg[register_s!(op)] as i32) < self.reg[register_t!(op)] as i32{
                            self.system_call((op >> 6) & 0b1111111111)
                        }
                    }
                    0b110011 => {
                        //TITU
                        if self.reg[register_s!(op)] < self.reg[register_t!(op)]{
                            self.system_call((op >> 6) & 0b1111111111)
                        }
                    }
                    0b110110 => {
                        //TNE
                        if self.reg[register_s!(op)] != self.reg[register_t!(op)]{
                            self.system_call((op >> 6) & 0b1111111111)
                        }
                    }

                    _ => self.invalid_op_code(),
                }
            }
            //Jump instructions
            0b000010 => {
                //jump
                self.pc = (self.pc & 0b11110000000000000000000000000000) | jump_immediate_address!(op);
            }
            0b000011 => {
                //jal
                self.reg[31] = self.pc;
                self.pc = (self.pc & 0b11110000000000000000000000000000) | jump_immediate_address!(op);
            }
            // IMMEDIATE formmated instructions

            // arthmetic
            0b001000 => {
                //ADDI
                self.reg[immediate_t!(op)] = (self.reg[immediate_s!(op)] as i32
                    + immediate_immediate_signed_extended!(op) as i32)
                    as u32;
            }
            0b001001 => {
                //ADDIU
                self.reg[immediate_t!(op)] = (self.reg[immediate_s!(op)] as u32).wrapping_add(immediate_immediate_signed_extended!(op) as u32);
            }
            0b001100 => {
                //ANDI
                self.reg[immediate_t!(op)] = self.reg[immediate_s!(op)]
                    & immediate_immediate_zero_extended!(op)
            }
            0b001101 => {
                //ORI
                self.reg[immediate_t!(op)] = self.reg[immediate_s!(op)] as u32
                    | immediate_immediate_zero_extended!(op) as u32
            }
            0b001110 => {
                //XORI
                self.reg[immediate_t!(op)] = self.reg[immediate_s!(op)] as u32
                    ^ immediate_immediate_zero_extended!(op) as u32
            }

            // constant manupulating inctructions
            0b001111 => {
                //LUI
                self.reg[immediate_t!(op)] = immediate_immediate_zero_extended!(op) << 16; 
            }
            // these were replaced
            // 0b011001 => {
            //     //LHI
            //     let t = immediate_t!(op) as usize;
            //     self.reg[t] =
            //         self.reg[t] & 0xFFFF | immediate_immediate_unsigned_hi!(op) as u32;
            // }
            // 0b011000 => {
            //     //LLO
            //     let t = immediate_t!(op) as usize;
            //     self.reg[t] =
            //         self.reg[t] & 0xFFFF0000 | immediate_immediate_zero_extended!(op) as u32;
            // }

            // comparison Instructions
            0b001010 => {
                //SLTI
                self.reg[immediate_t!(op)] = {
                    if (self.reg[immediate_s!(op)] as i32)
                        < (immediate_immediate_signed_extended!(op) as i32)
                    {
                        1
                    } else {
                        0
                    }
                }
            }
            0b001011 => {
                //SLTIU
                self.reg[immediate_t!(op)] = {
                    if (self.reg[immediate_s!(op)] as u32)
                        < (immediate_immediate_signed_extended!(op) as u32)
                    {
                        1
                    } else {
                        0
                    }
                }
            }

            // branch instructions
            0b000100 => {
                //BEQ
                if get_reg!(immediate_s!(op)) == get_reg!(immediate_t!(op)) {
                    self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                }
            }
            0b000001 => {
                match immediate_t!(op){
                    0b00001 => {
                        //BGEZ
                        if (self.reg[immediate_s!(op)] as i32) >= 0 {
                            self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                        }
                    }
                    0b00000 => {
                        //BLTZ
                        if (self.reg[immediate_s!(op)] as i32) < 0 {
                            self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                        }
                    }
                    _ => {
                        self.invalid_op_code();
                    }
                }
            }
            0b000111 => {
                //BGTZ
                if self.reg[immediate_s!(op)] as i32 > 0 {
                    self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                }
            }

            0b000110 => {
                //BLEZ
                if self.reg[immediate_s!(op)] as i32 <= 0 {
                    self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                }
            }
            0b000101 => {
                //BNE
                if self.reg[immediate_s!(op)] != self.reg[immediate_t!(op) as usize] {
                    self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                }
            }

            //load unsinged instructions
            0b100010 => {
                //LWL
                let address = (self.reg[immediate_s!(op)] as i32 + immediate_immediate_signed_extended!(op) as i32) as u32;
                let reg_num = immediate_t!(op);
                let mut thing:[u8; 4] = unsafe{core::mem::transmute(self.reg[reg_num])};
                thing[3] = self.mem.get_u8(address);
                thing[2] = self.mem.get_u8(address + 1);
                self.reg[reg_num] = unsafe{core::mem::transmute(thing)};
            }
            0b100110 => {
                //LWR
                let address = (self.reg[immediate_s!(op)] as i32 + immediate_immediate_signed_extended!(op) as i32) as u32;
                let reg_num = immediate_t!(op);
                let mut thing:[u8; 4] = unsafe{core::mem::transmute(self.reg[reg_num])};
                thing[0] = self.mem.get_u8(address);
                thing[1] = self.mem.get_u8(address.wrapping_sub(1));
                self.reg[reg_num] = unsafe{core::mem::transmute(thing)};
            }


            //save unaliged instructions
            0b101010 => {
                //SWL
                let address = (self.reg[immediate_s!(op)] as i32 + immediate_immediate_signed_extended!(op) as i32) as u32;
                let reg_num = immediate_t!(op);
                let thing:[u8; 4] = unsafe{core::mem::transmute(self.reg[reg_num])};
                self.mem.set_u8(address, thing[3]);
                self.mem.set_u8(address + 1, thing[2]);
            }
            0b101110 => {
                //SWR
                let address = (self.reg[immediate_s!(op)] as i32 + immediate_immediate_signed_extended!(op) as i32) as u32;
                let reg_num = immediate_t!(op);
                let thing:[u8; 4] = unsafe{core::mem::transmute(self.reg[reg_num])};
                self.mem.set_u8(address, thing[0]);
                self.mem.set_u8(address.wrapping_sub(1), thing[1]);
            }

            // load instrictions
            0b100000 => {
                //LB
                self.reg[immediate_t!(op)] = self.mem.get_i8(
                    (self.reg[immediate_s!(op)] as i32 + immediate_immediate_signed_extended!(op) as i32)
                        as u32,
                ) as u32
            }
            0b100100 => {
                //LBU
                self.reg[immediate_t!(op)] = self.mem.get_u8(
                    (self.reg[immediate_s!(op)] as i32 + immediate_immediate_signed_extended!(op) as i32)
                        as u32,
                ) as u32
            }
            0b100001 => {
                //LH
                let address = (self.reg[immediate_s!(op)] as i32
                    + immediate_immediate_signed_extended!(op) as i32)
                    as u32;

                #[cfg(feature = "memory_allignment_check")]
                if address & 0b1 == 0 {
                    self.reg[immediate_t!(op)] = self.mem.get_i16_alligned(address) as u32
                } else {
                    self.memory_error(0);
                }
                #[cfg(not(feature = "memory_allignment_check"))]
                {
                    self.reg[immediate_t!(op)] = self.mem.get_i16_alligned(address) as u32
                }
            }
            0b100101 => {
                //LHU
                let address = (self.reg[immediate_s!(op)] as i32
                    + immediate_immediate_signed_extended!(op) as i32)
                    as u32;

                #[cfg(feature = "memory_allignment_check")]
                if address & 0b1 == 0 {
                    self.reg[immediate_t!(op)] = self.mem.get_u16_alligned(address) as u32
                } else {
                    self.memory_error(0);
                }
                #[cfg(not(feature = "memory_allignment_check"))]
                {
                    self.reg[immediate_t!(op)] = self.mem.get_u16_alligned(address) as u32
                }
            }
            0b100011 => {
                //LW
                let address = (self.reg[immediate_s!(op)] as i32
                    + immediate_immediate_signed_extended!(op) as i32)
                    as u32;

                #[cfg(feature = "memory_allignment_check")]
                if address & 0b11 == 0 {
                    self.reg[immediate_t!(op)] = self.mem.get_u32_alligned(address) as u32
                } else {
                    self.memory_error(1);
                }
                #[cfg(not(feature = "memory_allignment_check"))]
                {
                    self.reg[immediate_t!(op)] = self.mem.get_u32_alligned(address) as u32
                }
            }

            // store instructions
            0b101000 => {
                //SB
                self.mem.set_u8(
                    (self.reg[immediate_s!(op)] as i32 + immediate_immediate_signed_extended!(op) as i32)
                        as u32,
                    (self.reg[immediate_t!(op)] & 0xFF) as u8,
                );
            }
            0b101001 => {
                //SH
                let address = (self.reg[immediate_s!(op)] as i32
                    + immediate_immediate_signed_extended!(op) as i32)
                    as u32;

                if address & 0b1 == 0 {
                    self.mem.set_u16_alligned(
                        address,
                        (self.reg[immediate_t!(op)] & 0xFFFF) as u16,
                    );
                } else {
                    self.memory_error(3);
                }
                #[cfg(not(feature = "memory_allignment_check"))]
                {
                    self.mem.set_u16_alligned(
                        address,
                        (self.reg[immediate_t!(op)] & 0xFFFF) as u16,
                    );
                }
            }
            0b101011 => {
                //SW
                let address = (self.reg[immediate_s!(op)] as i32
                    + immediate_immediate_signed_extended!(op) as i32)
                    as u32;
                if address & 0b11 == 0 {
                    self.mem
                        .set_u32_alligned(address, (self.reg[immediate_t!(op)]) as u32);
                } else {
                    self.memory_error(4);
                }
                #[cfg(not(feature = "memory_allignment_check"))]
                {
                    self.mem
                        .set_u32_alligned(address, (self.reg[immediate_t!(op)]) as u32);
                }
            }

            _ => self.invalid_op_code(),
        }
    }
}