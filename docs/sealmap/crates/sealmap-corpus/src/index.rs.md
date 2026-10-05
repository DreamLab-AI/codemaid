---
sealmap: 2
source: crates/sealmap-corpus/src/index.rs
module: "sym:cargo sealmap_corpus . index/"
language: rust
source_hash: blake3:ead80007ceb6ffff7f797a44e38f08521198a343ae9979966c7293ee7d4db9c1
lines: 136
fragments: 4
---
# `sym:cargo sealmap_corpus . index/` · crates/sealmap-corpus/src/index.rs
> `_index.json`: the merge map for orchestrating agents.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__index___tCallRef["CallRef"] {
    <<struct>>
    +target: SymbolId
    +confidence: Confidence
    +line: u32
    +expands: Option#lt;String#gt;
  }
  class sealmap_corpus__index___tDocumentEntry["DocumentEntry"] {
    <<struct>>
    +source: SourcePath
    +document: SourcePath
    +module: SymbolId
    +source_hash: ContentHash
    +document_hash: ContentHash
    +fragments: Vec#lt;FragmentEntry#gt;
  }
  class sealmap_corpus__index___tFragmentEntry["FragmentEntry"] {
    <<struct>>
    +id: String
    +kind: FragmentKind
    +document: SourcePath
    +symbol: SymbolId
    +span: Span
    +sig_hash: Fingerprint
    +body_hash: Fingerprint
    +participants: Vec#lt;SymbolId#gt;
    +calls: Vec#lt;CallRef#gt;
    +truncated: usize
    +hash: ContentHash
  }
  class sealmap_corpus__index___tFragmentKind["FragmentKind"] {
    <<enum>>
    Structure
    Sequence
  }
  class sealmap_corpus__index___tIndex["Index"] {
    <<struct>>
    +schema_version: u32
    +generator: String
    +codebase: String
    +stats: CodebaseStats
    +documents: Vec#lt;DocumentEntry#gt;
    +fragment(&self, id: &str) Option#lt;&FragmentEntry#gt;
    +fragments(&self) impl Iterator#lt;Item = &FragmentEntry#gt;
    ~link_expansions(crate)
    ~new(crate) Self
  }
  class sealmap_corpus__index["sealmap_corpus::index"] {
    <<module>>
    -is_zero(n: &usize) bool
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__symbol___tConfidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__hash___tContentHash["ContentHash"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__hash___tFingerprint["Fingerprint"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__symbol___tSpan["Span"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__codebase___tCodebaseStats["CodebaseStats"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  sealmap_corpus__index___tCallRef *-- sealmap_model__sym___tSymbolId : target
  sealmap_corpus__index___tCallRef *-- sealmap_model__symbol___tConfidence : confidence
  sealmap_corpus__index___tDocumentEntry o-- sealmap_corpus__index___tFragmentEntry : fragments
  sealmap_corpus__index___tDocumentEntry *-- sealmap_model__hash___tContentHash : source_hash, document_hash
  sealmap_corpus__index___tDocumentEntry *-- sealmap_model__path___tSourcePath : source, document
  sealmap_corpus__index___tDocumentEntry *-- sealmap_model__sym___tSymbolId : module
  sealmap_corpus__index___tFragmentEntry o-- sealmap_corpus__index___tCallRef : calls
  sealmap_corpus__index___tFragmentEntry *-- sealmap_corpus__index___tFragmentKind : kind
  sealmap_corpus__index___tFragmentEntry *-- sealmap_model__hash___tContentHash : hash
  sealmap_corpus__index___tFragmentEntry *-- sealmap_model__hash___tFingerprint : sig_hash, body_hash
  sealmap_corpus__index___tFragmentEntry *-- sealmap_model__path___tSourcePath : document
  sealmap_corpus__index___tFragmentEntry o-- sealmap_model__sym___tSymbolId : symbol, participants
  sealmap_corpus__index___tFragmentEntry *-- sealmap_model__symbol___tSpan : span
  sealmap_corpus__index___tIndex o-- sealmap_corpus__index___tDocumentEntry : documents
  sealmap_corpus__index___tIndex ..> sealmap_corpus__index___tFragmentEntry
  sealmap_corpus__index___tIndex ..> sealmap_model__codebase___tCodebase
  sealmap_corpus__index___tIndex *-- sealmap_model__codebase___tCodebaseStats : stats
```

## `sym:cargo sealmap_corpus . index/Index#new().`
`pub(crate) fn new(cb: &Codebase) -> Self` · L101-L109
```mermaid
sequenceDiagram
  participant sealmap_corpus__index___tIndex as Index
  participant sealmap_model__codebase___tCodebase as Codebase
  sealmap_corpus__index___tIndex->>sealmap_model__codebase___tCodebase: stats()
```

## `sym:cargo sealmap_corpus . index/Index#link_expansions().`
`pub(crate) fn link_expansions(&mut self)` · L111-L125
> Fill in [`CallRef::expands`] once all fragments are known.
```mermaid
sequenceDiagram
  participant sealmap_corpus__index___tIndex as Index
  sealmap_corpus__index___tIndex->>sealmap_corpus__index___tIndex: fragments()
```

## `sym:cargo sealmap_corpus . index/Index#fragment().`
`pub fn fragment(&self, id: &str) -> Option<&FragmentEntry>` · L132-L135
> Look up a fragment by id.
```mermaid
sequenceDiagram
  participant sealmap_corpus__index___tIndex as Index
  sealmap_corpus__index___tIndex->>sealmap_corpus__index___tIndex: fragments()
```
