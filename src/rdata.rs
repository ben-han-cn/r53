use crate::message_render::MessageRender;
use crate::rdatas;
use crate::rr_type::RRType;
use crate::util::{InputBuffer, StringBuffer};
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

    pub fn to_wire(&self, to_wireer: &mut MessageRender) -> Result<()> {
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

    pub fn from_str(typ: RRType, s: &str) -> Result<Self> {
        let mut buf = StringBuffer::new(s);
        Self::from_buffer(typ, &mut buf)
    }

    pub fn from_buffer(typ: RRType, buf: &mut StringBuffer) -> Result<Self> {
        match typ {
            RRType::A => rdatas::A::from_str(buf).map(RData::A),
            RRType::AAAA => rdatas::AAAA::from_str(buf).map(RData::AAAA),
            RRType::NS => rdatas::NS::from_str(buf).map(|ns| RData::NS(Box::new(ns))),
            RRType::CNAME => {
                rdatas::CName::from_str(buf).map(|cname| RData::CName(Box::new(cname)))
            }
            RRType::SOA => rdatas::SOA::from_str(buf).map(|soa| RData::SOA(Box::new(soa))),
            RRType::PTR => rdatas::PTR::from_str(buf).map(|ptr| RData::PTR(Box::new(ptr))),
            RRType::MX => rdatas::MX::from_str(buf).map(|mx| RData::MX(Box::new(mx))),
            RRType::NAPTR => {
                rdatas::NAPTR::from_str(buf).map(|naptr| RData::NAPTR(Box::new(naptr)))
            }
            RRType::OPT => rdatas::OPT::from_str(buf).map(|opt| RData::OPT(Box::new(opt))),
            RRType::SRV => rdatas::SRV::from_str(buf).map(|srv| RData::SRV(Box::new(srv))),
            RRType::TXT => rdatas::TXT::from_str(buf).map(|txt| RData::TXT(Box::new(txt))),
            _ => bail!("rrtype {} isn't support", typ.to_string()),
        }
    }
}

impl fmt::Display for RData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RData::A(ref a) => write!(f, "{}", a),
            RData::AAAA(ref aaaa) => write!(f, "{}", aaaa),
            RData::NS(ref ns) => write!(f, "{}", ns),
            RData::CName(ref cname) => write!(f, "{}", cname),
            RData::SOA(ref soa) => write!(f, "{}", soa),
            RData::PTR(ref ptr) => write!(f, "{}", ptr),
            RData::MX(ref mx) => write!(f, "{}", mx),
            RData::NAPTR(ref naptr) => write!(f, "{}", naptr),
            RData::OPT(ref opt) => write!(f, "{}", opt),
            RData::SRV(ref srv) => write!(f, "{}", srv),
            RData::TXT(ref txt) => write!(f, "{}", txt),
        }
    }
}
