use std::fmt::{self, Display};
use std::str::FromStr;

use anyhow::bail;

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

impl Display for Rcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl FromStr for Rcode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_ref() {
            "NOERROR" => Ok(Rcode::NoError),
            "FORMERR" => Ok(Rcode::FormErr),
            "SERVFAIL" => Ok(Rcode::ServFail),
            "NXDOMAIN" => Ok(Rcode::NXDomain),
            "NOTIMP" => Ok(Rcode::NotImp),
            "REFUSED" => Ok(Rcode::Refused),
            "YXDOMAIN" => Ok(Rcode::YXDomain),
            "YXRRSET" => Ok(Rcode::YXRRset),
            "NXRRSET" => Ok(Rcode::NXRRset),
            "NOTAUTH" => Ok(Rcode::NotAuth),
            "NOTZONE" => Ok(Rcode::NotZone),
            "RESERVED" => Ok(Rcode::Reserved),
            _ => bail!("unknow rcode {}", s),
        }
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

    #[test]
    pub fn test_rcode_from_str() {
        assert_eq!("noerror".parse::<Rcode>().unwrap(), Rcode::NoError);
        assert_eq!("NOERROR".parse::<Rcode>().unwrap(), Rcode::NoError);
        assert!("OERROR".parse::<Rcode>().is_err());
    }
}
