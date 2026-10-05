---
sealmap: 2
source: crates/sealmap-corpus/src/naming.rs
module: "sym:cargo sealmap_corpus . naming/"
language: rust
source_hash: blake3:29958f2c506c8d599f2296e29b0a4220209a8488faf9393efa065f672d8c7785
lines: 80
fragments: 5
---
# `sym:cargo sealmap_corpus . naming/` · crates/sealmap-corpus/src/naming.rs
> Stable ids, short labels and sequence lanes.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__naming___tLane["Lane"] {
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
  class sealmap_corpus___tCorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_mermaid__escape___tIdent["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  sealmap_corpus__naming ..> sealmap_corpus___tCorpusOptions
  sealmap_corpus__naming ..> sealmap_corpus__naming___tLane
  sealmap_corpus__naming ..> sealmap_mermaid__escape___tIdent
  sealmap_corpus__naming ..> sealmap_model__codebase___tCodebase
  sealmap_corpus__naming ..> sealmap_model__sym___tSymbolId
  sealmap_corpus__naming___tLane *-- sealmap_model__sym___tSymbolId : id
```

## `sym:cargo sealmap_corpus . naming/ident().`
`pub(crate) fn ident(id: &SymbolId) -> Ident` · L8-L12
> Diagram id for a symbol (stable across all documents), derived from the id's structure by [`Ident::from_symbol`].
```mermaid
sequenceDiagram
  participant sealmap_corpus__naming as naming mod
  participant sealmap_mermaid__escape___tIdent as Ident
  sealmap_corpus__naming->>sealmap_mermaid__escape___tIdent: Ident::from_symbol(id)
```

## `sym:cargo sealmap_corpus . naming/assert_unique_idents().`
`pub(crate) fn assert_unique_idents(cb: &Codebase)` · L14-L31
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

## `sym:cargo sealmap_corpus . naming/lane_of().`
`pub(crate) fn lane_of(cb: &Codebase, target: &SymbolId, opts: &CorpusOptions) -> Lane` · L42-L71
```mermaid
sequenceDiagram
  participant sealmap_corpus__naming as naming mod
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_corpus__naming->>sealmap_model__codebase___tCodebase: is_internal(target)
  opt cb.is_internal(target)
    sealmap_corpus__naming->>sealmap_model__codebase___tCodebase: owner_of(target)
    sealmap_corpus__naming->>sealmap_corpus__naming: alias_for(cb, &owner)
    Note over sealmap_corpus__naming: return Lane { alias: alias_for(cb, &owner), id: owner, …
  end
  sealmap_corpus__naming->>sealmap_model__sym___tSymbolId: is_unresolved()
  opt target.is_unresolved()
    sealmap_corpus__naming->>sealmap_model__sym___tSymbolId: SymbolId::unresolved(#quot;#quot;)
    Note over sealmap_corpus__naming: return Lane { id: SymbolId::unresolved(#quot;#quot;), alias: #quot;?#quot;.…
  end
  sealmap_corpus__naming->>sealmap_model__sym___tSymbolId: parent()
  opt let Some(parent) = target.parent()
    sealmap_corpus__naming->>sealmap_model__codebase___tCodebase: is_internal(&parent)
    opt cb.is_internal(&parent)
      sealmap_corpus__naming->>sealmap_corpus__naming: alias_for(cb, &parent)
      Note over sealmap_corpus__naming: return Lane { alias: alias_for(cb, &parent), id: parent…
    end
  end
  sealmap_corpus__naming->>sealmap_model__sym___tSymbolId: names()
  alt ExternalLanes::CrateRoot
    sealmap_corpus__naming->>sealmap_model__sym___tSymbolId: SymbolId::path(_)
    opt via unwrap_or_else
      sealmap_corpus__naming->>sealmap_model__sym___tSymbolId: SymbolId::unresolved(#quot;#quot;)
    end
  else ExternalLanes::Owner
    sealmap_corpus__naming->>sealmap_model__sym___tSymbolId: parent()
  end
```

## `sym:cargo sealmap_corpus . naming/alias_for().`
`pub(crate) fn alias_for(cb: &Codebase, id: &SymbolId) -> String` · L73-L80
> Alias for an internal lane: type name, or module name for modules.
```mermaid
sequenceDiagram
  participant sealmap_corpus__naming as naming mod
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_corpus__naming->>sealmap_model__codebase___tCodebase: symbol(id)
  opt None
    sealmap_corpus__naming->>sealmap_model__sym___tSymbolId: name()
  end
```
