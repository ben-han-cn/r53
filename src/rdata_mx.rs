use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

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

    pub fn from_str<'a>(iter: &mut Parser<'a>) -> Result<Self> {
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

    pub fn to_string(&self) -> String {
        [self.preference.to_string(), self.name.to_string()].join(" ")
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_mx_to_wire() {}
}
