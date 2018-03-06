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

    pub fn to_u16(&self) -> u16 {
        match *self {
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

    fn to_string(&self) -> &'static str {
        match *self {
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

    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self, Error> {
        buf.read_u16().map(|n| RRType::new(n))
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_ref() {
            "A" => Some(RRType::A),
            "NS" => Some(RRType::NS),
            "CNAME" => Some(RRType::CNAME),
            "SOA" => Some(RRType::SOA),
            "PTR" => Some(RRType::PTR),
            "MX" => Some(RRType::MX),
            "TXT" => Some(RRType::TXT),
            "AAAA" => Some(RRType::AAAA),
            "SRV" => Some(RRType::SRV),
            "NAPTR" => Some(RRType::NAPTR),
            "DNAME" => Some(RRType::DNAME),
            "OPT" => Some(RRType::OPT),
            "DS" => Some(RRType::DS),
            "RRSIG" => Some(RRType::RRSIG),
            "NSEC" => Some(RRType::NSEC),
            "DNSKEY" => Some(RRType::DNSKEY),
            "NSEC3" => Some(RRType::NSEC3),
            "NSEC3PARAM" => Some(RRType::NSEC3PARAM),
            "TSIG" => Some(RRType::TSIG),
            "IXFR" => Some(RRType::IXFR),
            "AXFR" => Some(RRType::AXFR),
            "ANY" => Some(RRType::ANY),
            _ => None,
        }
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
