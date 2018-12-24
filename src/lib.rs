extern crate take_mut;

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

    // fn as_leaf(&self) -> Option<&T> {
    //     if let Leaf(ref value) = self {
    //         Some(value)
    //     } else {
    //         None
    //     }
    // }

    // fn to_node(self) -> Option<Node<T>> {
    //     if let Node(node) = self {
    //         Some(node)
    //     } else {
    //         None
    //     }
    // }

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
        keys: [u8; 16],
        children: [Option<Box<NodeOrLeaf<T>>>; 16],
        nb_children: u8,
    },
    Node48 {
        keys: [u8; 256],
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
            Some(Node(ref node)) => node.contains(key),
            Some(Leaf(_))        => unreachable!(),
        }
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
            let child = self.insert_child_node(key[0], Node::new());
            child.insert(&key[1..], value, term)
        }
    }

    fn insert_child_node(&mut self, key: u8, child: Node<T>) -> &mut Node<T> {
        self.insert_child(key, Node(child));
        self.find_child_mut(key).unwrap()
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
            Node16 { .. } => {
                unimplemented!();
            }
            Node48 { .. } => {
                unimplemented!();
            }
            Node256 { .. } => {
                unimplemented!();
            }
        }

        // final step: upgrade and retry
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
                    keys: {
                        let mut keys = [0; 16];
                        keys[0] = key_0;
                        keys[1] = key_1;
                        keys[2] = key_2;
                        keys[3] = key_3;
                        keys
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

            Node16 { keys: old_keys, children: mut old_children, .. } => {
                let mut keys = [48; 256];
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

                for (i, key) in old_keys.iter().cloned().enumerate() {
                    keys[i] = key;
                    mem::swap(&mut children[i], &mut old_children[key as usize]);
                }

                Node48 {
                    keys: keys,
                    children: children,
                    nb_children: 16,
                }
            }

            Node48 { keys: keys, children: mut old_children, .. } => {
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

                for key in keys.iter().map(|k| *k as usize) {
                    mem::swap(&mut children[key], &mut old_children[keys[key] as usize]);
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

    fn contains(&self, key: &[u8]) -> bool {
        if key.is_empty() {
            true
        } else {
            self.find_child(key[0])
                .map(|n| n.contains(&key[1..]))
                .unwrap_or(false)
        }
    }

    fn find_child(&self, key: u8) -> Option<&Node<T>> {
        match self {
            Node4 { children } => {
                for child in children.iter() {
                    if let Some((k, child)) = child {
                        if key == *k {
                            return child.as_node();
                        }
                    }
                }
                None
            }
            Node16 { .. } => {
                unimplemented!();
            }
            Node48 { .. } => {
                unimplemented!();
            }
            Node256 { .. } => {
                unimplemented!();
            }
        }
    }

    fn find_child_mut(&mut self, key: u8) -> Option<&mut Node<T>> {
        unsafe { mem::transmute(self.find_child(key)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn it_can_store_less_than_4_parallel_entries() {
        let mut trie = Trie::for_utf8();
        trie.insert(b"aa", 1).unwrap();
        trie.insert(b"ab", 2).unwrap();
        trie.insert(b"ac", 3).unwrap();
        trie.insert(b"ad", 4).unwrap();
        trie.insert(b"aaa", 11).unwrap();
        trie.insert(b"aab", 12).unwrap();
        trie.insert(b"aac", 13).unwrap();
        trie.insert(b"aad", 14).unwrap();
    }

    #[test]
    fn it_can_store_empty_strings() {
        let mut trie = Trie::for_utf8();
        trie.insert(b"", 1).unwrap();
    }
}
