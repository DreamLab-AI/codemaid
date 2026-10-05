---
codemaid: 1
source: crates/codemaid-corpus/src/document.rs
module: codemaid_corpus::document
language: rust
source_hash: blake3:ff47d802312565ee209b95a6898e6aec35a31e33e05bf2e794aecba959773458
lines: 139
fragments: 3
---
# `codemaid_corpus::document` · crates/codemaid-corpus/src/document.rs
> One Markdown document per source file.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_corpus__document["codemaid_corpus::document"] {
    <<module>>
    ~const MARKER: &str
    ~front_matter_hash(crate) Option#lt;ContentHash#gt;
    -inline(text: &str) String
    ~is_generated(crate) bool
    ~render(crate) #40;SourcePath, String, DocumentEntry#41;
    -span_text(s: Span) String
  }
  class codemaid_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/codemaid-corpus/src/lib.rs>>
  }
  class codemaid_corpus__Lookup["Lookup#lt;'a#gt;"] {
    <<struct in crates/codemaid-corpus/src/lib.rs>>
  }
  class codemaid_corpus__index__DocumentEntry["DocumentEntry"] {
    <<struct in crates/codemaid-corpus/src/index.rs>>
  }
  class codemaid_model__codebase__Codebase["Codebase"] {
    <<struct in crates/codemaid-model/src/codebase.rs>>
  }
  class codemaid_model__hash__ContentHash["ContentHash"] {
    <<struct in crates/codemaid-model/src/hash.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class codemaid_model__source__SourceFile["SourceFile"] {
    <<struct in crates/codemaid-model/src/source.rs>>
  }
  class codemaid_model__symbol__Span["Span"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  codemaid_corpus__document ..> codemaid_corpus__CorpusOptions
  codemaid_corpus__document ..> codemaid_corpus__Lookup
  codemaid_corpus__document ..> codemaid_corpus__index__DocumentEntry
  codemaid_corpus__document ..> codemaid_model__codebase__Codebase
  codemaid_corpus__document ..> codemaid_model__hash__ContentHash
  codemaid_corpus__document ..> codemaid_model__path__SourcePath
  codemaid_corpus__document ..> codemaid_model__source__SourceFile
  codemaid_corpus__document ..> codemaid_model__symbol__Span
```

## `codemaid_corpus::document::render`
`pub(crate) fn render(cb: &Codebase, lookup: &Lookup<'_>, file: &SourceFile, opts: &CorpusOptions,) -> (SourcePath, String, DocumentEntry)` · L14-L118
```mermaid
sequenceDiagram
  participant codemaid_corpus__document as document mod
  participant codemaid_corpus as codemaid_corpus mod
  participant codemaid_model__codebase__Codebase as Codebase
  participant codemaid_corpus__structure as structure mod
  participant codemaid_corpus__Lookup as Lookup
  participant codemaid_model__symbol__SymbolKind as SymbolKind
  participant codemaid_model__hash__ContentHash as ContentHash
  participant codemaid_corpus__sequence as sequence mod
  participant codemaid_model__flow__Flow as Flow
  codemaid_corpus__document->>codemaid_corpus: document_path(&file.path)
  codemaid_corpus__document->>codemaid_model__codebase__Codebase: symbol(&file.module)
  opt let Some(doc) = module.and_then(| m | m.doc.as_deref…
    codemaid_corpus__document->>codemaid_corpus__document: inline(doc)
  end
  codemaid_corpus__document->>codemaid_corpus__structure: structure::render(cb, lookup, file, opts)
  opt let Some((text, _)) = structure::render(cb, lookup, …
    codemaid_corpus__document->>codemaid_corpus__Lookup: in_file(&file.path)
    loop each via filter
      codemaid_corpus__document->>codemaid_model__symbol__SymbolKind: ~is_type()
    end
    codemaid_corpus__document->>codemaid_model__hash__ContentHash: ContentHash::of_text(&text)
  end
  codemaid_corpus__document->>codemaid_corpus__Lookup: in_file(&file.path)
  loop each via filter
    codemaid_corpus__document->>codemaid_model__symbol__SymbolKind: ~is_callable()
  end
  loop for sym in callables
    codemaid_corpus__document->>codemaid_corpus__sequence: sequence::render(cb, sym, opts)
    codemaid_corpus__document->>codemaid_corpus__document: span_text(sym.span)
    opt let Some(doc) = &sym.doc
      codemaid_corpus__document->>codemaid_corpus__document: inline(doc)
    end
    opt via map
      codemaid_corpus__document->>codemaid_model__flow__Flow: ~calls()
    end
    codemaid_corpus__document->>codemaid_model__hash__ContentHash: ContentHash::of_text(&r.text)
  end
  codemaid_corpus__document->>codemaid_model__hash__ContentHash: ContentHash::of_text(&text)
```

## `codemaid_corpus::document::span_text`
`fn span_text(s: Span) -> String` · L125-L127
```mermaid
sequenceDiagram
  participant codemaid_corpus__document as document mod
  participant codemaid_model__symbol__Span as Span
  codemaid_corpus__document->>codemaid_model__symbol__Span: compact()
```
