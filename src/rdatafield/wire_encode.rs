use crate::message_render::MessageRender;
use crate::name::Name;
use std::net::{Ipv4Addr, Ipv6Addr};

pub fn name_to_wire(render: &mut MessageRender, name: &Name) {
    render.write_name(name, true);
}

pub fn name_uncompressed_to_wire(render: &mut MessageRender, name: &Name) {
    render.write_name(name, false);
}

pub fn ipv4_to_wire(render: &mut MessageRender, addr: Ipv4Addr) {
    addr.octets().iter().for_each(|x| render.write_u8(*x));
}

pub fn ipv6_to_wire(render: &mut MessageRender, addr: Ipv6Addr) {
    addr.octets().iter().for_each(|x| render.write_u8(*x));
}

pub fn u8_to_wire(render: &mut MessageRender, num: u8) {
    render.write_u8(num);
}

pub fn u16_to_wire(render: &mut MessageRender, num: u16) {
    render.write_u16(num);
}

pub fn u32_to_wire(render: &mut MessageRender, num: u32) {
    render.write_u32(num);
}

pub fn text_to_wire(render: &mut MessageRender, data: &Vec<Vec<u8>>) {
    data.iter().for_each(|d| byte_binary_to_wire(render, d));
}

pub fn byte_binary_to_wire(render: &mut MessageRender, data: &Vec<u8>) {
    render.write_u8(data.len() as u8);
    render.write_bytes(data.as_slice());
}

pub fn binary_to_wire(render: &mut MessageRender, data: &Vec<u8>) {
    render.write_bytes(data.as_slice());
}
