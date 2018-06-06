use error::*;

pub struct InputBuffer<'a> {
    pos: usize,
    datalen: usize,
    data: &'a [u8],
}

impl<'a> InputBuffer<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        InputBuffer {
            pos: 0,
            datalen: buf.len(),
            data: buf,
        }
    }

    pub fn set_data(&mut self, buf: &'a [u8]) {
        self.pos = 0;
        self.datalen = buf.len();
        self.data = buf;
    }

    pub fn len(&self) -> usize {
        self.datalen
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, p: usize) {
        assert!(p <= self.datalen);
        self.pos = p;
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        if self.pos + 1 > self.datalen {
            return Err(ErrorKind::InCompleteWire.into());
        }

        let num = self.data[self.pos];
        self.pos += 1;
        Ok(num)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        if self.pos + 2 > self.datalen {
            return Err(ErrorKind::InCompleteWire.into());
        }

        let mut num = (self.data[self.pos] as u16) << 8;
        num |= self.data[self.pos + 1] as u16;
        self.pos += 2;
        Ok(num)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        if self.pos + 4 > self.datalen {
            return Err(ErrorKind::InCompleteWire.into());
        }

        let mut num = (self.data[self.pos] as u32) << 24;
        num |= (self.data[self.pos + 1] as u32) << 16;
        num |= (self.data[self.pos + 2] as u32) << 8;
        num |= self.data[self.pos + 3] as u32;
        self.pos += 4;
        Ok(num)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.pos + len > self.datalen {
            return Err(ErrorKind::InCompleteWire.into());
        }

        let pos = self.pos;
        let data = &self.data[pos..(pos + len)];
        self.pos = pos + len;
        Ok(data)
    }
}
