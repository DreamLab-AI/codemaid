---
sealmap: 2
source: crates/sealmap-frontend/src/lower.rs
module: "sym:cargo sealmap_frontend . lower/"
language: rust
source_hash: blake3:7d83ef0f92f6151014de128be7b1a1bd29914b3e5baec362fd18f237814e0560
lines: 171
fragments: 4
---
# `sym:cargo sealmap_frontend . lower/` · crates/sealmap-frontend/src/lower.rs
> Flow normalisation: lower a raw flow to a model [`Flow`] once calls can be resolved.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_frontend__lower["sealmap_frontend::lower"] {
    <<module>>
    -lower_arms(arms: &[#40;String, Vec#lt;RawStep#gt;#41;], call: &mut F) Vec#lt;Arm#gt;
    +lower_flow(raw: &[RawStep], call: &mut F) Option#lt;Flow#gt;
    +lower_steps(raw: &[RawStep], call: &mut F) Vec#lt;Step#gt;
  }
  class sealmap_frontend__raw___tRawStep["RawStep"] {
    <<enum in crates/sealmap-frontend/src/raw.rs>>
  }
  class sealmap_model__flow___tArm["Arm"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__flow___tFlow["Flow"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__flow___tStep["Step"] {
    <<enum in crates/sealmap-model/src/flow.rs>>
  }
  sealmap_frontend__lower ..> sealmap_frontend__raw___tRawStep
  sealmap_frontend__lower ..> sealmap_model__flow___tArm
  sealmap_frontend__lower ..> sealmap_model__flow___tFlow
  sealmap_frontend__lower ..> sealmap_model__flow___tStep
```

## `sym:cargo sealmap_frontend . lower/lower_flow().`
`pub fn lower_flow<F>(raw: &[RawStep], call: &mut F) -> Option<Flow> where F: FnMut(&RawCall) -> Option<Call>,` · L34-L50
> Lower `raw` to a [`Flow`], resolving each call with `call` (which returns `None` to drop it).
```mermaid
sequenceDiagram
  participant sealmap_frontend__lower as lower mod
  participant sealmap_model__flow___tFlow as Flow
  sealmap_frontend__lower->>sealmap_frontend__lower: lower_steps(raw, call)
  opt steps.iter().all(| s | matches!(s, Step::Return(_)))
    Note over sealmap_frontend__lower: return None
  end
  sealmap_frontend__lower->>sealmap_model__flow___tFlow: Flow::new(steps)
```

## `sym:cargo sealmap_frontend . lower/lower_steps().`
`pub fn lower_steps<F>(raw: &[RawStep], call: &mut F) -> Vec<Step> where F: FnMut(&RawCall) -> Option<Call>,` · L52-L110
> Lower a step list without the top-level rules of [`lower_flow`].
```mermaid
sequenceDiagram
  participant sealmap_frontend__lower as lower mod
  loop for s in raw
    alt RawStep::Branch(raw_arms)
      opt via map
        sealmap_frontend__lower->>sealmap_frontend__lower: lower_steps(steps, call)
      end
    else RawStep::Parallel(arms)
      sealmap_frontend__lower->>sealmap_frontend__lower: lower_arms(arms, call)
    else RawStep::Loop(label, body)
      sealmap_frontend__lower->>sealmap_frontend__lower: lower_steps(body, call)
    else RawStep::Optional(label, body)
      sealmap_frontend__lower->>sealmap_frontend__lower: lower_steps(body, call)
    end
  end
```

## `sym:cargo sealmap_frontend . lower/lower_arms().`
`fn lower_arms<F>(arms: &[(String, Vec<RawStep>)], call: &mut F) -> Vec<Arm> where F: FnMut(&RawCall) -> Option<Call>,` · L112-L120
```mermaid
sequenceDiagram
  participant sealmap_frontend__lower as lower mod
  opt via map
    sealmap_frontend__lower->>sealmap_frontend__lower: lower_steps(steps, call)
  end
```
