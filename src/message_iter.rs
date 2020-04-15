use std::iter::FusedIterator;

use crate::message::{Message, SectionType};
use crate::rrset::RRset;

pub struct MessageIter<'a> {
    msg: &'a Message,
    section: SectionType,
    index: usize,
    answer_rrset_count: usize,
    authority_rrset_count: usize,
    additional_rrset_count: usize,
}

impl<'a> MessageIter<'a> {
    pub fn new(msg: &'a Message) -> Self {
        MessageIter {
            msg,
            section: SectionType::Answer,
            index: 0,
            answer_rrset_count: msg
                .section(SectionType::Answer)
                .map_or(0, |rrsets| rrsets.len()),
            authority_rrset_count: msg
                .section(SectionType::Answer)
                .map_or(0, |rrsets| rrsets.len()),
            additional_rrset_count: msg
                .section(SectionType::Answer)
                .map_or(0, |rrsets| rrsets.len()),
        }
    }
}

impl<'a> Iterator for MessageIter<'a> {
    type Item = (&'a RRset, SectionType);

    fn next(&mut self) -> Option<Self::Item> {
        match self.section {
            SectionType::Answer => {
                if self.index == self.answer_rrset_count {
                    self.index = 0;
                    self.section = SectionType::Authority;
                    return self.next();
                } else {
                    let rrset = &self.msg.section(self.section).unwrap()[self.index];
                    self.index += 1;
                    return Some((rrset, SectionType::Answer));
                }
            }
            SectionType::Authority => {
                if self.index == self.authority_rrset_count {
                    self.index = 0;
                    self.section = SectionType::Additional;
                    return self.next();
                } else {
                    let rrset = &self.msg.section(self.section).unwrap()[self.index];
                    self.index += 1;
                    return Some((rrset, SectionType::Authority));
                }
            }
            SectionType::Additional => {
                if self.index == self.additional_rrset_count {
                    return None;
                } else {
                    let rrset = &self.msg.section(self.section).unwrap()[self.index];
                    self.index += 1;
                    return Some((rrset, SectionType::Additional));
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remain = match self.section {
            SectionType::Answer => {
                self.answer_rrset_count - self.index
                    + self.authority_rrset_count
                    + self.additional_rrset_count
            }
            SectionType::Authority => {
                self.authority_rrset_count - self.index + self.additional_rrset_count
            }
            SectionType::Additional => self.additional_rrset_count - self.index,
        };
        (remain, Some(remain))
    }
}

impl<'a> ExactSizeIterator for MessageIter<'a> {}
impl<'a> FusedIterator for MessageIter<'a> {}
