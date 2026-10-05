---
id: COR-05
title: The seal commands, resolve, verify, stale and sign
area: corpus
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - crates/sealmap/src/main.rs
  - crates/sealmap/tests/cli.rs
  - crates/sealmap-corpus/src/seal/check.rs
  - docs/DESIGN.md
  - README.md
verified_commit: ae478d90d2b910101a1aa065cb4655e7defea332
---
## For developers

The seal commands are thin: each one loads the same context (a model of the
source root, the lock text and the topic files), calls one function of
`sealmap_corpus::seal` (COR-04), and prints text or JSON. The only process the
CLI spawns is for `stale --since`, which exports a git revision into a
temporary directory so the library can compare two models; git stays out of
the libraries (`docs/DESIGN.md:166`-`169`).

After this topic you should know what each command reads and writes, how a
revision becomes a second model, what the exit codes mean, and which options
a seal silently depends on.

## For the business

Five commands cover the whole seal workflow for a pipeline or a skill: ask
where a function is now, seal a reviewed topic, gate CI on every seal,
classify a chosen few, and list what changed since a release or a branch
point. The last one always succeeds, so it can feed a report without failing
a build; the gate fails on anything short of "still true".

One caution for adopters: a seal remembers functions by id, and the ids
depend on how the code was read (whether tests were included, what the
codebase is called when no manifest names it). Checking a seal with different
settings from the ones it was signed with makes sealed functions look absent.

## COR-05.1 What every seal command loads

```mermaid
sequenceDiagram
    autonumber
    participant CMD as resolve, seal-check, verify,<br/>stale or seal sign
    participant CX as Ctx load<br/>main.rs:395
    participant EX as extract<br/>main.rs:339
    participant FS as the file system
    participant RT as read_topics<br/>main.rs:411
    CMD->>CX: -C ROOT, --diagrams DIR, source flags
    CX->>EX: model of ROOT and the name used (main.rs:396)
    CX->>CX: DIR relative to ROOT unless absolute (main.rs:397)
    CX->>FS: DIR/seals.lock, a missing file is no lock (main.rs:398-402)
    CX->>RT: every md under DIR with a front-matter id (main.rs:404)
    RT->>FS: sorted entries, hidden names skipped (main.rs:421-426)
    RT-->>CX: topics keyed by path under DIR (main.rs:433-435)
    CX-->>CMD: model, name, lock text, topics (main.rs:405)
```

**What it shows.** One loader feeds every seal command, so they all see the
same model, the same lock bytes and the same topic set.

**Why it is this way.** The lock is kept as text, not parsed here, because
`verify` must judge its bytes for canonicality while `seal sign` only needs to
read it (`crates/sealmap/src/main.rs:390`,
`crates/sealmap/src/main.rs:612`-`613`). A topic is any Markdown file whose
front matter has an `id:`, so the corpus's README and indexes are ignored
(`crates/sealmap/src/main.rs:433`).

**Debt:** a seal does not record the extraction options that shape ids, so a
lock signed with `--tests` and checked without it reports its test symbols as
absent; only the flag's doc comment says so
(`crates/sealmap/src/main.rs:68`-`69`).

## COR-05.2 stale --since, two trees

```mermaid
sequenceDiagram
    autonumber
    participant US as developer or skill
    participant ST as stale<br/>main.rs:513
    participant GT as GitTree export<br/>main.rs:557
    participant GI as git and tar
    participant EX as extract<br/>main.rs:339
    participant LB as seal stale<br/>check.rs:481
    US->>ST: stale --since REV
    ST->>ST: refuse with --repo (main.rs:522-523)
    ST->>GT: ROOT and REV (main.rs:525)
    GT->>GI: rev-parse --show-prefix (main.rs:558)
    GT->>GI: archive REV:prefix piped into tar -x, temp dir (main.rs:563-571)
    GI-->>GT: the revision's tree, or an error (main.rs:579-583)
    ST->>EX: the old tree under the current tree's name (main.rs:526-527)
    ST->>LB: lock, old model, current model (main.rs:530)
    LB-->>ST: changed sealed symbols
    ST-->>US: topic, class, symbol, candidates, always exit 0 (main.rs:535, main.rs:548)
    Note over GT: the temp dir is removed on drop, main.rs:593-595
```

