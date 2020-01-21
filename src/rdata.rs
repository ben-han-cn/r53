use crate::message_render::MessageRender;
use crate::rdatas;
use crate::rr_type::RRType;
use crate::util::{InputBuffer, OutputBuffer};
use anyhow::{bail, Result};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RData {
    A(rdatas::A),
    AAAA(rdatas::AAAA),
    NS(Box<rdatas::NS>),
    CName(Box<rdatas::CName>),
    SOA(Box<rdatas::SOA>),
    PTR(Box<rdatas::PTR>),
    MX(Box<rdatas::MX>),
    NAPTR(Box<rdatas::NAPTR>),
    OPT(Box<rdatas::OPT>),
    SRV(Box<rdatas::SRV>),
    TXT(Box<rdatas::TXT>),
}

impl RData {
    pub fn from_wire(typ: RRType, buf: &mut InputBuffer, len: u16) -> Result<Self> {
        let pos = buf.position();
        let rdata = match typ {
            RRType::A => rdatas::A::from_wire(buf, len).map(RData::A),
            RRType::AAAA => rdatas::AAAA::from_wire(buf, len).map(RData::AAAA),
            RRType::NS => rdatas::NS::from_wire(buf, len).map(|ns| RData::NS(Box::new(ns))),
            RRType::CNAME => {
                rdatas::CName::from_wire(buf, len).map(|cname| RData::CName(Box::new(cname)))
            }
            RRType::SOA => rdatas::SOA::from_wire(buf, len).map(|soa| RData::SOA(Box::new(soa))),
            RRType::PTR => rdatas::PTR::from_wire(buf, len).map(|ptr| RData::PTR(Box::new(ptr))),
            RRType::MX => rdatas::MX::from_wire(buf, len).map(|mx| RData::MX(Box::new(mx))),
            RRType::NAPTR => {
                rdatas::NAPTR::from_wire(buf, len).map(|naptr| RData::NAPTR(Box::new(naptr)))
            }
            RRType::OPT => rdatas::OPT::from_wire(buf, len).map(|opt| RData::OPT(Box::new(opt))),
            RRType::SRV => rdatas::SRV::from_wire(buf, len).map(|srv| RData::SRV(Box::new(srv))),
            RRType::TXT => rdatas::TXT::from_wire(buf, len).map(|txt| RData::TXT(Box::new(txt))),
            _ => bail!("unknown rr type {}", typ.to_u16()),
        };

        if rdata.is_ok() && buf.position() - pos != (len as usize) {
            bail!("rdata len isn't correct");
        } else {
            rdata
        }
    }

    pub fn to_wire(&self, to_wireer: &mut MessageRender) {
        match *self {
            RData::A(ref a) => a.to_wire(to_wireer),
            RData::AAAA(ref aaaa) => aaaa.to_wire(to_wireer),
            RData::NS(ref ns) => ns.to_wire(to_wireer),
            RData::CName(ref cname) => cname.to_wire(to_wireer),
            RData::SOA(ref soa) => soa.to_wire(to_wireer),
            RData::PTR(ref ptr) => ptr.to_wire(to_wireer),
            RData::MX(ref mx) => mx.to_wire(to_wireer),
            RData::NAPTR(ref naptr) => naptr.to_wire(to_wireer),
            RData::OPT(ref opt) => opt.to_wire(to_wireer),
            RData::SRV(ref srv) => srv.to_wire(to_wireer),
            RData::TXT(ref txt) => txt.to_wire(to_wireer),
        }
    }
}
