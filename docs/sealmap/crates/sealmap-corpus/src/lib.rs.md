---
sealmap: 1
source: crates/sealmap-corpus/src/lib.rs
module: sealmap_corpus
language: rust
source_hash: blake3:8e8e49cffd2ddd776b409d62e540c40211856a0930a26ce8f27f036683c36c34
lines: 249
fragments: 6
---
# `sealmap_corpus` · crates/sealmap-corpus/src/lib.rs
> Projects a [`Codebase`] into a **contract-enforced, 1:1 corpus** of dense Mermaid diagrams plus machine-readable metadata, designed to be read by LLM agents an…

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__Corpus["Corpus"] {
    <<struct>>
    +files: BTreeMap#lt;SourcePath, String#gt;
    +index: Index
    +document(&self, path: &str) Option#lt;&str#gt;
  }
  class sealmap_corpus__CorpusOptions["CorpusOptions"] {
    <<struct>>
    +min_calls: usize
    +max_messages: usize
    +max_edges: usize
    +max_entities: usize
    +include_private: bool
    +external_lanes: ExternalLanes
    +emit_model: bool
    +pretty_json: bool
    +Default::default() Self
  }
  class sealmap_corpus__ExternalLanes["ExternalLanes"] {
    <<enum>>
    CrateRoot
    Owner
  }
  class sealmap_corpus__Lookup["Lookup#lt;'a#gt;"] {
    <<struct>>
    -by_file: BTreeMap#lt;&'a SourcePath, Vec#lt;&'a Symbol#gt;#gt;
    -children: BTreeMap#lt;&'a SymbolId, Vec#lt;&'a Symbol#gt;#gt;
    ~children(crate) impl Iterator#lt;Item = &'a Symbol#gt; + '_
    ~in_file(crate) impl Iterator#lt;Item = &'a Symbol#gt; + '_
    -new(cb: &'a Codebase) Self
  }
  class sealmap_corpus {
    <<module>>
    +const CORPUS_SCHEMA_VERSION: u32
    +const DOC_SUFFIX: &str
    +mod contract
    +mod document
    +document_path(source: &SourcePath) SourcePath
    +generate(codebase: &Codebase, options: &CorpusOptions) Corpus
    +mod index
    +is_reserved(path: &SourcePath) bool
    +mod naming
    +mod overview
    -readme() String
    +mod sequence
    +mod structure
    -to_json(value: &impl serde::Serialize, pretty: bool) String
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__path__SourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class serde__Serialize["serde::Serialize"] {
    <<external>>
  }
  class sealmap_corpus__index__Index["Index"] {
    <<struct in crates/sealmap-corpus/src/index.rs>>
  }
  class sealmap_model__symbol__Symbol["Symbol"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_corpus ..> sealmap_corpus__Corpus
  sealmap_corpus ..> sealmap_corpus__CorpusOptions
  sealmap_corpus ..> sealmap_model__codebase__Codebase
  sealmap_corpus ..> sealmap_model__path__SourcePath
  sealmap_corpus ..> serde__Serialize
  sealmap_corpus__Corpus *-- sealmap_corpus__index__Index : index
  sealmap_corpus__Corpus o-- sealmap_model__path__SourcePath : files
  sealmap_corpus__CorpusOptions *-- sealmap_corpus__ExternalLanes : external_lanes
  sealmap_corpus__Lookup ..> sealmap_model__codebase__Codebase
  sealmap_corpus__Lookup o-- sealmap_model__path__SourcePath : by_file
  sealmap_corpus__Lookup o-- sealmap_model__symbol__Symbol : by_file, children
  sealmap_corpus__Lookup o-- sealmap_model__symbol__SymbolId : children
```

## `sealmap_corpus::Corpus::document`
`pub fn document(&self, path: &str) -> Option<&str>` · L166-L169
> Contents of a generated file by relative path.
```mermaid
sequenceDiagram
  participant sealmap_corpus__Corpus as Corpus
  participant sealmap_model__path__SourcePath as SourcePath
  sealmap_corpus__Corpus->>sealmap_model__path__SourcePath: SourcePath::new(path)
```

## `sealmap_corpus::generate`
`pub fn generate(codebase: &Codebase, options: &CorpusOptions) -> Corpus` · L172-L193
> Generate the full corpus in memory.
```mermaid
sequenceDiagram
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_corpus__naming as naming mod
  participant sealmap_corpus__index__Index as Index
  participant sealmap_corpus__Lookup as Lookup
  participant sealmap_corpus__document as document mod
  participant sealmap_model__path__SourcePath as SourcePath
  participant sealmap_corpus__overview as overview mod
  sealmap_corpus->>sealmap_corpus__naming: naming::assert_unique_idents(codebase)
  sealmap_corpus->>sealmap_corpus__index__Index: Index::new(codebase)
  sealmap_corpus->>sealmap_corpus__Lookup: Lookup::new(codebase)
  loop for file in codebase.files.values()
    sealmap_corpus->>sealmap_corpus__document: document::render(codebase, &lookup, file, options)
  end
  sealmap_corpus->>sealmap_corpus__index__Index: link_expansions()
  opt closure
    sealmap_corpus->>sealmap_model__path__SourcePath: SourcePath::new(name)
  end
  sealmap_corpus->>sealmap_corpus__overview: overview::render(codebase, options)
  sealmap_corpus->>sealmap_corpus: readme()
  opt options.emit_model
    sealmap_corpus->>sealmap_corpus: to_json(codebase, options.pretty_json)
  end
  sealmap_corpus->>sealmap_corpus: to_json(&index, options.pretty_json)
```

## `sealmap_corpus::document_path`
`pub fn document_path(source: &SourcePath) -> SourcePath` · L230-L233
> Map a source path to its document path (`src/a.rs` → `src/a.rs.md`).
```mermaid
sequenceDiagram
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_model__path__SourcePath as SourcePath
  sealmap_corpus->>sealmap_model__path__SourcePath: with_suffix(DOC_SUFFIX)
```

## `sealmap_corpus::is_reserved`
`pub fn is_reserved(path: &SourcePath) -> bool` · L235-L238
> `true` for corpus-level files (`_index.json`, ...).
```mermaid
sequenceDiagram
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_model__path__SourcePath as SourcePath
  sealmap_corpus->>sealmap_model__path__SourcePath: file_name()
  sealmap_corpus->>sealmap_model__path__SourcePath: as_str()
```

## `sealmap_corpus::to_json`
`fn to_json(value: &impl serde::Serialize, pretty: bool) -> String` · L240-L245
```mermaid
sequenceDiagram
  participant sealmap_corpus as sealmap_corpus mod
  participant serde_json as serde_json ext
  alt pretty
    sealmap_corpus->>serde_json: serde_json::to_string_pretty(value)
  else
    sealmap_corpus->>serde_json: serde_json::to_string(value)
  end
```
