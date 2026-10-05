---
codemaid: 1
source: crates/codemaid-rust/src/raw.rs
module: codemaid_rust::raw
language: rust
source_hash: blake3:11e364cdc131060af0839a78344d83f33ef02816fda20d4cc8ee8c61cbc71bf0
lines: 141
fragments: 1
---
# `codemaid_rust::raw` · crates/codemaid-rust/src/raw.rs
> Pass-1 output: plain, unresolved data extracted from one file.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_rust__raw__Callee["Callee"] {
    <<enum>>
    Path#40;Segs#41;
    Method#123; recv: Recv, name: String #125;
  }
  class codemaid_rust__raw__RawCall["RawCall"] {
    <<struct>>
    +callee: Callee
    +label: String
    +kind: CallKind
    +awaited: bool
    +fallible: bool
    +line: u32
  }
  class codemaid_rust__raw__RawFile["RawFile"] {
    <<struct>>
    +path: SourcePath
    +role: FileRole
    +text_lines: u32
    +hash: ContentHash
    +modules: Vec#lt;RawModule#gt;
    +items: Vec#lt;RawItem#gt;
    +impls: Vec#lt;RawImpl#gt;
    +error: Option#lt;String#gt;
  }
  class codemaid_rust__raw__RawFn["RawFn"] {
    <<struct>>
    +name: String
    +vis: Visibility
    +span: Span
    +signature: String
    +doc: Option#lt;String#gt;
    +generics: Vec#lt;String#gt;
    +tags: Vec#lt;String#gt;
    +sig_refs: Vec#lt;Segs#gt;
    +flow: Vec#lt;RawStep#gt;
  }
  class codemaid_rust__raw__RawImpl["RawImpl"] {
    <<struct>>
    +module: Segs
    +self_ty: Option#lt;Segs#gt;
    +trait_: Option#lt;#40;Segs, String#41;#gt;
    +methods: Vec#lt;RawFn#gt;
  }
  class codemaid_rust__raw__RawItem["RawItem"] {
    <<struct>>
    +module: Segs
    +name: String
    +kind: SymbolKind
    +vis: Visibility
    +span: Span
    +signature: Option#lt;String#gt;
    +doc: Option#lt;String#gt;
    +generics: Vec#lt;String#gt;
    +tags: Vec#lt;String#gt;
    +members: Vec#lt;RawMember#gt;
    +sig_refs: Vec#lt;Segs#gt;
    +supertraits: Vec#lt;Segs#gt;
    +methods: Vec#lt;RawFn#gt;
    +flow: Vec#lt;RawStep#gt;
  }
  class codemaid_rust__raw__RawMember["RawMember"] {
    <<struct>>
    +name: String
    +kind: MemberKind
    +ty: Option#lt;String#gt;
    +vis: Visibility
    +span: Span
    +refs: Vec#lt;Segs#gt;
  }
  class codemaid_rust__raw__RawModule["RawModule"] {
    <<struct>>
    +path: Segs
    +span: Span
    +doc: Option#lt;String#gt;
    +vis: Visibility
    +uses: Vec#lt;RawUse#gt;
    +tags: Vec#lt;String#gt;
  }
  class codemaid_rust__raw__RawStep["RawStep"] {
    <<enum>>
    Call#40;RawCall#41;
    Branch#40;Vec#lt;#40;String, Vec#lt;RawStep#gt;#41;#gt;#41;
    Loop#40;String, Vec#lt;RawStep#gt;#41;
    Optional#40;String, Vec#lt;RawStep#gt;#41;
    Parallel#40;Vec#lt;#40;String, Vec#lt;RawStep#gt;#41;#gt;#41;
    Return#40;String, u32#41;
  }
  class codemaid_rust__raw__RawUse["RawUse"] {
    <<struct>>
    +alias: String
    +target: Segs
  }
  class codemaid_rust__raw__Recv["Recv"] {
    <<enum>>
    SelfValue
    SelfField#40;String#41;
    Typed#40;Vec#lt;Segs#gt;#41;
    Untyped
    Unknown
  }
  class codemaid_rust__raw__Segs["Segs"] {
    <<type>>
  }
  class codemaid_model__flow__CallKind["CallKind"] {
    <<enum in crates/codemaid-model/src/flow.rs>>
  }
  class codemaid_model__hash__ContentHash["ContentHash"] {
    <<struct in crates/codemaid-model/src/hash.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class codemaid_rust__layout__FileRole["FileRole"] {
    <<struct in crates/codemaid-rust/src/layout.rs>>
  }
  class codemaid_model__symbol__Span["Span"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__Visibility["Visibility"] {
    <<enum in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__SymbolKind["SymbolKind"] {
    <<enum in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__MemberKind["MemberKind"] {
    <<enum in crates/codemaid-model/src/symbol.rs>>
  }
  codemaid_rust__raw__Callee *-- codemaid_rust__raw__Recv : Method
  codemaid_rust__raw__Callee *-- codemaid_rust__raw__Segs : Path
  codemaid_rust__raw__RawCall *-- codemaid_model__flow__CallKind : kind
  codemaid_rust__raw__RawCall *-- codemaid_rust__raw__Callee : callee
  codemaid_rust__raw__RawFile *-- codemaid_model__hash__ContentHash : hash
  codemaid_rust__raw__RawFile *-- codemaid_model__path__SourcePath : path
  codemaid_rust__raw__RawFile *-- codemaid_rust__layout__FileRole : role
  codemaid_rust__raw__RawFile o-- codemaid_rust__raw__RawImpl : impls
  codemaid_rust__raw__RawFile o-- codemaid_rust__raw__RawItem : items
  codemaid_rust__raw__RawFile o-- codemaid_rust__raw__RawModule : modules
  codemaid_rust__raw__RawFn *-- codemaid_model__symbol__Span : span
  codemaid_rust__raw__RawFn *-- codemaid_model__symbol__Visibility : vis
  codemaid_rust__raw__RawFn o-- codemaid_rust__raw__RawStep : flow
  codemaid_rust__raw__RawFn o-- codemaid_rust__raw__Segs : sig_refs
  codemaid_rust__raw__RawImpl o-- codemaid_rust__raw__RawFn : methods
  codemaid_rust__raw__RawImpl o-- codemaid_rust__raw__Segs : module, self_ty, trait_
  codemaid_rust__raw__RawItem *-- codemaid_model__symbol__Span : span
  codemaid_rust__raw__RawItem *-- codemaid_model__symbol__SymbolKind : kind
  codemaid_rust__raw__RawItem *-- codemaid_model__symbol__Visibility : vis
  codemaid_rust__raw__RawItem o-- codemaid_rust__raw__RawFn : methods
  codemaid_rust__raw__RawItem o-- codemaid_rust__raw__RawMember : members
  codemaid_rust__raw__RawItem o-- codemaid_rust__raw__RawStep : flow
  codemaid_rust__raw__RawItem o-- codemaid_rust__raw__Segs : module, sig_refs, supertraits
  codemaid_rust__raw__RawMember *-- codemaid_model__symbol__MemberKind : kind
  codemaid_rust__raw__RawMember *-- codemaid_model__symbol__Span : span
  codemaid_rust__raw__RawMember *-- codemaid_model__symbol__Visibility : vis
  codemaid_rust__raw__RawMember o-- codemaid_rust__raw__Segs : refs
  codemaid_rust__raw__RawModule *-- codemaid_model__symbol__Span : span
  codemaid_rust__raw__RawModule *-- codemaid_model__symbol__Visibility : vis
  codemaid_rust__raw__RawModule o-- codemaid_rust__raw__RawUse : uses
  codemaid_rust__raw__RawModule *-- codemaid_rust__raw__Segs : path
  codemaid_rust__raw__RawStep *-- codemaid_rust__raw__RawCall : Call
  codemaid_rust__raw__RawUse *-- codemaid_rust__raw__Segs : target
  codemaid_rust__raw__Recv o-- codemaid_rust__raw__Segs : Typed
```
