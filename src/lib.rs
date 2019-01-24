use std::mem;

trait NodeImpl<'a, T> {
    fn insert_child(&mut self, key: u8, child: Child<'a, T>) -> Result<Option<Child<'a, T>>, Child<'a, T>>;

    fn insert_child_if_not_exists(&mut self, key: u8, child: Child<'a, T>) -> Result<(), Child<'a, T>>;

    fn find_child(&self, key: u8) -> Option<&Child<'a, T>>;

    fn upgrade(self: Box<Self>) -> Box<dyn NodeImpl<'a, T> + 'a>;
}

#[cfg(feature = "node4")]
mod node4;

#[cfg(feature = "node4")]
use self::node4::Node4 as DefaultNode;

#[cfg(feature = "node16")]
mod node16;

#[cfg(all(not(feature = "node4"), feature = "node16"))]
use self::node16::Node16 as DefaultNode;

#[cfg(feature = "node48")]
mod node48;

#[cfg(all(not(feature = "node4"), not(feature = "node16"), feature = "node48"))]
use self::node48::Node48 as DefaultNode;

// always included
mod node256;

#[cfg(all(not(feature = "node4"), not(feature = "node16"), not(feature = "node48")))]
use self::node256::Node256 as DefaultNode;

pub struct Trie<'a, T> {
    root: Option<Child<'a, T>>,
    term: u8,
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
                self.root = Some(Child::Node(node));
                inserted
            }
            Some(Child::Node(ref mut node)) => node.insert(key, value, self.term),
            Some(Child::Leaf(_))            => unreachable!(),
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
            Some(Child::Node(ref node)) => node.contains(key, self.term),
            Some(Child::Leaf(_))        => unreachable!(),
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
            Some(Child::Node(ref node)) => node.get(key, self.term),
            Some(Child::Leaf(_))        => unreachable!(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

struct Node<'a, T: 'a>(Box<dyn NodeImpl<'a, T> + 'a>);

impl<'a, T> Node<'a, T> {
    fn new() -> Self {
        Node(Box::new(DefaultNode::default()))
    }

    fn insert(&mut self, key: &[u8], value: T, term: u8) -> Option<T> {
        if key.is_empty() {
            self.insert_child(term, Child::Leaf(value))
                .map(|n| n.to_leaf().unwrap())
        } else {
            self.insert_child_if_not_exists(key[0], Child::Node(Node::new()));
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

    fn insert_child(&mut self, key: u8, child: Child<'a, T>) -> Option<Child<'a, T>> {
        let result = self.0.insert_child(key, child);
        match result {
            Ok(replaced_child) => replaced_child,
            Err(child)         => {
                self.upgrade();
                self.insert_child(key, child)
            }
        }
    }

    fn insert_child_if_not_exists(&mut self, key: u8, child: Child<'a, T>) {
        let result = self.0.insert_child_if_not_exists(key, child);
        if let Err(child) = result {
            self.upgrade();
            self.insert_child_if_not_exists(key, child)
        }
    }

    fn find_child(&self, key: u8) -> Option<&Child<'a, T>> {
        self.0.find_child(key)
    }

    fn upgrade(&mut self) {
        take_mut::take(&mut self.0, NodeImpl::upgrade);
    }

    fn find_child_mut(&mut self, key: u8) -> Option<&mut Child<'_, T>> {
        unsafe { mem::transmute(self.find_child(key)) }
    }
}

pub(crate) enum Child<'a, T: 'a> {
    Node(Node<'a, T>),
    Leaf(T),
}

impl<'a, T> Child<'a, T> {
    fn as_node(&self) -> Option<&Node<'a, T>> {
        if let Child::Node(ref node) = self {
            Some(node)
        } else {
            None
        }
    }

    fn as_node_mut(&mut self) -> Option<&mut Node<'a, T>> {
        if let Child::Node(ref mut node) = self {
            Some(node)
        } else {
            None
        }
    }

    fn as_leaf(&self) -> Option<&T> {
        if let Child::Leaf(ref value) = self {
            Some(value)
        } else {
            None
        }
    }

    fn to_leaf(self) -> Option<T> {
        if let Child::Leaf(value) = self {
            Some(value)
        } else {
            None
        }
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
