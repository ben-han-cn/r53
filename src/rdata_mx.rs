use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MX {
    pub preference: u16,
    pub name: Name,
}

impl MX {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        let preference = buf.read_u16()?;
        let name = Name::from_wire(buf)?;
        Ok(MX { preference, name })
    }

    pub fn from_parser<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let preference = iter.next_field::<u16>("MX", "preference")?;
        let name = iter.next_field::<Name>("MX", "name")?;
        Ok(MX { preference, name })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_u16(self.preference);
        render.write_name(&self.name, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_u16(self.preference);
        self.name.to_wire(buf);
    }
}

impl fmt::Display for MX {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.preference, self.name)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_mx_to_wire() {}
}
