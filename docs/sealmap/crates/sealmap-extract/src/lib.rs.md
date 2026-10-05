---
sealmap: 2
source: crates/sealmap-extract/src/lib.rs
module: "sym:cargo sealmap_extract ."
language: rust
source_hash: blake3:aaf26fdcb4e50c9630c0cfa55d251de38fb4160ad4d4f9683d824b947b5cdadf
lines: 86
fragments: 1
---
# `sym:cargo sealmap_extract .` · crates/sealmap-extract/src/lib.rs
> The language-neutral half of a sealmap language adapter.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_extract___tDiagnostic["Diagnostic"] {
    <<struct>>
    +file: SourcePath
    +message: String
  }
  class sealmap_extract___tExtraction["Extraction"] {
    <<struct>>
    +codebase: Codebase
    +diagnostics: Vec#lt;Diagnostic#gt;
  }
  class sealmap_extract___tReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
  }
  class sealmap_extract {
    <<module>>
    +mod confidence
    +mod fingerprint
    +mod ids
    +mod isolate
    +mod labels
    +mod lower
    +mod raw
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  sealmap_extract___tDiagnostic *-- sealmap_model__path___tSourcePath : file
  sealmap_extract___tExtraction o-- sealmap_extract___tDiagnostic : diagnostics
  sealmap_extract___tExtraction *-- sealmap_model__codebase___tCodebase : codebase
```
