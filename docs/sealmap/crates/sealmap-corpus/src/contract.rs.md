---
sealmap: 2
source: crates/sealmap-corpus/src/contract.rs
module: "sym:cargo sealmap_corpus . contract/"
language: rust
source_hash: blake3:5dbda05b5570872402ab655b7be92d887beda9ce813d056e708d4ef30c3ffd26
lines: 183
fragments: 7
---
# `sym:cargo sealmap_corpus . contract/` · crates/sealmap-corpus/src/contract.rs
> The 1:1 contract: verify a corpus directory against freshly generated output, and write a directory into compliance.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__contract___tDrift["Drift"] {
    <<enum>>
    Missing
    Orphaned
    Stale
    Modified
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
  }
  class sealmap_corpus__contract___tDriftEntry["DriftEntry"] {
    <<struct>>
    +path: SourcePath
    +drift: Drift
  }
  class sealmap_corpus__contract___tReport["Report"] {
    <<struct>>
    +entries: Vec#lt;DriftEntry#gt;
    +checked: usize
    +count(&self, drift: Drift) usize
    +is_clean(&self) bool
  }
  class sealmap_corpus__contract["sealmap_corpus::contract"] {
    <<module>>
    +corpus_hash(corpus: &Corpus) ContentHash
    +read_dir_corpus(dir: &Path) io::Result#lt;BTreeMap#lt;SourcePath, String#gt;#gt;
    -read_rec(root: &Path, dir: &Path, out: &mut BTreeMap#lt;SourcePath, String#gt;) io::Result#lt;#40;#41;#gt;
    -remove_empty_parents(root: &Path, file: &Path)
    +verify(dir: &Path, expected: &Corpus) io::Result#lt;Report#gt;
    +verify_against(expected: &Corpus, actual: &BTreeMap#lt;SourcePath, String#gt;) Report
    +write(dir: &Path, expected: &Corpus) io::Result#lt;Report#gt;
  }
  class sealmap_corpus___tCorpus["Corpus"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_model__hash___tContentHash["ContentHash"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  sealmap_corpus__contract ..> sealmap_corpus___tCorpus
  sealmap_corpus__contract ..> sealmap_corpus__contract___tReport
  sealmap_corpus__contract ..> sealmap_model__hash___tContentHash
  sealmap_corpus__contract ..> sealmap_model__path___tSourcePath
  sealmap_corpus__contract___tDriftEntry *-- sealmap_corpus__contract___tDrift : drift
  sealmap_corpus__contract___tDriftEntry *-- sealmap_model__path___tSourcePath : path
  sealmap_corpus__contract___tReport ..> sealmap_corpus__contract___tDrift
  sealmap_corpus__contract___tReport o-- sealmap_corpus__contract___tDriftEntry : entries
```

## `sym:cargo sealmap_corpus . contract/verify_against().`
`pub fn verify_against(expected: &Corpus, actual: &BTreeMap<SourcePath, String>) -> Report` · L72-L105
> Compare `expected` (freshly generated) with `actual` (file path → text as found on disk or elsewhere).
```mermaid
sequenceDiagram
  participant sealmap_corpus__contract as contract mod
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_corpus__document as document mod
  loop for (path, want) in &expected.files
    opt Some(have)
      sealmap_corpus__contract->>sealmap_corpus: is_reserved(path)
      opt not is_reserved(path)
        sealmap_corpus__contract->>sealmap_corpus__document: front_matter_hash(want)
        sealmap_corpus__contract->>sealmap_corpus__document: front_matter_hash(have)
      end
    end
  end
  loop for (path, text) in actual
    sealmap_corpus__contract->>sealmap_corpus__document: is_generated(text)
  end
```

## `sym:cargo sealmap_corpus . contract/read_dir_corpus().`
`pub fn read_dir_corpus(dir: &Path) -> io::Result<BTreeMap<SourcePath, String>>` · L107-L115
> Read every file under `dir` (UTF-8 only) keyed by relative path.
```mermaid
sequenceDiagram
  participant sealmap_corpus__contract as contract mod
  opt dir.exists()
    sealmap_corpus__contract->>sealmap_corpus__contract: read_rec(dir, dir, &out)?
  end
```

## `sym:cargo sealmap_corpus . contract/read_rec().`
`fn read_rec(root: &Path, dir: &Path, out: &mut BTreeMap<SourcePath, String>) -> io::Result<()>` · L117-L131
```mermaid
sequenceDiagram
  participant sealmap_corpus__contract as contract mod
  participant sealmap_model__path___tSourcePath as SourcePath
  loop for e in entries
    alt ty.is_dir()
      sealmap_corpus__contract->>sealmap_corpus__contract: read_rec(root, &path(), out)?
    else if ty.is_file()
      sealmap_corpus__contract->>sealmap_model__path___tSourcePath: SourcePath::relative_to(&path(), root)
    end
  end
```

## `sym:cargo sealmap_corpus . contract/verify().`
`pub fn verify(dir: &Path, expected: &Corpus) -> io::Result<Report>` · L133-L136
> Verify the corpus in `dir` against `expected`.
```mermaid
sequenceDiagram
  participant sealmap_corpus__contract as contract mod
  sealmap_corpus__contract->>sealmap_corpus__contract: read_dir_corpus(dir)?
  sealmap_corpus__contract->>sealmap_corpus__contract: verify_against(expected, &_)
```

## `sym:cargo sealmap_corpus . contract/write().`
`pub fn write(dir: &Path, expected: &Corpus) -> io::Result<Report>` · L138-L160
> Bring `dir` into compliance with `expected`: write missing, stale and modified files, delete orphaned generated documents (and nothing else).
```mermaid
sequenceDiagram
  participant sealmap_corpus__contract as contract mod
  sealmap_corpus__contract->>sealmap_corpus__contract: verify(dir, expected)?
  loop for e in &report.entries
    opt Drift::Orphaned
      sealmap_corpus__contract->>sealmap_corpus__contract: remove_empty_parents(dir, &path)
    end
  end
```

## `sym:cargo sealmap_corpus . contract/corpus_hash().`
`pub fn corpus_hash(corpus: &Corpus) -> ContentHash` · L172-L183
> Hash of a whole corpus (all paths and contents), for cheap equality checks across machines.
```mermaid
sequenceDiagram
  participant sealmap_corpus__contract as contract mod
  participant sealmap_model__hash___tContentHash as ContentHash
  sealmap_corpus__contract->>sealmap_model__hash___tContentHash: ContentHash::of_bytes(as_bytes())
```
