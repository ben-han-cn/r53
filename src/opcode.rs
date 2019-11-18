use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opcode {
    Query,
    IQuery,
    Status,
    Notify,
    Update,
    Reserved,
}

impl Opcode {
    pub fn new(value: u8) -> Self {
        match value {
            0 => Opcode::Query,
            1 => Opcode::IQuery,
            2 => Opcode::Status,
            4 => Opcode::Notify,
            5 => Opcode::Update,
            _ => Opcode::Reserved,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            Opcode::Query => 0,
            Opcode::IQuery => 1,
            Opcode::Status => 2,
            Opcode::Notify => 4,
            Opcode::Update => 5,
            Opcode::Reserved => 6,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Opcode::Query => "QUERY",
            Opcode::IQuery => "IQUERY",
            Opcode::Status => "STATUS",
            Opcode::Notify => "NOTIFY",
            Opcode::Update => "UPDATE",
            Opcode::Reserved => "RESERVED",
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_rcode_equal() {
        assert_eq!(Opcode::Query.to_u8(), 0);
        assert_eq!(Opcode::Query.to_string(), "QUERY");
        assert_eq!(Opcode::Notify.to_string(), "NOTIFY");
    }
}
