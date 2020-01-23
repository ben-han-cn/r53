use crate::name::Name;
use crate::util::hex::to_hex;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::{fmt, str};

pub fn name_to_str(f: &mut fmt::Formatter, name: &Name) -> fmt::Result {
    write!(f, "{} ", name)
}

pub fn ipv4_to_str(f: &mut fmt::Formatter, addr: Ipv4Addr) -> fmt::Result {
    write!(f, "{} ", addr)
}

pub fn ipv6_to_str(f: &mut fmt::Formatter, addr: Ipv6Addr) -> fmt::Result {
    write!(f, "{} ", addr)
}

pub fn u8_to_str(f: &mut fmt::Formatter, num: u8) -> fmt::Result {
    write!(f, "{} ", num)
}

pub fn u16_to_str(f: &mut fmt::Formatter, num: u16) -> fmt::Result {
    write!(f, "{} ", num)
}

pub fn u32_to_str(f: &mut fmt::Formatter, num: u32) -> fmt::Result {
    write!(f, "{} ", num)
}

pub fn text_to_str(f: &mut fmt::Formatter, data: &Vec<Vec<u8>>) -> fmt::Result {
    for d in data {
        string_to_str(f, d)?;
    }
    Ok(())
}

pub fn string_to_str(f: &mut fmt::Formatter, data: &Vec<u8>) -> fmt::Result {
    write!(
        f,
        "\"{}\" ",
        str::from_utf8(data.as_slice()).unwrap_or("invalid utf8 str")
    )
}

pub fn binary_to_str(f: &mut fmt::Formatter, data: &Vec<u8>) -> fmt::Result {
    write!(f, "{} ", to_hex(data.as_slice()))
}
