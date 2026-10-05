---
sealmap: 1
source: crates/sealmap-model/src/source.rs
module: sealmap_model::source
language: rust
source_hash: blake3:246c5cf2052103eb63127f2470838188916e55d6e5913571cc1fed18b66ea870
lines: 187
fragments: 5
---
# `sealmap_model::source` · crates/sealmap-model/src/source.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__source__LoadOptions["LoadOptions"] {
    <<struct>>
    +extensions: Vec#lt;String#gt;
    +skip_dirs: Vec#lt;String#gt;
    +max_file_bytes: u64
    +skip_hidden: bool
    +respect_ignore_files: bool
    +Default::default() Self
  }
  class sealmap_model__source__SourceFile["SourceFile"] {
    <<struct>>
    +path: SourcePath
    +language: String
    +module: SymbolId
    +hash: ContentHash
    +lines: u32
    +new(path: SourcePath, language: impl Into#lt;String#gt;, module: SymbolId, text: &str) Self
  }
  class sealmap_model__source__SourceSet["SourceSet"] {
    <<struct>>
    -files: BTreeMap#lt;SourcePath, String#gt;
    +get(&self, path: &SourcePath) Option#lt;&str#gt;
    +get_str(&self, path: &str) Option#lt;&str#gt;
    +insert(&mut self, path: impl AsRef#lt;str#gt;, text: impl AsRef#lt;str#gt;) Result#lt;#40;#41;, crate::PathError#gt;
    +is_empty(&self) bool
    +iter(&self) impl Iterator#lt;Item = #40;&SourcePath, &str#41;#gt;
    +len(&self) usize
    +load_dir(root: &Path, options: &LoadOptions) io::Result#lt;Self#gt;
    +new() Self
    +paths(&self) impl Iterator#lt;Item = &SourcePath#gt;
    +retain(&mut self, mut keep: impl FnMut#40;&SourcePath#41; -> bool)
  }
  class sealmap_model__hash__ContentHash["ContentHash"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__path__SourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__path__PathError["PathError"] {
    <<enum in crates/sealmap-model/src/path.rs>>
  }
  sealmap_model__source__SourceFile *-- sealmap_model__hash__ContentHash : hash
  sealmap_model__source__SourceFile *-- sealmap_model__path__SourcePath : path
  sealmap_model__source__SourceFile *-- sealmap_model__symbol__SymbolId : module
  sealmap_model__source__SourceSet ..> sealmap_model__path__PathError
  sealmap_model__source__SourceSet o-- sealmap_model__path__SourcePath : files
  sealmap_model__source__SourceSet ..> sealmap_model__source__LoadOptions
```

## `sealmap_model::source::SourceFile::new`
`pub fn new(path: SourcePath, language: impl Into<String>, module: SymbolId, text: &str) -> Self` · L30-L39
> Describe `text` as the file at `path`.
```mermaid
sequenceDiagram
  participant sealmap_model__source__SourceFile as SourceFile
  participant sealmap_model__hash__ContentHash as ContentHash
  sealmap_model__source__SourceFile->>sealmap_model__hash__ContentHash: ContentHash::of_text(text)
```

## `sealmap_model::source::SourceSet::insert`
`pub fn insert(&mut self, path: impl AsRef<str>, text: impl AsRef<str>) -> Result<(), crate::PathError>` · L104-L110
> Insert or replace a file.
```mermaid
sequenceDiagram
  participant sealmap_model__source__SourceSet as SourceSet
  participant sealmap_model__path__SourcePath as SourcePath
  participant sealmap_model__hash as hash mod
  sealmap_model__source__SourceSet->>sealmap_model__path__SourcePath: SourcePath::new(path)?
  sealmap_model__source__SourceSet->>sealmap_model__hash: normalise_newlines(as_ref())
```

## `sealmap_model::source::SourceSet::get_str`
`pub fn get_str(&self, path: &str) -> Option<&str>` · L117-L120
> Text of the file at a raw path string (normalised first).
```mermaid
sequenceDiagram
  participant sealmap_model__source__SourceSet as SourceSet
  participant sealmap_model__path__SourcePath as SourcePath
  sealmap_model__source__SourceSet->>sealmap_model__path__SourcePath: SourcePath::new(path)
  opt via and_then
    sealmap_model__source__SourceSet->>sealmap_model__source__SourceSet: get(&p)
  end
```

## `sealmap_model::source::SourceSet::load_dir`
`pub fn load_dir(root: &Path, options: &LoadOptions) -> io::Result<Self>` · L147-L186
> Recursively load every matching file under `root`.
```mermaid
sequenceDiagram
  participant sealmap_model__source__SourceSet as SourceSet
  participant ignore as ignore ext
  participant sealmap_model__path__SourcePath as SourcePath
  participant sealmap_model__hash as hash mod
  sealmap_model__source__SourceSet->>ignore: WalkBuilder::WalkBuilder::new(root)
  loop for entry in walker
    sealmap_model__source__SourceSet->>sealmap_model__path__SourcePath: SourcePath::relative_to(path, root)
    sealmap_model__source__SourceSet->>sealmap_model__hash: normalise_newlines(&text)
  end
```
