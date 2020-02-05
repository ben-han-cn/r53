use crate::name::Name;
use crate::util::InputBuffer;
use anyhow::{ensure, Result};
use std::net::{Ipv4Addr, Ipv6Addr};

pub fn name_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(Name, u16)> {
    _name_from_wire(buf, len)
}

pub fn name_uncompressed_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(Name, u16)> {
    _name_from_wire(buf, len)
}

fn _name_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(Name, u16)> {
    let p = buf.position();
    let name = Name::from_wire(buf)?;
    let name_len = (buf.position() - p) as u16;
    ensure!(len >= name_len, "wire is too short for domain name");
    Ok((name, len - name_len))
}

pub fn ipv4_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(Ipv4Addr, u16)> {
    ensure!(len >= 4, "wire is too short for ipv4 address");
    let ip = buf
        .read_bytes(4)
        .map(|bytes| Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]))?;
    Ok((ip, len - 4))
}

pub fn ipv6_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(Ipv6Addr, u16)> {
    ensure!(len >= 16, "wire is too short for ipv6 address");
    let ip = buf.read_bytes(16).map(|bytes| {
        let mut octs = [0; 16];
        octs.copy_from_slice(bytes);
        Ipv6Addr::from(octs)
    })?;
    Ok((ip, len - 16))
}

pub fn u8_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(u8, u16)> {
    ensure!(len >= 1, "wire is too short for u8");
    Ok((buf.read_u8()?, len - 1))
}

pub fn u16_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(u16, u16)> {
    ensure!(len >= 2, "wire is too short for u16");
    Ok((buf.read_u16()?, len - 2))
}

pub fn u32_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(u32, u16)> {
    ensure!(len >= 4, "wire is too short for u32");
    Ok((buf.read_u32()?, len - 4))
}

pub fn text_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(Vec<Vec<u8>>, u16)> {
    let mut data = Vec::new();
    let mut left_len = len;
    while left_len > 0 {
        let (txt, len) = byte_binary_from_wire(buf, left_len)?;
        data.push(txt);
        left_len = len;
    }
    Ok((data, 0))
}

pub fn byte_binary_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(Vec<u8>, u16)> {
    let dl = buf.read_u8()? as u16;
    ensure!(len > dl, "wire is too short for byte binary");
    let data = buf.read_bytes(dl as usize)?;
    Ok((data.to_vec(), len - dl - 1))
}

pub fn binary_from_wire(buf: &mut InputBuffer, len: u16) -> Result<(Vec<u8>, u16)> {
    let data = buf.read_bytes(len as usize)?;
    Ok((data.to_vec(), 0))
}
