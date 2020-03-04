use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdatafield::*;
use crate::util::{InputBuffer, StringBuffer};
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
    #[field(codec = "u32", display = "u32")]
    pub serial: u32,
    #[field(codec = "u32", display = "u32")]
    pub refresh: u32,
    #[field(codec = "u32", display = "u32")]
    pub retry: u32,
    #[field(codec = "u32", display = "u32")]
    pub expire: u32,
    #[field(codec = "u32", display = "u32")]
    pub minimum: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct SRV {
    #[field(codec = "u16", display = "u16")]
    pub priority: u16,
    #[field(codec = "u16", display = "u16")]
    pub weight: u16,
    #[field(codec = "u16", display = "u16")]
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
    #[field(codec = "u16", display = "u16")]
    pub preference: u16,
    #[field(codec = "name", display = "name")]
    pub name: Name,
}

#[derive(Debug, Clone, Eq, PartialEq, Rdata)]
pub struct NAPTR {
    #[field(codec = "u16", display = "u16")]
    pub order: u16,
    #[field(codec = "u16", display = "u16")]
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
    #[field(codec = "binary", display = "binary")]
    pub data: Vec<u8>,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str;
    #[test]
    fn test_rdata_from_str() {
        //soa
        let soa = SOA::from_str(&mut StringBuffer::new(
            "a.gtld-servers.net. nstld.verisign-grs.com. 1579589122 1800 900 604800 86400",
        ))
        .unwrap();
        assert_eq!(soa.mname, Name::new("a.gtld-servers.net").unwrap());
        assert_eq!(soa.rname, Name::new("nstld.verisign-grs.com.").unwrap());
        assert_eq!(soa.serial, 1579589122);
        assert_eq!(soa.refresh, 1800);
        assert_eq!(soa.retry, 900);
        assert_eq!(soa.expire, 604800);
        assert_eq!(soa.minimum, 86400);

        //ns
        let ns = NS::from_str(&mut StringBuffer::new("a.com")).unwrap();
        assert_eq!(ns.name, Name::new("a.com").unwrap());

        //a
        let a = A::from_str(&mut StringBuffer::new("1.1.1.1")).unwrap();
        assert_eq!(a.host, "1.1.1.1".parse::<Ipv4Addr>().unwrap());

        //aaaa
        let aaaa = AAAA::from_str(&mut StringBuffer::new("192::5")).unwrap();
        assert_eq!(aaaa.host, "192::5".parse::<Ipv6Addr>().unwrap());

        //srv
        let srv = SRV::from_str(&mut StringBuffer::new("0 1 5061 server.org")).unwrap();
        assert_eq!(srv.priority, 0);
        assert_eq!(srv.weight, 1);
        assert_eq!(srv.port, 5061);
        assert_eq!(srv.target, Name::new("server.org").unwrap());

        //naptr
        let naptr = NAPTR::from_str(&mut StringBuffer::new(
            "50 50 \"S\" \"SIPS+D2T\" \"!^.*$!sip:customer-service@example.com!\" server.hang3a.zone.")
        )
        .unwrap();
        assert_eq!(naptr.order, 50);
        assert_eq!(naptr.preference, 50);
        assert_eq!(str::from_utf8(&naptr.flags).unwrap(), "S");
        assert_eq!(str::from_utf8(&naptr.services).unwrap(), "SIPS+D2T");
        assert_eq!(
            str::from_utf8(&naptr.regexp).unwrap(),
            "!^.*$!sip:customer-service@example.com!"
        );
        assert_eq!(naptr.replacement, Name::new("server.hang3a.zone.").unwrap());

        //mx
        let mx = MX::from_str(&mut StringBuffer::new("10 mail.com")).unwrap();
        assert_eq!(mx.preference, 10);
        assert_eq!(mx.name, Name::new("mail.com").unwrap());

        //cname
        let cname = CName::from_str(&mut StringBuffer::new("a.b.c")).unwrap();
        assert_eq!(cname.name, Name::new("a.b.c").unwrap());

        //txt1
        let txt1 = TXT::from_str(&mut StringBuffer::new(r#""foo" "bar""#)).unwrap();
        assert_eq!(str::from_utf8(&txt1.data[0]).unwrap(), "foo");
        assert_eq!(str::from_utf8(&txt1.data[1]).unwrap(), "bar");

        //txt2
        let txt2 = TXT::from_str(&mut StringBuffer::new(r#""foo bar""#)).unwrap();
        assert_eq!(str::from_utf8(&txt2.data[0]).unwrap(), "foo bar");

        //txt3
        let txt3 = TXT::from_str(&mut StringBuffer::new("\"foo\010bar\"")).unwrap();
        assert_eq!(str::from_utf8(&txt3.data[0]).unwrap(), "foo\010bar");

        //txt4
        let txt4 = TXT::from_str(&mut StringBuffer::new(r#""foo\"xx\" bar""#)).unwrap();
        assert_eq!(str::from_utf8(&txt4.data[0]).unwrap(), r#"foo"xx" bar"#);
    }

    #[test]
    fn test_rdata_to_str() {
        //soa
        let soa_str =
            "a.gtld-servers.net. nstld.verisign-grs.com. 1579589122 1800 900 604800 86400";
        let soa = SOA::from_str(&mut StringBuffer::new(soa_str)).unwrap();
        assert_eq!(soa.to_string(), soa_str);

        //a
        let a_str = "1.1.1.1";
        let a = A::from_str(&mut StringBuffer::new(a_str)).unwrap();
        assert_eq!(a.to_string(), a_str);
    }
}
