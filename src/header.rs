use crate::header_flag::{clear_flag, is_flag_set, set_flag, setted_flags, HeaderFlag};
use crate::message_render::MessageRender;
use crate::opcode::Opcode;
use crate::rcode::Rcode;
use crate::util::InputBuffer;
use anyhow::Result;
use std::fmt;

const HEADERFLAG_MASK: u16 = 0x87b0;
const OPCODE_MASK: u16 = 0x7800;
const OPCODE_SHIFT: u16 = 11;
const RCODE_MASK: u16 = 0x000f;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Header {
    pub id: u16,
    pub flag: u16,
    pub opcode: Opcode,
    pub rcode: Rcode,
    pub qd_count: u16,
    pub an_count: u16,
    pub ns_count: u16,
    pub ar_count: u16,
}

impl Header {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        let id = buf.read_u16()?;
        let flag = buf.read_u16()?;
        let qd_count = buf.read_u16()?;
        let an_count = buf.read_u16()?;
        let ns_count = buf.read_u16()?;
        let ar_count = buf.read_u16()?;
        Ok(Header {
            id,
            flag: flag & HEADERFLAG_MASK,
            opcode: Opcode::new(((flag & OPCODE_MASK) >> OPCODE_SHIFT) as u8),
            rcode: Rcode::new((flag & RCODE_MASK) as u8),
            qd_count,
            an_count,
            ns_count,
            ar_count,
        })
    }

    pub fn clear(&mut self) {
        self.id = 0;
        self.flag = 0;
        self.qd_count = 0;
        self.an_count = 0;
        self.ns_count = 0;
        self.ar_count = 0;
    }

    pub fn setted_flags(&self) -> Vec<HeaderFlag> {
        setted_flags(self.flag)
    }

    pub fn is_flag_set(&self, flag: HeaderFlag) -> bool {
        is_flag_set(self.flag, flag)
    }

    pub fn set_flag(&mut self, flag: HeaderFlag, set: bool) {
        if set {
            set_flag(&mut self.flag, flag);
        } else {
            clear_flag(&mut self.flag, flag);
        }
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<()> {
        render.write_u16(self.id)?;
        render.write_u16(self.header_flag())?;
        render.write_u16(self.qd_count)?;
        render.write_u16(self.an_count)?;
        render.write_u16(self.ns_count)?;
        render.write_u16(self.ar_count)
    }

    fn header_flag(&self) -> u16 {
        let mut flag: u16 = ((u16::from(self.opcode.to_u8())) << OPCODE_SHIFT) & OPCODE_MASK;
        flag |= (u16::from(self.rcode.to_u8())) & RCODE_MASK;
        flag |= self.flag & HEADERFLAG_MASK;
        flag
    }
}

impl Default for Header {
    fn default() -> Header {
        Header {
            id: 52091,
            flag: 0,
            opcode: Opcode::Query,
            rcode: Rcode::NoError,
            qd_count: 1,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        }
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            ";; ->>HEADER<<- opcode: {}, status: {}, id: {}",
            self.opcode, self.rcode, self.id,
        )?;
        write!(f, ";; flags: ")?;
        for flag in self.setted_flags() {
            write!(f, " {}", flag)?;
        }
        write!(f, "; ")?;
        write!(f, "QUERY: {}, ", self.qd_count)?;
        write!(f, "ANSWER: {}, ", self.an_count)?;
        write!(f, "AUTHORITY: {}, ", self.ns_count)?;
        write!(f, "ADDITIONAL: {}, ", self.ar_count)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::hex::from_hex;

    #[test]
    fn test_from_to_wire() {
        let raw = from_hex("04b085000001000200010002").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let header = Header::from_wire(&mut buf).unwrap();
        assert_eq!(
            header.setted_flags(),
            vec![
                HeaderFlag::QueryRespone,
                HeaderFlag::AuthAnswer,
                HeaderFlag::RecursionDesired,
            ]
        );
        assert_eq!(header.id, 1200);
        assert_eq!(header.qd_count, 1);
        assert_eq!(header.an_count, 2);
        assert_eq!(header.ns_count, 1);
        assert_eq!(header.ar_count, 2);
        assert!(header.is_flag_set(HeaderFlag::QueryRespone));

        let mut buf = [0; 512];
        let mut render = MessageRender::new(&mut buf);
        header.to_wire(&mut render);
        assert_eq!(raw.as_slice(), &buf[0..(raw.len())]);
    }
}
