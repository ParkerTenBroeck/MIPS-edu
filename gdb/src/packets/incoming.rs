use std::{num::ParseIntError, str::Utf8Error};

use crate::signal::Signal;

#[derive(Debug)]
pub enum Packet {
    Ack,
    Nack,
    Interrupt,
    Command(Command),
}

impl Packet {
    pub fn from_buff(buf: &[u8]) -> Result<Self, PacketParseError> {
        match buf[0] {
            b'$' => Ok(Packet::Command(
                Command::from_buf(&buf[1..(buf.len() - 3)])
                    .map_err(PacketParseError::CommandParseError)?,
            )),
            b'+' => Ok(Packet::Ack),
            b'-' => Ok(Packet::Nack),
            0x03 => Ok(Packet::Interrupt),
            header => Err(PacketParseError::UnexpectedHeader(header)),
        }
    }
}

#[derive(Debug)]
pub enum PacketParseError {
    UnexpectedHeader(u8),
    InvalidCheckSum(u8, u8),
    InvalidCommand(String),
    CommandParseError(CommandParseError),
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Command {
    ContinueAt(Option<u32>),
    StepAt(Option<u32>),
    ContinueAtSignal(Signal, Option<u32>),
    StepAtSignal(Signal, Option<u32>),

    ReadRegisters,
    WriteRegisters(),
    ReadRegister(u8),
    WriteRegister(u8, u32),

    ReadMemory(u32, u32),
    WriteMemory(u32, Vec<u8>),

    InsertSoftwareBreakpoint(u8, u32),
    RemoveSoftwareBreakpoint(u8, u32),

    Reset,
    MustReplayEmpty,
    ExceptionReason,
    Unreconized,
    Kill,

    qLaunchGDBServer,
    qQueryGDBServer,

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
    qMemoryRegionInfo(u32),

    SelectExecutionThread(u8),
    SelectRegisterThread(u8),
    SelectMemoryThread(u8),

    QStartNoAckMode,
}

#[derive(Debug)]
pub enum CommandParseError {
    InvalidUFT8(Utf8Error),
    ParseIntError(ParseIntError),
    MalformedCommand,
}

impl Command {
    pub fn from_buf(buf: &[u8]) -> Result<Self, CommandParseError> {
        macro_rules! create_command {
            ($command:ident $s:literal => $b:block, $($tt:tt)*) => {
                if $command == $s{
                    $b
                }else{
                    create_command!($command $($tt)*)
                }
            };
            ($command:ident $s:literal = $a:ident => $b:block, $($tt:tt)*) => {
                if let Some($a) = $command.strip_prefix($s) {
                    $b
                }else{
                    create_command!($command $($tt)*)
                }
            };

            ($command:ident $s:literal => $b:expr, $($tt:tt)*) => {
                if $command == $s{
                    $b
                }else{
                    create_command!($command $($tt)*)
                }
            };
            ($command:ident $s:literal = $a:ident => $b:expr, $($tt:tt)*) => {
                if let Some($a) = $command.strip_prefix($s) {
                    $b
                }else{
                    create_command!($command $($tt)*)
                }
            };


            ($command:ident $s:literal => $b:block) => {
                if $command == $s{
                    $b
                }else{
                    println!("unreconized input: {:#?}: {}", $command, $command.len());
                    Command::Unreconized
                }
            };
            ($command:ident $s:literal = $a:ident => $b:block) => {
                if let Some($a) = $command.strip_prefix($s) {
                    $b
                }else{
                    println!("unreconized input: {:#?}: {}", $command, $command.len());
                    Command::Unreconized
                }
            };
            ($command:ident $s:literal => $b:expr) => {
                if $command == $s{
                    $b
                }else{
                    println!("unreconized input: {:#?}: {}", $command, $command.len());
                    Command::Unreconized
                }
            };
            ($command:ident $s:literal = $a:ident => $b:expr) => {
                if let Some($a) = $command.strip_prefix($s) {
                    $b
                }else{
                    println!("unreconized input: {:#?}: {}", $command, $command.len());
                    Command::Unreconized
                }
            };
        }

        let command = std::str::from_utf8(buf).map_err(CommandParseError::InvalidUFT8)?;

        Ok(create_command!(command
            "?" => Command::ExceptionReason,

            "g" => Command::ReadRegisters,
            'G' = _args => {panic!()},
            'p' = args => u8::from_str_radix(args, 16).map(Command::ReadRegister).map_err(CommandParseError::ParseIntError)?,

            'm' = args => {
                let (add, len) = args.split_once(',').map_or(Err(CommandParseError::MalformedCommand), Ok)?;
                let add = u32::from_str_radix(add, 16).map_err(CommandParseError::ParseIntError)?;
                let len = u32::from_str_radix(len, 16).map_err(CommandParseError::ParseIntError)?;
                Command::ReadMemory(add, len)
            },
            'M' = args => {
                let (addr, tmp) = args.split_once(',').map_or(Err(CommandParseError::MalformedCommand), Ok)?;
                let (len, data_arg) = tmp.split_once(':').map_or(Err(CommandParseError::MalformedCommand), Ok)?;
                let addr = u32::from_str_radix(addr, 16).map_err(CommandParseError::ParseIntError)?;
                let len = u32::from_str_radix(len, 16).map_err(CommandParseError::ParseIntError)?;
                if data_arg.len() != len as usize * 2{
                    Err(CommandParseError::MalformedCommand)?
                }else{
                    let mut data = Vec::with_capacity(len as usize);
                    let mut iter = data_arg.chars();
                    while let Some(un) = iter.next(){
                        let ln = iter.next().map_or(Err(CommandParseError::MalformedCommand), Ok)?;
                        let un = un.to_digit(16).map_or(Err(CommandParseError::MalformedCommand), Ok)?;
                        let ln = ln.to_digit(16).map_or(Err(CommandParseError::MalformedCommand), Ok)?;
                        data.push((un << 4 | ln) as u8)
                    }
                    Command::WriteMemory(addr, data)
                }
            },

            "r" => Command::Reset,
            "k" => Command::Kill,

            "z0," = args => {
                let (addr, kind) = args.split_once(',').ok_or(CommandParseError::MalformedCommand)?;
                let addr = u32::from_str_radix(addr, 16).map_err(CommandParseError::ParseIntError)?;
                let kind = u8::from_str_radix(kind, 16).map_err(CommandParseError::ParseIntError)?;
                Command::RemoveSoftwareBreakpoint(kind, addr)
            },
            "Z0," = args => {
                let (addr, kind) = args.split_once(',').ok_or(CommandParseError::MalformedCommand)?;
                let addr = u32::from_str_radix(addr, 16).map_err(CommandParseError::ParseIntError)?;
                let kind = u8::from_str_radix(kind, 16).map_err(CommandParseError::ParseIntError)?;
                Command::InsertSoftwareBreakpoint(kind, addr)
            },


            'c' = arg => Command::ContinueAt(if arg.is_empty(){
                            None
                        }else{
                            Some(u32::from_str_radix(arg, 16).map_err(CommandParseError::ParseIntError)?)
                        }),
            's' = arg => Command::StepAt(if arg.is_empty(){
                            None
                        }else{
                            Some(u32::from_str_radix(arg, 16).map_err(CommandParseError::ParseIntError)?)
                        }),
            'C' = arg => {
                let (sig, arr) = if let Some((sig, arr)) = arg.split_once(';'){
                    (sig, Some(u32::from_str_radix(arr, 16).map_err(CommandParseError::ParseIntError)?))
                }else{
                    (arg, None)
                };
                let sig = u8::from_str_radix(sig, 16).map_err(CommandParseError::ParseIntError)?;
                let sig = Signal::from_protocol_u8(sig);
                Command::ContinueAtSignal(sig, arr)
            },
            'S' = arg => {
                let (sig, arr) = if let Some((sig, arr)) = arg.split_once(';'){
                    (sig, Some(u32::from_str_radix(arr, 16).map_err(CommandParseError::ParseIntError)?))
                }else{
                    (arg, None)
                };
                let sig = u8::from_str_radix(sig, 16).map_err(CommandParseError::ParseIntError)?;
                let sig = Signal::from_protocol_u8(sig);
                Command::StepAtSignal(sig, arr)
            },

            "vMustReplyEmpty" => Command::MustReplayEmpty,

            "Hc" = arg => u8::from_str_radix(arg.trim_start_matches('-'), 16).map(Command::SelectExecutionThread).map_err(CommandParseError::ParseIntError)?,
            "Hg" = arg => u8::from_str_radix(arg.trim_start_matches('-'), 16).map(Command::SelectRegisterThread).map_err(CommandParseError::ParseIntError)?,
            "Hm" = arg => u8::from_str_radix(arg.trim_start_matches('-'), 16).map(Command::SelectMemoryThread).map_err(CommandParseError::ParseIntError)?,

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
                Command::qSupported(args)
            },

            "qQueryGDBServer" => Command::qQueryGDBServer,
            "qLaunchGDBServer" = _args => Command::qLaunchGDBServer,

            "qTStatus" => Command::qTStatus,
            "qfThreadInfo" => Command::qfThreadInfo,
            "qsThreadInfo" => Command::qsThreadInfo,
            "qC" => Command::qC,
            "qAttached" => Command::qAttached,
            "qOffsets" => Command::qOffsets,
            "qHostInfo" => Command::qHostInfo,
            "qProcessInfo" => Command::qProcessInfo,
            "qRegisterInfo" = arg => u8::from_str_radix(arg, 16).map(Command::qRegisterInfo).map_err(CommandParseError::ParseIntError)?,
            "qMemoryRegionInfo:" = arg => {
                let page = u32::from_str_radix(arg, 16).map_err(CommandParseError::ParseIntError)?;
                Command::qMemoryRegionInfo(page & !0xFFFF)
            },

            "QStartNoAckMode" => Command::QStartNoAckMode
        ))
    }
}
