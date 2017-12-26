use std::fmt;
use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;
use super::error::Error;

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
            28 => RRType::AAAA,
            255 => RRType::ANY,
            252 => RRType::AXFR,
            5 => RRType::CNAME,
            15 => RRType::MX,
            2 => RRType::NS,
            41 => RRType::OPT,
            12 => RRType::PTR,
            6 => RRType::SOA,
            33 => RRType::SRV,
            16 => RRType::TXT,
            48 => RRType::DNSKEY,
            43 => RRType::DS,
            47 => RRType::NSEC,
            50 => RRType::NSEC3,
            51 => RRType::NSEC3PARAM,
            46 => RRType::RRSIG,
            250 => RRType::TSIG,
            35 => RRType::NAPTR,
            _ => RRType::Unknown(value),
        }
    }

    pub fn to_u16(&self) -> u16 {
        match *self {
            RRType::A => 1,
            RRType::NS => 2,
            RRType::CNAME => 5,
            RRType::SOA => 6,
            RRType::AAAA => 28,
            RRType::ANY => 255,
            RRType::AXFR => 252,
            RRType::IXFR => 251,
            RRType::MX => 15,
            RRType::OPT => 41,
            RRType::PTR => 12,
            RRType::SRV => 33,
            RRType::TXT => 16,
            RRType::DNSKEY => 48,
            RRType::DS => 43,
            RRType::NSEC => 47,
            RRType::NSEC3 => 50,
            RRType::NSEC3PARAM => 51,
            RRType::RRSIG => 46,
            RRType::TSIG => 250,
            RRType::NAPTR => 35,
            RRType::Unknown(c) => c,
        }
    }

    fn to_string(&self) -> &'static str {
        match *self {
            RRType::A => "A",
            RRType::AAAA => "AAAA",
            RRType::ANY => "ANY",
            RRType::AXFR => "AXFR",
            RRType::CNAME => "CNAME",
            RRType::IXFR => "IXFR",
            RRType::MX => "MX",
            RRType::NS => "NS",
            RRType::OPT => "OPT",
            RRType::PTR => "PTR",
            RRType::SOA => "SOA",
            RRType::SRV => "SRV",
            RRType::TXT => "TXT",
            RRType::DNSKEY => "DNSKEY",
            RRType::DS => "DS",
            RRType::NSEC => "NSEC",
            RRType::NSEC3 => "NSEC3",
            RRType::NSEC3PARAM => "NSEC3PARAM",
            RRType::RRSIG => "RRSIG",
            RRType::TSIG => "TSIG",
            RRType::NAPTR => "NAPTR",
            RRType::Unknown(_) => "Unknown",
        }
    }

    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self, Error> {
        buf.read_u16().map(|n| RRType::new(n))
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_u16(self.to_u16());
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_u16(self.to_u16());
    }
}

impl fmt::Display for RRType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_rrtype_equal() {
        assert_eq!(RRType::A.to_u16(), 1);
    }
}
