use crate::header::Header;
use crate::header_flag::HeaderFlag;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::opcode::Opcode;
use crate::question::Question;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::rrset::RRset;
use crate::util::InputBuffer;
use anyhow::{bail, Result};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NotifyRequest {
    pub header: Header,
    pub question: Question,
    pub soa: Option<RRset>,
}

impl NotifyRequest {
    pub fn new(zone: Name, soa: Option<RRset>) -> Self {
        let mut header: Header = Header {
            opcode: Opcode::Notify,
            ..Default::default()
        };
        header.set_flag(HeaderFlag::AuthAnswer, true);
        header.qd_count = 1;
        if soa.is_some() {
            header.an_count = 1;
        }

        NotifyRequest {
            header,
            question: Question {
                name: zone,
                typ: RRType::SOA,
                class: RRClass::IN,
            },
            soa,
        }
    }

    fn from_wire(
        header: Header,
        question: Question,
        raw_after_question: &mut InputBuffer,
    ) -> Result<Self> {
        let mut soa = None;
        if header.an_count == 1 {
            let rrset = RRset::from_wire(raw_after_question)?;
            if rrset.typ != RRType::SOA {
                bail!("answer section of notify should be soa");
            }
            soa = Some(rrset);
        }
        Ok(NotifyRequest {
            header,
            question,
            soa,
        })
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<usize> {
        self.header.to_wire(render)?;
        self.question.to_wire(render)?;
        if let Some(ref soa) = self.soa {
            soa.to_wire(render)?;
        }
        Ok(render.len())
    }
}

impl fmt::Display for NotifyRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "notify for zone {}", self.question.name)?;
        if let Some(ref soa) = self.soa {
            writeln!(f, "soa {}", soa)?
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NotifyResponse {
    pub header: Header,
    pub question: Question,
}

impl NotifyResponse {
    pub fn new(mut req: NotifyRequest) -> Self {
        req.header.set_flag(HeaderFlag::QueryRespone, true);
        NotifyResponse {
            header: req.header,
            question: req.question,
        }
    }

    fn from_wire(
        header: Header,
        question: Question,
        _raw_after_question: &mut InputBuffer,
    ) -> Result<Self> {
        Ok(NotifyResponse { header, question })
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<usize> {
        self.header.to_wire(render)?;
        self.question.to_wire(render)?;
        Ok(render.len())
    }
}
