use crate::message_render::MessageRender;
use crate::name::Name;
use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::Result;

pub fn name_to_wire(render: &mut MessageRender, name: &Name) -> Result<()> {
    render.write_name(name, true)
}

pub fn name_uncompressed_to_wire(render: &mut MessageRender, name: &Name) -> Result<()> {
    render.write_name(name, false)
}

pub fn ipv4_to_wire(render: &mut MessageRender, addr: Ipv4Addr) -> Result<()> {
    for x in &addr.octets() {
        render.write_u8(*x)?;
    }
    Ok(())
}

pub fn ipv6_to_wire(render: &mut MessageRender, addr: Ipv6Addr) -> Result<()> {
    for x in &addr.octets() {
        render.write_u8(*x)?;
    }
    Ok(())
}

pub fn u8_to_wire(render: &mut MessageRender, num: u8) -> Result<()> {
    render.write_u8(num)
}

pub fn u16_to_wire(render: &mut MessageRender, num: u16) -> Result<()> {
    render.write_u16(num)
}

pub fn u32_to_wire(render: &mut MessageRender, num: u32) -> Result<()> {
    render.write_u32(num)
}

pub fn text_to_wire(render: &mut MessageRender, data: &[Vec<u8>]) -> Result<()> {
    for d in data {
        byte_binary_to_wire(render, d)?;
    }
    Ok(())
}

pub fn byte_binary_to_wire(render: &mut MessageRender, data: &[u8]) -> Result<()> {
    render.write_u8(data.len() as u8)?;
    render.write_bytes(data)
}

pub fn binary_to_wire(render: &mut MessageRender, data: &[u8]) -> Result<()> {
    render.write_bytes(data)
}
