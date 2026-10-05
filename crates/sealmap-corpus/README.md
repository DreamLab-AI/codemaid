# sealmap-corpus

[![crates.io](https://img.shields.io/crates/v/sealmap-corpus.svg)](https://crates.io/crates/sealmap-corpus)
[![docs.rs](https://img.shields.io/docsrs/sealmap-corpus)](https://docs.rs/sealmap-corpus)

Projects a `sealmap_model::Codebase` into a corpus of dense Mermaid diagrams
with exactly one document per source file, plus an overview, a JSON index that
links every call to the fragment that expands it, and the model itself. It
also writes the corpus to disk and checks a directory against it, touching
only files it generated.

Its `seal` module keeps hand-written diagram topics true to the code. A
topic cites symbols by `sym:` id; `seals.lock` records the `sig_hash` and
`body_hash` each had when the topic was reviewed, plus a hash of the topic's
prose. `seal::verify` classifies every seal as holds, behaviour, contract,
absent (with rename candidates), unparsable (fail closed), unsealed
citation, orphan, prose edited or lock fault; `seal::stale` compares two
models; `seal::sign` writes a seal. All of it is pure: no file system, no
git.

Part of [sealmap](https://github.com/DreamLab-AI/sealmap). Review packs
(`pack`) are planned.

```rust
use sealmap_corpus::{CorpusOptions, generate};
use sealmap_model::SourceSet;
use sealmap_rust::{RustOptions, extract};

let mut src = SourceSet::new();
src.insert("src/lib.rs", "pub struct A; impl A { pub fn go(&self) { helper(); } }\nfn helper() {}").unwrap();
let model = extract(&src, &RustOptions { name: "demo".into(), ..Default::default() }).codebase;

let corpus = generate(&model, &CorpusOptions::default());
let doc = corpus.document("src/lib.rs.md").unwrap();
assert!(doc.contains("demo___tA->>demo: helper()"));
assert!(corpus.document("_index.json").is_some());
```

## Licence

Licensed under either of [Apache License, Version 2.0](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-APACHE) or [MIT licence](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
