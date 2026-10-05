---
sealmap: 1
source: crates/sealmap-mermaid/src/escape.rs
module: sealmap_mermaid::escape
language: rust
source_hash: blake3:46494e9a91ca6512ac7b351b4543c82ba0b73574b76a3dbc341a2406c6f9f660
lines: 273
fragments: 3
---
# `sealmap_mermaid::escape` · crates/sealmap-mermaid/src/escape.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__escape__Ident["Ident"] {
    <<struct>>
    -0: String
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    +from_path(path: &str) Self
    +new(raw: &str) Self
  }
  class sealmap_mermaid__escape["sealmap_mermaid::escape"] {
    <<module>>
    -const RESERVED: &[&str]
    +escape_text(text: &str) String
    +escape_type(text: &str, is_field: bool) String
    -fnv1a64(bytes: &[u8]) u64
    -is_plain_path(path: &str) bool
  }
```

## `sealmap_mermaid::escape::Ident::from_path`
`pub fn from_path(path: &str) -> Self` · L78-L102
> Map a `::`-separated path to an id, using `__` as the separator.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__escape__Ident as Ident
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__escape__Ident->>sealmap_mermaid__escape: is_plain_path(path)
  opt is_plain_path(path)
    opt RESERVED.iter().any(| r | r.eq_ignore_ascii_case(&jo…
      Note over sealmap_mermaid__escape__Ident: return Self(joined + #quot;___#quot;)
    end
    Note over sealmap_mermaid__escape__Ident: return Self(joined)
  end
  sealmap_mermaid__escape__Ident->>sealmap_mermaid__escape: fnv1a64(as_bytes())
```

## `sealmap_mermaid::escape::escape_type`
`pub fn escape_type(text: &str, is_field: bool) -> String` · L186-L217
> Escape a type or member text for a class-diagram body.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__escape->>sealmap_mermaid__escape: escape_text(&replace())
```
