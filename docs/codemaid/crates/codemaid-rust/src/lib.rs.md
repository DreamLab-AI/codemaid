---
codemaid: 1
source: crates/codemaid-rust/src/lib.rs
module: codemaid_rust
language: rust
source_hash: blake3:e56cfa7f5558e5297df6b8a100ccd8946bc31a02e1f8e8f3a6cb20e2b040f407
lines: 199
fragments: 3
---
# `codemaid_rust` · crates/codemaid-rust/src/lib.rs
> A pure-Rust frontend that turns Rust source into a [`codemaid_model::Codebase`]: modules, types, traits, functions and methods, the relations between them, and…

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_rust__Diagnostic["Diagnostic"] {
    <<struct>>
    +file: SourcePath
    +message: String
  }
  class codemaid_rust__ExternalCalls["ExternalCalls"] {
    <<enum>>
    All
    NonStd
    None
  }
  class codemaid_rust__Extraction["Extraction"] {
    <<struct>>
    +codebase: Codebase
    +diagnostics: Vec#lt;Diagnostic#gt;
  }
  class codemaid_rust__RustOptions["RustOptions"] {
    <<struct>>
    +name: String
    +include_tests: bool
    +external_calls: ExternalCalls
    +Default::default() Self
  }
  class codemaid_rust {
    <<module>>
    +mod collect
    +extract(sources: &SourceSet, options: &RustOptions) Extraction
    +extract_dir(root: &Path, options: &RustOptions) std::io::Result#lt;#40;SourceSet, Extraction#41;#gt;
    +mod layout
    +mod raw
    +mod resolve
    +mod tidy
  }
  class codemaid_model__source__SourceSet["SourceSet"] {
    <<struct in crates/codemaid-model/src/source.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class codemaid_model__codebase__Codebase["Codebase"] {
    <<struct in crates/codemaid-model/src/codebase.rs>>
  }
  codemaid_rust ..> codemaid_model__source__SourceSet
  codemaid_rust ..> codemaid_rust__Extraction
  codemaid_rust ..> codemaid_rust__RustOptions
  codemaid_rust__Diagnostic *-- codemaid_model__path__SourcePath : file
  codemaid_rust__Extraction *-- codemaid_model__codebase__Codebase : codebase
  codemaid_rust__Extraction o-- codemaid_rust__Diagnostic : diagnostics
  codemaid_rust__RustOptions *-- codemaid_rust__ExternalCalls : external_calls
```

## `codemaid_rust::extract`
`pub fn extract(sources: &SourceSet, options: &RustOptions) -> Extraction` · L158-L179
> Extract a [`Codebase`] from the `.rs` (and `Cargo.toml`) files in `sources`.
```mermaid
sequenceDiagram
  participant codemaid_rust as codemaid_rust mod
  participant codemaid_rust__layout as layout mod
  participant codemaid_model__source__SourceSet as SourceSet
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__resolve as resolve mod
  codemaid_rust->>codemaid_rust__layout: layout::plan(sources, &options.name)
  loop each via filter_map
    codemaid_rust->>codemaid_model__source__SourceSet: get(&path)
  end
  opt via map
    codemaid_rust->>codemaid_rust__collect: collect::collect_file(p, r, t, options)
  end
  opt via map
    codemaid_rust->>codemaid_rust__collect: collect::collect_file(p, r, t, options)
  end
  codemaid_rust->>codemaid_rust__resolve: resolve::build(&options.name, files, options)
```

## `codemaid_rust::extract_dir`
`pub fn extract_dir(root: &Path, options: &RustOptions) -> std::io::Result<(SourceSet, Extraction)>` · L181-L199
> Load `root` from disk (`.rs` and `Cargo.toml` files, skipping `target/`, hidden directories and the like) and [`extract`] it.
```mermaid
sequenceDiagram
  participant codemaid_rust as codemaid_rust mod
  participant codemaid_model__source__LoadOptions as LoadOptions
  participant codemaid_model__source__SourceSet as SourceSet
  codemaid_rust->>codemaid_model__source__LoadOptions: LoadOptions::default()
  codemaid_rust->>codemaid_model__source__SourceSet: SourceSet::load_dir(root, &load)?
  codemaid_rust->>codemaid_model__source__SourceSet: retain(|..|)
  codemaid_rust->>codemaid_rust: extract(&sources, &options)
```
