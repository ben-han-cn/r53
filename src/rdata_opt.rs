use error::Error;
use message_render::MessageRender;
use util::hex::to_hex;
use util::{InputBuffer, OutputBuffer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OPT {
    data: Vec<u8>,
}

impl OPT {
    pub fn from_wire(buf: &mut InputBuffer, len: u16) -> Result<Self, Error> {
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
}

#[cfg(test)]
mod test {
    #[test]
    fn test_opt_to_wire() {}
}
