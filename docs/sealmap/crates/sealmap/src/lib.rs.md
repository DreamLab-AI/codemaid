---
sealmap: 2
source: crates/sealmap/src/lib.rs
module: "sym:cargo sealmap ."
language: rust
source_hash: blake3:d93889aaafd0d90342dd7d632663bd02de942a30c2b38af6ff545a646b245b45
lines: 131
fragments: 4
---
# `sym:cargo sealmap .` · crates/sealmap/src/lib.rs
> Deterministic, dense **codebase → Mermaid** generation for LLM agents.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap___tOptions["Options"] {
    <<struct>>
    +rust: rust::RustOptions
    +corpus: CorpusOptions
  }
  class sealmap___tReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
  }
  class sealmap {
    <<module>>
    +generate_dir(root: &Path, options: &Options) io::Result#lt;Corpus#gt;
    +generate_repos(repos: &[#40;&str, &Path#41;], options: &Options) io::Result#lt;Corpus#gt;
    +load_repos(repos: &[#40;&str, &Path#41;]) io::Result#lt;SourceSet#gt;
  }
  class sealmap_corpus___tCorpus["Corpus"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_model__source___tSourceSet["SourceSet"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_corpus___tCorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_rust___tRustOptions["RustOptions"] {
    <<struct in crates/sealmap-rust/src/lib.rs>>
  }
  sealmap ..> sealmap___tOptions
  sealmap ..> sealmap_corpus___tCorpus
  sealmap ..> sealmap_model__source___tSourceSet
  sealmap___tOptions *-- sealmap_corpus___tCorpusOptions : corpus
  sealmap___tOptions *-- sealmap_rust___tRustOptions : rust
```

## `sym:cargo sealmap . generate_dir().`
`pub fn generate_dir(root: &Path, options: &Options) -> io::Result<Corpus>` · L94-L98
> Load, extract and project a single directory.
```mermaid
sequenceDiagram
  participant sealmap as sealmap mod
  participant sealmap_rust as sealmap_rust mod
  participant sealmap_corpus as sealmap_corpus mod
  sealmap->>sealmap_rust: rust::extract_dir(root, &options.rust)?
  sealmap->>sealmap_corpus: generate(&extraction.codebase, &options.corpus)
```

## `sym:cargo sealmap . generate_repos().`
`pub fn generate_repos(repos: &[(&str, &Path)], options: &Options) -> io::Result<Corpus>` · L100-L111
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

## `sym:cargo sealmap . load_repos().`
`pub fn load_repos(repos: &[(&str, &Path)]) -> io::Result<SourceSet>` · L113-L126
> Load several repositories into one [`SourceSet`] under `<name>/` prefixes.
```mermaid
sequenceDiagram
  participant sealmap as sealmap mod
  participant sealmap_model__source___tSourceSet as SourceSet
  sealmap->>sealmap_model__source___tSourceSet: SourceSet::new()
  loop for (name, path) in repos
    sealmap->>sealmap_model__source___tSourceSet: SourceSet::load_dir(path, &load)?
    sealmap->>sealmap_model__source___tSourceSet: iter()
    loop for (p, text) in set.iter()
      opt p.extension() == Some(#quot;rs#quot;) || p.file_name() == #quot;Car…
        sealmap->>sealmap_model__source___tSourceSet: insert(_, text)
      end
    end
  end
```
