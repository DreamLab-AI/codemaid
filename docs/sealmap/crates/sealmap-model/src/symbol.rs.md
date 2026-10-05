---
sealmap: 2
source: crates/sealmap-model/src/symbol.rs
module: "sym:cargo sealmap_model . symbol/"
language: rust
source_hash: blake3:5ae15b00bf188cda3635a124ea517fd55f84095749187840d4d4e3e790ae838d
lines: 341
fragments: 3
---
# `sym:cargo sealmap_model . symbol/` · crates/sealmap-model/src/symbol.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__symbol___tConfidence["Confidence"] {
    <<enum>>
    Exact
    Inferred
    External
    +marker(self) char
  }
  class sealmap_model__symbol___tMember["Member"] {
    <<struct>>
    +name: String
    +kind: MemberKind
    +ty: Option#lt;String#gt;
    +visibility: Visibility
    +span: Span
    +refs: Vec#lt;SymbolId#gt;
  }
  class sealmap_model__symbol___tMemberKind["MemberKind"] {
    <<enum>>
    Field
    Variant
    AssocConst
    AssocType
    RequiredMethod
  }
  class sealmap_model__symbol___tRelation["Relation"] {
    <<struct>>
    +from: SymbolId
    +to: SymbolId
    +kind: RelationKind
    +confidence: Confidence
    +new(from: SymbolId, to: SymbolId, kind: RelationKind, confidence: Confidence) Self
  }
  class sealmap_model__symbol___tRelationKind["RelationKind"] {
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
  class sealmap_model__symbol___tSpan["Span"] {
    <<struct>>
    +start_line: u32
    +start_col: u32
    +end_line: u32
    +end_col: u32
    +compact(&self) String
    +new(start_line: u32, start_col: u32, end_line: u32, end_col: u32) Self
  }
  class sealmap_model__symbol___tSymbol["Symbol"] {
    <<struct>>
    +id: SymbolId
    +name: String
    +kind: SymbolKind
    +visibility: Visibility
    +file: SourcePath
    +span: Span
    +sig_hash: Fingerprint
    +body_hash: Fingerprint
    +parent: Option#lt;SymbolId#gt;
    +signature: Option#lt;String#gt;
    +doc: Option#lt;String#gt;
    +generics: Vec#lt;String#gt;
    +tags: Vec#lt;String#gt;
    +members: Vec#lt;Member#gt;
    +flow: Option#lt;Flow#gt;
    +new(id: SymbolId, name: impl Into#lt;String#gt;, kind: SymbolKind, file: SourcePath) Self
  }
  class sealmap_model__symbol___tSymbolKind["SymbolKind"] {
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
    +descriptor(self, name: impl Into#lt;String#gt;) Descriptor
    +is_callable(self) bool
    +is_type(self) bool
    +keyword(self) &'static str
  }
  class sealmap_model__symbol___tVisibility["Visibility"] {
    <<enum>>
    Public
    Crate
    Restricted#40;String#41;
    Private
    +uml_marker(&self) char
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__flow___tFlow["Flow"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__hash___tFingerprint["Fingerprint"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__sym___tDescriptor["Descriptor"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  sealmap_model__symbol___tMember o-- sealmap_model__sym___tSymbolId : refs
  sealmap_model__symbol___tMember *-- sealmap_model__symbol___tMemberKind : kind
  sealmap_model__symbol___tMember *-- sealmap_model__symbol___tSpan : span
  sealmap_model__symbol___tMember *-- sealmap_model__symbol___tVisibility : visibility
  sealmap_model__symbol___tRelation *-- sealmap_model__sym___tSymbolId : from, to
  sealmap_model__symbol___tRelation *-- sealmap_model__symbol___tConfidence : confidence
  sealmap_model__symbol___tRelation *-- sealmap_model__symbol___tRelationKind : kind
  sealmap_model__symbol___tSymbol o-- sealmap_model__flow___tFlow : flow
  sealmap_model__symbol___tSymbol *-- sealmap_model__hash___tFingerprint : sig_hash, body_hash
  sealmap_model__symbol___tSymbol *-- sealmap_model__path___tSourcePath : file
  sealmap_model__symbol___tSymbol o-- sealmap_model__sym___tSymbolId : id, parent
  sealmap_model__symbol___tSymbol o-- sealmap_model__symbol___tMember : members
  sealmap_model__symbol___tSymbol *-- sealmap_model__symbol___tSpan : span
  sealmap_model__symbol___tSymbol *-- sealmap_model__symbol___tSymbolKind : kind
  sealmap_model__symbol___tSymbol *-- sealmap_model__symbol___tVisibility : visibility
  sealmap_model__symbol___tSymbolKind ..> sealmap_model__sym___tDescriptor
```

## `sym:cargo sealmap_model . symbol/SymbolKind#descriptor().`
`pub fn descriptor(self, name: impl Into<String>) -> Descriptor` · L52-L69
> The `sym:` descriptor for a symbol of this kind called `name`: a namespace for modules, a type for structs, enums, unions, traits and aliases, a method for fun…
```mermaid
sequenceDiagram
  participant sealmap_model__symbol___tSymbolKind as SymbolKind
  participant sealmap_model__sym___tDescriptor as Descriptor
  alt Self::Module
    sealmap_model__symbol___tSymbolKind->>sealmap_model__sym___tDescriptor: Descriptor::namespace(name)
  else Self::Struct | Self::Enum | Self::Union | Self::Trait |…
    sealmap_model__symbol___tSymbolKind->>sealmap_model__sym___tDescriptor: Descriptor::r#35;type(name)
  else Self::Function | Self::Method
    sealmap_model__symbol___tSymbolKind->>sealmap_model__sym___tDescriptor: Descriptor::method(name)
  else Self::Const | Self::Static
    sealmap_model__symbol___tSymbolKind->>sealmap_model__sym___tDescriptor: Descriptor::term(name)
  else Self::Macro
    sealmap_model__symbol___tSymbolKind->>sealmap_model__sym___tDescriptor: Descriptor::r#35;macro(name)
  end
```

## `sym:cargo sealmap_model . symbol/Symbol#new().`
`pub fn new(id: SymbolId, name: impl Into<String>, kind: SymbolKind, file: SourcePath) -> Self` · L232-L253
> A symbol with only the required fields set; everything else empty / private, and both fingerprints unset ([`Fingerprint::is_unset`]).
```mermaid
sequenceDiagram
  participant sealmap_model__symbol___tSymbol as Symbol
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant _sealmap_model as sealmap_model ext
  sealmap_model__symbol___tSymbol->>sealmap_model__sym___tSymbolId: parent()
  sealmap_model__symbol___tSymbol->>_sealmap_model: ~Span::Span::default()
  sealmap_model__symbol___tSymbol->>_sealmap_model: ~Fingerprint::Fingerprint::default()
  sealmap_model__symbol___tSymbol->>_sealmap_model: ~Fingerprint::Fingerprint::default()
```
