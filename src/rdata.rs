use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;
use rr_type::RRType;
use super::error::Error;
use rdata_a;
use rdata_ns;
use rdata_aaaa;
use rdata_cname;
use rdata_soa;
use rdata_ptr;
use rdata_mx;
use rdata_naptr;
use rdata_opt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RData {
    A(rdata_a::A),
    NS(rdata_ns::NS),
    AAAA(rdata_aaaa::AAAA),
    CName(rdata_cname::CName),
    SOA(rdata_soa::SOA),
    PTR(rdata_ptr::PTR),
    MX(rdata_mx::MX),
    NAPTR(rdata_naptr::NAPTR),
    OPT(rdata_opt::OPT),
}

impl RData {
    pub fn from_wire(typ: RRType, buf: &mut InputBuffer, len: u16) -> Result<Self, Error> {
        let pos = buf.position();
        let rdata = match typ {
            RRType::A => rdata_a::A::from_wire(buf, len).map(|a| RData::A(a)),
            RRType::NS => rdata_ns::NS::from_wire(buf, len).map(|ns| RData::NS(ns)),
            RRType::AAAA => rdata_aaaa::AAAA::from_wire(buf, len).map(|aaaa| RData::AAAA(aaaa)),
            RRType::CNAME => {
                rdata_cname::CName::from_wire(buf, len).map(|cname| RData::CName(cname))
            }
            RRType::SOA => rdata_soa::SOA::from_wire(buf, len).map(|soa| RData::SOA(soa)),
            RRType::MX => rdata_mx::MX::from_wire(buf, len).map(|mx| RData::MX(mx)),
            RRType::NAPTR => {
                rdata_naptr::NAPTR::from_wire(buf, len).map(|naptr| RData::NAPTR(naptr))
            }
            RRType::OPT => rdata_opt::OPT::from_wire(buf, len).map(|opt| RData::OPT(opt)),
            _ => Err(Error::UnknownRRType),
        };

        if rdata.is_ok() && buf.position() - pos != (len as usize) {
            Err(Error::RdataLenIsNotCorrect)
        } else {
            rdata
        }
    }

    pub fn rend(&self, render: &mut MessageRender) {
        match *self {
            RData::A(ref a) => a.rend(render),
            RData::NS(ref ns) => ns.rend(render),
            RData::AAAA(ref aaaa) => aaaa.rend(render),
            RData::CName(ref cname) => cname.rend(render),
            RData::SOA(ref soa) => soa.rend(render),
            RData::MX(ref mx) => mx.rend(render),
            RData::NAPTR(ref naptr) => naptr.rend(render),
            RData::OPT(ref opt) => opt.rend(render),
            _ => (),
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        match *self {
            RData::A(ref a) => a.to_wire(buf),
            RData::NS(ref ns) => ns.to_wire(buf),
            RData::AAAA(ref aaaa) => aaaa.to_wire(buf),
            RData::CName(ref cname) => cname.to_wire(buf),
            RData::SOA(ref soa) => soa.to_wire(buf),
            RData::MX(ref mx) => mx.to_wire(buf),
            RData::NAPTR(ref naptr) => naptr.to_wire(buf),
            RData::OPT(ref opt) => opt.to_wire(buf),
            _ => (),
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            RData::A(ref a) => a.to_string(),
            RData::NS(ref ns) => ns.to_string(),
            RData::AAAA(ref aaaa) => aaaa.to_string(),
            RData::CName(ref cname) => cname.to_string(),
            RData::SOA(ref soa) => soa.to_string(),
            RData::MX(ref mx) => mx.to_string(),
            RData::NAPTR(ref naptr) => naptr.to_string(),
            RData::OPT(ref opt) => opt.to_string(),
            _ => "".to_owned(),
        }
    }
}
