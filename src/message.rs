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
use failure::Result;
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

    pub fn rend(&self, render: &mut MessageRender) {
        if let Some(rrsets) = self.0.as_ref() {
            rrsets.iter().for_each(|rrset| rrset.rend(render));
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        if let Some(rrsets) = self.0.as_ref() {
            rrsets.iter().for_each(|rrset| rrset.to_wire(buf));
        }
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(rrsets) = self.0.as_ref() {
            rrsets
                .iter()
                .map(|ref rrset| writeln!(f, "{}", rrset))
                .collect()
        } else {
            Ok(())
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

    pub fn rend(&self, render: &mut MessageRender) {
        self.header.rend(render);
        self.question.as_ref().map(|q| q.rend(render));
        self.sections
            .iter()
            .for_each(|section| section.rend(render));
        if let Some(edns) = self.edns.as_ref() {
            edns.rend(render)
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.header.to_wire(buf);
        self.question.as_ref().map(|q| q.to_wire(buf));
        self.sections
            .iter()
            .for_each(|section| section.to_wire(buf));
        if let Some(edns) = self.edns.as_ref() {
            edns.to_wire(buf)
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

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.header)?;
        if let Some(edns) = self.edns.as_ref() {
            write!(f, ";; OPT PSEUDOSECTION:\n{}", edns)?;
        }

        if let Some(question) = self.question.as_ref() {
            writeln!(f, ";; QUESTION SECTION:\n{}", question)?;
        }

        if self.header.an_count > 0 {
            write!(f, "\n;; ANSWER SECTION:\n{}", self.sections[0])?;
        }

        if self.header.ns_count > 0 {
            write!(f, "\n;; AUTHORITY SECTION:\n{}", self.sections[1])?;
        }

        if self.header.ar_count > 0 {
            write!(f, "\n;; ADDITIONAL SECTION:\n{}", self.sections[2])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::header_flag::HeaderFlag;
    use crate::message_builder::MessageBuilder;
    use crate::name::Name;
    use crate::opcode::Opcode;
    use crate::rcode::Rcode;
    use crate::rr_type::RRType;
    use crate::util::hex::from_hex;
    use std::str::FromStr;

    fn build_desired_message() -> Message {
        let mut msg = Message::with_query(Name::new("test.example.com.").unwrap(), RRType::A);
        {
            let mut builder = MessageBuilder::new(&mut msg);
            builder
                .id(1200)
                .opcode(Opcode::Query)
                .rcode(Rcode::NoError)
                .set_flag(HeaderFlag::QueryRespone)
                .set_flag(HeaderFlag::AuthAnswer)
                .set_flag(HeaderFlag::RecursionDesired)
                .add_answer(RRset::from_str("test.example.com. 3600 IN A 192.0.2.2").unwrap())
                .add_answer(RRset::from_str("test.example.com. 3600 IN A 192.0.2.1").unwrap())
                .add_auth(RRset::from_str("example.com. 3600 IN NS ns1.example.com.").unwrap())
                .add_additional(RRset::from_str("ns1.example.com. 3600 IN A 2.2.2.2").unwrap())
                .edns(Edns {
                    versoin: 0,
                    extened_rcode: 0,
                    udp_size: 4096,
                    dnssec_aware: false,
                    options: None,
                })
                .done();
        }
        msg
    }

    #[test]
    fn test_message_to_wire() {
        let raw =
            from_hex("04b0850000010002000100020474657374076578616d706c6503636f6d0000010001c00c0001000100000e100004c0000202c00c0001000100000e100004c0000201c0110002000100000e100006036e7331c011c04e0001000100000e100004020202020000291000000000000000").unwrap();
        let message = Message::from_wire(raw.as_slice()).unwrap();
        let desired_message = build_desired_message();
        assert_eq!(message, desired_message);

        let mut render = MessageRender::new();
        desired_message.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());

        let raw =
            from_hex("04b08500000100010001000103656565066e69757a756f036f72670000100001c00c0010000100000e10001302446f03796f750477616e7402746f03646965c0100002000100000e10001404636e7331097a646e73636c6f7564036e6574000000291000000000000000").unwrap();
        let message = Message::from_wire(raw.as_slice()).unwrap();
        println!("msg:{}", message.to_string());
        let mut render = MessageRender::new();
        message.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
        assert_eq!(
            message.sections[SectionType::Answer as usize]
                .0
                .as_ref()
                .unwrap()[0],
            RRset::from_str("eee.niuzuo.org.	3600	IN	TXT	\"Do\" \"you\" \"want\" \"to\" \"die\"")
                .unwrap()
        );
    }
}
