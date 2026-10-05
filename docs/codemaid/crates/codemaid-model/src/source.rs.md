---
codemaid: 1
source: crates/codemaid-model/src/source.rs
module: codemaid_model::source
language: rust
source_hash: blake3:1348d2e4aeb837f6728445aeed4313a5ee2fa44da2210d4f23d722d01ae4c8fc
lines: 179
fragments: 6
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
  class codemaid_model__source["codemaid_model::source"] {
    <<module>>
    -walk(root: &Path, dir: &Path, options: &LoadOptions, set: &mut SourceSet) io::Result#lt;#40;#41;#gt;
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
  codemaid_model__source ..> codemaid_model__source__LoadOptions
  codemaid_model__source ..> codemaid_model__source__SourceSet
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
`pub fn insert(&mut self, path: impl AsRef<str>, text: impl AsRef<str>) -> Result<(), crate::PathError>` · L98-L104
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
`pub fn get_str(&self, path: &str) -> Option<&str>` · L111-L114
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
`pub fn load_dir(root: &Path, options: &LoadOptions) -> io::Result<Self>` · L141-L149
> Recursively load every matching file under `root`.
```mermaid
sequenceDiagram
  participant codemaid_model__source__SourceSet as SourceSet
  participant codemaid_model__source as source mod
  codemaid_model__source__SourceSet->>codemaid_model__source: walk(root, root, options, &set)?
```

## `codemaid_model::source::walk`
`fn walk(root: &Path, dir: &Path, options: &LoadOptions, set: &mut SourceSet) -> io::Result<()>` · L152-L179
```mermaid
sequenceDiagram
  participant codemaid_model__source as source mod
  participant codemaid_model__path__SourcePath as SourcePath
  participant codemaid_model__hash as hash mod
  loop for entry in entries
    alt ty.is_dir()
      opt !options.skip_dirs.iter().any(| d | d == name.as_ref…
        codemaid_model__source->>codemaid_model__source: walk(root, &path, options, set)?
      end
    else if ty.is_file()
      codemaid_model__source->>codemaid_model__path__SourcePath: SourcePath::relative_to(&path, root)
      codemaid_model__source->>codemaid_model__hash: normalise_newlines(&text)
    end
  end
```