**What it shows.** A revision becomes a second model by exporting only the
subdirectory the root points at, so paths line up, and reading it under the
current codebase name, so ids line up.

**Why it is this way.** Each sealed symbol's baseline is its hashes in the old
model when it is there, else its sealed hashes, so `--since` reports what
changed since that revision whatever the lock says
(`crates/sealmap-corpus/src/seal/check.rs:486`-`489`). The CLI test commits a
behaviour change, edits a signature after it, and sees only the contract
change with `--since HEAD` (`crates/sealmap/tests/cli.rs:116`, `crates/sealmap/tests/cli.rs:141`).

**Debt:** `--since` needs `tar` on the path as well as `git`
(`crates/sealmap/src/main.rs:571`), and a process killed mid-run leaves its
`sealmap-since-*` directory in the system temp directory, since only a normal
drop removes it (`crates/sealmap/src/main.rs:593`-`595`).

## COR-05.3 seal sign, from the command line

```mermaid
flowchart TB
    A["seal sign TOPIC --reviewer --model, maybe --date<br/>main.rs:609"]
    L["lock parsed leniently, a non-canonical<br/>lock is rewritten canonically<br/>main.rs:612-613"]
    T{"TOPIC an id? else a path under the<br/>cwd or the diagrams dir<br/>main.rs:648-662"}
    D["date: --date, else today in UTC<br/>main.rs:621"]
    S["seal sign in the library<br/>main.rs:623"]
    R["refused: message, exit 1, nothing written<br/>main.rs:625-627"]
    W1["topic written only if the pointer changed it<br/>main.rs:631-632"]
    W2["lock written canonically<br/>main.rs:634"]
    A --> L --> T --> D --> S
    S -->|error| R
    S -->|ok| W1 --> W2
```

**What it shows.** Signing reads, derives and refuses before it writes; a
successful sign writes at most two files, the topic first and the lock last.

**Why it is this way.** The library does the derivation and the CLI only the
IO, so a skill can call either (`docs/DESIGN.md:80`-`85`). The default date is
computed from the system clock without a date library
(`crates/sealmap/src/main.rs:666`-`673`).

**Invariant:** a failure between the two writes leaves a pointer with no
entry, which `verify` reports as a lock fault, so an interrupted sign can
never pass the gate (`crates/sealmap/src/main.rs:632`-`634`,
`crates/sealmap-corpus/src/seal/check.rs:383`).

## COR-05.4 Exit codes

```mermaid
flowchart TB
    E2["2: usage or IO error, any command<br/>main.rs:215"]
    V["verify, seal-check: 0 when every finding<br/>holds, else 1<br/>main.rs:508"]
    R["resolve: 0 when found, 1 when absent<br/>or unparsable<br/>main.rs:477"]
    S["stale: always 0<br/>main.rs:548"]
    G["seal sign: 0 sealed, 1 refused<br/>main.rs:627"]
    C["generate --check: 0 clean, 1 drift<br/>main.rs:279"]
```

**What it shows.** Exit 1 always means "the check ran and said no", and 2
always means the command could not run.

**Why it is this way.** `stale` is a report, not a gate, so a skill can call
it on every commit without failing the build; the design gives it exit 0
(`docs/DESIGN.md:157`). `resolve` fails when the id is not found so a script
that checks diagram edges can branch on it (`README.md:287`-`288`); the codes
are pinned end to end (`crates/sealmap/tests/cli.rs:55`,
`crates/sealmap/tests/cli.rs:96`).
