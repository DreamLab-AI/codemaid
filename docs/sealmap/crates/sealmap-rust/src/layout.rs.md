---
sealmap: 1
source: crates/sealmap-rust/src/layout.rs
module: sealmap_rust::layout
language: rust
source_hash: blake3:f9aad6047eec8378d37e7b3b1a67d2ccb5d2ed4f821de4c1318299aef0ff8724
lines: 210
fragments: 4
---
# `sealmap_rust::layout` · crates/sealmap-rust/src/layout.rs
> Mapping files to crates and module paths using Cargo conventions.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_rust__layout__FileRole["FileRole"] {
    <<struct>>
    +crate_name: String
    +module: Vec#lt;String#gt;
    +target: TargetKind
  }
  class sealmap_rust__layout__Package["Package"] {
    <<struct>>
    -dir: String
    -name: String
    -has_lib: bool
    -lib_path: Option#lt;String#gt;
  }
  class sealmap_rust__layout__TargetKind["TargetKind"] {
    <<enum>>
    Lib
    Bin
    Test
    Example
    Bench
  }
  class sealmap_rust__layout["sealmap_rust::layout"] {
    <<module>>
    -join(dir: &str, p: &str) String
    -module_from_rel(krate: &str, rel: &str) Vec#lt;String#gt;
    ~plan(crate) BTreeMap#lt;SourcePath, FileRole#gt;
    -role_in_package(pkg: &Package, rel: &str, full: &str) Option#lt;FileRole#gt;
    -sanitize(s: &str) String
  }
  class sealmap_model__path__SourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__source__SourceSet["SourceSet"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  sealmap_rust__layout ..> sealmap_model__path__SourcePath
  sealmap_rust__layout ..> sealmap_model__source__SourceSet
  sealmap_rust__layout ..> sealmap_rust__layout__FileRole
  sealmap_rust__layout ..> sealmap_rust__layout__Package
  sealmap_rust__layout__FileRole *-- sealmap_rust__layout__TargetKind : target
```

## `sealmap_rust::layout::plan`
`pub(crate) fn plan(sources: &SourceSet, fallback: &str) -> BTreeMap<SourcePath, FileRole>` · L36-L86
> Discover packages from every `Cargo.toml` with a `[package]` table.
```mermaid
sequenceDiagram
  participant sealmap_rust__layout as layout mod
  participant sealmap_model__source__SourceSet as SourceSet
  participant sealmap_model__path__SourcePath as SourcePath
  sealmap_rust__layout->>sealmap_model__source__SourceSet: iter()
  loop for (path, text) in sources.iter()
    opt via and_then
      sealmap_rust__layout->>sealmap_rust__layout: join(dir, p)
      sealmap_rust__layout->>sealmap_model__path__SourcePath: SourcePath::new(join())
    end
    sealmap_rust__layout->>sealmap_rust__layout: join(dir, #quot;src/lib.rs#quot;)
    sealmap_rust__layout->>sealmap_model__source__SourceSet: get_str(&default_lib)
  end
  sealmap_rust__layout->>sealmap_model__source__SourceSet: paths()
  loop for path in sources.paths()
    alt Some(pkg)
      sealmap_rust__layout->>sealmap_rust__layout: role_in_package(pkg, rel, p)
    else None
      sealmap_rust__layout->>sealmap_rust__layout: sanitize(fallback)
      sealmap_rust__layout->>sealmap_rust__layout: sanitize(fallback)
      sealmap_rust__layout->>sealmap_rust__layout: module_from_rel(&sanitize(), unwrap_or())
    end
  end
```

## `sealmap_rust::layout::role_in_package`
`fn role_in_package(pkg: &Package, rel: &str, full: &str) -> Option<FileRole>` · L88-L138
```mermaid
sequenceDiagram
  participant sealmap_rust__layout as layout mod
  opt let Some(lib) = &pkg.lib_path
    opt lib == full
      Note over sealmap_rust__layout: return Some(FileRole { crate_name: pkg.name.clone(), mo…
    end
  end
  opt let Some(rest) = rel.strip_prefix(#quot;src/#quot;)
    opt rest == #quot;main.rs#quot;
      Note over sealmap_rust__layout: return Some(FileRole { crate_name: name.clone(), module…
    end
    sealmap_rust__layout->>sealmap_rust__layout: module_from_rel(&name, rest)
    Note over sealmap_rust__layout: return Some(FileRole { crate_name: name.clone(), module…
  end
  sealmap_rust__layout->>sealmap_rust__layout: sanitize(&_)
  opt Some(t)
    sealmap_rust__layout->>sealmap_rust__layout: module_from_rel(&crate_name, t)
  end
```

## `sealmap_rust::layout::module_from_rel`
`fn module_from_rel(krate: &str, rel: &str) -> Vec<String>` · L140-L156
> `a/b.rs` → `[krate, a, b]`; `a/mod.rs` → `[krate, a]`; `lib.rs` → `[krate]`.
```mermaid
sequenceDiagram
  participant sealmap_rust__layout as layout mod
  loop for (i, part) in parts.iter().enumerate()
    alt last
      opt !(stem == #quot;mod#quot; || (i == 0 && (stem == #quot;lib#quot; || stem…
        sealmap_rust__layout->>sealmap_rust__layout: sanitize(stem)
      end
    else
      sealmap_rust__layout->>sealmap_rust__layout: sanitize(part)
    end
  end
```
