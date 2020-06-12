use crate::message_render::MessageRender;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::rrset::{RRTtl, RRset};
use std::fmt;

use anyhow::Result;

const VERSION_SHIFT: u32 = 16;
const EXTRCODE_SHIFT: u32 = 24;
const VERSION_MASK: u32 = 0x00ff_0000;
const EXTFLAG_DO: u32 = 0x0000_8000;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Edns {
    pub versoin: u8,
    pub extened_rcode: u8,
    pub udp_size: u16,
    pub dnssec_aware: bool,
    pub options: Option<Vec<EdnsOption>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EdnsOption {}

impl Edns {
    pub fn from_rrset(rrset: &RRset) -> Edns {
        assert!(rrset.typ == RRType::OPT);

        let flags = rrset.ttl.0;
        Edns {
            versoin: ((flags & VERSION_MASK) >> VERSION_SHIFT) as u8,
            udp_size: rrset.class.to_u16(),
            extened_rcode: (flags >> EXTRCODE_SHIFT) as u8,
            dnssec_aware: (flags & EXTFLAG_DO) != 0,
            options: None,
        }
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<()> {
        let mut flags = u32::from(self.extened_rcode) << EXTRCODE_SHIFT;
        flags |= (u32::from(self.versoin) << VERSION_SHIFT) & VERSION_MASK;
        if self.dnssec_aware {
            flags |= EXTFLAG_DO;
        }

        render.write_u8(0)?;
        RRType::OPT.to_wire(render)?;
        RRClass::Unknown(self.udp_size).to_wire(render)?;
        RRTtl(flags).to_wire(render)?;
        render.write_u16(0)
    }

    pub fn rr_count(&self) -> usize {
        match self.options {
            Some(ref options) => options.len(),
            None => 1,
        }
    }
}

impl fmt::Display for Edns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "; EDNS: version: {}, ", self.versoin)?;
        if self.dnssec_aware {
            write!(f, "flags: do; ")?;
        }
        write!(f, "udp: {}", self.udp_size)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::{hex::from_hex, InputBuffer};

    #[test]
    fn test_edns_to_wire() {
        let raw = from_hex("0000291000000000000000").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let rrset = RRset::from_wire(&mut buf).unwrap();
        let edns = Edns::from_rrset(&rrset);
        let desired_edns = Edns {
            versoin: 0,
            extened_rcode: 0,
            udp_size: 4096,
            dnssec_aware: false,
            options: None,
        };
        assert_eq!(edns, desired_edns);

        let mut buf = [0; 512];
        let mut render = MessageRender::new(&mut buf);
        desired_edns.to_wire(&mut render);
        assert_eq!(raw.as_slice(), &buf[0..(raw.len())]);
    }
}
