pub struct PacketStateMachine {
    buf: Vec<u8>,
    state: PacketStateMachineStates,
}

impl Default for PacketStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketStateMachine {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            state: PacketStateMachineStates::Ready,
        }
    }

    pub fn incomming_data(&mut self, data: u8) -> Option<&[u8]> {
        use PacketStateMachineStates as State;
        match self.state {
            State::Ready => {
                if data == b'$' {
                    self.state = State::CommandBody;
                }
                self.buf.clear()
            }
            State::CommandBody if data == b'#' => self.state = State::CheckSum1,
            State::CommandBody => {}
            State::CheckSum1 => self.state = State::CheckSum2,
            State::CheckSum2 => self.state = State::Ready,
        }
        self.buf.push(data);
        if matches!(self.state, State::Ready) {
            match std::str::from_utf8(self.buf.as_slice()) {
                Ok(str) => {
                    log::trace!("<-- {}", str);
                }
                Err(err) => {
                    log::debug!(
                        "<-- INVALID UFT8 PACKET: {}: {:?}",
                        err,
                        self.buf.as_slice()
                    );
                }
            }
            Some(self.buf.as_slice())
        } else {
            None
        }
    }
}

enum PacketStateMachineStates {
    Ready,
    CommandBody,
    CheckSum1,
    CheckSum2,
}
