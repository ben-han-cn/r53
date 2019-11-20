use crate::label_sequence::LabelSequence;
use crate::name::lower_case;
use crate::name::Name;
use crate::name::NameComparisonResult;
use crate::name::NameRelation;
use std::{cmp, fmt};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LabelSlice<'a> {
    data: &'a [u8],
    offsets: &'a [u8],
    first_label: usize,
    last_label: usize,
}

impl<'a> LabelSlice<'a> {
    pub fn from_name(name: &'a Name) -> LabelSlice {
        LabelSlice {
            data: name.raw_data(),
            offsets: name.offsets(),
            first_label: 0,
            last_label: name.label_count() - 1,
        }
    }

    pub fn from_label_sequence(ls: &'a LabelSequence) -> LabelSlice {
        LabelSlice {
            data: ls.data(),
            offsets: ls.offsets(),
            first_label: 0,
            last_label: ls.label_count() - 1,
        }
    }

    #[inline]
    pub fn offsets(&self) -> &'a [u8] {
        &self.offsets[self.first_label..=self.last_label]
    }

    #[inline]
    pub fn data(&self) -> &'a [u8] {
        let first_label_index: usize = usize::from(self.offsets[self.first_label]);
        &self.data[first_label_index..first_label_index + self.len()]
    }

    #[inline]
    pub fn len(&self) -> usize {
        let last_label_len: u8 = self.data[usize::from(self.offsets[self.last_label])] + 1;
        usize::from(self.offsets[self.last_label] - self.offsets[self.first_label] + last_label_len)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn first_label(&self) -> usize {
        self.first_label
    }

    #[inline]
    pub fn last_label(&self) -> usize {
        self.last_label
    }

    #[inline]
    pub fn label_count(&self) -> usize {
        self.last_label - self.first_label + 1
    }

    pub fn equals(&self, other: &LabelSlice, case_sensitive: bool) -> bool {
        if self.len() != other.len() {
            false
        } else if case_sensitive {
            self.data() == other.data()
        } else {
            self.data().eq_ignore_ascii_case(other.data())
        }
    }

    pub fn compare(&self, other: &LabelSlice, case_sensitive: bool) -> NameComparisonResult {
        let mut nlabels: usize = 0;
        let mut l1: usize = self.label_count();
        let mut l2: usize = other.label_count();
        let ldiff = (l1 as isize) - (l2 as isize);
        let mut l = cmp::min(l1, l2);

        while l > 0 {
            l -= 1;
            l1 -= 1;
            l2 -= 1;
            let mut pos1: usize = usize::from(self.offsets[l1 + self.first_label]);
            let mut pos2: usize = usize::from(other.offsets[l2 + other.first_label]);
            let count1: usize = usize::from(self.data[pos1]);
            let count2: usize = usize::from(other.data[pos2]);
            pos1 += 1;
            pos2 += 1;
            let cdiff: isize = (count1 as isize) - (count2 as isize);
            let mut count = cmp::min(count1, count2);

            while count > 0 {
                let mut label1: u8 = self.data[pos1];
                let mut label2: u8 = other.data[pos2];
                let chdiff: i8;
                if !case_sensitive {
                    label1 = lower_case(label1 as usize);
                    label2 = lower_case(label2 as usize);
                }
                chdiff = (label1) as i8 - (label2) as i8;
                if chdiff != 0 {
                    return NameComparisonResult {
                        order: chdiff,
                        common_label_count: nlabels as u8,
                        relation: if nlabels == 0 {
                            NameRelation::None
                        } else {
                            NameRelation::CommonAncestor
                        },
                    };
                }
                count -= 1;
                pos1 += 1;
                pos2 += 1;
            }
            if cdiff != 0 {
                return NameComparisonResult {
                    order: cdiff as i8,
                    common_label_count: nlabels as u8,
                    relation: if nlabels == 0 {
                        NameRelation::None
                    } else {
                        NameRelation::CommonAncestor
                    },
                };
            }
            nlabels += 1;
        }

        NameComparisonResult {
            order: ldiff as i8,
            common_label_count: nlabels as u8,
            relation: if ldiff < 0 {
                NameRelation::SuperDomain
            } else if ldiff > 0 {
                NameRelation::SubDomain
            } else {
                NameRelation::Equal
            },
        }
    }

    #[inline]
    pub fn strip_left(&mut self, index: usize) {
        assert!(index < self.label_count());
        self.first_label += index;
    }

    #[inline]
    pub fn strip_right(&mut self, index: usize) {
        assert!(index < self.label_count());
        self.last_label -= index;
    }
}

