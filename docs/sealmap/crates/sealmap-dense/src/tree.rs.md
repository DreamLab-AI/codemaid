---
sealmap: 2
source: crates/sealmap-dense/src/tree.rs
module: "sym:cargo sealmap_dense . tree/"
language: rust
source_hash: blake3:b23d703c03ccfe5d3d69cd43861c1ce650e360105e08535a47de9cd2ae8878c2
lines: 292
fragments: 10
---
# `sym:cargo sealmap_dense . tree/` · crates/sealmap-dense/src/tree.rs
> Indented call trees.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_dense__tree___tCallerWriter["CallerWriter#lt;'w, 'a#gt;"] {
    <<struct>>
    -cb: &'a Codebase
    -shorts: &'w ShortNames#lt;'a#gt;
    -graph: &'w Graph#lt;'a#gt;
    -max_depth: usize
    -listed: BTreeSet#lt;&'a SymbolId#gt;
    -path: Vec#lt;&'a SymbolId#gt;
    ~out: String
    -level(&mut self, of: &'a SymbolId, depth: usize)
    ~new(crate) Self
    ~root(crate)
  }
  class sealmap_dense__tree___tGraph["Graph#lt;'a#gt;"] {
    <<struct>>
    ~flows: BTreeMap#lt;&'a SymbolId, &'a [Step]#gt;
    ~callees: BTreeMap#lt;&'a SymbolId, BTreeSet#lt;&'a SymbolId#gt;#gt;
    ~callers: BTreeMap#lt;&'a SymbolId, BTreeMap#lt;&'a SymbolId, Confidence#gt;#gt;
    ~new(crate) Self
  }
  class sealmap_dense__tree___tTreeWriter["TreeWriter#lt;'w, 'a#gt;"] {
    <<struct>>
    -cb: &'a Codebase
    -shorts: &'w ShortNames#lt;'a#gt;
    -graph: &'w Graph#lt;'a#gt;
    ~out: String
    -max_depth: usize
    -expanded: BTreeSet#lt;&'a SymbolId#gt;
    -on_path: BTreeSet#lt;&'a SymbolId#gt;
    -pending: Option#lt;VecDeque#lt;&'a SymbolId#gt;#gt;
    -call(&mut self, call: &'a Call, indent: usize, depth: usize)
    -fragment(&mut self, indent: usize, keyword: &str, label: &str)
    ~is_expanded(crate) bool
    ~new(crate) Self
    ~root(crate)
    -steps(&mut self, steps: &'a [Step], indent: usize, depth: usize)
    -tree(&mut self, root: &'a SymbolId, mark: &str)
  }
  class sealmap_dense__tree["sealmap_dense::tree"] {
    <<module>>
    ~call_text(crate)
    -indent_by(out: &mut String, n: usize)
  }
  class sealmap_dense__short___tShortNames["ShortNames#lt;'a#gt;"] {
    <<struct in crates/sealmap-dense/src/short.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__flow___tCall["Call"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__flow___tStep["Step"] {
    <<enum in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__symbol___tConfidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_dense__tree ..> sealmap_dense__short___tShortNames
  sealmap_dense__tree ..> sealmap_model__codebase___tCodebase
  sealmap_dense__tree ..> sealmap_model__flow___tCall
  sealmap_dense__tree___tCallerWriter o-- sealmap_dense__short___tShortNames : shorts
  sealmap_dense__tree___tCallerWriter o-- sealmap_dense__tree___tGraph : graph
  sealmap_dense__tree___tCallerWriter o-- sealmap_model__codebase___tCodebase : cb
  sealmap_dense__tree___tCallerWriter o-- sealmap_model__sym___tSymbolId : listed, path
  sealmap_dense__tree___tGraph ..> sealmap_model__codebase___tCodebase
  sealmap_dense__tree___tGraph o-- sealmap_model__flow___tStep : flows
  sealmap_dense__tree___tGraph o-- sealmap_model__sym___tSymbolId : flows, callees, callers
  sealmap_dense__tree___tGraph o-- sealmap_model__symbol___tConfidence : callers
  sealmap_dense__tree___tTreeWriter o-- sealmap_dense__short___tShortNames : shorts
  sealmap_dense__tree___tTreeWriter o-- sealmap_dense__tree___tGraph : graph
  sealmap_dense__tree___tTreeWriter o-- sealmap_model__codebase___tCodebase : cb
  sealmap_dense__tree___tTreeWriter ..> sealmap_model__flow___tCall
  sealmap_dense__tree___tTreeWriter ..> sealmap_model__flow___tStep
  sealmap_dense__tree___tTreeWriter o-- sealmap_model__sym___tSymbolId : expanded, on_path, pending
```

## `sym:cargo sealmap_dense . tree/Graph#new().`
`pub(crate) fn new(cb: &'a Codebase) -> Self` · L30-L47
```mermaid
sequenceDiagram
  participant sealmap_dense__tree___tGraph as Graph
  participant sealmap_model__flow___tFlow as Flow
  loop for sym in cb.symbols.values()
    sealmap_dense__tree___tGraph->>sealmap_model__flow___tFlow: ~calls()
  end
```

## `sym:cargo sealmap_dense . tree/TreeWriter#root().`
`pub(crate) fn root(&mut self, root: &'a SymbolId, mark: &str)` · L88-L97
> A tree rooted at `root`, its header followed by `mark` (`""` for an entry point or a slice seed, `" ↺"` for a callable only cycles reach); then, in the full pr…
```mermaid
sequenceDiagram
  participant sealmap_dense__tree___tTreeWriter as TreeWriter
  sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: tree(root, mark)
  loop while let Some(next) = self.pending.as_mut().and_then(V…
    sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: tree(next, #quot; …#quot;)
  end
```

## `sym:cargo sealmap_dense . tree/TreeWriter#tree().`
`fn tree(&mut self, root: &'a SymbolId, mark: &str)` · L99-L111
```mermaid
sequenceDiagram
  participant sealmap_dense__tree___tTreeWriter as TreeWriter
  participant sealmap_dense__short___tShortNames as ShortNames
  opt let-else
    Note over sealmap_dense__tree___tTreeWriter: return
  end
  opt self.expanded.contains(root)
    Note over sealmap_dense__tree___tTreeWriter: return
  end
  sealmap_dense__tree___tTreeWriter->>sealmap_dense__short___tShortNames: get(root)
  sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: steps(steps, 1, 1)
```

## `sym:cargo sealmap_dense . tree/TreeWriter#steps().`
`fn steps(&mut self, steps: &'a [Step], indent: usize, depth: usize)` · L113-L144
```mermaid
sequenceDiagram
  participant sealmap_dense__tree___tTreeWriter as TreeWriter
  participant sealmap_dense__skeleton as skeleton mod
  loop for step in steps
    alt Step::Call(call)
      sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: call(call, indent, depth)
    else Step::Branch { arms }
      loop for (i, arm) in arms.iter().enumerate()
        sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: fragment(indent, _, &arm.label)
        sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: steps(&arm.steps, _, depth)
      end
    else Step::Parallel { arms }
      loop for (i, arm) in arms.iter().enumerate()
        sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: fragment(indent, _, &arm.label)
        sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: steps(&arm.steps, _, depth)
      end
    else Step::Loop { label, body }
      sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: fragment(indent, #quot;loop#quot;, label)
      sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: steps(body, _, depth)
    else Step::Optional { label, body }
      sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: fragment(indent, #quot;opt#quot;, label)
      sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: steps(body, _, depth)
    else Step::Return(exit)
      sealmap_dense__tree___tTreeWriter->>sealmap_dense__skeleton: one_line(&exit.label)
      sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: fragment(indent, _, &label)
    end
  end
```

## `sym:cargo sealmap_dense . tree/TreeWriter#fragment().`
`fn fragment(&mut self, indent: usize, keyword: &str, label: &str)` · L146-L158
> `<indent><keyword> <label>`; an `else` labelled `else` is written once.
```mermaid
sequenceDiagram
  participant sealmap_dense__tree___tTreeWriter as TreeWriter
  participant sealmap_dense__skeleton as skeleton mod
  participant sealmap_dense__tree as tree mod
  sealmap_dense__tree___tTreeWriter->>sealmap_dense__skeleton: one_line(label)
  sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree: indent_by(&self.out, indent)
```

## `sym:cargo sealmap_dense . tree/TreeWriter#call().`
`fn call(&mut self, call: &'a Call, indent: usize, depth: usize)` · L160-L190
```mermaid
sequenceDiagram
  participant sealmap_dense__tree___tTreeWriter as TreeWriter
  participant sealmap_dense__tree as tree mod
  sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree: indent_by(&self.out, indent)
  sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree: call_text(&self.out, self.cb, self.shorts, call)
  opt _
    Note over sealmap_dense__tree___tTreeWriter: return
  end
  opt not self.on_path.contains(target)
    opt let Some(steps) = self.graph.flows.get(target).copie…
      sealmap_dense__tree___tTreeWriter->>sealmap_dense__tree___tTreeWriter: steps(steps, _, _)
    end
  end
```

## `sym:cargo sealmap_dense . tree/call_text().`
`pub(crate) fn call_text(out: &mut String, cb: &Codebase, shorts: &ShortNames<'_>, call: &Call)` · L193-L224
> `<confidence><name><args><.await><?>`: `~` inferred, `?` external, nothing for exact.
```mermaid
sequenceDiagram
  participant sealmap_dense__tree as tree mod
  participant sealmap_dense__short___tShortNames as ShortNames
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_dense__skeleton as skeleton mod
  sealmap_dense__tree->>sealmap_dense__short___tShortNames: get(&call.target)
  opt _
    sealmap_dense__tree->>sealmap_model__sym___tSymbolId: ~display_path()
  end
  sealmap_dense__tree->>sealmap_dense__skeleton: one_line(&call.label)
```

## `sym:cargo sealmap_dense . tree/CallerWriter#root().`
`pub(crate) fn root(&mut self, seed: &'a SymbolId)` · L242-L254
> The callers of `seed` to `max_depth` levels; nothing if it has none or was already listed.
```mermaid
sequenceDiagram
  participant sealmap_dense__tree___tCallerWriter as CallerWriter
  participant sealmap_dense__short___tShortNames as ShortNames
  opt !self.graph.callers.contains_key(seed) || self.liste…
    Note over sealmap_dense__tree___tCallerWriter: return
  end
  sealmap_dense__tree___tCallerWriter->>sealmap_dense__short___tShortNames: get(seed)
  sealmap_dense__tree___tCallerWriter->>sealmap_dense__tree___tCallerWriter: level(seed, 1)
```

## `sym:cargo sealmap_dense . tree/CallerWriter#level().`
`fn level(&mut self, of: &'a SymbolId, depth: usize)` · L256-L287
```mermaid
sequenceDiagram
  participant sealmap_dense__tree___tCallerWriter as CallerWriter
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_dense__skeleton as skeleton mod
  participant sealmap_dense__tree as tree mod
  participant sealmap_dense__short___tShortNames as ShortNames
  opt let-else
    Note over sealmap_dense__tree___tCallerWriter: return
  end
  loop each via sort_by
    sealmap_dense__tree___tCallerWriter->>sealmap_model__codebase___tCodebase: symbol(a.0)
    sealmap_dense__tree___tCallerWriter->>sealmap_model__codebase___tCodebase: symbol(b.0)
    opt (Some(x), Some(y))
      sealmap_dense__tree___tCallerWriter->>sealmap_dense__skeleton: order_key(x)
      sealmap_dense__tree___tCallerWriter->>sealmap_dense__skeleton: order_key(y)
    end
  end
  loop for (caller, confidence) in direct
    sealmap_dense__tree___tCallerWriter->>sealmap_dense__tree: indent_by(&self.out, depth)
    sealmap_dense__tree___tCallerWriter->>sealmap_dense__short___tShortNames: get(caller)
    opt not self.path.contains(&caller)
      sealmap_dense__tree___tCallerWriter->>sealmap_dense__tree___tCallerWriter: level(caller, _)
    end
  end
```
