# sealmap-model

[![crates.io](https://img.shields.io/crates/v/sealmap-model.svg)](https://crates.io/crates/sealmap-model)
[![docs.rs](https://img.shields.io/docsrs/sealmap-model)](https://docs.rs/sealmap-model)

A language-neutral, deterministic model of a codebase: source files, the
symbols they define, the typed relations between them, and the ordered call
flow of every function. It is the shared vocabulary of
[sealmap](https://github.com/DreamLab-AI/sealmap): frontends produce a
`Codebase`, projections consume one, and neither side depends on the other.

- **Ordered collections only**, so iteration order is a function of the data.
- **Normalised paths** (`SourcePath`): relative, `/`-separated, no `.`/`..`.
- **Stable hashing** (`ContentHash`): BLAKE3 over newline-normalised text.
- **In-memory sources** (`SourceSet`), loaded from disk honouring
  `.gitignore`, so tests and embedders never need the file system.

```rust
use sealmap_model::*;

let mut cb = Codebase::new("demo");
let file = SourcePath::new("src/lib.rs").unwrap();
cb.add_file(SourceFile::new(file.clone(), "rust", SymbolId::new("demo"), "pub struct A;"));
cb.add_symbol(Symbol::new(SymbolId::new("demo::A"), "A", SymbolKind::Struct, file.clone()));
cb.add_symbol(Symbol::new(SymbolId::new("demo::B"), "B", SymbolKind::Struct, file.clone()));
cb.add_relation(Relation::new(
    SymbolId::new("demo::A"),
    SymbolId::new("demo::B"),
    RelationKind::FieldType,
    Confidence::Exact,
));

assert_eq!(cb.symbols_in_file(&file).count(), 2);
assert_eq!(cb.relations_from(&SymbolId::new("demo::A")).count(), 1);
```

## Licence

Licensed under either of [Apache License, Version 2.0](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-APACHE) or [MIT licence](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
