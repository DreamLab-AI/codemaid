---
codemaid: 1
source: crates/codemaid-corpus/src/naming.rs
module: codemaid_corpus::naming
language: rust
source_hash: blake3:7ef20d8d01abe8decdd4183eb7b393072ed97f8a8062e4cc1c5d6128adb3d862
lines: 78
fragments: 5
---
# `codemaid_corpus::naming` · crates/codemaid-corpus/src/naming.rs
> Stable ids, short labels and sequence lanes.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_corpus__naming__Lane["Lane"] {
    <<struct>>
    +id: SymbolId
    +alias: String
    +prefix: String
    +external: bool
  }
  class codemaid_corpus__naming["codemaid_corpus::naming"] {
    <<module>>
    ~alias_for(crate) String
    ~assert_unique_idents(crate)
    ~ident(crate) Ident
    ~lane_of(crate) Lane
  }
  class codemaid_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/codemaid-corpus/src/lib.rs>>
  }
  class codemaid_mermaid__escape__Ident["Ident"] {
    <<struct in crates/codemaid-mermaid/src/escape.rs>>
  }
  class codemaid_model__codebase__Codebase["Codebase"] {
    <<struct in crates/codemaid-model/src/codebase.rs>>
  }
  class codemaid_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  codemaid_corpus__naming ..> codemaid_corpus__CorpusOptions
  codemaid_corpus__naming ..> codemaid_corpus__naming__Lane
  codemaid_corpus__naming ..> codemaid_mermaid__escape__Ident
  codemaid_corpus__naming ..> codemaid_model__codebase__Codebase
  codemaid_corpus__naming ..> codemaid_model__symbol__SymbolId
  codemaid_corpus__naming__Lane *-- codemaid_model__symbol__SymbolId : id
```

## `codemaid_corpus::naming::ident`
`pub(crate) fn ident(id: &SymbolId) -> Ident` · L8-L11
> Diagram id for a symbol (stable across all documents).
```mermaid
sequenceDiagram
  participant codemaid_corpus__naming as naming mod
  participant codemaid_model__symbol__SymbolId as SymbolId
  participant codemaid_mermaid__escape__Ident as Ident
  codemaid_corpus__naming->>codemaid_model__symbol__SymbolId: as_str()
  codemaid_corpus__naming->>codemaid_mermaid__escape__Ident: Ident::from_path(as_str())
```

## `codemaid_corpus::naming::assert_unique_idents`
`pub(crate) fn assert_unique_idents(cb: &Codebase)` · L13-L30
> Every symbol and relation endpoint must get its own diagram id: a shared id would silently merge two nodes in every diagram both appear in, and break merge-by-…
```mermaid
sequenceDiagram
  participant codemaid_corpus__naming as naming mod
  loop for id in ids
    codemaid_corpus__naming->>codemaid_corpus__naming: ident(id)
    opt let Some(prev) = seen.insert(ident(id), id)
      codemaid_corpus__naming->>codemaid_corpus__naming: ident(id)
    end
  end
```

## `codemaid_corpus::naming::lane_of`
`pub(crate) fn lane_of(cb: &Codebase, target: &SymbolId, opts: &CorpusOptions) -> Lane` · L41-L69
```mermaid
sequenceDiagram
  participant codemaid_corpus__naming as naming mod
  participant codemaid_model__codebase__Codebase as Codebase
  participant codemaid_model__symbol__SymbolId as SymbolId
  codemaid_corpus__naming->>codemaid_model__codebase__Codebase: is_internal(target)
  opt cb.is_internal(target)
    codemaid_corpus__naming->>codemaid_model__codebase__Codebase: owner_of(target)
    codemaid_corpus__naming->>codemaid_corpus__naming: alias_for(cb, &owner)
    Note over codemaid_corpus__naming: return Lane { alias: alias_for(cb, &owner), id: owner, …
  end
  codemaid_corpus__naming->>codemaid_model__symbol__SymbolId: as_str()
  opt s.starts_with(#quot;?::#quot;)
    codemaid_corpus__naming->>codemaid_model__symbol__SymbolId: SymbolId::new(#quot;unresolved#quot;)
    Note over codemaid_corpus__naming: return Lane { id: SymbolId::new(#quot;unresolved#quot;), alias: #quot;…
  end
  codemaid_corpus__naming->>codemaid_model__symbol__SymbolId: parent()
  opt let Some(parent) = target.parent()
    codemaid_corpus__naming->>codemaid_model__codebase__Codebase: is_internal(&parent)
    opt cb.is_internal(&parent)
      codemaid_corpus__naming->>codemaid_corpus__naming: alias_for(cb, &parent)
      Note over codemaid_corpus__naming: return Lane { alias: alias_for(cb, &parent), id: parent…
    end
  end
  alt ExternalLanes::CrateRoot
    codemaid_corpus__naming->>codemaid_model__symbol__SymbolId: SymbolId::new(root)
  else ExternalLanes::Owner
    codemaid_corpus__naming->>codemaid_model__symbol__SymbolId: parent()
  end
```

## `codemaid_corpus::naming::alias_for`
`pub(crate) fn alias_for(cb: &Codebase, id: &SymbolId) -> String` · L71-L78
> Alias for an internal lane: type name, or module name for modules.
```mermaid
sequenceDiagram
  participant codemaid_corpus__naming as naming mod
  participant codemaid_model__codebase__Codebase as Codebase
  participant codemaid_model__symbol__SymbolId as SymbolId
  codemaid_corpus__naming->>codemaid_model__codebase__Codebase: symbol(id)
  opt None
    codemaid_corpus__naming->>codemaid_model__symbol__SymbolId: name()
  end
```
