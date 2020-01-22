use crate::edns::Edns;
use crate::header::Header;
use crate::header_flag::HeaderFlag;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::question::Question;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::rrset::RRset;
use crate::util::{InputBuffer, OutputBuffer};
use anyhow::Result;
use rand;
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub enum SectionType {
    Answer = 0,
    Authority = 1,
    Additional = 2,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Section(pub Option<Vec<RRset>>);

impl Section {
    fn rr_count(&self) -> usize {
        self.0.as_ref().map_or(0, |rrsets| {
            rrsets
                .iter()
                .fold(0, |count, ref rrset| count + rrset.rr_count())
        })
    }

    pub fn from_wire(buf: &mut InputBuffer, rr_count: u16) -> Result<Self> {
        if rr_count == 0 {
            return Ok(Section(None));
        }

        let mut rrsets = Vec::with_capacity(rr_count as usize);
        let mut last_rrset = RRset::from_wire(buf)?;
        for _ in 1..rr_count {
            let mut rrset = RRset::from_wire(buf)?;
            if rrset.is_same_rrset(&last_rrset) {
                last_rrset.rdatas.push(rrset.rdatas.remove(0));
            } else {
                rrsets.push(last_rrset);
                last_rrset = rrset;
            }
        }
        rrsets.push(last_rrset);
        Ok(Section(Some(rrsets)))
    }

    pub fn to_wire(&self, render: &mut MessageRender) {
        if let Some(rrsets) = self.0.as_ref() {
            rrsets.iter().for_each(|rrset| rrset.to_wire(render));
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Message {
    pub header: Header,
    pub question: Option<Question>,
    pub sections: [Section; 3],
    pub edns: Option<Edns>,
}

impl Message {
    pub fn with_query(name: Name, qtype: RRType) -> Self {
        let mut header: Header = Default::default();
        header.set_flag(HeaderFlag::RecursionDesired, true);
        header.id = rand::random::<u16>();
        Message {
            header,
            question: Some(Question {
                name,
                typ: qtype,
                class: RRClass::IN,
            }),
            sections: [Section(None), Section(None), Section(None)],
            edns: None,
        }
    }

    pub fn from_wire(raw: &[u8]) -> Result<Self> {
        let buf = &mut InputBuffer::new(raw);
        let header = Header::from_wire(buf)?;
        let question = if header.qd_count == 1 {
            Some(Question::from_wire(buf)?)
        } else {
            None
        };

        let answer = Section::from_wire(buf, header.an_count)?;
        let auth = Section::from_wire(buf, header.ns_count)?;
        let mut additional = Section::from_wire(buf, header.ar_count)?;

        let mut edns = None;
        if header.ar_count > 0 {
            let rrsets = additional.0.as_mut().unwrap();
            if rrsets[rrsets.len() - 1].typ == RRType::OPT {
                edns = Some(Edns::from_rrset(&rrsets.pop().unwrap()));
            }
        }

        Ok(Message {
            header,
            question,
            sections: [answer, auth, additional],
            edns,
        })
    }

    pub fn recalculate_header(&mut self) {
        self.header.qd_count = 1;
        self.header.an_count = self.sections[0].rr_count() as u16;
        self.header.ns_count = self.sections[1].rr_count() as u16;
        self.header.ar_count = self.sections[2].rr_count() as u16;
        self.header.ar_count += self.edns.as_ref().map_or(0, |edns| edns.rr_count() as u16);
    }

    pub fn to_wire(&self, render: &mut MessageRender) {
        self.header.to_wire(render);
        self.question.as_ref().map(|q| q.to_wire(render));
        self.sections
            .iter()
            .for_each(|section| section.to_wire(render));
        if let Some(edns) = self.edns.as_ref() {
            edns.to_wire(render)
        }
    }

    pub fn section_mut(&mut self, section: SectionType) -> Option<&mut Vec<RRset>> {
        self.sections[section as usize].0.as_mut()
    }

    pub fn section(&self, section: SectionType) -> Option<&Vec<RRset>> {
        self.sections[section as usize].0.as_ref()
    }

    pub fn take_section(&mut self, section: SectionType) -> Option<Vec<RRset>> {
        self.sections[section as usize].0.take()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_message_from_wire() {
        let axfr_msg = vec![
            230, 100, 132, 128, 0, 1, 0, 15, 0, 0, 0, 0, 7, 101, 120, 97, 109, 112, 108, 101, 0, 0,
            252, 0, 1, 192, 12, 0, 6, 0, 1, 0, 0, 14, 16, 0, 29, 6, 109, 110, 97, 109, 101, 49, 0,
            0, 119, 54, 59, 43, 0, 0, 0, 20, 0, 0, 0, 20, 0, 27, 175, 128, 0, 0, 14, 16, 192, 12,
            0, 2, 0, 1, 0, 0, 14, 16, 0, 4, 1, 97, 192, 12, 192, 78, 0, 1, 0, 1, 0, 0, 14, 16, 0,
            4, 73, 80, 65, 49, 4, 97, 97, 97, 97, 192, 12, 0, 28, 0, 1, 0, 0, 14, 16, 0, 16, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 5, 99, 110,
            97, 109, 101, 192, 12, 0, 5, 0, 1, 0, 0, 14, 16, 0, 14, 12, 99, 110, 97, 109, 101, 45,
            116, 97, 114, 103, 101, 116, 0, 2, 109, 120, 192, 12, 0, 15, 0, 1, 0, 0, 14, 16, 0, 9,
            0, 10, 4, 109, 97, 105, 108, 192, 12, 5, 110, 97, 112, 116, 114, 192, 12, 0, 35, 0, 1,
            0, 0, 14, 16, 0, 32, 255, 255, 255, 255, 6, 98, 108, 117, 114, 103, 104, 5, 98, 108,
            111, 114, 102, 9, 58, 40, 46, 42, 41, 58, 92, 49, 58, 3, 102, 111, 111, 0, 3, 112, 116,
            114, 192, 12, 0, 12, 0, 1, 0, 0, 14, 16, 0, 9, 3, 102, 111, 111, 3, 110, 101, 116, 0,
            3, 115, 114, 118, 192, 12, 0, 33, 0, 1, 0, 0, 14, 16, 0, 32, 255, 255, 255, 255, 255,
            255, 12, 111, 108, 100, 45, 115, 108, 111, 119, 45, 98, 111, 120, 7, 101, 120, 97, 109,
            112, 108, 101, 3, 99, 111, 109, 0, 4, 116, 120, 116, 49, 192, 12, 0, 16, 0, 1, 0, 0,
            14, 16, 0, 12, 11, 102, 111, 111, 32, 102, 111, 111, 32, 102, 111, 111, 4, 116, 120,
            116, 50, 192, 12, 0, 16, 0, 1, 0, 0, 14, 16, 0, 8, 3, 102, 111, 111, 3, 98, 97, 114, 4,
            116, 120, 116, 51, 192, 12, 0, 16, 0, 1, 0, 0, 14, 16, 0, 8, 7, 102, 111, 111, 32, 98,
            97, 114, 4, 116, 120, 116, 52, 192, 12, 0, 16, 0, 1, 0, 0, 14, 16, 0, 8, 7, 102, 111,
            111, 10, 98, 97, 114, 4, 116, 120, 116, 53, 192, 12, 0, 16, 0, 1, 0, 0, 14, 16, 0, 6,
            5, 34, 102, 111, 111, 34, 192, 12, 0, 6, 0, 1, 0, 0, 14, 16, 0, 23, 192, 37, 0, 119,
            54, 59, 43, 0, 0, 0, 20, 0, 0, 0, 20, 0, 27, 175, 128, 0, 0, 14, 16,
        ];
        let msg = Message::from_wire(axfr_msg.as_slice()).unwrap();
        assert_eq!(msg.header.id, 58980);
    }
}
