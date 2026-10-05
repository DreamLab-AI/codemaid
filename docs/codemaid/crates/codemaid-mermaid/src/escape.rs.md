---
codemaid: 1
source: crates/codemaid-mermaid/src/escape.rs
module: codemaid_mermaid::escape
language: rust
source_hash: blake3:703102bfb7f618fb11a5e0d53cd32938464548d2a7618b3bf6de01978446f7b0
lines: 273
fragments: 3
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
    -fnv1a64(bytes: &[u8]) u64
    -is_plain_path(path: &str) bool
  }
```

## `codemaid_mermaid::escape::Ident::from_path`
`pub fn from_path(path: &str) -> Self` · L78-L102
> Map a `::`-separated path to an id, using `__` as the separator.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__escape__Ident as Ident
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__escape__Ident->>codemaid_mermaid__escape: is_plain_path(path)
  opt is_plain_path(path)
    opt RESERVED.iter().any(| r | r.eq_ignore_ascii_case(&jo…
      Note over codemaid_mermaid__escape__Ident: return Self(joined + #quot;___#quot;)
    end
    Note over codemaid_mermaid__escape__Ident: return Self(joined)
  end
  codemaid_mermaid__escape__Ident->>codemaid_mermaid__escape: fnv1a64(as_bytes())
```

## `codemaid_mermaid::escape::escape_type`
`pub fn escape_type(text: &str, is_field: bool) -> String` · L186-L217
> Escape a type or member text for a class-diagram body.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__escape->>codemaid_mermaid__escape: escape_text(&replace())
```
