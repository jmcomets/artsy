#![feature(test)]


extern crate test;

use test::Bencher;
use artsy::Trie;

use std::collections::HashMap;

struct LexicographicBytesGenerator<'a> {
    source_bytes: &'a [u8],
    current_bytes: Vec<u8>,
    next_byte_index: usize,
}

impl<'a> LexicographicBytesGenerator<'a> {
    fn new(source_bytes: &'a [u8]) -> Self {
        LexicographicBytesGenerator {
            current_bytes: vec![],
            next_byte_index: source_bytes.len(),
            source_bytes: source_bytes,
        }
    }

    fn next_bytes(&mut self) -> &[u8] {
        if self.next_byte_index == self.source_bytes.len() {
            self.next_byte_index = 0;
            self.current_bytes.push(0);
        }

        let index = self.current_bytes.len() - 1;
        self.current_bytes[index] = self.source_bytes[self.next_byte_index];
        self.next_byte_index += 1;

        &self.current_bytes
    }
}

const ASCII_CHARS: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz";

fn ordered_ascii_words(n: usize) -> Vec<(Vec<u8>, usize)> {
    let mut gen = LexicographicBytesGenerator::new(ASCII_CHARS);
    (0..n)
        .map(|_| gen.next_bytes().to_owned())
        .enumerate()
        .map(|(a, b)| (b, a))
        .collect()
}

#[bench]
fn trie_100_ascii_words_insertion(bench: &mut Bencher) {
    let entries = ordered_ascii_words(100);

    bench.iter(|| {
        let mut trie = Trie::for_ascii();
        for (key, value) in entries.iter() {
            trie.insert(key, value).unwrap();
        }
    })
}

#[bench]
fn hashmap_100_ascii_words_insertion(bench: &mut Bencher) {
    let entries = ordered_ascii_words(100);

    bench.iter(|| {
        let mut trie = HashMap::new();
        for (key, value) in entries.iter() {
            trie.insert(key, value);
        }
    })
}

#[bench]
fn trie_100_ascii_words_retrieval(bench: &mut Bencher) {
    let entries = ordered_ascii_words(100);

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
fn hashmap_100_ascii_words_retrieval(bench: &mut Bencher) {
    let entries = ordered_ascii_words(100);

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

// 1000

#[bench]
fn trie_1000_ascii_words_insertion(bench: &mut Bencher) {
    let entries = ordered_ascii_words(1000);

    bench.iter(|| {
        let mut trie = Trie::for_ascii();
        for (key, value) in entries.iter() {
            trie.insert(key, value).unwrap();
        }
    })
}

#[bench]
fn hashmap_1000_ascii_words_insertion(bench: &mut Bencher) {
    let entries = ordered_ascii_words(1000);

    bench.iter(|| {
        let mut trie = HashMap::new();
        for (key, value) in entries.iter() {
            trie.insert(key, value);
        }
    })
}

#[bench]
fn trie_1000_ascii_words_retrieval(bench: &mut Bencher) {
    let entries = ordered_ascii_words(1000);

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
fn hashmap_1000_ascii_words_retrieval(bench: &mut Bencher) {
    let entries = ordered_ascii_words(1000);

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
