---
sealmap: 1
source: crates/sealmap-mermaid/src/lib.rs
module: sealmap_mermaid
language: rust
source_hash: blake3:080101d5aa5890bd0f3632b7935f667b7c7e08ce7389e70166097ae73b7ffda7
lines: 83
fragments: 1
---
# `sealmap_mermaid` · crates/sealmap-mermaid/src/lib.rs
> Typed, deterministic writers for the Mermaid diagram kinds that explain code best: [`SequenceDiagram`] (behaviour), [`ClassDiagram`] (structure), [`ErDiagram`]…

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__ReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
  }
  class sealmap_mermaid {
    <<module>>
    +mod class
    +mod er
    +mod escape
    +mod flowchart
    +mod sequence
    +mod writer
  }
```
