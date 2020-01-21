use crate::message_render::MessageRender;
use crate::util::{InputBuffer, OutputBuffer};
use anyhow::{self, bail, Result};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[allow(dead_code)]
pub enum RRClass {
    IN,
    CH,
    HS,
    NONE,
    ANY,
    Unknown(u16),
}

impl RRClass {
    pub fn new(value: u16) -> Self {
        match value {
            1 => RRClass::IN,
            3 => RRClass::CH,
            4 => RRClass::HS,
            254 => RRClass::NONE,
            255 => RRClass::ANY,
            code => RRClass::Unknown(code),
        }
    }

    pub fn to_u16(self) -> u16 {
        match self {
            RRClass::IN => 1,
            RRClass::CH => 3,
            RRClass::HS => 4,
            RRClass::NONE => 254,
            RRClass::ANY => 255,
            RRClass::Unknown(code) => code,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            RRClass::IN => "IN",
            RRClass::CH => "CH",
            RRClass::HS => "HS",
            RRClass::NONE => "NONE",
            RRClass::ANY => "ANY",
            RRClass::Unknown(_) => "Unknown",
        }
    }

    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        buf.read_u16().map(RRClass::new)
    }

    pub fn to_wire(self, render: &mut MessageRender) {
        render.write_u16(self.to_u16());
    }
}

impl FromStr for RRClass {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_ref() {
            "IN" => Ok(RRClass::IN),
            "CH" => Ok(RRClass::CH),
            "HS" => Ok(RRClass::HS),
            "NONE" => Ok(RRClass::NONE),
            "ANY" => Ok(RRClass::ANY),
            _ => bail!("invalid class string {}", s),
        }
    }
}

impl fmt::Display for RRClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_rrclass_equal() {
        assert_eq!(RRClass::IN.to_u16(), 1);
        assert_eq!(RRClass::IN.to_str(), "IN");
    }
}
