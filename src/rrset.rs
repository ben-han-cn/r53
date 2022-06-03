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

    pub fn to_wire(self, render: &mut MessageRender) -> Result<()> {
        render.write_u32(self.0)
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

#[derive(Debug, Clone)]
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

    pub fn from_strs<T: AsRef<str>>(ss: &[T]) -> Result<Self> {
        assert!(!ss.is_empty());

        let last = RRset::from_str(ss[0].as_ref())?;
        if ss.len() == 1 {
            return Ok(last);
        }

        if last.typ == RRType::OPT {
            bail!("opt rrset can only has one rr");
        }

        ss[1..].iter().try_fold(last, |mut rrset, s| {
            let mut current = RRset::from_str(s.as_ref())?;
            if rrset.typ != current.typ {
                bail!("rr in one rrset should has same type");
            }
            rrset.rdatas.push(current.rdatas.remove(0));
            Ok(rrset)
        })
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<()> {
        if self.rdatas.is_empty() {
            self.name.to_wire(render)?;
            self.typ.to_wire(render)?;
            self.class.to_wire(render)?;
            self.ttl.to_wire(render)?;
            render.write_u16(0)?;
        } else {
            for rdata in &self.rdatas {
                self.name.to_wire(render)?;
                self.typ.to_wire(render)?;
                self.class.to_wire(render)?;
                self.ttl.to_wire(render)?;
                let pos = render.len();
                render.skip(2)?;
                rdata.to_wire(render)?;
                let rdlen = render.len() - pos - 2;
                render.write_u16_at(pos, rdlen as u16)?;
            }
        }
        Ok(())
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

impl PartialEq for RRset {
    //in many cases, ttl should be ingnored for rrset equality
    fn eq(&self, other: &RRset) -> bool {
        if !self.is_same_rrset(other) {
            return false;
        }

        let rdata_count = self.rdatas.len();
        if rdata_count != other.rdatas.len() {
            return false;
        }

        //for rdata count smaller than 4
        //compare at most 9 times, otherwise sort then compare
        if rdata_count < 4 {
            self.rdatas.iter().all(|rdata| {
                other
                    .rdatas
                    .iter()
                    .position(|other_rdata| rdata == other_rdata)
                    .is_some()
            })
        } else {
            let mut rdatas1 = self.rdatas.clone();
            rdatas1.sort_unstable();
            let mut rdatas2 = other.rdatas.clone();
            rdatas2.sort_unstable();
            rdatas1 == rdatas2
        }
    }
}

impl Eq for RRset {}

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

        let rdata = RData::from_string_buffer(typ, &mut labels)?;
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
            .map(|rdata| writeln!(f, "{}\t{}", self.header(), rdata))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_rrset_eq() {
        let rrset1_str = vec![
            "example.com.       3600    IN      A       5.5.5.5",
            "example.com.       3600    IN      A       1.1.1.1",
            "example.com.       3600    IN      A       2.2.2.2",
            "example.com.       3600    IN      A       3.3.3.3",
            "example.com.       3600    IN      A       4.4.4.4",
        ];
        let rrset2_str = vec![
            "example.com.       360    IN      A       2.2.2.2",
            "example.com.       360    IN      A       3.3.3.3",
            "example.com.       360    IN      A       4.4.4.4",
            "example.com.       360    IN      A       1.1.1.1",
            "example.com.       360    IN      A       5.5.5.5",
        ];
        assert_eq!(
            RRset::from_strs(rrset1_str.as_slice()).unwrap(),
            RRset::from_strs(rrset2_str.as_slice()).unwrap()
        );

        let rrset1_str = vec![
            "example.com.       3600    IN      A       1.1.1.1",
            "example.com.       3600    IN      A       2.2.2.2",
        ];
        let rrset2_str = vec![
            "example.com.       360    IN      A       2.2.2.2",
            "example.com.       360    IN      A       1.1.1.1",
        ];
        assert_eq!(
            RRset::from_strs(rrset1_str.as_slice()).unwrap(),
            RRset::from_strs(rrset2_str.as_slice()).unwrap()
        );

        let rrset1_str = vec![
            "example.com.       3600    IN      A       1.1.1.1",
            "example.com.       3600    IN      A       3.3.3.3",
        ];
        let rrset2_str = vec![
            "example.com.       3600    IN      A       2.2.2.2",
            "example.com.       3600    IN      A       1.1.1.1",
        ];
        assert_ne!(
            RRset::from_strs(rrset1_str.as_slice()).unwrap(),
            RRset::from_strs(rrset2_str.as_slice()).unwrap()
        );

        let rrset1_str = vec![
            "example.com.       3600    IN      A       1.1.1.1",
            "example.com.       3600    IN      A       2.2.2.2",
            "example.com.       3600    IN      A       3.3.3.3",
        ];
        let rrset2_str = vec![
            "example.com.       3600    IN      A       2.2.2.2",
            "example.com.       3600    IN      A       1.1.1.1",
        ];
        assert_ne!(
            RRset::from_strs(rrset1_str.as_slice()).unwrap(),
            RRset::from_strs(rrset2_str.as_slice()).unwrap()
        );
    }
}
