---
sealmap: 1
source: crates/sealmap-model/src/lib.rs
module: sealmap_model
language: rust
source_hash: blake3:89486a3fda3e7e7bf32b3896e7dfe954750d33466442e30ce51bd96158bc1519
lines: 98
fragments: 1
---
# `sealmap_model` · crates/sealmap-model/src/lib.rs
> A language-neutral, **deterministic** model of a codebase: source files, the symbols they define, and the typed relations between those symbols.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__ReadmeDoctests["ReadmeDoctests"] {
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
    +mod symbol
  }
```
