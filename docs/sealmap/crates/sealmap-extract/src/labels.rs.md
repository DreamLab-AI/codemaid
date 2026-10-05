---
sealmap: 2
source: crates/sealmap-extract/src/labels.rs
module: "sym:cargo sealmap_extract . labels/"
language: rust
source_hash: blake3:24feab9b3605943cdc36d404f19c2b33a83d22afa3d6da033a7ca7f811b90d04
lines: 189
fragments: 4
---
# `sym:cargo sealmap_extract . labels/` · crates/sealmap-extract/src/labels.rs
> Label rules: how source text becomes the short strings on diagram arrows and fragments.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_extract__labels___tDeferredShape["DeferredShape"] {
    <<enum>>
    Parallel
    Loop
    Optional
  }
  class sealmap_extract__labels["sealmap_extract::labels"] {
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
  sealmap_extract__labels ..> sealmap_extract__labels___tDeferredShape
```

## `sym:cargo sealmap_extract . labels/squeeze().`
`pub fn squeeze(raw: &str) -> String` · L56-L79
> Apply the spacing rules until the string stops changing.
```mermaid
sequenceDiagram
  participant sealmap_extract__labels as labels mod
  sealmap_extract__labels->>sealmap_extract__labels: call_parens(&s)
```

## `sym:cargo sealmap_extract . labels/call_label().`
`pub fn call_label(name: &str, args: &[String]) -> String` · L113-L118
> The message for a call: `name(arg, arg)`, clipped to [`LABEL_MAX`].
```mermaid
sequenceDiagram
  participant sealmap_extract__labels as labels mod
  sealmap_extract__labels->>sealmap_extract__labels: clip(&_, LABEL_MAX)
```

## `sym:cargo sealmap_extract . labels/condition_label().`
`pub fn condition_label(text: &str) -> String` · L120-L124
> The label for a condition (`if` / `while` guard), clipped as if it were prefixed with `if ` so that the guard and the `if` arm agree.
```mermaid
sequenceDiagram
  participant sealmap_extract__labels as labels mod
  sealmap_extract__labels->>sealmap_extract__labels: clip(&_, LABEL_MAX)
```
