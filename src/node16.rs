use std::mem;

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

use super::{
    Child,
    NodeImpl,
};

#[cfg(feature = "node48")]
use crate::node48::Node48;

#[cfg(not(feature = "node48"))]
use crate::node256::Node256;

pub(crate) struct Node16<'a, T> {
    child_indices: [u8; 16],
    children: [Option<Box<Child<'a, T>>>; 16],
    nb_children: u8,
}

impl<'a, T> Default for Node16<'a, T> {
    fn default() -> Self {
        Node16 {
            child_indices: [0; 16],
            children: [
                None, None, None, None,
                None, None, None, None,
                None, None, None, None,
                None, None, None, None,
            ],
            nb_children: 0
        }
    }
}

impl<'a, T> Node16<'a, T> {
    #[cfg(feature = "node4")]
    pub fn new(child_indices: [u8; 16], children: [Option<Box<Child<'a, T>>>; 16], nb_children: u8) -> Self {
        Node16 { child_indices, children, nb_children }
    }

    #[cfg(feature = "node48")]
    fn upgrade_to_node48(mut self: Box<Self>) -> Box<Node48<'a, T>> {
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

        for i in 0..self.child_indices.len() {
            child_indices[self.child_indices[i] as usize] = i as u8;
            mem::swap(&mut self.children[i], &mut children[i]);
        }

        Box::new(Node48::new(child_indices, children, 16))
    }

    #[cfg(not(feature = "node48"))]
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
            mem::swap(&mut children[key], &mut self.children[i]);
        }

        Box::new(Node256::new(children))
    }


    #[cfg(any(feature = "no-simd", all(not(target_arch = "x86"), not(target_arch = "x86_64"))))]
    fn find_child_index(&self, key: u8) -> Option<usize> {
        for i in 0..self.nb_children {
            if self.child_indices[i] == key {
                return Some(i);
            }
        }
        None
    }

    #[cfg(all(not(feature = "no-simd"), any(target_arch = "x86", target_arch = "x86_64")))]
    fn find_child_index(&self, key: u8) -> Option<usize> {
        // `key_vec` is 16 repeated copies of the searched-for byte, one for every possible
        // position in `child_indices` that needs to be searched.
        let key_vec = unsafe { _mm_set1_epi8(key as i8) };
        let indices_vec = unsafe { _mm_loadu_si128(self.child_indices.as_ptr() as *const _) };

        // Compare all `child_indices` in parallel. Don't worry if some of the keys
        // aren't valid, we'll mask the results to only consider the valid ones below.
        let matches = unsafe { _mm_cmpeq_epi8(key_vec, indices_vec) };

        // Apply a mask to select only the first `nb_children` values.
        let mask = (1 << self.nb_children) - 1;
        let match_bits = unsafe { _mm_movemask_epi8(matches) & mask };

        // The child's index is the first '1' in `match_bits`
        if match_bits != 0 {
            Some(match_bits.trailing_zeros() as usize)
        } else {
            None
        }
    }
}

impl<'a, T> NodeImpl<'a, T> for Node16<'a, T> {
    fn update_child(&mut self, key: u8, child: Child<'a, T>) -> Result<(), Child<'a, T>> {
        if let Some(_) = self.find_child_index(key) {
            return Ok(());
        } else {
            // If we're adding a new entry, there should be less than 16 entries.
            if self.nb_children < 16 {
                self.child_indices[self.nb_children as usize] = key;
                self.children[self.nb_children as usize] = Some(Box::new(child));
                self.nb_children += 1;
                return Ok(());
            }
        }

        Err(child)
    }

    fn insert_child(&mut self, key: u8, mut child: Child<'a, T>) -> Result<Option<Child<'a, T>>, Child<'a, T>> {
        if let Some(index) = self.find_child_index(key) {
            mem::swap(&mut child, self.children[index as usize].as_mut().unwrap());
            return Ok(Some(child));
        } else {
            // If we're adding a new entry, there should be less than 16 entries.
            if self.nb_children < 16 {
                self.child_indices[self.nb_children as usize] = key;
                self.children[self.nb_children as usize] = Some(Box::new(child));
                self.nb_children += 1;
                return Ok(None);
            }
        }

        Err(child)
    }

    fn upgrade(self: Box<Self>) -> Box<dyn NodeImpl<'a, T> + 'a> {
        #[cfg(feature = "node48")] {
            self.upgrade_to_node48()
        }

        #[cfg(not(feature = "node48"))] {
            self.upgrade_to_node256()
        }
    }

    fn find_child(&self, key: u8) -> Option<&Child<'a, T>> {
        if let Some(index) = self.find_child_index(key) {
            self.children[index as usize].as_ref().map(|x| &**x)
        } else {
            None
        }
    }
}
