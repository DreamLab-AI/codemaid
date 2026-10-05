---
sealmap: 2
source: crates/sealmap-corpus/src/document.rs
module: "sym:cargo sealmap_corpus . document/"
language: rust
source_hash: blake3:e299e3762f42c42e95b8770a05df25f7fc7a8cad75e3fabcc61bfdc1c848eb96
lines: 155
fragments: 4
---
# `sym:cargo sealmap_corpus . document/` · crates/sealmap-corpus/src/document.rs
> One Markdown document per source file.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__document["sealmap_corpus::document"] {
    <<module>>
    ~const MARKER: &str
    -code(text: &str) String
    ~front_matter_hash(crate) Option#lt;ContentHash#gt;
    -inline(text: &str) String
    ~is_generated(crate) bool
    ~render(crate) #40;SourcePath, String, DocumentEntry#41;
    -span_text(s: Span) String
    -yaml_str(text: &str) String
  }
  class sealmap_corpus___tCorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_corpus___tLookup["Lookup#lt;'a#gt;"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_corpus__index___tDocumentEntry["DocumentEntry"] {
    <<struct in crates/sealmap-corpus/src/index.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__hash___tContentHash["ContentHash"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__source___tSourceFile["SourceFile"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_model__symbol___tSpan["Span"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_corpus__document ..> sealmap_corpus___tCorpusOptions
  sealmap_corpus__document ..> sealmap_corpus___tLookup
  sealmap_corpus__document ..> sealmap_corpus__index___tDocumentEntry
  sealmap_corpus__document ..> sealmap_model__codebase___tCodebase
  sealmap_corpus__document ..> sealmap_model__hash___tContentHash
  sealmap_corpus__document ..> sealmap_model__path___tSourcePath
  sealmap_corpus__document ..> sealmap_model__source___tSourceFile
  sealmap_corpus__document ..> sealmap_model__symbol___tSpan
```

## `sym:cargo sealmap_corpus . document/render().`
`pub(crate) fn render(cb: &Codebase, lookup: &Lookup<'_>, file: &SourceFile, opts: &CorpusOptions,) -> (SourcePath, String, DocumentEntry)` · L14-L122
```mermaid
sequenceDiagram
  participant sealmap_corpus__document as document mod
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_corpus__structure as structure mod
  participant sealmap_corpus___tLookup as Lookup
  participant sealmap_model__symbol___tSymbolKind as SymbolKind
  participant sealmap_model__hash___tContentHash as ContentHash
  participant sealmap_corpus__sequence as sequence mod
  participant sealmap_model__flow___tFlow as Flow
  sealmap_corpus__document->>sealmap_corpus: document_path(&file.path)
  sealmap_corpus__document->>sealmap_model__codebase___tCodebase: symbol(&file.module)
  sealmap_corpus__document->>sealmap_corpus__document: code(&to_string())
  opt let Some(doc) = module.and_then(| m | m.doc.as_deref…
    sealmap_corpus__document->>sealmap_corpus__document: inline(doc)
  end
  sealmap_corpus__document->>sealmap_corpus__structure: structure::render(cb, lookup, file, opts)
  opt let Some((text, _)) = structure::render(cb, lookup, …
    sealmap_corpus__document->>sealmap_corpus___tLookup: in_file(&file.path)
    loop each via filter
      sealmap_corpus__document->>sealmap_model__symbol___tSymbolKind: ~is_type()
    end
    sealmap_corpus__document->>sealmap_model__hash___tContentHash: ContentHash::of_text(&text)
  end
  sealmap_corpus__document->>sealmap_corpus___tLookup: in_file(&file.path)
  loop each via filter
    sealmap_corpus__document->>sealmap_model__symbol___tSymbolKind: ~is_callable()
  end
  loop for sym in callables
    sealmap_corpus__document->>sealmap_corpus__sequence: sequence::render(cb, sym, opts)
    sealmap_corpus__document->>sealmap_corpus__document: code(&to_string())
    sealmap_corpus__document->>sealmap_corpus__document: span_text(sym.span)
    opt let Some(doc) = &sym.doc
      sealmap_corpus__document->>sealmap_corpus__document: inline(doc)
    end
    opt via map
      sealmap_corpus__document->>sealmap_model__flow___tFlow: ~calls()
    end
    sealmap_corpus__document->>sealmap_model__hash___tContentHash: ContentHash::of_text(&r.text)
  end
  sealmap_corpus__document->>sealmap_corpus__document: yaml_str(&to_string())
  sealmap_corpus__document->>sealmap_model__hash___tContentHash: ContentHash::of_text(&text)
```

## `sym:cargo sealmap_corpus . document/yaml_str().`
`fn yaml_str(text: &str) -> String` · L124-L128
> A double-quoted YAML scalar (JSON string syntax is valid YAML), so ids holding `: ` or ` #` cannot be misread as structure or comments.
```mermaid
sequenceDiagram
  participant sealmap_corpus__document as document mod
  participant _serde_json as serde_json ext
  sealmap_corpus__document->>_serde_json: serde_json::to_string(text)
```

## `sym:cargo sealmap_corpus . document/span_text().`
`fn span_text(s: Span) -> String` · L141-L143
```mermaid
sequenceDiagram
  participant sealmap_corpus__document as document mod
  participant sealmap_model__symbol___tSpan as Span
  sealmap_corpus__document->>sealmap_model__symbol___tSpan: compact()
```
