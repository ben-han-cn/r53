use crate::error::DNSError;
use crate::message_render::MessageRender;
use crate::rdata_a;
use crate::rdata_aaaa;
use crate::rdata_cname;
use crate::rdata_dname;
use crate::rdata_mx;
use crate::rdata_naptr;
use crate::rdata_ns;
use crate::rdata_opt;
use crate::rdata_ptr;
use crate::rdata_soa;
use crate::rdata_srv;
use crate::rdata_txt;
use crate::rdatafield_string_parser::Parser;
use crate::rr_type::RRType;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RData {
    A(rdata_a::A),
    AAAA(rdata_aaaa::AAAA),
    NS(Box<rdata_ns::NS>),
    CName(Box<rdata_cname::CName>),
    SOA(Box<rdata_soa::SOA>),
    PTR(Box<rdata_ptr::PTR>),
    MX(Box<rdata_mx::MX>),
    NAPTR(Box<rdata_naptr::NAPTR>),
    DName(Box<rdata_dname::DName>),
    OPT(Box<rdata_opt::OPT>),
    SRV(Box<rdata_srv::SRV>),
    TXT(Box<rdata_txt::TXT>),
}

impl RData {
    pub fn from_wire(typ: RRType, buf: &mut InputBuffer, len: u16) -> Result<Self> {
        let pos = buf.position();
        let rdata = match typ {
            RRType::A => rdata_a::A::from_wire(buf, len).map(RData::A),
            RRType::AAAA => rdata_aaaa::AAAA::from_wire(buf, len).map(RData::AAAA),
            RRType::NS => rdata_ns::NS::from_wire(buf, len).map(|ns| RData::NS(Box::new(ns))),
            RRType::CNAME => {
                rdata_cname::CName::from_wire(buf, len).map(|cname| RData::CName(Box::new(cname)))
            }
            RRType::SOA => rdata_soa::SOA::from_wire(buf, len).map(|soa| RData::SOA(Box::new(soa))),
            RRType::PTR => rdata_ptr::PTR::from_wire(buf, len).map(|ptr| RData::PTR(Box::new(ptr))),
            RRType::MX => rdata_mx::MX::from_wire(buf, len).map(|mx| RData::MX(Box::new(mx))),
            RRType::NAPTR => {
                rdata_naptr::NAPTR::from_wire(buf, len).map(|naptr| RData::NAPTR(Box::new(naptr)))
            }
            RRType::DNAME => {
                rdata_dname::DName::from_wire(buf, len).map(|dname| RData::DName(Box::new(dname)))
            }
            RRType::OPT => rdata_opt::OPT::from_wire(buf, len).map(|opt| RData::OPT(Box::new(opt))),
            RRType::SRV => rdata_srv::SRV::from_wire(buf, len).map(|srv| RData::SRV(Box::new(srv))),
            RRType::TXT => rdata_txt::TXT::from_wire(buf, len).map(|txt| RData::TXT(Box::new(txt))),
            _ => Err(DNSError::UnknownRRType(typ.to_u16()).into()),
        };

        if rdata.is_ok() && buf.position() - pos != (len as usize) {
            Err(DNSError::RdataLenIsNotCorrect.into())
        } else {
            rdata
        }
    }

    pub fn rend(&self, render: &mut MessageRender) {
        match *self {
            RData::A(ref a) => a.rend(render),
            RData::AAAA(ref aaaa) => aaaa.rend(render),
            RData::NS(ref ns) => ns.rend(render),
            RData::CName(ref cname) => cname.rend(render),
            RData::SOA(ref soa) => soa.rend(render),
            RData::PTR(ref ptr) => ptr.rend(render),
            RData::MX(ref mx) => mx.rend(render),
            RData::NAPTR(ref naptr) => naptr.rend(render),
            RData::DName(ref dname) => dname.rend(render),
            RData::OPT(ref opt) => opt.rend(render),
            RData::SRV(ref srv) => srv.rend(render),
            RData::TXT(ref txt) => txt.rend(render),
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        match *self {
            RData::A(ref a) => a.to_wire(buf),
            RData::AAAA(ref aaaa) => aaaa.to_wire(buf),
            RData::NS(ref ns) => ns.to_wire(buf),
            RData::CName(ref cname) => cname.to_wire(buf),
            RData::SOA(ref soa) => soa.to_wire(buf),
            RData::PTR(ref ptr) => ptr.to_wire(buf),
            RData::MX(ref mx) => mx.to_wire(buf),
            RData::NAPTR(ref naptr) => naptr.to_wire(buf),
            RData::DName(ref dname) => dname.to_wire(buf),
            RData::OPT(ref opt) => opt.to_wire(buf),
            RData::SRV(ref srv) => srv.to_wire(buf),
            RData::TXT(ref txt) => txt.to_wire(buf),
        }
    }

    pub fn from_str(typ: RRType, s: &str) -> Result<Self> {
        let mut labels = Parser::new(s.trim());
        Self::from_parser(typ, &mut labels)
    }

    pub fn from_parser<'a>(typ: RRType, rdata_str: &mut Parser<'a>) -> Result<Self> {
        match typ {
            RRType::A => rdata_a::A::from_parser(rdata_str).map(RData::A),
            RRType::AAAA => rdata_aaaa::AAAA::from_parser(rdata_str).map(RData::AAAA),
            RRType::NS => rdata_ns::NS::from_parser(rdata_str).map(|ns| RData::NS(Box::new(ns))),
            RRType::CNAME => rdata_cname::CName::from_parser(rdata_str)
                .map(|cname| RData::CName(Box::new(cname))),
            RRType::SOA => {
                rdata_soa::SOA::from_parser(rdata_str).map(|soa| RData::SOA(Box::new(soa)))
            }
            RRType::PTR => {
                rdata_ptr::PTR::from_parser(rdata_str).map(|ptr| RData::PTR(Box::new(ptr)))
            }
            RRType::MX => rdata_mx::MX::from_parser(rdata_str).map(|mx| RData::MX(Box::new(mx))),
            RRType::NAPTR => rdata_naptr::NAPTR::from_parser(rdata_str)
                .map(|naptr| RData::NAPTR(Box::new(naptr))),
            RRType::DNAME => rdata_dname::DName::from_parser(rdata_str)
                .map(|dname| RData::DName(Box::new(dname))),
            RRType::OPT => {
                rdata_opt::OPT::from_parser(rdata_str).map(|opt| RData::OPT(Box::new(opt)))
            }
            RRType::SRV => {
                rdata_srv::SRV::from_parser(rdata_str).map(|srv| RData::SRV(Box::new(srv)))
            }
            RRType::TXT => {
                rdata_txt::TXT::from_parser(rdata_str).map(|txt| RData::TXT(Box::new(txt)))
            }
            _ => Err(DNSError::RRTypeIsNotSupport.into()),
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
            RData::DName(ref dname) => write!(f, "{}", dname),
            RData::OPT(ref opt) => write!(f, "{}", opt),
            RData::SRV(ref srv) => write!(f, "{}", srv),
            RData::TXT(ref txt) => write!(f, "{}", txt),
        }
    }
}
