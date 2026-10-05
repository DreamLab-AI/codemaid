---
sealmap: 1
source: crates/sealmap-rust/src/tidy.rs
module: sealmap_rust::tidy
language: rust
source_hash: blake3:3c4a7c6f39fdb55a958336084c1053db0289d933ad4495f656d7321b6ada5b1c
lines: 124
fragments: 3
---
# `sealmap_rust::tidy` · crates/sealmap-rust/src/tidy.rs
> Compact, deterministic printing of token streams.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_rust__tidy["sealmap_rust::tidy"] {
    <<module>>
    -const RULES: &[#40;&str, &str#41;]
    -call_parens(s: &str) String
    ~clip(crate) String
    ~squeeze(crate) String
    ~tokens(crate) String
  }
  class quote__ToTokens["quote::ToTokens"] {
    <<external>>
  }
  sealmap_rust__tidy ..> quote__ToTokens
```

## `sealmap_rust::tidy::tokens`
`pub(crate) fn tokens(node: &impl ToTokens) -> String` · L41-L44
> Print any syn node compactly.
```mermaid
sequenceDiagram
  participant sealmap_rust__tidy as tidy mod
  participant quote as quote ext
  sealmap_rust__tidy->>quote: ToTokens::to_token_stream()
  sealmap_rust__tidy->>sealmap_rust__tidy: squeeze(&to_string())
```

## `sealmap_rust::tidy::squeeze`
`pub(crate) fn squeeze(raw: &str) -> String` · L46-L69
> Apply the spacing rules until the string stops changing.
```mermaid
sequenceDiagram
  participant sealmap_rust__tidy as tidy mod
  sealmap_rust__tidy->>sealmap_rust__tidy: call_parens(&s)
```
