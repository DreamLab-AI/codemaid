---
sealmap: 2
source: crates/sealmap-rust/src/lib.rs
module: "sym:cargo sealmap_rust ."
language: rust
source_hash: blake3:d040c9f637192fd8d0baf869a1f1470c8bc2e5fb280cb6f9cce8715b3fa38ec6
lines: 210
fragments: 5
---
# `sym:cargo sealmap_rust .` · crates/sealmap-rust/src/lib.rs
> A pure-Rust frontend that turns Rust source into a [`sealmap_model::Codebase`]: modules, types, traits, functions and methods, the relations between them, and …

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_rust___tJob["Job#lt;'a#gt;"] {
    <<type>>
  }
  class sealmap_rust___tReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
  }
  class sealmap_rust___tRustOptions["RustOptions"] {
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
    +mod fingerprint
    +mod layout
    +load_dir(root: &Path) std::io::Result#lt;SourceSet#gt;
    +mod raw
    +mod resolve
    +mod tidy
  }
  class sealmap_frontend___tExtraction["Extraction"] {
    <<struct in crates/sealmap-frontend/src/lib.rs>>
  }
  class sealmap_model__source___tSourceSet["SourceSet"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_rust__raw___tRawFile["RawFile"] {
    <<struct in crates/sealmap-rust/src/raw.rs>>
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_rust__layout___tFileRole["FileRole"] {
    <<struct in crates/sealmap-rust/src/layout.rs>>
  }
  class sealmap_frontend__confidence___tExternalCalls["ExternalCalls"] {
    <<enum in crates/sealmap-frontend/src/confidence.rs>>
  }
  sealmap_rust ..> sealmap_frontend___tExtraction
  sealmap_rust ..> sealmap_model__source___tSourceSet
  sealmap_rust ..> sealmap_rust___tJob
  sealmap_rust ..> sealmap_rust___tRustOptions
  sealmap_rust ..> sealmap_rust__raw___tRawFile
  sealmap_rust___tJob ..> sealmap_model__path___tSourcePath
  sealmap_rust___tJob ..> sealmap_rust__layout___tFileRole
  sealmap_rust___tRustOptions *-- sealmap_frontend__confidence___tExternalCalls : external_calls
```

## `sym:cargo sealmap_rust . extract().`
`pub fn extract(sources: &SourceSet, options: &RustOptions) -> Extraction` · L148-L163
> Extract a [`Codebase`](sealmap_model::Codebase) from the `.rs` (and `Cargo.toml`) files in `sources`.
```mermaid
sequenceDiagram
  participant sealmap_rust as sealmap_rust mod
  participant sealmap_rust__layout as layout mod
  participant sealmap_model__source___tSourceSet as SourceSet
  participant sealmap_rust__resolve as resolve mod
  sealmap_rust->>sealmap_rust__layout: layout::plan(sources, &options.name)
  loop each via filter_map
    sealmap_rust->>sealmap_model__source___tSourceSet: get(&path)
  end
  sealmap_rust->>sealmap_rust: collect_all(&jobs, options)
  sealmap_rust->>sealmap_rust__resolve: resolve::build(&options.name, files, options)
```

## `sym:cargo sealmap_rust . collect_all().`
`fn collect_all(jobs: &[Job<'_>], options: &RustOptions) -> Vec<raw::RawFile>` · L167-L177
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

## `sym:cargo sealmap_rust . load_dir().`
`pub fn load_dir(root: &Path) -> std::io::Result<SourceSet>` · L179-L187
> Load the `.rs` and `Cargo.toml` files under `root` (honouring `.gitignore`, skipping `target/`, hidden directories and the like) without extracting them.
```mermaid
sequenceDiagram
  participant sealmap_rust as sealmap_rust mod
  participant sealmap_model__source___tLoadOptions as LoadOptions
  participant sealmap_model__source___tSourceSet as SourceSet
  sealmap_rust->>sealmap_model__source___tLoadOptions: LoadOptions::default()
  sealmap_rust->>sealmap_model__source___tSourceSet: SourceSet::load_dir(root, &load)?
  sealmap_rust->>sealmap_model__source___tSourceSet: retain(|..|)
```

## `sym:cargo sealmap_rust . extract_dir().`
`pub fn extract_dir(root: &Path, options: &RustOptions) -> std::io::Result<(SourceSet, Extraction)>` · L189-L205
> Load `root` from disk (`.rs` and `Cargo.toml` files, skipping `target/`, hidden directories and the like) and [`extract`] it.
```mermaid
sequenceDiagram
  participant sealmap_rust as sealmap_rust mod
  sealmap_rust->>sealmap_rust: load_dir(root)?
  sealmap_rust->>sealmap_rust: extract(&sources, &options)
```
