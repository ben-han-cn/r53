use std::iter::FusedIterator;

use crate::message::{Message, SectionType, ALL_SECTIONS};
use crate::rrset::RRset;

pub struct MessageIter<'a> {
    index: usize,
    back_index: usize,
    answer_rrset_count: usize,
    authority_rrset_count: usize,
    additional_rrset_count: usize,
    rrsets: Vec<&'a RRset>,
}

impl<'a> MessageIter<'a> {
    pub fn new(msg: &'a Message) -> Self {
        let answer_rrset_count = msg.section_rrset_count(SectionType::Answer);
        let authority_rrset_count = msg.section_rrset_count(SectionType::Authority);
        let additional_rrset_count = msg.section_rrset_count(SectionType::Additional);
        let len = answer_rrset_count + additional_rrset_count + authority_rrset_count;
        let mut rrsets = Vec::with_capacity(len);
        if len > 0 {
            for typ in ALL_SECTIONS {
                if let Some(rrsets_) = msg.section(*typ) {
                    rrsets_.iter().for_each(|rrset| rrsets.push(rrset));
                }
            }
        }

        MessageIter {
            index: 0,
            back_index: len,
            answer_rrset_count,
            authority_rrset_count,
            additional_rrset_count,
            rrsets,
        }
    }
}

impl<'a> Iterator for MessageIter<'a> {
    type Item = (&'a RRset, SectionType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.back_index {
            return None;
        }

        let typ = if self.index < self.answer_rrset_count {
            SectionType::Answer
        } else if self.index < self.answer_rrset_count + self.authority_rrset_count {
            SectionType::Authority
        } else {
            SectionType::Additional
        };

        let item = (self.rrsets[self.index], typ);
        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remain =
            self.answer_rrset_count + self.authority_rrset_count + self.additional_rrset_count
                - self.index;
        (remain, Some(remain))
    }
}

impl<'a> DoubleEndedIterator for MessageIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == self.back_index {
            return None;
        }

        self.back_index -= 1;
        let typ = if self.back_index < self.answer_rrset_count {
            SectionType::Answer
        } else if self.index < self.answer_rrset_count + self.authority_rrset_count {
            SectionType::Authority
        } else {
            SectionType::Additional
        };
        let item = (self.rrsets[self.back_index], typ);
        Some(item)
    }
}

