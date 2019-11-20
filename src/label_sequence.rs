use crate::error::DNSError;
use crate::label_slice::LabelSlice;
use crate::name::{self, string_parse, Name};
use failure::{self, Result};
use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt,
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct LabelSequence {
    data: Vec<u8>,
    offsets: Vec<u8>,
}

impl LabelSequence {
    pub fn new(data: Vec<u8>, offsets: Vec<u8>) -> LabelSequence {
        LabelSequence { data, offsets }
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn offsets(&self) -> &[u8] {
        self.offsets.as_slice()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn equals(&self, other: &LabelSequence, case_sensitive: bool) -> bool {
        if self.len() != other.len() {
            false
        } else if case_sensitive {
            self.data() == other.data()
        } else {
            self.data().eq_ignore_ascii_case(other.data())
        }
    }

    pub fn label_count(&self) -> usize {
        self.offsets.len()
    }

    pub fn split(&mut self, start_label: usize, label_count: usize) -> Result<LabelSequence> {
        let max_label_count = self.label_count() as usize;
        if start_label >= max_label_count || label_count == 0 {
            return Err(DNSError::InvalidLabelIndex.into());
        }

        let mut label_count = label_count;
        if start_label + label_count > max_label_count {
            label_count = max_label_count - start_label;
        }

        let last_label = start_label + label_count - 1;
        let last_label_len: u8 = self.data[usize::from(self.offsets[last_label])] + 1;
        let data_length: u8 = self.offsets[last_label] + last_label_len;
        let data_offset: u8 = data_length - self.offsets[start_label];
        let data: Vec<u8> = self
            .data
            .drain(self.offsets[start_label] as usize..data_length as usize)
            .collect();
        let mut offsets: Vec<u8> = self.offsets.drain(start_label..=last_label).collect();

        if start_label == 0 {
            for v in &mut self.offsets {
                *v -= data_offset;
            }
        } else {
            for (i, v) in (&mut self.offsets).iter_mut().enumerate() {
                if i >= start_label {
                    *v -= data_offset;
                }
            }
            let curr_label_value = offsets[0];
            for v in &mut offsets {
                *v -= curr_label_value;
            }
        }

        Ok(LabelSequence { data, offsets })
    }

    pub fn concat_all(&self, suffixes: &[&LabelSequence]) -> Result<Name> {
        if self.is_absolute() {
            if suffixes.is_empty() {
                return Ok(Name::from_raw(self.data.clone(), self.offsets.clone()));
            } else {
                return Err(DNSError::InvalidLabelSequnceConcatParam.into());
            }
        }

        let mut middle_seq_is_absolute = false;
        let mut last_seq_is_absolute = true;
        let suffixe_count = suffixes.len();
        let (final_length, final_label_count) = suffixes.iter().enumerate().fold(
            (self.len(), self.label_count()),
            |(mut len, mut label_count), (index, seq)| {
                if index != suffixe_count - 1 {
                    if seq.is_absolute() {
                        middle_seq_is_absolute = true;
                    }
                } else if !seq.is_absolute() {
                    last_seq_is_absolute = false;
                }
                len += seq.len();
                label_count += seq.label_count();
                (len, label_count)
            },
        );
        if middle_seq_is_absolute || !last_seq_is_absolute {
            return Err(DNSError::InvalidLabelSequnceConcatParam.into());
        }
        if final_length > name::MAX_WIRE_LEN {
            return Err(DNSError::TooLongName.into());
        } else if final_label_count > name::MAX_LABEL_COUNT as usize {
            return Err(DNSError::TooLongLabel.into());
        }

        let mut data = Vec::with_capacity(final_length as usize);
        data.extend_from_slice(self.data.as_ref());
        suffixes
            .iter()
            .for_each(|suffix| data.extend_from_slice(suffix.data.as_ref()));

        let mut offsets = Vec::with_capacity(final_label_count as usize);
        let mut offset_pos: usize = 0;
        for _ in 0..final_label_count {
            offsets.push(offset_pos as u8);
            offset_pos = offset_pos + data[offset_pos] as usize + 1;
        }

        Ok(Name::from_raw(data, offsets))
    }

    pub fn is_absolute(&self) -> bool {
        self.data[self.data.len() - 1] == 0
    }
}

impl PartialEq for LabelSequence {
    fn eq(&self, other: &LabelSequence) -> bool {
        self.equals(other, false)
    }
}

impl Eq for LabelSequence {}

impl PartialOrd for LabelSequence {
    fn partial_cmp(&self, other: &LabelSequence) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LabelSequence {
    fn cmp(&self, other: &LabelSequence) -> Ordering {
        let self_slice = LabelSlice::from_label_sequence(self);
        let other_slice = LabelSlice::from_label_sequence(other);
        let result = self_slice.compare(&other_slice, false);
        if result.order < 0 {
            Ordering::Less
        } else if result.order > 0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl fmt::Display for LabelSequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", LabelSlice::from_label_sequence(self))
    }
}

impl FromStr for LabelSequence {
    type Err = failure::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let len = s.len();
        match string_parse(s.as_bytes(), 0, len, false) {
            Ok((data, offsets)) => Ok(LabelSequence { data, offsets }),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::LabelSequence;
    use crate::name::Name;
    use std::str::FromStr;

    #[test]
    fn test_label_sequence_split() {
        let www_google_com_cn = Name::new("www.google.com.cn.").unwrap();
        let mut www_google_com_cn = www_google_com_cn.into_label_sequence(0, 4);
        let google_com_cn = www_google_com_cn.split(1, 4).unwrap();
        assert_eq!(www_google_com_cn.data(), [3, 119, 119, 119]);
        assert_eq!(www_google_com_cn.offsets(), [0]);
        assert_eq!(
            google_com_cn.data(),
            [6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 2, 99, 110, 0]
        );
        assert_eq!(google_com_cn.offsets(), [0, 7, 11, 14]);
    }

    #[test]
    fn test_label_sequence_concat_all() {
        assert_eq!(
            LabelSequence::from_str("a.b.c.")
                .unwrap()
                .concat_all(&[])
                .unwrap(),
            Name::from_str("a.b.c").unwrap()
        );

        assert_eq!(
            LabelSequence::from_str("a")
                .unwrap()
                .concat_all(&[
                    &LabelSequence::from_str("b").unwrap(),
                    &LabelSequence::from_str("c.").unwrap(),
                ])
                .unwrap(),
            Name::from_str("a.b.c").unwrap()
        );

        assert_eq!(
            LabelSequence::from_str("a")
                .unwrap()
                .concat_all(&[&LabelSequence::from_str(".").unwrap(),])
                .unwrap(),
            Name::from_str("a").unwrap()
        );

        assert_eq!(
            LabelSequence::from_str(".")
                .unwrap()
                .concat_all(&[])
                .unwrap(),
            Name::from_str(".").unwrap()
        );

        assert!(LabelSequence::from_str("a.")
            .unwrap()
            .concat_all(&[
                &LabelSequence::from_str("b").unwrap(),
                &LabelSequence::from_str("c.").unwrap(),
            ])
            .is_err());

        assert!(LabelSequence::from_str("a")
            .unwrap()
            .concat_all(&[
                &LabelSequence::from_str("b.").unwrap(),
                &LabelSequence::from_str("c.").unwrap(),
            ])
            .is_err());

        assert!(LabelSequence::from_str("a")
            .unwrap()
            .concat_all(&[
                &LabelSequence::from_str("b").unwrap(),
                &LabelSequence::from_str("c").unwrap(),
            ])
            .is_err());
    }
}
