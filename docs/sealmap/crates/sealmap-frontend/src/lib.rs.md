---
sealmap: 1
source: crates/sealmap-frontend/src/lib.rs
module: sealmap_frontend
language: rust
source_hash: blake3:415a52feeeac8ba8ca4bd453a257b72faa69663e114fe63ef4fd90e5d752c4d1
lines: 83
fragments: 1
---
# `sealmap_frontend` · crates/sealmap-frontend/src/lib.rs
> The language-neutral half of a sealmap frontend.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_frontend__Diagnostic["Diagnostic"] {
    <<struct>>
    +file: SourcePath
    +message: String
  }
  class sealmap_frontend__Extraction["Extraction"] {
    <<struct>>
    +codebase: Codebase
    +diagnostics: Vec#lt;Diagnostic#gt;
  }
  class sealmap_frontend__ReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
  }
  class sealmap_frontend {
    <<module>>
    +mod confidence
    +mod ids
    +mod isolate
    +mod labels
    +mod lower
    +mod raw
  }
  class sealmap_model__path__SourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  sealmap_frontend__Diagnostic *-- sealmap_model__path__SourcePath : file
  sealmap_frontend__Extraction o-- sealmap_frontend__Diagnostic : diagnostics
  sealmap_frontend__Extraction *-- sealmap_model__codebase__Codebase : codebase
```
