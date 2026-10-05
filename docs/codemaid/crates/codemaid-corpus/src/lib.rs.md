---
codemaid: 1
source: crates/codemaid-corpus/src/lib.rs
module: codemaid_corpus
language: rust
source_hash: blake3:8a278b17ac6fcfe085e189e29550eec523646b2667e0bea6d0ae839439811e4e
lines: 212
fragments: 6
---
# `codemaid_corpus` · crates/codemaid-corpus/src/lib.rs
> Projects a [`Codebase`] into a **contract-enforced, 1:1 corpus** of dense Mermaid diagrams plus machine-readable metadata, designed to be read by LLM agents an…

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_corpus__Corpus["Corpus"] {
    <<struct>>
    +files: BTreeMap#lt;SourcePath, String#gt;
    +index: Index
    +document(&self, path: &str) Option#lt;&str#gt;
  }
  class codemaid_corpus__CorpusOptions["CorpusOptions"] {
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
  class codemaid_corpus__ExternalLanes["ExternalLanes"] {
    <<enum>>
    CrateRoot
    Owner
  }
  class codemaid_corpus {
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
  class codemaid_model__codebase__Codebase["Codebase"] {
    <<struct in crates/codemaid-model/src/codebase.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class serde__Serialize["serde::Serialize"] {
    <<external>>
  }
  class codemaid_corpus__index__Index["Index"] {
    <<struct in crates/codemaid-corpus/src/index.rs>>
  }
  codemaid_corpus ..> codemaid_corpus__Corpus
  codemaid_corpus ..> codemaid_corpus__CorpusOptions
  codemaid_corpus ..> codemaid_model__codebase__Codebase
  codemaid_corpus ..> codemaid_model__path__SourcePath
  codemaid_corpus ..> serde__Serialize
  codemaid_corpus__Corpus *-- codemaid_corpus__index__Index : index
  codemaid_corpus__Corpus o-- codemaid_model__path__SourcePath : files
  codemaid_corpus__CorpusOptions *-- codemaid_corpus__ExternalLanes : external_lanes
```

## `codemaid_corpus::Corpus::document`
`pub fn document(&self, path: &str) -> Option<&str>` · L166-L169
> Contents of a generated file by relative path.
```mermaid
sequenceDiagram
  participant codemaid_corpus__Corpus as Corpus
  participant codemaid_model__path__SourcePath as SourcePath
  codemaid_corpus__Corpus->>codemaid_model__path__SourcePath: SourcePath::new(path)
```

## `codemaid_corpus::generate`
`pub fn generate(codebase: &Codebase, options: &CorpusOptions) -> Corpus` · L172-L191
> Generate the full corpus in memory.
```mermaid
sequenceDiagram
  participant codemaid_corpus as codemaid_corpus mod
  participant codemaid_corpus__index__Index as Index
  participant codemaid_corpus__document as document mod
  participant codemaid_model__path__SourcePath as SourcePath
  participant codemaid_corpus__overview as overview mod
  codemaid_corpus->>codemaid_corpus__index__Index: Index::new(codebase)
  loop for file in codebase.files.values()
    codemaid_corpus->>codemaid_corpus__document: document::render(codebase, file, options)
  end
  codemaid_corpus->>codemaid_corpus__index__Index: link_expansions()
  opt closure
    codemaid_corpus->>codemaid_model__path__SourcePath: SourcePath::new(name)
  end
  codemaid_corpus->>codemaid_corpus__overview: overview::render(codebase, options)
  codemaid_corpus->>codemaid_corpus: readme()
  opt options.emit_model
    codemaid_corpus->>codemaid_corpus: to_json(codebase, options.pretty_json)
  end
  codemaid_corpus->>codemaid_corpus: to_json(&index, options.pretty_json)
```

## `codemaid_corpus::document_path`
`pub fn document_path(source: &SourcePath) -> SourcePath` · L193-L196
> Map a source path to its document path (`src/a.rs` → `src/a.rs.md`).
```mermaid
sequenceDiagram
  participant codemaid_corpus as codemaid_corpus mod
  participant codemaid_model__path__SourcePath as SourcePath
  codemaid_corpus->>codemaid_model__path__SourcePath: with_suffix(DOC_SUFFIX)
```

## `codemaid_corpus::is_reserved`
`pub fn is_reserved(path: &SourcePath) -> bool` · L198-L201
> `true` for corpus-level files (`_index.json`, ...).
```mermaid
sequenceDiagram
  participant codemaid_corpus as codemaid_corpus mod
  participant codemaid_model__path__SourcePath as SourcePath
  codemaid_corpus->>codemaid_model__path__SourcePath: file_name()
  codemaid_corpus->>codemaid_model__path__SourcePath: as_str()
```

## `codemaid_corpus::to_json`
`fn to_json(value: &impl serde::Serialize, pretty: bool) -> String` · L203-L208
```mermaid
sequenceDiagram
  participant codemaid_corpus as codemaid_corpus mod
  participant serde_json as serde_json ext
  alt pretty
    codemaid_corpus->>serde_json: serde_json::to_string_pretty(value)
  else
    codemaid_corpus->>serde_json: serde_json::to_string(value)
  end
```
