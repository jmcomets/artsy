use std::mem;

use super::{
    Child,
    NodeImpl,
};

pub(crate) struct Node256<'a, T> {
    children: [Option<Box<Child<'a, T>>>; 256]
}

impl<'a, T> Default for Node256<'a, T> {
    fn default() -> Self {
        Node256 {
            children: [
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
            ]
        }
    }
}

impl<'a, T> Node256<'a, T> {
    #[cfg(any(feature = "node4", feature = "node16", feature = "node48"))]
    pub fn new(children: [Option<Box<Child<'a, T>>>; 256]) -> Self {
        Node256 { children }
    }
}

impl<'a, T> NodeImpl<'a, T> for Node256<'a, T> {
    fn update_child(&mut self, key: u8, child: Child<'a, T>) -> Result<(), Child<'a, T>> {
        if let Some(_) = self.children[key as usize].as_mut() {
            return Ok(());
        }

        self.children[key as usize] = Some(Box::new(child));
        return Ok(());
    }

    fn insert_child(&mut self, key: u8, mut child: Child<'a, T>) -> Result<Option<Child<'a, T>>, Child<'a, T>> {
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

    fn find_child(&self, key: u8) -> Option<&Child<'a, T>> {
        self.children[key as usize].as_ref().map(|x| &**x)
    }
}
