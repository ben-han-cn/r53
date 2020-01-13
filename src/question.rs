use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::util::{InputBuffer, OutputBuffer};
use anyhow::Result;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Question {
    pub name: Name,
    pub typ: RRType,
    pub class: RRClass,
}

impl Question {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        let name = Name::from_wire(buf)?;
        let typ = RRType::from_wire(buf)?;
        let class = RRClass::from_wire(buf)?;
        Ok(Question { name, typ, class })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        self.name.rend(render);
        self.typ.rend(render);
        self.class.rend(render);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.name.to_wire(buf);
        self.typ.to_wire(buf);
        self.class.to_wire(buf);
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.name, self.class, self.typ)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::hex::from_hex;

    #[test]
    fn test_question_to_wire() {
        let raw = from_hex("03666f6f076578616d706c6503636f6d0000020001").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let q = Question::from_wire(&mut buf).unwrap();
        let desired_q = Question {
            name: Name::new("foo.example.com.").unwrap(),
            typ: RRType::NS,
            class: RRClass::IN,
        };
        assert_eq!(q, desired_q);

        let mut render = MessageRender::new();
        desired_q.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
    }
}
