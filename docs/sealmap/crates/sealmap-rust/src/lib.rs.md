---
sealmap: 1
source: crates/sealmap-rust/src/lib.rs
module: sealmap_rust
language: rust
source_hash: blake3:43849a51fddc77d8c4790358733b888b1e27f54a3faa58de015b90476beba3fd
lines: 246
fragments: 5
---
# `sealmap_rust` · crates/sealmap-rust/src/lib.rs
> A pure-Rust frontend that turns Rust source into a [`sealmap_model::Codebase`]: modules, types, traits, functions and methods, the relations between them, and …

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_rust__Diagnostic["Diagnostic"] {
    <<struct>>
    +file: SourcePath
    +message: String
  }
  class sealmap_rust__ExternalCalls["ExternalCalls"] {
    <<enum>>
    All
    NonStd
    None
  }
  class sealmap_rust__Extraction["Extraction"] {
    <<struct>>
    +codebase: Codebase
    +diagnostics: Vec#lt;Diagnostic#gt;
  }
  class sealmap_rust__Job["Job#lt;'a#gt;"] {
    <<type>>
  }
  class sealmap_rust__RustOptions["RustOptions"] {
    <<struct>>
    +name: String
    +include_tests: bool
    +external_calls: ExternalCalls
    +Default::default() Self
  }
  class sealmap_rust {
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
  class sealmap_model__source__SourceSet["SourceSet"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_rust__raw__RawFile["RawFile"] {
    <<struct in crates/sealmap-rust/src/raw.rs>>
  }
  class sealmap_model__path__SourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_rust__layout__FileRole["FileRole"] {
    <<struct in crates/sealmap-rust/src/layout.rs>>
  }
  sealmap_rust ..> sealmap_model__source__SourceSet
  sealmap_rust ..> sealmap_rust__Extraction
  sealmap_rust ..> sealmap_rust__Job
  sealmap_rust ..> sealmap_rust__RustOptions
  sealmap_rust ..> sealmap_rust__raw__RawFile
  sealmap_rust__Diagnostic *-- sealmap_model__path__SourcePath : file
  sealmap_rust__Extraction *-- sealmap_model__codebase__Codebase : codebase
  sealmap_rust__Extraction o-- sealmap_rust__Diagnostic : diagnostics
  sealmap_rust__Job ..> sealmap_model__path__SourcePath
  sealmap_rust__Job ..> sealmap_rust__layout__FileRole
  sealmap_rust__RustOptions *-- sealmap_rust__ExternalCalls : external_calls
```

## `sealmap_rust::extract`
`pub fn extract(sources: &SourceSet, options: &RustOptions) -> Extraction` · L158-L173
> Extract a [`Codebase`] from the `.rs` (and `Cargo.toml`) files in `sources`.
```mermaid
sequenceDiagram
  participant sealmap_rust as sealmap_rust mod
  participant sealmap_rust__layout as layout mod
  participant sealmap_model__source__SourceSet as SourceSet
  participant sealmap_rust__resolve as resolve mod
  sealmap_rust->>sealmap_rust__layout: layout::plan(sources, &options.name)
  loop each via filter_map
    sealmap_rust->>sealmap_model__source__SourceSet: get(&path)
  end
  sealmap_rust->>sealmap_rust: collect_all(&jobs, options)
  sealmap_rust->>sealmap_rust__resolve: resolve::build(&options.name, files, options)
```

## `sealmap_rust::collect_all`
`fn collect_all(jobs: &[Job<'_>], options: &RustOptions) -> Vec<raw::RawFile>` · L185-L218
> Pass 1 over every file, on threads with [`COLLECT_STACK_BYTES`] of stack.
```mermaid
sequenceDiagram
  participant sealmap_rust as sealmap_rust mod
  participant sealmap_rust__collect as collect mod
  participant rayon as rayon ext
  opt closure
    opt closure
      sealmap_rust->>sealmap_rust__collect: collect::collect_file(p, r, t, options)
    end
    opt via unwrap_or_else
      sealmap_rust->>sealmap_rust__collect: collect::failed_file(p, r, t, _)
    end
  end
  sealmap_rust->>rayon: ThreadPoolBuilder::ThreadPoolBuilder::new()
  opt let Ok(pool) = rayon::ThreadPoolBuilder::new().stack…
    Note over sealmap_rust: return pool.install(| | jobs.par_iter().map(one).collec…
  end
```

## `sealmap_rust::load_dir`
`pub fn load_dir(root: &Path) -> std::io::Result<SourceSet>` · L220-L228
> Load the `.rs` and `Cargo.toml` files under `root` (honouring `.gitignore`, skipping `target/`, hidden directories and the like) without extracting them.
```mermaid
sequenceDiagram
  participant sealmap_rust as sealmap_rust mod
  participant sealmap_model__source__LoadOptions as LoadOptions
  participant sealmap_model__source__SourceSet as SourceSet
  sealmap_rust->>sealmap_model__source__LoadOptions: LoadOptions::default()
  sealmap_rust->>sealmap_model__source__SourceSet: SourceSet::load_dir(root, &load)?
  sealmap_rust->>sealmap_model__source__SourceSet: retain(|..|)
```

## `sealmap_rust::extract_dir`
`pub fn extract_dir(root: &Path, options: &RustOptions) -> std::io::Result<(SourceSet, Extraction)>` · L230-L246
> Load `root` from disk (`.rs` and `Cargo.toml` files, skipping `target/`, hidden directories and the like) and [`extract`] it.
```mermaid
sequenceDiagram
  participant sealmap_rust as sealmap_rust mod
  sealmap_rust->>sealmap_rust: load_dir(root)?
  sealmap_rust->>sealmap_rust: extract(&sources, &options)
```
