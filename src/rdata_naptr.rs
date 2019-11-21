use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NAPTR {
    pub order: u16,
    pub preference: u16,
    pub flags: u16,
    pub services: u16,
    pub replacement: Name,
}

impl NAPTR {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        let order = buf.read_u16()?;
        let preference = buf.read_u16()?;
        let flags = buf.read_u16()?;
        let services = buf.read_u16()?;
        let replacement = Name::from_wire(buf)?;
        Ok(NAPTR {
            order,
            preference,
            flags,
            services,
            replacement,
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

    pub fn from_parser<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let order = iter.next_field::<u16>("NAPTR", "order")?;
        let preference = iter.next_field::<u16>("NAPTR", "preference")?;
        let flags = iter.next_field::<u16>("NAPTR", "flags")?;
        let services = iter.next_field::<u16>("NAPTR", "services")?;
        let replacement = iter.next_field::<Name>("NAPTR", "replacement")?;
        Ok(NAPTR {
            order,
            preference,
            flags,
            services,
            replacement,
        })
    }
}

impl fmt::Display for NAPTR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.order, self.preference, self.flags, self.services, self.replacement
        )
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_naptr_to_wire() {}
}
