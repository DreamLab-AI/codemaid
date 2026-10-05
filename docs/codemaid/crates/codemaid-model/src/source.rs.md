---
codemaid: 1
source: crates/codemaid-model/src/source.rs
module: codemaid_model::source
language: rust
source_hash: blake3:6e223a4a2875bfde5fccde3148b3057694855178b0e52c6d10311269c3a0b42e
lines: 187
fragments: 5
---
# `codemaid_model::source` · crates/codemaid-model/src/source.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_model__source__LoadOptions["LoadOptions"] {
    <<struct>>
    +extensions: Vec#lt;String#gt;
    +skip_dirs: Vec#lt;String#gt;
    +max_file_bytes: u64
    +skip_hidden: bool
    +respect_ignore_files: bool
    +Default::default() Self
  }
  class codemaid_model__source__SourceFile["SourceFile"] {
    <<struct>>
    +path: SourcePath
    +language: String
    +module: SymbolId
    +hash: ContentHash
    +lines: u32
    +new(path: SourcePath, language: impl Into#lt;String#gt;, module: SymbolId, text: &str) Self
  }
  class codemaid_model__source__SourceSet["SourceSet"] {
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
  class codemaid_model__hash__ContentHash["ContentHash"] {
    <<struct in crates/codemaid-model/src/hash.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class codemaid_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__path__PathError["PathError"] {
    <<enum in crates/codemaid-model/src/path.rs>>
  }
  codemaid_model__source__SourceFile *-- codemaid_model__hash__ContentHash : hash
  codemaid_model__source__SourceFile *-- codemaid_model__path__SourcePath : path
  codemaid_model__source__SourceFile *-- codemaid_model__symbol__SymbolId : module
  codemaid_model__source__SourceSet ..> codemaid_model__path__PathError
  codemaid_model__source__SourceSet o-- codemaid_model__path__SourcePath : files
  codemaid_model__source__SourceSet ..> codemaid_model__source__LoadOptions
```

## `codemaid_model::source::SourceFile::new`
`pub fn new(path: SourcePath, language: impl Into<String>, module: SymbolId, text: &str) -> Self` · L30-L39
> Describe `text` as the file at `path`.
```mermaid
sequenceDiagram
  participant codemaid_model__source__SourceFile as SourceFile
  participant codemaid_model__hash__ContentHash as ContentHash
  codemaid_model__source__SourceFile->>codemaid_model__hash__ContentHash: ContentHash::of_text(text)
```

## `codemaid_model::source::SourceSet::insert`
`pub fn insert(&mut self, path: impl AsRef<str>, text: impl AsRef<str>) -> Result<(), crate::PathError>` · L104-L110
> Insert or replace a file.
```mermaid
sequenceDiagram
  participant codemaid_model__source__SourceSet as SourceSet
  participant codemaid_model__path__SourcePath as SourcePath
  participant codemaid_model__hash as hash mod
  codemaid_model__source__SourceSet->>codemaid_model__path__SourcePath: SourcePath::new(path)?
  codemaid_model__source__SourceSet->>codemaid_model__hash: normalise_newlines(as_ref())
```

## `codemaid_model::source::SourceSet::get_str`
`pub fn get_str(&self, path: &str) -> Option<&str>` · L117-L120
> Text of the file at a raw path string (normalised first).
```mermaid
sequenceDiagram
  participant codemaid_model__source__SourceSet as SourceSet
  participant codemaid_model__path__SourcePath as SourcePath
  codemaid_model__source__SourceSet->>codemaid_model__path__SourcePath: SourcePath::new(path)
  opt via and_then
    codemaid_model__source__SourceSet->>codemaid_model__source__SourceSet: get(&p)
  end
```

## `codemaid_model::source::SourceSet::load_dir`
`pub fn load_dir(root: &Path, options: &LoadOptions) -> io::Result<Self>` · L147-L186
> Recursively load every matching file under `root`.
```mermaid
sequenceDiagram
  participant codemaid_model__source__SourceSet as SourceSet
  participant ignore as ignore ext
  participant codemaid_model__path__SourcePath as SourcePath
  participant codemaid_model__hash as hash mod
  codemaid_model__source__SourceSet->>ignore: WalkBuilder::WalkBuilder::new(root)
  loop for entry in walker
    codemaid_model__source__SourceSet->>codemaid_model__path__SourcePath: SourcePath::relative_to(path, root)
    codemaid_model__source__SourceSet->>codemaid_model__hash: normalise_newlines(&text)
  end
```
