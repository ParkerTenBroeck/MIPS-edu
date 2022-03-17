use std::time::Duration;

use crate::memory::Memory;

//macros
//jump encoding
macro_rules! jump_immediate_address{
    ($expr:expr) => {
        ($expr as u32) & 0b00000011111111111111111111111111
    }
}

macro_rules! jump_immediate_offset{
    ($expr:expr) => {
        (($expr as i32) << 6) >> 4
    }
}

//immediate encoding
macro_rules! immediate_immediate{
    ($expr:expr) => {
        (($expr as i32) << 16) >> 16
    }
}
macro_rules! immediate_immediate_address{
    ($expr:expr) => {
        (($expr as i32) << 16) >> 14
    }
}

macro_rules! immediate_immediate_unsigned{
    ($expr:expr) => {
        (($expr as u32) & 0xFFFF)
    }
}

macro_rules! immediate_immediate_unsigned_hi{
    ($expr:expr) => {
        (($expr as u32) << 16)
    }
}

macro_rules! immediate_s{
    ($expr:expr) => {
        ((($expr as u32) >> 21) & 0b11111) as usize
    }
}

macro_rules! immediate_t{
    ($expr:expr) => {
        ((($expr as u32) >> 16) & 0b11111) as usize
    }
}

macro_rules! register_s{
    ($expr:expr) => {
        ((($expr as u32) >> 21) & 0b11111) as usize
    }
}

macro_rules! register_t{
    ($expr:expr) => {
        ((($expr as u32) >> 16) & 0b11111) as usize
    }
}

macro_rules! register_d{
    ($expr:expr) => {
        ((($expr as u32) >> 11) & 0b11111) as usize
    }
}

macro_rules! register_a{
    ($expr:expr) => {
        (($expr as u32) >> 6) & 0b11111
    }
}
//Macros

#[derive(Default)]
pub struct MipsCpu{
    pub(crate) pc: u32,
    pub(crate) reg: [u32; 32],
    pub(crate) lo: u32,
    pub(crate) hi: u32,
    running: bool,
    finished: bool,
    paused: bool,
    is_paused: bool,
    pub(crate) mem: Memory,
    #[cfg(feature = "external_handlers")]
    external_handler: CpuExternalHandler,
}

#[cfg(feature = "external_handlers")]
pub struct CpuExternalHandler{
    pub arithmetic_error: fn (&mut MipsCpu, u32),
    pub memory_error: fn (&mut MipsCpu, u32),
    pub invalid_opcode: fn (&mut MipsCpu),
    pub system_call: fn (&mut MipsCpu, u32),
    pub system_call_error: fn (&mut MipsCpu, u32, u32, &str),
}

#[cfg(feature = "external_handlers")]
impl Default for CpuExternalHandler{
    fn default() -> Self {
        fn f1(_a1: &mut MipsCpu, _a2: u32){
            panic!("Unimplemented External Handler for CPU");
        }
        fn f2(_a1: &mut MipsCpu){
            panic!("Unimplemented External Handler for CPU");
        }
        fn f3(_a1: &mut MipsCpu, _a2: u32, _a3: u32, _a4: &str){
            panic!("Unimplemented External Handler for CPU");
        }

        Self {
            arithmetic_error: f1,
            memory_error: f1,
            system_call: f1,
            invalid_opcode: f2,
            system_call_error: f3,
        }
    }
}




impl MipsCpu{
    #[allow(unused)]
    pub fn new() -> Self{
        MipsCpu{
            pc: 0,
            reg: [0;32],
            lo: 0,
            hi: 0,
            running: false,
            finished: true,
            paused: false,
            is_paused: true,
            mem: Memory::new(),
            ..Default::default()
        }
    }

