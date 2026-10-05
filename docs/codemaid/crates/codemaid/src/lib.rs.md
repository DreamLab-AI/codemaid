---
codemaid: 1
source: crates/codemaid/src/lib.rs
module: codemaid
language: rust
source_hash: blake3:0e1c823db198557689cc5039489b93d7a8acf02d5a8470126913ac30534bef81
lines: 116
fragments: 4
---
# `codemaid` · crates/codemaid/src/lib.rs
> Deterministic, dense **codebase → Mermaid** generation for LLM agents.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid__Options["Options"] {
    <<struct>>
    +rust: rust::RustOptions
    +corpus: CorpusOptions
  }
  class codemaid {
    <<module>>
    +generate_dir(root: &Path, options: &Options) io::Result#lt;Corpus#gt;
    +generate_repos(repos: &[#40;&str, &Path#41;], options: &Options) io::Result#lt;Corpus#gt;
    +load_repos(repos: &[#40;&str, &Path#41;]) io::Result#lt;SourceSet#gt;
  }
  class codemaid_corpus__Corpus["Corpus"] {
    <<struct in crates/codemaid-corpus/src/lib.rs>>
  }
  class codemaid_model__source__SourceSet["SourceSet"] {
    <<struct in crates/codemaid-model/src/source.rs>>
  }
  class codemaid_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/codemaid-corpus/src/lib.rs>>
  }
  class codemaid_rust__RustOptions["RustOptions"] {
    <<struct in crates/codemaid-rust/src/lib.rs>>
  }
  codemaid ..> codemaid__Options
  codemaid ..> codemaid_corpus__Corpus
  codemaid ..> codemaid_model__source__SourceSet
  codemaid__Options *-- codemaid_corpus__CorpusOptions : corpus
  codemaid__Options *-- codemaid_rust__RustOptions : rust
```

## `codemaid::generate_dir`
`pub fn generate_dir(root: &Path, options: &Options) -> io::Result<Corpus>` · L84-L88
> Load, extract and project a single directory.
```mermaid
sequenceDiagram
  participant codemaid as codemaid mod
  participant codemaid_rust as codemaid_rust mod
  participant codemaid_corpus as codemaid_corpus mod
  codemaid->>codemaid_rust: rust::extract_dir(root, &options.rust)?
  codemaid->>codemaid_corpus: generate(&extraction.codebase, &options.corpus)
```

## `codemaid::generate_repos`
`pub fn generate_repos(repos: &[(&str, &Path)], options: &Options) -> io::Result<Corpus>` · L90-L101
> Load several repositories as one codebase.
```mermaid
sequenceDiagram
  participant codemaid as codemaid mod
  participant codemaid_rust as codemaid_rust mod
  participant codemaid_corpus as codemaid_corpus mod
  codemaid->>codemaid: load_repos(repos)?
  codemaid->>codemaid_rust: rust::extract(&sources, &ro)
  codemaid->>codemaid_corpus: generate(&extraction.codebase, &options.corpus)
```

## `codemaid::load_repos`
`pub fn load_repos(repos: &[(&str, &Path)]) -> io::Result<SourceSet>` · L103-L116
> Load several repositories into one [`SourceSet`] under `<name>/` prefixes.
```mermaid
sequenceDiagram
  participant codemaid as codemaid mod
  participant codemaid_model__source__SourceSet as SourceSet
  codemaid->>codemaid_model__source__SourceSet: SourceSet::new()
  loop for (name, path) in repos
    codemaid->>codemaid_model__source__SourceSet: SourceSet::load_dir(path, &load)?
    codemaid->>codemaid_model__source__SourceSet: iter()
    loop for (p, text) in set.iter()
      opt p.extension() == Some(#quot;rs#quot;) || p.file_name() == #quot;Car…
        codemaid->>codemaid_model__source__SourceSet: insert(_, text)
      end
    end
  end
```
