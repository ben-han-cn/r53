use anyhow::{anyhow, bail, ensure, Result};
use std::str::from_utf8;
use std::str::FromStr;

pub struct StringBuffer<'a> {
    raw: &'a [u8],
    pos: usize,
}

impl<'a> StringBuffer<'a> {
    pub fn new(raw: &'a str) -> Self {
        debug_assert!(!raw.is_empty());

        StringBuffer {
            raw: raw.as_bytes(),
            pos: 0,
        }
    }

    pub fn read_text(&mut self) -> Result<Vec<Vec<u8>>> {
        let mut data = Vec::new();
        loop {
            self.skip_whitespace();
            if self.is_eos() {
                break;
            }
            let slice = self.read_text_slice()?;
            data.push(slice);
        }
        ensure!(!data.is_empty(), "quote isn't in pair",);
        Ok(data)
    }

    pub fn read_text_slice(&mut self) -> Result<Vec<u8>> {
        if self.raw[self.pos] != b'"' {
            bail!("text isn't quoted");
        }

        self.pos += 1;
        let mut data = Vec::new();
        let mut escape = false;
        while !self.is_eos() {
            let c = self.raw[self.pos];
            if c == b'\\' && !escape {
                escape = true;
                self.pos += 1;
            } else {
                if c == b'"' && !escape {
                    self.pos += 1;
                    if data.is_empty() {
                        bail!("empty text slice");
                    } else {
                        return Ok(data);
                    }
                } else if escape && c.is_ascii_digit() {
                    if self.raw.len() - self.pos < 3 {
                        bail!("num is short than 3 bytes");
                    }
                    let num: u8 = from_utf8(&self.raw[self.pos..(self.pos + 3)])?.parse()?;
                    data.push(num);
                    self.pos += 3;
                } else {
                    data.push(c);
                    self.pos += 1;
                }
                escape = false;
            }
        }
        bail!("quote isn't in pair");
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eos() && self.raw[self.pos].is_ascii_whitespace() {
            self.pos += 1
        }
    }

    pub fn read<T>(&mut self) -> Result<T>
    where
        T: FromStr,
        <T as std::str::FromStr>::Err: ToString,
    {
        if let Some(s) = self.read_str() {
            s.parse::<T>().map_err(|e| anyhow!(e.to_string()))
        } else {
            bail!("empty string",)
        }
    }

    pub fn read_str(&mut self) -> Option<&'a str> {
        self.skip_whitespace();
        let start = self.pos;
        while !self.is_eos() && !self.raw[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
        if self.pos == start {
            None
        } else {
            Some(from_utf8(&self.raw[start..self.pos]).unwrap())
        }
    }

    pub fn left_str(self) -> Option<&'a str> {
        if self.is_eos() {
            None
        } else {
            Some(from_utf8(&self.raw[self.pos..]).unwrap())
        }
    }

    fn is_eos(&self) -> bool {
        self.pos == self.raw.len()
    }
}

impl<'a> Iterator for StringBuffer<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        self.read_str()
    }
}

#[cfg(test)]
mod test {
    use super::StringBuffer;
    #[test]
    fn test_parser_iterator() {
        let s = " example.org. 100 IN SOA xxx.net. ns.example.org. 100 1800 900 604800 86400    ";
        let mut iter = StringBuffer::new(s);
        let mut split_white = s.split_whitespace();
        let mut label_count = 0;
        loop {
            if let Some(label) = iter.next() {
                assert_eq!(label, split_white.next().unwrap());
                label_count += 1;
            } else {
                break;
            }
        }
        assert_eq!(label_count, 11);
    }

    #[test]
    fn test_into_string() {
        let s = " example.org. 100 IN SOA xxx.net. ns.example.org. 100 1800 900 604800 86400    ";
        let mut iter = StringBuffer::new(s);
        iter.next();
        iter.next();
        assert_eq!(
            iter.left_str().unwrap(),
            " IN SOA xxx.net. ns.example.org. 100 1800 900 604800 86400    "
        );
    }

    #[test]
    fn test_read_text() {
        let s = r#" "abc" "edf""#;
        let data = StringBuffer::new(s).read_text().unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], "abc".as_bytes().to_vec());
        assert_eq!(data[1], "edf".as_bytes().to_vec());

        let s = r#" "abc edf""#;
        let data = StringBuffer::new(s).read_text().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], "abc edf".as_bytes().to_vec());

        let s = r#" "abc\"cd\" edf""#;
        let data = StringBuffer::new(s).read_text().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], r#"abc"cd" edf"#.as_bytes().to_vec());

        let s = r#" "a\011d" "#;
        let data = StringBuffer::new(s).read_text().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0][1], 11);
    }
}