    #[allow(unused)]
    pub fn get_general_registers(&self) -> &[u32; 32] { &self.reg }
    #[allow(unused)]
    pub fn get_hi_register(&self) -> u32 { self.hi }
    #[allow(unused)]
    pub fn get_lo_register(&self) -> u32 { self.lo }
    #[allow(unused)]
    pub fn get_pc(&self) -> u32 { self.pc }
    #[allow(unused)]
    pub fn get_mem(&self) -> &Memory { 
        &self.mem
     }

    #[allow(unused)]
    pub fn get_general_registers_mut(&mut self) -> &mut [u32; 32] { &mut self.reg }
    #[allow(unused)]
    pub fn get_hi_register_mut(&mut self) -> &mut u32 { &mut self.hi }
    #[allow(unused)]
    pub fn get_lo_register_mut(&mut self) -> &mut u32 { &mut self.lo }
    #[allow(unused)]
    pub fn get_pc_mut(&mut self) -> &mut u32 { &mut self.pc }
    #[allow(unused)]
    pub fn get_mem_mut(&mut self) -> &mut Memory { &mut self.mem }

    #[allow(unused)]
    pub fn is_running(&self) -> bool{
        self.running | !self.finished
    }

    #[allow(unused)]
    pub fn is_paused(&self) -> bool{
        self.is_paused
    }

    #[allow(unused)]
    pub fn stop(&mut self){
        self.running = false;
    }

    #[allow(unused)]
    pub fn reset(&mut self){
        self.pc = 0;
        self.reg = [0;32];
        self.lo = 0;
        self.hi = 0;
    }

    #[allow(unused)]
    pub fn clear(&mut self){
        self.reset();
        self.mem.unload_all_pages();
    }

