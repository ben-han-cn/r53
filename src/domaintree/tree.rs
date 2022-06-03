use crate::label_slice::LabelSlice;
use crate::name::{Name, NameRelation};
use std::{marker::PhantomData, mem};

use crate::domaintree::flag::Color;
use crate::domaintree::node::{connect_child, get_sibling, Node, NodePtr};
use crate::domaintree::node_chain::NodeChain;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FindResultFlag {
    ExacatMatch,
    NotFound,
    PartialMatch,
}

pub struct FindResult<'a, T: 'a> {
    pub node: NodePtr<T>,
    pub flag: FindResultFlag,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> FindResult<'a, T> {
    fn new(_tree: &'a DomainTree<T>) -> Self {
        FindResult {
            node: NodePtr::null(),
            flag: FindResultFlag::NotFound,
            phantom: PhantomData,
        }
    }

    pub fn get_value(&self) -> Option<&'a T> {
        if self.flag == FindResultFlag::NotFound {
            None
        } else {
            debug_assert!(!self.node.is_null());
            unsafe { (*self.node.0).value.as_ref() }
        }
    }

    pub fn get_value_mut(&self) -> Option<&'a mut T> {
        if self.flag == FindResultFlag::NotFound {
            None
        } else {
            debug_assert!(!self.node.is_null());
            unsafe { (*self.node.0).value.as_mut() }
        }
    }
}

pub struct DomainTree<T> {
    root: NodePtr<T>,
    len: usize,
}

impl<T> Default for DomainTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for DomainTree<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T: Clone> Clone for DomainTree<T> {
    fn clone(&self) -> DomainTree<T> {
        unsafe {
            let mut new = DomainTree::new();
            new.root = self.root.deep_clone();
            new.len = self.len;
            new
        }
    }
}

