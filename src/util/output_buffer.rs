pub struct OutputBuffer {
    data: Vec<u8>,
}

impl OutputBuffer {
    pub fn new(len: usize) -> Self {
        OutputBuffer { data: Vec::with_capacity(len) }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn at(&self, pos: usize) -> u8 {
        if pos >= self.len() {
            panic!("random read out of range");
        }

        self.data[pos]
    }

    pub fn skip(&mut self, len: usize) {
        let new_cap = self.len() + len;
        self.data.reserve(new_cap);
        self.data.append(&mut vec![0; len]);
    }

    pub fn trim(&mut self, len: usize) {
        if len > self.len() {
            panic!("trim too many bytes");
        }

        let keep_len = self.len() - len;
        self.data.truncate(keep_len);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn write_u8(&mut self, d: u8) {
        self.data.push(d);
    }

    pub fn write_u8_at(&mut self, d: u8, pos: usize) {
        if pos + 1 > self.len() {
            panic!("write out of range");
        }

        self.data[pos] = d;
    }

    pub fn write_u16(&mut self, d: u16) {
        self.data.push(((d & 0xff00) >> 8) as u8);
        self.data.push((d & 0x00ff) as u8);
    }

    pub fn write_u16_at(&mut self, d: u16, pos: usize) {
        if pos + 2 > self.len() {
            panic!("write out of range");
        }

        self.data[pos] = ((d & 0xff00) >> 8) as u8;
        self.data[pos + 1] = (d & 0x00ff) as u8;
    }

    pub fn write_u32(&mut self, d: u32) {
        self.data.push(((d & 0xff000000) >> 24) as u8);
        self.data.push(((d & 0x00ff0000) >> 16) as u8);
        self.data.push(((d & 0x0000ff00) >> 8) as u8);
        self.data.push((d & 0x000000ff) as u8);
    }

    pub fn write_data(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }
}
