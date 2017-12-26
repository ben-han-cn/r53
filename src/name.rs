use std::fmt;
use std::cmp;
use super::error::Error;
use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;

#[derive(Eq, PartialEq, Debug)]
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
pub const COMPRESS_POINTER_MARK8: u8  = 0xc0;
pub const COMPRESS_POINTER_MARK16: u16  = 0xc000;

#[derive(Debug, Clone)]
pub struct Name {
    raw: Vec<u8>,
    offsets: Vec<u8>,
    length: u8,
    label_count: u8,
}

pub fn root() -> Name {
    Name {
        length: 1,
        label_count: 1,
        raw: vec![0],
        offsets: vec![0],
    }
}

pub struct NameComparisonResult {
    pub order: i8,
    pub common_label_count: u8,
    pub relation: NameRelation,
}

#[derive(Eq, PartialEq)]
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

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

static DIGITAL_VALUES: &'static [i8] = &[
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 16
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 32
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 48
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, -1, -1, -1, -1, -1, // 64
    -1, 10, 11, 12, 13, 14, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 80
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 96
    -1, 10, 11, 12, 13, 14, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 112
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, // 128
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1
];

#[inline]
fn digitvalue(c: usize) -> i8 {
    DIGITAL_VALUES[c]
}

pub static MAP_TO_LOWER: &'static [u8] =
&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c,
0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b,
0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a,
0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
0x7a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86,
0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95,
0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4,
0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb0, 0xb1, 0xb2, 0xb3,
0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xc0, 0xc1, 0xc2,
0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf, 0xd0, 0xd1,
0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf, 0xe0,
0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe,
0xff];

#[inline]
fn lower_caes(c: usize) -> u8 {
    MAP_TO_LOWER[c]
}

fn string_parse(name_raw: &[u8],
                start_pos: usize,
                end: usize,
                downcase: bool)
