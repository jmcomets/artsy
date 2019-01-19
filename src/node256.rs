use std::mem;

use super::{
    NodeOrLeaf,
    NodeImpl,
};

pub(crate) struct Node256<'a, T> {
    children: [Option<Box<NodeOrLeaf<'a, T>>>; 256]
}

impl<'a, T> Node256<'a, T> {
    pub fn new(children: [Option<Box<NodeOrLeaf<'a, T>>>; 256]) -> Self {
        Node256 { children }
    }
}

impl<'a, T> NodeImpl<'a, T> for Node256<'a, T> {
    fn insert_child_if_not_exists(&mut self, key: u8, child: NodeOrLeaf<'a, T>) -> Result<(), NodeOrLeaf<'a, T>> {
        if let Some(_) = self.children[key as usize].as_mut() {
            return Ok(());
        }

        self.children[key as usize] = Some(Box::new(child));
        return Ok(());
    }

    fn insert_child(&mut self, key: u8, mut child: NodeOrLeaf<'a, T>) -> Result<Option<NodeOrLeaf<'a, T>>, NodeOrLeaf<'a, T>> {
        if let Some(existing_child) = self.children[key as usize].as_mut() {
            mem::swap(&mut child, existing_child);
            return Ok(Some(child));
        }

        self.children[key as usize] = Some(Box::new(child));
        Ok(None)
    }

    fn upgrade(self: Box<Self>) -> Box<dyn NodeImpl<'a, T> + 'a> {
        unreachable!();
    }

    fn find_child(&self, key: u8) -> Option<&NodeOrLeaf<'a, T>> {
        self.children[key as usize].as_ref().map(|x| &**x)
    }
}
