use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield::*;
use crate::util::{InputBuffer, OutputBuffer};
use anyhow::{ensure, Result};
use rdata_derive::Rdata;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct A {
    #[field(codec = "ipv4", display = "ipv4")]
    pub host: Ipv4Addr,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct AAAA {
    #[field(codec = "ipv6", display = "ipv6")]
    pub host: Ipv6Addr,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct SOA {
    #[field(codec = "name", display = "name")]
    pub mname: Name,
    #[field(codec = "name", display = "name")]
    pub rname: Name,
    #[field(codec = "u32", display = "int")]
    pub serial: u32,
    #[field(codec = "u32", display = "int")]
    pub refresh: u32,
    #[field(codec = "u32", display = "int")]
    pub retry: u32,
    #[field(codec = "u32", display = "int")]
    pub expire: u32,
    #[field(codec = "u32", display = "int")]
    pub minimum: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct SRV {
    #[field(codec = "u16", display = "int")]
    pub priority: u16,
    #[field(codec = "u16", display = "int")]
    pub weight: u16,
    #[field(codec = "u16", display = "int")]
    pub port: u16,
    #[field(codec = "name", display = "name")]
    pub target: Name,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct TXT {
    #[field(codec = "text", display = "text")]
    pub data: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct NS {
    #[field(codec = "name", display = "name")]
    pub name: Name,
}
#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct PTR {
    #[field(codec = "name", display = "name")]
    pub name: Name,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct CName {
    #[field(codec = "name", display = "name")]
    pub name: Name,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct MX {
    #[field(codec = "u16", display = "int")]
    pub preference: u16,
    #[field(codec = "name", display = "name")]
    pub name: Name,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct NAPTR {
    #[field(codec = "u16", display = "int")]
    pub order: u16,
    #[field(codec = "u16", display = "int")]
    pub preference: u16,
    #[field(codec = "byte_binary", display = "string")]
    pub flags: Vec<u8>,
    #[field(codec = "byte_binary", display = "string")]
    pub services: Vec<u8>,
    #[field(codec = "byte_binary", display = "string")]
    pub regexp: Vec<u8>,
    #[field(codec = "name", display = "name")]
    pub replacement: Name,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct OPT {
    #[field(codec = "binary", display = "string")]
    pub data: Vec<u8>,
}
