---
sealmap: 1
source: crates/sealmap-mermaid/src/writer.rs
module: sealmap_mermaid::writer
language: rust
source_hash: blake3:c5e0f8a6eee8df7ddb76e69364fcb940b40926861e92ea129d435e58b222f904
lines: 73
fragments: 1
---
# `sealmap_mermaid::writer` · crates/sealmap-mermaid/src/writer.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__writer__CodeWriter["CodeWriter"] {
    <<struct>>
    -buf: String
    -depth: usize
    +depth(&self) usize
    +finish(mut self) String
    +indented(&mut self, f: impl FnOnce#40;&mut Self#41;) &mut Self
    +line(&mut self, text: impl AsRef#lt;str#gt;) &mut Self
    +new() Self
  }
```
