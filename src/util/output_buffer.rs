use std::mem;

pub struct OutputBuffer {
    data: Box<[u8]>,
    len: usize,
}

impl OutputBuffer {
    pub fn new(len: usize) -> Self {
        OutputBuffer {
            data: vec![0; len].into_boxed_slice(),
            len: 0,
        }
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

    #[inline]
    pub fn data(&self) -> &[u8] {
        self.data[0..self.len()].as_ref()
    }

    pub fn take_data(&mut self) -> Vec<u8> {
        let cap = self.capacity();
        let old = mem::replace(&mut self.data, Vec::with_capacity(cap).into_boxed_slice());
        old.into_vec()
    }

    pub fn at(&self, pos: usize) -> u8 {
        assert!(pos < self.len());
        self.data[pos]
    }

    pub fn skip(&mut self, len: usize) {
        assert!(self.len() + len < self.capacity());
        self.len += len;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn write_u8(&mut self, d: u8) {
        let pos = self.len();
        assert!(pos + 1 <= self.capacity());
        self.data[pos] = d;
        self.len += 1;
    }

    pub fn write_u8_at(&mut self, d: u8, pos: usize) {
        assert!(pos < self.len());
        self.data[pos] = d;
    }

    pub fn write_u16(&mut self, d: u16) {
        let pos = self.len();
        assert!(pos + 2 <= self.capacity());
        self.data[pos] = ((d & 0xff00) >> 8) as u8;
        self.data[pos + 1] = (d & 0x00ff) as u8;
        self.len += 2;
    }

    pub fn write_u16_at(&mut self, d: u16, pos: usize) {
        assert!(pos + 2 <= self.len());
        self.data[pos] = ((d & 0xff00) >> 8) as u8;
        self.data[pos + 1] = (d & 0x00ff) as u8;
    }

    pub fn write_u32(&mut self, d: u32) {
        let pos = self.len();
        assert!(pos + 4 <= self.capacity());
        self.data[pos] = ((d & 0xff00_0000) >> 24) as u8;
        self.data[pos + 1] = ((d & 0x00ff_0000) >> 16) as u8;
        self.data[pos + 2] = ((d & 0x0000_ff00) >> 8) as u8;
        self.data[pos + 3] = (d & 0x0000_00ff) as u8;
        self.len += 4;
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        let pos = self.len();
        assert!(pos + data.len() <= self.capacity());
        self.data[pos..(pos + data.len())].copy_from_slice(data);
        self.len += data.len();
    }
}
