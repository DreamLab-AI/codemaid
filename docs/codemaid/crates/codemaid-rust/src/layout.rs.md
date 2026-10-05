---
codemaid: 1
source: crates/codemaid-rust/src/layout.rs
module: codemaid_rust::layout
language: rust
source_hash: blake3:a2b43c7a72c6d7f99d0da6c81efec9e118737fc3756f6c630416be728a3390a9
lines: 210
fragments: 4
---
# `codemaid_rust::layout` · crates/codemaid-rust/src/layout.rs
> Mapping files to crates and module paths using Cargo conventions.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_rust__layout__FileRole["FileRole"] {
    <<struct>>
    +crate_name: String
    +module: Vec#lt;String#gt;
    +target: TargetKind
  }
  class codemaid_rust__layout__Package["Package"] {
    <<struct>>
    -dir: String
    -name: String
    -has_lib: bool
    -lib_path: Option#lt;String#gt;
  }
  class codemaid_rust__layout__TargetKind["TargetKind"] {
    <<enum>>
    Lib
    Bin
    Test
    Example
    Bench
  }
  class codemaid_rust__layout["codemaid_rust::layout"] {
    <<module>>
    -join(dir: &str, p: &str) String
    -module_from_rel(krate: &str, rel: &str) Vec#lt;String#gt;
    ~plan(crate) BTreeMap#lt;SourcePath, FileRole#gt;
    -role_in_package(pkg: &Package, rel: &str, full: &str) Option#lt;FileRole#gt;
    -sanitize(s: &str) String
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class codemaid_model__source__SourceSet["SourceSet"] {
    <<struct in crates/codemaid-model/src/source.rs>>
  }
  codemaid_rust__layout ..> codemaid_model__path__SourcePath
  codemaid_rust__layout ..> codemaid_model__source__SourceSet
  codemaid_rust__layout ..> codemaid_rust__layout__FileRole
  codemaid_rust__layout ..> codemaid_rust__layout__Package
  codemaid_rust__layout__FileRole *-- codemaid_rust__layout__TargetKind : target
```

## `codemaid_rust::layout::plan`
`pub(crate) fn plan(sources: &SourceSet, fallback: &str) -> BTreeMap<SourcePath, FileRole>` · L36-L86
> Discover packages from every `Cargo.toml` with a `[package]` table.
```mermaid
sequenceDiagram
  participant codemaid_rust__layout as layout mod
  participant codemaid_model__source__SourceSet as SourceSet
  participant codemaid_model__path__SourcePath as SourcePath
  codemaid_rust__layout->>codemaid_model__source__SourceSet: iter()
  loop for (path, text) in sources.iter()
    opt via and_then
      codemaid_rust__layout->>codemaid_rust__layout: join(dir, p)
      codemaid_rust__layout->>codemaid_model__path__SourcePath: SourcePath::new(join())
    end
    codemaid_rust__layout->>codemaid_rust__layout: join(dir, #quot;src/lib.rs#quot;)
    codemaid_rust__layout->>codemaid_model__source__SourceSet: get_str(&default_lib)
  end
  codemaid_rust__layout->>codemaid_model__source__SourceSet: paths()
  loop for path in sources.paths()
    alt Some(pkg)
      codemaid_rust__layout->>codemaid_rust__layout: role_in_package(pkg, rel, p)
    else None
      codemaid_rust__layout->>codemaid_rust__layout: sanitize(fallback)
      codemaid_rust__layout->>codemaid_rust__layout: sanitize(fallback)
      codemaid_rust__layout->>codemaid_rust__layout: module_from_rel(&sanitize(), unwrap_or())
    end
  end
```

## `codemaid_rust::layout::role_in_package`
`fn role_in_package(pkg: &Package, rel: &str, full: &str) -> Option<FileRole>` · L88-L138
```mermaid
sequenceDiagram
  participant codemaid_rust__layout as layout mod
  opt let Some(lib) = &pkg.lib_path
    opt lib == full
      Note over codemaid_rust__layout: return Some(FileRole { crate_name: pkg.name.clone(), mo…
    end
  end
  opt let Some(rest) = rel.strip_prefix(#quot;src/#quot;)
    opt rest == #quot;main.rs#quot;
      Note over codemaid_rust__layout: return Some(FileRole { crate_name: name.clone(), module…
    end
    codemaid_rust__layout->>codemaid_rust__layout: module_from_rel(&name, rest)
    Note over codemaid_rust__layout: return Some(FileRole { crate_name: name.clone(), module…
  end
  codemaid_rust__layout->>codemaid_rust__layout: sanitize(&_)
  opt Some(t)
    codemaid_rust__layout->>codemaid_rust__layout: module_from_rel(&crate_name, t)
  end
```

## `codemaid_rust::layout::module_from_rel`
`fn module_from_rel(krate: &str, rel: &str) -> Vec<String>` · L140-L156
> `a/b.rs` → `[krate, a, b]`; `a/mod.rs` → `[krate, a]`; `lib.rs` → `[krate]`.
```mermaid
sequenceDiagram
  participant codemaid_rust__layout as layout mod
  loop for (i, part) in parts.iter().enumerate()
    alt last
      opt !(stem == #quot;mod#quot; || (i == 0 && (stem == #quot;lib#quot; || stem…
        codemaid_rust__layout->>codemaid_rust__layout: sanitize(stem)
      end
    else
      codemaid_rust__layout->>codemaid_rust__layout: sanitize(part)
    end
  end
```
