---
sealmap: 1
source: crates/sealmap-corpus/src/document.rs
module: sealmap_corpus::document
language: rust
source_hash: blake3:1f39f0348a744d33a81b772d289bf394150f72c0c4f4ea26a320297960a68a6b
lines: 139
fragments: 3
---
# `sealmap_corpus::document` · crates/sealmap-corpus/src/document.rs
> One Markdown document per source file.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__document["sealmap_corpus::document"] {
    <<module>>
    ~const MARKER: &str
    ~front_matter_hash(crate) Option#lt;ContentHash#gt;
    -inline(text: &str) String
    ~is_generated(crate) bool
    ~render(crate) #40;SourcePath, String, DocumentEntry#41;
    -span_text(s: Span) String
  }
  class sealmap_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_corpus__Lookup["Lookup#lt;'a#gt;"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_corpus__index__DocumentEntry["DocumentEntry"] {
    <<struct in crates/sealmap-corpus/src/index.rs>>
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__hash__ContentHash["ContentHash"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__path__SourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__source__SourceFile["SourceFile"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_model__symbol__Span["Span"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_corpus__document ..> sealmap_corpus__CorpusOptions
  sealmap_corpus__document ..> sealmap_corpus__Lookup
  sealmap_corpus__document ..> sealmap_corpus__index__DocumentEntry
  sealmap_corpus__document ..> sealmap_model__codebase__Codebase
  sealmap_corpus__document ..> sealmap_model__hash__ContentHash
  sealmap_corpus__document ..> sealmap_model__path__SourcePath
  sealmap_corpus__document ..> sealmap_model__source__SourceFile
  sealmap_corpus__document ..> sealmap_model__symbol__Span
```

## `sealmap_corpus::document::render`
`pub(crate) fn render(cb: &Codebase, lookup: &Lookup<'_>, file: &SourceFile, opts: &CorpusOptions,) -> (SourcePath, String, DocumentEntry)` · L14-L118
```mermaid
sequenceDiagram
  participant sealmap_corpus__document as document mod
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_model__codebase__Codebase as Codebase
  participant sealmap_corpus__structure as structure mod
  participant sealmap_corpus__Lookup as Lookup
  participant sealmap_model__symbol__SymbolKind as SymbolKind
  participant sealmap_model__hash__ContentHash as ContentHash
  participant sealmap_corpus__sequence as sequence mod
  participant sealmap_model__flow__Flow as Flow
  sealmap_corpus__document->>sealmap_corpus: document_path(&file.path)
  sealmap_corpus__document->>sealmap_model__codebase__Codebase: symbol(&file.module)
  opt let Some(doc) = module.and_then(| m | m.doc.as_deref…
    sealmap_corpus__document->>sealmap_corpus__document: inline(doc)
  end
  sealmap_corpus__document->>sealmap_corpus__structure: structure::render(cb, lookup, file, opts)
  opt let Some((text, _)) = structure::render(cb, lookup, …
    sealmap_corpus__document->>sealmap_corpus__Lookup: in_file(&file.path)
    loop each via filter
      sealmap_corpus__document->>sealmap_model__symbol__SymbolKind: ~is_type()
    end
    sealmap_corpus__document->>sealmap_model__hash__ContentHash: ContentHash::of_text(&text)
  end
  sealmap_corpus__document->>sealmap_corpus__Lookup: in_file(&file.path)
  loop each via filter
    sealmap_corpus__document->>sealmap_model__symbol__SymbolKind: ~is_callable()
  end
  loop for sym in callables
    sealmap_corpus__document->>sealmap_corpus__sequence: sequence::render(cb, sym, opts)
    sealmap_corpus__document->>sealmap_corpus__document: span_text(sym.span)
    opt let Some(doc) = &sym.doc
      sealmap_corpus__document->>sealmap_corpus__document: inline(doc)
    end
    opt via map
      sealmap_corpus__document->>sealmap_model__flow__Flow: ~calls()
    end
    sealmap_corpus__document->>sealmap_model__hash__ContentHash: ContentHash::of_text(&r.text)
  end
  sealmap_corpus__document->>sealmap_model__hash__ContentHash: ContentHash::of_text(&text)
```

## `sealmap_corpus::document::span_text`
`fn span_text(s: Span) -> String` · L125-L127
```mermaid
sequenceDiagram
  participant sealmap_corpus__document as document mod
  participant sealmap_model__symbol__Span as Span
  sealmap_corpus__document->>sealmap_model__symbol__Span: compact()
```
