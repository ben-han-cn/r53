use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdata::RData;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::util::{InputBuffer, OutputBuffer};
use anyhow::{self, bail, Result};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RRTtl(pub u32);

impl RRTtl {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        buf.read_u32().map(RRTtl)
    }

    pub fn to_wire(self, render: &mut MessageRender) {
        render.write_u32(self.0);
    }
}

impl FromStr for RRTtl {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s.parse::<u32>() {
            Ok(num) => Ok(RRTtl(num)),
            Err(e) => bail!("ttl isn't a valid number:{}", e),
        }
    }
}

impl fmt::Display for RRTtl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RRset {
    pub name: Name,
    pub typ: RRType,
    pub class: RRClass,
    pub ttl: RRTtl,
    pub rdatas: Vec<RData>,
}

impl RRset {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        let name = Name::from_wire(buf)?;
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
            name,
            typ,
            class,
            ttl,
            rdatas,
        })
    }

    pub fn to_wire(&self, render: &mut MessageRender) {
        if self.rdatas.is_empty() {
            self.name.to_wire(render);
            self.typ.to_wire(render);
            self.class.to_wire(render);
            self.ttl.to_wire(render);
            render.write_u16(0)
        } else {
            self.rdatas.iter().for_each(|rdata| {
                self.name.to_wire(render);
                self.typ.to_wire(render);
                self.class.to_wire(render);
                self.ttl.to_wire(render);
                let pos = render.len();
                render.skip(2);
                rdata.to_wire(render);
                let rdlen = render.len() - pos - 2;
                render.write_u16_at(rdlen as u16, pos);
            })
        }
    }

    fn header(&self) -> String {
        [
            self.name.to_string(),
            self.ttl.to_string(),
            self.class.to_string(),
            self.typ.to_string(),
        ]
        .join("\t")
    }

    pub fn rr_count(&self) -> usize {
        self.rdatas.len()
    }

    pub fn is_same_rrset(&self, other: &RRset) -> bool {
        self.typ == other.typ && self.name.eq(&other.name)
    }
}

impl fmt::Display for RRset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.rdatas
            .iter()
            .map(|rdata| write!(f, "{}\t{}\n", self.header(), rdata))
            .collect()
    }
}
