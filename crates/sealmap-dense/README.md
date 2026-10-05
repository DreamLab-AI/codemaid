# sealmap-dense

[![crates.io](https://img.shields.io/crates/v/sealmap-dense.svg)](https://crates.io/crates/sealmap-dense)
[![docs.rs](https://img.shields.io/docsrs/sealmap-dense)](https://docs.rs/sealmap-dense)

The compact **agent projection** of a `sealmap_model::Codebase`: what an LLM
agent reads instead of the source, or instead of a Mermaid corpus, when it
needs the shape of a codebase. Depends only on `sealmap-model`.

- **Skeletons.** One Rust-like line per symbol, grouped by file in source
  order: visibility, signature, fields or variants, implemented traits, the
  line span `L<start>-<end>` and a short name (`@Db.insert`).
- **Call trees.** Indented one space per level from every entry point, with
  `alt`/`opt`/`loop`/`par` fragments and early exits. Each callable is
  expanded once, so later calls end in `^`; cycles end in `↺`; a depth cut
  ends in `…` and continues in a tree of its own. Calls are marked `~`
  inferred or `?` external (exact is unmarked). Every call site appears
  exactly once.
- **`_index.txt`.** One line per symbol, `<short> <sym:id> <path>:L<a>-<b>`,
  so an agent can turn a short name back into a full `sym:` id and a source
  window. Short names are unique per output and derived from the ids alone;
  colliding names are all qualified together, one step at a time.
- **Slices.** The dense text for a set of symbols plus their callers and
  callees to a depth, with its own index section, byte-identical for
  identical inputs. A byte budget **refuses** an oversized slice, naming the
  overflow and each seed's size alone, instead of truncating it.

Output is a pure function of the model: byte-identical on every run and
machine.

## Measured

Rust sources run through `sealmap-rust` (tests excluded), sizes in bytes,
tokens estimated as bytes / 4.

| Codebase | Source | `dense.txt` | ×source | with `_index.txt` | Mermaid corpus | Tokens per call edge, dense / Mermaid |
|---|---:|---:|---:|---:|---:|---:|
| sealmap (this workspace) | 441 k | 127 k | **0.29** | 0.46 | 389 k (0.88×) | 12.7 / 34.0 |
| tokio 1.53.2 | 3.49 M | 598 k | **0.17** | 0.32 | 1.68 M (0.48×) | 14.5 / 44.6 |
| VisionClaw | 13.9 M | 3.02 M | **0.22** | 0.38 | 8.62 M (0.62×) | 18.4 / 52.5 |

Tokens per call edge divide the call-tree section (fragments and tree headers
included) by the number of call sites; the Mermaid figure does the same for
the corpus's sequence diagrams.

## Example

```rust
use sealmap_dense::{Dense, DenseOptions, SliceOptions};
use sealmap_model::{SourceSet, SymbolId};
use sealmap_rust::{RustOptions, extract};

let mut src = SourceSet::new();
src.insert("Cargo.toml", "[package]\nname = \"shop\"").unwrap();
src.insert(
    "src/lib.rs",
    "pub struct Db;\nimpl Db { pub fn insert(&mut self, id: u64) { audit(id); } }\nfn audit(_id: u64) {}",
)
.unwrap();
let model = extract(&src, &RustOptions { name: "shop".into(), ..Default::default() }).codebase;

let dense = Dense::new(&model);
let out = dense.render(&DenseOptions::default());
assert!(out.text.contains("pub fn insert(&mut self, id: u64) L2-2 @Db.insert\n"));
assert!(out.text.ends_with("# calls\nDb.insert\n audit(id)\n"));
assert!(out.index.contains("Db.insert sym:cargo shop . Db#insert(). src/lib.rs:L2-2\n"));

// A slice for a review pack: `audit` and its callers, one level up.
let audit = SymbolId::parse("sym:cargo shop . audit().").unwrap();
let slice = dense.slice([&audit], &SliceOptions::new(1).max_bytes(4096)).unwrap();
assert!(slice.contains("# callers\naudit\n Db.insert\n"));
```

## Licence

Licensed under either of [Apache License, Version 2.0](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-APACHE) or [MIT licence](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
