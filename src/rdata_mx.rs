use error::Error;
use message_render::MessageRender;
use name::Name;
use util::{InputBuffer, OutputBuffer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MX {
    preference: u16,
    name: Name,
}

impl MX {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self, Error> {
        let preference = buf.read_u16()?;
        let name = Name::from_wire(buf, false)?;
        Ok(MX {
            preference: preference,
            name: name,
        })
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
