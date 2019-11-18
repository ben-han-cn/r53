use crate::error::DNSError;
use crate::message_render::MessageRender;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
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
    AAAA,
    SRV,
    NAPTR,
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
            28 => RRType::AAAA,
            16 => RRType::TXT,
            33 => RRType::SRV,
            35 => RRType::NAPTR,
            39 => RRType::DNAME,
            41 => RRType::OPT,
            43 => RRType::DS,
            46 => RRType::RRSIG,
            47 => RRType::NSEC,
            48 => RRType::DNSKEY,
            50 => RRType::NSEC3,
            51 => RRType::NSEC3PARAM,
            250 => RRType::TSIG,
            252 => RRType::AXFR,
            255 => RRType::ANY,
            _ => RRType::Unknown(value),
        }
    }

    pub fn to_u16(self) -> u16 {
        match self {
            RRType::A => 1,
            RRType::NS => 2,
            RRType::CNAME => 5,
            RRType::SOA => 6,
            RRType::PTR => 12,
            RRType::MX => 15,
            RRType::TXT => 16,
            RRType::AAAA => 28,
            RRType::SRV => 33,
            RRType::NAPTR => 35,
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
            RRType::AAAA => "AAAA",
            RRType::SRV => "SRV",
            RRType::NAPTR => "NAPTR",
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
            RRType::Unknown(_) => "Unknown",
        }
    }

    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        buf.read_u16().map(RRType::new)
    }

    pub fn rend(self, render: &mut MessageRender) {
        render.write_u16(self.to_u16());
    }

    pub fn to_wire(self, buf: &mut OutputBuffer) {
        buf.write_u16(self.to_u16());
    }
}

impl fmt::Display for RRType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl FromStr for RRType {
    type Err = failure::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_ref() {
            "A" => Ok(RRType::A),
            "NS" => Ok(RRType::NS),
            "CNAME" => Ok(RRType::CNAME),
            "SOA" => Ok(RRType::SOA),
            "PTR" => Ok(RRType::PTR),
            "MX" => Ok(RRType::MX),
            "TXT" => Ok(RRType::TXT),
            "AAAA" => Ok(RRType::AAAA),
            "SRV" => Ok(RRType::SRV),
            "NAPTR" => Ok(RRType::NAPTR),
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
            _ => Err(DNSError::UnknownRRType(0).into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_rrtype_equal() {
        assert_eq!(RRType::A.to_u16(), 1);
        assert_eq!(RRType::A.to_str(), "A");
    }
}
