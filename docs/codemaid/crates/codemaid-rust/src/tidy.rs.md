---
codemaid: 1
source: crates/codemaid-rust/src/tidy.rs
module: codemaid_rust::tidy
language: rust
source_hash: blake3:3c4a7c6f39fdb55a958336084c1053db0289d933ad4495f656d7321b6ada5b1c
lines: 124
fragments: 3
---
# `codemaid_rust::tidy` · crates/codemaid-rust/src/tidy.rs
> Compact, deterministic printing of token streams.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_rust__tidy["codemaid_rust::tidy"] {
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
  codemaid_rust__tidy ..> quote__ToTokens
```

## `codemaid_rust::tidy::tokens`
`pub(crate) fn tokens(node: &impl ToTokens) -> String` · L41-L44
> Print any syn node compactly.
```mermaid
sequenceDiagram
  participant codemaid_rust__tidy as tidy mod
  participant quote as quote ext
  codemaid_rust__tidy->>quote: ToTokens::to_token_stream()
  codemaid_rust__tidy->>codemaid_rust__tidy: squeeze(&to_string())
```

## `codemaid_rust::tidy::squeeze`
`pub(crate) fn squeeze(raw: &str) -> String` · L46-L69
> Apply the spacing rules until the string stops changing.
```mermaid
sequenceDiagram
  participant codemaid_rust__tidy as tidy mod
  codemaid_rust__tidy->>codemaid_rust__tidy: call_parens(&s)
```
