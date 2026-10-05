---
codemaid: 1
source: crates/codemaid-corpus/src/index.rs
module: codemaid_corpus::index
language: rust
source_hash: blake3:191efb14999c1abf07dd582c95dad3681f0cae143c227eec77b1afcab877f89a
lines: 131
fragments: 4
---
# `codemaid_corpus::index` · crates/codemaid-corpus/src/index.rs
> `_index.json`: the merge map for orchestrating agents.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_corpus__index__CallRef["CallRef"] {
    <<struct>>
    +target: SymbolId
    +confidence: Confidence
    +line: u32
    +expands: Option#lt;String#gt;
  }
  class codemaid_corpus__index__DocumentEntry["DocumentEntry"] {
    <<struct>>
    +source: SourcePath
    +document: SourcePath
    +module: SymbolId
    +source_hash: ContentHash
    +document_hash: ContentHash
    +fragments: Vec#lt;FragmentEntry#gt;
  }
  class codemaid_corpus__index__FragmentEntry["FragmentEntry"] {
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
  class codemaid_corpus__index__FragmentKind["FragmentKind"] {
    <<enum>>
    Structure
    Sequence
  }
  class codemaid_corpus__index__Index["Index"] {
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
  class codemaid_corpus__index["codemaid_corpus::index"] {
    <<module>>
    -is_zero(n: &usize) bool
  }
  class codemaid_model__symbol__Confidence["Confidence"] {
    <<enum in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__hash__ContentHash["ContentHash"] {
    <<struct in crates/codemaid-model/src/hash.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class codemaid_model__symbol__Span["Span"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__codebase__Codebase["Codebase"] {
    <<struct in crates/codemaid-model/src/codebase.rs>>
  }
  class codemaid_model__codebase__CodebaseStats["CodebaseStats"] {
    <<struct in crates/codemaid-model/src/codebase.rs>>
  }
  codemaid_corpus__index__CallRef *-- codemaid_model__symbol__Confidence : confidence
  codemaid_corpus__index__CallRef *-- codemaid_model__symbol__SymbolId : target
  codemaid_corpus__index__DocumentEntry o-- codemaid_corpus__index__FragmentEntry : fragments
  codemaid_corpus__index__DocumentEntry *-- codemaid_model__hash__ContentHash : source_hash, document_hash
  codemaid_corpus__index__DocumentEntry *-- codemaid_model__path__SourcePath : source, document
  codemaid_corpus__index__DocumentEntry *-- codemaid_model__symbol__SymbolId : module
  codemaid_corpus__index__FragmentEntry o-- codemaid_corpus__index__CallRef : calls
  codemaid_corpus__index__FragmentEntry *-- codemaid_corpus__index__FragmentKind : kind
  codemaid_corpus__index__FragmentEntry *-- codemaid_model__hash__ContentHash : hash
  codemaid_corpus__index__FragmentEntry *-- codemaid_model__path__SourcePath : document
  codemaid_corpus__index__FragmentEntry *-- codemaid_model__symbol__Span : span
  codemaid_corpus__index__FragmentEntry o-- codemaid_model__symbol__SymbolId : symbol, participants
  codemaid_corpus__index__Index o-- codemaid_corpus__index__DocumentEntry : documents
  codemaid_corpus__index__Index ..> codemaid_corpus__index__FragmentEntry
  codemaid_corpus__index__Index ..> codemaid_model__codebase__Codebase
  codemaid_corpus__index__Index *-- codemaid_model__codebase__CodebaseStats : stats
```

## `codemaid_corpus::index::Index::new`
`pub(crate) fn new(cb: &Codebase) -> Self` · L97-L105
```mermaid
sequenceDiagram
  participant codemaid_corpus__index__Index as Index
  participant codemaid_model__codebase__Codebase as Codebase
  codemaid_corpus__index__Index->>codemaid_model__codebase__Codebase: stats()
```

## `codemaid_corpus::index::Index::link_expansions`
`pub(crate) fn link_expansions(&mut self)` · L107-L120
> Fill in [`CallRef::expands`] once all fragments are known.
```mermaid
sequenceDiagram
  participant codemaid_corpus__index__Index as Index
  codemaid_corpus__index__Index->>codemaid_corpus__index__Index: fragments()
```

## `codemaid_corpus::index::Index::fragment`
`pub fn fragment(&self, id: &str) -> Option<&FragmentEntry>` · L127-L130
> Look up a fragment by id.
```mermaid
sequenceDiagram
  participant codemaid_corpus__index__Index as Index
  codemaid_corpus__index__Index->>codemaid_corpus__index__Index: fragments()
```
