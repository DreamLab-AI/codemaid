# sealmap-corpus

[![crates.io](https://img.shields.io/crates/v/sealmap-corpus.svg)](https://crates.io/crates/sealmap-corpus)
[![docs.rs](https://img.shields.io/docsrs/sealmap-corpus)](https://docs.rs/sealmap-corpus)

Projects a `sealmap_model::Codebase` into a corpus of dense Mermaid diagrams
with exactly one document per source file, plus an overview, a JSON index that
links every call to the fragment that expands it, and the model itself. It
also writes the corpus to disk and checks a directory against it (`verify`),
touching only files it generated.

Part of [sealmap](https://github.com/DreamLab-AI/sealmap). The 0.2 line adds
the seal lockfile, `resolve`, `stale`, `seal-check`, `pack` and a
corpus-wide `verify` here (planned).

```rust
use sealmap_corpus::{CorpusOptions, generate};
use sealmap_model::SourceSet;
use sealmap_rust::{RustOptions, extract};

let mut src = SourceSet::new();
src.insert("src/lib.rs", "pub struct A; impl A { pub fn go(&self) { helper(); } }\nfn helper() {}").unwrap();
let model = extract(&src, &RustOptions { name: "demo".into(), ..Default::default() }).codebase;

let corpus = generate(&model, &CorpusOptions::default());
let doc = corpus.document("src/lib.rs.md").unwrap();
assert!(doc.contains("demo__A->>demo: helper()"));
assert!(corpus.document("_index.json").is_some());
```

## Licence

Licensed under either of [Apache License, Version 2.0](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-APACHE) or [MIT licence](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
