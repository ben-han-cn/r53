use crate::domaintree::flag::{Color, NodeFlag};
use crate::label_sequence::LabelSequence;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::mem::swap;
use std::ptr;

pub struct Node<T> {
    pub flag: NodeFlag,
    pub left: NodePtr<T>,
    pub right: NodePtr<T>,
    pub parent: NodePtr<T>,
    pub down: NodePtr<T>,
    pub name: LabelSequence,
    pub value: Option<T>,
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "k:{:?} v:{:?} c:{:?}",
            self.name,
            self.value,
            self.flag.get_color(),
        )
    }
}

#[derive(Debug)]
pub struct NodePtr<T>(pub *mut Node<T>);

impl<T> Clone for NodePtr<T> {
    fn clone(&self) -> NodePtr<T> {
        NodePtr(self.0)
    }
}

impl<T> Copy for NodePtr<T> {}
unsafe impl<T> Send for NodePtr<T> {}
unsafe impl<T> Sync for NodePtr<T> {}

impl<T> Ord for NodePtr<T> {
    fn cmp(&self, other: &NodePtr<T>) -> Ordering {
        unsafe { (*self.0).name.cmp(&(*other.0).name) }
    }
}

impl<T> PartialOrd for NodePtr<T> {
    fn partial_cmp(&self, other: &NodePtr<T>) -> Option<Ordering> {
        unsafe { Some((*self.0).name.cmp(&(*other.0).name)) }
    }
}

impl<T> PartialEq for NodePtr<T> {
    fn eq(&self, other: &NodePtr<T>) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for NodePtr<T> {}

impl<T> NodePtr<T> {
    pub fn new(name: LabelSequence, v: Option<T>) -> NodePtr<T> {
        let node = Node {
            flag: NodeFlag::default(),
            left: NodePtr::null(),
            right: NodePtr::null(),
            parent: NodePtr::null(),
            down: NodePtr::null(),
            name,
            value: v,
        };
        NodePtr(Box::into_raw(Box::new(node)))
    }

    pub fn set_color(self, color: Color) {
        unsafe {
            (*self.0).flag.set_color(color);
        }
    }

    pub fn get_color(self) -> Color {
        if self.is_null() {
            Color::Black
        } else {
            unsafe { (*self.0).flag.get_color() }
        }
    }

    pub fn is_red(self) -> bool {
        self.get_color() == Color::Red
    }

    pub fn is_black(self) -> bool {
        !self.is_red()
    }

    pub fn set_subtree_root(self, enable: bool) {
        unsafe {
            (*self.0).flag.set_subtree_root(enable);
        }
    }

    pub fn is_subtree_root(self) -> bool {
        unsafe { (*self.0).flag.is_subtree_root() }
    }

    pub fn set_callback(self, enable: bool) {
        unsafe {
            (*self.0).flag.set_callback(enable);
        }
    }

    pub fn is_callback_enabled(self) -> bool {
        unsafe { (*self.0).flag.is_callback_enabled() }
    }

    pub fn set_wildcard(self, enable: bool) {
        unsafe {
            (*self.0).flag.set_wildcard(enable);
        }
    }

    pub fn is_wildcard(self) -> bool {
        unsafe { (*self.0).flag.is_wildcard() }
    }

    pub fn get_name(&self) -> &LabelSequence {
        unsafe { &(*self.0).name }
    }

    pub fn get_value(&self) -> &Option<T> {
        unsafe { &(*self.0).value }
    }

    pub fn get_value_mut(&mut self) -> &mut Option<T> {
        unsafe { &mut (*self.0).value }
    }

    pub fn set_name(self, n: LabelSequence) {
        unsafe {
            (*self.0).name = n;
        }
    }

    pub fn set_value(self, v: Option<T>) -> Option<T> {
        let mut back = v;
        unsafe {
            swap(&mut (*self.0).value, &mut back);
        }
        back
    }

    pub fn set_parent(self, parent: NodePtr<T>) {
        unsafe { (*self.0).parent = parent }
    }

    pub fn set_left(self, left: NodePtr<T>) {
        unsafe { (*self.0).left = left }
    }

    pub fn set_right(self, right: NodePtr<T>) {
        unsafe { (*self.0).right = right }
    }

    pub fn parent(self) -> NodePtr<T> {
        unsafe { (*self.0).parent }
    }

    pub fn down(self) -> NodePtr<T> {
        unsafe { (*self.0).down }
    }

    pub fn set_down(self, down: NodePtr<T>) {
        unsafe { (*self.0).down = down }
    }

    pub fn grand_parent(self) -> NodePtr<T> {
        let parent = self.parent();
        if parent.is_null() {
            NodePtr::null()
        } else {
            parent.parent()
        }
    }

    pub fn uncle(self) -> NodePtr<T> {
        let grand_parent = self.grand_parent();
        if grand_parent.is_null() {
            return NodePtr::null();
        }

        if self.parent() == grand_parent.left() {
            grand_parent.right()
        } else {
            grand_parent.left()
        }
    }

