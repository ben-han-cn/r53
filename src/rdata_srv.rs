use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SRV {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: Name,
}

impl SRV {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        let priority = buf.read_u16()?;
        let weight = buf.read_u16()?;
        let port = buf.read_u16()?;
        let target = Name::from_wire(buf)?;
        Ok(SRV {
            priority,
            weight,
            port,
            target,
        })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_u16(self.priority);
        render.write_u16(self.weight);
        render.write_u16(self.port);
        render.write_name(&self.target, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_u16(self.priority);
        buf.write_u16(self.weight);
        buf.write_u16(self.port);
        self.target.to_wire(buf);
    }

    pub fn to_string(&self) -> String {
        [
            self.priority.to_string(),
            self.weight.to_string(),
            self.port.to_string(),
            self.target.to_string(),
        ]
        .join(" ")
    }

    pub fn from_str<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let priority = iter.next_field::<u16>("SRV", "priority")?;
        let weight = iter.next_field::<u16>("SRV", "weight")?;
        let port = iter.next_field::<u16>("SRV", "port")?;
        let target = iter.next_field::<Name>("SRV", "target")?;
        Ok(SRV {
            priority,
            weight,
            port,
            target,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::hex::from_hex;

    #[test]
    fn test_srv_to_wire() {
        //12 10 53 www.baidu.com.
        //000c000a003503777705626169647503636f6d00
        let raw = from_hex("000c000a00350377777705626169647503636f6d00").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let srv = SRV::from_wire(&mut buf, raw.len() as u16).unwrap();
        assert_eq!(Ok(srv.priority), "12".parse());
        assert_eq!(Ok(srv.weight), "10".parse());
        assert_eq!(Ok(srv.port), "53".parse());
        assert_eq!(&srv.target, &Name::new("www.baidu.com").unwrap());

        let mut render = MessageRender::new();
        srv.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
        assert_eq!(srv.to_string(), "12 10 53 www.baidu.com.");
    }
}
