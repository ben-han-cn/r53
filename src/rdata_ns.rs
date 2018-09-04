use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;
use name::Name;
use error::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NS {
    name: Name,
}

impl NS {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self, Error> {
        Name::from_wire(buf, false).map(|name| NS { name: name })
    }

    pub fn from_string(name_str: &str) -> Result<Self, Error> {
        let name = Name::new(name_str, false)?;
        Ok(NS { name: name })
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
    use super::*;
    use util::hex::from_hex;

    #[test]
    fn test_ns_to_wire() {
        let raw = from_hex("0474657374076578616d706c6503636f6d00").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let ns = NS::from_wire(&mut buf, raw.len() as u16).unwrap();
        assert_eq!(&ns.name, &Name::new("test.example.com", false).unwrap());

        let mut render = MessageRender::new();
        ns.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
        assert_eq!(ns.to_string(), "test.example.com.");
    }
}
