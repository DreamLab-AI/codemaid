---
sealmap: 1
source: crates/sealmap-corpus/src/index.rs
module: sealmap_corpus::index
language: rust
source_hash: blake3:09d178e93c5944130390ec1416aeabcbf7fec61b08d7f13395fafb694b3684c9
lines: 131
fragments: 4
---
# `sealmap_corpus::index` · crates/sealmap-corpus/src/index.rs
> `_index.json`: the merge map for orchestrating agents.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__index__CallRef["CallRef"] {
    <<struct>>
    +target: SymbolId
    +confidence: Confidence
    +line: u32
    +expands: Option#lt;String#gt;
  }
  class sealmap_corpus__index__DocumentEntry["DocumentEntry"] {
    <<struct>>
    +source: SourcePath
    +document: SourcePath
    +module: SymbolId
    +source_hash: ContentHash
    +document_hash: ContentHash
    +fragments: Vec#lt;FragmentEntry#gt;
  }
  class sealmap_corpus__index__FragmentEntry["FragmentEntry"] {
    <<struct>>
    +id: String
    +kind: FragmentKind
    +document: SourcePath
    +symbol: SymbolId
    +span: Span
    +participants: Vec#lt;SymbolId#gt;
    +calls: Vec#lt;CallRef#gt;
    +truncated: usize
    +hash: ContentHash
  }
  class sealmap_corpus__index__FragmentKind["FragmentKind"] {
    <<enum>>
    Structure
    Sequence
  }
  class sealmap_corpus__index__Index["Index"] {
    <<struct>>
    +schema: u32
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
  class sealmap_model__symbol__Confidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__hash__ContentHash["ContentHash"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__path__SourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__symbol__Span["Span"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__codebase__CodebaseStats["CodebaseStats"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  sealmap_corpus__index__CallRef *-- sealmap_model__symbol__Confidence : confidence
  sealmap_corpus__index__CallRef *-- sealmap_model__symbol__SymbolId : target
  sealmap_corpus__index__DocumentEntry o-- sealmap_corpus__index__FragmentEntry : fragments
  sealmap_corpus__index__DocumentEntry *-- sealmap_model__hash__ContentHash : source_hash, document_hash
  sealmap_corpus__index__DocumentEntry *-- sealmap_model__path__SourcePath : source, document
  sealmap_corpus__index__DocumentEntry *-- sealmap_model__symbol__SymbolId : module
  sealmap_corpus__index__FragmentEntry o-- sealmap_corpus__index__CallRef : calls
  sealmap_corpus__index__FragmentEntry *-- sealmap_corpus__index__FragmentKind : kind
  sealmap_corpus__index__FragmentEntry *-- sealmap_model__hash__ContentHash : hash
  sealmap_corpus__index__FragmentEntry *-- sealmap_model__path__SourcePath : document
  sealmap_corpus__index__FragmentEntry *-- sealmap_model__symbol__Span : span
  sealmap_corpus__index__FragmentEntry o-- sealmap_model__symbol__SymbolId : symbol, participants
  sealmap_corpus__index__Index o-- sealmap_corpus__index__DocumentEntry : documents
  sealmap_corpus__index__Index ..> sealmap_corpus__index__FragmentEntry
  sealmap_corpus__index__Index ..> sealmap_model__codebase__Codebase
  sealmap_corpus__index__Index *-- sealmap_model__codebase__CodebaseStats : stats
```

## `sealmap_corpus::index::Index::new`
`pub(crate) fn new(cb: &Codebase) -> Self` · L97-L105
```mermaid
sequenceDiagram
  participant sealmap_corpus__index__Index as Index
  participant sealmap_model__codebase__Codebase as Codebase
  sealmap_corpus__index__Index->>sealmap_model__codebase__Codebase: stats()
```

## `sealmap_corpus::index::Index::link_expansions`
`pub(crate) fn link_expansions(&mut self)` · L107-L120
> Fill in [`CallRef::expands`] once all fragments are known.
```mermaid
sequenceDiagram
  participant sealmap_corpus__index__Index as Index
  sealmap_corpus__index__Index->>sealmap_corpus__index__Index: fragments()
```

## `sealmap_corpus::index::Index::fragment`
`pub fn fragment(&self, id: &str) -> Option<&FragmentEntry>` · L127-L130
> Look up a fragment by id.
```mermaid
sequenceDiagram
  participant sealmap_corpus__index__Index as Index
  sealmap_corpus__index__Index->>sealmap_corpus__index__Index: fragments()
```
