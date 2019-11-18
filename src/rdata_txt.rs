use crate::message_render::MessageRender;
use crate::rdatafield_string_parser::Parser;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::str::from_utf8;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TXT {
    pub data: Vec<Vec<u8>>,
}

impl TXT {
    pub fn from_wire(buf: &mut InputBuffer, len: u16) -> Result<Self> {
        let mut read_len = 0;
        let mut data = Vec::new();
        while read_len < len {
            let sl = buf.read_u8()?;
            let bytes = buf.read_bytes(sl as usize)?;
            read_len += (sl + 1) as u16;
            data.push(bytes.to_vec());
        }
        Ok(TXT { data })
    }

    pub fn from_str<'a>(parser: &mut Parser<'a>) -> Result<Self> {
        let data = parser.next_txt("TXT", "data")?;
        Ok(TXT { data })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        for data in &self.data {
            render.write_u8(data.len() as u8);
            render.write_bytes(data.as_slice());
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        for data in &self.data {
            buf.write_u8(data.len() as u8);
            buf.write_bytes(data.as_slice());
        }
    }

    pub fn to_string(&self) -> String {
        self.data.iter().fold(String::new(), |mut s, data| {
            s.push_str("\"");
            s.push_str(from_utf8(data).unwrap());
            s.push_str("\" ");
            s
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_txt_to_wire() {}
}
