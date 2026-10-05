---
codemaid: 1
source: crates/codemaid-model/src/symbol.rs
module: codemaid_model::symbol
language: rust
source_hash: blake3:0a8dcdac8c709c41215362e20445cf9bc313d3c6ce70facb727b85b0a5c650a1
lines: 371
fragments: 2
---
# `codemaid_model::symbol` · crates/codemaid-model/src/symbol.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_model__symbol__Confidence["Confidence"] {
    <<enum>>
    Exact
    Inferred
    External
    +marker(self) char
  }
  class codemaid_model__symbol__Member["Member"] {
    <<struct>>
    +name: String
    +kind: MemberKind
    +ty: Option#lt;String#gt;
    +visibility: Visibility
    +span: Span
    +refs: Vec#lt;SymbolId#gt;
  }
  class codemaid_model__symbol__MemberKind["MemberKind"] {
    <<enum>>
    Field
    Variant
    AssocConst
    AssocType
    RequiredMethod
  }
  class codemaid_model__symbol__Relation["Relation"] {
    <<struct>>
    +from: SymbolId
    +to: SymbolId
    +kind: RelationKind
    +confidence: Confidence
    +new(from: SymbolId, to: SymbolId, kind: RelationKind, confidence: Confidence) Self
  }
  class codemaid_model__symbol__RelationKind["RelationKind"] {
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
  class codemaid_model__symbol__Span["Span"] {
    <<struct>>
    +start_line: u32
    +start_col: u32
    +end_line: u32
    +end_col: u32
    +compact(&self) String
    +new(start_line: u32, start_col: u32, end_line: u32, end_col: u32) Self
  }
  class codemaid_model__symbol__Symbol["Symbol"] {
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
  class codemaid_model__symbol__SymbolId["SymbolId"] {
    <<struct>>
    -0: String
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    +child(&self, segment: &str) SymbolId
    +name(&self) &str
    +new(path: impl Into#lt;String#gt;) Self
    +parent(&self) Option#lt;SymbolId#gt;
  }
  class codemaid_model__symbol__SymbolKind["SymbolKind"] {
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
  class codemaid_model__symbol__Visibility["Visibility"] {
    <<enum>>
    Public
    Crate
    Restricted#40;String#41;
    Private
    +uml_marker(&self) char
  }
  class codemaid_model__flow__Flow["Flow"] {
    <<struct in crates/codemaid-model/src/flow.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  codemaid_model__symbol__Member *-- codemaid_model__symbol__MemberKind : kind
  codemaid_model__symbol__Member *-- codemaid_model__symbol__Span : span
  codemaid_model__symbol__Member o-- codemaid_model__symbol__SymbolId : refs
  codemaid_model__symbol__Member *-- codemaid_model__symbol__Visibility : visibility
  codemaid_model__symbol__Relation *-- codemaid_model__symbol__Confidence : confidence
  codemaid_model__symbol__Relation *-- codemaid_model__symbol__RelationKind : kind
  codemaid_model__symbol__Relation *-- codemaid_model__symbol__SymbolId : from, to
  codemaid_model__symbol__Symbol o-- codemaid_model__flow__Flow : flow
  codemaid_model__symbol__Symbol *-- codemaid_model__path__SourcePath : file
  codemaid_model__symbol__Symbol o-- codemaid_model__symbol__Member : members
  codemaid_model__symbol__Symbol *-- codemaid_model__symbol__Span : span
  codemaid_model__symbol__Symbol o-- codemaid_model__symbol__SymbolId : id, parent
  codemaid_model__symbol__Symbol *-- codemaid_model__symbol__SymbolKind : kind
  codemaid_model__symbol__Symbol *-- codemaid_model__symbol__Visibility : visibility
```

## `codemaid_model::symbol::Symbol::new`
`pub fn new(id: SymbolId, name: impl Into<String>, kind: SymbolKind, file: SourcePath) -> Self` · L265-L283
> A symbol with only the required fields set; everything else empty / private.
```mermaid
sequenceDiagram
  participant codemaid_model__symbol__Symbol as Symbol
  participant codemaid_model__symbol__SymbolId as SymbolId
  participant codemaid_model__symbol__Span as Span
  codemaid_model__symbol__Symbol->>codemaid_model__symbol__SymbolId: parent()
  codemaid_model__symbol__Symbol->>codemaid_model__symbol__Span: ~Span::default()
```
