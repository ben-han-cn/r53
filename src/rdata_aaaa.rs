use std::net::Ipv6Addr;

use error::Error;
use message_render::MessageRender;
use util::{InputBuffer, OutputBuffer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AAAA {
    host: Ipv6Addr,
}

fn get_ipv6_addr(buf: &mut InputBuffer) -> Result<Ipv6Addr, Error> {
    buf.read_bytes(16).map(|bytes| {
        let mut octs = [0; 16];
        octs.copy_from_slice(bytes);
        Ipv6Addr::from(octs)
    })
}

impl AAAA {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self, Error> {
        get_ipv6_addr(buf).map(|addr| AAAA { host: addr })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        self.host
            .octets()
            .into_iter()
            .for_each(|x| render.write_u8(*x));
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.host
            .octets()
            .into_iter()
            .for_each(|x| buf.write_u8(*x));
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
    fn test_aaaa_to_wire() {
        let raw = from_hex("20010db8000000000000000000001234").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let aaaa = AAAA::from_wire(&mut buf, raw.len() as u16).unwrap();
        assert_eq!(Ok(aaaa.host), "2001:db8::1234".parse());

        let mut render = MessageRender::new();
        aaaa.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
        assert_eq!(aaaa.to_string(), "2001:db8::1234");
    }
}
