---
codemaid: 1
source: crates/codemaid-model/src/hash.rs
module: codemaid_model::hash
language: rust
source_hash: blake3:da8ea6172ba0357e157b8acc2b1792cf0736f3bb6a977ccfb1b35cdfde50b9a2
lines: 80
fragments: 3
---
# `codemaid_model::hash` · crates/codemaid-model/src/hash.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_model__hash__ContentHash["ContentHash"] {
    <<struct>>
    -0: String
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    +of_bytes(bytes: &[u8]) Self
    +of_text(text: &str) Self
    +parse(s: &str) Option#lt;Self#gt;
    +short(&self, n: usize) &str
  }
  class codemaid_model__hash["codemaid_model::hash"] {
    <<module>>
    ~normalise_newlines(crate) std::borrow::Cow#lt;'_, str#gt;
  }
```

## `codemaid_model::hash::ContentHash::of_text`
`pub fn of_text(text: &str) -> Self` · L24-L28
> Hash `text` after normalising line endings.
```mermaid
sequenceDiagram
  participant codemaid_model__hash__ContentHash as ContentHash
  participant codemaid_model__hash as hash mod
  codemaid_model__hash__ContentHash->>codemaid_model__hash: normalise_newlines(text)
```

## `codemaid_model::hash::ContentHash::of_bytes`
`pub fn of_bytes(bytes: &[u8]) -> Self` · L30-L33
> Hash raw bytes with no normalisation.
```mermaid
sequenceDiagram
  participant codemaid_model__hash__ContentHash as ContentHash
  participant blake3 as blake3 ext
  codemaid_model__hash__ContentHash->>blake3: blake3::hash(bytes)
```
