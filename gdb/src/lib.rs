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
}

#[derive(Debug)]
#[allow(unused)]
enum PacketIn {
    Empty,
    ContinueAt(Option<u32>),
    ReadRegisters,
    WriteRegisters(),
    ReadMemory(u32, u32),
    WriteMemory(u32, Vec<u8>),
    ReadRegister(u8),
    WriteRegister(u8, u32),
    Reset,
    StepAt(Option<u32>),
    VCtrlC,
    SetThread,
    MustReplayEmpty,
    ExceptionReason,
    Unreconized,
    QueryOffsets,
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

    pub fn generate_packet_data(self) -> String {
        let plus = true;
        let mut msg = match self.pack {
            Some(Ok(ok)) => match ok {
                PacketOutSucsess::Ok => "OK".into(),
                PacketOutSucsess::Empty => "".into(),
                PacketOutSucsess::ReadRegisters(regs) => {
                    let mut msg = String::new();
                    for reg in regs {
                        msg += format!("{:08X}", reg).as_str();
                    }
                    msg
                }
                PacketOutSucsess::ExceptionReason(msg) => msg,
                PacketOutSucsess::ReadMemory(vec) => {
                    let mut msg = String::new();
                    for byte in vec {
                        msg += format!("{:02X}", byte).as_str();
                    }
                    msg
                }
            },
            Some(Err(err)) => format!("E{:20X}", err.err_id),
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

        if plus {
            msg.insert(0, '+');
        } else {
            msg.insert(0, '-');
        }

        msg
    }
}

#[derive(Debug)]
enum PacketOutSucsess {
    Ok,
    Empty,
    ReadRegisters([u32; 39]),
    ExceptionReason(String),
    ReadMemory(Vec<u8>),
}

struct PacketOutError {
    err_id: u8,
}

impl<T: CpuExternalHandler> DebugServer<T> {
    pub fn new(emulator: EmulatorInterface<T>) -> Self {
        Self { emulator }
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
        let mut data = [0; 2000];
        while match stream.read(&mut data) {
            Ok(size) => {
                let mut str = std::str::from_utf8(&data[..size]).unwrap();

                if !str.is_empty() {
                    println!();
                }

                while !str.is_empty() {
                    let len = if str.starts_with('+'){
                        println!("ack");
                        1
                    }else if str.starts_with('-'){
                        println!("- ack");
                        //stream.shutdown(Shutdown::Both).unwrap();
                        1
                    }else if let Some(index) = str.find('#') && str.starts_with('$'){
                        let end_index = index + 3;
                        let message = &str[1..index];
                        let got_check_sum = u8::from_str_radix(&str[(index+1)..end_index], 16).unwrap();

                        let mut check_sum = 0u8;
                        for byte in message.as_bytes(){
                            check_sum = check_sum.wrapping_add(*byte);
                        }
                        if check_sum == got_check_sum{
                            let response = self.parse_command(message);
                            if !response.is_empty(){
                                _ = stream.write(response.as_bytes());
                            }
                        }else{
                            println!("checksum doesnt match: {}", str);
                            break;
                        }
                        end_index
                    }else if str == "\u{3}"{
                        println!("Cntr + c (break)");
                        _ = self.emulator.stop();
                        1
                    }else{
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

    fn parse_command(&mut self, command: &str) -> String {
        let packet_in = match command {
            "?" => PacketIn::ExceptionReason,
            "g" => PacketIn::ReadRegisters,
            "r" => PacketIn::Reset,
            "vMustReplyEmpty" => PacketIn::MustReplayEmpty,
            "qOffsets" => PacketIn::QueryOffsets,
            _ => {
                if let Some(_command) = command.strip_prefix('G') {
                    todo!(); //PacketIn::WriteRegisters(u32::from_str_radix(&command[1..], 16).map_or(None, Option::Some))
                } else if let Some(command) = command.strip_prefix('p') {
                    u8::from_str_radix(command, 16)
                        .map_or(PacketIn::Unreconized, PacketIn::ReadRegister)
                } else if let Some(command) = command.strip_prefix('c') {
                    PacketIn::ContinueAt(u32::from_str_radix(command, 16).ok())
                } else if let Some(command) = command.strip_prefix('s') {
                    PacketIn::StepAt(u32::from_str_radix(command, 16).ok())
                } else if let Some(command) = command.strip_prefix('m') {
                    let mut iter = command.split(',');
                    if let (Some(add), Some(len)) = (iter.next(), iter.next())
                        && let (Ok(add), Ok(len)) = (u32::from_str_radix(add, 16), u32::from_str_radix(len, 16)){
                            PacketIn::ReadMemory(add, len)
                        }else{
                            PacketIn::Unreconized
                        }
                } else if let Some(command) = command.strip_prefix('M') {
                    let mut iter = command.split([',', ':']);
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
                    // }else if let Some(_command) = command.strip_prefix('H'){
                    //     PacketIn::SetThread
                    // }else if let Some(_command) = command.strip_prefix('H'){
                    //     PacketIn::SetThread
                    // }else if let Some(command) = command.strip_prefix('q'){
                    //     PacketIn::Empty
                    //     //u8::from_str_radix(command, 16).map_or(PacketIn::Unreconized,  PacketIn::ReadRegister)
                } else {
                    println!("unreconized input: {:#?}: {}", command, command.len());
                    PacketIn::Unreconized
                }
            }
        };

        if !matches!(packet_in, PacketIn::Unreconized) {
            println!("packet in: {:#?}", packet_in);
        }

        let packet_out = match packet_in {
            PacketIn::Empty => PacketOut::EMPTY,
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
            PacketIn::ReadRegisters => {
                PacketOut::new(PacketOutSucsess::ReadRegisters(self.read_registers()))
            }
            PacketIn::WriteRegisters() => todo!(),
            PacketIn::ReadMemory(address, length) => self
                .read_memory(address, length)
                .map_or(PacketOut::err(0), |vec| {
                    PacketOut::new(PacketOutSucsess::ReadMemory(vec))
                }),
            PacketIn::WriteMemory(addr, data) => self.write_to_mem(addr, &data),
            PacketIn::ReadRegister(_) => todo!(),
            PacketIn::WriteRegister(_, _) => todo!(),
            PacketIn::Reset => todo!(),
            PacketIn::StepAt(_) => todo!(),
            PacketIn::VCtrlC => todo!(),
            PacketIn::ExceptionReason => {
                PacketOut::new(PacketOutSucsess::ExceptionReason("S05".into()))
            }
            PacketIn::Unreconized => PacketOut::EMPTY,
            PacketIn::MustReplayEmpty => PacketOut::EMPTY,
            PacketIn::SetThread => PacketOut::OK,
            PacketIn::QueryOffsets => {
                PacketOut::EMPTY //PacketOut::new(PacketOutSucsess::GeneralText("Text=000;Data=000".into()))
            }
        };

        println!("response: {:#?}", packet_out);

        let response_data = packet_out.generate_packet_data();
        println!("response data: {:#?}", response_data);
        response_data

        // let mut response: String = "+".into();
        // response += match command{
        //     "?" => Self::generate_reply("S05"),
        //     "g" => self.read_registers(),
        //     // "c" => {
        //     //     if self.emulator.start_new_thread().is_ok(){
        //     //         Self::generate_reply("Ok")
        //     //     }else{
        //     //         return "-$#00".into()
        //     //     }
        //     // },
        //     // "si"|"stepi" => {
        //     //     if self.emulator.step_new_thread().is_ok(){
        //     //         Self::generate_reply("Ok")
        //     //     }else{
        //     //         return "-$#00".into()
        //     //     }
        //     // },
        //     _ => {
        //         if command.starts_with('G'){
        //             if self.write_register(&command[1..]){
        //                 return "_$#00".into()
        //             }else{
        //                 return "-$#00".into()
        //             }
        //         }else{
        //             Self::generate_reply("")
        //         }
        //     },
        // }.as_str();
    }

    // fn write_register(&mut self, str: &str) -> bool {
    //     true
    // }

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
