---
id: COR-03
title: Generate --check, write and the command line
area: corpus
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - crates/sealmap-corpus/src/contract.rs
  - crates/sealmap-corpus/src/document.rs
  - crates/sealmap-corpus/src/lib.rs
  - crates/sealmap-corpus/tests/contract.rs
  - crates/sealmap/src/main.rs
  - crates/sealmap/src/lib.rs
  - .github/workflows/ci.yml
  - docs/DESIGN.md
  - README.md
verified_commit: 4ed7a51f92f7241a1c420e49e8036a8d9189912a
---
## For developers

`verify` and `write` in `contract.rs` are the 0.1 contract between a generated
corpus and a directory on disk. `verify_against` compares a freshly generated
`Corpus` with what a directory holds and classifies every difference as
missing, orphaned, stale or modified; `write` brings the directory into
compliance and only ever deletes files that carry the generator's header
(`crates/sealmap-corpus/src/contract.rs:1`-`2`,
`crates/sealmap-corpus/src/lib.rs:12`-`29`).

Since step 3 the command line reaches that contract only through
`sealmap generate`: plain `generate` writes, and `generate --check` compares
and writes nothing (`crates/sealmap/src/main.rs:311`-`322`). The name `verify`
now belongs to the seal gate (COR-04, COR-05), and nothing generated is
committed; CI checks instead that two fresh generations agree (DEL-01). This
topic draws the generate path, the drift classes and the shared flags.

## For the business

The generated corpus is a view, rebuilt in seconds whenever someone wants it,
and never a file set a pipeline has to keep in step. A developer who keeps a
copy on disk can ask whether it is still current with one flag, and gets an
exact answer: which files are missing, which are left over, which are out of
date because the code changed, and which were edited by hand or by a
different generator version.

For an adopter migrating from 0.1, the one change to plan for is the name:
a pipeline that ran `sealmap verify` against a committed corpus now wants
`sealmap generate --check`, and `sealmap verify` means the seal gate.

## COR-03.1 sealmap generate, end to end

```mermaid
sequenceDiagram
    autonumber
    participant US as developer or CI
    participant GE as generate<br/>main.rs:300
    participant EX as extract<br/>main.rs:381
    participant AD as sealmap-rust extract
    participant PR as sealmap generate
    participant CT as contract verify or write
    US->>GE: generate PATH -o DIR, maybe --check
    GE->>GE: map flags onto the corpus options (main.rs:301)
    GE->>EX: sources, single dir or several repos (main.rs:309)
    EX->>EX: codebase name from --name, the dir or the repo names (main.rs:393)
    EX->>AD: extract once (main.rs:402)
    AD-->>EX: Extraction, diagnostics printed as warnings (main.rs:403)
    EX-->>GE: model and the name used (main.rs:382)
    GE->>PR: corpus in memory (main.rs:310)
    alt --check
        GE->>CT: compare DIR with the fresh corpus (main.rs:312)
        CT-->>GE: Report
        GE-->>US: one padded line per drift, exit 1 if any (main.rs:314, main.rs:321)
    else write
        GE->>CT: bring DIR into compliance (main.rs:323)
        GE-->>US: counts written, removed, unchanged (main.rs:326)
    end
```

**What it shows.** Both branches regenerate the whole corpus in memory first;
`--check` then compares and `write` repairs, and nothing is trusted from disk
except the bytes being compared.

**Why it is this way.** The extraction helper returns the codebase name it
used, so `stale --since` can read a second tree under the same name
(`crates/sealmap/src/main.rs:378`-`381`). Errors in arguments or IO exit 2,
drift under `--check` exits 1 (`crates/sealmap/src/main.rs:256`,
`crates/sealmap/src/main.rs:321`). A drift name is padded through
`to_string`, because `Drift`'s `Display` writes with `write_str` and ignores a
width (`crates/sealmap/src/main.rs:314`,
`crates/sealmap-corpus/src/contract.rs:32`).

## COR-03.2 Classifying a difference

```mermaid
sequenceDiagram
    autonumber
    participant E as each expected file<br/>src/contract.rs:80
    participant CT as the directory comparison
    E->>CT: is it present in the directory (src/contract.rs:81)
    alt not present
        CT-->>E: Missing (src/contract.rs:82)
    else present
        alt byte-identical (src/contract.rs:83)
            CT-->>E: clean
        else differs
            alt reserved underscore file (src/contract.rs:85)
                CT-->>E: Modified (src/contract.rs:86)
            else a document
                alt same source_hash in the front matter (src/contract.rs:89)
                    CT-->>E: Modified, hand edit or new<br/>generator or options (src/contract.rs:90)
                else the source hash differs
                    CT-->>E: Stale, the source changed (src/contract.rs:91)
                end
            end
        end
    end
    Note over E,CT: each md on disk not expected and carrying<br/>the header is Orphaned (src/contract.rs:99-100)
```

**What it shows.** The front-matter source hash is what separates "the code
changed" from "the document was touched"; anything in the directory that is
not a generated document is ignored.

**Why it is this way.** Recording the source hash in each document makes drift
detectable without re-parsing the old state (`crates/sealmap-corpus/src/lib.rs:16`-`17`);
`verify_classifies_every_kind_of_drift` pins all four classes
(`crates/sealmap-corpus/tests/contract.rs:106`).

