---
sealmap: 2
source: crates/sealmap-model/src/lib.rs
module: "sym:cargo sealmap_model ."
language: rust
source_hash: blake3:d558a4932b49b7c097546724de7f1ba827fe31807c7ffd72e85192f8da6ab68a
lines: 115
fragments: 1
---
# `sym:cargo sealmap_model .` · crates/sealmap-model/src/lib.rs
> A language-neutral, **deterministic** model of a codebase: source files, the symbols they define, and the typed relations between those symbols.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model___tReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
  }
  class sealmap_model {
    <<module>>
    +const MODEL_SCHEMA_VERSION: u32
    +mod codebase
    +mod flow
    +mod hash
    +mod path
    +mod source
    +mod sym
    +mod symbol
  }
```
