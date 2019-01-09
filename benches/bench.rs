#![feature(test)]

extern crate artsy;
extern crate test;

use test::Bencher;
use artsy::Trie;

use std::collections::HashMap;

const ASCII_CHARS: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz";

struct BytesGenerator {
    current: Vec<u8>,
    next_ascii_char: usize,
}

impl BytesGenerator {
    fn new() -> Self {
        BytesGenerator {
            current: vec![],
            next_ascii_char: ASCII_CHARS.len(),
        }
    }

    fn next_bytes(&mut self) -> &[u8] {
        if self.next_ascii_char == ASCII_CHARS.len() {
            self.next_ascii_char = 0;
            self.current.push(0);
        }

        let index = self.current.len() - 1;
        self.current[index] = ASCII_CHARS[self.next_ascii_char];
        self.next_ascii_char += 1;

        &self.current
    }
}

fn insertion_data(n: usize) -> Vec<(Vec<u8>, usize)> {
    let mut gen = BytesGenerator::new();
    (0..n)
        .map(|_| gen.next_bytes().to_owned())
        .enumerate()
        .map(|(a, b)| (b, a))
        .collect()
}

#[bench]
fn trie_insertion(bench: &mut Bencher) {
    let entries = insertion_data(1000);

    bench.iter(|| {
        let mut trie = Trie::for_ascii();
        for (key, value) in entries.iter() {
            trie.insert(key, value).unwrap();
        }
    })
}

#[bench]
fn hashmap_insertion(bench: &mut Bencher) {
    let entries = insertion_data(1000);

    bench.iter(|| {
        let mut trie = HashMap::new();
        for (key, value) in entries.iter() {
            trie.insert(key, value);
        }
    })
}

#[bench]
fn trie_retrieval(bench: &mut Bencher) {
    let entries = insertion_data(1000);

    let mut trie = Trie::for_ascii();
    for (key, value) in entries.iter() {
        trie.insert(key, value).unwrap();
    }

    bench.iter(|| {
        for (key, value) in entries.iter() {
            assert_eq!(trie.get(key).unwrap(), Some(&value));
        }
    })
}

#[bench]
fn hashmap_retrieval(bench: &mut Bencher) {
    let entries = insertion_data(1000);

    let mut map = HashMap::new();
    for (key, value) in entries.iter() {
        map.insert(key, value);
    }

    bench.iter(|| {
        for (key, value) in entries.iter() {
            assert_eq!(map.get(key), Some(&value));
        }
    })
}
