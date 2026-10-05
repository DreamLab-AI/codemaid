---
sealmap: 1
source: crates/sealmap-model/src/lib.rs
module: sealmap_model
language: rust
source_hash: blake3:1d5d59dfbcb236aa622146365ec698729ba5f029ef9693a55794a30b1e798707
lines: 92
fragments: 1
---
# `sealmap_model` · crates/sealmap-model/src/lib.rs
> A language-neutral, **deterministic** model of a codebase: source files, the symbols they define, and the typed relations between those symbols.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model {
    <<module>>
    +const MODEL_SCHEMA_VERSION: u32
    +mod codebase
    +mod flow
    +mod hash
    +mod path
    +mod source
    +mod symbol
  }
```
