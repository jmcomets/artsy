use std::mem;

use super::{
    Child,
    NodeImpl,
};

use crate::node256::Node256;

pub(crate) struct Node48<'a, T> {
    child_indices: [u8; 256],
    children: [Option<Box<Child<'a, T>>>; 48],
    nb_children: u8,
}

impl<'a, T> Node48<'a, T> {
    #[cfg(any(feature = "node4", feature = "node16"))]
    pub fn new(child_indices: [u8; 256], children: [Option<Box<Child<'a, T>>>; 48], nb_children: u8) -> Self {
        Node48 { child_indices, children, nb_children }
    }

    fn upgrade_to_node256(mut self: Box<Self>) -> Box<Node256<'a, T>> {
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

        for i in 0..self.child_indices.len() {
            let key = self.child_indices[i] as usize;
            mem::swap(&mut children[key], &mut self.children[self.child_indices[key] as usize]);
        }

        Box::new(Node256::new(children))
    }
}

impl<'a, T> Default for Node48<'a, T> {
    fn default() -> Self {
        Node48 {
            child_indices: [48; 256],
            children: [
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
            ],
            nb_children: 0,
        }
    }
}

impl<'a, T> NodeImpl<'a, T> for Node48<'a, T> {
    fn insert_child_if_not_exists(&mut self, key: u8, child: Child<'a, T>) -> Result<(), Child<'a, T>> {
        let ref mut index = self.child_indices[key as usize];
        if *index >= 48 {
            // If we're adding a new entry, there should be less than 48 entries.
                if self.nb_children < 48 {
                    *index = self.nb_children;
                    self.children[*index as usize] = Some(Box::new(child));
                    self.nb_children += 1;
                    return Ok(());
                }
            } else {
                return Ok(());
            }

            Err(child)
        }

        fn insert_child(&mut self, key: u8, mut child: Child<'a, T>) -> Result<Option<Child<'a, T>>, Child<'a, T>> {
            let ref mut index = self.child_indices[key as usize];
            if *index >= 48 {
                // If we're adding a new entry, there should be less than 48 entries.
                if self.nb_children < 48 {
                    *index = self.nb_children;
                    self.children[*index as usize] = Some(Box::new(child));
                    self.nb_children += 1;
                return Ok(None);
            }
        } else {
            mem::swap(&mut child, self.children[*index as usize].as_mut().unwrap());
            return Ok(Some(child));
        }

        Err(child)
    }

    fn upgrade(self: Box<Self>) -> Box<dyn NodeImpl<'a, T> + 'a> {
        self.upgrade_to_node256()
    }

    fn find_child(&self, key: u8) -> Option<&Child<'a, T>> {
        let index = self.child_indices[key as usize];
        if index < 48 {
            self.children[index as usize].as_ref().map(|x| &**x)
        } else {
            None
        }
    }
}
