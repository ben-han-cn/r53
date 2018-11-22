use name::{
    hash_raw, Name, COMPRESS_POINTER_MARK16, COMPRESS_POINTER_MARK8, MAP_TO_LOWER, MAX_LABEL_COUNT,
};
use util::{InputBuffer, OutputBuffer};

const MAX_COMPRESS_POINTER: usize = 0x3fff;

#[derive(Clone, Copy)]
struct OffSetItem {
    hash: u32,
    pos: u16,
    len: u8,
}

struct NameComparator<'a> {
    buffer: &'a OutputBuffer,
    hash: u32,
    case_sensitive: bool,
}

struct NameRef<'a> {
    parent_level: u8,
    name: &'a Name,
}

impl<'a> NameRef<'a> {
    fn from_name(name: &'a Name) -> Self {
        NameRef {
            parent_level: 0,
            name: name,
        }
    }

    fn parent(&mut self) {
        self.parent_level += 1;
    }

    fn is_root(&self) -> bool {
        self.parent_level + 1 == self.name.label_count
    }

    fn raw_data(&self) -> &[u8] {
        let offset = self.name.offsets[self.parent_level as usize] as usize;
        &self.name.raw_data()[offset..]
    }

    fn hash(&self, case_sensitive: bool) -> u32 {
        hash_raw(self.raw_data(), case_sensitive)
    }
}

impl<'a> NameComparator<'a> {
    pub fn compare(&self, item: &OffSetItem, name_buffer: &mut InputBuffer) -> bool {
        if item.hash != self.hash || item.len != (name_buffer.len() as u8) {
            return false;
        }

        let mut item_pos = item.pos;
        loop {
            let label = self.next_label(item_pos);
            let mut name_label_len = name_buffer.read_u8().unwrap();
            if name_label_len != label.0 {
                return false;
            } else if name_label_len == 0 {
                break;
            }

            item_pos = label.1;
            while name_label_len > 0 {
                let ch1 = self.buffer.at(item_pos as usize);
                let ch2 = name_buffer.read_u8().unwrap();
                if self.case_sensitive {
                    if ch1 != ch2 {
                        return false;
                    }
                } else {
                    if MAP_TO_LOWER[ch1 as usize] != MAP_TO_LOWER[ch2 as usize] {
                        return false;
                    }
                }
                item_pos += 1;
                name_label_len -= 1;
            }
        }
        true
    }

    fn next_label(&self, pos: u16) -> (u8, u16) {
        let mut next_pos = pos as usize;
        let mut b = self.buffer.at(next_pos);
        while b & COMPRESS_POINTER_MARK8 == COMPRESS_POINTER_MARK8 {
            let nb = self.buffer.at(next_pos + 1) as u16;
            next_pos = (((b & !(COMPRESS_POINTER_MARK8 as u8)) as u16) * 256 + nb) as usize;
            b = self.buffer.at(next_pos);
        }
        (b, (next_pos + 1) as u16)
    }
}

const BUCKETS: usize = 64;
const RESERVED_ITEMS: usize = 16;
const NO_OFFSET: u16 = 65535;
const MAX_MESSAGE_LEN: u32 = 512;

pub struct MessageRender {
    buffer: OutputBuffer,
    truncated: bool,
    case_sensitive: bool,
    table: Vec<Vec<OffSetItem>>,
    label_hashes: [u32; MAX_LABEL_COUNT as usize],
}

impl MessageRender {
    pub fn new() -> Self {
        let mut render = MessageRender {
            buffer: OutputBuffer::new(MAX_MESSAGE_LEN as usize),
            truncated: false,
            case_sensitive: true,
            table: Vec::new(),
            label_hashes: [0; MAX_LABEL_COUNT as usize],
        };

        for _ in 0..BUCKETS {
            let mut items = Vec::new();
            items.reserve(RESERVED_ITEMS);
            render.table.push(items);
        }
        render
    }

    pub fn is_trancated(&self) -> bool {
        self.truncated
    }

    pub fn set_trancated(&mut self) {
        self.truncated = true;
    }

    pub fn find_offset(&self, name_buffer: &mut InputBuffer, hash: u32) -> u16 {
        let bucket_id = hash % (BUCKETS as u32);
        let comparator = NameComparator {
            buffer: &self.buffer,
            hash: hash,
            case_sensitive: self.case_sensitive,
        };
        for item in &self.table[bucket_id as usize] {
            if comparator.compare(&item, name_buffer) {
                return item.pos;
            }
        }
        NO_OFFSET
    }

    pub fn add_offset(&mut self, hash: u32, offset: u16, len: u8) {
        let bucket_id = hash % (BUCKETS as u32);
        self.table[bucket_id as usize].push(OffSetItem {
            hash: hash,
            pos: offset,
            len: len,
        });
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.truncated = false;
        self.case_sensitive = false;
        for i in 0..BUCKETS {
            self.table[i].clear()
        }
    }

