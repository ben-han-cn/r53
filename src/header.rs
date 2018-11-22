use error::Error;
use header_flag::{clear_flag, set_flag, setted_flags, HeaderFlag};
use message_render::MessageRender;
use opcode::Opcode;
use rcode::Rcode;
use std::fmt::Write;
use util::{InputBuffer, OutputBuffer};

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
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self, Error> {
        let id = try!(buf.read_u16());
        let flag = try!(buf.read_u16());
        let qd_count = try!(buf.read_u16());
        let an_count = try!(buf.read_u16());
        let ns_count = try!(buf.read_u16());
        let ar_count = try!(buf.read_u16());
        Ok(Header {
            id: id,
            flag: flag & HEADERFLAG_MASK,
            opcode: Opcode::new(((flag & OPCODE_MASK) >> OPCODE_SHIFT) as u8),
            rcode: Rcode::new((flag & RCODE_MASK) as u8),
            qd_count: qd_count,
            an_count: an_count,
            ns_count: ns_count,
            ar_count: ar_count,
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

    pub fn set_flag(&mut self, flag: HeaderFlag, set: bool) {
        if set {
            set_flag(&mut self.flag, flag);
        } else {
            clear_flag(&mut self.flag, flag);
        }
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_u16(self.id);
        render.write_u16(self.header_flag());
        render.write_u16(self.qd_count);
        render.write_u16(self.an_count);
        render.write_u16(self.ns_count);
        render.write_u16(self.ar_count);
    }

    fn header_flag(&self) -> u16 {
        let mut flag: u16 = (((self.opcode.to_u8()) as u16) << OPCODE_SHIFT) & OPCODE_MASK;
        flag |= ((self.rcode.to_u8()) as u16) & RCODE_MASK;
        flag |= self.flag & HEADERFLAG_MASK;
        flag
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_u16(self.id);
        buf.write_u16(self.header_flag());
        buf.write_u16(self.qd_count);
        buf.write_u16(self.an_count);
        buf.write_u16(self.ns_count);
        buf.write_u16(self.ar_count);
    }

    pub fn to_string(&self) -> String {
        let mut header_str = String::new();
        write!(
            &mut header_str,
            ";; ->>HEADER<<- opcode: {}, status: {}, id: {}\n",
            self.opcode.to_string(),
            self.rcode.to_string(),
            self.id
        ).unwrap();
        write!(&mut header_str, ";; flags: ").unwrap();
        for flag in self.setted_flags() {
            write!(&mut header_str, " {}", flag.to_string()).unwrap();
        }
        write!(&mut header_str, "; ").unwrap();
        write!(&mut header_str, "QUERY: {}, ", self.qd_count).unwrap();
        write!(&mut header_str, "ANSWER: {}, ", self.an_count).unwrap();
        write!(&mut header_str, "AUTHORITY: {}, ", self.ns_count).unwrap();
        write!(&mut header_str, "ADDITIONAL: {}, ", self.ar_count).unwrap();
        write!(&mut header_str, "\n").unwrap();
        header_str
    }
}

impl Default for Header {
    fn default() -> Header {
        return Header {
            id: 52091,
            flag: 0,
            opcode: Opcode::Query,
            rcode: Rcode::NoError,
            qd_count: 1,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use util::hex::from_hex;

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

        let mut render = MessageRender::new();
        header.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
    }
}
