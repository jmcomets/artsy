artsy
=====

[![Travis badge](https://travis-ci.org/jmcomets/artsy.svg?branch=master)](https://travis-ci.org/jmcomets/artsy)
[![crates.io badge](https://img.shields.io/crates/v/artsy.svg)](https://crates.io/crates/artsy)

[Documentation][]

A work-in-progress implementation of an [ART Tree][paper].

## Features

### Terminator Customization

Although the original ART paper only considers ASCII strings as keys, therefore
defines `\0` as string terminator, the idea can be extended to UTF-8 by using
`0xff` as string terminator. Of course, you can use any terminator if you're
inserting raw bytes.

By default, you should use `for_utf8()` if you're using `String` keys.

Terminator customization is why each lookup/insertion/deletion method returns a
`Result<.., KeyContainsTerminator>`; each method is also accompanied by a
equivalent unchecked version.

### Filtering used node types

Although the original ART paper uses 4 different types of nodes (4, 16, 48 and
256 children), you may not want the node types used for sparser tries or the
intermediate node types (for whatever reason). Each node type, other than the
final trie node of 256 children, may be disabled by building *without* its
feature flag enabled:

```bash
cargo build --features "node4" # only use the smallest node, fallback to Node256

cargo build --features "node4 node16" # disable Node48

cargo build --features "node16" # only use the intermediate node with SIMD search
```

### Disabling SIMD for Node16

SIMD should be enabled by default (if you system supports it). To explicitly
disable SIMD, build with the feature `no-simd` set.

## Examples

### Insert / Lookup

```rust
let mut map = Trie::for_utf8();
map.insert(b"a", 0).unwrap();
map.insert(b"ac", 1).unwrap();

assert_eq!(map.get(b"a").unwrap(), Some(&0));
assert_eq!(map.get(b"ac").unwrap(), Some(&1));
assert_eq!(map.get(b"ab").unwrap(), None);
```

## Todo List

- [ ] Refactor insert/update child, removing duplication & extra find after insertion
- [ ] Key/Value/Items `Iterator`
- [ ] Deletion
- [ ] Path Compression

[paper]: https://db.in.tum.de/~leis/papers/ART.pdf

[Documentation]: https://docs.rs/artsy
