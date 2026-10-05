---
sealmap: 2
source: crates/sealmap-model/src/flow.rs
module: "sym:cargo sealmap_model . flow/"
language: rust
source_hash: blake3:dedf67b71b9a4d004936b3b7362287cdd96a13d62225f2b3c73077041f8f6f10
lines: 205
fragments: 4
---
# `sym:cargo sealmap_model . flow/` · crates/sealmap-model/src/flow.rs
> Ordered call/control flow of a function body: the raw material for sequence diagrams.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__flow___tArm["Arm"] {
    <<struct>>
    +label: String
    +steps: Vec#lt;Step#gt;
  }
  class sealmap_model__flow___tCall["Call"] {
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
  class sealmap_model__flow___tCallKind["CallKind"] {
    <<enum>>
    Function
    Method
    Macro
  }
  class sealmap_model__flow___tExit["Exit"] {
    <<struct>>
    +label: String
    +line: u32
  }
  class sealmap_model__flow___tFlow["Flow"] {
    <<struct>>
    +steps: Vec#lt;Step#gt;
    +call_count(&self) usize
    +calls(&self) impl Iterator#lt;Item = &Call#gt;
    +depth(&self) usize
    +is_empty(&self) bool
    +new(steps: Vec#lt;Step#gt;) Self
  }
  class sealmap_model__flow___tStep["Step"] {
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
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__symbol___tConfidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_model__flow ..> sealmap_model__flow___tCall
  sealmap_model__flow ..> sealmap_model__flow___tStep
  sealmap_model__flow___tArm o-- sealmap_model__flow___tStep : steps
  sealmap_model__flow___tCall *-- sealmap_model__flow___tCallKind : kind
  sealmap_model__flow___tCall *-- sealmap_model__sym___tSymbolId : target
  sealmap_model__flow___tCall *-- sealmap_model__symbol___tConfidence : confidence
  sealmap_model__flow___tFlow ..> sealmap_model__flow___tCall
  sealmap_model__flow___tFlow o-- sealmap_model__flow___tStep : steps
  sealmap_model__flow___tStep o-- sealmap_model__flow___tArm : Branch, Parallel
  sealmap_model__flow___tStep *-- sealmap_model__flow___tCall : Call
  sealmap_model__flow___tStep *-- sealmap_model__flow___tExit : Return
```

## `sym:cargo sealmap_model . flow/Flow#calls().`
`pub fn calls(&self) -> impl Iterator<Item = &Call>` · L54-L59
> Every call in depth-first source order.
```mermaid
sequenceDiagram
  participant sealmap_model__flow___tFlow as Flow
  participant sealmap_model__flow as flow mod
  sealmap_model__flow___tFlow->>sealmap_model__flow: collect_calls(&self.steps, &out)
```

## `sym:cargo sealmap_model . flow/Flow#call_count().`
`pub fn call_count(&self) -> usize` · L61-L64
> Number of calls in the whole tree.
```mermaid
sequenceDiagram
  participant sealmap_model__flow___tFlow as Flow
  sealmap_model__flow___tFlow->>sealmap_model__flow___tFlow: calls()
```

## `sym:cargo sealmap_model . flow/collect_calls().`
`fn collect_calls<'a>(steps: &'a [Step], out: &mut Vec<&'a Call>)` · L84-L97
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
