---
sealmap: 2
source: crates/sealmap-model/src/path.rs
module: "sym:cargo sealmap_model . path/"
language: rust
source_hash: blake3:71ec8f56ee82d18a42af6f151187af0edb43994096c0c523aa4410f031489ee9
lines: 150
fragments: 2
---
# `sym:cargo sealmap_model . path/` · crates/sealmap-model/src/path.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__path___tPathError["PathError"] {
    <<enum>>
    Absolute#40;String#41;
    Escapes#40;String#41;
    Empty
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct>>
    -0: String
    +TryFrom#lt;String#gt;::try_from(value: String) Result#lt;Self, Self::Error#gt;
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    +components(&self) impl Iterator#lt;Item = &str#gt;
    +extension(&self) Option#lt;&str#gt;
    +file_name(&self) &str
    +new(raw: impl AsRef#lt;str#gt;) Result#lt;Self, PathError#gt;
    +relative_to(path: &Path, root: &Path) Result#lt;Self, PathError#gt;
    +with_suffix(&self, suffix: &str) Self
  }
  class _String["String"] {
    <<external>>
    +From#lt;SourcePath#gt;::from(value: SourcePath) Self
  }
  sealmap_model__path___tSourcePath ..> sealmap_model__path___tPathError
  _String ..> sealmap_model__path___tSourcePath
```

## `sym:cargo sealmap_model . path/SourcePath#extension().`
`pub fn extension(&self) -> Option<&str>` · L96-L100
> The extension without the dot, if any.
```mermaid
sequenceDiagram
  participant sealmap_model__path___tSourcePath as SourcePath
  sealmap_model__path___tSourcePath->>sealmap_model__path___tSourcePath: file_name()
```
