use anyhow::{bail, Result};

pub struct InputBuffer<'a> {
    pos: usize,
    len: usize,
    data: &'a [u8],
}

impl<'a> InputBuffer<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        InputBuffer {
            pos: 0,
            len: buf.len(),
            data: buf,
        }
    }

    pub fn set_data(&mut self, buf: &'a [u8]) {
        self.pos = 0;
        self.len = buf.len();
        self.data = buf;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, p: usize) -> Result<()> {
        if p <= self.len {
            self.pos = p;
            Ok(())
        } else {
            bail!("set position out of range");
        }
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        if self.pos < self.len {
            let num = self.data[self.pos];
            self.pos += 1;
            Ok(num)
        } else {
            bail!("no space for u8 left in buffer");
        }
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        if self.pos + 1 < self.len {
            let mut num = u16::from(self.data[self.pos]) << 8;
            num |= u16::from(self.data[self.pos + 1]);
            self.pos += 2;
            Ok(num)
        } else {
            bail!("no space for u16 left in buffer");
        }
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        if self.pos + 3 < self.len {
            let mut num = u32::from(self.data[self.pos]) << 24;
            num |= u32::from(self.data[self.pos + 1]) << 16;
            num |= u32::from(self.data[self.pos + 2]) << 8;
            num |= u32::from(self.data[self.pos + 3]);
            self.pos += 4;
            Ok(num)
        } else {
            bail!("no space for u32 left in buffer");
        }
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.pos + len <= self.len {
            let pos = self.pos;
            let data = &self.data[pos..(pos + len)];
            self.pos = pos + len;
            Ok(data)
        } else {
            bail!("no space for bytes with len {} left in buffer", len);
        }
    }
}
