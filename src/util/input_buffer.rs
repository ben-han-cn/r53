pub struct InputBuffer<'a> {
    pos:     usize,
    datalen: usize,
    data:    &'a [u8], 
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

    pub fn postion(&self) -> usize {
        self.pos
    }

    pub fn set_postion(&mut self, p: usize){
        if p > self.datalen {
            panic!("out of range");
        }
        self.pos = p;
    }

    pub fn read_u8(&mut self) -> u8 {
        if self.pos + 1 > self.datalen {
            panic!("read u8 out of range");
        }

        let num = self.data[self.pos];
        self.pos += 1;
        num
    }

    pub fn read_u16(&mut self) -> u16 {
        if self.pos + 2 > self.datalen {
            panic!("read u16 out of range");
        }

        let mut num = (self.data[self.pos] as u16) << 8;
        num |= self.data[self.pos + 1] as u16;
        self.pos += 2;
        num
    }

    pub fn read_u32(&mut self) -> u32 {
        if self.pos + 4 > self.datalen {
            panic!("read u32 out of range");
        }

        let mut num = (self.data[self.pos] as u32) << 24;
        num |= (self.data[self.pos + 1] as u32) << 16;
        num |= (self.data[self.pos + 2] as u32) << 8;
        num |= self.data[self.pos + 3] as u32;
        self.pos += 4;
        num
    }

    pub fn read_bytes(&mut self, len: usize) -> &'a [u8] {
        if self.pos + len > self.datalen {
            panic!("read byts out of range");
        }

        let pos = self.pos;
        let data = &self.data[pos..(pos+len)];
        self.pos = pos + len;
        data
    }
}
