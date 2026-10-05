---
codemaid: 1
source: crates/codemaid-model/src/codebase.rs
module: codemaid_model::codebase
language: rust
source_hash: blake3:fab2ce693d8f980de8eb6920c76cca477e11b1d04078c2ae7e011490d3b0f98f
lines: 158
fragments: 4
---
# `codemaid_model::codebase` · crates/codemaid-model/src/codebase.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_model__codebase__Codebase["Codebase"] {
    <<struct>>
    +schema: u32
    +name: String
    +files: BTreeMap#lt;SourcePath, SourceFile#gt;
    +symbols: BTreeMap#lt;SymbolId, Symbol#gt;
    +relations: BTreeSet#lt;Relation#gt;
    +add_file(&mut self, file: SourceFile)
    +add_relation(&mut self, relation: Relation)
    +add_symbol(&mut self, symbol: Symbol)
    +children(&'a self, id: &'a SymbolId) impl Iterator#lt;Item = &'a Symbol#gt;
    +file(&self, path: &SourcePath) Option#lt;&SourceFile#gt;
    +is_internal(&self, id: &SymbolId) bool
    +new(name: impl Into#lt;String#gt;) Self
    +owner_of(&self, id: &SymbolId) Option#lt;SymbolId#gt;
    +relations_from(&'a self, id: &'a SymbolId) impl Iterator#lt;Item = &'a Relation#gt;
    +relations_of_kind(&self, kind: RelationKind) impl Iterator#lt;Item = &Relation#gt;
    +relations_to(&'a self, id: &'a SymbolId) impl Iterator#lt;Item = &'a Relation#gt;
    +stats(&self) CodebaseStats
    +symbol(&self, id: &SymbolId) Option#lt;&Symbol#gt;
    +symbols_in_file(&'a self, path: &'a SourcePath) impl Iterator#lt;Item = &'a Symbol#gt;
    +symbols_of_kind(&self, kind: SymbolKind) impl Iterator#lt;Item = &Symbol#gt;
  }
  class codemaid_model__codebase__CodebaseStats["CodebaseStats"] {
    <<struct>>
    +files: usize
    +symbols: usize
    +relations: usize
    +flows: usize
    +calls: usize
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class codemaid_model__source__SourceFile["SourceFile"] {
    <<struct in crates/codemaid-model/src/source.rs>>
  }
  class codemaid_model__symbol__Relation["Relation"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__RelationKind["RelationKind"] {
    <<enum in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__Symbol["Symbol"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__SymbolKind["SymbolKind"] {
    <<enum in crates/codemaid-model/src/symbol.rs>>
  }
  codemaid_model__codebase__Codebase ..> codemaid_model__codebase__CodebaseStats
  codemaid_model__codebase__Codebase o-- codemaid_model__path__SourcePath : files
  codemaid_model__codebase__Codebase o-- codemaid_model__source__SourceFile : files
  codemaid_model__codebase__Codebase o-- codemaid_model__symbol__Relation : relations
  codemaid_model__codebase__Codebase ..> codemaid_model__symbol__RelationKind
  codemaid_model__codebase__Codebase o-- codemaid_model__symbol__Symbol : symbols
  codemaid_model__codebase__Codebase o-- codemaid_model__symbol__SymbolId : symbols
  codemaid_model__codebase__Codebase ..> codemaid_model__symbol__SymbolKind
```

## `codemaid_model::codebase::Codebase::relations_from`
`pub fn relations_from<'a>(&'a self, id: &'a SymbolId) -> impl Iterator<Item = &'a Relation>` · L95-L102
> Relations whose source is `id`.
```mermaid
sequenceDiagram
  participant codemaid_model__codebase__Codebase as Codebase
  participant codemaid_model__symbol__SymbolId as SymbolId
  participant codemaid_model__symbol__Relation as Relation
  codemaid_model__codebase__Codebase->>codemaid_model__symbol__SymbolId: SymbolId::new(#quot;#quot;)
  codemaid_model__codebase__Codebase->>codemaid_model__symbol__Relation: Relation::new(clone(), new(), Contains, Exact)
```

## `codemaid_model::codebase::Codebase::owner_of`
`pub fn owner_of(&self, id: &SymbolId) -> Option<SymbolId>` · L114-L125
> The innermost *type* that owns `id` (the type for a method), or the module for free items.
```mermaid
sequenceDiagram
  participant codemaid_model__codebase__Codebase as Codebase
  participant codemaid_model__symbol__SymbolId as SymbolId
  opt None
    codemaid_model__codebase__Codebase->>codemaid_model__symbol__SymbolId: parent()
  end
```

## `codemaid_model::codebase::Codebase::stats`
`pub fn stats(&self) -> CodebaseStats` · L127-L142
> Summary counts.
```mermaid
sequenceDiagram
  participant codemaid_model__codebase__Codebase as Codebase
  participant codemaid_model__codebase__CodebaseStats as CodebaseStats
  participant codemaid_model__flow__Flow as Flow
  codemaid_model__codebase__Codebase->>codemaid_model__codebase__CodebaseStats: ~CodebaseStats::default()
  loop for sym in self.symbols.values()
    opt let Some(flow) = &sym.flow
      codemaid_model__codebase__Codebase->>codemaid_model__flow__Flow: ~call_count()
    end
  end
```
