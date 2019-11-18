use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PTR {
    pub name: Name,
}

impl PTR {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        Name::from_wire(buf).map(|name| PTR { name })
    }

    pub fn from_str<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let name = iter.next_field::<Name>("PTR", "Name")?;
        Ok(PTR { name })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_name(&self.name, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.name.to_wire(buf);
    }

    pub fn to_string(&self) -> String {
        self.name.to_string()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_ptr_to_wire() {}
}
