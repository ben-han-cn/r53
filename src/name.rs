use crate::label_sequence::LabelSequence;
use crate::label_slice::LabelSlice;
use crate::message_render::MessageRender;
use crate::util::InputBuffer;
use anyhow::{self, bail, ensure, Result};
use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NameRelation {
    SuperDomain,
    SubDomain,
    Equal,
    CommonAncestor,
    None,
}

pub const MAX_WIRE_LEN: usize = 255;
pub const MAX_LABEL_COUNT: u8 = 128;
pub const MAX_LABEL_LEN: u8 = 63;
pub const COMPRESS_POINTER_MARK8: u8 = 0xc0;
pub const COMPRESS_POINTER_MARK16: u16 = 0xc000;

#[derive(Clone)]
pub struct Name {
    raw: Vec<u8>,
    offsets: Vec<u8>,
}

pub fn root() -> Name {
    Name {
        raw: vec![0],
        offsets: vec![0],
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NameComparisonResult {
    pub order: i8,
    pub common_label_count: u8,
    pub relation: NameRelation,
}

#[derive(Debug, Eq, PartialEq)]
enum FtStat {
    Init,
    Start,
    Ordinary,
    Initialescape,
    Escape,
    Escdecimal,
}

#[derive(Eq, PartialEq)]
enum FwStat {
    Start,
    Ordinary,
    NewCurrent,
}

#[inline]
fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

static DIGITAL_VALUES: &[i8] = &[
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 16
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 32
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 48
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, -1, -1, -1, -1, -1, // 64
    -1, 10, 11, 12, 13, 14, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 80
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 96
    -1, 10, 11, 12, 13, 14, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 112
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 128
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];

#[inline]
fn digitvalue(c: usize) -> i8 {
    DIGITAL_VALUES[c]
}

pub static MAP_TO_LOWER: &[u8] = &[
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
    0x40, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
    0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
    0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
    0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
    0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
    0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
    0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

#[inline]
pub fn lower_case(c: usize) -> u8 {
    MAP_TO_LOWER[c]
}

pub fn string_parse(
    name_raw: &[u8],
    start_pos: usize,
    end: usize,
    as_absolute: bool,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut start = start_pos;
    let mut data: Vec<u8> = Vec::with_capacity(end - start + 1);
    let mut offsets: Vec<u8> = Vec::new();
    let mut count = 0;
    let mut digits = 0;
    let mut value: i32 = 0;
    let mut done = false;
    let mut is_root = false;
    let mut state = FtStat::Init;
    let mut c: char;

    offsets.push(0);
    'outer: loop {
        if data.len() >= MAX_WIRE_LEN || start == end {
            break;
        }
        c = name_raw[start] as char;
        start += 1;

        'inner: loop {
            if state == FtStat::Init {
                if c == '.' {
                    ensure!(start == end, "label isn't terminated");
                    is_root = true;
                } else if c == '@' && start == end {
                    is_root = true;
                }

                if is_root {
                    data.push(0);
                    done = true;
                    break 'outer;
                }
                state = FtStat::Start;
            } else if state == FtStat::Start {
                data.push(0);
                count = 0;
                if c == '\\' {
                    state = FtStat::Initialescape;
                } else {
                    state = FtStat::Ordinary;
                }
            } else if state == FtStat::Ordinary {
                if c == '.' {
                    ensure!(count != 0, "duplicate period in name");
                    data[offsets[offsets.len() - 1] as usize] = count;
                    offsets.push(data.len() as u8);
                    if start == end {
                        data.push(0);
                        done = true;
                        break 'outer;
                    }
                    state = FtStat::Start;
                } else if c == '\\' {
                    state = FtStat::Escape;
                } else {
                    count += 1;
                    ensure!(count <= MAX_LABEL_LEN, "label len exceed limit");
                    data.push(c as u8);
                }
                break 'inner;
            } else if state == FtStat::Initialescape {
                ensure!(c != '[', "invalid label character");
                state = FtStat::Escape;
            } else if state == FtStat::Escape {
                if !is_digit(c) {
                    count += 1;
                    ensure!(count <= MAX_LABEL_LEN, "label len exceed limit");
                    data.push(c as u8);
                    state = FtStat::Ordinary;
                } else {
                    digits = 0;
                    value = 0;
                    state = FtStat::Escdecimal;
                }
            } else if state == FtStat::Escdecimal {
                ensure!(is_digit(c), "invalid decimal format");
                value *= 10;
                value += i32::from(digitvalue(c as usize));
                digits += 1;
                if digits == 3 {
                    ensure!(value <= 255, "invalid decimal format");
                    count += 1;
                    ensure!(count <= MAX_LABEL_LEN, "label len exceed limit");
                    data.push(c as u8);
                    state = FtStat::Ordinary;
                }
                break 'inner;
            } else {
                panic!("impossible state");
            }
        }
    }

    if !done {
        ensure!(data.len() < MAX_WIRE_LEN, "name length exceed limit");
        assert!(start == end);
        ensure!(state == FtStat::Ordinary, "name isn't complete");
        assert!(count != 0);
        data[offsets[offsets.len() - 1] as usize] = count as u8;
        if as_absolute {
            offsets.push(data.len() as u8);
            data.push(0);
        }
    }

    Ok((data, offsets))
}

impl Name {
    pub fn new(name: &str) -> Result<Name> {
        let name_len = name.len();
        match string_parse(name.as_bytes(), 0, name_len, true) {
            Ok((data, offsets)) => Ok(Name { raw: data, offsets }),
            Err(e) => Err(e),
        }
    }

    pub(crate) fn from_raw(raw: Vec<u8>, offsets: Vec<u8>) -> Self {
        Name { raw, offsets }
    }

    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        let mut n: usize = 0;
        let mut nused: usize = 0;
        let mut cused: usize = 0;
        let mut done = false;
        let mut data: Vec<u8> = Vec::with_capacity(15);
        let mut offsets: Vec<u8> = Vec::with_capacity(5);
        let mut seen_pointer: bool = false;
        let mut state = FwStat::Start;
        let mut current = buf.position() as usize;
        let pos_beg = current;
        let mut biggest_pointer = current;
        let mut new_current: usize = 0;

        while current < buf.len() && !done {
            let c = buf.read_u8()?;
            current += 1;
            if !seen_pointer {
                cused += 1;
            }

            if state == FwStat::Start {
                if c <= MAX_LABEL_LEN {
                    offsets.push(nused as u8);
                    nused += (c as usize) + 1;
                    ensure!(nused <= MAX_WIRE_LEN, "name length exceed limit");
                    data.push(c);
                    if c == 0 {
                        done = true;
                    }
                    n = c as usize;
                    state = FwStat::Ordinary;
                } else if c & COMPRESS_POINTER_MARK8 == COMPRESS_POINTER_MARK8 {
                    new_current = (c & !COMPRESS_POINTER_MARK8) as usize;
                    n = 1;
                    state = FwStat::NewCurrent;
                } else {
                    bail!("invalid label count");
                }
            } else if state == FwStat::Ordinary {
                data.push(c);
                n -= 1;
                if n == 0 {
                    state = FwStat::Start
                }
            } else if state == FwStat::NewCurrent {
                new_current *= 256;
                new_current += c as usize;
                n -= 1;
                if n != 0 {
                    break;
                }
                ensure!(new_current < biggest_pointer, "invalid compress pointer");
                biggest_pointer = new_current;
                current = new_current;
                buf.set_position(current)?;
                seen_pointer = true;
                state = FwStat::Start;
            }
        }

        ensure!(done, "in complete name");
        buf.set_position(pos_beg + cused)?;
        Ok(Name { raw: data, offsets })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len() as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn label_count(&self) -> usize {
        self.offsets.len() as usize
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<()> {
        render.write_name(self, true)
    }

    pub fn into_label_sequence(mut self, first_label: usize, last_label: usize) -> LabelSequence {
        let last_label_len: u8 = self.raw[usize::from(self.offsets[last_label])] + 1;
        let data_length: u8 = self.offsets[last_label] + last_label_len;
        self.raw.drain(data_length as usize..);
        self.offsets.drain(last_label + 1..);
        if first_label != 0 {
            self.raw.drain(0..self.offsets[first_label] as usize);
            self.offsets.drain(0..first_label);
            let curr_label_value = self.offsets[0];
            for v in &mut self.offsets {
                *v -= curr_label_value;
            }
        }

        LabelSequence::new(self.raw, self.offsets)
    }

    pub fn get_relation(&self, other: &Name) -> NameComparisonResult {
        LabelSlice::from_name(self).compare(&LabelSlice::from_name(other), false)
    }

    pub fn concat_all(&self, suffixes: &[&Name]) -> Result<Name> {
        let mut final_length = self.len();
        let mut final_label_count = self.label_count();
        let suffix_count = suffixes.len();
        for suffix in suffixes {
            final_length += suffix.len() - 1;
            final_label_count += suffix.label_count() - 1;
        }

        ensure!(
            final_length <= MAX_WIRE_LEN,
            "concat label generate too long name"
        );
        ensure!(
            final_label_count <= MAX_LABEL_COUNT as usize,
            "label count exceed limit"
        );

        let mut raw = Vec::with_capacity(final_length as usize);
        raw.extend_from_slice(&self.raw[..(self.len() as usize - 1)]);
        for suffix in &suffixes[..(suffix_count as usize - 1)] {
            raw.extend_from_slice(&suffix.raw[..(suffix.len() as usize - 1)])
        }
        raw.extend_from_slice(&(suffixes[suffix_count - 1].raw[..]));

        let mut offsets = Vec::with_capacity(final_label_count as usize);
        offsets.extend_from_slice(&self.offsets[..]);
        let mut copied_len = self.label_count();
        for suffix in suffixes {
            let last_offset = offsets[copied_len as usize - 1];
            offsets.extend_from_slice(&suffix.offsets[1..(suffix.label_count() as usize)]);
            for i in copied_len..(copied_len + suffix.label_count() - 1) {
                offsets[i as usize] += last_offset as u8
            }
            copied_len += suffix.label_count() - 1;
        }

        Ok(Name { raw, offsets })
    }

    pub fn concat(&self, suffix: &Name) -> Result<Name> {
        self.concat_all(&[suffix])
    }

    pub fn reverse(&self) -> Name {
        if self.label_count() == 1 {
            return self.clone();
        }

        let mut raw = Vec::with_capacity(self.len() as usize);
        let mut offsets = Vec::with_capacity(self.label_count() as usize);
        let mut label_len = 0;
        let mut i = (self.label_count() - 2) as i8;
        while i >= 0 {
            let label_start = self.offsets[i as usize] as usize;
            let label_end = self.offsets[(i + 1) as usize] as usize;
            raw.extend_from_slice(&self.raw[label_start..label_end]);
            offsets.push(label_len as u8);
            label_len += label_end - label_start;
            i -= 1;
        }

        raw.push(0);
        offsets.push(label_len as u8);
        Name { raw, offsets }
    }

    pub fn split(&self, start_label: usize, label_count_: usize) -> Result<Name> {
        let max_label_count = self.label_count() as usize;
        ensure!(start_label < max_label_count, "invalid split index");
        let mut label_count = label_count_;
        if start_label + label_count > max_label_count {
            label_count = max_label_count - start_label;
        }

        if start_label + label_count == (self.label_count() as usize) {
            let mut offsets = Vec::with_capacity(label_count);
            let first_offset = self.offsets[start_label];
            offsets.extend_from_slice(&self.offsets[start_label..]);
            let start_pos = offsets[0] as usize;
            let mut raw = Vec::with_capacity(self.len() as usize - start_pos);
            raw.extend_from_slice(&self.raw[start_pos..]);
            for offset in offsets.iter_mut().take(label_count) {
                *offset -= first_offset;
            }
            Ok(Name { raw, offsets })
        } else {
            let mut offsets = Vec::with_capacity(label_count + 1);
            offsets.extend_from_slice(&self.offsets[start_label..=start_label + label_count]);
            let mut raw = Vec::with_capacity((offsets[label_count] - offsets[0] + 1) as usize);
            raw.extend_from_slice(
                &self.raw[(offsets[0] as usize)..(offsets[label_count] as usize)],
            );

            let first_offset = self.offsets[start_label];
            for offset in offsets.iter_mut().take(label_count + 1) {
                *offset -= first_offset;
            }
            raw.push(0);
            Ok(Name { raw, offsets })
        }
    }

    pub fn parent(&self, level: usize) -> Result<Name> {
        self.split(level, self.label_count() as usize - level)
    }

    pub fn as_lowercase(&mut self) {
        let mut label_count = self.label_count();
        let mut p: usize = 0;
        while label_count > 0 {
            label_count -= 1;
            let mut label_len = self.raw[p];
            p += 1;
            while label_len > 0 {
                self.raw[p] = lower_case(self.raw[p] as usize);
                p += 1;
                label_len -= 1;
            }
        }
    }

    pub fn strip_left(&self, label_count: usize) -> Name {
        assert!(label_count < (self.label_count() as usize));

        if label_count == 0 {
            return self.clone();
        }

        let new_label_count = (self.label_count() as usize) - label_count;
        let mut offsets = Vec::with_capacity(new_label_count);
        offsets.extend_from_slice(&self.offsets[label_count..]);
        let start_pos = self.offsets[label_count] as usize;
        for offset in offsets.iter_mut().take(new_label_count) {
            *offset -= start_pos as u8;
        }
        let new_length = self.len() as usize - start_pos;
        let mut raw = Vec::with_capacity(new_length);
        raw.extend_from_slice(&self.raw[start_pos..]);
        Name { raw, offsets }
    }

    pub fn into_ancestor(mut self, label_count: usize) -> Name {
        assert!(label_count < (self.label_count() as usize));

        if label_count == 0 {
            return self;
        }

        let new_label_count = (self.label_count() as usize) - label_count;
        let start_pos = self.offsets[label_count] as usize;
        self.offsets = self.offsets.split_off(label_count);
        for i in 0..new_label_count {
            self.offsets[i] -= start_pos as u8;
        }
        self.raw = self.raw.split_off(start_pos);
        self
    }

    pub fn strip_right(&self, label_count: usize) -> Name {
        assert!(label_count < self.label_count() as usize);

        if label_count == 0 {
            return self.clone();
        }

        let new_label_count = self.label_count() as usize - label_count;
        let end_label = new_label_count - 1;
        let end_pos = self.offsets[end_label] as usize;
        let mut raw = Vec::with_capacity(end_pos + 1);
        raw.extend_from_slice(&self.raw[0..=end_pos]);
        raw[end_pos] = 0;

        let mut offsets = Vec::with_capacity(new_label_count);
        offsets.extend_from_slice(&self.offsets[0..=end_label]);
        Name { raw, offsets }
    }

    pub fn into_child(mut self, label_count: usize) -> Name {
        assert!(label_count < self.label_count() as usize);

        if label_count == 0 {
            return self;
        }

        let new_label_count = self.label_count() as usize - label_count;
        let end_label = new_label_count - 1;
        let end_pos = self.offsets[end_label] as usize;
        self.raw.truncate(end_pos + 1);
        self.raw[end_pos] = 0;
        self.offsets.truncate(new_label_count);
        self
    }

    pub fn is_subdomain(&self, parent: &Name) -> bool {
        if self.len() < parent.len() || self.label_count() < parent.label_count() {
            return false;
        }

        let mut i = self.len() - 1;
        let mut j = parent.len() - 1;
        while j > 0 {
            if lower_case(parent.raw[j as usize] as usize)
                != lower_case(self.raw[i as usize] as usize)
            {
                return false;
            }
            j -= 1;
            i -= 1;
        }
        true
    }

    pub fn is_wildcard(&self) -> bool {
        if self.raw.len() < 3 || self.offsets.len() < 2 || self.offsets[1] != 2 {
            false
        } else {
            self.raw[0] == 1 && self.raw[1] == b'*'
        }
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        self.raw.len() == 1 && self.raw[0] == 0
    }

    #[inline]
    pub fn raw_data(&self) -> &[u8] {
        self.raw.as_slice()
    }

    #[inline]
    pub fn offsets(&self) -> &[u8] {
        self.offsets.as_slice()
    }
}

impl FromStr for Name {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Name::new(s)
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", LabelSlice::from_name(self))
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Name) -> bool {
        if self.len() != other.len() || self.label_count() != other.label_count() {
            return false;
        }

        let mut pos = 0;
        let mut l = self.label_count();
        while l > 0 {
            let mut count = self.raw[pos as usize];
            if count != other.raw[pos as usize] {
                return false;
            }

            pos += 1;

            while count > 0 {
                count -= 1;
                if lower_case(self.raw[pos as usize] as usize)
                    != lower_case(other.raw[pos as usize] as usize)
                {
                    return false;
                }
                pos += 1;
            }
            l -= 1;
        }

        true
    }
}

impl Eq for Name {}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Name) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Name) -> Ordering {
        let relation = self.get_relation(other);
        relation.order.cmp(&0)
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for c in self.raw.as_slice() {
            state.write_u8(lower_case(*c as usize));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name_parse() {
        let name: Name = "www.000.\\231\\167\\187\\229\\138\\168.".parse().unwrap();
        assert_eq!(name.label_count(), 4);
    }

    #[test]
    fn test_name_concat() {
        let www_knet_cn = Name::new("www.knet.Cn").unwrap();
        let www_knet = Name::new("www.knet").unwrap();
        let cn = Name::new("cn").unwrap();

        let relation = www_knet_cn.get_relation(&www_knet.concat(&cn).unwrap());
        assert_eq!(relation.order, 0);
        assert_eq!(relation.common_label_count, 4);
        assert_eq!(relation.relation, NameRelation::Equal);

        assert_eq!(
            www_knet_cn.reverse().to_string(),
            "Cn.knet.www.".to_string()
        );

        assert_eq!(
            www_knet_cn.split(0, 1).unwrap().to_string(),
            "www.".to_string()
        );
        assert_eq!(
            www_knet_cn.split(0, 4).unwrap().to_string(),
            "www.knet.Cn.".to_string()
        );
        assert_eq!(
            www_knet_cn.split(1, 3).unwrap().to_string(),
            "knet.Cn.".to_string()
        );
        assert_eq!(
            www_knet_cn.split(1, 2).unwrap().to_string(),
            "knet.Cn.".to_string()
        );
        assert_eq!(
            www_knet_cn.split(1, 1).unwrap().to_string(),
            "knet.".to_string()
        );

        assert_eq!(
            www_knet_cn.parent(0).unwrap().to_string(),
            "www.knet.Cn.".to_string()
        );
        assert_eq!(
            www_knet_cn.parent(1).unwrap().to_string(),
            "knet.Cn.".to_string()
        );
        assert_eq!(
            www_knet_cn.parent(2).unwrap().to_string(),
            "Cn.".to_string()
        );
        assert_eq!(www_knet_cn.parent(3).unwrap().to_string(), ".".to_string());
        assert!(www_knet_cn.parent(4).is_err())
    }

    #[test]
    fn test_name_compare() {
        let www_knet_cn_mix_case = Name::new("www.KNET.cN").unwrap();
        let www_knet_cn = Name::new("www.knet.cn.").unwrap();
        let relation = www_knet_cn.get_relation(&www_knet_cn_mix_case);
        assert_eq!(relation.order, 0);
        assert_eq!(relation.common_label_count, 4);
        assert_eq!(relation.relation, NameRelation::Equal);
        assert!(www_knet_cn == www_knet_cn_mix_case);

        let www_knet_com = Name::new("www.knet.com").unwrap();
        let relation = www_knet_cn.get_relation(&www_knet_com);
        assert!(relation.order < 0);
        assert!(www_knet_cn < www_knet_com);
        assert_eq!(relation.common_label_count, 1);
        assert_eq!(relation.relation, NameRelation::CommonAncestor);

        let baidu_com = Name::new("baidu.com.").unwrap();
        let www_baidu_com = Name::new("www.baidu.com").unwrap();
        let relation = baidu_com.get_relation(&www_baidu_com);
        assert!(relation.order < 0);
        assert!(baidu_com < www_baidu_com);
        assert_eq!(relation.common_label_count, 3);
        assert_eq!(relation.relation, NameRelation::SuperDomain);

        let range1 = Name::new("f14OMOZF16PGI-Wh2FGVXQ8I6Ma8NFuX3yH.UlG5SSbTOzya-acxXEK0W9D4pewmpEyhJ5VMQT1qdDk5xUOZo.3Go5Nbx0-wJBKnOHobRncMDVWqpekEBMYWaa1RChZelAqqIENfv-EGh.YpG6Natyn0av0VQd2aSmf05bt5WkkZao-4O9hU8ZO2WNgVu2C6sOGraLdZPg.p6S.CkQ.NM.").unwrap();
        let range2 = Name::new("qI0.BUHM.n.").unwrap();
        let relation = range1.get_relation(&range2);
        assert!(relation.order > 0);
        assert!(range1 > range2);
    }

    #[test]
    fn test_name_strip() {
        let www_knet_cn_mix_case = Name::new("www.KNET.cN").unwrap();
        assert_eq!(&www_knet_cn_mix_case.strip_left(1).to_string(), "KNET.cN.");
        assert_eq!(&www_knet_cn_mix_case.strip_left(2).to_string(), "cN.");
        assert_eq!(&www_knet_cn_mix_case.strip_left(3).to_string(), ".");
        assert_eq!(
            &www_knet_cn_mix_case.strip_right(1).to_string(),
            "www.KNET."
        );
        assert_eq!(&www_knet_cn_mix_case.strip_right(2).to_string(), "www.");
        assert_eq!(&www_knet_cn_mix_case.strip_right(3).to_string(), ".");

        let mut name = www_knet_cn_mix_case.clone();
        let ancestors = ["KNET.cN.", "cN.", "."];
        for i in 0..3 {
            name = name.into_ancestor(1);
            assert_eq!(name.to_string(), ancestors[i]);
        }

        let mut name = www_knet_cn_mix_case.clone();
        let children = ["www.KNET.", "www.", "."];
        for i in 0..3 {
            name = name.into_child(1);
            assert_eq!(name.to_string(), children[i]);
        }
    }

    fn hash_helper(name: &Name) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_name_hash() {
        let name1 = Name::new("wwwnnnnnnnnnnnnn.KNET.cNNNNNNNNN").unwrap();
        let name2 = Name::new("wwwnnnnnnnnnnnnn.KNET.cNNNNNNNNn").unwrap();
        let name3 = Name::new("wwwnnnnnnnnnnnnn.KNET.cNNNNNNNNN.baidu.com.cn.net").unwrap();
        assert_eq!(hash_helper(&name1), hash_helper(&name2));
        assert_ne!(hash_helper(&name1), hash_helper(&name3));
    }

    #[test]
    fn test_short_name() {
        let name = Name::new("c").unwrap();
        assert_eq!(name.to_string(), "c.");
    }

    #[test]
    fn test_name_is_subdomain() {
        let www_knet_cn = Name::new("www.knet.Cn").unwrap();
        let www_knet = Name::new("www.knet").unwrap();
        let knet_cn = Name::new("knet.Cn").unwrap();
        let cn = Name::new("cn").unwrap();
        let knet = Name::new("kNet").unwrap();
        let root = root();
        assert!(
            www_knet_cn.is_subdomain(&knet_cn)
                && knet_cn.is_subdomain(&cn)
                && knet_cn.is_subdomain(&root)
                && cn.is_subdomain(&root)
                && knet.is_subdomain(&root)
                && www_knet_cn.is_subdomain(&root)
                && www_knet.is_subdomain(&root)
                && www_knet.is_subdomain(&www_knet)
                && root.is_subdomain(&root)
        );
        assert!(
            knet.is_subdomain(&knet_cn) == false
                && knet.is_subdomain(&cn) == false
                && root.is_subdomain(&cn) == false
                && www_knet.is_subdomain(&www_knet_cn) == false
        );
    }

    #[test]
    fn test_is_wildcard() {
        let wildcard_names = vec!["*", "*.a", "*.*.a"];
        let not_wildcard_names = vec!["a.*", "a.*.a", "a.*.*.a"];
        for name_str in wildcard_names {
            let name = Name::new(name_str).unwrap();
            assert!(name.is_wildcard());
        }
        for name_str in not_wildcard_names {
            let name = Name::new(name_str).unwrap();
            assert!(name.is_wildcard() == false);
        }
    }

    #[test]
    fn test_is_root() {
        let root_names = vec!["."];
        let not_root_names = vec!["a", "a.a"];
        for name_str in root_names {
            let name = Name::new(name_str).unwrap();
            assert!(name.is_root());
        }
        for name_str in not_root_names {
            let name = Name::new(name_str).unwrap();
            assert!(name.is_root() == false);
        }
        let name = Name::new("a.a.a").unwrap();
        assert!(name.parent(3).unwrap().is_root());
    }
}
