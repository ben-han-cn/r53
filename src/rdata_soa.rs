use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use anyhow::Result;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SOA {
    pub mname: Name,
    pub rname: Name,
    pub serial: u32,
    pub refresh: u32,
    pub retry: u32,
    pub expire: u32,
    pub minimum: u32,
}

impl SOA {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        let mname = Name::from_wire(buf)?;
        let rname = Name::from_wire(buf)?;
        let serial = buf.read_u32()?;
        let refresh = buf.read_u32()?;
        let retry = buf.read_u32()?;
        let expire = buf.read_u32()?;
        let minimum = buf.read_u32()?;
        Ok(SOA {
            mname,
            rname,
            serial,
            refresh,
            retry,
            expire,
            minimum,
        })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_name(&self.mname, true);
        render.write_name(&self.rname, true);
        render.write_u32(self.serial);
        render.write_u32(self.refresh);
        render.write_u32(self.retry);
        render.write_u32(self.expire);
        render.write_u32(self.minimum);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.mname.to_wire(buf);
        self.rname.to_wire(buf);
        buf.write_u32(self.serial);
        buf.write_u32(self.refresh);
        buf.write_u32(self.retry);
        buf.write_u32(self.expire);
        buf.write_u32(self.minimum);
    }

    pub fn from_parser<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let mname = iter.next_field::<Name>("SOA", "mname")?;
        let rname = iter.next_field::<Name>("SOA", "rname")?;
        let serial = iter.next_field::<u32>("SOA", "serial")?;
        let refresh = iter.next_field::<u32>("SOA", "refresh")?;
        let retry = iter.next_field::<u32>("SOA", "retry")?;
        let expire = iter.next_field::<u32>("SOA", "expire")?;
        let minimum = iter.next_field::<u32>("SOA", "minimum")?;
        Ok(SOA {
            mname,
            rname,
            serial,
            refresh,
            retry,
            expire,
            minimum,
        })
    }
}

impl fmt::Display for SOA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.mname,
            self.rname,
            self.serial,
            self.refresh,
            self.retry,
            self.expire,
            self.minimum,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::hex::from_hex;

    #[test]
    fn test_soa_to_wire() {
        let raw =
            from_hex("002b026e73076578616d706c6503636f6d0004726f6f74c00577ce5bb900000e100000012c0036ee80000004b0").unwrap();
        let raw_len = (raw.len() - 2) as u16;
        let mut buf = InputBuffer::new(raw.as_slice());
        buf.set_position(2);
        let soa = SOA::from_wire(&mut buf, raw_len).unwrap();
        assert_eq!(soa.mname.to_string(), "ns.example.com.");
        assert_eq!(soa.rname.to_string(), "root.example.com.");
        assert_eq!(soa.serial, 2010012601);
        assert_eq!(soa.refresh, 3600);
        assert_eq!(soa.retry, 300);
        assert_eq!(soa.expire, 3600000);
        assert_eq!(soa.minimum, 1200);

        let mut render = MessageRender::new();
        render.write_u16(raw_len);
        soa.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
        assert_eq!(
            soa.to_string(),
            "ns.example.com. root.example.com. 2010012601 3600 300 3600000 1200"
        );
    }
}