impl<T> DomainTree<T> {
    pub fn new() -> DomainTree<T> {
        DomainTree {
            root: NodePtr::null(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }

    unsafe fn left_rotate(&mut self, root: *mut *mut Node<T>, node: NodePtr<T>) {
        let right = node.right();
        let rleft = right.left();
        node.set_right(rleft);
        if !rleft.is_null() {
            rleft.set_parent(node);
        }

        let parent = node.parent();
        right.set_parent(parent);
        if !node.is_subtree_root() {
            right.set_subtree_root(false);
            if node == parent.left() {
                parent.set_left(right);
            } else {
                parent.set_right(right);
            }
        } else {
            right.set_subtree_root(true);
            *root = right.get_pointer();
        }
        right.set_left(node);
        node.set_parent(right);
        node.set_subtree_root(false);
    }

    unsafe fn right_rotate(&mut self, root: *mut *mut Node<T>, node: NodePtr<T>) {
        let left = node.left();
        let lright = left.right();
        node.set_left(lright);
        if !lright.is_null() {
            lright.set_parent(node);
        }

        let parent = node.parent();
        left.set_parent(parent);
        if !node.is_subtree_root() {
            left.set_subtree_root(false);
            if node == parent.right() {
                parent.set_right(left);
            } else {
                parent.set_left(left);
            }
        } else {
            left.set_subtree_root(true);
            *root = left.get_pointer();
        }
        left.set_right(node);
        node.set_parent(left);
        node.set_subtree_root(false);
    }

    unsafe fn insert_fixup(&mut self, root: *mut *mut Node<T>, node_: NodePtr<T>) {
        let mut node = node_;
        while node.get_pointer() != *root {
            let mut parent = node.parent();
            if parent.is_black() {
                break;
            }

            let uncle = node.uncle();
            let grand_parent = node.grand_parent();
            if !uncle.is_null() && uncle.is_red() {
                parent.set_color(Color::Black);
                uncle.set_color(Color::Black);
                grand_parent.set_color(Color::Red);
                node = grand_parent;
            } else {
                if node == parent.right() && parent == grand_parent.left() {
                    node = parent;
                    self.left_rotate(root, parent);
                } else if node == parent.left() && parent == grand_parent.right() {
                    node = parent;
                    self.right_rotate(root, parent);
                }
                parent = node.parent();
                parent.set_color(Color::Black);
                grand_parent.set_color(Color::Red);
                if node == parent.left() {
                    self.right_rotate(root, grand_parent);
                } else {
                    self.left_rotate(root, grand_parent);
                }
                break;
            }
        }
        (**root).flag.set_color(Color::Black);
    }

    //if old node exists, overwrite the value, and return old value
    pub fn insert(&mut self, target: Name, v: Option<T>) -> (NodePtr<T>, Option<Option<T>>) {
        let mut parent = NodePtr::null();
        let mut up = NodePtr::null();
        let mut current = self.root;
        let mut order = -1;
        let mut target_slice = LabelSlice::from_name(&target);
        while !current.is_null() {
            let current_name = LabelSlice::from_label_sequence(current.get_name());
            let compare_result = target_slice.compare(&current_name, false);
            match compare_result.relation {
                NameRelation::Equal => unsafe {
                    return (current, Some(mem::replace(&mut (*current.0).value, v)));
                },
                NameRelation::None => {
                    parent = current;
                    order = compare_result.order;
                    current = if order < 0 {
                        current.left()
                    } else {
                        current.right()
                    };
                }
                NameRelation::SubDomain => {
                    parent = NodePtr::null();
                    up = current;
                    target_slice.strip_right(compare_result.common_label_count as usize);
                    current = current.down();
                }
                _ => {
                    unsafe {
                        self.node_fission(&mut current, compare_result.common_label_count as usize);
                    }
                    current = current.parent();
                }
            }
        }

        let current_root = if !up.is_null() {
            up.get_double_pointer_of_down()
        } else {
            self.root.get_double_pointer()
        };
        self.len += 1;
        let first_label = target_slice.first_label();
        let last_label = target_slice.last_label();
        let node = NodePtr::new(target.into_label_sequence(first_label, last_label), v);
        node.set_parent(parent);
        if parent.is_null() {
            unsafe {
                *current_root = node.get_pointer();
            }
            node.set_color(Color::Black);
            node.set_subtree_root(true);
            node.set_parent(up);
        } else if order < 0 {
            node.set_subtree_root(false);
            parent.set_left(node);
            unsafe {
                self.insert_fixup(current_root, node);
            }
        } else {
            node.set_subtree_root(false);
            parent.set_right(node);
            unsafe {
                self.insert_fixup(current_root, node);
            }
        }
        (node, None)
    }

    unsafe fn node_fission(&mut self, node: &mut NodePtr<T>, parent_label_count: usize) {
        let up = node.split_to_parent(parent_label_count);
        up.set_parent(node.parent());
        connect_child(self.root.get_double_pointer(), *node, *node, up);
        up.set_down(*node);
        node.set_parent(up);
        up.set_left(node.left());
        if !node.left().is_null() {
            node.left().set_parent(up);
        }
        up.set_right(node.right());
        if !node.right().is_null() {
            node.right().set_parent(up);
        }
        node.set_left(NodePtr::null());
        node.set_right(NodePtr::null());
        up.set_color(node.get_color());
        node.set_color(Color::Black);
        up.set_subtree_root(node.is_subtree_root());
        node.set_subtree_root(true);
        self.len += 1;
    }

    pub fn find(&self, target: &Name) -> FindResult<T> {
        let mut node_chain = NodeChain::new(self);
        self.find_node(target, &mut node_chain)
    }

    pub fn find_node<'a>(&'a self, target_: &Name, chain: &mut NodeChain<'a, T>) -> FindResult<T> {
        self.find_node_ext(
            target_,
            chain,
            &mut None::<fn(_, _, &mut Option<usize>) -> bool>,
            &mut None,
        )
    }

    pub fn find_node_ext<'a, P, F: FnMut(NodePtr<T>, Name, &mut P) -> bool>(
        &'a self,
        target: &Name,
        chain: &mut NodeChain<'a, T>,
        callback: &mut Option<F>,
        param: &mut P,
    ) -> FindResult<T> {
        let mut node = self.root;
        let mut result = FindResult::new(self);
        let mut target_slice = LabelSlice::from_name(target);
        while !node.is_null() {
            let current_slice = LabelSlice::from_label_sequence(node.get_name());
            chain.last_compared = node;
            chain.last_compared_result = target_slice.compare(&current_slice, false);
            match chain.last_compared_result.relation {
                NameRelation::Equal => {
                    chain.push(node);
                    result.flag = FindResultFlag::ExacatMatch;
                    result.node = node;
                    break;
                }
                NameRelation::None => {
                    if chain.last_compared_result.order < 0 {
                        node = node.left();
                    } else {
                        node = node.right();
                    }
                }
                NameRelation::SubDomain => {
                    result.flag = FindResultFlag::PartialMatch;
                    result.node = node;
                    if node.is_callback_enabled()
                        && callback.is_some()
                        && callback.as_mut().unwrap()(
                            node,
                            chain.get_absolute_name(node.get_name()),
                            param,
                        )
                    {
                        break;
                    }
                    chain.push(node);
                    target_slice
                        .strip_right(chain.last_compared_result.common_label_count as usize);
                    node = node.down();
                }
                _ => {
                    break;
                }
            }
        }
        result
    }

