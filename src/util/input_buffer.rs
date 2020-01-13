use anyhow::{ensure, Result};

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

    pub fn is_empty(&self) -> bool {
        self.datalen == 0
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, p: usize) {
        assert!(p <= self.datalen);
        self.pos = p;
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        ensure!(
            self.pos + 1 <= self.datalen,
            "no space for u8 left in buffer"
        );
        let num = self.data[self.pos];
        self.pos += 1;
        Ok(num)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        ensure!(
            self.pos + 2 <= self.datalen,
            "no space for u16 left in buffer"
        );
        let mut num = u16::from(self.data[self.pos]) << 8;
        num |= u16::from(self.data[self.pos + 1]);
        self.pos += 2;
        Ok(num)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        ensure!(
            self.pos + 4 <= self.datalen,
            "no space for u32 left in buffer"
        );
        let mut num = u32::from(self.data[self.pos]) << 24;
        num |= u32::from(self.data[self.pos + 1]) << 16;
        num |= u32::from(self.data[self.pos + 2]) << 8;
        num |= u32::from(self.data[self.pos + 3]);
        self.pos += 4;
        Ok(num)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        ensure!(
            self.pos + len <= self.datalen,
            "no space for bytes with len {} left in buffer",
            len
        );

        let pos = self.pos;
        let data = &self.data[pos..(pos + len)];
        self.pos = pos + len;
        Ok(data)
    }
}
