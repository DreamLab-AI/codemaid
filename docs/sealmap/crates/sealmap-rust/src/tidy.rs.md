---
sealmap: 2
source: crates/sealmap-rust/src/tidy.rs
module: "sym:cargo sealmap_rust . tidy/"
language: rust
source_hash: blake3:1d84f2a659e19f14583d5bf0a1b67932d3d22b67dfd7589cfca56e5aa84453a1
lines: 9
fragments: 2
---
# `sym:cargo sealmap_rust . tidy/` · crates/sealmap-rust/src/tidy.rs
> Compact printing of syn nodes, through the shared label rules.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_rust__tidy["sealmap_rust::tidy"] {
    <<module>>
    ~tokens(crate) String
  }
  class _quote__ToTokens["quote::ToTokens"] {
    <<external>>
  }
  sealmap_rust__tidy ..> _quote__ToTokens
```

## `sym:cargo sealmap_rust . tidy/tokens().`
`pub(crate) fn tokens(node: &impl ToTokens) -> String` · L6-L9
> Print any syn node compactly (see [`squeeze`]).
```mermaid
sequenceDiagram
  participant sealmap_rust__tidy as tidy mod
  participant _quote as quote ext
  participant sealmap_extract__labels as labels mod
  sealmap_rust__tidy->>_quote: ToTokens::to_token_stream()
  sealmap_rust__tidy->>sealmap_extract__labels: squeeze(&to_string())
```
