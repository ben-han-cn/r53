use std::net::Ipv4Addr;

use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;
use super::error::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct A {
    host: Ipv4Addr,
}

fn get_ipv4_addr(buf: &mut InputBuffer) -> Result<Ipv4Addr> {
    buf.read_bytes(4).map(|bytes| {
        Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])
    })
}

impl A {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        get_ipv4_addr(buf).map(|addr| A { host: addr })
    }

    pub fn from_string(ip_str: &str) -> Result<Self> {
        match ip_str.parse() {
            Ok(ip) => Ok(A{host: ip}),
            Err(_) => Err(ErrorKind::InvalidIPv4Address.into())
        }
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

    pub fn to_string(&self) -> String {
        format!("{}", self.host)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use util::hex::from_hex;

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
