use std::mem;

use super::{
    Child,
    NodeImpl,
};

#[cfg(feature = "node16")]
use crate::node16::Node16;

#[cfg(all(not(feature = "node16"), feature = "node48"))]
use crate::node48::Node48;

#[cfg(not(any(feature = "node16", feature = "node48")))]
use crate::node256::Node256;

pub struct Node4<'a, T> {
    children: [Option<(u8, Box<Child<'a, T>>)>; 4],
}

impl<'a, T> Default for Node4<'a, T> {
    fn default() -> Self {
        Node4 { children: [None, None, None, None] }
    }
}

impl<'a, T> Node4<'a, T> {
    #[cfg(feature = "node16")]
    fn upgrade_to_node16(mut self: Box<Self>) -> Box<Node16<'a, T>> {
        let (key_0, child_0) = self.children[0].take().unwrap();
        let (key_1, child_1) = self.children[1].take().unwrap();
        let (key_2, child_2) = self.children[2].take().unwrap();
        let (key_3, child_3) = self.children[3].take().unwrap();

        let child_indices = {
            let mut child_indices = [0; 16];
            child_indices[0] = key_0;
            child_indices[1] = key_1;
            child_indices[2] = key_2;
            child_indices[3] = key_3;
            child_indices
        };

        let children = {
            let mut children: [Option<Box<Child<'a, T>>>; 16] = Default::default();
            children[0] = Some(child_0);
            children[1] = Some(child_1);
            children[2] = Some(child_2);
            children[3] = Some(child_3);
            children
        };

        Box::new(Node16::new(child_indices, children, 4))
    }

    #[cfg(all(not(feature = "node16"), feature = "node48"))]
    fn upgrade_to_node48(mut self: Box<Self>) -> Box<Node48<'a, T>> {
        let (key_0, child_0) = self.children[0].take().unwrap();
        let (key_1, child_1) = self.children[1].take().unwrap();
        let (key_2, child_2) = self.children[2].take().unwrap();
        let (key_3, child_3) = self.children[3].take().unwrap();

        let mut child_indices = [48; 256];
        let mut children: [Option<Box<Child<'a, T>>>; 48] = [
            None, None, None, None, None, None,
            None, None, None, None, None, None,
            None, None, None, None, None, None,
            None, None, None, None, None, None,
            None, None, None, None, None, None,
            None, None, None, None, None, None,
            None, None, None, None, None, None,
            None, None, None, None, None, None,
        ];

        child_indices[key_0 as usize] = 0; children[0] = Some(child_0);
        child_indices[key_1 as usize] = 1; children[1] = Some(child_1);
        child_indices[key_2 as usize] = 2; children[2] = Some(child_2);
        child_indices[key_3 as usize] = 3; children[3] = Some(child_3);

        Box::new(Node48::new(child_indices, children, 16))
    }

    #[cfg(not(any(feature = "node16", feature = "node48")))]
    fn upgrade_to_node256(mut self: Box<Self>) -> Box<Node256<'a, T>> {
        let (key_0, child_0) = self.children[0].take().unwrap();
        let (key_1, child_1) = self.children[1].take().unwrap();
        let (key_2, child_2) = self.children[2].take().unwrap();
        let (key_3, child_3) = self.children[3].take().unwrap();

        let mut children: [Option<Box<Child<'a, T>>>; 256] = [
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

        children[key_0 as usize] = Some(child_0);
        children[key_1 as usize] = Some(child_1);
        children[key_2 as usize] = Some(child_2);
        children[key_3 as usize] = Some(child_3);

        Box::new(Node256::new(children))
    }
}

impl<'a, T> NodeImpl<'a, T> for Node4<'a, T> {
    fn insert_child(&mut self, key: u8, mut child: Child<'a, T>) -> Result<Option<Child<'a, T>>, Child<'a, T>> {
        // 1st step: try to replace existing entry
        for existing_child in self.children.iter_mut() {
            if let Some((k, existing_child)) = existing_child {
                if key == *k {
                    mem::swap(&mut child, existing_child);
                    return Ok(Some(child));
                }
            }
        }

        // 2nd step: try to add a new entry
        for existing_child in self.children.iter_mut() {
            if existing_child.is_none() {
                *existing_child = Some((key, Box::new(child)));
                return Ok(None);
            }
        }

        Err(child)
    }

    fn update_child(&mut self, key: u8, child: Child<'a, T>) -> Result<(), Child<'a, T>> {
        // 1st step: try to replace existing entry
        for existing_child in self.children.iter_mut() {
            if let Some((k, _)) = existing_child {
                if key == *k {
                    return Ok(());
                }
            }
        }

        // 2nd step: try to add a new entry
        for existing_child in self.children.iter_mut() {
            if existing_child.is_none() {
                *existing_child = Some((key, Box::new(child)));
                return Ok(());
            }
        }

        Err(child)
    }

    fn upgrade(self: Box<Self>) -> Box<dyn NodeImpl<'a, T> + 'a> {
        #[cfg(feature = "node16")] {
            self.upgrade_to_node16()
        }

        #[cfg(all(not(feature = "node16"), feature = "node48"))] {
            self.upgrade_to_node48()
        }

        #[cfg(not(any(feature = "node16", feature = "node48")))] {
            self.upgrade_to_node256()
        }
    }

    fn find_child(&self, key: u8) -> Option<&Child<'a, T>> {
        for child in self.children.iter() {
            if let Some((k, child)) = child {
                if key == *k {
                    return Some(&child);
                }
            }
        }
        None
    }
}
