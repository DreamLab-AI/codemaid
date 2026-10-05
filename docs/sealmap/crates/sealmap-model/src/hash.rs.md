---
sealmap: 1
source: crates/sealmap-model/src/hash.rs
module: sealmap_model::hash
language: rust
source_hash: blake3:77503e795b8d693b5805f1d92011694c912f83bb67106e3c8af599b699e66034
lines: 80
fragments: 3
---
# `sealmap_model::hash` · crates/sealmap-model/src/hash.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__hash__ContentHash["ContentHash"] {
    <<struct>>
    -0: String
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    +of_bytes(bytes: &[u8]) Self
    +of_text(text: &str) Self
    +parse(s: &str) Option#lt;Self#gt;
    +short(&self, n: usize) &str
  }
  class sealmap_model__hash["sealmap_model::hash"] {
    <<module>>
    ~normalise_newlines(crate) std::borrow::Cow#lt;'_, str#gt;
  }
```

## `sealmap_model::hash::ContentHash::of_text`
`pub fn of_text(text: &str) -> Self` · L24-L28
> Hash `text` after normalising line endings.
```mermaid
sequenceDiagram
  participant sealmap_model__hash__ContentHash as ContentHash
  participant sealmap_model__hash as hash mod
  sealmap_model__hash__ContentHash->>sealmap_model__hash: normalise_newlines(text)
```

## `sealmap_model::hash::ContentHash::of_bytes`
`pub fn of_bytes(bytes: &[u8]) -> Self` · L30-L33
> Hash raw bytes with no normalisation.
```mermaid
sequenceDiagram
  participant sealmap_model__hash__ContentHash as ContentHash
  participant blake3 as blake3 ext
  sealmap_model__hash__ContentHash->>blake3: blake3::hash(bytes)
```
