use crate::{
    connection::{Connection},
    packets::{
        incoming::{Command, Packet, PacketParseError},
        psm::PacketStateMachine,
        response::{ResponseWritter},
    },
    target::Target, signal::Signal,
};

enum GDBState {
    Idle,
    Running,
    CtrlCInt,
    Disconnected(DisconnectReason),
}

#[derive(Debug)]
pub enum GDBError<C: Connection> {
    ConnectionRead(C::Error),
    ConnectionWrite(C::Error),
    ConnectionFlush(C::Error),
    PacketParseError(PacketParseError),
    ClientSentNack,
    CommandError,
    ExternalError,
}

#[derive(Clone, Copy, Debug)]
pub enum DisconnectReason {
    TargetExited(u8),
    TargetTerminated(Signal),
    Kill,
}

struct GDBStubCfg {
    no_ack_mode: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for GDBStubCfg {
    fn default() -> Self {
        Self { no_ack_mode: false }
    }
}

pub struct GDBStub<C: Connection, T: Target> {
    connection: C,
    target: T,
    state: GDBState,
    ptm: PacketStateMachine,
    cfg: GDBStubCfg,
    async_data: Vec<String>
}

pub enum StopReason {
    DoneStep,
    Signal(Signal),
    Exited(u8),
    Terminated(Signal),
    SwBreak,
    HwBreak,
}

impl<C: Connection, T: Target> GDBStub<C, T> {
    pub fn new(target: T, connection: C) -> Self {
        Self {
            connection,
            target,
            state: GDBState::Idle,
            ptm: PacketStateMachine::new(),
            cfg: Default::default(),
            async_data: Vec::new()
        }
    }

    pub fn has_data_to_read(&mut self) -> bool {
        self.connection.peek().map(|b| b.is_some()).unwrap_or(true)
    }

    pub fn check_non_blocking(&mut self) -> Result<Option<DisconnectReason>, GDBError<C>> {
        

        

        let res = if self.has_data_to_read() {
            match self.state {
                GDBState::Idle | GDBState::Running => {
                    let byte = self.connection.read().map_err(GDBError::ConnectionRead)?;
                    
                    self.incomming_data(byte)?;

                    // for message in &self.async_data{
                    //     self.connection.write_all(message.as_bytes()).map_err(GDBError::ConnectionRead)?;
                    // }

                    Ok(None)
                }
                GDBState::CtrlCInt => Ok(None),
                GDBState::Disconnected(reason) => Ok(Some(reason)),
            }
        } else {
            match self.state {
                GDBState::Disconnected(reason) => Ok(Some(reason)),
                _ => Ok(None),
            }
        };
    
        self.async_data.clear();

        res
    }

    pub fn target_stop(&mut self, reason: StopReason) -> Result<(), GDBError<C>> {
        //if true{
        //    return Ok(())
        //}
        //let mut buff = String::new();
        //let mut f_conn = BufferedConnection::new(&mut buff); 
        let mut res = ResponseWritter::new(&mut self.connection);
        match reason {
            StopReason::DoneStep => {
                res.write(b'S').map_err(GDBError::ConnectionWrite)?;
                res.write_hex(Signal::SIGTRAP as u8)
                    .map_err(GDBError::ConnectionWrite)?;
                self.state = GDBState::Idle;
            }
            StopReason::Signal(sig) => {
                res.write(b'S').map_err(GDBError::ConnectionWrite)?;
                res.write_hex(sig as u8)
                    .map_err(GDBError::ConnectionWrite)?;
                self.state = GDBState::Idle;
            }
            StopReason::Exited(code) => {
                res.write(b'W').map_err(GDBError::ConnectionWrite)?;
                res.write_hex(code).map_err(GDBError::ConnectionWrite)?;
                self.state = GDBState::Disconnected(DisconnectReason::TargetExited(code));
            }
            StopReason::Terminated(sig) => {
                res.write(b'X').map_err(GDBError::ConnectionWrite)?;
                res.write_hex(sig as u8)
                    .map_err(GDBError::ConnectionWrite)?;
                self.state = GDBState::Disconnected(DisconnectReason::TargetTerminated(sig));
            }
            StopReason::SwBreak => {
                res.write(b'S').map_err(GDBError::ConnectionWrite)?;
                res.write_hex(Signal::SIGTRAP as u8)
                    .map_err(GDBError::ConnectionWrite)?;
                res.write_str("swbreak:;")
                    .map_err(GDBError::ConnectionWrite)?;
                self.state = GDBState::Idle;
            }
            StopReason::HwBreak => {
                res.write(b'S').map_err(GDBError::ConnectionWrite)?;
                res.write_hex(Signal::SIGTRAP as u8)
                    .map_err(GDBError::ConnectionWrite)?;
                res.write_str("hwbreak:;")
                    .map_err(GDBError::ConnectionWrite)?;
                self.state = GDBState::Idle;
            }
        }
        res.flush().map_err(GDBError::ConnectionWrite)?;
        //self.async_data.push(buff);
        Ok(())
    }

    pub fn is_target_running_or_inturrupt(&self) -> bool {
        matches!(self.state, GDBState::Running | GDBState::CtrlCInt)
    }

    pub fn incomming_data(&mut self, byte: u8) -> Result<(), GDBError<C>> {
        if let Some(buf) = self.ptm.incomming_data(byte) {
            let packet = Packet::from_buff(buf).map_err(GDBError::PacketParseError)?;
            log::trace!("<-- {:?}", packet);
            self.incomming_packet(packet)?;
        }
        Ok(())
    }