-> Result<(Vec<u8>, Vec<u8>), &'static str> {
    let mut start = start_pos;
    let mut data: Vec<u8> = Vec::with_capacity(end - start + 1);
    let mut offsets: Vec<u8> = Vec::new();
    let mut count = 0;
    let mut digits = 0;
    let mut value: i32 = 0;
    let mut done = false;
    let mut is_root = false;
    let mut state = FtStat::Init;
    let mut next_u8 = true;
    let mut c: char = 0 as char;

    offsets.push(0);
    while data.len() < MAX_WIRE_LEN && start != end && done == false {
        if next_u8 {
            c = name_raw[start] as char;
            start += 1;
        }

        if state == FtStat::Init {
            if c == '.' {
                if start != end {
                    return Err("non terminating empty label");
                } else {
                    is_root = true;
                }
            } else if c == '@' && start == end {
                is_root = true;
            }

            if is_root {
                data.push(0);
                done = true;
                break;
            }
            state = FtStat::Start;
            next_u8 = false;
        } else if state == FtStat::Start {
            data.push(0);
            count = 0;
            if c == '\\' {
                state = FtStat::Initialescape;
                break;
            }
            state = FtStat::Ordinary;
            next_u8 = false;
        } else if state == FtStat::Ordinary {
            if c == '.' {
                if count == 0 {
                    return Err("duplicate period");
                }
                data[offsets[offsets.len() - 1] as usize] = count;
                offsets.push(data.len() as u8);
                if start == end {
                    data.push(0);
                    done = true;
                }
                state = FtStat::Start;
            } else if c == '\\' {
                state = FtStat::Escape;
            } else {
                count += 1;
                if count > MAX_LABEL_LEN {
                    return Err("too long label");
                }
                if downcase {
                    data.push(lower_caes(c as usize));
                } else {
                    data.push(c as u8);
                }
            }
            next_u8 = true;
        } else if state == FtStat::Initialescape {
            if c == '[' {
                return Err("invalid label type");
            }
            state = FtStat::Escape;
            next_u8 = false;
        } else if state == FtStat::Escape {
            if is_digit(c) == false {
                count += 1;
                if count > MAX_LABEL_LEN {
                    return Err("too long label");
                }
                if downcase {
                    data.push(lower_caes(c as usize));
                } else {
                    data.push(c as u8);
                }
                state = FtStat::Ordinary;
                break;
            }
            digits = 0;
            value = 0;
            state = FtStat::Escdecimal;
            next_u8 = false;
        } else if state == FtStat::Escdecimal {
            if is_digit(c) == false {
                return Err("mixture of escaped digit and non-digit");
            }
            value = value * 10;
            value = value + digitvalue(c as usize) as i32;
            digits += 1;
            if digits == 3 {
                if value > 255 {
                    return Err("escaped decimal is to larg");
                }
                count += 1;
                if count > MAX_LABEL_LEN {
                    return Err("lable is too long");
                }
                if downcase {
                    data.push(lower_caes(c as usize));
                } else {
                    data.push(c as u8);
                }
                state = FtStat::Ordinary;
            }
            next_u8 = true;
        } else {
            panic!("impossible state");
        }
    }

    if done == false {
        if data.len() == MAX_WIRE_LEN {
            return Err("too long name");
        }
        if start != end {
            panic!("start should equal to end");
        }
        if state != FtStat::Ordinary {
            return Err("incomplete textural name");
        } else {
            if count == 0 {
                panic!("count shouldn't equal to zero");
            }
            data[offsets[offsets.len() - 1] as usize] = count as u8;
            offsets.push(data.len() as u8);
            data.push(0);
        }
    }

    Ok((data, offsets))
}

    impl Name {
        pub fn new(name: &str, downcase: bool) -> Result<Name, &'static str> {
            let name_len = name.len();
            match string_parse(name.as_bytes(), 0, name_len, downcase) {
                Ok((data, offsets)) => {
                    Ok(Name {
                        length: data.len() as u8,
                        label_count: offsets.len() as u8,
                        raw: data,
                        offsets: offsets,
                    })
                }
                Err(e) => Err(e),
            }
        }

        pub fn from_wire(buf: &mut InputBuffer, downcase: bool) -> Result<Self, Error> {
            let mut n: usize= 0;
            let mut nused: usize = 0;
            let mut cused: usize = 0;
            let mut done = false;
            let mut data: Vec<u8> = Vec::with_capacity(15);
            let mut offsets: Vec<u8> = Vec::with_capacity(5);
            let mut seen_pointer: bool = false;
            let mut state = FwStat::Start;
            let mut current = buf.postion() as usize;
            let pos_beg = current;
            let mut biggest_pointer = current;
            let mut new_current: usize = 0;

            while current < buf.len() && done == false {
                let mut c = try!(buf.read_u8());
                current += 1;
                if seen_pointer == false {
                    cused += 1;
                }

                if state == FwStat::Start {
                    if c <= MAX_LABEL_LEN {
                        offsets.push(nused as u8);
                        if nused + (c as usize) + 1 > MAX_WIRE_LEN {
                            return Err(Error::TooLongName);
                        }

                        nused += (c as usize) + 1;
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
                        return Err(Error::InvalidLabelCharacter);
                    }
                } else if state == FwStat::Ordinary {
                    if downcase {
                        c = MAP_TO_LOWER[c as usize];
                    }
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
                    if new_current >= biggest_pointer {
                        return Err(Error::BadCompressPointer);
                    }
                    biggest_pointer = new_current;
                    current = new_current;
                    buf.set_position(current);
                    seen_pointer = true;
                    state = FwStat::Start;
                }
            }

            if done == false {
                return Err(Error::InCompleteName);
            }

            buf.set_position(pos_beg + cused);
            Ok(Name {
                length: data.len() as u8,
                label_count: offsets.len() as u8,
                raw: data,
                offsets: offsets,
            })
        }

        pub fn len(&self) -> usize {
            self.length as usize
        }

        pub fn label_count(&self) -> usize {
            return self.label_count as usize;
        }

        pub fn to_wire(&self, buf: &mut OutputBuffer) {
            buf.write_bytes(&self.raw);
        }

        pub fn rend(&self, render: &mut MessageRender) {
            render.write_name(self, true);
        }

        pub fn to_string(&self) -> String {
            let mut buf = Vec::with_capacity(self.len());
            let special_char: Vec<u8> = vec![0x22, 0x28, 0x29, 0x2E, 0x3B, 0x5C, 0x40, 0x24]; //" ( ) . ; \\ @ $
            let mut i = 0;
            while i < self.length {
                let mut count = self.raw[i as usize];
                i += 1;

                if count == 0 {
                    buf.push('.' as u8);
                    break;
                }

                if buf.len() != 0 {
                    buf.push('.' as u8);
                }

                while count > 0 {
                    count -= 1;
                    let c: u8 = self.raw[i as usize];
                    i += 1;
                    if special_char.contains(&c) {
                        buf.push('\\' as u8);
                        buf.push(c);
                    } else {
                        if c > 0x20 && c < 0x7f {
                            buf.push(c);
                        } else {
                            buf.push(0x5c);
                            buf.push(0x30 + ((c / 100) % 10));
                            buf.push(0x30 + ((c / 10) % 10));
                            buf.push(0x30 + (c % 10));
                        }
                    }
                }
            }

            unsafe {
                String::from_utf8_unchecked(buf)
            }
        }


        pub fn get_relation(&self, other: &Name, case_sensitive: bool) -> NameComparisonResult {
            let mut l1 = self.label_count;
            let mut l2 = other.label_count;
            let ldiff = (l1 as i8) - (l2 as i8);
            let mut minl = cmp::min(l1, l2);
            let mut nlabels = 0;

            while minl > 0 {
                minl -= 1;
                l1 -= 1;
                l2 -= 1;
                let mut ps1 = self.offsets[l1 as usize];
                let mut ps2 = other.offsets[l2 as usize];
                let c1 = self.raw[ps1 as usize];
                let c2 = other.raw[ps2 as usize];
                ps1 += 1;
                ps2 += 1;

                let cdiff = (c1 as i8) - (c2 as i8);
                let mut mincount = cmp::min(c1, c2);

                while mincount > 0 {
                    let label1 = self.raw[ps1 as usize];
                    let label2 = other.raw[ps2 as usize];
                    let chdiff = if case_sensitive {
                        (label1 as i8) - (label2 as i8)
                    } else {
                        (lower_caes(label1 as usize) as i8) - (lower_caes(label2 as usize) as i8)
                    };

                    if chdiff != 0 {
                        if nlabels < 2 {
                            return NameComparisonResult {
                                order: chdiff,
                                common_label_count: nlabels,
                                relation: NameRelation::None,
                            };
                        } else {
                            return NameComparisonResult {
                                order: chdiff,
                                common_label_count: nlabels,
                                relation: NameRelation::CommonAncestor,
                            };
                        }
                    }
                    mincount -= 1;
                    ps1 += 1;
                    ps2 += 1;
                }

                if cdiff != 0 {
                    if nlabels == 0 {
                        return NameComparisonResult {
                            order: cdiff,
                            common_label_count: nlabels,
                            relation: NameRelation::None,
                        };
                    } else {
                        return NameComparisonResult {
                            order: cdiff,
                            common_label_count: nlabels,
                            relation: NameRelation::CommonAncestor,
                        };
                    }
                }
                nlabels += 1;
            }

            if ldiff < 0 {
                return NameComparisonResult {
                    order: ldiff,
                    common_label_count: nlabels,
                    relation: NameRelation::SuperDomain,
                };
            } else if ldiff > 0 {
                return NameComparisonResult {
                    order: ldiff,
                    common_label_count: nlabels,
                    relation: NameRelation::SubDomain,
                };
            } else {
                return NameComparisonResult {
                    order: ldiff,
                    common_label_count: nlabels,
                    relation: NameRelation::Equal,
                };
            }
        }

        pub fn concat_all(&self, suffixes: &[&Name]) -> Result<Name, &'static str> {
            let mut final_length = self.length;
            let mut final_label_count = self.label_count;
            let suffix_count = suffixes.len();
            for suffix in suffixes {
                final_length += suffix.length - 1;
                final_label_count += suffix.label_count - 1;
            }

            if (final_length as usize) > MAX_WIRE_LEN {
                return Err("names are too long to concat");
            } else if final_label_count > MAX_LABEL_COUNT {
                return Err("names has too many labels to concat");
            }

            let mut raw = Vec::with_capacity(final_length as usize);
            raw.extend_from_slice(&self.raw[..(self.length as usize - 1)]);
            for suffix in &suffixes[..(suffix_count as usize - 1)] {
                raw.extend_from_slice(&suffix.raw[..(suffix.length as usize - 1)])
            }
            raw.extend_from_slice(&(suffixes[suffix_count - 1].raw[..]));

            let mut offsets = Vec::with_capacity(final_label_count as usize);
            offsets.extend_from_slice(&self.offsets[..]);
            let mut copied_len = self.label_count;
            for suffix in suffixes {
                let last_offset = offsets[copied_len as usize - 1];
                offsets.extend_from_slice(&suffix.offsets[1..(suffix.label_count as usize)]);
                for i in copied_len .. (copied_len + suffix.label_count - 1) {
                    offsets[i as usize] += last_offset as u8
                }
                copied_len += suffix.label_count - 1;
            }

            Ok(Name {
                raw: raw,
                offsets: offsets,
                length: final_length,
                label_count: final_label_count,
            })
        }


        pub fn concat(&self, suffix: &Name) -> Result<Name, &'static str> {
            return self.concat_all(&[suffix])
        }

        pub fn reverse(&self) -> Name {
            if self.label_count == 1 {
                return self.clone();
            }

            let mut raw = Vec::with_capacity(self.length as usize);
            let mut offsets = Vec::with_capacity(self.label_count as usize);
            let mut label_len = 0;
            let mut i = (self.label_count - 2) as i8;
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
            Name {
                length: self.length,
                label_count: self.label_count,
                raw: raw,
                offsets: offsets,
            }
        }


        pub fn split(&self, start_label: usize, label_count: usize) -> Result<Name, &'static str> {
            if label_count == 0 || label_count > (self.label_count as usize) ||
                start_label + label_count > (self.label_count as usize) {
                    return Err("split range isn't valid");
                }

            if start_label + label_count == (self.label_count as usize) {
                let mut offsets = Vec::with_capacity(label_count);
                let first_offset = self.offsets[start_label];
                offsets.extend_from_slice(&self.offsets[start_label..]);
                let start_pos = offsets[0] as usize;
                let mut raw = Vec::with_capacity(self.length as usize - start_pos);
                raw.extend_from_slice(&self.raw[start_pos..]);
                for i in 0..label_count {
                    offsets[i] -= first_offset;
                }
                Ok(Name {
                    length: self.length - (start_pos as u8),
                    label_count: label_count as u8,
                    raw: raw,
                    offsets: offsets,
                })
            } else {
                let mut offsets = Vec::with_capacity(label_count + 1);
                offsets.extend_from_slice(&self.offsets[start_label..start_label + label_count + 1]);
                let mut raw = Vec::with_capacity((offsets[label_count] - offsets[0] + 1) as usize);
                raw.extend_from_slice(&self.raw[(offsets[0] as usize)..
                                      (offsets[label_count] as usize)]);

                let first_offset = self.offsets[start_label];
                for i in 0..(label_count + 1) {
                    offsets[i] -= first_offset;
                }
                raw.push(0);
                Ok(Name {
                    length: (raw.len() as u8),
                    label_count: (label_count as u8) + 1,
                    raw: raw,
                    offsets: offsets,
                })
            }
        }

        pub fn parent(&self, level: usize) -> Result<Name, &'static str> {
            self.split(level, self.label_count as usize - level)
        }

        pub fn downcase(&mut self) {
            let mut label_count = self.label_count;
            let mut p: usize = 0;
            while label_count > 0 {
                label_count -= 1;
                let mut label_len = self.raw[p];
                p += 1;
                while label_len > 0 {
                    self.raw[p] = lower_caes(self.raw[p] as usize);
                    p += 1;
                    label_len -= 1;
                }
            }
        }

        pub fn strip_left(&self, label_count: usize) -> Result<Name, &'static str> {
            if label_count >= (self.label_count as usize) {
                return Err("strip too many labels");
            }

            if label_count == 0 {
                return Ok(self.clone());
            }

            let new_label_count = (self.label_count as usize) - label_count;
            let mut offsets = Vec::with_capacity(new_label_count);
            offsets.extend_from_slice(&self.offsets[label_count..]);
            let start_pos = self.offsets[label_count] as usize;
            for i in 0..new_label_count {
                offsets[i] -= start_pos as u8;
            }
            let new_length = self.length as usize - start_pos;
            let mut raw = Vec::with_capacity(new_length);
            raw.extend_from_slice(&self.raw[start_pos..]);
            Ok(Name {
                length: new_length as u8,
                label_count: new_label_count as u8,
                raw: raw,
                offsets: offsets,
            })
        }

        pub fn clone(&self) -> Name {
            return Name {
                length: self.length,
                label_count: self.label_count,
                raw: self.raw.clone(),
                offsets: self.offsets.clone(),
            };
        }

        pub fn strip_right(&self, label_count: usize) -> Result<Name, &'static str> {
            if label_count >= self.label_count as usize {
                return Err("strip too many labels");
            }

            if label_count == 0 {
                return Ok(self.clone());
            }

            let new_label_count = self.label_count as usize - label_count;
            let end_label = new_label_count - 1;
            let end_pos = self.offsets[end_label] as usize;
            let mut raw = Vec::with_capacity(end_pos + 1);
            raw.extend_from_slice(&self.raw[0..end_pos + 1]);
            raw[end_pos] = 0;

            let mut offsets = Vec::with_capacity(new_label_count);
            offsets.extend_from_slice(&self.offsets[0..end_label + 1]);
            Ok(Name {
                length: end_pos as u8 + 1,
                label_count: new_label_count as u8,
                raw: raw,
                offsets: offsets,
            })
        }

        pub fn hash(&self, case_sensitive: bool) -> u32 {
            let mut hash: u32 = 0;
            let seed: u32 = 0x9e3779b9;
            if case_sensitive {
                for i in 0..(self.length as usize) {
                    hash ^= (self.raw[i] as u32)
                        .wrapping_add(seed)
                        .wrapping_add(hash << 6)
                        .wrapping_add(hash >> 2);
                }
            } else {
                for i in 0..(self.length as usize) {
                    hash ^= (lower_caes(self.raw[i] as usize) as u32)
                        .wrapping_add(seed)
                        .wrapping_add(hash << 6)
                        .wrapping_add(hash >> 2);
                }
            }
            hash
        }

        pub fn is_subdomain(&self, parent: &Name) -> bool {
            if self.length < parent.length || self.label_count < parent.label_count {
                return false;
            }

            let mut i = self.length - 1;
            let mut j = parent.length - 1;
            while j > 0 {
                if lower_caes(parent.raw[j as usize] as usize) != lower_caes(self.raw[i as usize] as usize) {
                    return false;
                }
                j -= 1;
                i -= 1;
            }
            return true;
        }

        pub fn raw_data(&self) -> &[u8] {
            self.raw.as_slice()
        }

        pub fn offsets(&self) -> &[u8] {
            self.offsets.as_slice()
        }
    }


    impl fmt::Display for Name {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.to_string())
        }
    }

    impl cmp::PartialEq for Name {
        fn eq(&self, other: &Name) -> bool {
            if self.length != other.length || self.label_count != other.label_count {
                return false;
            }

            let mut pos = 0;
            let mut l = self.label_count;
            while l > 0 {
                let mut count = self.raw[pos as usize];
                if count != other.raw[pos as usize] {
                    return false;
                }

                pos += 1;

                while count > 0 {
                    count -= 1;
                    if lower_caes(self.raw[pos as usize] as usize) !=
                        lower_caes(other.raw[pos as usize] as usize) {
                            return false;
                        }
                    pos += 1;
                }
                l -= 1;
            }

            true
        }
    }

    impl cmp::Eq for Name {}


    impl cmp::PartialOrd for Name {
        fn partial_cmp(&self, other: &Name) -> Option<cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl cmp::Ord for Name {
        fn cmp(&self, other: &Name) -> cmp::Ordering {
            let relation = self.get_relation(other, false);
            if relation.order < 0 {
                cmp::Ordering::Less
            } else if relation.order > 0 {
                cmp::Ordering::Greater
            } else {
                cmp::Ordering::Equal
            }
        }
    }


    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_name_concat() {
            let www_knet_cn = Name::new("www.knet.Cn", true).unwrap();
            let www_knet = Name::new("www.knet", true).unwrap();
            let cn = Name::new("cn", true).unwrap();

            let relation = www_knet_cn.get_relation(&www_knet.concat(&cn).unwrap(), false);
            assert_eq!(relation.order, 0);
            assert_eq!(relation.common_label_count, 4);
            assert_eq!(relation.relation, NameRelation::Equal);

            assert_eq!(www_knet_cn.reverse().to_string(),
            "cn.knet.www.".to_string());

            assert_eq!(www_knet_cn.split(0, 1).unwrap().to_string(),
            "www.".to_string());
            assert_eq!(www_knet_cn.split(0, 4).unwrap().to_string(),
            "www.knet.cn.".to_string());
            assert_eq!(www_knet_cn.split(1, 3).unwrap().to_string(),
            "knet.cn.".to_string());
            assert_eq!(www_knet_cn.split(1, 2).unwrap().to_string(),
            "knet.cn.".to_string());

            assert_eq!(www_knet_cn.parent(0).unwrap().to_string(),
            "www.knet.cn.".to_string());
            assert_eq!(www_knet_cn.parent(1).unwrap().to_string(),
            "knet.cn.".to_string());
            assert_eq!(www_knet_cn.parent(2).unwrap().to_string(),
            "cn.".to_string());
            assert_eq!(www_knet_cn.parent(3).unwrap().to_string(), ".".to_string());
            assert!(www_knet_cn.parent(4).is_err())
        }

        #[test]
        fn test_name_compare() {
            let www_knet_cn_mix_case = Name::new("www.KNET.cN", false).unwrap();
            let www_knet_cn = Name::new("www.knet.cn.", true).unwrap();
            let relation = www_knet_cn.get_relation(&www_knet_cn_mix_case, false);
            assert_eq!(relation.order, 0);
            assert_eq!(relation.common_label_count, 4);
            assert_eq!(relation.relation, NameRelation::Equal);

            let relation = www_knet_cn.get_relation(&www_knet_cn_mix_case, true);
            assert!(relation.order > 0);
            assert_eq!(relation.common_label_count, 1);
            assert_eq!(relation.relation, NameRelation::None);

            let www_knet_com = Name::new("www.knet.com", true).unwrap();
            let relation = www_knet_cn.get_relation(&www_knet_com, false);
            assert!(relation.order < 0);
            assert_eq!(relation.common_label_count, 1);
            assert_eq!(relation.relation, NameRelation::None);

            let baidu_com = Name::new("baidu.com.", true).unwrap();
            let www_baidu_com = Name::new("www.baidu.com", true).unwrap();
            let relation = baidu_com.get_relation(&www_baidu_com, false);
            assert!(relation.order < 0);
            assert_eq!(relation.common_label_count, 3);
            assert_eq!(relation.relation, NameRelation::SuperDomain);
        }

        #[test]
        fn test_name_strip() {
            let www_knet_cn_mix_case = Name::new("www.KNET.cN", true).unwrap();
            assert_eq!(&www_knet_cn_mix_case.strip_left(1).unwrap().to_string(),
            "knet.cn.");
            assert_eq!(&www_knet_cn_mix_case.strip_left(2).unwrap().to_string(),
            "cn.");
            assert_eq!(&www_knet_cn_mix_case.strip_left(3).unwrap().to_string(),
            ".");
            assert_eq!(&www_knet_cn_mix_case.strip_right(1).unwrap().to_string(),
            "www.knet.");
            assert_eq!(&www_knet_cn_mix_case.strip_right(2).unwrap().to_string(),
            "www.");
            assert_eq!(&www_knet_cn_mix_case.strip_right(3).unwrap().to_string(),
            ".");
        }

        #[test]
        fn test_name_hash() {
            let name1 = Name::new("wwwnnnnnnnnnnnnn.KNET.cNNNNNNNNN", false).unwrap();
            let name2 = Name::new("wwwnnnnnnnnnnnnn.KNET.cNNNNNNNNn", false).unwrap();
            let name3 = Name::new("wwwnnnnnnnnnnnnn.KNET.cNNNNNNNNN.baidu.com.cn.net",
                                  false).unwrap();
            assert_eq!(&name1.hash(false), &name2.hash(false));
            assert_ne!(&name1.hash(false), &name3.hash(false));
        }

        #[test]
        fn test_name_is_subdomain() {
            let www_knet_cn = Name::new("www.knet.Cn", false ).unwrap();
            let www_knet = Name::new("www.knet", false ).unwrap();
            let knet_cn = Name::new("knet.Cn", false ).unwrap();
            let cn = Name::new("cn", false ).unwrap();
            let knet = Name::new("kNet", false ).unwrap();
            let root = root();
            assert!(www_knet_cn.is_subdomain(&knet_cn) &&
                    knet_cn.is_subdomain(&cn) &&
                    knet_cn.is_subdomain(&root) &&
                    cn.is_subdomain(&root) &&
                    knet.is_subdomain(&root) &&
                    www_knet_cn.is_subdomain(&root) &&
                    www_knet.is_subdomain(&root) &&
                    root.is_subdomain(&root));
            assert!(knet.is_subdomain(&knet_cn) == false &&
                    knet.is_subdomain(&cn) == false &&
                    root.is_subdomain(&cn) == false &&
                    www_knet.is_subdomain(&www_knet_cn) == false);
        }
    }
