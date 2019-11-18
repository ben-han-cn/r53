use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Rcode {
    NoError,
    FormErr,
    ServFail,
    NXDomain,
    NotImp,
    Refused,
    YXDomain,
    YXRRset,
    NXRRset,
    NotAuth,
    NotZone,
    Reserved,
}

impl Rcode {
    pub fn new(value: u8) -> Self {
        match value {
            0 => Rcode::NoError,
            1 => Rcode::FormErr,
            2 => Rcode::ServFail,
            3 => Rcode::NXDomain,
            4 => Rcode::NotImp,
            5 => Rcode::Refused,
            6 => Rcode::YXDomain,
            7 => Rcode::YXRRset,
            8 => Rcode::NXRRset,
            9 => Rcode::NotAuth,
            10 => Rcode::NotZone,
            _ => Rcode::Reserved,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            Rcode::NoError => 0,
            Rcode::FormErr => 1,
            Rcode::ServFail => 2,
            Rcode::NXDomain => 3,
            Rcode::NotImp => 4,
            Rcode::Refused => 5,
            Rcode::YXDomain => 6,
            Rcode::YXRRset => 7,
            Rcode::NXRRset => 8,
            Rcode::NotAuth => 9,
            Rcode::NotZone => 10,
            Rcode::Reserved => 11,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Rcode::NoError => "NOERROR",
            Rcode::FormErr => "FORMERR",
            Rcode::ServFail => "SERVFAIL",
            Rcode::NXDomain => "NXDOMAIN",
            Rcode::NotImp => "NOTIMP",
            Rcode::Refused => "REFUSED",
            Rcode::YXDomain => "YXDOMAIN",
            Rcode::YXRRset => "YXRRSET",
            Rcode::NXRRset => "NXRRSET",
            Rcode::NotAuth => "NOTAUTH",
            Rcode::NotZone => "NOTZONE",
            Rcode::Reserved => "RESERVED",
        }
    }
}

impl fmt::Display for Rcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_rcode_equal() {
        assert_eq!(Rcode::NoError.to_u8(), 0);
        assert_eq!(Rcode::NoError.to_string(), "NOERROR");
    }
}
