---
sealmap: 2
source: crates/sealmap-rust/src/raw.rs
module: "sym:cargo sealmap_rust . raw/"
language: rust
source_hash: blake3:43f9883da8e7ee5fccddc041f490a25b7dc090d273153d99b4f9ad0334c6ac0b
lines: 113
fragments: 1
---
# `sym:cargo sealmap_rust . raw/` · crates/sealmap-rust/src/raw.rs
> Pass-1 output: plain, unresolved data extracted from one file.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_rust__raw___tRawFile["RawFile"] {
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
  class sealmap_rust__raw___tRawFn["RawFn"] {
    <<struct>>
    +name: String
    +vis: Visibility
    +span: Span
    +signature: String
    +doc: Option#lt;String#gt;
    +generics: Vec#lt;String#gt;
    +type_params: Vec#lt;String#gt;
    +tags: Vec#lt;String#gt;
    +sig_refs: Vec#lt;Segs#gt;
    +flow: Vec#lt;RawStep#gt;
    +sig_hash: Fingerprint
    +body_hash: Fingerprint
  }
  class sealmap_rust__raw___tRawImpl["RawImpl"] {
    <<struct>>
    +module: Segs
    +self_ty: Option#lt;Segs#gt;
    +trait_: Option#lt;#40;Segs, String#41;#gt;
    +type_params: Vec#lt;String#gt;
    +methods: Vec#lt;RawFn#gt;
  }
  class sealmap_rust__raw___tRawItem["RawItem"] {
    <<struct>>
    +module: Segs
    +name: String
    +kind: SymbolKind
    +vis: Visibility
    +span: Span
    +signature: Option#lt;String#gt;
    +doc: Option#lt;String#gt;
    +generics: Vec#lt;String#gt;
    +type_params: Vec#lt;String#gt;
    +tags: Vec#lt;String#gt;
    +members: Vec#lt;RawMember#gt;
    +sig_refs: Vec#lt;Segs#gt;
    +supertraits: Vec#lt;Segs#gt;
    +methods: Vec#lt;RawFn#gt;
    +flow: Vec#lt;RawStep#gt;
    +sig_hash: Fingerprint
    +body_hash: Fingerprint
  }
  class sealmap_rust__raw___tRawMember["RawMember"] {
    <<struct>>
    +name: String
    +kind: MemberKind
    +ty: Option#lt;String#gt;
    +vis: Visibility
    +span: Span
    +refs: Vec#lt;Segs#gt;
    +type_params: Vec#lt;String#gt;
  }
  class sealmap_rust__raw___tRawModule["RawModule"] {
    <<struct>>
    +path: Segs
    +span: Span
    +doc: Option#lt;String#gt;
    +vis: Visibility
    +uses: Vec#lt;RawUse#gt;
    +tags: Vec#lt;String#gt;
    +sig_hash: Fingerprint
    +body_hash: Fingerprint
  }
  class sealmap_rust__raw___tRawUse["RawUse"] {
    <<struct>>
    +alias: String
    +target: Segs
  }
  class sealmap_model__hash___tContentHash["ContentHash"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_rust__layout___tFileRole["FileRole"] {
    <<struct in crates/sealmap-rust/src/layout.rs>>
  }
  class sealmap_extract__raw___tRawStep["RawStep"] {
    <<enum in crates/sealmap-extract/src/raw.rs>>
  }
  class sealmap_extract__raw___tSegs["Segs"] {
    <<type in crates/sealmap-extract/src/raw.rs>>
  }
  class sealmap_model__hash___tFingerprint["Fingerprint"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__symbol___tSpan["Span"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol___tVisibility["Visibility"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol___tSymbolKind["SymbolKind"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol___tMemberKind["MemberKind"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_rust__raw___tRawFile *-- sealmap_model__hash___tContentHash : hash
  sealmap_rust__raw___tRawFile *-- sealmap_model__path___tSourcePath : path
  sealmap_rust__raw___tRawFile *-- sealmap_rust__layout___tFileRole : role
  sealmap_rust__raw___tRawFile o-- sealmap_rust__raw___tRawImpl : impls
  sealmap_rust__raw___tRawFile o-- sealmap_rust__raw___tRawItem : items
  sealmap_rust__raw___tRawFile o-- sealmap_rust__raw___tRawModule : modules
  sealmap_rust__raw___tRawFn o-- sealmap_extract__raw___tRawStep : flow
  sealmap_rust__raw___tRawFn o-- sealmap_extract__raw___tSegs : sig_refs
  sealmap_rust__raw___tRawFn *-- sealmap_model__hash___tFingerprint : sig_hash, body_hash
  sealmap_rust__raw___tRawFn *-- sealmap_model__symbol___tSpan : span
  sealmap_rust__raw___tRawFn *-- sealmap_model__symbol___tVisibility : vis
  sealmap_rust__raw___tRawImpl o-- sealmap_extract__raw___tSegs : module, self_ty, trait_
  sealmap_rust__raw___tRawImpl o-- sealmap_rust__raw___tRawFn : methods
  sealmap_rust__raw___tRawItem o-- sealmap_extract__raw___tRawStep : flow
  sealmap_rust__raw___tRawItem o-- sealmap_extract__raw___tSegs : module, sig_refs, supertraits
  sealmap_rust__raw___tRawItem *-- sealmap_model__hash___tFingerprint : sig_hash, body_hash
  sealmap_rust__raw___tRawItem *-- sealmap_model__symbol___tSpan : span
  sealmap_rust__raw___tRawItem *-- sealmap_model__symbol___tSymbolKind : kind
  sealmap_rust__raw___tRawItem *-- sealmap_model__symbol___tVisibility : vis
  sealmap_rust__raw___tRawItem o-- sealmap_rust__raw___tRawFn : methods
  sealmap_rust__raw___tRawItem o-- sealmap_rust__raw___tRawMember : members
  sealmap_rust__raw___tRawMember o-- sealmap_extract__raw___tSegs : refs
  sealmap_rust__raw___tRawMember *-- sealmap_model__symbol___tMemberKind : kind
  sealmap_rust__raw___tRawMember *-- sealmap_model__symbol___tSpan : span
  sealmap_rust__raw___tRawMember *-- sealmap_model__symbol___tVisibility : vis
  sealmap_rust__raw___tRawModule *-- sealmap_extract__raw___tSegs : path
  sealmap_rust__raw___tRawModule *-- sealmap_model__hash___tFingerprint : sig_hash, body_hash
  sealmap_rust__raw___tRawModule *-- sealmap_model__symbol___tSpan : span
  sealmap_rust__raw___tRawModule *-- sealmap_model__symbol___tVisibility : vis
  sealmap_rust__raw___tRawModule o-- sealmap_rust__raw___tRawUse : uses
  sealmap_rust__raw___tRawUse *-- sealmap_extract__raw___tSegs : target
```
