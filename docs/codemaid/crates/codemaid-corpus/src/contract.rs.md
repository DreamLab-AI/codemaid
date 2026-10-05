---
codemaid: 1
source: crates/codemaid-corpus/src/contract.rs
module: codemaid_corpus::contract
language: rust
source_hash: blake3:5a32fdb7e6aeda74b51bb646c2a593d3dd04c417556bbae6dce5f129c2350b0d
lines: 183
fragments: 8
---
# `codemaid_corpus::contract` · crates/codemaid-corpus/src/contract.rs
> The 1:1 contract: verify a corpus directory against freshly generated output, and write a directory into compliance.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_corpus__contract__Drift["Drift"] {
    <<enum>>
    Missing
    Orphaned
    Stale
    Modified
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
  }
  class codemaid_corpus__contract__DriftEntry["DriftEntry"] {
    <<struct>>
    +path: SourcePath
    +drift: Drift
  }
  class codemaid_corpus__contract__Report["Report"] {
    <<struct>>
    +entries: Vec#lt;DriftEntry#gt;
    +checked: usize
    +count(&self, drift: Drift) usize
    +is_clean(&self) bool
  }
  class codemaid_corpus__contract["codemaid_corpus::contract"] {
    <<module>>
    +corpus_hash(corpus: &Corpus) ContentHash
    +read_dir_corpus(dir: &Path) io::Result#lt;BTreeMap#lt;SourcePath, String#gt;#gt;
    -read_rec(root: &Path, dir: &Path, out: &mut BTreeMap#lt;SourcePath, String#gt;) io::Result#lt;#40;#41;#gt;
    -remove_empty_parents(root: &Path, file: &Path)
    +verify(dir: &Path, expected: &Corpus) io::Result#lt;Report#gt;
    +verify_against(expected: &Corpus, actual: &BTreeMap#lt;SourcePath, String#gt;) Report
    +write(dir: &Path, expected: &Corpus) io::Result#lt;Report#gt;
  }
  class codemaid_corpus__Corpus["Corpus"] {
    <<struct in crates/codemaid-corpus/src/lib.rs>>
  }
  class codemaid_model__hash__ContentHash["ContentHash"] {
    <<struct in crates/codemaid-model/src/hash.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  codemaid_corpus__contract ..> codemaid_corpus__Corpus
  codemaid_corpus__contract ..> codemaid_corpus__contract__Report
  codemaid_corpus__contract ..> codemaid_model__hash__ContentHash
  codemaid_corpus__contract ..> codemaid_model__path__SourcePath
  codemaid_corpus__contract__DriftEntry *-- codemaid_corpus__contract__Drift : drift
  codemaid_corpus__contract__DriftEntry *-- codemaid_model__path__SourcePath : path
  codemaid_corpus__contract__Report ..> codemaid_corpus__contract__Drift
  codemaid_corpus__contract__Report o-- codemaid_corpus__contract__DriftEntry : entries
```

## `codemaid_corpus::contract::verify_against`
`pub fn verify_against(expected: &Corpus, actual: &BTreeMap<SourcePath, String>) -> Report` · L72-L105
> Compare `expected` (freshly generated) with `actual` (file path → text as found on disk or elsewhere).
```mermaid
sequenceDiagram
  participant codemaid_corpus__contract as contract mod
  participant codemaid_corpus as codemaid_corpus mod
  participant codemaid_corpus__document as document mod
  loop for (path, want) in &expected.files
    opt Some(have)
      codemaid_corpus__contract->>codemaid_corpus: is_reserved(path)
      opt not is_reserved(path)
        codemaid_corpus__contract->>codemaid_corpus__document: front_matter_hash(want)
        codemaid_corpus__contract->>codemaid_corpus__document: front_matter_hash(have)
      end
    end
  end
  loop for (path, text) in actual
    codemaid_corpus__contract->>codemaid_corpus__document: is_generated(text)
  end
```

## `codemaid_corpus::contract::read_dir_corpus`
`pub fn read_dir_corpus(dir: &Path) -> io::Result<BTreeMap<SourcePath, String>>` · L107-L115
> Read every file under `dir` (UTF-8 only) keyed by relative path.
```mermaid
sequenceDiagram
  participant codemaid_corpus__contract as contract mod
  opt dir.exists()
    codemaid_corpus__contract->>codemaid_corpus__contract: read_rec(dir, dir, &out)?
  end
```

## `codemaid_corpus::contract::read_rec`
`fn read_rec(root: &Path, dir: &Path, out: &mut BTreeMap<SourcePath, String>) -> io::Result<()>` · L117-L131
```mermaid
sequenceDiagram
  participant codemaid_corpus__contract as contract mod
  participant codemaid_model__path__SourcePath as SourcePath
  loop for e in entries
    alt ty.is_dir()
      codemaid_corpus__contract->>codemaid_corpus__contract: read_rec(root, &path(), out)?
    else if ty.is_file()
      codemaid_corpus__contract->>codemaid_model__path__SourcePath: SourcePath::relative_to(&path(), root)
    end
  end
```

## `codemaid_corpus::contract::verify`
`pub fn verify(dir: &Path, expected: &Corpus) -> io::Result<Report>` · L133-L136
> Verify the corpus in `dir` against `expected`.
```mermaid
sequenceDiagram
  participant codemaid_corpus__contract as contract mod
  codemaid_corpus__contract->>codemaid_corpus__contract: read_dir_corpus(dir)?
  codemaid_corpus__contract->>codemaid_corpus__contract: verify_against(expected, &_)
```

## `codemaid_corpus::contract::write`
`pub fn write(dir: &Path, expected: &Corpus) -> io::Result<Report>` · L138-L160
> Bring `dir` into compliance with `expected`: write missing, stale and modified files, delete orphaned generated documents (and nothing else).
```mermaid
sequenceDiagram
  participant codemaid_corpus__contract as contract mod
  codemaid_corpus__contract->>codemaid_corpus__contract: verify(dir, expected)?
  loop for e in &report.entries
    opt Drift::Orphaned
      codemaid_corpus__contract->>codemaid_corpus__contract: remove_empty_parents(dir, &path)
    end
  end
```

## `codemaid_corpus::contract::remove_empty_parents`
`fn remove_empty_parents(root: &Path, file: &Path)` · L162-L170
```mermaid
sequenceDiagram
  participant codemaid_corpus__contract as contract mod
  participant codemaid_model__symbol__SymbolId as SymbolId
  loop while let Some(dir) = cur
    codemaid_corpus__contract->>codemaid_model__symbol__SymbolId: ~parent()
  end
```

## `codemaid_corpus::contract::corpus_hash`
`pub fn corpus_hash(corpus: &Corpus) -> ContentHash` · L172-L183
> Hash of a whole corpus (all paths and contents), for cheap equality checks across machines.
```mermaid
sequenceDiagram
  participant codemaid_corpus__contract as contract mod
  participant codemaid_model__hash__ContentHash as ContentHash
  codemaid_corpus__contract->>codemaid_model__hash__ContentHash: ContentHash::of_bytes(as_bytes())
```
