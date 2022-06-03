use crate::header::Header;
use crate::header_flag::HeaderFlag;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::question::Question;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::util::InputBuffer;
use anyhow::{bail, Result};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Request {
    pub header: Header,
    pub question: Question,
}

impl Request {
    pub fn new(name: Name, qtype: RRType) -> Self {
        let mut header: Header = Default::default();
        header.set_flag(HeaderFlag::RecursionDesired, true);
        header.id = rand::random::<u16>();
        Request {
            header,
            question: Question {
                name,
                typ: qtype,
                class: RRClass::IN,
            },
        }
    }

    pub fn from_wire(raw: &[u8]) -> Result<Self> {
        let buf = &mut InputBuffer::new(raw);
        let header = Header::from_wire(buf)?;
        let question = if header.qd_count == 1 {
            Question::from_wire(buf)?
        } else {
            bail!("request has no question");
        };

        if header.an_count != 0 {
            bail!("request has answer");
        }

        if header.an_count != 0 {
            bail!("request has auth");
        }

        Ok(Request { header, question })
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<usize> {
        self.header.to_wire(render)?;
        self.question.to_wire(render)?;
        Ok(render.len())
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.header)?;
        writeln!(f, ";; QUESTION SECTION:\n{}\n", self.question)?;
        Ok(())
    }
}
