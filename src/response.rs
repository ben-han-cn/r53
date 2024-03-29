use crate::edns::Edns;
use crate::header::Header;
use crate::header_flag::HeaderFlag;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::question::Question;
use crate::response_iter::ResponseIter;
use crate::rr_type::RRType;
use crate::rrset::RRset;
use crate::util::InputBuffer;
use anyhow::{bail, Result};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SectionType {
    Answer = 0,
    Authority = 1,
    Additional = 2,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Section(pub Option<Vec<RRset>>);

pub const ALL_SECTIONS: [SectionType; 3] = [
    SectionType::Answer,
    SectionType::Authority,
    SectionType::Additional,
];

impl Section {
    fn rr_count(&self) -> usize {
        self.0.as_ref().map_or(0, |rrsets| {
            rrsets.iter().fold(0, |count, ref rrset| {
                let rr_count = rrset.rr_count();
                //for rrset has no rdata, count it as 1
                if rr_count == 0 {
                    count + 1
                } else {
                    count + rr_count
                }
            })
        })
    }

    pub fn from_wire(buf: &mut InputBuffer, rr_count: u16, typ: SectionType) -> Result<Self> {
        if rr_count == 0 {
            return Ok(Section(None));
        }

        let mut rrsets = Vec::with_capacity(rr_count as usize);
        let mut last_rrset = RRset::from_wire(buf)?;
        if last_rrset.typ == RRType::OPT && typ != SectionType::Additional {
            bail!("opt record must resides in addtional section")
        }

        for _ in 1..rr_count {
            let mut rrset = RRset::from_wire(buf)?;
            if rrset.is_same_rrset(&last_rrset) {
                if rrset.typ == RRType::OPT {
                    bail!("opt rrset can only have one rr");
                }
                last_rrset.rdatas.push(rrset.rdatas.remove(0));
            } else {
                rrsets.push(last_rrset);
                last_rrset = rrset;
            }
        }
        rrsets.push(last_rrset);
        Ok(Section(Some(rrsets)))
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<()> {
        if let Some(rrsets) = self.0.as_ref() {
            for rrset in rrsets {
                rrset.to_wire(render)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(rrsets) = self.0.as_ref() {
            rrsets
                .iter()
                .map(|ref rrset| {
                    if rrset.typ != RRType::OPT {
                        write!(f, "{}", rrset)
                    } else {
                        Ok(())
                    }
                })
                .collect()
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Response {
    pub header: Header,
    pub question: Question,
    pub sections: [Section; 3],
}

impl Response {
    pub fn with_question(name: Name, qtype: RRType) -> Self {
        let mut header: Header = Default::default();
        header.set_flag(HeaderFlag::RecursionDesired, true);
        header.id = rand::random::<u16>();
        Response {
            header,
            question: Question::new(name, qtype),
            sections: [Section(None), Section(None), Section(None)],
        }
    }

    pub fn from_wire(raw: &[u8]) -> Result<Self> {
        let buf = &mut InputBuffer::new(raw);
        let header = Header::from_wire(buf)?;
        if header.qd_count != 1 {
            bail!("response doesn't have one question");
        }

        let question = Question::from_wire(buf)?;
        let answer = Section::from_wire(buf, header.an_count, SectionType::Answer)?;
        let auth = Section::from_wire(buf, header.ns_count, SectionType::Authority)?;
        let additional = Section::from_wire(buf, header.ar_count, SectionType::Additional)?;

        Ok(Response {
            header,
            question,
            sections: [answer, auth, additional],
        })
    }

    pub fn get_edns(&self) -> Option<Edns> {
        self.sections[2]
            .0
            .as_ref()
            .map(|rrsets| {
                if let Some(rrset) = rrsets.last() {
                    if rrset.typ == RRType::OPT {
                        return Some(Edns::from_rrset(rrset));
                    }
                }
                None
            })
            .flatten()
    }

    pub fn recalculate_header(&mut self) {
        self.header.qd_count = 1;
        self.header.an_count = self.sections[0].rr_count() as u16;
        self.header.ns_count = self.sections[1].rr_count() as u16;
        self.header.ar_count = self.sections[2].rr_count() as u16;
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<usize> {
        self.header.to_wire(render)?;
        self.question.to_wire(render)?;

        let pos_after_question = render.len();
        //if has truncate, only keep question
        for section in &self.sections {
            if let Err(_) = section.to_wire(render) {
                self.truncate(render, pos_after_question);
                return Ok(pos_after_question);
            }
        }
        Ok(render.len())
    }

    fn truncate(&self, render: &mut MessageRender, pos: usize) {
        let mut header = self.header.clone();
        header.set_flag(HeaderFlag::Truncation, true);
        render.write_u16_at(2, header.header_flag()).unwrap();
        //skip question section count
        render.write_u16_at(6, 0).unwrap();
        render.write_u16_at(8, 0).unwrap();
        render.write_u16_at(10, 0).unwrap();
        render.truncate(pos).unwrap();
    }

    pub fn section_mut(&mut self, section: SectionType) -> Option<&mut Vec<RRset>> {
        self.sections[section as usize].0.as_mut()
    }

    pub fn section(&self, section: SectionType) -> Option<&Vec<RRset>> {
        self.sections[section as usize].0.as_ref()
    }

    pub fn section_rrset_count(&self, section: SectionType) -> usize {
        self.section(section).map_or(0, |rrsets| rrsets.len())
    }

    pub fn take_section(&mut self, section: SectionType) -> Option<Vec<RRset>> {
        self.sections[section as usize].0.take()
    }

    pub fn clear_section(&mut self, section: SectionType) {
        self.sections[section as usize] = Section(None)
    }

    pub fn iter(&self) -> ResponseIter<'_> {
        ResponseIter::new(&self)
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.header)?;
        if let Some(edns) = self.get_edns() {
            write!(f, ";; OPT PSEUDOSECTION:\n{}", edns)?;
        }

        writeln!(f, ";; QUESTION SECTION:\n{}\n", self.question)?;

        if self.header.an_count > 0 {
            write!(f, ";; ANSWER SECTION:\n{}\n", self.sections[0])?;
        }

        if self.header.ns_count > 0 {
            write!(f, ";; AUTHORITY SECTION:\n{}\n", self.sections[1])?;
        }

        if self.header.ar_count > 0 {
            write!(f, ";; ADDITIONAL SECTION:\n{}", self.sections[2])?;
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a Response {
    type IntoIter = ResponseIter<'a>;
    type Item = (&'a RRset, SectionType);

    fn into_iter(self) -> ResponseIter<'a> {
        ResponseIter::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rrset;
    use std::str::FromStr;
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
        let msg = Response::from_wire(axfr_msg.as_slice()).unwrap();
        assert_eq!(msg.header.id, 58980);
        assert_eq!(msg.header.an_count, 15);
        assert_eq!(msg.header.ns_count, 0);
        assert_eq!(msg.header.ar_count, 0);

        let rrset_strs = vec![
            "example.	3600	IN	SOA	mname1. . 2000042795 20 20 1814400 3600",
            "example.	3600	IN	NS	a.example.",
            "a.example.	3600	IN	A	73.80.65.49",
            "aaaa.example.	3600	IN	AAAA	ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
            "cname.example.	3600	IN	CNAME	cname-target.",
            "mx.example.	3600	IN	MX	10 mail.example.",
            r#"naptr.example.	3600	IN	NAPTR	65535 65535 "blurgh" "blorf" ":(.*):\\1:" foo."#,
            "ptr.example.	3600	IN	PTR	foo.net.",
            "srv.example.	3600	IN	SRV	65535 65535 65535 old-slow-box.example.com.",
            r#"txt1.example.	3600	IN	TXT	"foo foo foo""#,
            r#"txt2.example.	3600	IN	TXT	"foo" "bar""#,
            r#"txt3.example.	3600	IN	TXT	"foo bar""#,
            r#"txt4.example.	3600	IN	TXT	"foo\010bar""#,
            r#"txt5.example.	3600	IN	TXT	"\"foo\"""#,
            "example.	3600	IN	SOA	mname1. . 2000042795 20 20 1814400 3600",
        ];

        let answers = msg.section(SectionType::Answer).unwrap();
        for (i, rrset_str) in rrset_strs.iter().enumerate() {
            let rrset = rrset::RRset::from_str(*rrset_str).unwrap();
            assert_eq!(answers[i], rrset);
        }
    }

    #[test]
    fn test_message_truncate() {
        let raw = vec![
            0xc8, 0x6d, 0x81, 0x80, 0x0, 0x1, 0x0, 0x1c, 0x0, 0x0, 0x0, 0x1, 0x3, 0x62, 0x67, 0x70,
            0x3, 0x73, 0x6c, 0x62, 0xe, 0x68, 0x74, 0x74, 0x70, 0x64, 0x6e, 0x73, 0x2d, 0x61, 0x6c,
            0x69, 0x65, 0x63, 0x32, 0x1, 0x6c, 0x7, 0x62, 0x79, 0x74, 0x65, 0x64, 0x6e, 0x73, 0x3,
            0x6e, 0x65, 0x74, 0x0, 0x0, 0x1, 0x0, 0x1, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8b, 0xe0, 0x38, 0xbf, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x81, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x8c, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x88, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x8a, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x84, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x6a, 0xf, 0x7c, 0x5e, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x87, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x89, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x65, 0x84, 0xae, 0x92, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x6a, 0xe, 0x17, 0xb, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x86, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8b, 0xc4, 0xc8, 0xf1, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8b, 0xc4, 0xc1, 0xc4, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x8e, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x2f, 0x64, 0xa7, 0xc4, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x2f, 0x65, 0xbe, 0x78, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8b, 0xc4, 0xd0, 0x3d, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x65, 0x84, 0xaa, 0x2e, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x83, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x8b, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x6a, 0xf, 0xc3, 0x1, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x8d, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x85, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x90, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x2f, 0x66, 0x9d, 0x9a, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x8f, 0xc0, 0xc, 0x0, 0x1, 0x0, 0x1, 0x0, 0x0,
            0x0, 0x26, 0x0, 0x4, 0x8, 0x85, 0x7b, 0x82, 0x0, 0x0, 0x29, 0x2, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0,
        ];
        let msg = Response::from_wire(raw.as_slice()).unwrap();

        let mut buf = [0; 512];
        let mut render = MessageRender::new(&mut buf);
        msg.to_wire(&mut render).unwrap();
        assert!(render.len() < raw.len());
        let data_len = render.len();
        let truncated_msg = Response::from_wire(&buf[0..data_len]).unwrap();
        assert!(truncated_msg.header.id == msg.header.id);
        assert!(truncated_msg.question == msg.question);
        assert!(truncated_msg.header.is_flag_set(HeaderFlag::Truncation));
        assert!(truncated_msg.header.an_count == 0);
        assert!(truncated_msg.header.ns_count == 0);
        assert!(truncated_msg.header.ar_count == 0);

        let mut buf = [0; 1024];
        let mut render = MessageRender::new(&mut buf);
        msg.to_wire(&mut render).unwrap();
        assert!(render.len() == raw.len());
    }

    #[test]
    fn test_big_message() {
        let raw = vec![
            0x56, 0xcf, 0x81, 0x0, 0x0, 0x1, 0x0, 0xe, 0x0, 0x0, 0x0, 0x1, 0x7, 0x7a, 0x65, 0x6e,
            0x64, 0x65, 0x73, 0x6b, 0x3, 0x63, 0x6f, 0x6d, 0x0, 0x0, 0x10, 0x0, 0x1, 0xc0, 0xc,
            0x0, 0x10, 0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0, 0x3b, 0x3a, 0x6d, 0x6f, 0x6e, 0x67,
            0x6f, 0x64, 0x62, 0x2d, 0x73, 0x69, 0x74, 0x65, 0x2d, 0x76, 0x65, 0x72, 0x69, 0x66,
            0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x3d, 0x65, 0x33, 0x30, 0x46, 0x55, 0x7a,
            0x57, 0x4d, 0x42, 0x6d, 0x64, 0x4d, 0x54, 0x65, 0x4b, 0x71, 0x53, 0x6b, 0x48, 0x76,
            0x62, 0x6f, 0x6b, 0x54, 0x76, 0x65, 0x36, 0x70, 0x5a, 0x4f, 0x76, 0x77, 0xc0, 0xc, 0x0,
            0x10, 0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0, 0x2e, 0x2d, 0x74, 0x68, 0x6f, 0x75, 0x73,
            0x61, 0x6e, 0x64, 0x65, 0x79, 0x65, 0x73, 0x3a, 0x70, 0x36, 0x78, 0x68, 0x34, 0x72,
            0x69, 0x63, 0x36, 0x6a, 0x75, 0x6f, 0x66, 0x37, 0x72, 0x76, 0x73, 0x6b, 0x79, 0x68,
            0x38, 0x39, 0x76, 0x6b, 0x38, 0x77, 0x76, 0x6f, 0x72, 0x75, 0x71, 0x37, 0xc0, 0xc, 0x0,
            0x10, 0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0, 0xf9, 0xf8, 0x76, 0x3d, 0x73, 0x70, 0x66,
            0x31, 0x20, 0x69, 0x70, 0x34, 0x3a, 0x31, 0x30, 0x33, 0x2e, 0x31, 0x33, 0x2e, 0x36,
            0x39, 0x2e, 0x30, 0x2f, 0x32, 0x34, 0x20, 0x69, 0x70, 0x34, 0x3a, 0x31, 0x30, 0x33,
            0x2e, 0x31, 0x35, 0x31, 0x2e, 0x31, 0x39, 0x32, 0x2e, 0x30, 0x2f, 0x32, 0x33, 0x20,
            0x69, 0x70, 0x34, 0x3a, 0x31, 0x30, 0x33, 0x2e, 0x39, 0x36, 0x2e, 0x32, 0x33, 0x2e,
            0x30, 0x2f, 0x32, 0x34, 0x20, 0x69, 0x70, 0x34, 0x3a, 0x31, 0x30, 0x34, 0x2e, 0x34,
            0x33, 0x2e, 0x32, 0x34, 0x33, 0x2e, 0x32, 0x33, 0x37, 0x20, 0x69, 0x70, 0x34, 0x3a,
            0x31, 0x30, 0x38, 0x2e, 0x31, 0x37, 0x37, 0x2e, 0x38, 0x2e, 0x30, 0x2f, 0x32, 0x31,
            0x20, 0x69, 0x70, 0x34, 0x3a, 0x31, 0x30, 0x38, 0x2e, 0x31, 0x37, 0x37, 0x2e, 0x39,
            0x36, 0x2e, 0x30, 0x2f, 0x31, 0x39, 0x20, 0x69, 0x70, 0x34, 0x3a, 0x31, 0x32, 0x34,
            0x2e, 0x34, 0x37, 0x2e, 0x31, 0x35, 0x30, 0x2e, 0x30, 0x2f, 0x32, 0x34, 0x20, 0x69,
            0x70, 0x34, 0x3a, 0x31, 0x32, 0x34, 0x2e, 0x34, 0x37, 0x2e, 0x31, 0x38, 0x39, 0x2e,
            0x30, 0x2f, 0x32, 0x34, 0x20, 0x69, 0x70, 0x34, 0x3a, 0x31, 0x33, 0x30, 0x2e, 0x32,
            0x31, 0x31, 0x2e, 0x30, 0x2e, 0x30, 0x2f, 0x32, 0x32, 0x20, 0x69, 0x70, 0x34, 0x3a,
            0x31, 0x34, 0x36, 0x2e, 0x31, 0x30, 0x31, 0x2e, 0x37, 0x38, 0x2e, 0x30, 0x2f, 0x32,
            0x34, 0x20, 0x69, 0x70, 0x34, 0x3a, 0x31, 0x37, 0x30, 0x2e, 0x31, 0x30, 0x2e, 0x31,
            0x32, 0x39, 0x2e, 0x30, 0x2f, 0x32, 0x34, 0x20, 0x69, 0x6e, 0x63, 0x6c, 0x75, 0x64,
            0x65, 0x3a, 0x5f, 0x73, 0x70, 0x66, 0x31, 0x2e, 0x7a, 0x65, 0x6e, 0x64, 0x65, 0x73,
            0x6b, 0x2e, 0x63, 0x6f, 0x6d, 0xc0, 0xc, 0x0, 0x10, 0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0,
            0x12, 0x11, 0x39, 0x34, 0x32, 0x35, 0x35, 0x32, 0x33, 0x36, 0x32, 0x2d, 0x32, 0x34,
            0x38, 0x32, 0x34, 0x34, 0x37, 0xc0, 0xc, 0x0, 0x10, 0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0,
            0xa, 0x9, 0x45, 0x4e, 0x4a, 0x55, 0x54, 0x59, 0x52, 0x59, 0x51, 0xc0, 0xc, 0x0, 0x10,
            0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0, 0xe, 0xd, 0x4d, 0x53, 0x3d, 0x6d, 0x73, 0x38, 0x37,
            0x30, 0x35, 0x32, 0x30, 0x35, 0x34, 0xc0, 0xc, 0x0, 0x10, 0x0, 0x1, 0x0, 0x0, 0xa,
            0x41, 0x0, 0xe, 0xd, 0x4d, 0x53, 0x3d, 0x6d, 0x73, 0x39, 0x31, 0x34, 0x34, 0x35, 0x39,
            0x31, 0x31, 0xc0, 0xc, 0x0, 0x10, 0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0, 0x5f, 0x5e, 0x61,
            0x74, 0x6c, 0x61, 0x73, 0x73, 0x69, 0x61, 0x6e, 0x2d, 0x64, 0x6f, 0x6d, 0x61, 0x69,
            0x6e, 0x2d, 0x76, 0x65, 0x72, 0x69, 0x66, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e,
            0x3d, 0x31, 0x45, 0x42, 0x54, 0x30, 0x61, 0x4e, 0x7a, 0x48, 0x76, 0x70, 0x6a, 0x41,
            0x30, 0x6d, 0x55, 0x71, 0x64, 0x43, 0x46, 0x2b, 0x71, 0x55, 0x2f, 0x67, 0x73, 0x53,
            0x38, 0x71, 0x78, 0x6a, 0x6a, 0x6f, 0x35, 0x30, 0x77, 0x59, 0x56, 0x63, 0x34, 0x6e,
            0x6c, 0x43, 0x6f, 0x6f, 0x4b, 0x41, 0x4a, 0x5a, 0x55, 0x6d, 0x56, 0x49, 0x32, 0x4d,
            0x6e, 0x73, 0x65, 0x41, 0x4e, 0x78, 0x32, 0x64, 0x64, 0xc0, 0xc, 0x0, 0x10, 0x0, 0x1,
            0x0, 0x0, 0xa, 0x41, 0x0, 0x25, 0x24, 0x63, 0x61, 0x33, 0x2d, 0x38, 0x35, 0x32, 0x33,
            0x34, 0x34, 0x63, 0x66, 0x63, 0x35, 0x36, 0x61, 0x34, 0x37, 0x64, 0x62, 0x38, 0x62,
            0x31, 0x65, 0x37, 0x62, 0x38, 0x30, 0x35, 0x61, 0x61, 0x66, 0x66, 0x31, 0x32, 0x66,
            0xc0, 0xc, 0x0, 0x10, 0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0, 0x2e, 0x2d, 0x64, 0x6f, 0x63,
            0x75, 0x73, 0x69, 0x67, 0x6e, 0x3d, 0x37, 0x30, 0x63, 0x65, 0x64, 0x34, 0x30, 0x35,
            0x2d, 0x36, 0x31, 0x38, 0x31, 0x2d, 0x34, 0x37, 0x35, 0x33, 0x2d, 0x61, 0x38, 0x31,
            0x31, 0x2d, 0x62, 0x66, 0x30, 0x32, 0x31, 0x38, 0x65, 0x38, 0x63, 0x35, 0x63, 0x64,
            0xc0, 0xc, 0x0, 0x10, 0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0, 0x45, 0x44, 0x67, 0x6f, 0x6f,
            0x67, 0x6c, 0x65, 0x2d, 0x73, 0x69, 0x74, 0x65, 0x2d, 0x76, 0x65, 0x72, 0x69, 0x66,
            0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x3d, 0x4c, 0x59, 0x79, 0x47, 0x55, 0x79,
            0x38, 0x6a, 0x35, 0x4a, 0x65, 0x4f, 0x74, 0x5f, 0x46, 0x61, 0x64, 0x75, 0x77, 0x31,
            0x54, 0x4d, 0x50, 0x54, 0x41, 0x52, 0x34, 0x66, 0x45, 0x7a, 0x78, 0x66, 0x58, 0x44,
            0x6f, 0x54, 0x35, 0x42, 0x37, 0x39, 0x67, 0x69, 0x77, 0xc0, 0xc, 0x0, 0x10, 0x0, 0x1,
            0x0, 0x0, 0xa, 0x41, 0x0, 0x45, 0x44, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x2d, 0x73,
            0x69, 0x74, 0x65, 0x2d, 0x76, 0x65, 0x72, 0x69, 0x66, 0x69, 0x63, 0x61, 0x74, 0x69,
            0x6f, 0x6e, 0x3d, 0x65, 0x67, 0x6e, 0x4d, 0x58, 0x73, 0x55, 0x47, 0x4f, 0x45, 0x63,
            0x4a, 0x48, 0x66, 0x30, 0x41, 0x45, 0x65, 0x36, 0x73, 0x41, 0x32, 0x75, 0x69, 0x31,
            0x68, 0x78, 0x56, 0x35, 0x36, 0x64, 0x56, 0x50, 0x35, 0x65, 0x6b, 0x6b, 0x30, 0x32,
            0x39, 0x43, 0x38, 0x4d, 0xc0, 0xc, 0x0, 0x10, 0x0, 0x1, 0x0, 0x0, 0xa, 0x41, 0x0, 0x26,
            0x25, 0x6d, 0x61, 0x69, 0x6c, 0x72, 0x75, 0x2d, 0x76, 0x65, 0x72, 0x69, 0x66, 0x69,
            0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x3a, 0x20, 0x63, 0x32, 0x38, 0x37, 0x63, 0x34,
            0x64, 0x35, 0x32, 0x62, 0x62, 0x35, 0x66, 0x64, 0x33, 0x63, 0xc0, 0xc, 0x0, 0x10, 0x0,
            0x1, 0x0, 0x0, 0xa, 0x41, 0x0, 0x3b, 0x3a, 0x6d, 0x69, 0x72, 0x6f, 0x2d, 0x76, 0x65,
            0x72, 0x69, 0x66, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x3d, 0x33, 0x36, 0x37,
            0x33, 0x61, 0x37, 0x64, 0x32, 0x62, 0x65, 0x64, 0x31, 0x64, 0x39, 0x64, 0x39, 0x35,
            0x33, 0x36, 0x33, 0x39, 0x36, 0x30, 0x62, 0x35, 0x38, 0x34, 0x32, 0x63, 0x32, 0x33,
            0x37, 0x33, 0x31, 0x32, 0x63, 0x66, 0x34, 0x32, 0x61, 0x0, 0x0, 0x29, 0x4, 0xd0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0,
        ];
        let msg = Response::from_wire(raw.as_slice()).unwrap();
        println!("----> msg {}", msg);
    }
}
