---
sealmap: 1
source: crates/sealmap-frontend/src/confidence.rs
module: sealmap_frontend::confidence
language: rust
source_hash: blake3:d060018393cc4de9ecc8502f7004e9524e1a96c1ee03086c23de654dbb49f6f6
lines: 124
fragments: 2
---
# `sealmap_frontend::confidence` · crates/sealmap-frontend/src/confidence.rs
> The confidence lattice, the external-call policy and call-edge aggregation.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_frontend__confidence__ExternalCalls["ExternalCalls"] {
    <<enum>>
    All
    NonStd
    None
    +keeps(self, confidence: Confidence, is_dependency: impl FnOnce#40;#41; -> bool) bool
  }
  class sealmap_frontend__confidence["sealmap_frontend::confidence"] {
    <<module>>
    +aggregate_calls(cb: &mut Codebase)
    +strongest(a: Confidence, b: Confidence) Confidence
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__symbol__Confidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_frontend__confidence ..> sealmap_model__codebase__Codebase
  sealmap_frontend__confidence ..> sealmap_model__symbol__Confidence
  sealmap_frontend__confidence__ExternalCalls ..> sealmap_model__symbol__Confidence
```

## `sealmap_frontend::confidence::aggregate_calls`
`pub fn aggregate_calls(cb: &mut Codebase)` · L63-L79
> Add one [`RelationKind::Calls`] relation per distinct (caller, target) pair found in the flows of `cb`, with the strongest confidence among the calls behind it.
```mermaid
sequenceDiagram
  participant sealmap_frontend__confidence as confidence mod
  participant sealmap_model__flow__Flow as Flow
  participant sealmap_model__symbol__Relation as Relation
  participant sealmap_model__codebase__Codebase as Codebase
  loop for s in cb.symbols.values()
    opt let Some(flow) = &s.flow
      sealmap_frontend__confidence->>sealmap_model__flow__Flow: ~calls()
      loop for c in flow.calls()
        sealmap_frontend__confidence->>sealmap_frontend__confidence: strongest(_, c.confidence)
      end
    end
  end
  loop for ((from, to), c) in calls
    sealmap_frontend__confidence->>sealmap_model__symbol__Relation: Relation::new(from, to, Calls, c)
    sealmap_frontend__confidence->>sealmap_model__codebase__Codebase: add_relation(new())
  end
```
