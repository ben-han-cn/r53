pub fn from_hex(hex_str: &str) -> Option<Vec<u8>> {
    let mut b = Vec::with_capacity(hex_str.len() / 2);
    let mut modulus = 0;
    let mut buf = 0;

    for (_, byte) in hex_str.bytes().enumerate() {
        buf <<= 4;
        match byte {
            b'A'..=b'F' => buf |= byte - b'A' + 10,
            b'a'..=b'f' => buf |= byte - b'a' + 10,
            b'0'..=b'9' => buf |= byte - b'0',
            b' ' | b'\r' | b'\n' | b'\t' => {
                buf >>= 4;
                continue;
            }
            _ => return None,
        }
        modulus += 1;
        if modulus == 2 {
            modulus = 0;
            b.push(buf);
        }
    }

    match modulus {
        0 => Some(b.into_iter().collect()),
        _ => None,
    }
}

static CHARS: &[u8] = b"0123456789abcdef";
pub fn to_hex(data: &[u8]) -> String {
    let mut v = Vec::with_capacity(data.len() * 2);
    for &byte in data.iter() {
        v.push(CHARS[(byte >> 4) as usize]);
        v.push(CHARS[(byte & 0xf) as usize]);
    }

    unsafe { String::from_utf8_unchecked(v) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_hex() {
        let data = from_hex("01a3").unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], 1);
        assert_eq!(data[1], 163);
        assert_eq!(to_hex(&data), "01a3");

        assert!(from_hex("01a31").is_none());
        assert!(from_hex("01a31g").is_none());
    }
}
