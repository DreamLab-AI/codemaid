---
codemaid: 1
source: crates/codemaid-model/src/flow.rs
module: codemaid_model::flow
language: rust
source_hash: blake3:44102c4859c26e913209e83fa8981991e2d86d8c944e09fe001193027a31add5
lines: 206
fragments: 4
---
# `codemaid_model::flow` · crates/codemaid-model/src/flow.rs
> Ordered call/control flow of a function body: the raw material for sequence diagrams.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_model__flow__Arm["Arm"] {
    <<struct>>
    +label: String
    +steps: Vec#lt;Step#gt;
  }
  class codemaid_model__flow__Call["Call"] {
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
  class codemaid_model__flow__CallKind["CallKind"] {
    <<enum>>
    Function
    Method
    Macro
  }
  class codemaid_model__flow__Exit["Exit"] {
    <<struct>>
    +label: String
    +line: u32
  }
  class codemaid_model__flow__Flow["Flow"] {
    <<struct>>
    +steps: Vec#lt;Step#gt;
    +call_count(&self) usize
    +calls(&self) impl Iterator#lt;Item = &Call#gt;
    +depth(&self) usize
    +is_empty(&self) bool
    +new(steps: Vec#lt;Step#gt;) Self
  }
  class codemaid_model__flow__Step["Step"] {
    <<enum>>
    Call#40;Call#41;
    Branch#123; #35;[doc = #quot; Arms in source order.Arms without calls are…
    Loop#123; #35;[doc = #quot; Loop header, e.g.#96;for item in items#96;.#quot;] lab…
    Optional#123; #35;[doc = #quot; Condition, e.g.#96;if let Some#40;x#41; = cache.get#40;…
    Parallel#123; #35;[doc = #quot; Arms.#quot;] arms: Vec#lt;Arm#gt;, #125;
    Return#40;Exit#41;
  }
  class codemaid_model__flow["codemaid_model::flow"] {
    <<module>>
    -collect_calls(steps: &'a [Step], out: &mut Vec#lt;&'a Call#gt;)
  }
  class codemaid_model__symbol__Confidence["Confidence"] {
    <<enum in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  codemaid_model__flow ..> codemaid_model__flow__Call
  codemaid_model__flow ..> codemaid_model__flow__Step
  codemaid_model__flow__Arm o-- codemaid_model__flow__Step : steps
  codemaid_model__flow__Call *-- codemaid_model__flow__CallKind : kind
  codemaid_model__flow__Call *-- codemaid_model__symbol__Confidence : confidence
  codemaid_model__flow__Call *-- codemaid_model__symbol__SymbolId : target
  codemaid_model__flow__Flow ..> codemaid_model__flow__Call
  codemaid_model__flow__Flow o-- codemaid_model__flow__Step : steps
  codemaid_model__flow__Step o-- codemaid_model__flow__Arm : Branch, Parallel
  codemaid_model__flow__Step *-- codemaid_model__flow__Call : Call
  codemaid_model__flow__Step *-- codemaid_model__flow__Exit : Return
```

## `codemaid_model::flow::Flow::calls`
`pub fn calls(&self) -> impl Iterator<Item = &Call>` · L55-L60
> Every call in depth-first source order.
```mermaid
sequenceDiagram
  participant codemaid_model__flow__Flow as Flow
  participant codemaid_model__flow as flow mod
  codemaid_model__flow__Flow->>codemaid_model__flow: collect_calls(&self.steps, &out)
```

## `codemaid_model::flow::Flow::call_count`
`pub fn call_count(&self) -> usize` · L62-L65
> Number of calls in the whole tree.
```mermaid
sequenceDiagram
  participant codemaid_model__flow__Flow as Flow
  codemaid_model__flow__Flow->>codemaid_model__flow__Flow: calls()
```

## `codemaid_model::flow::collect_calls`
`fn collect_calls<'a>(steps: &'a [Step], out: &mut Vec<&'a Call>)` · L85-L98
```mermaid
sequenceDiagram
  participant codemaid_model__flow as flow mod
  loop for step in steps
    alt Step::Branch { arms,..} | Step::Parallel { arms }
      loop for arm in arms
        codemaid_model__flow->>codemaid_model__flow: collect_calls(&arm.steps, out)
      end
    else Step::Loop { body,..} | Step::Optional { body,..}
      codemaid_model__flow->>codemaid_model__flow: collect_calls(body, out)
    end
  end
```
