use crate::name::{Name, COMPRESS_POINTER_MARK16, COMPRESS_POINTER_MARK8, MAX_LABEL_COUNT};
use crate::util::{InputBuffer, OutputBuffer};

const MAX_COMPRESS_POINTER: usize = 0x3fff;
const HASH_SEED: u32 = 0x9e37_79b9;

#[derive(Clone, Copy)]
struct OffSetItem {
    len: u8,
    pos: u16,
    hash: u32,
}

impl Default for OffSetItem {
    fn default() -> Self {
        OffSetItem {
            len: 0,
            pos: 0,
            hash: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct NameComparator<'a> {
    buffer: &'a OutputBuffer,
    hash: u32,
}

struct NameRef<'a> {
    parent_level: u8,
    name: &'a Name,
}

impl<'a> NameRef<'a> {
    fn from_name(name: &'a Name) -> Self {
        NameRef {
            parent_level: 0,
            name,
        }
    }

    fn parent(&mut self) {
        self.parent_level += 1;
    }

    fn is_root(&self) -> bool {
        self.parent_level + 1 == self.name.label_count() as u8
    }

    fn raw_data(&self) -> &[u8] {
        let offset = self.name.offsets()[self.parent_level as usize] as usize;
        &self.name.raw_data()[offset..]
    }

    fn hash(&self) -> u32 {
        self.raw_data().iter().fold(0, |hash, c| {
            hash ^ (u32::from(*c)
                .wrapping_add(HASH_SEED)
                .wrapping_add(hash << 6)
                .wrapping_add(hash >> 2))
        })
    }
}

impl<'a> NameComparator<'a> {
    pub fn compare(self, item: OffSetItem, name_buffer: &mut InputBuffer) -> bool {
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
                if ch1 != ch2 {
                    return false;
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
            let nb = u16::from(self.buffer.at(next_pos + 1));
            next_pos = (u16::from(b & !(COMPRESS_POINTER_MARK8 as u8)) * 256 + nb) as usize;
            b = self.buffer.at(next_pos);
        }
        (b, (next_pos + 1) as u16)
    }
}

const BUCKETS: usize = 64;
const RESERVED_ITEMS: usize = 16;
const NO_OFFSET: u16 = 65535;
const MAX_MESSAGE_LEN: usize = 512;

pub struct MessageRender {
    buffer: OutputBuffer,
    truncated: bool,
    table: [[OffSetItem; RESERVED_ITEMS]; BUCKETS],
    item_counts: [usize; BUCKETS],
    label_hashes: [u32; MAX_LABEL_COUNT as usize],
}

impl Default for MessageRender {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageRender {
    pub fn new() -> Self {
        Self::with_capacity(MAX_MESSAGE_LEN)
    }

    pub fn with_capacity(len: usize) -> Self {
        MessageRender {
            buffer: OutputBuffer::new(len),
            truncated: false,
            table: [[OffSetItem::default(); RESERVED_ITEMS]; BUCKETS],
            item_counts: [0; BUCKETS],
            label_hashes: [0; MAX_LABEL_COUNT as usize],
        }
    }

    pub fn is_trancated(&self) -> bool {
        self.truncated
    }

    pub fn set_trancated(&mut self) {
        self.truncated = true;
    }

    fn find_offset(&self, name_buffer: &mut InputBuffer, hash: u32) -> u16 {
        let bucket_id = hash % (BUCKETS as u32);
        let comparator = NameComparator {
            buffer: &self.buffer,
            hash,
        };
        let items = &self.table[bucket_id as usize];
        for i in 0..self.item_counts[bucket_id as usize] {
            if comparator.compare(items[i], name_buffer) {
                return items[i].pos;
            }
        }
        NO_OFFSET
    }

    fn add_offset(&mut self, hash: u32, offset: u16, len: u8) {
        let bucket_id = hash % (BUCKETS as u32);
        let item_count = self.item_counts[bucket_id as usize];
        if item_count + 1 == RESERVED_ITEMS {
            panic!("too many offset with same hash");
        }
        self.table[bucket_id as usize][item_count] = OffSetItem {
            hash,
            pos: offset,
            len,
        };
        self.item_counts[bucket_id as usize] = item_count + 1;
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.truncated = false;
        for i in 0..BUCKETS {
            self.item_counts[i] = 0;
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

            self.label_hashes[label_uncompressed] = parent.hash();
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
        if !compress || label_uncompressed == label_count {
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

    pub fn is_empty(&self) -> bool {
        self.buffer.len() == 0
    }

    pub fn skip(&mut self, len: usize) {
        self.buffer.skip(len);
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
    use crate::message::Message;
    use crate::name::Name;
    use crate::util::hex::from_hex;

    #[test]
    fn test_write_name() {
        let a_example_com = Name::new("a.example.com").unwrap();
        let b_example_com = Name::new("b.example.com").unwrap();
        let a_example_org = Name::new("a.example.org").unwrap();
        let mut render = MessageRender::with_capacity(0x3fff + MAX_MESSAGE_LEN);

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

        let raw =
            from_hex("e3808583000100000001000001320131033136380331393207696e2d61646472046172706100000c0001033136380331393207494e2d4144445204415250410000060001000151800017c02a00000000000000708000001c2000093a8000015180").unwrap();
        render.clear();
        let msg = Message::from_wire(raw.as_slice()).unwrap();
        msg.to_wire(&mut render);
        assert_eq!(raw.as_slice(), render.data());
    }
}