    fn incomming_packet(&mut self, packet: Packet) -> Result<(), GDBError<C>> {
        match packet {
            Packet::Ack => Ok(()),
            Packet::Nack => Err(GDBError::ClientSentNack),
            Packet::Interrupt => {
                self.state = GDBState::CtrlCInt;
                self.target.inturrupt();
                Ok(())
            }
            Packet::Command(command) => {
                if !self.cfg.no_ack_mode {
                    self.connection
                        .write(b'+')
                        .map_err(GDBError::ConnectionWrite)?;
                    log::trace!("--> +");
                }

                let (state, response) = self.handle_command(command)?;

                if !matches!(state, GDBState::Disconnected(DisconnectReason::Kill)) {
                    response.flush().map_err(GDBError::ConnectionFlush)?;
                }
                Ok(())
            }
        }
    }

    fn handle_command(
        &mut self,
        command: Command,
    ) -> Result<(&GDBState, ResponseWritter<C>), GDBError<C>> {
        let mut response = ResponseWritter::new(&mut self.connection);

        match command {
            Command::ContinueAt(addr) => {
                self.state = GDBState::Running;
                self.target.continue_at(addr);
                response
                    .write_str("OK")
                    .map_err(GDBError::ConnectionWrite)?;
            }
            Command::StepAt(addr) => {
                self.state = GDBState::Running;
                self.target.step_at(addr);
                response
                    .write_str("OK")
                    .map_err(GDBError::ConnectionWrite)?;
            }
            Command::ContinueAtSignal(_, addr) => {
                self.state = GDBState::Running;
                self.target.continue_at(addr);
                response
                    .write_str("OK")
                    .map_err(GDBError::ConnectionWrite)?;
            }
            Command::StepAtSignal(_, addr) => {
                self.state = GDBState::Running;
                self.target.step_at(addr);
                response
                    .write_str("OK")
                    .map_err(GDBError::ConnectionWrite)?;
            }
            Command::Reset => {}

            Command::ReadRegister(_) => {
                response
                    .write_str("00000000")
                    .map_err(GDBError::ConnectionWrite)?;
            }
            Command::ReadRegisters => {
                for _ in 0..38 {
                    response
                        .write_str("00000000")
                        .map_err(GDBError::ConnectionWrite)?;
                }
            }
            Command::WriteRegister(_, _) => {}
            Command::WriteRegisters() => {}

            Command::ReadMemory(addr, size) => {
                for i in 0..size{
                    response
                    .write_hex(addr.wrapping_add(i) as u8)
                    .map_err(GDBError::ConnectionWrite)?;
                }
            }
            Command::WriteMemory(_, _) => {}

            Command::MustReplayEmpty => {}

            Command::ExceptionReason => {
                response.write_str("S").map_err(GDBError::ConnectionWrite)?;
                response
                    .write_hex(Signal::SIGTRAP as u8)
                    .map_err(GDBError::ConnectionWrite)?;
            }

            Command::Unreconized => {}

            Command::Kill => self.state = GDBState::Disconnected(DisconnectReason::Kill),
            Command::qSupported(_) => response
                .write_str("QStartNoAckMode+")
                .map_err(GDBError::ConnectionWrite)?,
            Command::qTStatus => {}
            Command::qfThreadInfo => {
                response.write_str("m1")
                .map_err(GDBError::ConnectionWrite)?
            }
            Command::qsThreadInfo => {
                response.write(b'l')
                .map_err(GDBError::ConnectionWrite)?
            }
            Command::qC => {
                response.write_str("QC1")
                .map_err(GDBError::ConnectionWrite)?
            }
            Command::qAttached => {}
            Command::qOffsets => {}
            Command::qHostInfo => {
                response
                    .write_str(
                        "triple:6D6970732D756E6B6E6F776E2D6C696E75782D676E75;endian:big;ptrsize:4;",
                    )
                    .map_err(GDBError::ConnectionWrite)?;
            }
            Command::qProcessInfo => response
                .write_str("pid:1;endian:big;")
                .map_err(GDBError::ConnectionWrite)?,
            Command::qRegisterInfo(reg) => {
                if let Some(reg_info) = REGISTER_INFO.get(reg as usize) {
                    response
                        .write_str(reg_info)
                        .map_err(GDBError::ConnectionWrite)?;
                } else {
                    
                }
            }
            Command::qMemoryRegionInfo => response
                .write_str("start:0;size:FFFFFFFF;permissions:rwx;")
                .map_err(GDBError::ConnectionWrite)?,
            Command::SelectExecutionThread(_) => {
                response
                    .write_str("OK")
                    .map_err(GDBError::ConnectionWrite)?;
            }
            Command::SelectRegisterThread(_) => {
                response
                    .write_str("OK")
                    .map_err(GDBError::ConnectionWrite)?;
            }
            Command::SelectMemoryThread(_) => {
                response
                    .write_str("OK")
                    .map_err(GDBError::ConnectionWrite)?;
            }
            Command::QStartNoAckMode => {
                self.cfg.no_ack_mode = true;
                response
                    .write_str("OK")
                    .map_err(GDBError::ConnectionWrite)?;
            }
        }
        Ok((&self.state, response))
    }
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
