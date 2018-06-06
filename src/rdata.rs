use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;
use rr_type::RRType;
use super::error::*;
use rdata_a;
use rdata_ns;
use rdata_aaaa;
use rdata_cname;
use rdata_soa;
use rdata_ptr;
use rdata_mx;
use rdata_naptr;
use rdata_dname;
use rdata_opt;

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
    DNAME(Box<rdata_dname::DName>),
    OPT(Box<rdata_opt::OPT>),
}

impl RData {
    pub fn from_wire(typ: RRType, buf: &mut InputBuffer, len: u16) -> Result<Self> {
        let pos = buf.position();
        let rdata = match typ {
            RRType::A => rdata_a::A::from_wire(buf, len).map(|a| RData::A(a)),
            RRType::AAAA => rdata_aaaa::AAAA::from_wire(buf, len).map(|aaaa| RData::AAAA(aaaa)),
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
                rdata_dname::DName::from_wire(buf, len).map(|dname| RData::DNAME(Box::new(dname)))
            }
            RRType::OPT => rdata_opt::OPT::from_wire(buf, len).map(|opt| RData::OPT(Box::new(opt))),
            _ => Err(ErrorKind::UnknownRRType.into()),
        };

        if rdata.is_ok() && buf.position() - pos != (len as usize) {
            Err(ErrorKind::RdataLenIsNotCorrect.into())
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
            RData::DNAME(ref dname) => dname.rend(render),
            RData::OPT(ref opt) => opt.rend(render),
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
            RData::DNAME(ref dname) => dname.to_wire(buf),
            RData::OPT(ref opt) => opt.to_wire(buf),
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            RData::A(ref a) => a.to_string(),
            RData::AAAA(ref aaaa) => aaaa.to_string(),
            RData::NS(ref ns) => ns.to_string(),
            RData::CName(ref cname) => cname.to_string(),
            RData::SOA(ref soa) => soa.to_string(),
            RData::PTR(ref ptr) => ptr.to_string(),
            RData::MX(ref mx) => mx.to_string(),
            RData::NAPTR(ref naptr) => naptr.to_string(),
            RData::DNAME(ref dname) => dname.to_string(),
            RData::OPT(ref opt) => opt.to_string(),
        }
    }
}
