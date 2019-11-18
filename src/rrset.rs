use crate::error::DNSError;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdata::RData;
use crate::rdatafield_string_parser::Parser;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::util::{InputBuffer, OutputBuffer};
use failure::{self, Result};
use std::fmt::Write;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RRTtl(pub u32);

impl RRTtl {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        buf.read_u32().map(RRTtl)
    }

    pub fn rend(self, render: &mut MessageRender) {
        render.write_u32(self.0);
    }

    pub fn to_wire(self, buf: &mut OutputBuffer) {
        buf.write_u32(self.0);
    }

    fn to_string(self) -> String {
        self.0.to_string()
    }
}

impl FromStr for RRTtl {
    type Err = failure::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s.parse::<u32>() {
            Ok(num) => Ok(RRTtl(num)),
            Err(_) => Err(DNSError::InvalidTtlString.into()),
        }
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

    pub fn rend(&self, render: &mut MessageRender) {
        if self.rdatas.is_empty() {
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
        if self.rdatas.is_empty() {
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
            writeln!(&mut rrset_str, "{}\t{}", self.header(), rdata.to_string()).unwrap();
        });
        rrset_str
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
    type Err = failure::Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let mut labels = Parser::new(s.trim());

        let name = if let Some(name_str) = labels.next() {
            Name::from_str(name_str)?
        } else {
            return Err(DNSError::InvalidRRsetString.into());
        };

        let ttl = if let Some(ttl_str) = labels.next() {
            RRTtl::from_str(ttl_str)?
        } else {
            return Err(DNSError::InvalidRRsetString.into());
        };

        let mut short_of_class = false;
        let cls_str = if let Some(cls_str) = labels.next() {
            cls_str
        } else {
            return Err(DNSError::InvalidRRsetString.into());
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
            return Err(DNSError::InvalidRRsetString.into());
        };

        let rdata = RData::from_parser(typ, &mut labels)?;
        Ok(RRset {
            name,
            typ,
            class,
            ttl,
            rdatas: vec![rdata],
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::hex::from_hex;

    #[test]
    fn test_rrset_to_wire() {
        let raw =
            from_hex("0474657374076578616d706c6503636f6d000001000100000e100004c0000201").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let rrset = RRset::from_wire(&mut buf).unwrap();
        let desired_rrset = RRset::from_str("test.example.com. 3600 IN A 192.0.2.1").unwrap();
        assert_eq!(rrset, desired_rrset);
        let mut render = MessageRender::new();
        desired_rrset.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
    }

    #[test]
    fn test_rrset_from_string() {
        let rrset_strs = vec![
            "example.org. 100 IN SOA xxx.net. ns.example.org. 100 1800 900 604800 86400",
            "example.org. 200 IN NS ns.example.org.",
            "example.org. 300 IN A 192.0.2.1",
            "ns.example.org. 400 IN AAAA 2001:db8::2",
            "cname.example.org. 500 IN CNAME canonical.example.org",
            "_sip._udp.example.com 600 SRV 5 100 5060 sip-udp01.example.com.",
            "mydomain.com. 700 IN MX 0 mydomain.com.",
            "16.3.0.122.in-addr.arpa. 800 IN PTR name.net",
        ];

        let typs = vec![
            RRType::SOA,
            RRType::NS,
            RRType::A,
            RRType::AAAA,
            RRType::CNAME,
            RRType::SRV,
            RRType::MX,
            RRType::PTR,
        ];

        for (index, rrset_str) in rrset_strs.iter().enumerate() {
            let rrset = RRset::from_str(*rrset_str).expect("parse rrset failed");
            assert_eq!(rrset.typ, typs[index]);
            assert_eq!(
                rrset.ttl,
                RRTtl::from_str(format!("{}", (index + 1) * 100).as_ref()).unwrap()
            );
        }
    }
}
