use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::util::InputBuffer;
use anyhow::Result;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Question {
    pub name: Name,
    pub typ: RRType,
    pub class: RRClass,
}

impl Question {
    pub fn new(name: Name, typ: RRType) -> Self {
        Question {
            name,
            typ,
            class: RRClass::IN,
        }
    }

    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        let name = Name::from_wire(buf)?;
        let typ = RRType::from_wire(buf)?;
        let class = RRClass::from_wire(buf)?;
        Ok(Question { name, typ, class })
    }

    pub fn to_wire(&self, render: &mut MessageRender) -> Result<()> {
        self.name.to_wire(render)?;
        self.typ.to_wire(render)?;
        self.class.to_wire(render)
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.name, self.class, self.typ)
    }
}
