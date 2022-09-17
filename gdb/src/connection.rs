use std::net::TcpStream;

pub trait Connection {
    type Error: std::fmt::Debug;

    fn write(&mut self, byte: u8) -> Result<(), Self::Error>;
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        for b in buf {
            self.write(*b)?;
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Self::Error>;
    fn on_session_start(&mut self) -> Result<(), Self::Error>;
    fn on_session_end(&mut self) -> Result<(), Self::Error>;
    fn read(&mut self) -> Result<u8, Self::Error>;
    fn peek(&mut self) -> Result<Option<u8>, Self::Error>;

    fn string_repr(&self) -> Option<String> { None }
}

impl Connection for TcpStream {
    type Error = std::io::Error;

    fn write(&mut self, byte: u8) -> Result<(), Self::Error> {
        std::io::Write::write(self, &[byte])?;
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        std::io::Write::write_all(self, buf)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        std::io::Write::flush(self)?;
        Ok(())
    }

    fn on_session_start(&mut self) -> Result<(), Self::Error> {
        self.set_nodelay(true)
    }

    fn read(&mut self) -> Result<u8, Self::Error> {
        self.set_nonblocking(false)?;

        let mut buf = [0u8];
        match std::io::Read::read_exact(self, &mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(e) => Err(e),
        }
    }

    fn peek(&mut self) -> Result<Option<u8>, Self::Error> {
        self.set_nonblocking(true)?;

        let mut buf = [0u8];
        match Self::peek(self, &mut buf) {
            Ok(_) => Ok(Some(buf[0])),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn on_session_end(&mut self) -> Result<(), Self::Error> {
        self.shutdown(std::net::Shutdown::Both)?;
        Ok(())
    }

    fn string_repr(&self) -> Option<String> {
        let local = if let Ok(local) = self.local_addr(){
            local
        }else{
            return None;
        };
        let peer = if let Ok(peer) = self.peer_addr(){
            peer
        }else{
            return None;
        };
        Some(format!("{{\n\tlocal: {}\n\tpeer:  {}\n}}", local,peer))
    }
}