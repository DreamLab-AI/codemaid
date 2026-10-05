---
sealmap: 1
source: crates/sealmap-corpus/src/naming.rs
module: sealmap_corpus::naming
language: rust
source_hash: blake3:7c3019181c8e6ee8fb5223ae735613e67d81f35b0b5c9a9f4a11c6624452a584
lines: 78
fragments: 5
---
# `sealmap_corpus::naming` · crates/sealmap-corpus/src/naming.rs
> Stable ids, short labels and sequence lanes.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__naming__Lane["Lane"] {
    <<struct>>
    +id: SymbolId
    +alias: String
    +prefix: String
    +external: bool
  }
  class sealmap_corpus__naming["sealmap_corpus::naming"] {
    <<module>>
    ~alias_for(crate) String
    ~assert_unique_idents(crate)
    ~ident(crate) Ident
    ~lane_of(crate) Lane
  }
  class sealmap_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_mermaid__escape__Ident["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_corpus__naming ..> sealmap_corpus__CorpusOptions
  sealmap_corpus__naming ..> sealmap_corpus__naming__Lane
  sealmap_corpus__naming ..> sealmap_mermaid__escape__Ident
  sealmap_corpus__naming ..> sealmap_model__codebase__Codebase
  sealmap_corpus__naming ..> sealmap_model__symbol__SymbolId
  sealmap_corpus__naming__Lane *-- sealmap_model__symbol__SymbolId : id
```

## `sealmap_corpus::naming::ident`
`pub(crate) fn ident(id: &SymbolId) -> Ident` · L8-L11
> Diagram id for a symbol (stable across all documents).
```mermaid
sequenceDiagram
  participant sealmap_corpus__naming as naming mod
  participant sealmap_model__symbol__SymbolId as SymbolId
  participant sealmap_mermaid__escape__Ident as Ident
  sealmap_corpus__naming->>sealmap_model__symbol__SymbolId: as_str()
  sealmap_corpus__naming->>sealmap_mermaid__escape__Ident: Ident::from_path(as_str())
```

## `sealmap_corpus::naming::assert_unique_idents`
`pub(crate) fn assert_unique_idents(cb: &Codebase)` · L13-L30
> Every symbol and relation endpoint must get its own diagram id: a shared id would silently merge two nodes in every diagram both appear in, and break merge-by-…
```mermaid
sequenceDiagram
  participant sealmap_corpus__naming as naming mod
  loop for id in ids
    sealmap_corpus__naming->>sealmap_corpus__naming: ident(id)
    opt let Some(prev) = seen.insert(ident(id), id)
      sealmap_corpus__naming->>sealmap_corpus__naming: ident(id)
    end
  end
```

## `sealmap_corpus::naming::lane_of`
`pub(crate) fn lane_of(cb: &Codebase, target: &SymbolId, opts: &CorpusOptions) -> Lane` · L41-L69
```mermaid
sequenceDiagram
  participant sealmap_corpus__naming as naming mod
  participant sealmap_model__codebase__Codebase as Codebase
  participant sealmap_model__symbol__SymbolId as SymbolId
  sealmap_corpus__naming->>sealmap_model__codebase__Codebase: is_internal(target)
  opt cb.is_internal(target)
    sealmap_corpus__naming->>sealmap_model__codebase__Codebase: owner_of(target)
    sealmap_corpus__naming->>sealmap_corpus__naming: alias_for(cb, &owner)
    Note over sealmap_corpus__naming: return Lane { alias: alias_for(cb, &owner), id: owner, …
  end
  sealmap_corpus__naming->>sealmap_model__symbol__SymbolId: as_str()
  opt s.starts_with(#quot;?::#quot;)
    sealmap_corpus__naming->>sealmap_model__symbol__SymbolId: SymbolId::new(#quot;unresolved#quot;)
    Note over sealmap_corpus__naming: return Lane { id: SymbolId::new(#quot;unresolved#quot;), alias: #quot;…
  end
  sealmap_corpus__naming->>sealmap_model__symbol__SymbolId: parent()
  opt let Some(parent) = target.parent()
    sealmap_corpus__naming->>sealmap_model__codebase__Codebase: is_internal(&parent)
    opt cb.is_internal(&parent)
      sealmap_corpus__naming->>sealmap_corpus__naming: alias_for(cb, &parent)
      Note over sealmap_corpus__naming: return Lane { alias: alias_for(cb, &parent), id: parent…
    end
  end
  alt ExternalLanes::CrateRoot
    sealmap_corpus__naming->>sealmap_model__symbol__SymbolId: SymbolId::new(root)
  else ExternalLanes::Owner
    sealmap_corpus__naming->>sealmap_model__symbol__SymbolId: parent()
  end
```

## `sealmap_corpus::naming::alias_for`
`pub(crate) fn alias_for(cb: &Codebase, id: &SymbolId) -> String` · L71-L78
> Alias for an internal lane: type name, or module name for modules.
```mermaid
sequenceDiagram
  participant sealmap_corpus__naming as naming mod
  participant sealmap_model__codebase__Codebase as Codebase
  participant sealmap_model__symbol__SymbolId as SymbolId
  sealmap_corpus__naming->>sealmap_model__codebase__Codebase: symbol(id)
  opt None
    sealmap_corpus__naming->>sealmap_model__symbol__SymbolId: name()
  end
```
