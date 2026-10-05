---
sealmap: 2
source: crates/sealmap-extract/src/raw.rs
module: "sym:cargo sealmap_extract . raw/"
language: rust
source_hash: blake3:9b3c6d66747a323664485d26c0ae6fe27c3f316976551ecc7f2e184e2ce2a7bd
lines: 204
fragments: 2
---
# `sym:cargo sealmap_extract . raw/` · crates/sealmap-extract/src/raw.rs
> The raw flow IR: what a language adapter records while walking a function body, before any name is resolved.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_extract__raw___tCallee["Callee"] {
    <<enum>>
    Path#40;Segs#41;
    Method#123; #35;[doc = #quot; What the method is called on.#quot;] recv: Recv,…
  }
  class sealmap_extract__raw___tRawCall["RawCall"] {
    <<struct>>
    +callee: Callee
    +label: String
    +kind: CallKind
    +awaited: bool
    +fallible: bool
    +line: u32
    +new(callee: Callee, label: String, kind: CallKind, line: u32) Self
    +to_call(&self, target: SymbolId, confidence: Confidence) Call
  }
  class sealmap_extract__raw___tRawStep["RawStep"] {
    <<enum>>
    Call#40;RawCall#41;
    Branch#40;Vec#lt;#40;String, Vec#lt;RawStep#gt;#41;#gt;#41;
    Loop#40;String, Vec#lt;RawStep#gt;#41;
    Optional#40;String, Vec#lt;RawStep#gt;#41;
    Parallel#40;Vec#lt;#40;String, Vec#lt;RawStep#gt;#41;#gt;#41;
    Return#40;String, u32#41;
  }
  class sealmap_extract__raw___tRecv["Recv"] {
    <<enum>>
    SelfValue
    SelfField#40;String#41;
    Typed#40;Vec#lt;Segs#gt;#41;
    Untyped
    Returned#40;Segs#41;
    Derived#40;Box#lt;Recv#gt;#41;
    Computed#40;Box#lt;Recv#gt;#41;
    Unknown
  }
  class sealmap_extract__raw___tSegs["Segs"] {
    <<type>>
  }
  class sealmap_extract__raw["sealmap_extract::raw"] {
    <<module>>
    +last_call_mut(out: &mut [RawStep]) Option#lt;&mut RawStep#gt;
    +place_deferred(callee: &str, deferred: Vec#lt;Vec#lt;RawStep#gt;#gt;, looping: &[&str], out: &mut Vec#lt;RawStep#gt;)
    +push_arms(arms: Vec#lt;#40;String, Vec#lt;RawStep#gt;#41;#gt;, out: &mut Vec#lt;RawStep#gt;)
  }
  class sealmap_model__flow___tCall["Call"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__flow___tCallKind["CallKind"] {
    <<enum in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__symbol___tConfidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_extract__raw ..> sealmap_extract__raw___tRawStep
  sealmap_extract__raw___tCallee *-- sealmap_extract__raw___tRecv : Method
  sealmap_extract__raw___tCallee *-- sealmap_extract__raw___tSegs : Path
  sealmap_extract__raw___tRawCall *-- sealmap_extract__raw___tCallee : callee
  sealmap_extract__raw___tRawCall ..> sealmap_model__flow___tCall
  sealmap_extract__raw___tRawCall *-- sealmap_model__flow___tCallKind : kind
  sealmap_extract__raw___tRawCall ..> sealmap_model__sym___tSymbolId
  sealmap_extract__raw___tRawCall ..> sealmap_model__symbol___tConfidence
  sealmap_extract__raw___tRawStep *-- sealmap_extract__raw___tRawCall : Call
  sealmap_extract__raw___tRecv o-- sealmap_extract__raw___tSegs : Typed, Returned
```

## `sym:cargo sealmap_extract . raw/place_deferred().`
`pub fn place_deferred(callee: &str, deferred: Vec<Vec<RawStep>>, looping: &[&str], out: &mut Vec<RawStep>)` · L155-L171
> Place the bodies of closures passed to `callee` after the call itself, since they run during (not before) it.
```mermaid
sequenceDiagram
  participant sealmap_extract__raw as raw mod
  participant sealmap_extract__labels as labels mod
  loop for steps in deferred
    sealmap_extract__raw->>sealmap_extract__labels: deferred_shape(callee, looping)
    sealmap_extract__raw->>sealmap_extract__labels: deferred_label(callee, shape)
  end
```
