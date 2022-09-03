use mips_emulator::cpu::{CpuExternalHandler, EmulatorInterface};
use mips_emulator::memory::page_pool::MemoryDefaultAccess;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
    thread,
};

pub type DebugServerHandle = Arc<RwLock<InnerDebugServerHandle>>;

#[derive(Default)]
pub struct InnerDebugServerHandle {}

pub struct DebugServer<T: CpuExternalHandler> {
    emulator: EmulatorInterface<T>,
    no_ack_mode: bool,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
enum PacketIn {
    ContinueAt(Option<u32>),
    ReadRegisters,
    WriteRegisters(),
    ReadMemory(u32, u32),
    WriteMemory(u32, Vec<u8>),
    ReadRegister(u8),
    WriteRegister(u8, u32),
    Reset,
    StepAt(Option<u32>),
    MustReplayEmpty,
    ExceptionReason,
    Unreconized,
    Kill,
    //general querry commands
    qSupported(Vec<(String, bool, Option<String>)>),
    qTStatus,
    qfThreadInfo,
    qsThreadInfo,
    qC,
    qAttached,
    qOffsets,
    qHostInfo,
    qProcessInfo,
    qRegisterInfo(u8),
    qMemoryRegionInfo,

    SelectExecutionThread(u8),
    SelectRegisterThread(u8),
    SelectMemoryThread(u8),

    QStartNoAckMode,
}

struct PacketOut {
    pack: Option<Result<PacketOutSucsess, PacketOutError>>,
}
impl Debug for PacketOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.pack {
            Some(Ok(ok)) => f.debug_struct("PacketOut").field("pack", ok).finish(),
            Some(Err(err)) => f
                .debug_struct("PacketOut - err")
                .field("err_id", &err.err_id)
                .finish(),
            None => f.debug_struct("PacketOut - none").finish(),
        }
    }
}

impl PacketOut {
    const EMPTY: PacketOut = PacketOut {
        pack: Some(Ok(PacketOutSucsess::Empty)),
    };
    const OK: PacketOut = PacketOut {
        pack: Some(Ok(PacketOutSucsess::Ok)),
    };

    #[allow(unused)]
    const NO_REPLY: PacketOut = PacketOut { pack: None };

    fn new(ok: PacketOutSucsess) -> Self {
        Self { pack: Some(Ok(ok)) }
    }

    fn err(id: u8) -> Self {
        Self {
            pack: Some(Err(PacketOutError { err_id: id })),
        }
    }

    fn ok_string(str: impl Into<String>) -> Self {
        Self {
            pack: Some(Ok(PacketOutSucsess::OkString(str.into()))),
        }
    }

    pub fn generate_packet_data(self) -> String {
        let mut msg = match self.pack {
            Some(Ok(ok)) => match ok {
                PacketOutSucsess::Ok => "OK".into(),
                PacketOutSucsess::OkString(string) => string,
                PacketOutSucsess::Empty => "".into(),
                PacketOutSucsess::ReadRegisters(regs) => {
                    let mut msg = String::new();
                    for reg in regs {
                        msg += format!("{:08X}", reg).as_str();
                    }
                    msg
                }
                PacketOutSucsess::ExceptionReason(reason) => stopped(reason),
                PacketOutSucsess::ReadMemory(vec) => {
                    let mut msg = String::new();
                    for byte in vec {
                        msg += format!("{:02X}", byte).as_str();
                    }
                    msg
                }
                PacketOutSucsess::ReadRegister(reg) => format!("{:08X}", reg),
            },
            Some(Err(err)) => error(err.err_id),
            None => return "".into(),
        };

        let mut check_sum = 0u8;
        for byte in msg.as_str().as_bytes() {
            check_sum = check_sum.wrapping_add(*byte);
        }

        msg.push('#');
        msg.insert(0, '$');

        let str = format!("{:02X}", check_sum);
        msg.push_str(str.as_str());

        msg
    }
}

#[derive(Debug)]
enum PacketOutSucsess {
    Ok,
    OkString(String),
    Empty,
    ReadRegisters([u32; 39]),
    ReadRegister(u32),
    ExceptionReason(u8),
    ReadMemory(Vec<u8>),
}

struct PacketOutError {
    err_id: u8,
}

impl<T: CpuExternalHandler> DebugServer<T> {
    pub fn new(emulator: EmulatorInterface<T>) -> Self {
        Self {
            emulator,
            no_ack_mode: false,
        }
    }

