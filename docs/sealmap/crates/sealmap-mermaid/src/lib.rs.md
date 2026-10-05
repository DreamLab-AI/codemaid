---
sealmap: 1
source: crates/sealmap-mermaid/src/lib.rs
module: sealmap_mermaid
language: rust
source_hash: blake3:ff155f488defa98ed2c44f4c3b4120406bc5d99df1f5a81a22a952bcbc29269d
lines: 77
fragments: 1
---
# `sealmap_mermaid` · crates/sealmap-mermaid/src/lib.rs
> Typed, deterministic writers for the Mermaid diagram kinds that explain code best: [`SequenceDiagram`] (behaviour), [`ClassDiagram`] (structure), [`ErDiagram`]…

## structure
```mermaid
classDiagram
  direction LR
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
