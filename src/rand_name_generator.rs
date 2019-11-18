use crate::name::{Name, MAX_LABEL_COUNT, MAX_LABEL_LEN, MAX_WIRE_LEN};
use rand::prelude::*;
use std::cmp::Ord;

const VALID_DOMAIN_CHAR: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
              abcdefghijklmnopqrstuvwxyz\
              0123456789-";

pub struct RandNameGenerator {
    rng: ThreadRng,
}

impl RandNameGenerator {
    pub fn new() -> Self {
        RandNameGenerator { rng: thread_rng() }
    }

    fn gen_label(&mut self, len: u8) -> String {
        let mut label = String::with_capacity(len as usize);
        for _i in 0..len {
            let index = self.rng.gen_range(0, VALID_DOMAIN_CHAR.len());
            label.push(VALID_DOMAIN_CHAR[index] as char);
        }
        label
    }

    pub fn gen_name_string(&mut self) -> String {
        let len = self.rng.gen_range(1, MAX_WIRE_LEN - 1) as u8;
        let label_count = self.rng.gen_range(1, MAX_LABEL_COUNT);
        let mut name = String::with_capacity(len as usize);
        let mut generated_len = 0;
        for _i in 0..label_count {
            let max_label_len = MAX_LABEL_LEN.min(len - generated_len);
            let mut label_len = if max_label_len > 1 {
                self.rng.gen_range(1, max_label_len)
            } else {
                1
            };
            if generated_len + label_len + 1 == len {
                label_len += 1;
            }
            name.push_str(self.gen_label(label_len).as_ref());
            generated_len += label_len;
            if generated_len < len {
                name.push('.');
                generated_len += 1;
            }
            if generated_len == len {
                break;
            }
        }
        name
    }
}

impl Iterator for RandNameGenerator {
    type Item = Name;
    fn next(&mut self) -> Option<Name> {
        Some(Name::new(self.gen_name_string().as_ref()).unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_gen_name() {
        let mut gen = RandNameGenerator::new();
        let mut names = HashSet::new();
        let mut duplicate = 0;
        for _i in 0..1000 {
            let name = gen.next().unwrap();
            if names.contains(&name) {
                duplicate += 1;
            }
            names.insert(name);
        }
        assert!(duplicate < 3);
    }
}
