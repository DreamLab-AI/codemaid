# sealmap-rust

[![crates.io](https://img.shields.io/crates/v/sealmap-rust.svg)](https://crates.io/crates/sealmap-rust)
[![docs.rs](https://img.shields.io/docsrs/sealmap-rust)](https://docs.rs/sealmap-rust)

A pure-Rust language adapter for [sealmap](https://github.com/DreamLab-AI/sealmap): it
turns Rust source into a `sealmap_model::Codebase` of modules, types, traits,
functions and methods, the relations between them, and an ordered call flow
for every function body.

- **No toolchain at runtime.** It parses with `syn` and never runs `rustc` or
  `cargo`, so it works on any checkout, including code that does not compile.
- **Workspace-wide resolution** through `use` imports (renames, globs,
  `pub use` re-exports), `crate`/`self`/`super`/`Self`, sibling crates and
  receiver types.
- **Honest confidence.** Every call is `exact`, `inferred` or `external`.
- **`sym:` ids and fingerprints.** Every symbol gets a kind-explicit id
  (`sym:cargo shop . db/Db#insert().`) plus `sig_hash` and `body_hash`,
  which ignore comments, whitespace, rustfmt rewrites and moves.
- **Hardened** for large trees: `.gitignore`-aware loading, big-stack
  collection with a per-file panic guard, linear glob resolution.
  VisionClaw (934 files) extracts and renders in about 1 s.
- **Deterministic**, with or without the `parallel` feature (on by default).

```rust
use sealmap_model::{SourceSet, SymbolId};
use sealmap_rust::{RustOptions, extract};

let mut src = SourceSet::new();
src.insert("Cargo.toml", "[package]\nname = \"shop\"").unwrap();
src.insert("src/lib.rs", r#"
    pub struct Db;
    impl Db { pub fn insert(&self, id: u64) {} }
    pub struct Orders { db: Db }
    impl Orders { pub fn place(&self, id: u64) { self.db.insert(id); } }
"#).unwrap();

let out = extract(&src, &RustOptions::default());
let place = out.codebase.symbol(&SymbolId::parse("sym:cargo shop . Orders#place().").unwrap()).unwrap();
let calls: Vec<_> = place.flow.as_ref().unwrap().calls().map(|c| c.target.to_string()).collect();
assert_eq!(calls, ["sym:cargo shop . Db#insert()."]);
```

## Licence

Licensed under either of [Apache License, Version 2.0](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-APACHE) or [MIT licence](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
