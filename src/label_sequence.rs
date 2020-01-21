use crate::label_slice::LabelSlice;
use crate::name::{self, string_parse, Name};
use anyhow::{self, bail, ensure, Result};
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
        ensure!(
            start_label < max_label_count && label_count > 0,
            "invalide label index"
        );

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
                bail!("concat label to absolute name");
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
        ensure!(
            !middle_seq_is_absolute && last_seq_is_absolute,
            "has absolute label in concat"
        );
        ensure!(
            final_length <= name::MAX_WIRE_LEN,
            "concat label generate too long name"
        );
        ensure!(
            final_label_count <= name::MAX_LABEL_COUNT as usize,
            "label count exceed limit"
        );

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
