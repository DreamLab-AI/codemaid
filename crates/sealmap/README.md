# sealmap

[![crates.io](https://img.shields.io/crates/v/sealmap.svg)](https://crates.io/crates/sealmap)
[![docs.rs](https://img.shields.io/docsrs/sealmap)](https://docs.rs/sealmap)

Deterministic code maps for LLM development harnesses: the facade and
command-line tool of [sealmap](https://github.com/DreamLab-AI/sealmap).

sealmap reads source without compiling it, builds a language-neutral model
(symbols, relations, the ordered call flow of every function) and projects it
into Mermaid diagrams with one document per source file. The same sources
always give byte-identical output. Every symbol has a stable `sym:` id (no
file path) and two fingerprints: `sig_hash` for its contract, `body_hash` for
its implementation, both blind to formatting, comments and moves. Sealed
diagram contracts checked in CI are planned for 0.2 (see the
[design](https://github.com/DreamLab-AI/sealmap/blob/main/docs/DESIGN.md)).

| Crate | Role |
|---|---|
| [`sealmap-model`](https://crates.io/crates/sealmap-model) | language-neutral code model |
| [`sealmap-mermaid`](https://crates.io/crates/sealmap-mermaid) | typed Mermaid writers and injective diagram ids (no dependencies without the default `model` feature) |
| [`sealmap-extract`](https://crates.io/crates/sealmap-extract) | logic shared by every language adapter (replaces `sealmap-frontend` 0.1.0) |
| [`sealmap-rust`](https://crates.io/crates/sealmap-rust) | Rust language adapter (syn) |
| [`sealmap-corpus`](https://crates.io/crates/sealmap-corpus) | projections, index, write and verify |
| [`sealmap-dense`](https://crates.io/crates/sealmap-dense) | compact agent projection: skeletons, call trees, short-name index, budgeted slices (`sealmap::dense`) |

## Command line

```sh
cargo install sealmap

sealmap generate .                  # write the corpus, model and index to .sealmap/
sealmap verify   .                  # exit 1 if .sealmap/ no longer matches the sources
sealmap model    .  > model.json    # just the model
sealmap generate --repo api=../api --repo core=../core -o .sealmap   # several repositories as one
```

## Library

```rust,no_run
use std::path::Path;

let corpus = sealmap::generate_dir(Path::new("."), &sealmap::Options::default())?;
let report = sealmap::corpus::write(Path::new(".sealmap"), &corpus)?;
println!("{} files updated", report.entries.len());
# Ok::<(), std::io::Error>(())
```

Ids and fingerprints, on sources held in memory:

```rust
use sealmap::model::{Fingerprint, SymbolId};
use sealmap::{SourceSet, rust};

let mut src = SourceSet::new();
src.insert("src/lib.rs", "pub struct Db;\nimpl Db { pub fn get(&self) -> u8 { 1 } }").unwrap();
let model = rust::extract(&src, &rust::RustOptions { name: "shop".into(), ..Default::default() }).codebase;

let id = SymbolId::parse("sym:cargo shop . Db#get().").unwrap();
let get = model.symbol(&id).unwrap();
assert!(get.sig_hash.to_string().starts_with(Fingerprint::PREFIX));

// Reformatting and comments change neither fingerprint.
let mut again = SourceSet::new();
again.insert("src/lib.rs", "pub struct Db;\n\n// note\nimpl Db {\n    pub fn get(&self) -> u8 {\n        1\n    }\n}\n").unwrap();
let model2 = rust::extract(&again, &rust::RustOptions { name: "shop".into(), ..Default::default() }).codebase;
let get2 = model2.symbol(&id).unwrap();
assert_eq!((get.sig_hash, get.body_hash), (get2.sig_hash, get2.body_hash));
```

## Licence

Licensed under either of [Apache License, Version 2.0](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-APACHE) or [MIT licence](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
