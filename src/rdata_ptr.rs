use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PTR {
    pub name: Name,
}

impl PTR {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        Name::from_wire(buf).map(|name| PTR { name })
    }

    pub fn from_parser<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let name = iter.next_field::<Name>("PTR", "Name")?;
        Ok(PTR { name })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_name(&self.name, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.name.to_wire(buf);
    }
}

impl fmt::Display for PTR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_ptr_to_wire() {}
}
