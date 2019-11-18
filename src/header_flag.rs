use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HeaderFlag {
    QueryRespone,
    AuthAnswer,
    Truncation,
    RecursionDesired,
    RecursionAvailable,
    AuthenticData,
    CheckDisable,
}

impl HeaderFlag {
    pub fn flag_mask(self) -> u16 {
        match self {
            HeaderFlag::QueryRespone => 0x8000,
            HeaderFlag::AuthAnswer => 0x0400,
            HeaderFlag::Truncation => 0x0200,
            HeaderFlag::RecursionDesired => 0x0100,
            HeaderFlag::RecursionAvailable => 0x0080,
            HeaderFlag::AuthenticData => 0x0020,
            HeaderFlag::CheckDisable => 0x0010,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            HeaderFlag::QueryRespone => "qr",
            HeaderFlag::AuthAnswer => "aa",
            HeaderFlag::Truncation => "tc",
            HeaderFlag::RecursionDesired => "rd",
            HeaderFlag::RecursionAvailable => "ra",
            HeaderFlag::AuthenticData => "ad",
            HeaderFlag::CheckDisable => "cd",
        }
    }
}

impl fmt::Display for HeaderFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.to_str())
    }
}

pub fn set_flag(header_flags: &mut u16, flag: HeaderFlag) {
    *header_flags |= flag.flag_mask()
}

pub fn clear_flag(header_flags: &mut u16, flag: HeaderFlag) {
    *header_flags &= !flag.flag_mask()
}

pub fn is_flag_set(header_flags: u16, flag: HeaderFlag) -> bool {
    (header_flags & flag.flag_mask()) != 0
}

pub fn setted_flags(header_flags: u16) -> Vec<HeaderFlag> {
    let mut flags = vec![
        HeaderFlag::QueryRespone,
        HeaderFlag::AuthAnswer,
        HeaderFlag::Truncation,
        HeaderFlag::RecursionDesired,
        HeaderFlag::RecursionAvailable,
        HeaderFlag::AuthenticData,
        HeaderFlag::CheckDisable,
    ];
    flags.retain(|&flag| is_flag_set(header_flags, flag));
    flags
}
