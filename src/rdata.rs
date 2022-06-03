use crate::message_render::MessageRender;
use crate::rdatas;
use crate::rr_type::RRType;
use crate::util::{InputBuffer, StringBuffer};
use anyhow::{bail, Result};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum RData {
    A(rdatas::A),
    AAAA(rdatas::AAAA),
    NS(Box<rdatas::NS>),
    CName(Box<rdatas::CName>),
    SOA(Box<rdatas::SOA>),
    PTR(Box<rdatas::PTR>),
    MX(Box<rdatas::MX>),
    TXT(Box<rdatas::TXT>),
    RP(Box<rdatas::RP>),
    NAPTR(Box<rdatas::NAPTR>),
    CERT(Box<rdatas::CERT>),
    DNAME(Box<rdatas::DName>),
    OPT(Box<rdatas::OPT>),
    SRV(Box<rdatas::SRV>),
    DS(Box<rdatas::DS>),
    RRSig(Box<rdatas::RRSig>),
    NSEC(Box<rdatas::NSEC>),
    DNSKey(Box<rdatas::DNSKey>),
    NSEC3(Box<rdatas::NSEC3>),
    NSEC3Param(Box<rdatas::NSEC3Param>),
    URI(Box<rdatas::URI>),
    CAA(Box<rdatas::CAA>),
}

macro_rules! impl_coder_for_rdata {
    ($([$rr_type:pat, $rdata_type:ty, $rdata_varient:path]),+) => {
        impl RData {
            pub fn from_wire(typ: RRType, buf: &mut InputBuffer, len: u16) -> Result<Self> {
                let pos = buf.position();
                let rdata = match typ {
                    RRType::A => rdatas::A::from_wire(buf, len).map(RData::A),
                    RRType::AAAA => rdatas::AAAA::from_wire(buf, len).map(RData::AAAA),
                    $($rr_type => <$rdata_type>::from_wire(buf, len).map(|rd| $rdata_varient(Box::new(rd))),)+
                        _ => bail!("unknown rr type {}", typ.as_u16()),
                };
                if rdata.is_ok() && buf.position() - pos != (len as usize) {
                    bail!("rdata len isn't correct");
                }
                rdata
            }

            pub fn from_str(typ: RRType, s: &str) -> Result<Self> {
                let mut buf = StringBuffer::new(s);
                Self::from_string_buffer(typ, &mut buf)
            }

            pub(crate) fn from_string_buffer(typ: RRType, buf: &mut StringBuffer) -> Result<Self> {
                match typ {
                    RRType::A => rdatas::A::from_str(buf).map(RData::A),
                    RRType::AAAA => rdatas::AAAA::from_str(buf).map(RData::AAAA),
                    $($rr_type => <$rdata_type>::from_str(buf).map(|rd| $rdata_varient(Box::new(rd))),)+
                        _ => bail!("unknown rr type {}", typ.as_u16()),
                }
            }

            pub fn to_wire(&self, render: &mut MessageRender) -> Result<()> {
                match *self {
                    RData::A(ref a) => a.to_wire(render),
                    RData::AAAA(ref aaaa) => aaaa.to_wire(render),
                    $($rdata_varient(ref rd) => rd.to_wire(render),)+
                }
            }
        }

        impl fmt::Display for RData {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    RData::A(ref a) => write!(f, "{}", a),
                    RData::AAAA(ref aaaa) => write!(f, "{}", aaaa),
                    $($rdata_varient(ref rd) => write!(f, "{}", rd),)+
                }
            }
        }
    }
}

impl_coder_for_rdata!(
    [RRType::NS, rdatas::NS, RData::NS],
    [RRType::CNAME, rdatas::CName, RData::CName],
    [RRType::SOA, rdatas::SOA, RData::SOA],
    [RRType::PTR, rdatas::PTR, RData::PTR],
    [RRType::MX, rdatas::MX, RData::MX],
    [RRType::TXT, rdatas::TXT, RData::TXT],
    [RRType::RP, rdatas::RP, RData::RP],
    [RRType::NAPTR, rdatas::NAPTR, RData::NAPTR],
    [RRType::CERT, rdatas::CERT, RData::CERT],
    [RRType::OPT, rdatas::OPT, RData::OPT],
    [RRType::DNAME, rdatas::DName, RData::DNAME],
    [RRType::SRV, rdatas::SRV, RData::SRV],
    [RRType::DS, rdatas::DS, RData::DS],
    [RRType::RRSIG, rdatas::RRSig, RData::RRSig],
    [RRType::NSEC, rdatas::NSEC, RData::NSEC],
    [RRType::DNSKEY, rdatas::DNSKey, RData::DNSKey],
    [RRType::NSEC3, rdatas::NSEC3, RData::NSEC3],
    [RRType::NSEC3PARAM, rdatas::NSEC3Param, RData::NSEC3Param],
    [RRType::NSEC3PARAM, rdatas::NSEC3Param, RData::NSEC3Param],
    [RRType::URI, rdatas::URI, RData::URI],
    [RRType::CAA, rdatas::CAA, RData::CAA]
);
