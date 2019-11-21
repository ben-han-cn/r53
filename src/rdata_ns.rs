use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NS {
    pub name: Name,
}

impl NS {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        Name::from_wire(buf).map(|name| NS { name })
    }

    pub fn from_parser<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let name = iter.next_field::<Name>("NS", "name")?;
        Ok(NS { name })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_name(&self.name, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.name.to_wire(buf);
    }
}

impl fmt::Display for NS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::hex::from_hex;

    #[test]
    fn test_ns_to_wire() {
        let raw = from_hex("0474657374076578616d706c6503636f6d00").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let ns = NS::from_wire(&mut buf, raw.len() as u16).unwrap();
        assert_eq!(&ns.name, &Name::new("test.example.com").unwrap());

        let mut render = MessageRender::new();
        ns.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
        assert_eq!(ns.to_string(), "test.example.com.");
    }
}
