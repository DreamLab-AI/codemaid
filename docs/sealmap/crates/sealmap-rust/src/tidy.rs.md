---
sealmap: 1
source: crates/sealmap-rust/src/tidy.rs
module: sealmap_rust::tidy
language: rust
source_hash: blake3:e49801d4de7f4610834dc6976ce0ea3b51accae8e8731b5d903782c5092f13eb
lines: 9
fragments: 2
---
# `sealmap_rust::tidy` · crates/sealmap-rust/src/tidy.rs
> Compact printing of syn nodes, through the shared label rules.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_rust__tidy["sealmap_rust::tidy"] {
    <<module>>
    ~tokens(crate) String
  }
  class quote__ToTokens["quote::ToTokens"] {
    <<external>>
  }
  sealmap_rust__tidy ..> quote__ToTokens
```

## `sealmap_rust::tidy::tokens`
`pub(crate) fn tokens(node: &impl ToTokens) -> String` · L6-L9
> Print any syn node compactly (see [`squeeze`]).
```mermaid
sequenceDiagram
  participant sealmap_rust__tidy as tidy mod
  participant quote as quote ext
  participant sealmap_frontend__labels as labels mod
  sealmap_rust__tidy->>quote: ToTokens::to_token_stream()
  sealmap_rust__tidy->>sealmap_frontend__labels: squeeze(&to_string())
```
