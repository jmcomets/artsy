

use artsy::Trie;
use std::env;

struct BytesGenerator<'a> {
    alphabet: &'a [u8],
    next_index: usize,
    current: Vec<u8>,
}

impl<'a> BytesGenerator<'a> {
    fn new(alphabet: &'a [u8]) -> Self {
        BytesGenerator {
            alphabet: alphabet,
            next_index: alphabet.len(),
            current: vec![],
        }
    }

    fn next_bytes(&mut self) -> &[u8] {
        if self.next_index == self.alphabet.len() {
            self.next_index = 0;
            self.current.push(0);
        }

        let index = self.current.len() - 1;
        self.current[index] = self.alphabet[self.next_index];
        self.next_index += 1;

        &self.current
    }
}

fn insertion_data(alphabet: &[u8], n: usize) -> Vec<(Vec<u8>, usize)> {
    let mut gen = BytesGenerator::new(alphabet);
    (0..n)
        .map(|_| gen.next_bytes().to_owned())
        .enumerate()
        .map(|(a, b)| (b, a))
        .collect()
}

fn main() {
    const THREE_CHARS: &'static [u8] = b"abc";
    const TEN_CHARS: &'static [u8] = b"abcdefghij";
    const ASCII_LOWER_CHARS: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz";
    const ASCII_LOWER_AND_UPPER_CHARS: &'static [u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let nb_entries: usize = env::args().nth(1).unwrap().parse().unwrap();
    let entries = insertion_data(ASCII_LOWER_AND_UPPER_CHARS, nb_entries);

    let mut trie = Trie::for_ascii();
    for (key, value) in entries.iter() {
        trie.insert(key, value).unwrap();
    }
}
