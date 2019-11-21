use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DName {
    pub target: Name,
}

impl DName {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        Name::from_wire(buf).map(|name| DName { target: name })
    }

    pub fn from_parser<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let target = iter.next_field::<Name>("DName", "Name")?;
        Ok(DName { target })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_name(&self.target, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.target.to_wire(buf);
    }
}

impl fmt::Display for DName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.target)
    }
}
