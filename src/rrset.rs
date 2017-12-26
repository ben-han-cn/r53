use util::{InputBuffer, OutputBuffer};
use message_render::MessageRender;
use super::error::Error;
use name::Name;
use rr_type::RRType;
use rr_class::RRClass;
use rdata::RData;
use std::fmt::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RRTtl(u32);


impl RRTtl {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self, Error> {
        buf.read_u32().map(|n| RRTtl(n))
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_u32(self.0);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_u32(self.0);
    }

    fn to_string(&self) -> String {
        self.0.to_string()
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RRset {
    name: Name,
    typ: RRType,
    class: RRClass,
    ttl: RRTtl,
    rdatas: Vec<RData>,
}


impl RRset {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self, Error> {
        let name = Name::from_wire(buf, false)?;
        let typ = RRType::from_wire(buf)?;
        let class = RRClass::from_wire(buf)?;
        let ttl = RRTtl::from_wire(buf)?;
        let rdlen = buf.read_u16()?;
        let mut rdatas = Vec::with_capacity(1);
        if rdlen > 0 {
            let rdata = RData::from_wire(typ, buf, rdlen)?;
            rdatas.push(rdata);
        }
        Ok(RRset {
            name: name,
            typ: typ,
            class: class,
            ttl: ttl,
            rdatas: rdatas,
        })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        if self.rdatas.len() == 0 {
            self.name.rend(render);
            self.typ.rend(render);
            self.class.rend(render);
            self.ttl.rend(render);
            render.write_u16(0)
        } else {
            self.rdatas.iter().for_each(|rdata| {
                self.name.rend(render);
                self.typ.rend(render);
                self.class.rend(render);
                self.ttl.rend(render);
                let pos = render.len();
                render.skip(2);
                rdata.rend(render);
                let rdlen = render.len() - pos - 2;
                render.write_u16_at(rdlen as u16, pos);
            })
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        if self.rdatas.len() == 0 {
            self.name.to_wire(buf);
            self.typ.to_wire(buf);
            self.class.to_wire(buf);
            self.ttl.to_wire(buf);
            buf.write_u16(0)
        } else {
            self.rdatas.iter().for_each(|rdata| {
                self.name.to_wire(buf);
                self.typ.to_wire(buf);
                self.class.to_wire(buf);
                self.ttl.to_wire(buf);
                let pos = buf.len();
                buf.skip(2);
                rdata.to_wire(buf);
                let rdlen = buf.len() - pos - 2;
                buf.write_u16_at(rdlen as u16, pos);
            })
        }
    }

    pub fn to_string(&self) -> String {
        let mut rrset_str = String::new();
        self.rdatas.iter().for_each(|rdata| {
            write!(&mut rrset_str, "{}\t{}\t", self.header(), rdata.to_string()).unwrap();
        });
        rrset_str
    }

    fn header(&self) -> String {
        [
            self.name.to_string(),
            self.ttl.to_string(),
            self.class.to_string(),
            self.typ.to_string(),
        ].join("\t")
    }
}
