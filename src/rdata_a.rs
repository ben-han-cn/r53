use crate::message_render::MessageRender;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::fmt;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct A {
    pub host: Ipv4Addr,
}

fn get_ipv4_addr(buf: &mut InputBuffer) -> Result<Ipv4Addr> {
    buf.read_bytes(4)
        .map(|bytes| Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]))
}

impl A {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        get_ipv4_addr(buf).map(|addr| A { host: addr })
    }

    pub fn from_parser<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let ip = iter.next_field::<Ipv4Addr>("A", "Host")?;
        Ok(A { host: ip })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        let segments = self.host.octets();
        render.write_u8(segments[0]);
        render.write_u8(segments[1]);
        render.write_u8(segments[2]);
        render.write_u8(segments[3]);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        let segments = self.host.octets();
        buf.write_u8(segments[0]);
        buf.write_u8(segments[1]);
        buf.write_u8(segments[2]);
        buf.write_u8(segments[3]);
    }
}

impl fmt::Display for A {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.host)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::hex::from_hex;

    #[test]
    fn test_a_to_wire() {
        let raw = from_hex("c0000201").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let a = A::from_wire(&mut buf, raw.len() as u16).unwrap();
        assert_eq!(Ok(a.host), "192.0.2.1".parse());

        let mut render = MessageRender::new();
        a.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
        assert_eq!(a.to_string(), "192.0.2.1");
    }
}
