use anyhow::{bail, Result};

pub struct OutputBuffer<'a> {
    data: &'a mut [u8],
    len: usize,
}

impl<'a> OutputBuffer<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        OutputBuffer { data, len: 0 }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn at(&self, pos: usize) -> Result<u8> {
        if pos < self.len() {
            Ok(self.data[pos])
        } else {
            bail!("pos is out of range");
        }
    }

    pub fn skip(&mut self, len: usize) -> Result<()> {
        if self.len() + len < self.capacity() {
            self.len += len;
            Ok(())
        } else {
            bail!("skip out of the range");
        }
    }

    pub fn write_u8(&mut self, d: u8) -> Result<()> {
        let pos = self.len();
        if pos < self.capacity() {
            self.data[pos] = d;
            self.len += 1;
            Ok(())
        } else {
            bail!("write_u8 out of the range");
        }
    }

    pub fn write_u8_at(&mut self, d: u8, pos: usize) -> Result<()> {
        if pos < self.len() {
            self.data[pos] = d;
            Ok(())
        } else {
            bail!("write_u8_at out of the range");
        }
    }

    pub fn write_u16(&mut self, d: u16) -> Result<()> {
        let pos = self.len();
        if pos + 2 <= self.capacity() {
            self.data[pos] = ((d & 0xff00) >> 8) as u8;
            self.data[pos + 1] = (d & 0x00ff) as u8;
            self.len += 2;
            Ok(())
        } else {
            bail!("write_u16 out of the range");
        }
    }

    pub fn write_u16_at(&mut self, d: u16, pos: usize) -> Result<()> {
        if pos + 2 <= self.len() {
            self.data[pos] = ((d & 0xff00) >> 8) as u8;
            self.data[pos + 1] = (d & 0x00ff) as u8;
            Ok(())
        } else {
            bail!("write_u16_at out of the range");
        }
    }

    pub fn write_u32(&mut self, d: u32) -> Result<()> {
        let pos = self.len();
        if pos + 4 <= self.capacity() {
            self.data[pos] = ((d & 0xff00_0000) >> 24) as u8;
            self.data[pos + 1] = ((d & 0x00ff_0000) >> 16) as u8;
            self.data[pos + 2] = ((d & 0x0000_ff00) >> 8) as u8;
            self.data[pos + 3] = (d & 0x0000_00ff) as u8;
            self.len += 4;
            Ok(())
        } else {
            bail!("write_u32 out of the range");
        }
    }

    pub fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        let pos = self.len();
        if pos + data.len() <= self.capacity() {
            self.data[pos..(pos + data.len())].copy_from_slice(data);
            self.len += data.len();
            Ok(())
        } else {
            bail!("write_bytes out of range");
        }
    }
}
