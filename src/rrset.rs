use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdata::RData;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::util::{InputBuffer, StringBuffer};
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
        if rdlen == 0 {
            if typ != RRType::OPT {
                bail!("only opt record could has zero rdata");
            }
        } else {
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

    pub fn from_strs(ss: &[&str]) -> Result<Self> {
        assert!(!ss.is_empty());

        let last = RRset::from_str(ss[0])?;
        if ss.len() == 1 {
            return Ok(last);
        }

        if last.typ == RRType::OPT {
            bail!("opt rrset can only has one rr");
        }

        ss[1..].iter().try_fold(last, |mut rrset, s| {
            let mut current = RRset::from_str(s)?;
            if rrset.typ != current.typ {
                bail!("rr in one rrset should has same type");
            }
            rrset.rdatas.push(current.rdatas.remove(0));
            Ok(rrset)
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

impl FromStr for RRset {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let mut labels = StringBuffer::new(s.trim());

        let name = if let Some(name_str) = labels.next() {
            Name::from_str(name_str)?
        } else {
            bail!("parse name failed");
        };

        let ttl = if let Some(ttl_str) = labels.next() {
            RRTtl::from_str(ttl_str)?
        } else {
            bail!("parse ttl failed");
        };

        let mut short_of_class = false;
        let cls_str = if let Some(cls_str) = labels.next() {
            cls_str
        } else {
            bail!("parse class failed");
        };

        let class = match RRClass::from_str(cls_str) {
            Ok(cls) => cls,
            Err(_) => {
                short_of_class = true;
                RRClass::IN
            }
        };

        let typ = if short_of_class {
            RRType::from_str(cls_str)?
        } else if let Some(typ_str) = labels.next() {
            RRType::from_str(typ_str)?
        } else {
            bail!("parse type failed");
        };

        let rdata = RData::from_buffer(typ, &mut labels)?;
        Ok(RRset {
            name,
            typ,
            class,
            ttl,
            rdatas: vec![rdata],
        })
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
