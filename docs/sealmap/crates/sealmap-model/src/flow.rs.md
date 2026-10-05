---
sealmap: 1
source: crates/sealmap-model/src/flow.rs
module: sealmap_model::flow
language: rust
source_hash: blake3:be872e716806a42fdfd7f9589f3b52c07a99cadfc4ea2052fb6c6c9f09d710cd
lines: 206
fragments: 4
---
# `sealmap_model::flow` · crates/sealmap-model/src/flow.rs
> Ordered call/control flow of a function body: the raw material for sequence diagrams.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__flow__Arm["Arm"] {
    <<struct>>
    +label: String
    +steps: Vec#lt;Step#gt;
  }
  class sealmap_model__flow__Call["Call"] {
    <<struct>>
    +target: SymbolId
    +label: String
    +kind: CallKind
    +confidence: Confidence
    +awaited: bool
    +fallible: bool
    +line: u32
    +new(target: SymbolId, label: impl Into#lt;String#gt;, confidence: Confidence) Self
  }
  class sealmap_model__flow__CallKind["CallKind"] {
    <<enum>>
    Function
    Method
    Macro
  }
  class sealmap_model__flow__Exit["Exit"] {
    <<struct>>
    +label: String
    +line: u32
  }
  class sealmap_model__flow__Flow["Flow"] {
    <<struct>>
    +steps: Vec#lt;Step#gt;
    +call_count(&self) usize
    +calls(&self) impl Iterator#lt;Item = &Call#gt;
    +depth(&self) usize
    +is_empty(&self) bool
    +new(steps: Vec#lt;Step#gt;) Self
  }
  class sealmap_model__flow__Step["Step"] {
    <<enum>>
    Call#40;Call#41;
    Branch#123; #35;[doc = #quot; Arms in source order.Arms without calls are…
    Loop#123; #35;[doc = #quot; Loop header, e.g.#96;for item in items#96;.#quot;] lab…
    Optional#123; #35;[doc = #quot; Condition, e.g.#96;if let Some#40;x#41; = cache.get#40;…
    Parallel#123; #35;[doc = #quot; Arms.#quot;] arms: Vec#lt;Arm#gt;, #125;
    Return#40;Exit#41;
  }
  class sealmap_model__flow["sealmap_model::flow"] {
    <<module>>
    -collect_calls(steps: &'a [Step], out: &mut Vec#lt;&'a Call#gt;)
  }
  class sealmap_model__symbol__Confidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_model__flow ..> sealmap_model__flow__Call
  sealmap_model__flow ..> sealmap_model__flow__Step
  sealmap_model__flow__Arm o-- sealmap_model__flow__Step : steps
  sealmap_model__flow__Call *-- sealmap_model__flow__CallKind : kind
  sealmap_model__flow__Call *-- sealmap_model__symbol__Confidence : confidence
  sealmap_model__flow__Call *-- sealmap_model__symbol__SymbolId : target
  sealmap_model__flow__Flow ..> sealmap_model__flow__Call
  sealmap_model__flow__Flow o-- sealmap_model__flow__Step : steps
  sealmap_model__flow__Step o-- sealmap_model__flow__Arm : Branch, Parallel
  sealmap_model__flow__Step *-- sealmap_model__flow__Call : Call
  sealmap_model__flow__Step *-- sealmap_model__flow__Exit : Return
```

## `sealmap_model::flow::Flow::calls`
`pub fn calls(&self) -> impl Iterator<Item = &Call>` · L55-L60
> Every call in depth-first source order.
```mermaid
sequenceDiagram
  participant sealmap_model__flow__Flow as Flow
  participant sealmap_model__flow as flow mod
  sealmap_model__flow__Flow->>sealmap_model__flow: collect_calls(&self.steps, &out)
```

## `sealmap_model::flow::Flow::call_count`
`pub fn call_count(&self) -> usize` · L62-L65
> Number of calls in the whole tree.
```mermaid
sequenceDiagram
  participant sealmap_model__flow__Flow as Flow
  sealmap_model__flow__Flow->>sealmap_model__flow__Flow: calls()
```

## `sealmap_model::flow::collect_calls`
`fn collect_calls<'a>(steps: &'a [Step], out: &mut Vec<&'a Call>)` · L85-L98
```mermaid
sequenceDiagram
  participant sealmap_model__flow as flow mod
  loop for step in steps
    alt Step::Branch { arms,..} | Step::Parallel { arms }
      loop for arm in arms
        sealmap_model__flow->>sealmap_model__flow: collect_calls(&arm.steps, out)
      end
    else Step::Loop { body,..} | Step::Optional { body,..}
      sealmap_model__flow->>sealmap_model__flow: collect_calls(body, out)
    end
  end
```
