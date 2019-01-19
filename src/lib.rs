use std::mem;

mod node4;
mod node16;
mod node48;
mod node256;

pub struct Trie<'a, T> {
    root: Option<NodeOrLeaf<'a, T>>,
    term: u8,
}

pub(crate) enum NodeOrLeaf<'a, T: 'a> {
    Node(Node<'a, T>),
    Leaf(T),
}

impl<'a, T> NodeOrLeaf<'a, T> {
    fn as_node(&self) -> Option<&Node<'a, T>> {
        if let NodeOrLeaf::Node(ref node) = self {
            Some(node)
        } else {
            None
        }
    }

    fn as_node_mut(&mut self) -> Option<&mut Node<'a, T>> {
        if let NodeOrLeaf::Node(ref mut node) = self {
            Some(node)
        } else {
            None
        }
    }

    fn as_leaf(&self) -> Option<&T> {
        if let NodeOrLeaf::Leaf(ref value) = self {
            Some(value)
        } else {
            None
        }
    }

    fn to_leaf(self) -> Option<T> {
        if let NodeOrLeaf::Leaf(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

struct Node<'a, T: 'a>(Box<dyn NodeImpl<'a, T> + 'a>);

trait NodeImpl<'a, T> {
    fn insert_child(&mut self, key: u8, child: NodeOrLeaf<'a, T>) -> Result<Option<NodeOrLeaf<'a, T>>, NodeOrLeaf<'a, T>>;

    fn insert_child_if_not_exists(&mut self, key: u8, child: NodeOrLeaf<'a, T>) -> Result<(), NodeOrLeaf<'a, T>>;

    fn find_child(&self, key: u8) -> Option<&NodeOrLeaf<'a, T>>;

    fn upgrade(self: Box<Self>) -> Box<dyn NodeImpl<'a, T> + 'a>;
}

#[derive(Debug)]
pub struct KeyContainsTerminator;

impl<'a, T> Trie<'a, T> {
    pub fn with_terminator(term: u8) -> Trie<'a, T> {
        Trie {
            root: None,
            term: term,
        }
    }

    pub fn for_ascii() -> Trie<'a, T> {
        Self::with_terminator(0)
    }

    pub fn for_utf8() -> Trie<'a, T> {
        Self::with_terminator(0xff)
    }

    pub fn insert(&mut self, key: &[u8], value: T) -> Result<Option<T>, KeyContainsTerminator> {
        if !key.contains(&self.term) {
            Ok(self.insert_impl(key, value))
        } else {
            Err(KeyContainsTerminator)
        }
    }

    pub unsafe fn insert_unchecked(&mut self, key: &[u8], value: T) -> Option<T> {
        self.insert_impl(key, value)
    }

    fn insert_impl(&mut self, key: &[u8], value: T) -> Option<T> {
        match self.root {
            None => {
                let mut node = Node::new();
                let inserted = node.insert(key, value, self.term);
                self.root = Some(NodeOrLeaf::Node(node));
                inserted
            }
            Some(NodeOrLeaf::Node(ref mut node)) => node.insert(key, value, self.term),
            Some(NodeOrLeaf::Leaf(_))            => unreachable!(),
        }
    }

    pub fn contains(&self, key: &[u8]) -> Result<bool, KeyContainsTerminator> {
        if !key.contains(&self.term) {
            Ok(self.contains_impl(key))
        } else {
            Err(KeyContainsTerminator)
        }
    }

    pub unsafe fn contains_unchecked(&self, key: &[u8]) -> bool {
        self.contains_impl(key)
    }

    fn contains_impl(&self, key: &[u8]) -> bool {
        match self.root {
            None                             => false,
            Some(NodeOrLeaf::Node(ref node)) => node.contains(key, self.term),
            Some(NodeOrLeaf::Leaf(_))        => unreachable!(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<&T>, KeyContainsTerminator> {
        if !key.contains(&self.term) {
            Ok(self.get_impl(key))
        } else {
            Err(KeyContainsTerminator)
        }
    }

    pub unsafe fn get_unchecked(&self, key: &[u8]) -> Option<&T> {
        self.get_impl(key)
    }

    fn get_impl(&self, key: &[u8]) -> Option<&T> {
        match self.root {
            None                             => None,
            Some(NodeOrLeaf::Node(ref node)) => node.get(key, self.term),
            Some(NodeOrLeaf::Leaf(_))        => unreachable!(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

impl<'a, T> Node<'a, T> {
    fn new() -> Self {
        Node(Box::new(node4::Node4::new()))
    }

    fn insert(&mut self, key: &[u8], value: T, term: u8) -> Option<T> {
        if key.is_empty() {
            self.insert_child(term, NodeOrLeaf::Leaf(value))
                .map(|n| n.to_leaf().unwrap())
        } else {
            self.insert_child_if_not_exists(key[0], NodeOrLeaf::Node(Node::new()));
            let child = self.find_child_mut(key[0]).unwrap().as_node_mut().unwrap();
            child.insert(&key[1..], value, term)
        }
    }

    fn contains(&self, key: &[u8], term: u8) -> bool {
        self.get(key, term).is_some()
    }

    fn get(&self, key: &[u8], term: u8) -> Option<&T> {
        if key.is_empty() {
            self.find_child(term)
                .map(|n| n.as_leaf().unwrap())
        } else {
            self.find_child(key[0])
                .and_then(|n| n.as_node())
                .and_then(|node| node.get(&key[1..], term))
        }
    }

    fn upgrade(&mut self) {
        take_mut::take(&mut self.0, NodeImpl::upgrade);
    }

    fn insert_child_if_not_exists(&mut self, key: u8, child: NodeOrLeaf<'a, T>) {
        let result = self.0.insert_child_if_not_exists(key, child);
        if let Err(child) = result {
            self.upgrade();
            self.insert_child_if_not_exists(key, child)
        }
    }

    fn insert_child(&mut self, key: u8, child: NodeOrLeaf<'a, T>) -> Option<NodeOrLeaf<'a, T>> {
        let result = self.0.insert_child(key, child);
        match result {
            Ok(replaced_child) => replaced_child,
            Err(child)         => {
                self.upgrade();
                self.insert_child(key, child)
            }
        }
    }

    fn find_child(&self, key: u8) -> Option<&NodeOrLeaf<'a, T>> {
        self.0.find_child(key)
    }

    fn find_child_mut(&mut self, key: u8) -> Option<&mut NodeOrLeaf<'_, T>> {
        unsafe { mem::transmute(self.find_child(key)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    trait TrieTestExtensions<T: Clone + PartialEq + Debug> {
        fn check_insertion(&mut self, key: &[u8], value: T);

        fn check_existence(&mut self, key: &[u8], value: T);
    }

    impl<'a, T: 'a + Clone + PartialEq + Debug> TrieTestExtensions<T> for Trie<'a, T> {
        fn check_insertion(&mut self, key: &[u8], value: T) {
            self.insert(key, value.clone()).unwrap();
            self.check_existence(key, value);
        }

        fn check_existence(&mut self, key: &[u8], value: T) {
            assert_eq!(self.get(key).unwrap(), Some(&value));
        }
    }

    #[test]
    fn it_works() {
        let mut trie = Trie::for_utf8();
        trie.check_insertion(b"the answer", 42);
    }

    #[test]
    fn it_works_for_empty_strings() {
        let mut trie = Trie::for_utf8();
        trie.check_insertion(b"", 1);
    }

    #[test]
    fn it_is_empty_by_default() {
        let trie = Trie::<()>::for_utf8();
        assert!(trie.is_empty());
    }

    #[test]
    fn it_doesnt_overwrite_entries_with_a_common_prefix() {
        let mut trie = Trie::for_utf8();
        trie.insert(b"a", 1).unwrap();
        trie.insert(b"ab", 2).unwrap();
        assert_eq!(trie.get(b"a").unwrap(), Some(&1));
        assert_eq!(trie.get(b"ab").unwrap(), Some(&2));
    }

    #[test]
    fn it_can_store_more_than_4_parallel_entries() {
        let mut trie = Trie::for_utf8();
        // 1) insert
        trie.check_insertion(b"a", 1);
        trie.check_insertion(b"b", 2);
        trie.check_insertion(b"c", 3);
        trie.check_insertion(b"d", 4);
        trie.check_insertion(b"e", 5);
        // 2) verify
        trie.check_existence(b"a", 1);
        trie.check_existence(b"b", 2);
        trie.check_existence(b"c", 3);
        trie.check_existence(b"d", 4);
        trie.check_existence(b"e", 5);
    }

    #[test]
    fn it_can_store_more_than_16_parallel_entries() {
        let mut trie = Trie::for_utf8();
        // 1) insert
        trie.check_insertion(b"a", 1);
        trie.check_insertion(b"c", 2);
        trie.check_insertion(b"d", 3);
        trie.check_insertion(b"e", 4);
        trie.check_insertion(b"f", 5);
        trie.check_insertion(b"g", 6);
        trie.check_insertion(b"h", 7);
        trie.check_insertion(b"i", 8);
        trie.check_insertion(b"j", 9);
        trie.check_insertion(b"k", 10);
        trie.check_insertion(b"l", 11);
        trie.check_insertion(b"m", 12);
        trie.check_insertion(b"n", 13);
        trie.check_insertion(b"o", 14);
        trie.check_insertion(b"p", 15);
        trie.check_insertion(b"q", 16);
        trie.check_insertion(b"r", 17);
        // 2) verify
        trie.check_existence(b"a", 1);
        trie.check_existence(b"c", 2);
        trie.check_existence(b"d", 3);
        trie.check_existence(b"e", 4);
        trie.check_existence(b"f", 5);
        trie.check_existence(b"g", 6);
        trie.check_existence(b"h", 7);
        trie.check_existence(b"i", 8);
        trie.check_existence(b"j", 9);
        trie.check_existence(b"k", 10);
        trie.check_existence(b"l", 11);
        trie.check_existence(b"m", 12);
        trie.check_existence(b"n", 13);
        trie.check_existence(b"o", 14);
        trie.check_existence(b"p", 15);
        trie.check_existence(b"q", 16);
        trie.check_existence(b"r", 17);
    }
}
