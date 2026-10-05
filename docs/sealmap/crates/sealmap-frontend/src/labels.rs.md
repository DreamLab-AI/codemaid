---
sealmap: 2
source: crates/sealmap-frontend/src/labels.rs
module: "sym:cargo sealmap_frontend . labels/"
language: rust
source_hash: blake3:5ebe7e82cd01012619e4cfdf3af5077d817e9cb374d1886529cd419471c39026
lines: 189
fragments: 4
---
# `sym:cargo sealmap_frontend . labels/` · crates/sealmap-frontend/src/labels.rs
> Label rules: how source text becomes the short strings on diagram arrows and fragments.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_frontend__labels___tDeferredShape["DeferredShape"] {
    <<enum>>
    Parallel
    Loop
    Optional
  }
  class sealmap_frontend__labels["sealmap_frontend::labels"] {
    <<module>>
    +const ARG_LITERAL_MAX: usize
    +const DOC_SUMMARY_MAX: usize
    +const LABEL_MAX: usize
    -const RULES: &[#40;&str, &str#41;]
    +const SIGNATURE_MAX: usize
    +call_label(name: &str, args: &[String]) String
    -call_parens(s: &str) String
    +clip(s: &str, max: usize) String
    +condition_label(text: &str) String
    +deferred_label(callee: &str, shape: DeferredShape) String
    +deferred_shape(callee: &str, looping: &[&str]) DeferredShape
    +squeeze(raw: &str) String
  }
  sealmap_frontend__labels ..> sealmap_frontend__labels___tDeferredShape
```

## `sym:cargo sealmap_frontend . labels/squeeze().`
`pub fn squeeze(raw: &str) -> String` · L56-L79
> Apply the spacing rules until the string stops changing.
```mermaid
sequenceDiagram
  participant sealmap_frontend__labels as labels mod
  sealmap_frontend__labels->>sealmap_frontend__labels: call_parens(&s)
```

## `sym:cargo sealmap_frontend . labels/call_label().`
`pub fn call_label(name: &str, args: &[String]) -> String` · L113-L118
> The message for a call: `name(arg, arg)`, clipped to [`LABEL_MAX`].
```mermaid
sequenceDiagram
  participant sealmap_frontend__labels as labels mod
  sealmap_frontend__labels->>sealmap_frontend__labels: clip(&_, LABEL_MAX)
```

## `sym:cargo sealmap_frontend . labels/condition_label().`
`pub fn condition_label(text: &str) -> String` · L120-L124
> The label for a condition (`if` / `while` guard), clipped as if it were prefixed with `if ` so that the guard and the `if` arm agree.
```mermaid
sequenceDiagram
  participant sealmap_frontend__labels as labels mod
  sealmap_frontend__labels->>sealmap_frontend__labels: clip(&_, LABEL_MAX)
```