    pub fn write_name(&mut self, name: &Name, compress: bool) {
        let label_count = name.label_count();
        let mut label_uncompressed = 0;
        let mut offset = NO_OFFSET;
        let mut parent = NameRef::from_name(name);
        //TODO, use reference instead of name copy to find offset
        while label_uncompressed < label_count {
            if label_uncompressed > 0 {
                parent.parent();
            }

            if parent.is_root() {
                label_uncompressed += 1;
                break;
            }

            self.label_hashes[label_uncompressed] = parent.hash(self.case_sensitive);
            if compress {
                offset = self.find_offset(
                    &mut InputBuffer::new(parent.raw_data()),
                    self.label_hashes[label_uncompressed],
                );
                if offset != NO_OFFSET {
                    break;
                }
            }
            label_uncompressed += 1;
        }

        let mut name_pos = self.buffer.len();
        if compress == false || label_uncompressed == label_count {
            self.buffer.write_bytes(name.raw_data());
        } else if label_uncompressed > 0 {
            let pos = name.offsets()[label_uncompressed as usize];
            self.buffer.write_bytes(&name.raw_data()[0..(pos as usize)]);
        }

        if compress && (offset != NO_OFFSET) {
            offset |= COMPRESS_POINTER_MARK16;
            self.buffer.write_u16(offset);
        }

        let mut name_len = name.len();
        for i in 0..label_uncompressed {
            let label_len = self.buffer.at(name_pos);
            if label_len == 0 {
                break;
            }

            if name_pos > MAX_COMPRESS_POINTER {
                break;
            }

            let hash = self.label_hashes[i];
            self.add_offset(hash, name_pos as u16, name_len as u8);
            name_pos += (label_len + 1) as usize;
            name_len -= (label_len + 1) as usize;
        }
    }

    pub fn data(&self) -> &[u8] {
        self.buffer.data()
    }

    pub fn take_data(&mut self) -> Vec<u8> {
        self.buffer.take_data()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn skip(&mut self, len: usize) {
        self.buffer.skip(len);
    }

    pub fn trim(&mut self, len: usize) {
        self.buffer.trim(len);
    }

    pub fn write_u8(&mut self, d: u8) {
        self.buffer.write_u8(d);
    }

    pub fn write_u8_at(&mut self, d: u8, pos: usize) {
        self.buffer.write_u8_at(d, pos)
    }

    pub fn write_u16(&mut self, d: u16) {
        self.buffer.write_u16(d);
    }

    pub fn write_u16_at(&mut self, d: u16, pos: usize) {
        self.buffer.write_u16_at(d, pos)
    }

    pub fn write_u32(&mut self, d: u32) {
        self.buffer.write_u32(d);
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buffer.write_bytes(data);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use message::Message;
    use name::Name;
    use util::hex::from_hex;

    #[test]
    fn test_write_name() {
        let a_example_com = Name::new("a.example.com", true).unwrap();
        let b_example_com = Name::new("b.example.com", true).unwrap();
        let a_example_org = Name::new("a.example.org", true).unwrap();
        let mut render = MessageRender::new();

        let raw = from_hex("0161076578616d706c6503636f6d000162c0020161076578616d706c65036f726700")
            .unwrap();
        render.write_name(&a_example_com, true);
        render.write_name(&b_example_com, true);
        render.write_name(&a_example_org, true);
        assert_eq!(raw.as_slice(), render.data());

        let raw =
            from_hex("0161076578616d706c6503636f6d00ffff0162076578616d706c6503636f6d00").unwrap();
        render.clear();
        let offset: usize = 0x3fff;
        render.skip(offset);
        render.write_name(&a_example_com, true);
        render.write_name(&a_example_com, true);
        render.write_name(&b_example_com, true);
        assert_eq!(raw.as_slice(), &render.data()[offset..]);

        let raw =
            from_hex("0161076578616d706c6503636f6d000162076578616d706c6503636f6d00c00f").unwrap();
        render.clear();
        render.write_name(&a_example_com, true);
        render.write_name(&b_example_com, false);
        render.write_name(&b_example_com, true);
        assert_eq!(raw.as_slice(), render.data());

        let raw = from_hex("0161076578616d706c6503636f6d000162c002c00f").unwrap();
        render.clear();
        render.write_name(&a_example_com, true);
        render.write_name(&b_example_com, true);
        render.write_name(&b_example_com, true);
        assert_eq!(raw.as_slice(), render.data());

        let raw = from_hex("0161076578616d706c6503636f6d000162c0020161076578616d706c65036f726700")
            .unwrap();
        render.clear();
        let b_example_com_cs = Name::new("b.exAmple.CoM", false).unwrap();
        render.write_name(&a_example_com, true);
        render.write_name(&b_example_com_cs, true);
        render.write_name(&a_example_org, true);
        assert_eq!(raw.as_slice(), render.data());

        let raw =
            from_hex("e3808583000100000001000001320131033136380331393207696e2d61646472046172706100000c0001033136380331393207494e2d4144445204415250410000060001000151800017c02a00000000000000708000001c2000093a8000015180").unwrap();
        render.clear();
        let msg = Message::from_wire(raw.as_slice()).unwrap();
        render.case_sensitive = true;
        msg.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
    }
}
