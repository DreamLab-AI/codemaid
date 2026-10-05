---
sealmap: 2
source: crates/sealmap-mermaid/src/lib.rs
module: "sym:cargo sealmap_mermaid ."
language: rust
source_hash: blake3:f9a6760340c883b60d65f8ec465822fc7735a5ce69c45840916837bbc897cddb
lines: 88
fragments: 1
---
# `sym:cargo sealmap_mermaid .` · crates/sealmap-mermaid/src/lib.rs
> Typed, deterministic writers for the Mermaid diagram kinds that explain code best: [`SequenceDiagram`] (behaviour), [`ClassDiagram`] (structure), [`ErDiagram`]…

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid___tReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
  }
  class sealmap_mermaid {
    <<module>>
    +mod class
    +mod er
    +mod escape
    +mod flowchart
    +mod sequence
    +mod symbol
    +mod writer
  }
```
