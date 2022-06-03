#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NodeFlag(u16);

impl NodeFlag {
    #[inline]
    fn set(&mut self, is_enable: bool, mask: u16) {
        if is_enable {
            self.enable(mask);
        } else {
            self.disable(mask);
        }
    }

    #[inline]
    fn enable(&mut self, mask: u16) {
        self.0 |= mask
    }

    #[inline]
    fn disable(&mut self, mask: u16) {
        self.0 &= !mask
    }

    #[inline]
    fn is_enable(self, mask: u16) -> bool {
        (self.0 & mask) != 0
    }
}

impl Default for NodeFlag {
    fn default() -> Self {
        NodeFlag(0)
    }
}

const COLOR_MASK: u16 = 0x0001;
const SUBTREE_ROOT_MASK: u16 = 0x0002;
const CALLBACK_MASK: u16 = 0x0004;
const WILDCARD_MASK: u16 = 0x0008;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
}

impl NodeFlag {
    #[inline]
    pub fn is_red(self) -> bool {
        !self.is_black()
    }

    #[inline]
    pub fn is_black(self) -> bool {
        self.is_enable(COLOR_MASK)
    }

    #[inline]
    pub fn set_color(&mut self, color: Color) {
        match color {
            Color::Red => self.disable(COLOR_MASK),
            Color::Black => self.enable(COLOR_MASK),
        }
    }

    #[inline]
    pub fn get_color(self) -> Color {
        if self.is_enable(COLOR_MASK) {
            Color::Black
        } else {
            Color::Red
        }
    }

    #[inline]
    pub fn set_subtree_root(&mut self, enable: bool) {
        self.set(enable, SUBTREE_ROOT_MASK)
    }

    #[inline]
    pub fn is_subtree_root(self) -> bool {
        self.is_enable(SUBTREE_ROOT_MASK)
    }

    #[inline]
    pub fn set_callback(&mut self, enable: bool) {
        self.set(enable, CALLBACK_MASK)
    }

    #[inline]
    pub fn is_callback_enabled(self) -> bool {
        self.is_enable(CALLBACK_MASK)
    }

    #[inline]
    pub fn set_wildcard(&mut self, enable: bool) {
        self.set(enable, WILDCARD_MASK)
    }

    #[inline]
    pub fn is_wildcard(self) -> bool {
        self.is_enable(WILDCARD_MASK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag() {
        let mut flag = NodeFlag::default();
        assert!(flag.is_red());
        assert!(flag.is_subtree_root() == false);
        assert!(flag.is_callback_enabled() == false);
        assert!(flag.is_wildcard() == false);

        flag.set_color(Color::Red);
        assert!(flag.is_red());
        flag.set_color(Color::Black);
        assert!(flag.is_black());
        flag.set_subtree_root(true);
        assert!(flag.is_subtree_root());
        flag.set_callback(true);
        assert!(flag.is_callback_enabled());
        flag.set_wildcard(true);
        assert!(flag.is_wildcard());
        assert!(flag.is_callback_enabled());
    }
}
