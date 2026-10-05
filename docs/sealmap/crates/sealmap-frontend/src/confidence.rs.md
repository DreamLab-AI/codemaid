---
sealmap: 2
source: crates/sealmap-frontend/src/confidence.rs
module: "sym:cargo sealmap_frontend . confidence/"
language: rust
source_hash: blake3:965b3041921428ef808618f62e3b3dd80908be9b4961f153c92f73fd8792dd4c
lines: 134
fragments: 2
---
# `sym:cargo sealmap_frontend . confidence/` · crates/sealmap-frontend/src/confidence.rs
> The confidence lattice, the external-call policy and call-edge aggregation.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_frontend__confidence___tExternalCalls["ExternalCalls"] {
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
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__symbol___tConfidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_frontend__confidence ..> sealmap_model__codebase___tCodebase
  sealmap_frontend__confidence ..> sealmap_model__symbol___tConfidence
  sealmap_frontend__confidence___tExternalCalls ..> sealmap_model__symbol___tConfidence
```

## `sym:cargo sealmap_frontend . confidence/aggregate_calls().`
`pub fn aggregate_calls(cb: &mut Codebase)` · L63-L79
> Add one [`RelationKind::Calls`] relation per distinct (caller, target) pair found in the flows of `cb`, with the strongest confidence among the calls behind it.
```mermaid
sequenceDiagram
  participant sealmap_frontend__confidence as confidence mod
  participant sealmap_model__flow___tFlow as Flow
  participant sealmap_model__symbol___tRelation as Relation
  participant sealmap_model__codebase___tCodebase as Codebase
  loop for s in cb.symbols.values()
    opt let Some(flow) = &s.flow
      sealmap_frontend__confidence->>sealmap_model__flow___tFlow: ~calls()
      loop for c in flow.calls()
        sealmap_frontend__confidence->>sealmap_frontend__confidence: strongest(_, c.confidence)
      end
    end
  end
  loop for ((from, to), c) in calls
    sealmap_frontend__confidence->>sealmap_model__symbol___tRelation: Relation::new(from, to, Calls, c)
    sealmap_frontend__confidence->>sealmap_model__codebase___tCodebase: add_relation(new())
  end
```
