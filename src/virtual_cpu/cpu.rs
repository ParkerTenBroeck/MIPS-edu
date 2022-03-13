use virtual_cpu::memory::Memory;

pub struct MipsCpu{
    pub(crate) pc: u32,
    pub(crate) reg: [u32; 32],
    lo: u32,
    hi: u32,
    running: bool,
    finished: bool,
    pub mem: Memory,
}

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

    #[allow(unused)]
    pub fn get_general_registers(&self) -> &[u32; 32] { &self.reg }
    #[allow(unused)]
    pub fn get_hi_register(&self) -> u32 { self.hi }
    #[allow(unused)]
    pub fn get_lo_register(&self) -> u32 { self.lo }
    #[allow(unused)]
    pub fn get_pc(&self) -> u32 { self.pc }

    pub fn is_running(&self) -> bool{
        self.running | !self.finished
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

    pub fn clear(&mut self){
        self.reset();
        self.mem.unload_all_pages();
    }

    fn system_call(&mut self, call_id: u32){
        match call_id{
            0 => self.stop(),
            _ => {}
        }
    }

    fn arithmetic_error(&mut self){
        //clike::virtual_cpu::cpu::MipsCpu::start()
    }

    fn invalid_op_code(&mut self){

    }

    #[allow(arithmetic_overflow)]
    pub fn start(&mut self){
        if self.running || !self.finished {return;}

        //runs 2^16 * (2^15-1)*3+2 instructions (6442254338)
        //the version written in c++ seems to be around 17% faster
        //[0x64027FFFu32, 0x00000820, 0x20210001, 0x10220001, 0x0BFFFFFD, 0x68000000][(self.pc >> 2) as usize];//

        let test_prog = [0x64027FFFu32, 0x00000820, 0x20210001, 0x10220001, 0x0BFFFFFD, 0x68000000];//
        self.mem.copy_into_raw(0, &test_prog);


        self.running = true;
        self.finished = false;

        while{
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
                                (self.reg[register_s!(op)] as i32 + self.reg[register_t!((op))] as i32) as u32
                        }
                        #[allow(unreachable_patterns)]
                        0b100000 => { //ADDU
                            self.reg[register_d!(op)] =
                                self.reg[register_s!(op)] + self.reg[register_t!((op))]
                        }
                        0b100100 => { //AND
                            self.reg[register_d!(op)] =
                                self.reg[register_s!(op)] & self.reg[register_t!((op))]
                        }
                        0b011010 => { //DIV
                         let t = self.reg[register_d!(op)] as i32;
                            if t == 0{
                                let s = self.reg[register_s!(op)] as i32;
                                self.lo = (s / t) as u32;
                                self.hi = (s % t) as u32;
                            }else{
                                self.arithmetic_error();
                            }
                        }
                        0b011011 => { //DIVU
                            let t = self.reg[register_d!(op)];
                            if t == 0{
                                let s = self.reg[register_s!(op)];
                                self.lo = s / t;
                                self.hi = s % t;
                            }else{
                                self.arithmetic_error();
                            }

                        }
                        0b011000 => { //MULT
                            let t = self.reg[register_t!(op)] as i32 as i64;
                            let s = self.reg[register_s!(op)] as i32 as i64;
                            let result = t * s;
                            self.lo = (result & 0xFFFFFFFF) as u32;
                            self.hi = (result >> 32) as u32;
                        }
                        0b011001 => { //MULTU
                            let t = self.reg[register_t!(op)] as i32 as i64;
                            let s = self.reg[register_s!(op)] as i32 as i64;
                            let result = t * s;
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
                    self.reg[immediate_t!(op)] = self.mem.get_i16_alligned(
                        (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32) as u32
                }
                0b100101 => {//LHU
                    self.reg[immediate_t!(op)] = self.mem.get_u16_alligned(
                        (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32) as u32
                }
                0b100011 => {//LW
                    self.reg[immediate_t!(op)] = self.mem.get_u32_alligned(
                        (self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32) as u32
                }

                // store instructions
                0b101000 => {//SB
                    self.mem.set_u8((self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32, (self.reg[immediate_t!(op)] & 0xFF) as u8);
                }
                0b101001 => {//SH
                    self.mem.set_u16_alligned((self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32, (self.reg[immediate_t!(op)] & 0xFFFF) as u16);
                }
                0b101011 => {//SW
                    self.mem.set_u32_alligned((self.reg[immediate_s!(op)] as i32 + immediate_immediate_address!(op)) as u32, (self.reg[immediate_t!(op)]) as u32);
                }

            _ => self.invalid_op_code(),
            }

            self.running //do while self.running
        }{}
        self.finished = true;
    }
}