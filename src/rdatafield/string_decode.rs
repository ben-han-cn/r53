use crate::name::Name;
use crate::util::{hex::from_hex, StringBuffer};
use anyhow::{anyhow, Result};
use std::net::{Ipv4Addr, Ipv6Addr};

pub fn name_from_str(buf: &mut StringBuffer) -> Result<Name> {
    buf.read::<Name>()
}

pub fn ipv4_from_str(buf: &mut StringBuffer) -> Result<Ipv4Addr> {
    buf.read::<Ipv4Addr>()
}

pub fn ipv6_from_str(buf: &mut StringBuffer) -> Result<Ipv6Addr> {
    buf.read::<Ipv6Addr>()
}

pub fn u8_from_str(buf: &mut StringBuffer) -> Result<u8> {
    buf.read::<u8>()
}

pub fn u16_from_str(buf: &mut StringBuffer) -> Result<u16> {
    buf.read::<u16>()
}

pub fn u32_from_str(buf: &mut StringBuffer) -> Result<u32> {
    buf.read::<u32>()
}

pub fn text_from_str(buf: &mut StringBuffer) -> Result<Vec<Vec<u8>>> {
    buf.read_text()
}

pub fn string_from_str(buf: &mut StringBuffer) -> Result<Vec<u8>> {
    buf.read_str()
        .and_then(|s| Some(s.trim_matches('"').as_bytes().to_vec()))
        .ok_or(anyhow!("empty string"))
}

pub fn binary_from_str(buf: &mut StringBuffer) -> Result<Vec<u8>> {
    buf.read_str()
        .and_then(|s| from_hex(s))
        .ok_or(anyhow!("invalid hex"))
}
