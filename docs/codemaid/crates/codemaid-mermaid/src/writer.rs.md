---
codemaid: 1
source: crates/codemaid-mermaid/src/writer.rs
module: codemaid_mermaid::writer
language: rust
source_hash: blake3:607290aa9bec707775f34d414de104e086708ae7f55229b0d40d8b0c19608396
lines: 73
fragments: 1
---
# `codemaid_mermaid::writer` · crates/codemaid-mermaid/src/writer.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_mermaid__writer__CodeWriter["CodeWriter"] {
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
