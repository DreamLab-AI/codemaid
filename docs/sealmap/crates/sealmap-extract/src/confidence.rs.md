---
sealmap: 2
source: crates/sealmap-extract/src/confidence.rs
module: "sym:cargo sealmap_extract . confidence/"
language: rust
source_hash: blake3:4592a42b5509e92fe8bd137c772f38c99bcc42ac8457efa2aa17e97ef7a4a48d
lines: 134
fragments: 2
---
# `sym:cargo sealmap_extract . confidence/` · crates/sealmap-extract/src/confidence.rs
> The confidence lattice, the external-call policy and call-edge aggregation.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_extract__confidence___tExternalCalls["ExternalCalls"] {
    <<enum>>
    All
    NonStd
    None
    +keeps(self, confidence: Confidence, is_dependency: impl FnOnce#40;#41; -> bool) bool
  }
  class sealmap_extract__confidence["sealmap_extract::confidence"] {
    <<module>>
    +aggregate_calls(cb: &mut Codebase)
    +strongest(a: Confidence, b: Confidence) Confidence
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__symbol___tConfidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_extract__confidence ..> sealmap_model__codebase___tCodebase
  sealmap_extract__confidence ..> sealmap_model__symbol___tConfidence
  sealmap_extract__confidence___tExternalCalls ..> sealmap_model__symbol___tConfidence
```

## `sym:cargo sealmap_extract . confidence/aggregate_calls().`
`pub fn aggregate_calls(cb: &mut Codebase)` · L63-L79
> Add one [`RelationKind::Calls`] relation per distinct (caller, target) pair found in the flows of `cb`, with the strongest confidence among the calls behind it.
```mermaid
sequenceDiagram
  participant sealmap_extract__confidence as confidence mod
  participant sealmap_model__flow___tFlow as Flow
  participant sealmap_model__symbol___tRelation as Relation
  participant sealmap_model__codebase___tCodebase as Codebase
  loop for s in cb.symbols.values()
    opt let Some(flow) = &s.flow
      sealmap_extract__confidence->>sealmap_model__flow___tFlow: ~calls()
      loop for c in flow.calls()
        sealmap_extract__confidence->>sealmap_extract__confidence: strongest(_, c.confidence)
      end
    end
  end
  loop for ((from, to), c) in calls
    sealmap_extract__confidence->>sealmap_model__symbol___tRelation: Relation::new(from, to, Calls, c)
    sealmap_extract__confidence->>sealmap_model__codebase___tCodebase: add_relation(new())
  end
```
