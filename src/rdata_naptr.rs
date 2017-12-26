use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;
use name::Name;
use super::error::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NAPTR {
    order: u16,
    preference: u16,
    flags: u16,
    services: u16,
    replacement: Name,
}

impl NAPTR {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self, Error> {
        let order = buf.read_u16()?;
        let preference = buf.read_u16()?;
        let flags = buf.read_u16()?;
        let services = buf.read_u16()?;
        let replacement = Name::from_wire(buf, false)?;
        Ok(NAPTR {
            order: order,
            preference: preference,
            flags: flags,
            services: services,
            replacement: replacement,
        })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_u16(self.order);
        render.write_u16(self.preference);
        render.write_u16(self.flags);
        render.write_u16(self.services);
        render.write_name(&self.replacement, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_u16(self.order);
        buf.write_u16(self.preference);
        buf.write_u16(self.flags);
        buf.write_u16(self.services);
        self.replacement.to_wire(buf);
    }

    pub fn to_string(&self) -> String {
        [
            self.order.to_string(),
            self.preference.to_string(),
            self.flags.to_string(),
            self.services.to_string(),
            self.replacement.to_string(),
        ].join(" ")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use util::hex::from_hex;

    #[test]
    fn test_naptr_to_wire() {}
}