    pub fn remove(&mut self, name: &Name) -> Option<T> {
        let node = {
            let result = self.find(name);
            result.node
        };

        if !node.is_null() {
            self.remove_node(node)
        } else {
            None
        }
    }

    pub fn remove_node(&mut self, mut node: NodePtr<T>) -> Option<T> {
        let old_value = node.set_value(None);

        if !node.down().is_null() {
            return old_value;
        }

        loop {
            let mut up = node.get_upper_node();
            if !node.left().is_null() && !node.right().is_null() {
                let mut right_most = node.left();
                while !right_most.right().is_null() {
                    right_most = right_most.right();
                }
                unsafe {
                    node.exchange(right_most, self.root.get_double_pointer());
                }
            }

            let child = if !node.right().is_null() {
                node.right()
            } else {
                node.left()
            };

            unsafe {
                connect_child(self.root.get_double_pointer(), node, node, child);
            }

            if !child.is_null() {
                child.set_parent(node.parent());
                if child.parent().is_null() || child.parent().down() == child {
                    child.set_subtree_root(node.is_subtree_root());
                }
            }

            if node.is_black() {
                if !child.is_null() && child.is_red() {
                    child.set_color(Color::Black);
                } else {
                    let current_root = if !up.is_null() {
                        up.get_double_pointer_of_down()
                    } else {
                        self.root.get_double_pointer()
                    };

                    unsafe {
                        self.remove_fixup(current_root, child, node.parent());
                    }
                }
            }

            self.len -= 1;

            if up.is_null() || up.get_value().is_some() || !up.down().is_null() {
                break;
            }

            node = up;
        }
        old_value
    }

    unsafe fn remove_fixup(
        &mut self,
        root: *mut *mut Node<T>,
        mut child: NodePtr<T>,
        mut parent: NodePtr<T>,
    ) {
        while child.get_pointer() != *root && child.is_black() {
            if !parent.is_null() && parent.down().get_pointer() == *root {
                break;
            }

            let mut sibling = get_sibling(parent, child);
            if sibling.is_red() {
                parent.set_color(Color::Red);
                sibling.set_color(Color::Black);
                if parent.left() == child {
                    self.left_rotate(root, parent);
                } else {
                    self.right_rotate(root, parent);
                }
                sibling = get_sibling(parent, child);
            }
            if sibling.left().is_black() && sibling.right().is_black() {
                sibling.set_color(Color::Red);
                if parent.is_black() {
                    child = parent;
                    parent = parent.parent();
                    continue;
                } else {
                    parent.set_color(Color::Black);
                    break;
                }
            }

            let mut ss1 = sibling.left();
            let mut ss2 = sibling.right();
            if parent.left() != child {
                mem::swap(&mut ss1, &mut ss2);
            }

            if ss2.is_black() {
                sibling.set_color(Color::Red);
                ss1.set_color(Color::Black);
                if parent.left() == child {
                    self.right_rotate(root, sibling);
                } else {
                    self.left_rotate(root, sibling);
                }
                sibling = get_sibling(parent, child);
            }

            sibling.set_color(parent.get_color());
            parent.set_color(Color::Black);
            ss1 = sibling.left();
            ss2 = sibling.right();
            if parent.left() != child {
                mem::swap(&mut ss1, &mut ss2);
            }
            ss2.set_color(Color::Black);
            if parent.left() == child {
                self.left_rotate(root, parent);
            } else {
                self.right_rotate(root, parent);
            }

            break;
        }
    }

