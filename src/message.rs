use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;
use super::error::Error;
use rrset::RRset;
use rr_type::RRType;
use header::Header;
use question::Question;
use std::fmt::Write;
use edns::Edns;

pub enum SectionType {
    Answer = 0,
    Auth = 1,
    Additional = 2,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Section(Option<Vec<RRset>>);

impl Section {
    fn rr_count(&self) -> usize {
        match self.0 {
            Some(ref rrsets) => {
                rrsets.iter().fold(
                    0,
                    |count, ref rrset| count + rrset.rr_count(),
                    )
            }
            None => 0,
        }
    }

    pub fn from_wire(buf: &mut InputBuffer, rr_count: u16) -> Result<Self, Error> {
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
        if let Some(ref rrsets) = self.0 {
            for rrset in rrsets {
                rrset.rend(render);
            }
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        if let Some(ref rrsets) = self.0 {
            for rrset in rrsets {
                rrset.to_wire(buf);
            }
        }
    }

    pub fn to_string(&self) -> String {
        let mut rrset_str = String::new();
        if let Some(ref rrsets) = self.0 {
            for rrset in rrsets {
                write!(rrset_str, "{}", rrset.to_string()).unwrap();
            }
        }
        rrset_str
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Message {
    pub header: Header,
    pub question: Question,
    pub sections: [Section; 3],
    pub edns: Option<Edns>,
}

impl Message {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self, Error> {
        let header = Header::from_wire(buf)?;
        if header.qd_count != 1 {
            return Err(Error::ShortOfQuestion);
        }

        let question = Question::from_wire(buf)?;
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
            header: header,
            question: question,
            sections: [answer, auth, additional],
            edns: edns,
        })
    }

    pub fn recalculate_header(&mut self) {
        self.header.qd_count = 1;
        self.header.an_count = self.sections[0].rr_count() as u16;
        self.header.ns_count = self.sections[1].rr_count() as u16;
        self.header.ar_count = self.sections[2].rr_count() as u16;
        if let Some(ref edns) = self.edns {
            self.header.ar_count += edns.rr_count() as u16;
        }
    }

    pub fn rend(&self, render: &mut MessageRender) {
        self.header.rend(render);
        self.question.rend(render);
        self.sections.iter().for_each(
            |section| section.rend(render),
            );
        if let Some(ref edns) = self.edns {
            edns.rend(render);
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.header.to_wire(buf);
        self.question.to_wire(buf);
        self.sections.iter().for_each(
            |section| section.to_wire(buf),
            );
        if let Some(ref edns) = self.edns {
            edns.to_wire(buf);
        }
    }

    pub fn to_string(&self) -> String {
        let mut message_str = String::new();
        write!(message_str, "{}", self.header.to_string()).unwrap();
        if let Some(ref edns) = self.edns {
            write!(message_str, ";; OPT PSEUDOSECTION:\n{}", edns.to_string()).unwrap();
        }

        write!(
            message_str,
            ";; QUESTION SECTION:\n{}\n",
            self.question.to_string()
            ).unwrap();

        if self.header.an_count > 0 {
            write!(
                message_str,
                "\n;; ANSWER SECTION:\n{}",
                self.sections[0].to_string()
                ).unwrap();
        }

        if self.header.ns_count > 0 {
            write!(
                message_str,
                "\n;; AUTHORITY SECTION:\n{}",
                self.sections[1].to_string()
                ).unwrap();
        }

        if self.header.ar_count > 0 {
            write!(
                message_str,
                "\n;; ADDITIONAL SECTION:\n{}",
                self.sections[2].to_string()
                ).unwrap();
        }
        message_str
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use util::hex::from_hex;
    use super::super::rdata_a::A;
    use super::super::rdata_ns::NS;
    use super::super::opcode::Opcode;
    use super::super::name::Name;
    use super::super::rcode::Rcode;
    use super::super::util::InputBuffer;
    use super::super::header_flag::HeaderFlag;
    use super::super::rrset::RRTtl;
    use super::super::rdata::RData;
    use super::super::rr_class::RRClass;
    use super::super::rr_type::RRType;

    fn build_desired_message()-> Message {
        let mut header = Header {
            id: 1200,
            flag: 0,
            opcode: Opcode::Query,
            rcode: Rcode::NoError,
            qd_count: 1,
            an_count: 2,
            ns_count: 1,
            ar_count: 2,
        };
        header.set_flag(HeaderFlag::QueryRespone, true);
        header.set_flag(HeaderFlag::AuthAnswer, true);
        header.set_flag(HeaderFlag::RecursionDesired, true);

        let question = Question {
            name: Name::new("test.example.com.", false).unwrap(),
            typ: RRType::A,
            class: RRClass::IN,
        };

        let mut answer = Vec::with_capacity(1);
        answer.push(RRset {
            name: Name::new("test.example.com.", false).unwrap(),
            typ: RRType::A,
            class: RRClass::IN,
            ttl: RRTtl(3600),
            rdatas: [RData::A(A::from_string("192.0.2.2").unwrap()), RData::A(A::from_string("192.0.2.1").unwrap())].to_vec(),
        });

        let mut auth = Vec::with_capacity(1);
        auth.push(RRset {
            name: Name::new("example.com.", false).unwrap(),
            typ: RRType::NS,
            class: RRClass::IN,
            ttl: RRTtl(3600),
            rdatas: [RData::NS(NS::from_string("ns1.example.com.").unwrap())].to_vec(),
        });

        let mut additional = Vec::with_capacity(1);
        additional.push(RRset {
            name: Name::new("ns1.example.com.", false).unwrap(),
            typ: RRType::A,
            class: RRClass::IN,
            ttl: RRTtl(3600),
            rdatas: [RData::A(A::from_string("2.2.2.2").unwrap())].to_vec(),
        });

        let edns = Edns {
            versoin: 0,
            extened_rcode: 0,
            udp_size: 4096,
            dnssec_aware: false,
            options: None,
        };

        Message {
            header: header,
            question: question,
            sections: [Section(Some(answer)), Section(Some(auth)), Section(Some(additional))],
            edns: Some(edns),
        }
    }

    #[test]
    fn test_message_to_wire() {
        let raw =
            from_hex("04b0850000010002000100020474657374076578616d706c6503636f6d0000010001c00c0001000100000e10000
                     4c0000202c00c0001000100000e100004c0000201c0110002000100000e100006036e7331c011c04e0001000100000e100004020202020000
                     291000000000000000").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let message = Message::from_wire(&mut buf).unwrap();
        let desired_message = build_desired_message();
        assert_eq!(message, desired_message);

        let mut render = MessageRender::new();
        desired_message.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
    }
}
