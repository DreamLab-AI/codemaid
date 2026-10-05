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

Its `pack` module builds review packs: one deterministic text holding
chosen topics verbatim, the `sealmap-dense` slice of the code each cites and
a bounded window of that code's source, refused (never truncated) when over
a byte budget, or split into shards of whole topics. `pack::changed_topics`
picks the topics whose sealed or cited symbols changed between two models.

Part of [sealmap](https://github.com/DreamLab-AI/sealmap).

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

A review pack of one topic, refused when over budget:

```rust
use sealmap_corpus::pack::{PackError, PackInput, PackOptions, pack};
use sealmap_corpus::seal::Topics;
use sealmap_model::{SourcePath, SourceSet};
use sealmap_rust::{RustOptions, extract};

let mut src = SourceSet::new();
src.insert("src/lib.rs", "pub struct Ledger;\nimpl Ledger {\n    pub fn post(&self) {}\n}\n").unwrap();
let model = extract(&src, &RustOptions { name: "shop".into(), ..Default::default() }).codebase;
let mut topics = Topics::new();
topics.insert(
    SourcePath::new("ledger/01-posting.md").unwrap(),
    "---\nid: LED-01\n---\nPosting goes through `sym:cargo shop . Ledger#post().`.\n".into(),
);

let input = PackInput::new(&model, &src, &topics, "4f1a9de");
let ids = ["LED-01".to_string()];
let out = pack(&input, &ids, &PackOptions::default()).unwrap();
assert!(out.text.contains("==== dense LED-01 depth 1 "));
assert!(matches!(
    pack(&input, &ids, &PackOptions::default().budget(100)),
    Err(PackError::OverBudget { budget: 100, .. })
));
```

## Licence

Licensed under either of [Apache License, Version 2.0](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-APACHE) or [MIT licence](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
