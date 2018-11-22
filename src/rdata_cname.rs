use error::Error;
use message_render::MessageRender;
use name::Name;
use util::{InputBuffer, OutputBuffer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CName {
    name: Name,
}

impl CName {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self, Error> {
        Name::from_wire(buf, false).map(|name| CName { name: name })
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
    fn test_cname_to_wire() {}
}
