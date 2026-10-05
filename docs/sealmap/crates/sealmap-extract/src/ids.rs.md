---
sealmap: 2
source: crates/sealmap-extract/src/ids.rs
module: "sym:cargo sealmap_extract . ids/"
language: rust
source_hash: blake3:83a463b8d0a002d72d1c92cedead9b73ec1916f8ff6d89565d454665e29cc2d6
lines: 153
fragments: 8
---
# `sym:cargo sealmap_extract . ids/` · crates/sealmap-extract/src/ids.rs
> The symbol-id builder.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_extract__ids["sealmap_extract::ids"] {
    <<module>>
    +impl_method_id(module: &SymbolId, owner: &SymbolId, self_ty: &str, trait_: Option#lt;&str#gt;, name: &str,) SymbolId
    +item_id(scope: &SymbolId, kind: SymbolKind, name: &str) SymbolId
    +method_id(owner: &SymbolId, trait_: Option#lt;&str#gt;, name: &str) SymbolId
    +module_id(package: &Package, modules: &[String]) SymbolId
    +package(manager: &str, name: &str) Package
    +path_id(segments: &[String]) Option#lt;SymbolId#gt;
    +unresolved_method_id(name: &str) SymbolId
  }
  class sealmap_model__sym___tPackage["Package"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__symbol___tSymbolKind["SymbolKind"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_extract__ids ..> sealmap_model__sym___tPackage
  sealmap_extract__ids ..> sealmap_model__sym___tSymbolId
  sealmap_extract__ids ..> sealmap_model__symbol___tSymbolKind
```

## `sym:cargo sealmap_extract . ids/package().`
`pub fn package(manager: &str, name: &str) -> Package` · L52-L64
> The package for `name` under `manager` (`cargo`, `npm`) at the current tree.
```mermaid
sequenceDiagram
  participant sealmap_extract__ids as ids mod
  participant sealmap_model__sym___tPackage as Package
  sealmap_extract__ids->>sealmap_model__sym___tPackage: Package::current(manager, name)
  opt via unwrap_or_else
    sealmap_extract__ids->>sealmap_model__sym___tPackage: Package::current(manager, cleaned)
    opt via or_else
      sealmap_extract__ids->>sealmap_model__sym___tPackage: Package::current(#quot;unknown#quot;, #quot;_#quot;)
    end
  end
```

## `sym:cargo sealmap_extract . ids/module_id().`
`pub fn module_id(package: &Package, modules: &[String]) -> SymbolId` · L66-L70
> The id of the module at `modules` (the path below the package root; empty for the root itself).
```mermaid
sequenceDiagram
  participant sealmap_extract__ids as ids mod
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_extract__ids->>sealmap_model__sym___tSymbolId: SymbolId::global(clone(), collect())
```

## `sym:cargo sealmap_extract . ids/item_id().`
`pub fn item_id(scope: &SymbolId, kind: SymbolKind, name: &str) -> SymbolId` · L72-L77
> The id of item `name` of `kind` declared in `scope` (a module, or a type for associated items).
```mermaid
sequenceDiagram
  participant sealmap_extract__ids as ids mod
  participant sealmap_model__symbol___tSymbolKind as SymbolKind
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_extract__ids->>sealmap_model__symbol___tSymbolKind: descriptor(name)
  sealmap_extract__ids->>sealmap_model__sym___tSymbolId: child(descriptor())
  opt via unwrap_or_else
    sealmap_extract__ids->>sealmap_model__sym___tSymbolId: extend_path(name)
  end
```

## `sym:cargo sealmap_extract . ids/method_id().`
`pub fn method_id(owner: &SymbolId, trait_: Option<&str>, name: &str) -> SymbolId` · L79-L88
> The id of method `name` owned by `owner` (a type or trait), under a `[Trait]` descriptor when it implements `trait_` (spelled as written).
```mermaid
sequenceDiagram
  participant sealmap_extract__ids as ids mod
  participant sealmap_model__sym___tDescriptor as Descriptor
  participant sealmap_model__sym___tSymbolId as SymbolId
  opt Some(t)
    sealmap_extract__ids->>sealmap_model__sym___tDescriptor: Descriptor::type_parameter(t)
    sealmap_extract__ids->>sealmap_model__sym___tSymbolId: child(type_parameter())
  end
  opt via and_then
    sealmap_extract__ids->>sealmap_model__sym___tDescriptor: Descriptor::method(name)
    sealmap_extract__ids->>sealmap_model__sym___tSymbolId: ~child(method())
  end
  opt via unwrap_or_else
    sealmap_extract__ids->>sealmap_model__sym___tSymbolId: extend_path(name)
  end
```

## `sym:cargo sealmap_extract . ids/impl_method_id().`
`pub fn impl_method_id(module: &SymbolId, owner: &SymbolId, self_ty: &str, trait_: Option<&str>, name: &str,) -> SymbolId` · L90-L111
> The id of method `name` in an impl block found in `module`.
```mermaid
sequenceDiagram
  participant sealmap_extract__ids as ids mod
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym___tDescriptor as Descriptor
  sealmap_extract__ids->>sealmap_model__sym___tSymbolId: is_global()
  opt owner.is_global()
    sealmap_extract__ids->>sealmap_extract__ids: method_id(owner, trait_, name)
    Note over sealmap_extract__ids: return method_id(owner, trait_, name)
  end
  sealmap_extract__ids->>sealmap_model__sym___tDescriptor: Descriptor::r#35;type(#quot;impl#quot;)
  sealmap_extract__ids->>sealmap_model__sym___tDescriptor: Descriptor::type_parameter(self_ty)
  opt let Some(t) = trait_
    sealmap_extract__ids->>sealmap_model__sym___tDescriptor: Descriptor::type_parameter(t)
  end
  sealmap_extract__ids->>sealmap_model__sym___tDescriptor: Descriptor::method(name)
  loop each via try_fold
    sealmap_extract__ids->>sealmap_model__sym___tSymbolId: ~child(d)
  end
  opt via unwrap_or_else
    sealmap_extract__ids->>sealmap_model__sym___tSymbolId: extend_path(name)
  end
```

## `sym:cargo sealmap_extract . ids/path_id().`
`pub fn path_id(segments: &[String]) -> Option<SymbolId>` · L113-L118
> The id of a path as written whose kinds are unknown (`["serde_json", "to_string"]` → `sym:extern serde_json::to_string`).
```mermaid
sequenceDiagram
  participant sealmap_extract__ids as ids mod
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_extract__ids->>sealmap_model__sym___tSymbolId: SymbolId::path(cloned())
```

## `sym:cargo sealmap_extract . ids/unresolved_method_id().`
`pub fn unresolved_method_id(name: &str) -> SymbolId` · L120-L124
> The placeholder target of a method call whose receiver could not be resolved (`sym:? name`).
```mermaid
sequenceDiagram
  participant sealmap_extract__ids as ids mod
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_extract__ids->>sealmap_model__sym___tSymbolId: SymbolId::unresolved(name)
```
