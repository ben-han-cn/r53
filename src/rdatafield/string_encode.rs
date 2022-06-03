use crate::name::Name;
use crate::rr_type::RRType;
use crate::util::hex::to_hex;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use time::OffsetDateTime;

pub fn name_to_str(f: &mut fmt::Formatter, name: &Name) -> fmt::Result {
    write!(f, "{}", name)
}

pub fn ipv4_to_str(f: &mut fmt::Formatter, addr: Ipv4Addr) -> fmt::Result {
    write!(f, "{}", addr)
}

pub fn ipv6_to_str(f: &mut fmt::Formatter, addr: Ipv6Addr) -> fmt::Result {
    write!(f, "{}", addr)
}

pub fn u8_to_str(f: &mut fmt::Formatter, num: u8) -> fmt::Result {
    write!(f, "{}", num)
}

pub fn u16_to_str(f: &mut fmt::Formatter, num: u16) -> fmt::Result {
    write!(f, "{}", num)
}

pub fn rrtype_to_str(f: &mut fmt::Formatter, typ: RRType) -> fmt::Result {
    write!(f, "{}", typ)
}

pub fn u32_to_str(f: &mut fmt::Formatter, num: u32) -> fmt::Result {
    write!(f, "{}", num)
}

pub fn text_to_str(f: &mut fmt::Formatter, data: &[Vec<u8>]) -> fmt::Result {
    for d in data {
        string_to_str(f, d)?;
    }
    Ok(())
}

pub fn timestamp_to_str(f: &mut fmt::Formatter, secs: u32) -> fmt::Result {
    write!(
        f,
        "{}",
        OffsetDateTime::from_unix_timestamp(secs as i64).format("%Y%m%d%H%M%S")
    )
}

pub fn string_to_str(f: &mut fmt::Formatter, data: &[u8]) -> fmt::Result {
    let mut buf = Vec::new();
    for c in data {
        let ch = *c;
        if (ch < 0x20) || (ch >= 0x7f) {
            buf.push(b'\\');
            buf.push(0x30 + ((ch / 100) % 10));
            buf.push(0x30 + ((ch / 10) % 10));
            buf.push(0x30 + (ch % 10));
            continue;
        } else if ch == b'"' || ch == b';' || ch == b'\\' {
            buf.push(b'\\');
        }
        buf.push(ch);
    }
    write!(f, "\"{}\"", unsafe { String::from_utf8_unchecked(buf) })
}

pub fn binary_to_str(f: &mut fmt::Formatter, data: &[u8]) -> fmt::Result {
    write!(f, "{}", to_hex(data))
}

pub fn base64_to_str(f: &mut fmt::Formatter, data: &[u8]) -> fmt::Result {
    write!(f, "{}", base64::encode(data))
}
