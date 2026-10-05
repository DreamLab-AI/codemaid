---
sealmap: 2
source: crates/sealmap-model/src/codebase.rs
module: "sym:cargo sealmap_model . codebase/"
language: rust
source_hash: blake3:c3fb4016216914930c25ac1de353dd9091132328e124552d54d163a4420e1836
lines: 205
fragments: 6
---
# `sym:cargo sealmap_model . codebase/` · crates/sealmap-model/src/codebase.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct>>
    +schema_version: u32
    +name: String
    +files: BTreeMap#lt;SourcePath, SourceFile#gt;
    +symbols: BTreeMap#lt;SymbolId, Symbol#gt;
    +relations: BTreeSet#lt;Relation#gt;
    +add_file(&mut self, file: SourceFile)
    +add_relation(&mut self, relation: Relation)
    +add_symbol(&mut self, symbol: Symbol)
    +children(&'a self, id: &'a SymbolId) impl Iterator#lt;Item = &'a Symbol#gt;
    +file(&self, path: &SourcePath) Option#lt;&SourceFile#gt;
    +from_json(text: &str) Result#lt;Self, ModelJsonError#gt;
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
  class sealmap_model__codebase___tCodebaseStats["CodebaseStats"] {
    <<struct>>
    +files: usize
    +symbols: usize
    +relations: usize
    +flows: usize
    +calls: usize
  }
  class sealmap_model__codebase___tModelJsonError["ModelJsonError"] {
    <<enum>>
    Json#40;serde_json::Error#41;
    Version#123; #35;[doc = #quot; The version found#40;#96;schema_version#96;, or v1's…
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__source___tSourceFile["SourceFile"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__symbol___tRelation["Relation"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol___tRelationKind["RelationKind"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol___tSymbol["Symbol"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol___tSymbolKind["SymbolKind"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  class _serde_json__Error["serde_json::Error"] {
    <<external>>
  }
  sealmap_model__codebase___tCodebase ..> sealmap_model__codebase___tCodebaseStats
  sealmap_model__codebase___tCodebase ..> sealmap_model__codebase___tModelJsonError
  sealmap_model__codebase___tCodebase o-- sealmap_model__path___tSourcePath : files
  sealmap_model__codebase___tCodebase o-- sealmap_model__source___tSourceFile : files
  sealmap_model__codebase___tCodebase o-- sealmap_model__sym___tSymbolId : symbols
  sealmap_model__codebase___tCodebase o-- sealmap_model__symbol___tRelation : relations
  sealmap_model__codebase___tCodebase ..> sealmap_model__symbol___tRelationKind
  sealmap_model__codebase___tCodebase o-- sealmap_model__symbol___tSymbol : symbols
  sealmap_model__codebase___tCodebase ..> sealmap_model__symbol___tSymbolKind
  sealmap_model__codebase___tModelJsonError *-- _serde_json__Error : Json
```

## `sym:cargo sealmap_model . codebase/Codebase#from_json().`
`pub fn from_json(text: &str) -> Result<Self, ModelJsonError>` · L35-L59
> Read a model serialised as JSON, refusing any schema version other than [`MODEL_SCHEMA_VERSION`](crate::MODEL_SCHEMA_VERSION).
```mermaid
sequenceDiagram
  participant sealmap_model__codebase___tCodebase as Codebase
  participant _serde_json as serde_json ext
  sealmap_model__codebase___tCodebase->>_serde_json: serde_json::from_str(text)
  opt found != Some(crate::MODEL_SCHEMA_VERSION)
    Note over sealmap_model__codebase___tCodebase: return Err(ModelJsonError::Version { found, expected: c…
  end
  sealmap_model__codebase___tCodebase->>_serde_json: serde_json::from_str(text)
```

## `sym:cargo sealmap_model . codebase/Codebase#add_symbol().`
`pub fn add_symbol(&mut self, symbol: Symbol)` · L66-L87
> Add a symbol.
```mermaid
sequenceDiagram
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_model__hash___tFingerprint as Fingerprint
  opt Some(existing)
    sealmap_model__codebase___tCodebase->>sealmap_model__hash___tFingerprint: ~merge(symbol.sig_hash)
    sealmap_model__codebase___tCodebase->>sealmap_model__hash___tFingerprint: ~merge(symbol.body_hash)
  end
```

## `sym:cargo sealmap_model . codebase/Codebase#relations_from().`
`pub fn relations_from<'a>(&'a self, id: &'a SymbolId) -> impl Iterator<Item = &'a Relation>` · L126-L133
> Relations whose source is `id`.
```mermaid
sequenceDiagram
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__symbol___tRelation as Relation
  sealmap_model__codebase___tCodebase->>sealmap_model__sym___tSymbolId: SymbolId::min_value()
  sealmap_model__codebase___tCodebase->>sealmap_model__symbol___tRelation: Relation::new(clone(), min_value(), Contains, Exact)
```

## `sym:cargo sealmap_model . codebase/Codebase#owner_of().`
`pub fn owner_of(&self, id: &SymbolId) -> Option<SymbolId>` · L145-L156
> The innermost *type* that owns `id` (the type for a method), or the module for free items.
```mermaid
sequenceDiagram
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_model__sym___tSymbolId as SymbolId
  opt None
    sealmap_model__codebase___tCodebase->>sealmap_model__sym___tSymbolId: parent()
  end
```

## `sym:cargo sealmap_model . codebase/Codebase#stats().`
`pub fn stats(&self) -> CodebaseStats` · L158-L173
> Summary counts.
```mermaid
sequenceDiagram
  participant sealmap_model__codebase___tCodebase as Codebase
  participant _sealmap_model as sealmap_model ext
  participant sealmap_model__flow___tFlow as Flow
  sealmap_model__codebase___tCodebase->>_sealmap_model: ~CodebaseStats::CodebaseStats::default()
  loop for sym in self.symbols.values()
    opt let Some(flow) = &sym.flow
      sealmap_model__codebase___tCodebase->>sealmap_model__flow___tFlow: ~call_count()
    end
  end
```