    pub fn left(self) -> NodePtr<T> {
        unsafe { (*self.0).left }
    }

    pub fn right(self) -> NodePtr<T> {
        unsafe { (*self.0).right }
    }

    pub fn null() -> NodePtr<T> {
        NodePtr(ptr::null_mut())
    }

    pub fn subtree_root(self) -> NodePtr<T> {
        let mut node = self;
        while !node.is_subtree_root() {
            node = node.parent();
        }
        node
    }

    pub fn get_upper_node(self) -> NodePtr<T> {
        self.subtree_root().parent()
    }

    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    pub fn get_pointer(self) -> *mut Node<T> {
        self.0
    }

    pub fn get_double_pointer(&mut self) -> *mut *mut Node<T> {
        &mut self.0
    }

    pub fn get_double_pointer_of_down(&mut self) -> *mut *mut Node<T> {
        unsafe { &mut (*self.0).down.0 }
    }

    pub unsafe fn exchange(&mut self, lower: NodePtr<T>, root: *mut *mut Node<T>) {
        swap(&mut (*self.0).left, &mut (*lower.0).left);
        if lower.left() == lower {
            lower.set_left(*self);
        }

        swap(&mut (*self.0).right, &mut (*lower.0).right);
        if lower.right() == lower {
            lower.set_right(*self);
        }

        swap(&mut (*self.0).parent, &mut (*lower.0).parent);
        if self.parent() == *self {
            self.set_parent(lower);
        }

        let self_color = self.get_color();
        self.set_color(lower.get_color());
        lower.set_color(self_color);
        let self_is_subroot = self.is_subtree_root();
        self.set_subtree_root(lower.is_subtree_root());
        lower.set_subtree_root(self_is_subroot);

        connect_child(root, lower, *self, lower);
        if self.parent().left() == lower {
            self.parent().set_left(*self);
        } else if self.parent().right() == lower {
            self.parent().set_right(*self);
        }
        if !lower.right().is_null() {
            lower.right().set_parent(lower);
        }
        if !lower.left().is_null() {
            lower.left().set_parent(lower);
        }
    }

    pub unsafe fn split_to_parent(&mut self, parent_label_count: usize) -> NodePtr<T> {
        let name = &mut (*self.0).name;
        let parent = name
            .split(name.label_count() - parent_label_count, parent_label_count)
            .unwrap();
        NodePtr::new(parent, None)
    }
}

impl<T: Clone> NodePtr<T> {
    pub unsafe fn deep_clone(self) -> NodePtr<T> {
        let node = NodePtr::new((*self.0).name.clone(), (*self.0).value.clone());
        if !self.left().is_null() {
            node.set_left(self.left().deep_clone());
            node.left().set_parent(node);
        }
        if !self.right().is_null() {
            node.set_right(self.right().deep_clone());
            node.right().set_parent(node);
        }
        node
    }
}

pub fn get_sibling<T>(parent: NodePtr<T>, child: NodePtr<T>) -> NodePtr<T> {
    if parent.is_null() {
        NodePtr::null()
    } else if parent.left() == child {
        parent.right()
    } else {
        parent.left()
    }
}

pub unsafe fn connect_child<T>(
    root: *mut *mut Node<T>,
    current: NodePtr<T>,
    old: NodePtr<T>,
    new: NodePtr<T>,
) {
    let parent = current.parent();
    if parent.is_null() {
        *root = new.get_pointer();
    } else if parent.left() == old {
        parent.set_left(new)
    } else if parent.right() == old {
        parent.set_right(new);
    } else {
        parent.set_down(new);
    }
}

#[cfg(test)]
mod tests {
    use super::NodePtr;
    use crate::label_sequence::LabelSequence;
    use std::str::FromStr;

    #[test]
    fn test_set_value() {
        let name = LabelSequence::from_str("k1").unwrap();
        let n = NodePtr::new(name.clone(), Some("v1"));
        assert_eq!(n.get_value(), &Some("v1"));
        let old = n.set_value(Some("v2"));
        assert_eq!(old, Some("v1"));
        assert_eq!(n.get_value(), &Some("v2"));

        let old = n.set_value(None);
        assert_eq!(old, Some("v2"));
        assert_eq!(n.get_value(), &None);
    }

    #[test]
    fn test_double_pointer() {
        let name = LabelSequence::from_str("k1").unwrap();
        let mut n1 = NodePtr::new(name.clone(), Some("v1"));
        let n2 = NodePtr::new(name.clone(), Some("v2"));

        assert!(n1.is_red());
        assert_eq!(n1.get_value(), &Some("v1"));
        assert!(n1.down().is_null());

        let pp = n1.get_double_pointer_of_down();
        unsafe {
            *pp = n2.get_pointer();
        }
        assert_eq!(n1.down().get_value(), &Some("v2"));
    }
}
