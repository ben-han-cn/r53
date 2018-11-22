use error::Error;
use message_render::MessageRender;
use name::Name;
use util::{InputBuffer, OutputBuffer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SOA {
    mname: Name,
    rname: Name,
    serial: u32,
    refresh: u32,
    retry: u32,
    expire: u32,
    minimum: u32,
}

impl SOA {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self, Error> {
        let mname = Name::from_wire(buf, false)?;
        let rname = Name::from_wire(buf, false)?;
        let serial = buf.read_u32()?;
        let refresh = buf.read_u32()?;
        let retry = buf.read_u32()?;
        let expire = buf.read_u32()?;
        let minimum = buf.read_u32()?;
        Ok(SOA {
            mname: mname,
            rname: rname,
            serial: serial,
            refresh: refresh,
            retry: retry,
            expire: expire,
            minimum: minimum,
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

    pub fn to_string(&self) -> String {
        [
            self.mname.to_string(),
            self.rname.to_string(),
            self.serial.to_string(),
            self.refresh.to_string(),
            self.retry.to_string(),
            self.expire.to_string(),
            self.minimum.to_string(),
        ]
            .join(" ")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use util::hex::from_hex;

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
