---
codemaid: 1
source: crates/codemaid-mermaid/src/lib.rs
module: codemaid_mermaid
language: rust
source_hash: blake3:71761b48aa25a005c631f4005066b7d123cdd2e98d0af6005e3d6671ce9a5e22
lines: 77
fragments: 1
---
# `codemaid_mermaid` · crates/codemaid-mermaid/src/lib.rs
> Typed, deterministic writers for the Mermaid diagram kinds that explain code best: [`SequenceDiagram`] (behaviour), [`ClassDiagram`] (structure), [`ErDiagram`]…

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_mermaid {
    <<module>>
    +mod class
    +mod er
    +mod escape
    +mod flowchart
    +mod sequence
    +mod writer
  }
```