    #[allow(unused)]
    pub fn pause(&mut self){
        self.paused = true;
        while !self.is_paused{
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        use std::sync::{Arc, Mutex, Condvar};
use std::thread;

let pair = Arc::new((Mutex::new(false), Condvar::new()));
let pair2 = Arc::clone(&pair);

// Inside of our lock, spawn a new thread, and then wait for it to start.
thread::spawn(move|| {
    let (lock, cvar) = &*pair2;
    let mut started = lock.lock().unwrap();
    *started = true;
    // We notify the condvar that the value has changed.
    cvar.notify_one();
});

// Wait for the thread to start up.
let (lock, cvar) = &*pair;
let mut started = lock.lock().unwrap();
while !*started {
    started = cvar.wait(started).unwrap();
}
    }

    #[allow(unused)]
    pub fn resume(&mut self){
        self.paused = false;
    }

    
    #[inline(always)]
    #[allow(unused)]
    fn system_call_error(&mut self, call_id: u32, error_id: u32, message: &str){
        #[cfg(feature = "external_handlers")]
        {
            (self.external_handler.system_call_error)(self, call_id, error_id, message);
        }
        #[cfg(not(feature = "external_handlers"))]
        {
            println!("System Call: {} Error: {} Message: {}", call_id, error_id, message);
        }
    }

    #[inline(always)]
    fn memory_error(&mut self, error_id: u32){
        #[cfg(feature = "external_handlers")]
        {
            (self.external_handler.memory_error)(self, error_id);
        }
        #[cfg(not(feature = "external_handlers"))]
        {
            println!("Memory Error: {}", error_id);
        }
    }

    #[inline(always)]
    fn system_call(&mut self, call_id: u32){
        #[cfg(feature = "external_handlers")]
        {
            (self.external_handler.system_call)(self, call_id);
        }
        #[cfg(not(feature = "external_handlers"))]
        {
            match call_id{
                0 => self.stop(),
                1 => println!("{}", self.reg[4] as i32),
                4 => {
                    let _address = self.reg[4];
                },
                5 => {
                    let mut string = String::new();
                    let _ = std::io::stdin().read_line(&mut string);
                    match string.parse::<i32>() {
                        Ok(val) => self.reg[2] = val as u32,
                        Err(_) => {
                            match string.parse::<u32>() {
                                Ok(val) => self.reg[2] = val,
                                Err(_) => {
                                    self.system_call_error(call_id, 0, "unable to parse integer".into());
                                },
                            }
                        },
                    }
                },
                99 => {
                    
                },
                101 => {
                    match char::from_u32(self.reg[4]){
                        Some(val) => println!("{}", val),
                        None => println!("Invalid char{}", self.reg[4]),
                    }
                }
                102 => {
                    let mut string = String::new();
                    let _ = std::io::stdin().read_line(&mut string);
                    string = string.replace("\n", "");
                    string = string.replace("\r", "");
                    if string.len() != 1{
                        self.reg[2] = string.chars().next().unwrap() as u32;
                    }else{
                        self.system_call_error(call_id, 0, "invalid input");
                    }
                },
                105 => {
                    use std::{thread};
                    thread::sleep(Duration::from_millis(self.reg[4] as u64));
                },
                106 => {
                    static mut LAST:std::time::SystemTime = std::time::UNIX_EPOCH;
                    let time = unsafe{
                        std::time::SystemTime::now()
                        .duration_since(LAST).unwrap().as_millis()
                    };
                    if time < self.reg[4] as u128 {
                        let time = self.reg[4] as u128 - time;
                        std::thread::sleep(std::time::Duration::from_millis(time as u64));
                    }
                }
                107 => {
                    self.reg[2] = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap().as_millis() & 0xFFFFFFFFu128) as u32; 
                },
                108 => {
                    self.reg[2] = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap().as_micros() & 0xFFFFFFFFu128) as u32; 
                }
                111 => {
                    self.stop();
                }
                _ => {}
            }
        }
    }

    #[inline(always)]
    fn arithmetic_error(&mut self, id:u32){
        #[cfg(feature = "external_handlers")]
        {
            (self.external_handler.arithmetic_error)(self, id);
        }
        #[cfg(not(feature = "external_handlers"))]
        {
            println!("arithmetic error {}", id);
        } 
    }

    #[inline(always)]
    fn invalid_op_code(&mut self){
        #[cfg(feature = "external_handlers")]
        {
            (self.external_handler.invalid_opcode)(self);
        }
        #[cfg(not(feature = "external_handlers"))]
        {

        }
    }

    
    #[allow(dead_code)]
    pub fn start_new_thread(&'static mut self){
        if self.running || !self.finished {return;}

        self.running = true;
        self.finished = false;

        let _handle = std::thread::spawn(|| {
            // some work here
            println!("CPU Started");
            let start = std::time::SystemTime::now();
            self.run();
            let since_the_epoch = std::time::SystemTime::now()
                .duration_since(start)
                .expect("Time went backwards");
            println!("{:?}", since_the_epoch);
            println!("CPU stopping");
        });
    }

    #[allow(dead_code)]
    pub fn step_new_thread(&'static mut self){
        if self.running || !self.finished {return;}
        self.running = false;
        self.finished = false;

        let _handle = std::thread::spawn(|| {
            // some work here
            println!("CPU Step Started");
            let start = std::time::SystemTime::now();
            self.run();
            let since_the_epoch = std::time::SystemTime::now()
                .duration_since(start)
                .expect("Time went backwards");
            println!("{:?}", since_the_epoch);
            println!("CPU Step Stopping");
        });
    }

    #[allow(dead_code)]
    pub fn start_local(&mut self){
        if self.running || !self.finished {return;}
        
        self.running = true;
        self.finished = false;

        self.run();
    }
    
    #[allow(arithmetic_overflow)]
    fn run(&mut self){
    
        while{

            while self.paused{
                self.is_paused = true;
                std::thread::sleep(Duration::from_millis(1));
            }
            self.is_paused = false;


            let op = self.mem.get_u32_alligned(self.pc);//test_prog[(self.pc >> 2) as usize];//

            //if self.reg[1] & 0xFFFFFF == 0{
            //    println!("{:?}", self.reg[1]);
            //}
            //prevent overflow
            self.pc = self.pc.wrapping_add(4);

            match op >> 26{
                0 => {
                    match op & 0b111111{
                        // REGISTER formatted instructions

                        //arithmatic
                        0b100000 => { //ADD
                            self.reg[register_d!(op)] =
                                ((self.reg[register_s!(op)] as i32).wrapping_add(self.reg[register_t!((op))] as i32)) as u32
                        }
                        #[allow(unreachable_patterns)]
                        0b100000 => { //ADDU
                            self.reg[register_d!(op)] =
                                self.reg[register_s!(op)].wrapping_add(self.reg[register_t!((op))]) 
                        }
                        0b100100 => { //AND
                            self.reg[register_d!(op)] =
                                self.reg[register_s!(op)] & self.reg[register_t!((op))]
                        }
                        0b011010 => { //DIV
                         let t = self.reg[register_d!(op)] as i32;
                            if t == 0{
                                let s = self.reg[register_s!(op)] as i32;
                                self.lo = (s.wrapping_div(t)) as u32;
                                self.hi = (s.wrapping_rem(t)) as u32;
                            }else{
                                self.arithmetic_error(0);
                            }
                        }
                        0b011011 => { //DIVU
                            let t = self.reg[register_d!(op)];
                            if t == 0{
                                let s = self.reg[register_s!(op)];
                                self.lo = s.wrapping_div(t);
                                self.hi = s.wrapping_rem(t);
                            }else{
                                self.arithmetic_error(0);
                            }

                        }
                        0b011000 => { //MULT
                            let t = self.reg[register_t!(op)] as i32 as i64;
                            let s = self.reg[register_s!(op)] as i32 as i64;
                            let result = t.wrapping_mul(s);
                            self.lo = (result & 0xFFFFFFFF) as u32;
                            self.hi = (result >> 32) as u32;
                        }
                        0b011001 => { //MULTU
                            let t = self.reg[register_t!(op)] as u64;
                            let s = self.reg[register_s!(op)] as u64;
                            let result = t.wrapping_mul(s);
                            self.lo = (result & 0xFFFFFFFF) as u32;
                            self.hi = (result >> 32) as u32;
                        }
                        0b100111 => { //NOR
                        self.reg[register_d!(op)] =
                            (!(self.reg[register_s!(op)])) | self.reg[register_t!(op)];
                        }
                        0b100101 => { //OR
                            self.reg[register_d!(op)] =
                                self.reg[register_s!(op)] | self.reg[register_t!(op)];
                        }
                        0b100110 => { //XOR
                            self.reg[register_d!(op)] =
                                self.reg[register_s!(op)] ^ self.reg[register_t!(op)];
                        }
                        0b000000 => { //SLL
                            self.reg[register_d!(op)] =
                                self.reg[register_t!(op)] << register_a!(op);
                        }
                        0b000100 => { //SLLV
                            self.reg[register_d!(op)] =
                                self.reg[register_t!(op)] << self.reg[register_s!(op)];
                        }
                        0b000011 => { //SRA
                            self.reg[register_d!(op)] =
                                (self.reg[register_t!(op)] as i32 >> register_a!(op)) as u32;
                        }
                        0b000111 => { //SRAV
                            self.reg[register_d!(op)] =
                                (self.reg[register_t!(op)] as i32 >> self.reg[register_s!(op)]) as u32;
                        }
                        0b000010 => { //SRL
                            self.reg[register_d!(op)] =
                                self.reg[register_t!(op)] >> register_a!(op);
                        }
                        0b000110 => { //SRLV
                            self.reg[register_d!(op)] =
                                self.reg[register_t!(op)] >> self.reg[register_s!(op)];
                        }
                        0b100010 => { //SUB
                            self.reg[register_d!(op)] =
                                (self.reg[register_s!(op)] as i32 - self.reg[register_t!(op)] as i32) as u32;
                        }
                        0b100011 => { //SUBU
                            self.reg[register_d!(op)] =
                                self.reg[register_s!(op)] - self.reg[register_t!(op)];
                        }

                        //comparason
                        0b101010 => { //SLT
                            self.reg[register_d!(op)] = {
                                if (self.reg[register_s!(op)] as i32) < (self.reg[register_t!(op)] as i32){
                                    1
                                }else{
                                    0
                                }
                            }
                        }
                        0b101001 => { //SLTU
                            self.reg[register_d!(op)] = {
                                if self.reg[register_s!(op)] < self.reg[register_t!(op)] {
                                    1
                                } else {
                                    0
                                }
                            }
                        }

                        //jump
                        0b001001 => { //JALR
                            self.reg[31] = self.pc;
                            self.pc = self.reg[register_s!(op)];
                        }
                        0b001000 => { //JR
                            self.pc = self.reg[register_s!(op)];
                        }

                        //data movement
                        0b010000 => { //MFHI
                            self.reg[register_d!(op)] = self.hi;
                        }
                        0b010010 => { //MFLO
                            self.reg[register_d!(op)] = self.lo;
                        }
                        0b010001 => { //MTHI
                            self.hi = self.reg[register_s!(op)];
                        }
                        0b010011 => { //MTLO
                            self.lo = self.reg[register_s!(op)];
                        }
                        _ => self.invalid_op_code(),
                    }
                }
                //Jump formatted instruction
                0b000010 => {//jump
                    self.pc = (self.pc as i32 + jump_immediate_offset!(op)) as u32;
                }
                0b000011 => {//jal
                    self.reg[31] = self.pc;
                    self.pc = (self.pc as i32 + jump_immediate_offset!(op)) as u32;
                }
                0b011010 => {//trap
                    self.system_call(jump_immediate_address!(op));
                }
                // IMMEDIATE formmated instructions

                // arthmetic
                0b001000 => {//trap ADDI
                    self.reg[immediate_t!(op)] =
                        (self.reg[immediate_s!(op)] as i32 + immediate_immediate!(op) as i32) as u32;
                }
                0b001001 => {//trap ADDIU
                    self.reg[immediate_t!(op)] =
                        self.reg[immediate_s!(op)] as u32 + immediate_immediate_unsigned!(op) as u32;
                }
                0b001100 => {//trap ANDI
                    self.reg[immediate_t!(op)] =
                        self.reg[immediate_s!(op)] as u32 & immediate_immediate_unsigned!(op) as u32
                }
                0b001101 => {//trap ORI
                    self.reg[immediate_t!(op)] =
                        self.reg[immediate_s!(op)] as u32 | immediate_immediate_unsigned!(op) as u32
                }
                0b001110 => {//trap XORI
                    self.reg[immediate_t!(op)] =
                        self.reg[immediate_s!(op)] as u32 ^ immediate_immediate_unsigned!(op) as u32
                }
                // constant manupulating inctructions
                0b011001 => {//LHI
                    let t = immediate_t!(op) as usize;
                    self.reg[t] =
                        self.reg[t] & 0xFFFF | immediate_immediate_unsigned_hi!(op) as u32;
                }
                0b011000 => {//LLO
                    let t = immediate_t!(op) as usize;
                    self.reg[t] =
                        self.reg[t] & 0xFFFF0000 | immediate_immediate_unsigned!(op) as u32;
                }

                // comparison Instructions
                0b001010 => {//SLTI
                    self.reg[immediate_t!(op)] = {
                        if (self.reg[immediate_s!(op)] as i32) < (immediate_immediate!(op) as i32){
                           1
                        }else{
                            0
                        }
                    }
                }
                #[allow(unreachable_patterns)]
                0b001001 => {//SLTIU
                    self.reg[immediate_t!(op)] = {
                         if (self.reg[immediate_s!(op) as usize] as u32) < (immediate_immediate_unsigned!(op) as u32){
                            1
                         }else{
                             0
                         }
                     }
                }

                // branch instructions
                0b000100 => {//BEQ
                    if self.reg[immediate_s!(op)] == self.reg[immediate_t!(op)] {
                        self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                    }
                }
                0b000111 => {//BGTZ
                    if self.reg[immediate_s!(op)] > 0 {
                        self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                    }
                }
                0b000110 => {//BLEZ
                    if self.reg[immediate_s!(op)] <= 0 {
                        self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                    }
                }
                0b000101 => {//BNE
                    if self.reg[immediate_s!(op)] != self.reg[immediate_t!(op) as usize] {
                        self.pc = (self.pc as i32 + immediate_immediate_address!(op)) as u32;
                    }
                }

                // load instrictions
                0b100000 => {//LB
                    self.reg[immediate_t!(op)] = self.mem.get_i8(
                        (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32) as u32
                }
                0b100100 => {//LBU
                    
                    self.reg[immediate_t!(op)] = self.mem.get_u8(
                        (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32) as u32
                }
                0b100001 => {//LH
                    let address = (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32;
                    
                    #[cfg(feature = "memory_allignment_check")]
                    if address & 0b1 == 0{
                        self.reg[immediate_t!(op)] = self.mem.get_i16_alligned(address) as u32
                    }else{
                        self.memory_error(0);
                    }
                    #[cfg(not(feature = "memory_allignment_check"))]
                    {
                        self.reg[immediate_t!(op)] = self.mem.get_i16_alligned(address) as u32
                    }
                }
                0b100101 => {//LHU
                    let address = (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32;

                    #[cfg(feature = "memory_allignment_check")]
                    if address & 0b1 == 0{
                        self.reg[immediate_t!(op)] = self.mem.get_u16_alligned(address) as u32
                    }else{
                        self.memory_error(0);
                    }
                    #[cfg(not(feature = "memory_allignment_check"))]
                    {
                        self.reg[immediate_t!(op)] = self.mem.get_u16_alligned(address) as u32
                    }
                }
                0b100011 => {//LW
                    let address = (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32;

                    #[cfg(feature = "memory_allignment_check")]
                    if address & 0b11 == 0{
                        self.reg[immediate_t!(op)] = self.mem.get_u32_alligned(address) as u32
                    }else{
                        self.memory_error(1);
                    }
                    #[cfg(not(feature = "memory_allignment_check"))]
                    {
                        self.reg[immediate_t!(op)] = self.mem.get_u32_alligned(address) as u32
                    }
                }

                // store instructions
                0b101000 => {//SB
                    self.mem.set_u8((self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32, (self.reg[immediate_t!(op)] & 0xFF) as u8);
                }
                0b101001 => {//SH
                    let address = (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32;
                    
                    if address & 0b11 == 0{
                        self.mem.set_u16_alligned(address, (self.reg[immediate_t!(op)] & 0xFFFF) as u16);
                    }else{
                        self.memory_error(3);
                    }
                    #[cfg(not(feature = "memory_allignment_check"))]
                    {
                        self.mem.set_u16_alligned(address, (self.reg[immediate_t!(op)] & 0xFFFF) as u16);
                    }
                }
                0b101011 => {//SW
                    let address = (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32;
                    if address & 0b11 == 0{
                        self.mem.set_u32_alligned(address, (self.reg[immediate_t!(op)]) as u32);
                    }else{
                        self.memory_error(3);
                    }
                    #[cfg(not(feature = "memory_allignment_check"))]
                    {
                        self.mem.set_u32_alligned(address, (self.reg[immediate_t!(op)]) as u32);
                    }
                }

            _ => self.invalid_op_code(),
            }

            self.running //do while self.running
        }{}
        self.finished = true;
    }
}