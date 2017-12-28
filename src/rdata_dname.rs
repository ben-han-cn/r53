use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;
use name::Name;
use super::error::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DName {
    target: Name,
}

impl DName {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self, Error> {
        Name::from_wire(buf, false).map(|name| DName { target: name })
    }

    pub fn from_string(name_str: &str) -> Result<Self, Error> {
        let name = Name::new(name_str, false)?;
        Ok(DName { target: name })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_name(&self.target, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.target.to_wire(buf);
    }

    pub fn to_string(&self) -> String {
        self.target.to_string()
    }
}
