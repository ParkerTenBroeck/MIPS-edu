use crate::connection::Connection;

pub struct ResponseWritter<'a, C: Connection> {
    conn: &'a mut C,

    started: bool,
    check_sum: u8,
    msg: Vec<u8>,
}

impl<'a, C: Connection> ResponseWritter<'a, C> {
    pub fn flush(mut self) -> Result<(), C::Error> {
        let checksum = self.check_sum;
        self.write(b'#')?;
        self.write_hex(checksum)?;
        log::trace!("--> {}", String::from_utf8_lossy(&self.msg));
        self.conn.write_all(&self.msg)?;
        self.conn.flush()?;
        Ok(())
    }


    fn inner_write(&mut self, byte: u8) -> Result<(), C::Error> {
        if !self.started {
            self.started = true;
            self.msg.push(b'$');
            //self.conn.write(b'$')?;
        }
        self.msg.push(byte);
        self.check_sum = self.check_sum.wrapping_add(byte);
        //self.conn.write(byte)
        Ok(())
    }

    pub fn write_hex(&mut self, byte: u8) -> Result<(), C::Error> {
        for digit in [(byte & 0xf0) >> 4, byte & 0x0f] {
            let c = match digit {
                0..=9 => b'0' + digit,
                10..=15 => b'a' + digit - 10,
                _ => digit,
            };
            self.write(c)?;
        }
        Ok(())
    }

    pub fn write_str(&mut self, str: &str) -> Result<(), C::Error> {
        for &b in str.as_bytes().iter() {
            self.write(b)?
        }
        Ok(())
    }

    pub fn write_hex_buff(&mut self, data: &[u8]) -> Result<(), C::Error> {
        for &b in data {
            self.write_hex(b)?;
        }
        Ok(())
    }

    pub fn write(&mut self, byte: u8) -> Result<(), C::Error> {
        self.inner_write(byte)
    }

    pub fn new(conn: &'a mut C) -> Self {
        Self {
            conn,
            started: false,
            check_sum: 0,
            msg: Default::default(),
        }
    }
}
