---
codemaid: 1
source: crates/codemaid-rust/src/lib.rs
module: codemaid_rust
language: rust
source_hash: blake3:75c371f6033f6cd8a587c2a497806ddca60e2dc391c2b008fd3a54172b214833
lines: 246
fragments: 5
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
  class codemaid_rust__Job["Job#lt;'a#gt;"] {
    <<type>>
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
    -const COLLECT_STACK_BYTES: usize
    +mod collect
    -collect_all(jobs: &[Job#lt;'_#gt;], options: &RustOptions) Vec#lt;raw::RawFile#gt;
    +extract(sources: &SourceSet, options: &RustOptions) Extraction
    +extract_dir(root: &Path, options: &RustOptions) std::io::Result#lt;#40;SourceSet, Extraction#41;#gt;
    +mod layout
    +load_dir(root: &Path) std::io::Result#lt;SourceSet#gt;
    +mod raw
    +mod resolve
    +mod tidy
  }
  class codemaid_model__source__SourceSet["SourceSet"] {
    <<struct in crates/codemaid-model/src/source.rs>>
  }
  class codemaid_rust__raw__RawFile["RawFile"] {
    <<struct in crates/codemaid-rust/src/raw.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class codemaid_model__codebase__Codebase["Codebase"] {
    <<struct in crates/codemaid-model/src/codebase.rs>>
  }
  class codemaid_rust__layout__FileRole["FileRole"] {
    <<struct in crates/codemaid-rust/src/layout.rs>>
  }
  codemaid_rust ..> codemaid_model__source__SourceSet
  codemaid_rust ..> codemaid_rust__Extraction
  codemaid_rust ..> codemaid_rust__Job
  codemaid_rust ..> codemaid_rust__RustOptions
  codemaid_rust ..> codemaid_rust__raw__RawFile
  codemaid_rust__Diagnostic *-- codemaid_model__path__SourcePath : file
  codemaid_rust__Extraction *-- codemaid_model__codebase__Codebase : codebase
  codemaid_rust__Extraction o-- codemaid_rust__Diagnostic : diagnostics
  codemaid_rust__Job ..> codemaid_model__path__SourcePath
  codemaid_rust__Job ..> codemaid_rust__layout__FileRole
  codemaid_rust__RustOptions *-- codemaid_rust__ExternalCalls : external_calls
```

## `codemaid_rust::extract`
`pub fn extract(sources: &SourceSet, options: &RustOptions) -> Extraction` · L158-L173
> Extract a [`Codebase`] from the `.rs` (and `Cargo.toml`) files in `sources`.
```mermaid
sequenceDiagram
  participant codemaid_rust as codemaid_rust mod
  participant codemaid_rust__layout as layout mod
  participant codemaid_model__source__SourceSet as SourceSet
  participant codemaid_rust__resolve as resolve mod
  codemaid_rust->>codemaid_rust__layout: layout::plan(sources, &options.name)
  loop each via filter_map
    codemaid_rust->>codemaid_model__source__SourceSet: get(&path)
  end
  codemaid_rust->>codemaid_rust: collect_all(&jobs, options)
  codemaid_rust->>codemaid_rust__resolve: resolve::build(&options.name, files, options)
```

## `codemaid_rust::collect_all`
`fn collect_all(jobs: &[Job<'_>], options: &RustOptions) -> Vec<raw::RawFile>` · L185-L218
> Pass 1 over every file, on threads with [`COLLECT_STACK_BYTES`] of stack.
```mermaid
sequenceDiagram
  participant codemaid_rust as codemaid_rust mod
  participant codemaid_rust__collect as collect mod
  participant rayon as rayon ext
  opt closure
    opt closure
      codemaid_rust->>codemaid_rust__collect: collect::collect_file(p, r, t, options)
    end
    opt via unwrap_or_else
      codemaid_rust->>codemaid_rust__collect: collect::failed_file(p, r, t, _)
    end
  end
  codemaid_rust->>rayon: ThreadPoolBuilder::ThreadPoolBuilder::new()
  opt let Ok(pool) = rayon::ThreadPoolBuilder::new().stack…
    Note over codemaid_rust: return pool.install(| | jobs.par_iter().map(one).collec…
  end
```

## `codemaid_rust::load_dir`
`pub fn load_dir(root: &Path) -> std::io::Result<SourceSet>` · L220-L228
> Load the `.rs` and `Cargo.toml` files under `root` (honouring `.gitignore`, skipping `target/`, hidden directories and the like) without extracting them.
```mermaid
sequenceDiagram
  participant codemaid_rust as codemaid_rust mod
  participant codemaid_model__source__LoadOptions as LoadOptions
  participant codemaid_model__source__SourceSet as SourceSet
  codemaid_rust->>codemaid_model__source__LoadOptions: LoadOptions::default()
  codemaid_rust->>codemaid_model__source__SourceSet: SourceSet::load_dir(root, &load)?
  codemaid_rust->>codemaid_model__source__SourceSet: retain(|..|)
```

## `codemaid_rust::extract_dir`
`pub fn extract_dir(root: &Path, options: &RustOptions) -> std::io::Result<(SourceSet, Extraction)>` · L230-L246
> Load `root` from disk (`.rs` and `Cargo.toml` files, skipping `target/`, hidden directories and the like) and [`extract`] it.
```mermaid
sequenceDiagram
  participant codemaid_rust as codemaid_rust mod
  codemaid_rust->>codemaid_rust: load_dir(root)?
  codemaid_rust->>codemaid_rust: extract(&sources, &options)
```
