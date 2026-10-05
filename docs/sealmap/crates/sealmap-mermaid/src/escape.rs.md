---
sealmap: 2
source: crates/sealmap-mermaid/src/escape.rs
module: "sym:cargo sealmap_mermaid . escape/"
language: rust
source_hash: blake3:b0f790f0647a0857a34a152d256fc901d5a71c0bcf435ac6d6f65e033f92683c
lines: 175
fragments: 2
---
# `sym:cargo sealmap_mermaid . escape/` · crates/sealmap-mermaid/src/escape.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__escape___tIdent["Ident"] {
    <<struct>>
    -0: String
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    ~from_valid(crate) Self
    +new(raw: &str) Self
  }
  class sealmap_mermaid__escape["sealmap_mermaid::escape"] {
    <<module>>
    -const RESERVED: &[&str]
    +escape_text(text: &str) String
    +escape_type(text: &str, is_field: bool) String
    ~is_reserved(crate) bool
  }
```

## `sym:cargo sealmap_mermaid . escape/escape_type().`
`pub fn escape_type(text: &str, is_field: bool) -> String` · L129-L160
> Escape a type or member text for a class-diagram body.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__escape->>sealmap_mermaid__escape: escape_text(&replace())
```
