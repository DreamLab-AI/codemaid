---
sealmap: 1
source: crates/sealmap-frontend/src/ids.rs
module: sealmap_frontend::ids
language: rust
source_hash: blake3:e429a98e16fea80b8851d70c5600e05f897edf151a1015d3fba6605a9358c968
lines: 84
fragments: 7
---
# `sealmap_frontend::ids` · crates/sealmap-frontend/src/ids.rs
> The symbol-id builder.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_frontend__ids["sealmap_frontend::ids"] {
    <<module>>
    +const SEP: &str
    +item_id(module: &str, name: &str) SymbolId
    +method_id(owner: &SymbolId, trait_segment: Option#lt;&str#gt;, name: &str) SymbolId
    +module_id(segments: &[String]) SymbolId
    +module_path(segments: &[String]) String
    +parent_module_id(segments: &[String]) Option#lt;SymbolId#gt;
    +path_id(segments: &[String]) SymbolId
    +trait_impl_segment(trait_display: &str) String
    +unresolved_method_id(name: &str) SymbolId
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_frontend__ids ..> sealmap_model__symbol__SymbolId
```

## `sealmap_frontend::ids::module_id`
`pub fn module_id(segments: &[String]) -> SymbolId` · L29-L32
> The id of the module at `segments`.
```mermaid
sequenceDiagram
  participant sealmap_frontend__ids as ids mod
  sealmap_frontend__ids->>sealmap_frontend__ids: path_id(segments)
```

## `sealmap_frontend::ids::path_id`
`pub fn path_id(segments: &[String]) -> SymbolId` · L34-L38
> The id spelled by a path as written (`["serde_json", "to_string"]` → `serde_json::to_string`); used for targets outside the analysed code.
```mermaid
sequenceDiagram
  participant sealmap_frontend__ids as ids mod
  participant sealmap_model__symbol__SymbolId as SymbolId
  sealmap_frontend__ids->>sealmap_frontend__ids: module_path(segments)
  sealmap_frontend__ids->>sealmap_model__symbol__SymbolId: SymbolId::new(module_path())
```

## `sealmap_frontend::ids::parent_module_id`
`pub fn parent_module_id(segments: &[String]) -> Option<SymbolId>` · L40-L43
> The id of the enclosing module of the module at `segments`, if it has one.
```mermaid
sequenceDiagram
  participant sealmap_frontend__ids as ids mod
  opt via then
    sealmap_frontend__ids->>sealmap_frontend__ids: module_id(&_)
  end
```

## `sealmap_frontend::ids::item_id`
`pub fn item_id(module: &str, name: &str) -> SymbolId` · L45-L48
> The id of item `name` declared in `module` (a [`module_path()`]).
```mermaid
sequenceDiagram
  participant sealmap_frontend__ids as ids mod
  participant sealmap_model__symbol__SymbolId as SymbolId
  sealmap_frontend__ids->>sealmap_model__symbol__SymbolId: SymbolId::new(_)
```

## `sealmap_frontend::ids::method_id`
`pub fn method_id(owner: &SymbolId, trait_segment: Option<&str>, name: &str) -> SymbolId` · L57-L64
> The id of method `name` on `owner`, under a [`trait_impl_segment`] when it implements a trait.
```mermaid
sequenceDiagram
  participant sealmap_frontend__ids as ids mod
  participant sealmap_model__symbol__SymbolId as SymbolId
  alt Some(t)
    sealmap_frontend__ids->>sealmap_model__symbol__SymbolId: child(t)
  else None
    sealmap_frontend__ids->>sealmap_model__symbol__SymbolId: child(name)
  end
```

## `sealmap_frontend::ids::unresolved_method_id`
`pub fn unresolved_method_id(name: &str) -> SymbolId` · L66-L70
> The placeholder target of a method call whose receiver could not be resolved (`?::name`).
```mermaid
sequenceDiagram
  participant sealmap_frontend__ids as ids mod
  participant sealmap_model__symbol__SymbolId as SymbolId
  sealmap_frontend__ids->>sealmap_model__symbol__SymbolId: SymbolId::new(_)
```
