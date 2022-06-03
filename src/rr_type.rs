use crate::message_render::MessageRender;
use crate::util::InputBuffer;
use anyhow::{self, bail, Result};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum RRType {
    A,
    NS,
    CNAME,
    SOA,
    PTR,
    MX,
    TXT,
    RP,
    AAAA,
    SRV,
    NAPTR,
    CERT,
    DNAME,
    OPT,
    DS,
    RRSIG,
    NSEC,
    DNSKEY,
    NSEC3,
    NSEC3PARAM,
    TSIG,
    IXFR,
    AXFR,
    ANY,
    URI,
    CAA,
    Unknown(u16),
}

impl RRType {
    pub fn new(value: u16) -> Self {
        match value {
            1 => RRType::A,
            2 => RRType::NS,
            5 => RRType::CNAME,
            6 => RRType::SOA,
            12 => RRType::PTR,
            15 => RRType::MX,
            17 => RRType::RP,
            28 => RRType::AAAA,
            16 => RRType::TXT,
            33 => RRType::SRV,
            35 => RRType::NAPTR,
            37 => RRType::CERT,
            39 => RRType::DNAME,
            41 => RRType::OPT,
            43 => RRType::DS,
            46 => RRType::RRSIG,
            47 => RRType::NSEC,
            48 => RRType::DNSKEY,
            50 => RRType::NSEC3,
            51 => RRType::NSEC3PARAM,
            250 => RRType::TSIG,
            251 => RRType::IXFR,
            252 => RRType::AXFR,
            255 => RRType::ANY,
            256 => RRType::URI,
            257 => RRType::CAA,
            _ => RRType::Unknown(value),
        }
    }

    pub fn as_u16(&self) -> u16 {
        match *self {
            RRType::A => 1,
            RRType::NS => 2,
            RRType::CNAME => 5,
            RRType::SOA => 6,
            RRType::PTR => 12,
            RRType::MX => 15,
            RRType::TXT => 16,
            RRType::RP => 17,
            RRType::AAAA => 28,
            RRType::SRV => 33,
            RRType::NAPTR => 35,
            RRType::CERT => 37,
            RRType::DNAME => 39,
            RRType::OPT => 41,
            RRType::DS => 43,
            RRType::RRSIG => 46,
            RRType::NSEC => 47,
            RRType::DNSKEY => 48,
            RRType::NSEC3 => 50,
            RRType::NSEC3PARAM => 51,
            RRType::TSIG => 250,
            RRType::IXFR => 251,
            RRType::AXFR => 252,
            RRType::ANY => 255,
            RRType::URI => 256,
            RRType::CAA => 257,
            RRType::Unknown(c) => c,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            RRType::A => "A",
            RRType::NS => "NS",
            RRType::CNAME => "CNAME",
            RRType::SOA => "SOA",
            RRType::PTR => "PTR",
            RRType::MX => "MX",
            RRType::TXT => "TXT",
            RRType::RP => "RP",
            RRType::AAAA => "AAAA",
            RRType::SRV => "SRV",
            RRType::NAPTR => "NAPTR",
            RRType::CERT => "URI",
            RRType::DNAME => "DNAME",
            RRType::OPT => "OPT",
            RRType::DS => "DS",
            RRType::RRSIG => "RRSIG",
            RRType::NSEC => "NSEC",
            RRType::DNSKEY => "DNSKEY",
            RRType::NSEC3 => "NSEC3",
            RRType::NSEC3PARAM => "NSEC3PARAM",
            RRType::TSIG => "TSIG",
            RRType::IXFR => "IXFR",
            RRType::AXFR => "AXFR",
            RRType::ANY => "ANY",
            RRType::URI => "URI",
            RRType::CAA => "CAA",
            RRType::Unknown(_) => "Unknown",
        }
    }

    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        buf.read_u16().map(RRType::new)
    }

    pub fn to_wire(self, render: &mut MessageRender) -> Result<()> {
        render.write_u16(self.as_u16())
    }
}

impl fmt::Display for RRType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl FromStr for RRType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_ref() {
            "A" => Ok(RRType::A),
            "NS" => Ok(RRType::NS),
            "CNAME" => Ok(RRType::CNAME),
            "SOA" => Ok(RRType::SOA),
            "PTR" => Ok(RRType::PTR),
            "MX" => Ok(RRType::MX),
            "TXT" => Ok(RRType::TXT),
            "RP" => Ok(RRType::RP),
            "AAAA" => Ok(RRType::AAAA),
            "SRV" => Ok(RRType::SRV),
            "NAPTR" => Ok(RRType::NAPTR),
            "CERT" => Ok(RRType::CERT),
            "DNAME" => Ok(RRType::DNAME),
            "OPT" => Ok(RRType::OPT),
            "DS" => Ok(RRType::DS),
            "RRSIG" => Ok(RRType::RRSIG),
            "NSEC" => Ok(RRType::NSEC),
            "DNSKEY" => Ok(RRType::DNSKEY),
            "NSEC3" => Ok(RRType::NSEC3),
            "NSEC3PARAM" => Ok(RRType::NSEC3PARAM),
            "TSIG" => Ok(RRType::TSIG),
            "IXFR" => Ok(RRType::IXFR),
            "AXFR" => Ok(RRType::AXFR),
            "ANY" => Ok(RRType::ANY),
            "URI" => Ok(RRType::URI),
            "CAA" => Ok(RRType::CAA),
            _ => bail!("rr type {} doesn't support", s),
        }
    }
}

impl PartialOrd for RRType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.as_u16().cmp(&other.as_u16()))
    }
}

impl Ord for RRType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_u16().cmp(&other.as_u16())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_rrtype_equal() {
        assert_eq!(RRType::A.as_u16(), 1);
        assert_eq!(RRType::A.to_str(), "A");
    }
}
