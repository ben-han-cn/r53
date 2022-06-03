use crate::name::Name;
use crate::rr_type::RRType;
use crate::util::{hex::from_hex, StringBuffer};
use anyhow::{anyhow, bail, Result};
use std::net::{Ipv4Addr, Ipv6Addr};
use time::{Date, Time};

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

pub fn rrtype_from_str(buf: &mut StringBuffer) -> Result<RRType> {
    buf.read::<RRType>()
}

pub fn u32_from_str(buf: &mut StringBuffer) -> Result<u32> {
    buf.read::<u32>()
}

pub fn text_from_str(buf: &mut StringBuffer) -> Result<Vec<Vec<u8>>> {
    buf.read_text()
}

pub fn string_from_str(buf: &mut StringBuffer) -> Result<Vec<u8>> {
    buf.read_char_string()
}

pub fn timestamp_from_str(buf: &mut StringBuffer) -> Result<u32> {
    let ts = buf.read_str().ok_or(anyhow!("read timestamp failed"))?;
    if ts.len() != 14 {
        bail!("timestamp isn't in YYYYMMDDHHmmSS format");
    }

    Ok(
        Date::try_from_ymd(ts[0..4].parse()?, ts[4..6].parse()?, ts[6..8].parse()?)
            .map_err(|_| anyhow!("invalid date"))?
            .with_time(
                Time::try_from_hms(ts[8..10].parse()?, ts[10..12].parse()?, ts[12..14].parse()?)
                    .map_err(|_| anyhow!("invalid time"))?,
            )
            .assume_utc()
            .unix_timestamp() as u32,
    )
}

pub fn binary_from_str(buf: &mut StringBuffer) -> Result<Vec<u8>> {
    buf.read_str()
        .and_then(|s| from_hex(s))
        .ok_or(anyhow!("invalid hex"))
}

pub fn base64_from_str(buf: &mut StringBuffer) -> Result<Vec<u8>> {
    buf.read_left()
        .and_then(|s| {
            //rfc4034 allow whitespace in base64 encoded string
            //but rust base64 lib won't allow it so strip the
            let bs: String = s.chars().filter(|c| !c.is_whitespace()).collect();
            base64::decode(bs).ok()
        })
        .ok_or(anyhow!("invalid base64"))
}
