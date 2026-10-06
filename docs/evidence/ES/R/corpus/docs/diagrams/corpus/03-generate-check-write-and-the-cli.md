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
    participant CL as classifier
    participant OUT as class
    E->>CL: present in the directory (src/contract.rs:81)
    alt no
        CL->>OUT: Missing (src/contract.rs:82)
    else yes
        CL->>CL: byte-identical (src/contract.rs:83)
        alt yes
            CL->>OUT: clean
        else no, reserved underscore file (src/contract.rs:85)
            CL->>OUT: Modified (src/contract.rs:86)
        else same source_hash in the front matter (src/contract.rs:89)
            CL->>OUT: Modified, hand edit or new<br/>generator or options (src/contract.rs:90)
        else source changed
            CL->>OUT: Stale (src/contract.rs:91)
        end
    end
    Note over CL,OUT: each md on disk not expected and<br/>carrying the header, Orphaned (src/contract.rs:99-100)
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
    participant W as write
    participant F as a corpus file
    opt source edited
        F->>F: Clean becomes Stale
    end
    opt document hand-edited
        F->>F: Clean becomes Modified
    end
    opt generator version or options change
        F->>F: Clean becomes Modified
    end
    opt source deleted
        F->>F: Clean becomes Orphaned
    end
    opt new source
        F->>F: starts Missing
    end
    alt Stale
        W->>F: write rewrites it, Clean
    else Modified
        W->>F: write rewrites it, Clean
    else Missing
        W->>F: write creates it, Clean
    else Orphaned
        W->>F: write deletes it and empty parents
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
sequenceDiagram
    autonumber
    participant D as corpus directory
    participant RD as the reader
    participant M as path to text<br/>src/contract.rs:126
    D->>RD: read
    alt does not exist (src/contract.rs:111)
        RD->>M: empty map, every file Missing
    else exists
        RD->>RD: recursive read, entries sorted by name<br/>(src/contract.rs:119)
        alt readable as UTF-8 and<br/>relative to the root (src/contract.rs:125)
            RD->>M: entry
        else not readable in place
            RD->>RD: skipped
        end
    end
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
    autonumber
    participant CMD as commands, generate, model,<br/>dense, resolve, seal-check, verify, stale,<br/>seal sign, pack (main.rs:45-71)
    participant SRC as shared source flags<br/>--repo, --name, --tests (main.rs:76-86)
    participant GEN as generate only, -o, --check,<br/>--external, --owner-lanes, --min-calls,<br/>--public-only, --no-model, --pretty<br/>main.rs:89-119
    participant MOD as model, path, source flags,<br/>--external (main.rs:122-131)
    participant DNS as dense, path, source flags, -o,<br/>--depth, --external, --stats (main.rs:134-152)
    participant SITE as seal commands and pack<br/>-C ROOT, --diagrams DIR, source flags<br/>main.rs:156-165
    participant PCK as pack only, topics, --diff,<br/>--budget, --depth, --source-window, -o,<br/>--shard (main.rs:206-232, COR-06)
    participant OP as Options, rust and corpus halves<br/>sealmap/src/lib.rs:98
    participant LD as loading, one directory<br/>load_dir (main.rs:394)
    CMD->>SRC: shared
    CMD->>GEN: generate only
    CMD->>MOD: model
    CMD->>DNS: dense
    CMD->>SITE: site group
    SITE->>PCK: pack only
    GEN->>OP: parsed into
    alt one directory
        SRC->>LD: load_dir
    else several repos, each prefixed NAME/
        SRC->>LD: load_repos, sealmap/src/lib.rs:125
    end
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
