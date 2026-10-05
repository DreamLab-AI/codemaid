---
sealmap: 1
source: crates/sealmap-rust/src/lib.rs
module: sealmap_rust
language: rust
source_hash: blake3:93366a10a34532f64b47901a922f1e89aaa920d4a1009e99a41b838c9530ee62
lines: 184
fragments: 5
---
# `sealmap_rust` · crates/sealmap-rust/src/lib.rs
> A pure-Rust frontend that turns Rust source into a [`sealmap_model::Codebase`]: modules, types, traits, functions and methods, the relations between them, and …

## structure
```mermaid
classDiagram
  direction LR
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
  class sealmap_frontend__Extraction["Extraction"] {
    <<struct in crates/sealmap-frontend/src/lib.rs>>
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
  class sealmap_rust__layout__FileRole["FileRole"] {
    <<struct in crates/sealmap-rust/src/layout.rs>>
  }
  class sealmap_frontend__confidence__ExternalCalls["ExternalCalls"] {
    <<enum in crates/sealmap-frontend/src/confidence.rs>>
  }
  sealmap_rust ..> sealmap_frontend__Extraction
  sealmap_rust ..> sealmap_model__source__SourceSet
  sealmap_rust ..> sealmap_rust__Job
  sealmap_rust ..> sealmap_rust__RustOptions
  sealmap_rust ..> sealmap_rust__raw__RawFile
  sealmap_rust__Job ..> sealmap_model__path__SourcePath
  sealmap_rust__Job ..> sealmap_rust__layout__FileRole
  sealmap_rust__RustOptions *-- sealmap_frontend__confidence__ExternalCalls : external_calls
```

## `sealmap_rust::extract`
`pub fn extract(sources: &SourceSet, options: &RustOptions) -> Extraction` · L127-L142
> Extract a [`Codebase`](sealmap_model::Codebase) from the `.rs` (and `Cargo.toml`) files in `sources`.
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
`fn collect_all(jobs: &[Job<'_>], options: &RustOptions) -> Vec<raw::RawFile>` · L146-L156
> Pass 1 over every file, isolated per file (big stacks, panic guard; see [`sealmap_frontend::isolate`]).
```mermaid
sequenceDiagram
  participant sealmap_rust as sealmap_rust mod
  participant sealmap_frontend__isolate as isolate mod
  participant sealmap_rust__collect as collect mod
  sealmap_rust->>sealmap_frontend__isolate: map_isolated(jobs, COLLECT_STACK_BYTES, |..|, |..|)
  opt via map_isolated
    sealmap_rust->>sealmap_rust__collect: collect::collect_file(p, r, t, options)
  end
  opt via map_isolated
    sealmap_rust->>sealmap_rust__collect: collect::failed_file(p, r, t, _)
  end
```

## `sealmap_rust::load_dir`
`pub fn load_dir(root: &Path) -> std::io::Result<SourceSet>` · L158-L166
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
`pub fn extract_dir(root: &Path, options: &RustOptions) -> std::io::Result<(SourceSet, Extraction)>` · L168-L184
> Load `root` from disk (`.rs` and `Cargo.toml` files, skipping `target/`, hidden directories and the like) and [`extract`] it.
```mermaid
sequenceDiagram
  participant sealmap_rust as sealmap_rust mod
  sealmap_rust->>sealmap_rust: load_dir(root)?
  sealmap_rust->>sealmap_rust: extract(&sources, &options)
```