impl<'a> fmt::Display for LabelSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = Vec::with_capacity(self.len());
        let special_char: Vec<u8> = vec![0x22, 0x28, 0x29, 0x2E, 0x3B, 0x5C, 0x40, 0x24]; //" ( ) . ; \\ @ $
        let mut i = 0;
        let data = self.data();
        while i < self.len() {
            let mut count = data[i as usize];
            i += 1;

            if count == 0 {
                buf.push(b'.');
                break;
            }

            if !buf.is_empty() {
                buf.push(b'.');
            }

            while count > 0 {
                count -= 1;
                let c: u8 = data[i as usize];
                i += 1;
                if special_char.contains(&c) {
                    buf.push(b'\\');
                    buf.push(c);
                } else if c > 0x20 && c < 0x7f {
                    buf.push(c);
                } else {
                    buf.push(0x5c);
                    buf.push(0x30 + ((c / 100) % 10));
                    buf.push(0x30 + ((c / 10) % 10));
                    buf.push(0x30 + (c % 10));
                }
            }
        }

        write!(f, "{}", unsafe { String::from_utf8_unchecked(buf) })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::name::Name;
    #[test]
    fn test_label_slice_new() {
        //0377777705626169647503636f6d00
        let n1 = Name::new("www.baidu.com.").unwrap();
        let ls1 = LabelSlice::from_name(&n1);
        assert_eq!(
            ls1.data,
            [3, 119, 119, 119, 5, 98, 97, 105, 100, 117, 3, 99, 111, 109, 0]
        );
        assert_eq!(ls1.offsets, [0, 4, 10, 14]);
        assert_eq!(ls1.first_label, 0);
        assert_eq!(ls1.last_label, 3);

        let n2 = Name::new("www.baidu.coM.").unwrap();
        let ls2 = LabelSlice::from_name(&n2);
        assert_eq!(
            ls2.data,
            [3, 119, 119, 119, 5, 98, 97, 105, 100, 117, 3, 99, 111, 77, 0]
        );
        assert_eq!(ls2.offsets, [0, 4, 10, 14]);
        assert_eq!(ls2.first_label, 0);
        assert_eq!(ls2.last_label, 3);

        assert_eq!(
            ls1.data(),
            [3, 119, 119, 119, 5, 98, 97, 105, 100, 117, 3, 99, 111, 109, 0]
        );
        assert_eq!(
            ls2.data(),
            [3, 119, 119, 119, 5, 98, 97, 105, 100, 117, 3, 99, 111, 77, 0]
        );
        assert_eq!(ls1.equals(&ls2, false), true);
        assert_eq!(ls1.equals(&ls2, true), false);
        let grand_parent = Name::new("com").unwrap();
        let ls_grand_parent = LabelSlice::from_name(&grand_parent);
        let parent = Name::new("BaIdU.CoM").unwrap();
        let ls_parent = LabelSlice::from_name(&parent);
        let child = Name::new("wWw.bAiDu.cOm").unwrap();
        let mut ls_child = LabelSlice::from_name(&child);
        let brother = Name::new("AaA.bAiDu.cOm").unwrap();
        let ls_brother = LabelSlice::from_name(&brother);
        let other = Name::new("aAa.BaIdu.cN").unwrap();
        let mut ls_other = LabelSlice::from_name(&other);
        assert_eq!(
            ls_grand_parent.compare(&ls_parent, false).relation,
            NameRelation::SuperDomain
        );
        assert_eq!(
            ls_parent.compare(&ls_child, false).relation,
            NameRelation::SuperDomain
        );
        assert_eq!(
            ls_child.compare(&ls_parent, false).relation,
            NameRelation::SubDomain
        );
        assert_eq!(
            ls_child.compare(&ls_grand_parent, false).relation,
            NameRelation::SubDomain
        );
        assert_eq!(
            ls_child.compare(&ls_brother, false).relation,
            NameRelation::CommonAncestor
        );
        assert_eq!(
            ls_child.compare(&ls_child, false).relation,
            NameRelation::Equal
        );
        ls_child.strip_left(1);
        ls_other.strip_left(1);
        assert_eq!(
            ls_child.compare(&ls_other, false).relation,
            NameRelation::CommonAncestor
        );
        ls_child.strip_right(1);
        ls_other.strip_right(1);
        assert_eq!(
            ls_child.compare(&ls_other, false).relation,
            NameRelation::None
        );
        let ls_name = Name::new("1.www.google.com").unwrap();
        let mut ls_slice = LabelSlice::from_name(&ls_name);

        ls_slice.strip_left(1);
        ls_slice.strip_right(1);
        let first_label = ls_slice.first_label();
        assert_eq!(first_label, 1);
        let last_label = ls_slice.last_label();
        assert_eq!(last_label, 3);
        let ls_sequence = ls_name.into_label_sequence(first_label, last_label);
        let ls_slice2 = LabelSlice::from_label_sequence(&ls_sequence);

        let ls_name_2 = Name::new("www.google.com.").unwrap();
        let mut ls_slice3 = LabelSlice::from_name(&ls_name_2);
        ls_slice3.strip_right(1);
        assert_eq!(
            ls_slice2.compare(&ls_slice3, false).relation,
            NameRelation::Equal
        );
    }

    #[test]
    fn test_label_slice_root() {
        let n1 = Name::new(".").unwrap();
        let ls1 = LabelSlice::from_name(&n1);
        assert_eq!(ls1.len(), 1);

        let ls2 = LabelSlice::from_name(&n1);
        assert_eq!(ls2.len(), 1);
    }
}
