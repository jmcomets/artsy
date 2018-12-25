extern crate take_mut;

#[cfg(not(feature = "no-simd"))]
#[cfg(target_arch = "x86")]
use std::arch::x86::{
    _mm_cmpeq_epi8,
    _mm_loadu_si128,
    _mm_movemask_epi8,
    _mm_set1_epi8,
};

#[cfg(not(feature = "no-simd"))]
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    _mm_cmpeq_epi8,
    _mm_loadu_si128,
    _mm_movemask_epi8,
    _mm_set1_epi8,
};

use std::mem;

pub struct Trie<T> {
    root: Option<NodeOrLeaf<T>>,
    term: u8,
}

enum NodeOrLeaf<T> {
    Node(Node<T>),
    Leaf(T),
}
use NodeOrLeaf::*;

impl<T> NodeOrLeaf<T> {
    fn as_node(&self) -> Option<&Node<T>> {
        if let Node(ref node) = self {
            Some(node)
        } else {
            None
        }
    }

    fn as_node_mut(&mut self) -> Option<&mut Node<T>> {
        if let Node(ref mut node) = self {
            Some(node)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn as_leaf(&self) -> Option<&T> {
        if let Leaf(ref value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn as_leaf_mut(&mut self) -> Option<&mut T> {
        if let Leaf(ref mut value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn to_node(self) -> Option<Node<T>> {
        if let Node(node) = self {
            Some(node)
        } else {
            None
        }
    }

    fn to_leaf(self) -> Option<T> {
        if let Leaf(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

enum Node<T> {
    Node4 {
        children: [Option<(u8, Box<NodeOrLeaf<T>>)>; 4],
    },
    Node16 {
        child_indices: [u8; 16],
        children: [Option<Box<NodeOrLeaf<T>>>; 16],
        nb_children: u8,
    },
    Node48 {
        child_indices: [u8; 256],
        children: [Option<Box<NodeOrLeaf<T>>>; 48],
        nb_children: u8,
    },
    Node256 {
        children: [Option<Box<NodeOrLeaf<T>>>; 256]
    },
}
use Node::*;

#[derive(Debug)]
pub struct KeyContainsTerminator;

impl<T> Trie<T> {
    pub fn with_terminator(term: u8) -> Trie<T> {
        Trie {
            root: None,
            term: term,
        }
    }

    pub fn for_ascii() -> Trie<T> {
        Self::with_terminator(0)
    }

    pub fn for_utf8() -> Trie<T> {
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
                self.root = Some(Node(node));
                inserted
            }
            Some(Node(ref mut node)) => node.insert(key, value, self.term),
            Some(Leaf(_))            => unreachable!(),
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
            None                 => false,
            Some(Node(ref node)) => node.contains(key, self.term),
            Some(Leaf(_))        => unreachable!(),
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
            None                 => None,
            Some(Node(ref node)) => node.get(key, self.term),
            Some(Leaf(_))        => unreachable!(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

impl<T> Node<T> {
    fn new() -> Self {
        Node4 { children: [None, None, None, None] }
    }

    fn insert(&mut self, key: &[u8], value: T, term: u8) -> Option<T> {
        if key.is_empty() {
            self.insert_child(term, Leaf(value))
                .map(|n| n.to_leaf().unwrap())
        } else {
            self.insert_child(key[0], Node(Node::new()));
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

    fn insert_child(&mut self, key: u8, mut child: NodeOrLeaf<T>) -> Option<NodeOrLeaf<T>> {
        match self {
            Node4 { children } => {
                // 1st step: try to replace existing entry
                for existing_child in children.iter_mut() {
                    if let Some((k, existing_child)) = existing_child {
                        if key == *k {
                            mem::swap(&mut child, existing_child);
                            return Some(child);
                        }
                    }
                }

                // 2nd step: try to add a new entry
                for existing_child in children.iter_mut() {
                    if existing_child.is_none() {
                        *existing_child = Some((key, Box::new(child)));
                        return None;
                    }
                }
            }

            Node16 { child_indices, children, nb_children } => {
                if let Some(index) = node16_find_child_index(child_indices, *nb_children as usize, key) {
                    mem::swap(&mut child, children[index as usize].as_mut().unwrap());
                    return Some(child);
                } else {
                    // If we're adding a new entry, there should be less than 16 entries.
                    if *nb_children < 16 {
                        children[*nb_children as usize] = Some(Box::new(child));
                        *nb_children += 1;
                        return None;
                    }
                }
            }

            Node48 { child_indices, children, nb_children } => {
                let ref mut index = child_indices[key as usize];
                if *index >= 48 {
                    // If we're adding a new entry, there should be less than 48 entries.
                    if *nb_children < 48 {
                        *index = *nb_children;
                        children[*index as usize] = Some(Box::new(child));
                        *nb_children += 1;
                        return None;
                    }
                } else {
                    mem::swap(&mut child, children[*index as usize].as_mut().unwrap());
                    return Some(child);
                }
            }

            Node256 { children } => {
                if let Some(existing_child) = children[key as usize].as_mut() {
                    mem::swap(&mut child, existing_child);
                    return Some(child);
                }

                children[key as usize] = Some(Box::new(child));
                return None;
            }
        }

        // Insert did not succeed? Upgrade and retry.
        take_mut::take(self, Self::upgrade);
        self.insert_child(key, child)
    }

    fn upgrade(self) -> Self {
        match self {
            Node4 { mut children } => {
                let (key_0, child_0) = children[0].take().unwrap();
                let (key_1, child_1) = children[1].take().unwrap();
                let (key_2, child_2) = children[2].take().unwrap();
                let (key_3, child_3) = children[3].take().unwrap();

                Node16 {
                    child_indices: {
                        let mut child_indices = [0; 16];
                        child_indices[0] = key_0;
                        child_indices[1] = key_1;
                        child_indices[2] = key_2;
                        child_indices[3] = key_3;
                        child_indices
                    },
                    children: {
                        let mut children: [Option<Box<NodeOrLeaf<T>>>; 16] = Default::default();
                        children[0] = Some(child_0);
                        children[1] = Some(child_1);
                        children[2] = Some(child_2);
                        children[3] = Some(child_3);
                        children
                    },
                    nb_children: 4,
                }
            }

            Node16 { child_indices: old_child_indices, children: mut old_children, .. } => {
                let mut child_indices = [48; 256];
                let mut children: [Option<Box<NodeOrLeaf<T>>>; 48] = [
                    None, None, None, None, None, None,
                    None, None, None, None, None, None,
                    None, None, None, None, None, None,
                    None, None, None, None, None, None,
                    None, None, None, None, None, None,
                    None, None, None, None, None, None,
                    None, None, None, None, None, None,
                    None, None, None, None, None, None,
                ];

                for (i, j) in old_child_indices.iter().cloned().enumerate() {
                    child_indices[i] = j;
                    mem::swap(&mut children[i], &mut old_children[j as usize]);
                }

                Node48 {
                    child_indices: child_indices,
                    children: children,
                    nb_children: 16,
                }
            }

            Node48 { child_indices, children: mut old_children, .. } => {
                let mut children: [Option<Box<NodeOrLeaf<T>>>; 256] = [
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                ];

                for i in child_indices.iter() {
                    mem::swap(&mut children[*i as usize], &mut old_children[child_indices[*i as usize] as usize]);
                }

                Node256 {
                    children: children,
                }
            }

            Node256 { .. } => {
                unreachable!();
            }
        }
    }

    fn find_child(&self, key: u8) -> Option<&NodeOrLeaf<T>> {
        match self {
            Node4 { children } => {
                for child in children.iter() {
                    if let Some((k, child)) = child {
                        if key == *k {
                            return Some(&child);
                        }
                    }
                }
                None
            }

            Node16 { child_indices, children, nb_children } => {
                if let Some(index) = node16_find_child_index(child_indices, *nb_children as usize, key) {
                    children[index as usize].as_ref().map(|x| &**x)
                } else {
                    None
                }
            }

            Node48 { child_indices, children, .. } => {
                let index = child_indices[key as usize];
                if index < 48 {
                    children[index as usize].as_ref().map(|x| &**x)
                } else {
                    None
                }
            }

            Node256 { children } => {
                children[key as usize].as_ref().map(|x| &**x)
            }
        }
    }

    fn find_child_mut(&mut self, key: u8) -> Option<&mut NodeOrLeaf<T>> {
        unsafe { mem::transmute(self.find_child(key)) }
    }
}

#[cfg(feature = "no-simd")]
fn node16_find_child_index(child_indices: &[u8; 16], nb_children: usize, key: u8) -> Option<usize> {
    for i in 0..nb_children {
        if child_indices[i] == key {
            return Some(i);
        }
    }
    None
}

#[cfg(not(feature = "no-simd"))]
fn node16_find_child_index(child_indices: &[u8; 16], nb_children: usize, key: u8) -> Option<usize> {
    // `key_vec` is 16 repeated copies of the searched-for byte, one for every possible
    // position in `child_indices` that needs to be searched.
    let key_vec = unsafe { _mm_set1_epi8(key as i8) };
    let indices_vec = unsafe { _mm_loadu_si128(child_indices.as_ptr() as *const _) };

    // Compare all `child_indices` in parallel. Don't worry if some of the keys
    // aren't valid, we'll mask the results to only consider the valid ones below.
    let matches = unsafe { _mm_cmpeq_epi8(key_vec, indices_vec) };

    // Apply a mask to select only the first `nb_children` values.
    let mask = (1 << nb_children) - 1;
    let match_bits = unsafe { _mm_movemask_epi8(matches) & mask };

    // The child's index is the first '1' in `match_bits`
    if match_bits != 0 {
        Some(match_bits.leading_zeros() as usize)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    trait TrieTestExtensions<T: Clone + PartialEq + Debug> {
        fn check_insertion(&mut self, key: &[u8], value: T);
    }

    impl<T: Clone + PartialEq + Debug> TrieTestExtensions<T> for Trie<T> {
        fn check_insertion(&mut self, key: &[u8], value: T) {
            self.insert(key, value.clone()).unwrap();
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
    fn it_can_store_less_than_4_parallel_entries() {
        let mut trie = Trie::for_utf8();
        // 1) insert everything
        trie.check_insertion(b"aa", 1);
        trie.check_insertion(b"ab", 2);
        trie.check_insertion(b"ac", 3);
        trie.check_insertion(b"ad", 4);
        trie.check_insertion(b"aaa", 11);
        trie.check_insertion(b"aab", 12);
        trie.check_insertion(b"aac", 13);
        trie.check_insertion(b"aad", 14);
        // 2) check again (FIXME)
        //assert_eq!(trie.get(b"aa").unwrap(), Some(&1));
        //assert_eq!(trie.get(b"ab").unwrap(), Some(&2));
        //assert_eq!(trie.get(b"ac").unwrap(), Some(&3));
        //assert_eq!(trie.get(b"ad").unwrap(), Some(&4));
        //assert_eq!(trie.get(b"aaa").unwrap(), Some(&11));
        //assert_eq!(trie.get(b"aab").unwrap(), Some(&12));
        //assert_eq!(trie.get(b"aac").unwrap(), Some(&13));
        //assert_eq!(trie.get(b"aad").unwrap(), Some(&14));
    }
}