    pub fn start_debug_server(mut self) -> DebugServerHandle {
        let handle = DebugServerHandle::default();

        let j = thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
            println!("listening started, ready to accept");
            for stream in listener.incoming() {
                println!("accepted");
                let stream = stream.unwrap();
                if !self.handle_client(stream) {
                    break;
                }
            }
        });
        _ = j.join();

        handle
    }

    fn handle_client(&mut self, mut stream: TcpStream) -> bool {
        if !self.no_ack_mode {
            println!("->:+");
            if let Err(err) = stream.write(b"+") {
                println!("Error sending packet: {}", err);
                return false;
            }
        }

        let mut data = [0; 2000];
        while match stream.read(&mut data) {
            Ok(size) => {
                let mut str = std::str::from_utf8(&data[..size]).unwrap();

                if !str.is_empty() {
                    println!();
                }

                while !str.is_empty() {
                    let len = if str.starts_with('+'){
                        println!("<-:+");
                        println!("ack");
                        1
                    }else if str.starts_with('-'){
                        println!("<-:-");
                        println!("- ack");
                        1
                    }else if str.starts_with('\u{3}'){
                        println!("<-:\u{3}");
                        println!("Cntr + c (break)");
                        _ = self.emulator.stop();
                        1
                    }else if let Some(index) = str.find('#') && str.starts_with('$'){
                        println!("<-:{:#?}", &str[0..(index + 3)]);

                        let end_index = index + 3;
                        let message = &str[1..index];
                        let got_check_sum = u8::from_str_radix(&str[(index+1)..end_index], 16).unwrap();

                        let mut check_sum = 0u8;
                        for byte in message.as_bytes(){
                            check_sum = check_sum.wrapping_add(*byte);
                        }

                        if check_sum == got_check_sum{
                            let packet_in = Self::parse_incomming_packet(message);

                            if matches!(packet_in, PacketIn::Kill){
                                return false;
                            }

                            let response = self.generate_response_packet(packet_in);
                            if !response.is_empty(){
                                if let Err(err) = stream.write(response.as_bytes()){
                                    println!("Error sending packet: {}", err);
                                    return false;
                                }
                            }
                        }else{
                            println!("checksum doesnt match");
                            if let Err(err) = stream.write(PacketOut::err(0).generate_packet_data().as_bytes()){
                                println!("Error sending packet: {}", err);
                                return false;
                            }
                            break;
                        }
                        end_index
                    }else{
                        println!("<-:{:#?}", str);
                        println!("unreconized input: {:#?}", str);
                        break;
                    };
                    str = &str[len..];
                }
                true
            }
            Err(_) => {
                println!(
                    "An error occurred, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } {}
        true
    }

    fn parse_incomming_packet(command: &str) -> PacketIn {
        macro_rules! create_thing {
            ($command:ident $s:literal => $b:block, $($tt:tt)*) => {
                if $command == $s{
                    $b
                }else{
                    create_thing!($command $($tt)*)
                }
            };
            ($command:ident $s:literal = $a:ident => $b:block, $($tt:tt)*) => {
                if let Some($a) = $command.strip_prefix($s) {
                    $b
                }else{
                    create_thing!($command $($tt)*)
                }
            };

            ($command:ident $s:literal => $b:expr, $($tt:tt)*) => {
                if $command == $s{
                    $b
                }else{
                    create_thing!($command $($tt)*)
                }
            };
            ($command:ident $s:literal = $a:ident => $b:expr, $($tt:tt)*) => {
                if let Some($a) = $command.strip_prefix($s) {
                    $b
                }else{
                    create_thing!($command $($tt)*)
                }
            };


            ($command:ident $s:literal => $b:block) => {
                if $command == $s{
                    $b
                }else{
                    println!("unreconized input: {:#?}: {}", command, command.len());
                    PacketIn::Unreconized
                }
            };
            ($command:ident $s:literal = $a:ident => $b:block) => {
                if let Some($a) = $command.strip_prefix($s) {
                    $b
                }else{
                    println!("unreconized input: {:#?}: {}", command, command.len());
                    PacketIn::Unreconized
                }
            };
            ($command:ident $s:literal => $b:expr) => {
                if $command == $s{
                    $b
                }else{
                    println!("unreconized input: {:#?}: {}", command, command.len());
                    PacketIn::Unreconized
                }
            };
            ($command:ident $s:literal = $a:ident => $b:expr) => {
                if let Some($a) = $command.strip_prefix($s) {
                    $b
                }else{
                    println!("unreconized input: {:#?}: {}", command, command.len());
                    PacketIn::Unreconized
                }
            };
        }

        create_thing!(command
            "?" => PacketIn::ExceptionReason,

            "g" => PacketIn::ReadRegisters,
            'G' = _args => {panic!()},
            'p' = args => u8::from_str_radix(args, 16).map_or(PacketIn::Unreconized, PacketIn::ReadRegister),

            'm' = args => {
                let mut iter = args.split(',');
                if let (Some(add), Some(len)) = (iter.next(), iter.next())
                    && let (Ok(add), Ok(len)) = (u32::from_str_radix(add, 16), u32::from_str_radix(len, 16)){
                        PacketIn::ReadMemory(add, len)
                    }else{
                        PacketIn::Unreconized
                    }
            },
            'M' = args => {
                let mut iter = args.split([',', ':']);
                if let (Some(add), Some(len), Some(dat)) = (iter.next(), iter.next(), iter.next())
                    && let (Ok(addr), Ok(len)) = (u32::from_str_radix(add, 16), u32::from_str_radix(len, 16)){
                        if dat.len() != len as usize * 2{
                            PacketIn::Unreconized
                        }else{
                            let mut data = Vec::with_capacity(len as usize);
                            let mut iter = dat.chars();
                            while let (Some(un), Some(ln)) = (iter.next(), iter.next()){
                                data.push((un.to_digit(16).unwrap() << 4 | ln.to_digit(16).unwrap()) as u8)
                            }
                            PacketIn::WriteMemory(addr, data)
                        }
                    }else{
                        PacketIn::Unreconized
                    }
            },

            "r" => PacketIn::Reset,
            "k" => PacketIn::Kill,

            'c' = arg => PacketIn::ContinueAt(u32::from_str_radix(arg, 16).ok()),
            's' = arg => PacketIn::StepAt(u32::from_str_radix(arg, 16).ok()),

            "vMustReplyEmpty" => PacketIn::MustReplayEmpty,

            "Hc" = arg => u8::from_str_radix(arg, 16).map_or(PacketIn::Unreconized, PacketIn::SelectExecutionThread),
            "Hg" = arg => u8::from_str_radix(arg, 16).map_or(PacketIn::Unreconized, PacketIn::SelectRegisterThread),
            "Hm" = arg => u8::from_str_radix(arg, 16).map_or(PacketIn::Unreconized, PacketIn::SelectMemoryThread),

            "qSupported:" = raw_args => {
                let mut args: Vec<(String, bool, Option<String>)> = Vec::new();
                for arg in raw_args.split(';'){
                    if let Some(arg) = arg.strip_suffix('+'){
                        args.push((arg.into(), true, None));
                    }else if let Some(arg) = arg.strip_suffix('-'){
                        args.push((arg.into(), false, None));
                    }else if arg.contains('='){
                        if let Some((key, val)) = arg.split_once('='){
                            args.push((key.into(), true, Some(val.into())))
                        }
                    }
                }
                PacketIn::qSupported(args)
            },
            "qTStatus" => PacketIn::qTStatus,
            "qfThreadInfo" => PacketIn::qfThreadInfo,
            "qsThreadInfo" => PacketIn::qsThreadInfo,
            "qC" => PacketIn::qC,
            "qAttached" => PacketIn::qAttached,
            "qOffsets" => PacketIn::qOffsets,
            "qHostInfo" => PacketIn::qHostInfo,
            "qProcessInfo" => PacketIn::qProcessInfo,
            "qRegisterInfo" = arg => u8::from_str_radix(arg, 16).map_or(PacketIn::Unreconized, PacketIn::qRegisterInfo),
            "qMemoryRegionInfo" = _arg => PacketIn::qMemoryRegionInfo,

            "QStartNoAckMode" => PacketIn::QStartNoAckMode
        )
    }

    fn generate_response_packet(&mut self, packet_in: PacketIn) -> String {
        if !matches!(packet_in, PacketIn::Unreconized) {
            println!("packet: {:#?}", packet_in);
        }

        let packet_out = match packet_in {
            PacketIn::ContinueAt(address) => {
                if let Some(address) = address {
                    self.emulator.cpu_mut(|cpu| {
                        cpu.set_pc(address);
                    });
                }
                if self.emulator.start_new_thread().is_ok() {
                    PacketOut::OK
                } else {
                    PacketOut::err(0)
                }
            }
            PacketIn::StepAt(address) => {
                if let Some(address) = address {
                    self.emulator.cpu_mut(|cpu| {
                        cpu.set_pc(address);
                    });
                }
                if self.emulator.step_new_thread().is_ok() {
                    PacketOut::OK
                } else {
                    PacketOut::err(0)
                }
            }
            PacketIn::ReadRegisters => {
                PacketOut::new(PacketOutSucsess::ReadRegisters(self.read_registers()))
            }
            PacketIn::ReadRegister(reg) => PacketOut::new(PacketOutSucsess::ReadRegister(unsafe {
                match reg {
                    0..=31 => self.emulator.reg()[reg as usize],
                    32 => 0,
                    33 => self.emulator.lo(),
                    34 => self.emulator.pc(),
                    35 => 0,
                    36 => 0,
                    37 => self.emulator.pc(),
                    _ => panic!(),
                }
            })),

            PacketIn::WriteRegister(_, _) => todo!(),
            PacketIn::WriteRegisters() => todo!(),

            PacketIn::WriteMemory(addr, data) => self.write_to_mem(addr, &data),
            PacketIn::ReadMemory(address, length) => self
                .read_memory(address, length)
                .map_or(PacketOut::err(0), |vec| {
                    PacketOut::new(PacketOutSucsess::ReadMemory(vec))
                }),

            PacketIn::Reset => todo!(),
            PacketIn::ExceptionReason => PacketOut::new(PacketOutSucsess::ExceptionReason(5)),

            PacketIn::Unreconized => PacketOut::EMPTY,
            PacketIn::MustReplayEmpty => PacketOut::EMPTY,

            PacketIn::Kill => panic!("should have returned before this point!!"),

            PacketIn::qSupported(_args) => PacketOut::ok_string("QStartNoAckMode+"),
            PacketIn::qTStatus => PacketOut::EMPTY,
            PacketIn::qfThreadInfo => PacketOut::ok_string(thread_ids(&[0x11])),
            PacketIn::qsThreadInfo => PacketOut::ok_string("l"),
            PacketIn::qC => PacketOut::ok_string(current_thread_id(0x11)),
            PacketIn::qAttached => PacketOut::EMPTY,
            PacketIn::qOffsets => PacketOut::EMPTY,
            PacketIn::qHostInfo => PacketOut::ok_string(
                "triple:6D6970732D756E6B6E6F776E2D6C696E75782D676E75;endian:big;ptrsize:4;",
            ),
            PacketIn::qProcessInfo => PacketOut::ok_string("pid:1;endian:big;"),
            PacketIn::qRegisterInfo(index) => REGISTER_INFO
                .get(index as usize)
                .map_or(PacketOut::err(1), |ok| PacketOut::ok_string(*ok)),
            PacketIn::qMemoryRegionInfo => {
                PacketOut::ok_string("start:0;size:FFFFFFFF;permissions:rwx;")
            }
            PacketIn::QStartNoAckMode => {
                self.no_ack_mode = true;
                PacketOut::OK
            }
            PacketIn::SelectExecutionThread(_) => PacketOut::OK,
            PacketIn::SelectRegisterThread(_) => PacketOut::OK,
            PacketIn::SelectMemoryThread(_) => PacketOut::OK,
        };

        println!("response: {:#?}", packet_out);

        let response_data = packet_out.generate_packet_data();
        println!("->:{:#?}", response_data);
        response_data
    }

    fn read_registers(&mut self) -> [u32; 39] {
        let mut regs = [0u32; 39];
        unsafe {
            let cpu = &*self.emulator.raw_cpu();
            regs[0..32].copy_from_slice(cpu.reg());

            regs[32] = cpu.hi();
            regs[33] = cpu.lo();
            //regs[33] = (0x0); //bad
            //regs[34] = (0x0); //cause
            regs[36] = cpu.pc();
            //regs[37] = (0x0); //fcsr
            //regs[38] = (0x0); //fir
        }
        regs
    }

    fn read_memory(&mut self, addr: u32, len: u32) -> Option<Vec<u8>> {
        self.emulator.cpu_mut(|cpu| unsafe {
            match cpu
                .raw_mem()
                .slice_vec_or_none(addr, addr.checked_add(len)?)
            {
                mips_emulator::memory::emulator_memory::TernaryOption::Option1(slice) => {
                    Some((*slice).to_owned())
                }
                mips_emulator::memory::emulator_memory::TernaryOption::Option2(vec) => Some(vec),
                mips_emulator::memory::emulator_memory::TernaryOption::None => None,
            }
        })
    }

    pub(crate) fn write_to_mem(&mut self, addr: u32, data: &[u8]) -> PacketOut {
        self.emulator.cpu_mut(|cpu| {
            for (off, byte) in data.iter().enumerate() {
                unsafe {
                    if cpu
                        .raw_mem()
                        .set_u8_o_be(addr.wrapping_add(off as u32), *byte)
                        .is_err()
                    {
                        return PacketOut::err(0);
                    }
                }
            }
            PacketOut::OK
        })
    }
}

fn thread_ids(ids: &[u8]) -> String {
    let mut string = "m".into();
    let mut iter = ids.iter();
    if let Some(first) = iter.next() {
        string += format!("{:02X}", first).as_str();
    }
    for id in iter {
        string += format!(",{:02X}", id).as_str();
    }
    string
}

fn current_thread_id(id: u8) -> String {
    let mut string = "QC".into();
    string += format!("{:02X}", id).as_str();
    string
}

fn stopped(reason: u8) -> String {
    let mut string = "S".into();
    string += format!("{:02X}", reason).as_str();
    string
}

fn error(id: u8) -> String {
    let mut string = "E".into();
    string += format!("{:02X}", id).as_str();
    string
}

const REGISTER_INFO: [&str; 38] = [
  "name:r0;alt-name:zero;bitsize:32;offset:0;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r1;alt-name:at;bitsize:32;offset:4;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r2;alt-name:v0;bitsize:32;offset:8;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r3;alt-name:v1;bitsize:32;offset:12;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r4;alt-name:a0;bitsize:32;offset:16;encoding:uint;format:hex;set:General Purpose Registers;generic:arg1;",
  "name:r5;alt-name:a1;bitsize:32;offset:20;encoding:uint;format:hex;set:General Purpose Registers;generic:arg2;",
  "name:r6;alt-name:a2;bitsize:32;offset:24;encoding:uint;format:hex;set:General Purpose Registers;generic:arg3;",
  "name:r7;alt-name:a3;bitsize:32;offset:28;encoding:uint;format:hex;set:General Purpose Registers;generic:arg4;",
  "name:r8;alt-name:t0;bitsize:32;offset:32;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r9;alt-name:t1;bitsize:32;offset:36;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r10;alt-name:t2;bitsize:32;offset:40;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r11;alt-name:t3;bitsize:32;offset:44;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r12;alt-name:t4;bitsize:32;offset:48;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r13;alt-name:t5;bitsize:32;offset:52;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r14;alt-name:t6;bitsize:32;offset:56;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r15;alt-name:t7;bitsize:32;offset:60;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r16;alt-name:s0;bitsize:32;offset:64;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r17;alt-name:s1;bitsize:32;offset:68;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r18;alt-name:s2;bitsize:32;offset:72;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r19;alt-name:s3;bitsize:32;offset:76;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r20;alt-name:s4;bitsize:32;offset:80;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r21;alt-name:s5;bitsize:32;offset:84;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r22;alt-name:s6;bitsize:32;offset:88;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r23;alt-name:s7;bitsize:32;offset:92;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r24;alt-name:t8;bitsize:32;offset:96;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r25;alt-name:t9;bitsize:32;offset:100;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r26;alt-name:k0;bitsize:32;offset:104;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r27;alt-name:k1;bitsize:32;offset:108;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r28;alt-name:gp;bitsize:32;offset:112;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:r29;alt-name:sp;bitsize:32;offset:116;encoding:uint;format:hex;set:General Purpose Registers;generic:sp;",
  "name:r30;alt-name:fp;bitsize:32;offset:120;encoding:uint;format:hex;set:General Purpose Registers;generic:fp;",
  "name:r31;alt-name:ra;bitsize:32;offset:124;encoding:uint;format:hex;set:General Purpose Registers;generic:ra;",
  "name:sr;bitsize:32;offset:128;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:lo;bitsize:32;offset:132;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:hi;bitsize:32;offset:136;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:bad;bitsize:32;offset:140;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:cause;bitsize:32;offset:144;encoding:uint;format:hex;set:General Purpose Registers;",
  "name:pc;bitsize:32;offset:148;encoding:uint;format:hex;set:General Purpose Registers;generic:pc;",
];
