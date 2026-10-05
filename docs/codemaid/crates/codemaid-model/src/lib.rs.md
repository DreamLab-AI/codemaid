---
codemaid: 1
source: crates/codemaid-model/src/lib.rs
module: codemaid_model
language: rust
source_hash: blake3:8dd4d14d4e82dd820e16808515604bf2210cfb89bf02d44fb43fd21acf5129b0
lines: 92
fragments: 1
---
# `codemaid_model` · crates/codemaid-model/src/lib.rs
> A language-neutral, **deterministic** model of a codebase: source files, the symbols they define, and the typed relations between those symbols.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_model {
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
