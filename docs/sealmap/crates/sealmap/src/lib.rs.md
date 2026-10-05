---
sealmap: 1
source: crates/sealmap/src/lib.rs
module: sealmap
language: rust
source_hash: blake3:4362718b2cb8c4875bb678b1f82f43f82dccce57bd9bb30aec1c2a29fb295be0
lines: 124
fragments: 4
---
# `sealmap` · crates/sealmap/src/lib.rs
> Deterministic, dense **codebase → Mermaid** generation for LLM agents.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap__Options["Options"] {
    <<struct>>
    +rust: rust::RustOptions
    +corpus: CorpusOptions
  }
  class sealmap__ReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
  }
  class sealmap {
    <<module>>
    +generate_dir(root: &Path, options: &Options) io::Result#lt;Corpus#gt;
    +generate_repos(repos: &[#40;&str, &Path#41;], options: &Options) io::Result#lt;Corpus#gt;
    +load_repos(repos: &[#40;&str, &Path#41;]) io::Result#lt;SourceSet#gt;
  }
  class sealmap_corpus__Corpus["Corpus"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_model__source__SourceSet["SourceSet"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_rust__RustOptions["RustOptions"] {
    <<struct in crates/sealmap-rust/src/lib.rs>>
  }
  sealmap ..> sealmap__Options
  sealmap ..> sealmap_corpus__Corpus
  sealmap ..> sealmap_model__source__SourceSet
  sealmap__Options *-- sealmap_corpus__CorpusOptions : corpus
  sealmap__Options *-- sealmap_rust__RustOptions : rust
```

## `sealmap::generate_dir`
`pub fn generate_dir(root: &Path, options: &Options) -> io::Result<Corpus>` · L87-L91
> Load, extract and project a single directory.
```mermaid
sequenceDiagram
  participant sealmap as sealmap mod
  participant sealmap_rust as sealmap_rust mod
  participant sealmap_corpus as sealmap_corpus mod
  sealmap->>sealmap_rust: rust::extract_dir(root, &options.rust)?
  sealmap->>sealmap_corpus: generate(&extraction.codebase, &options.corpus)
```

## `sealmap::generate_repos`
`pub fn generate_repos(repos: &[(&str, &Path)], options: &Options) -> io::Result<Corpus>` · L93-L104
> Load several repositories as one codebase.
```mermaid
sequenceDiagram
  participant sealmap as sealmap mod
  participant sealmap_rust as sealmap_rust mod
  participant sealmap_corpus as sealmap_corpus mod
  sealmap->>sealmap: load_repos(repos)?
  sealmap->>sealmap_rust: rust::extract(&sources, &ro)
  sealmap->>sealmap_corpus: generate(&extraction.codebase, &options.corpus)
```

## `sealmap::load_repos`
`pub fn load_repos(repos: &[(&str, &Path)]) -> io::Result<SourceSet>` · L106-L119
> Load several repositories into one [`SourceSet`] under `<name>/` prefixes.
```mermaid
sequenceDiagram
  participant sealmap as sealmap mod
  participant sealmap_model__source__SourceSet as SourceSet
  sealmap->>sealmap_model__source__SourceSet: SourceSet::new()
  loop for (name, path) in repos
    sealmap->>sealmap_model__source__SourceSet: SourceSet::load_dir(path, &load)?
    sealmap->>sealmap_model__source__SourceSet: iter()
    loop for (p, text) in set.iter()
      opt p.extension() == Some(#quot;rs#quot;) || p.file_name() == #quot;Car…
        sealmap->>sealmap_model__source__SourceSet: insert(_, text)
      end
    end
  end
```