    fn clear_recurse(&mut self, current: NodePtr<T>) {
        if !current.is_null() {
            unsafe {
                self.clear_recurse(current.left());
                self.clear_recurse(current.right());
                self.clear_recurse(current.down());
                Box::from_raw(current.0);
            }
        }
    }

    pub fn clear(&mut self) {
        let root = self.root;
        self.root = NodePtr::null();
        self.clear_recurse(root);
    }

    pub fn dump(&self, depth: usize) {
        indent(depth);
        println!("tree has {} node(s)", self.len);
        self.dump_helper(self.root, depth);
    }

    fn dump_helper(&self, node: NodePtr<T>, depth: usize) {
        if node.is_null() {
            indent(depth);
            println!("NULL");
            return;
        }
        indent(depth);

        let parent = node.parent();
        if !parent.is_null() {
            if parent.left() == node {
                print!("left>");
            } else if parent.right() == node {
                print!("right>");
            }
        }

        print!("{} ({:?})", node.get_name().to_string(), node.get_color());
        if node.get_value().is_none() {
            print!("[invisible]");
        }
        if node.is_subtree_root() {
            print!(" [subtreeroot]");
        }
        println!();

        let down = node.down();
        if !down.is_null() {
            indent(depth + 1);
            println!("begin down from {}\n", down.get_name().to_string());
            self.dump_helper(down, depth + 1);
            indent(depth + 1);
            println!("end down from {}", down.get_name().to_string());
        }
        self.dump_helper(node.left(), depth + 1);
        self.dump_helper(node.right(), depth + 1);
    }
}

fn indent(depth: usize) {
    const INDENT_FOR_EACH_DEPTH: usize = 5;
    print!("{}", " ".repeat((depth * INDENT_FOR_EACH_DEPTH) as usize));
}

#[cfg(test)]
mod tests {
    use super::{DomainTree, FindResultFlag, NodeChain, NodePtr};
    use crate::name::Name;

    fn sample_names() -> Vec<(&'static str, i32)> {
        vec![
            "c",
            "b",
            "a",
            "x.d.e.f",
            "z.d.e.f",
            "g.h",
            "i.g.h",
            "o.w.y.d.e.f",
            "j.z.d.e.f",
            "p.w.y.d.e.f",
            "q.w.y.d.e.f",
        ]
        .iter()
        .zip(0..)
        .map(|(&s, v)| (s, v))
        .collect()
    }

