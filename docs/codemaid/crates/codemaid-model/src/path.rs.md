---
codemaid: 1
source: crates/codemaid-model/src/path.rs
module: codemaid_model::path
language: rust
source_hash: blake3:952a318da2fad4ac7f852050b0d88044e1950679d942408a29565f9779cd8885
lines: 150
fragments: 2
---
# `codemaid_model::path` · crates/codemaid-model/src/path.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_model__path__PathError["PathError"] {
    <<enum>>
    Absolute#40;String#41;
    Escapes#40;String#41;
    Empty
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
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
  class String {
    <<external>>
    +From#lt;SourcePath#gt;::from(value: SourcePath) Self
  }
  String ..> codemaid_model__path__SourcePath
  codemaid_model__path__SourcePath ..> codemaid_model__path__PathError
```

## `codemaid_model::path::SourcePath::extension`
`pub fn extension(&self) -> Option<&str>` · L96-L100
> The extension without the dot, if any.
```mermaid
sequenceDiagram
  participant codemaid_model__path__SourcePath as SourcePath
  codemaid_model__path__SourcePath->>codemaid_model__path__SourcePath: file_name()
```