impl<'a> ExactSizeIterator for MessageIter<'a> {}
impl<'a> FusedIterator for MessageIter<'a> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Name, RRType};
    use std::str::FromStr;

    #[test]
    fn test_message_iterator() {
        let www_knet_cn_response = vec![
            4, 176, 132, 0, 0, 1, 0, 1, 0, 4, 0, 9, 3, 119, 119, 119, 4, 107, 110, 101, 116, 2, 99,
            110, 0, 0, 1, 0, 1, 192, 12, 0, 1, 0, 1, 0, 0, 1, 44, 0, 4, 202, 173, 11, 42, 192, 16,
            0, 2, 0, 1, 0, 0, 14, 16, 0, 20, 4, 118, 110, 115, 49, 9, 122, 100, 110, 115, 99, 108,
            111, 117, 100, 3, 98, 105, 122, 0, 192, 16, 0, 2, 0, 1, 0, 0, 14, 16, 0, 20, 4, 105,
            110, 115, 49, 9, 122, 100, 110, 115, 99, 108, 111, 117, 100, 3, 99, 111, 109, 0, 192,
            16, 0, 2, 0, 1, 0, 0, 14, 16, 0, 21, 4, 100, 110, 115, 49, 9, 122, 100, 110, 115, 99,
            108, 111, 117, 100, 4, 105, 110, 102, 111, 0, 192, 16, 0, 2, 0, 1, 0, 0, 14, 16, 0, 20,
            4, 99, 110, 115, 49, 9, 122, 100, 110, 115, 99, 108, 111, 117, 100, 3, 110, 101, 116,
            0, 192, 57, 0, 1, 0, 1, 0, 1, 81, 128, 0, 4, 203, 99, 22, 3, 192, 57, 0, 1, 0, 1, 0, 1,
            81, 128, 0, 4, 203, 99, 23, 3, 192, 89, 0, 1, 0, 1, 0, 0, 14, 16, 0, 4, 27, 221, 63, 3,
            192, 89, 0, 1, 0, 1, 0, 0, 14, 16, 0, 4, 119, 167, 244, 44, 192, 121, 0, 1, 0, 1, 0, 0,
            14, 16, 0, 4, 114, 67, 46, 13, 192, 121, 0, 1, 0, 1, 0, 0, 14, 16, 0, 4, 114, 67, 46,
            14, 192, 154, 0, 1, 0, 1, 0, 1, 81, 128, 0, 4, 42, 62, 2, 24, 192, 154, 0, 1, 0, 1, 0,
            1, 81, 128, 0, 4, 42, 62, 2, 29, 0, 0, 41, 16, 0, 0, 0, 0, 0, 0, 0,
        ];
        let msg = Message::from_wire(www_knet_cn_response.as_slice()).unwrap();

        let answer1 = "www.knet.cn.	300	IN	A	202.173.11.42";
        let ns = vec![
            "knet.cn.	3600	IN	NS	vns1.zdnscloud.biz.",
            "knet.cn.	3600	IN	NS	ins1.zdnscloud.com.",
            "knet.cn.	3600	IN	NS	dns1.zdnscloud.info.",
            "knet.cn.	3600	IN	NS	cns1.zdnscloud.net.",
        ];
        let additional1 = vec![
            "vns1.zdnscloud.biz.	86400	IN	A	203.99.22.3",
            "vns1.zdnscloud.biz.	86400	IN	A	203.99.23.3",
        ];

        let additional2 = vec![
            "ins1.zdnscloud.com.	3600	IN	A	27.221.63.3",
            "ins1.zdnscloud.com.	3600	IN	A	119.167.244.44",
        ];

        let additional3 = vec![
            "dns1.zdnscloud.info.	3600	IN	A	114.67.46.13",
            "dns1.zdnscloud.info.	3600	IN	A	114.67.46.14",
        ];

        let additional4 = vec![
            "cns1.zdnscloud.net.	86400	IN	A	42.62.2.24",
            "cns1.zdnscloud.net.	86400	IN	A	42.62.2.29",
        ];

        assert_eq!(msg.iter().len(), 6);
        msg.iter()
            .enumerate()
            .for_each(|(i, (rrset, section))| match i {
                0 => {
                    assert_eq!(section, SectionType::Answer);
                    assert_eq!(*rrset, RRset::from_str(answer1).unwrap());
                }
                1 => {
                    assert_eq!(section, SectionType::Authority);
                    assert_eq!(*rrset, RRset::from_strs(ns.as_slice()).unwrap());
                }
                2 => {
                    assert_eq!(section, SectionType::Additional);
                    assert_eq!(*rrset, RRset::from_strs(additional1.as_slice()).unwrap());
                }
                3 => {
                    assert_eq!(section, SectionType::Additional);
                    assert_eq!(*rrset, RRset::from_strs(additional2.as_slice()).unwrap());
                }
                4 => {
                    assert_eq!(section, SectionType::Additional);
                    assert_eq!(*rrset, RRset::from_strs(additional3.as_slice()).unwrap());
                }
                5 => {
                    assert_eq!(section, SectionType::Additional);
                    assert_eq!(*rrset, RRset::from_strs(additional4.as_slice()).unwrap());
                }
                _ => {
                    assert!(false);
                }
            });

        let mut iter = msg.iter();
        let rrset = RRset::from_str(answer1).unwrap();
        assert_eq!(iter.next(), Some((&rrset, SectionType::Answer)));
        let rrset = RRset::from_strs(ns.as_slice()).unwrap();
        assert_eq!(iter.next(), Some((&rrset, SectionType::Authority)));
        let rrset = RRset::from_strs(additional4.as_slice()).unwrap();
        assert_eq!(iter.next_back(), Some((&rrset, SectionType::Additional)));
        let rrset = RRset::from_strs(additional3.as_slice()).unwrap();
        assert_eq!(iter.next_back(), Some((&rrset, SectionType::Additional)));
        let rrset = RRset::from_strs(additional1.as_slice()).unwrap();
        assert_eq!(iter.next(), Some((&rrset, SectionType::Additional)));
        let rrset = RRset::from_strs(additional2.as_slice()).unwrap();
        assert_eq!(iter.next(), Some((&rrset, SectionType::Additional)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_empty_message_iterator() {
        let msg = Message::with_query(Name::new(".").unwrap(), RRType::A);
        let mut iter = msg.iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
}