    fn build_tree(data: &Vec<(&'static str, i32)>) -> DomainTree<i32> {
        let mut tree = DomainTree::new();
        for (k, v) in data {
            tree.insert(Name::new(k).unwrap(), Some(*v));
        }
        tree
    }

    #[test]
    fn test_find() {
        let data = sample_names();
        let tree = build_tree(&data);
        assert_eq!(tree.len(), 14);

        for (n, v) in sample_names() {
            let mut node_chain = NodeChain::new(&tree);
            let result = tree.find_node(&Name::new(n).unwrap(), &mut node_chain);
            assert_eq!(result.flag, FindResultFlag::ExacatMatch);
            assert_eq!(result.node.get_value(), &Some(v));
        }

        let none_terminal = vec!["d.e.f", "w.y.d.e.f"];
        for n in &none_terminal {
            let mut node_chain = NodeChain::new(&tree);
            let result = tree.find_node(&Name::new(n).unwrap(), &mut node_chain);
            assert_eq!(result.flag, FindResultFlag::ExacatMatch);
            assert_eq!(result.node.get_value(), &None);
        }
    }

    #[test]
    fn test_delete() {
        let data = sample_names();
        let mut tree = build_tree(&data);
        assert_eq!(tree.len(), 14);
        for (n, v) in data {
            let result = tree.find(&Name::new(n).unwrap());
            assert_eq!(result.flag, FindResultFlag::ExacatMatch);
            let node = result.node;
            assert_eq!(tree.remove_node(node), Some(v));
        }
        assert_eq!(tree.len(), 0);
    }

    use std::cell::Cell;
    use std::rc::Rc;
    pub struct NumberWrapper(Rc<Cell<i32>>);
    impl NumberWrapper {
        fn new(c: Rc<Cell<i32>>) -> Self {
            c.set(c.get() + 1);
            NumberWrapper(c)
        }
    }

    impl Drop for NumberWrapper {
        fn drop(&mut self) {
            self.0.set(self.0.get() - 1);
        }
    }

    #[test]
    fn test_clean() {
        let num = Rc::new(Cell::new(0));
        {
            let mut tree = DomainTree::new();
            for name in vec!["a", "b", "c", "d"] {
                tree.insert(
                    Name::new(name).unwrap(),
                    Some(NumberWrapper::new(num.clone())),
                );
            }
            assert_eq!(num.get(), 4);
        }
        assert_eq!(num.get(), 0);
    }

    #[test]
    fn test_callback() {
        let mut tree = DomainTree::new();
        for name in vec!["a", "b", "c", "d"] {
            tree.insert(Name::new(name).unwrap(), Some(10));
        }
        let (n, _) = tree.insert(Name::new("e").unwrap(), Some(20));
        n.set_callback(true);

        tree.insert(Name::new("b.e").unwrap(), Some(30));
        let mut num = 0;
        let callback = |n: NodePtr<u32>, name: Name, num: &mut u32| {
            assert_eq!(name.to_string(), "e.");
            *num = *num + n.get_value().unwrap();
            false
        };
        let mut node_chain = NodeChain::new(&tree);
        let result = tree.find_node_ext(
            &Name::new("b.e").unwrap(),
            &mut node_chain,
            &mut Some(callback),
            &mut num,
        );
        assert_eq!(num, 20);
        assert_eq!(result.flag, FindResultFlag::ExacatMatch);
        assert_eq!(result.node.get_value(), &Some(30));

        let mut node_chain = NodeChain::new(&tree);
        tree.find_node_ext(
            &Name::new("e").unwrap(),
            &mut node_chain,
            &mut Some(callback),
            &mut num,
        );
        //only query sub domain, callback will be invoked
        assert_eq!(num, 20);

        //callback return true, skip travel
        let callback = |n: NodePtr<u32>, _, num: &mut u32| {
            *num = *num + n.get_value().unwrap();
            true
        };
        let mut node_chain = NodeChain::new(&tree);
        let result = tree.find_node_ext(
            &Name::new("b.e").unwrap(),
            &mut node_chain,
            &mut Some(callback),
            &mut num,
        );
        assert_eq!(num, 40);
        assert_eq!(result.flag, FindResultFlag::PartialMatch);
    }

    #[test]
    fn test_rand_tree_insert_and_search() {
        use crate::rand_name_generator::RandNameGenerator;
        for _ in 0..20 {
            let gen = RandNameGenerator::new();
            test_insert_delete_batch(gen.take(1000).collect::<Vec<Name>>());
        }
    }

    pub fn test_insert_delete_batch(names: Vec<Name>) {
        use std::collections::HashSet;
        let mut tree = DomainTree::<usize>::new();
        let mut duplicate_name_index = HashSet::new();
        for (i, name) in names.iter().enumerate() {
            let (_, old) = tree.insert(name.clone(), Some(i));
            //Some(None) == non-terminal node is created
            //None == new node
            if let Some(Some(v)) = old {
                assert!(name.eq(&names[v]));
                duplicate_name_index.insert(v);
            }
        }

        //duplicate insert should return old value
        for (i, name) in names.iter().enumerate() {
            if !duplicate_name_index.contains(&i) {
                let (_, old) = tree.insert(name.clone(), Some(i));
                assert_eq!(old.unwrap(), Some(i));
            }
        }

        for (i, name) in names.iter().enumerate() {
            if !duplicate_name_index.contains(&i) {
                let mut node_chain = NodeChain::new(&tree);
                let result = tree.find_node(name, &mut node_chain);
                assert_eq!(result.flag, FindResultFlag::ExacatMatch);
                assert_eq!(result.node.get_value(), &Some(i));
            }
        }

        for (i, name) in names.iter().enumerate() {
            if !duplicate_name_index.contains(&i) {
                let mut node_chain = NodeChain::new(&tree);
                let result = tree.find_node(&name, &mut node_chain);
                let node = result.node;
                assert_eq!(tree.remove_node(node).unwrap(), i);
            }
        }

        assert_eq!(tree.len(), 0);
    }
}
