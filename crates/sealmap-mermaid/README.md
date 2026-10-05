# sealmap-mermaid

[![crates.io](https://img.shields.io/crates/v/sealmap-mermaid.svg)](https://crates.io/crates/sealmap-mermaid)
[![docs.rs](https://img.shields.io/docsrs/sealmap-mermaid)](https://docs.rs/sealmap-mermaid)

Typed, deterministic [Mermaid](https://mermaid.js.org) writers with safe
escaping: sequence, class, ER and flowchart diagrams. **No dependencies.**

- Every label and identifier goes through one escaping layer, so generated
  diagrams parse, whatever the source text contained.
- `Ident::from_path` turns any path into a valid Mermaid id. Plain paths stay
  readable (`app::Db` → `app__Db`); anything that could collide gets a hash
  suffix, so the mapping is injective.
- Output is a pure function of the calls made, byte for byte.

Part of [sealmap](https://github.com/DreamLab-AI/sealmap), and usable on its
own by anything that emits Mermaid.

```rust
use sealmap_mermaid::{Arrow, Ident, SequenceDiagram};

let api = Ident::from_path("app::Api");
let db = Ident::from_path("app::Db");

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
