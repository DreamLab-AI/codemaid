---
sealmap: 2
source: crates/sealmap-corpus/src/lib.rs
module: "sym:cargo sealmap_corpus ."
language: rust
source_hash: blake3:85cafa8370052296885647d0358b012ebc84072766b97b125d8cb199cc389530
lines: 264
fragments: 6
---
# `sym:cargo sealmap_corpus .` · crates/sealmap-corpus/src/lib.rs
> Projects a [`Codebase`] into a **contract-enforced, 1:1 corpus** of dense Mermaid diagrams plus machine-readable metadata, designed to be read by LLM agents an…

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus___tCorpus["Corpus"] {
    <<struct>>
    +files: BTreeMap#lt;SourcePath, String#gt;
    +index: Index
    +document(&self, path: &str) Option#lt;&str#gt;
  }
  class sealmap_corpus___tCorpusOptions["CorpusOptions"] {
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
  class sealmap_corpus___tExternalLanes["ExternalLanes"] {
    <<enum>>
    CrateRoot
    Owner
  }
  class sealmap_corpus___tLookup["Lookup#lt;'a#gt;"] {
    <<struct>>
    -by_file: BTreeMap#lt;&'a SourcePath, Vec#lt;&'a Symbol#gt;#gt;
    -children: BTreeMap#lt;&'a SymbolId, Vec#lt;&'a Symbol#gt;#gt;
    ~children(crate) impl Iterator#lt;Item = &'a Symbol#gt; + '_
    ~in_file(crate) impl Iterator#lt;Item = &'a Symbol#gt; + '_
    -new(cb: &'a Codebase) Self
  }
  class sealmap_corpus___tReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
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
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class _serde__Serialize["serde::Serialize"] {
    <<external>>
  }
  class sealmap_corpus__index___tIndex["Index"] {
    <<struct in crates/sealmap-corpus/src/index.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__symbol___tSymbol["Symbol"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_corpus ..> sealmap_corpus___tCorpus
  sealmap_corpus ..> sealmap_corpus___tCorpusOptions
  sealmap_corpus ..> sealmap_model__codebase___tCodebase
  sealmap_corpus ..> sealmap_model__path___tSourcePath
  sealmap_corpus ..> _serde__Serialize
  sealmap_corpus___tCorpus *-- sealmap_corpus__index___tIndex : index
  sealmap_corpus___tCorpus o-- sealmap_model__path___tSourcePath : files
  sealmap_corpus___tCorpusOptions *-- sealmap_corpus___tExternalLanes : external_lanes
  sealmap_corpus___tLookup ..> sealmap_model__codebase___tCodebase
  sealmap_corpus___tLookup o-- sealmap_model__path___tSourcePath : by_file
  sealmap_corpus___tLookup o-- sealmap_model__sym___tSymbolId : children
  sealmap_corpus___tLookup o-- sealmap_model__symbol___tSymbol : by_file, children
```

## `sym:cargo sealmap_corpus . Corpus#document().`
`pub fn document(&self, path: &str) -> Option<&str>` · L176-L179
> Contents of a generated file by relative path.
```mermaid
sequenceDiagram
  participant sealmap_corpus___tCorpus as Corpus
  participant sealmap_model__path___tSourcePath as SourcePath
  sealmap_corpus___tCorpus->>sealmap_model__path___tSourcePath: SourcePath::new(path)
```

## `sym:cargo sealmap_corpus . generate().`
`pub fn generate(codebase: &Codebase, options: &CorpusOptions) -> Corpus` · L182-L203
> Generate the full corpus in memory.
```mermaid
sequenceDiagram
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_corpus__naming as naming mod
  participant sealmap_corpus__index___tIndex as Index
  participant sealmap_corpus___tLookup as Lookup
  participant sealmap_corpus__document as document mod
  participant sealmap_model__path___tSourcePath as SourcePath
  participant sealmap_corpus__overview as overview mod
  sealmap_corpus->>sealmap_corpus__naming: naming::assert_unique_idents(codebase)
  sealmap_corpus->>sealmap_corpus__index___tIndex: Index::new(codebase)
  sealmap_corpus->>sealmap_corpus___tLookup: Lookup::new(codebase)
  loop for file in codebase.files.values()
    sealmap_corpus->>sealmap_corpus__document: document::render(codebase, &lookup, file, options)
  end
  sealmap_corpus->>sealmap_corpus__index___tIndex: link_expansions()
  opt closure
    sealmap_corpus->>sealmap_model__path___tSourcePath: SourcePath::new(name)
  end
  sealmap_corpus->>sealmap_corpus__overview: overview::render(codebase, options)
  sealmap_corpus->>sealmap_corpus: readme()
  opt options.emit_model
    sealmap_corpus->>sealmap_corpus: to_json(codebase, options.pretty_json)
  end
  sealmap_corpus->>sealmap_corpus: to_json(&index, options.pretty_json)
```

## `sym:cargo sealmap_corpus . document_path().`
`pub fn document_path(source: &SourcePath) -> SourcePath` · L240-L243
> Map a source path to its document path (`src/a.rs` → `src/a.rs.md`).
```mermaid
sequenceDiagram
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_model__path___tSourcePath as SourcePath
  sealmap_corpus->>sealmap_model__path___tSourcePath: with_suffix(DOC_SUFFIX)
```

## `sym:cargo sealmap_corpus . is_reserved().`
`pub fn is_reserved(path: &SourcePath) -> bool` · L245-L248
> `true` for corpus-level files (`_index.json`, ...).
```mermaid
sequenceDiagram
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_model__path___tSourcePath as SourcePath
  sealmap_corpus->>sealmap_model__path___tSourcePath: file_name()
  sealmap_corpus->>sealmap_model__path___tSourcePath: as_str()
```

## `sym:cargo sealmap_corpus . to_json().`
`fn to_json(value: &impl serde::Serialize, pretty: bool) -> String` · L250-L255
```mermaid
sequenceDiagram
  participant sealmap_corpus as sealmap_corpus mod
  participant _serde_json as serde_json ext
  alt pretty
    sealmap_corpus->>_serde_json: serde_json::to_string_pretty(value)
  else
    sealmap_corpus->>_serde_json: serde_json::to_string(value)
  end
```
