use crate::message_render::MessageRender;
use crate::rdatafield_string_parser::Parser;
use crate::util::hex::to_hex;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OPT {
    pub data: Vec<u8>,
}

impl OPT {
    pub fn from_wire(buf: &mut InputBuffer, len: u16) -> Result<Self> {
        buf.read_bytes(len as usize).map(|data| OPT {
            data: data.to_vec(),
        })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_bytes(self.data.as_slice());
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_bytes(self.data.as_slice());
    }

    pub fn to_string(&self) -> String {
        to_hex(&self.data)
    }

    pub fn from_str<'a>(iter: &mut Parser<'a>) -> Result<Self> {
        let data = iter.next_hex("OPT", "data")?;
        Ok(OPT { data })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_opt_to_wire() {}
}