**Debt:** a reserved file is always `Modified` when it differs
(`crates/sealmap-corpus/src/contract.rs:85`-`86`), even when the difference is
that the code changed, so `_index.json` and `_model.json` never report stale.

## COR-03.3 A corpus file's drift states

```mermaid
sequenceDiagram
    autonumber
    participant NW as generate then write,<br/>or a new source
    participant ST as the file's drift state
    participant WR as write
    alt generate then write
        NW->>ST: Clean
        alt source edited
            ST->>ST: Stale
            WR->>ST: write rewrites it, back to Clean
        else document hand-edited
            ST->>ST: Modified
            WR->>ST: write rewrites it, back to Clean
        else generator version or options change
            ST->>ST: Modified
            WR->>ST: write rewrites it, back to Clean
        else source deleted
            ST->>ST: Orphaned
            WR->>ST: write deletes it and empty parents
        end
    else a new source
        NW->>ST: Missing
        WR->>ST: write creates it, back to Clean
    end
```

**What it shows.** Every drift is repaired by `write` in one step: stale,
modified and missing files are written, orphaned generated documents are
deleted, and unchanged files are not touched.

**Why it is this way.** Untouched files keep their mtimes, so a re-run is cheap
for anything watching the directory (`crates/sealmap-corpus/src/contract.rs:138`-`141`);
`write_converges_and_is_idempotent` pins that a second `write` changes
nothing (`crates/sealmap-corpus/tests/contract.rs:138`).

**Invariant:** `write` deletes only Markdown files whose YAML front matter
opens on the first line (`---`) and whose first key is the `sealmap: `
marker; a file that merely mentions the marker anywhere else is never touched
(`crates/sealmap-corpus/src/contract.rs:99`, `crates/sealmap-corpus/src/document.rs:153`-`155`).

## COR-03.4 Reading a directory

```mermaid
flowchart TB
    D["corpus directory"]
    EX{"exists?<br/>src/contract.rs:111"}
    EMP["empty map: every file Missing"]
    RR["recursive read, entries sorted by name<br/>src/contract.rs:119"]
    UT{"readable as UTF-8 and<br/>relative to the root?<br/>src/contract.rs:125"}
    IN["path to text<br/>src/contract.rs:126"]
    SK["skipped"]
    D --> EX
    EX -->|no| EMP
    EX -->|yes| RR --> UT
    UT -->|yes| IN
    UT -->|no| SK
```

**What it shows.** The directory is read into the same path-to-text map the
generator produces, in sorted order, so the comparison is a map diff.

**Why it is this way.** Sorting entries keeps the report order independent of
the file system (`crates/sealmap-corpus/src/contract.rs:119`); the whole corpus
can also be summarised as one hash for cheap cross-machine equality
(`crates/sealmap-corpus/src/contract.rs:174`-`183`).

## COR-03.5 Commands, flags and the facade

```mermaid
sequenceDiagram
    participant CMD as the commands<br/>generate, model, dense, resolve, seal-check,<br/>verify, stale, seal sign, pack<br/>main.rs:45-71
    participant SRC as shared source flags<br/>--repo, --name, --tests<br/>main.rs:76-86
    participant GEN as generate only<br/>-o, --check, --external, --owner-lanes,<br/>--min-calls, --public-only, --no-model, --pretty<br/>main.rs:89-119
    participant MOD as model<br/>path, source flags, --external<br/>main.rs:122-131
    participant DNS as dense<br/>path, source flags, -o, --depth,<br/>--external, --stats<br/>main.rs:134-152
    participant SITE as seal commands and pack<br/>-C ROOT, --diagrams DIR, source flags<br/>main.rs:156-165
    participant PCK as pack only<br/>topics, --diff, --budget, --depth,<br/>--source-window, -o, --shard<br/>main.rs:206-232, COR-06
    CMD->>SRC: every command shares them (main.rs:76-86)
    CMD->>GEN: generate alone carries them (main.rs:89-119)
    CMD->>MOD: model (main.rs:122-131)
    CMD->>DNS: dense (main.rs:134-152)
    CMD->>SITE: seal commands and pack (main.rs:156-165)
    SITE->>PCK: pack adds (main.rs:206-232)
    Note over GEN: Options, rust and corpus halves<br/>sealmap/src/lib.rs:98
    Note over SRC: one directory, load_dir, main.rs:394<br/>or several repos, each prefixed NAME,<br/>load_repos, sealmap/src/lib.rs:125
```

**What it shows.** Every command shares the flags that shape ids; only
`generate` carries the projection flags, and `model` takes the external-call
policy because it changes the flows it prints. The seal commands and `pack`
share one `Site` group (root, topic directory, source flags); the report
flags `--json` and `--all` sit beside it only where a command reports. Several repositories are
loaded into one `SourceSet` under name prefixes, so calls between them
resolve and the overview groups crates by repository
(`crates/sealmap/src/lib.rs:112`-`113`).

**Why it is this way.** The source flags decide which symbols exist and what
they are called, so a seal must be checked with the flags it was signed with
(`crates/sealmap/src/main.rs:73`-`74`); the facade exists so the common path
is one dependency (`crates/sealmap/src/lib.rs:35`-`36`), and the CLI is behind
the default `cli` feature, so a library user does not pull in clap.
