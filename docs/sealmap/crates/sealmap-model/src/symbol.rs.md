---
sealmap: 1
source: crates/sealmap-model/src/symbol.rs
module: sealmap_model::symbol
language: rust
source_hash: blake3:d7083feceafa6aee4aeab8e26442d607db3df03640dadc7d384d23d833f59a8f
lines: 371
fragments: 2
---
# `sealmap_model::symbol` · crates/sealmap-model/src/symbol.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__symbol__Confidence["Confidence"] {
    <<enum>>
    Exact
    Inferred
    External
    +marker(self) char
  }
  class sealmap_model__symbol__Member["Member"] {
    <<struct>>
    +name: String
    +kind: MemberKind
    +ty: Option#lt;String#gt;
    +visibility: Visibility
    +span: Span
    +refs: Vec#lt;SymbolId#gt;
  }
  class sealmap_model__symbol__MemberKind["MemberKind"] {
    <<enum>>
    Field
    Variant
    AssocConst
    AssocType
    RequiredMethod
  }
  class sealmap_model__symbol__Relation["Relation"] {
    <<struct>>
    +from: SymbolId
    +to: SymbolId
    +kind: RelationKind
    +confidence: Confidence
    +new(from: SymbolId, to: SymbolId, kind: RelationKind, confidence: Confidence) Self
  }
  class sealmap_model__symbol__RelationKind["RelationKind"] {
    <<enum>>
    Contains
    Implements
    Extends
    FieldType
    Uses
    Imports
    Calls
    +label(self) &'static str
  }
  class sealmap_model__symbol__Span["Span"] {
    <<struct>>
    +start_line: u32
    +start_col: u32
    +end_line: u32
    +end_col: u32
    +compact(&self) String
    +new(start_line: u32, start_col: u32, end_line: u32, end_col: u32) Self
  }
  class sealmap_model__symbol__Symbol["Symbol"] {
    <<struct>>
    +id: SymbolId
    +name: String
    +kind: SymbolKind
    +visibility: Visibility
    +file: SourcePath
    +span: Span
    +parent: Option#lt;SymbolId#gt;
    +signature: Option#lt;String#gt;
    +doc: Option#lt;String#gt;
    +generics: Vec#lt;String#gt;
    +tags: Vec#lt;String#gt;
    +members: Vec#lt;Member#gt;
    +flow: Option#lt;Flow#gt;
    +new(id: SymbolId, name: impl Into#lt;String#gt;, kind: SymbolKind, file: SourcePath) Self
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct>>
    -0: String
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    +child(&self, segment: &str) SymbolId
    +name(&self) &str
    +new(path: impl Into#lt;String#gt;) Self
    +parent(&self) Option#lt;SymbolId#gt;
  }
  class sealmap_model__symbol__SymbolKind["SymbolKind"] {
    <<enum>>
    Module
    Struct
    Enum
    Union
    Trait
    TypeAlias
    Function
    Method
    Const
    Static
    Macro
    +is_callable(self) bool
    +is_type(self) bool
    +keyword(self) &'static str
  }
  class sealmap_model__symbol__Visibility["Visibility"] {
    <<enum>>
    Public
    Crate
    Restricted#40;String#41;
    Private
    +uml_marker(&self) char
  }
  class sealmap_model__flow__Flow["Flow"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__path__SourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  sealmap_model__symbol__Member *-- sealmap_model__symbol__MemberKind : kind
  sealmap_model__symbol__Member *-- sealmap_model__symbol__Span : span
  sealmap_model__symbol__Member o-- sealmap_model__symbol__SymbolId : refs
  sealmap_model__symbol__Member *-- sealmap_model__symbol__Visibility : visibility
  sealmap_model__symbol__Relation *-- sealmap_model__symbol__Confidence : confidence
  sealmap_model__symbol__Relation *-- sealmap_model__symbol__RelationKind : kind
  sealmap_model__symbol__Relation *-- sealmap_model__symbol__SymbolId : from, to
  sealmap_model__symbol__Symbol o-- sealmap_model__flow__Flow : flow
  sealmap_model__symbol__Symbol *-- sealmap_model__path__SourcePath : file
  sealmap_model__symbol__Symbol o-- sealmap_model__symbol__Member : members
  sealmap_model__symbol__Symbol *-- sealmap_model__symbol__Span : span
  sealmap_model__symbol__Symbol o-- sealmap_model__symbol__SymbolId : id, parent
  sealmap_model__symbol__Symbol *-- sealmap_model__symbol__SymbolKind : kind
  sealmap_model__symbol__Symbol *-- sealmap_model__symbol__Visibility : visibility
```

## `sealmap_model::symbol::Symbol::new`
`pub fn new(id: SymbolId, name: impl Into<String>, kind: SymbolKind, file: SourcePath) -> Self` · L265-L283
> A symbol with only the required fields set; everything else empty / private.
```mermaid
sequenceDiagram
  participant sealmap_model__symbol__Symbol as Symbol
  participant sealmap_model__symbol__SymbolId as SymbolId
  participant sealmap_model__symbol__Span as Span
  sealmap_model__symbol__Symbol->>sealmap_model__symbol__SymbolId: parent()
  sealmap_model__symbol__Symbol->>sealmap_model__symbol__Span: ~Span::default()
```
