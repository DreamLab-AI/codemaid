---
codemaid: 1
source: crates/codemaid-mermaid/src/escape.rs
module: codemaid_mermaid::escape
language: rust
source_hash: blake3:79e4f2a97d162a07925de63681ab17612130b8c9ded0124c058202357958c778
lines: 165
fragments: 2
---
# `codemaid_mermaid::escape` · crates/codemaid-mermaid/src/escape.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_mermaid__escape__Ident["Ident"] {
    <<struct>>
    -0: String
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    +from_path(path: &str) Self
    +new(raw: &str) Self
  }
  class codemaid_mermaid__escape["codemaid_mermaid::escape"] {
    <<module>>
    -const RESERVED: &[&str]
    +escape_text(text: &str) String
    +escape_type(text: &str, is_field: bool) String
  }
```

## `codemaid_mermaid::escape::escape_type`
`pub fn escape_type(text: &str, is_field: bool) -> String` · L134-L165
> Escape a type or member text for a class-diagram body.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__escape->>codemaid_mermaid__escape: escape_text(&replace())
```
