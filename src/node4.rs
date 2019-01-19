use std::mem;

use super::{
    NodeOrLeaf,
    NodeImpl,
};

use crate::node16::Node16;

pub struct Node4<'a, T> {
    children: [Option<(u8, Box<NodeOrLeaf<'a, T>>)>; 4],
}

impl<'a, T> Node4<'a, T> {
    pub fn new() -> Self {
        Node4 { children: [None, None, None, None] }
    }
}

impl<'a, T> NodeImpl<'a, T> for Node4<'a, T> {
    fn insert_child(&mut self, key: u8, mut child: NodeOrLeaf<'a, T>) -> Result<Option<NodeOrLeaf<'a, T>>, NodeOrLeaf<'a, T>> {
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

    fn insert_child_if_not_exists(&mut self, key: u8, child: NodeOrLeaf<'a, T>) -> Result<(), NodeOrLeaf<'a, T>> {
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

    fn upgrade(mut self: Box<Self>) -> Box<dyn NodeImpl<'a, T> + 'a> {
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
            let mut children: [Option<Box<NodeOrLeaf<'a, T>>>; 16] = Default::default();
            children[0] = Some(child_0);
            children[1] = Some(child_1);
            children[2] = Some(child_2);
            children[3] = Some(child_3);
            children
        };

        Box::new(Node16::new(child_indices, children, 4))
    }

    fn find_child(&self, key: u8) -> Option<&NodeOrLeaf<'a, T>> {
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
