use crate::domaintree::{node::NodePtr, tree::DomainTree};
use crate::{name::NameComparisonResult, name::MAX_LABEL_COUNT, LabelSequence, Name, NameRelation};
use std::{fmt, marker::PhantomData};

pub struct NodeChain<'a, T: 'a> {
    pub level_count: usize,
    pub nodes: [NodePtr<T>; MAX_LABEL_COUNT as usize],
    pub last_compared: NodePtr<T>,
    pub last_compared_result: NameComparisonResult,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> fmt::Display for NodeChain<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.last_compared.is_null() {
            write!(f, "level: {}, last_compared is nil", self.level_count)
        } else {
            write!(
                f,
                "level: {}, last_compared_relation {:?}",
                self.level_count, self.last_compared_result
            )?;
            write!(f, " nodes:[")?;
            for n in &self.nodes[..self.level_count] {
                write!(f, "{},", n.get_name())?;
            }
            write!(f, " ]")?;
            Ok(())
        }
    }
}

impl<'a, T> NodeChain<'a, T> {
    pub fn new(_tree: &'a DomainTree<T>) -> Self {
        NodeChain {
            level_count: 0,
            nodes: [NodePtr::null(); MAX_LABEL_COUNT as usize],
            last_compared: NodePtr::null(),
            last_compared_result: NameComparisonResult {
                order: 0,
                common_label_count: 0,
                relation: NameRelation::Equal,
            },
            phantom: PhantomData,
        }
    }

    pub fn get_absolute_name(&self, child: &LabelSequence) -> Name {
        if self.level_count == 0 {
            child.concat_all(&[]).expect("get absolute name failed")
        } else {
            let mut names = [self.nodes[self.level_count - 1].get_name(); MAX_LABEL_COUNT as usize];
            for i in 1..self.level_count {
                names[i] = self.nodes[self.level_count - i - 1].get_name();
            }
            child
                .concat_all(&names[0..self.level_count])
                .expect("get absolute name failed")
        }
    }

    #[inline]
    pub fn top(&self) -> NodePtr<T> {
        self.nodes[self.level_count - 1]
    }

    #[inline]
    pub fn bottom(&self) -> NodePtr<T> {
        self.nodes[0]
    }

    pub fn push(&mut self, node: NodePtr<T>) {
        assert!(self.level_count < MAX_LABEL_COUNT as usize);
        self.nodes[self.level_count] = node;
        self.level_count += 1;
    }

    pub fn pop(&mut self) -> NodePtr<T> {
        assert!(self.level_count > 0);
        self.level_count -= 1;
        self.nodes[self.level_count]
    }
}

#[cfg(test)]
mod tests {
    use super::{DomainTree, NodeChain, NodePtr};
    use crate::label_sequence::LabelSequence;
    use crate::name::Name;
    use std::str::FromStr;

    #[test]
    fn test_get_absoulte_name() {
        let name_seq = LabelSequence::from_str("a.b.cn.").unwrap();
        let tree = DomainTree::new();
        let mut chain = NodeChain::new(&tree);
        assert_eq!(
            chain.get_absolute_name(&name_seq),
            Name::from_str("a.b.cn").unwrap()
        );

        chain.push(NodePtr::new(
            LabelSequence::from_str("cn.").unwrap(),
            Some("v1"),
        ));
        chain.push(NodePtr::new(
            LabelSequence::from_str("b").unwrap(),
            Some("v1"),
        ));
        assert_eq!(
            chain.get_absolute_name(&LabelSequence::from_str("a").unwrap()),
            Name::from_str("a.b.cn").unwrap()
        );

        chain.pop();
        assert_eq!(
            chain.get_absolute_name(&LabelSequence::from_str("a").unwrap()),
            Name::from_str("a.cn").unwrap()
        );
    }
}
