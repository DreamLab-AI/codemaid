# sealmap-mermaid

[![crates.io](https://img.shields.io/crates/v/sealmap-mermaid.svg)](https://crates.io/crates/sealmap-mermaid)
[![docs.rs](https://img.shields.io/docsrs/sealmap-mermaid)](https://docs.rs/sealmap-mermaid)

Typed, deterministic [Mermaid](https://mermaid.js.org) writers with safe
escaping: sequence, class, ER and flowchart diagrams. With
`default-features = false`, **no dependencies**.

- Every label and identifier goes through one escaping layer, so generated
  diagrams parse, whatever the source text contained.
- `Ident::from_symbol` (default feature `model`) turns a `sym:` symbol id into
  a Mermaid id by an injective encoding, readable where names are plain
  (`sym:cargo app . db/Db#get().` → `app__db___tDb___fget`). Two symbols never
  share a diagram id, by construction rather than by a collision check.
- `Ident::new` sanitises any other text into a valid id.
- Output is a pure function of the calls made, byte for byte.

Part of [sealmap](https://github.com/DreamLab-AI/sealmap), and usable on its
own by anything that emits Mermaid.

```rust
use sealmap_mermaid::{Arrow, Ident, SequenceDiagram};

let api = Ident::new("app__Api");
let db = Ident::new("app__Db");

let mut seq = SequenceDiagram::new();
seq.participant(api.clone(), "Api");
seq.participant(db.clone(), "Db");
seq.message(&api, &db, Arrow::Sync, "query(sql); fetch");

let text = seq.render();
assert!(text.starts_with("sequenceDiagram\n"));
// `;` would end the message early in Mermaid; it is escaped.
assert!(text.contains("app__Api->>app__Db: query(sql)#59; fetch"));
```

## Licence

Licensed under either of [Apache License, Version 2.0](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-APACHE) or [MIT licence](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
