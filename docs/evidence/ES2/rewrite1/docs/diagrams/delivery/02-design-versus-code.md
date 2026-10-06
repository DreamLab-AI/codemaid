---
id: DEL-02
title: Design versus code after the seal surface
area: delivery
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - docs/DESIGN.md
  - README.md
  - .gitignore
  - Cargo.toml
  - .github/workflows/ci.yml
  - crates/sealmap/Cargo.toml
  - crates/sealmap/src/main.rs
  - crates/sealmap-corpus/src/seal/lock.rs
  - crates/sealmap-corpus/src/seal/check.rs
  - crates/sealmap-corpus/src/pack.rs
verified_commit: 6cefaf431511f380d989b93790241b903077d673
---
## For developers

`docs/DESIGN.md` was accepted on 2026-10-05 with every recommendation taken
(`docs/DESIGN.md:3`). Steps 1 and 2 of its order of work landed the same day,
and step 3 followed in the tree: the lock, `resolve`, `stale`, `seal-check`,
`verify`, `seal sign`, the retirement of the committed generated corpus,
`sealmap-dense` with its `sealmap dense` subcommand, and `pack`
(`docs/DESIGN.md:298`-`303`, `README.md:364`-`367`). Step 3 closed with
the 0.2.0 publish (`docs/DESIGN.md:302`-`303`). This topic is the catalogue of the gap once `pack`
landed: what the design still describes that the code does not do, and the
questions the design leaves open.

It is a catalogue, not a plan. The design's order of work
(`docs/DESIGN.md:289`-`315`) is the plan. Where the build settled something
the design left loose, DESIGN now records it as built (`docs/DESIGN.md:72`-`74`,
`docs/DESIGN.md:80`-`94`, `docs/DESIGN.md:113`-`121`, `docs/DESIGN.md:171`-`191`), and the subsystem topics carry the detail: the
seal module in COR-04, the seal commands in COR-05, `generate --check` in
COR-03, `pack` in COR-06.

## For the business

An adopter can now use what the project is *for*: hand-written diagram topics
cite functions by stable id, a lockfile records the exact versions a reviewer
checked them against, and one command in CI says whether every seal still
holds, with no model involved. After a commit, a second command lists only
the topics whose sealed functions changed.

The dense agent projection exists: an agent can read the shape of the code
at about a quarter of the source's size. So do review packs: one bounded,
reproducible file holding the topics to review and the code they cite. The
designed variant that holds diagrams alone, for an outside reviewer who
should not see code, is still to come. The headline
cost saving is not measured yet; that is step 4. And this repository has not
sealed its own diagrams yet, so its CI does not run the seal gate (step 6).

## DEL-02.1 The design's layers against the tree

```mermaid
sequenceDiagram
    autonumber
    participant A as Authored layer<br/>docs/DESIGN.md:39
    participant S as Sealed layer<br/>docs/DESIGN.md:40
    participant G as Generated layer<br/>docs/DESIGN.md:41
    participant P as Review pack<br/>docs/DESIGN.md:42
    A->>A: topics citing sym ids
    Note over A: built, citations are found and checked<br/>by sealmap verify (check.rs:359)
    A->>S: next layer
    S->>S: seals.lock plus a sealed pointer per topic
    Note over S: built, canonical lock, pointer,<br/>topic_hash (lock.rs:57)
    S->>G: next layer
    G->>G: model, index, dense projection,<br/>optional 1:1 Mermaid, gitignored
    Note over G: model, index, 1:1 Mermaid and the dense<br/>projection under a gitignored .sealmap<br/>(.gitignore:3, main.rs:51)
    G->>P: next layer
    P->>P: pegged topics, dense slices,<br/>source windows
    Note over P: built, topics, dense slices, source windows<br/>(pack.rs:274), diagrams-only --review NOT BUILT
```

**What it shows.** All four layers exist, the generated one no longer
committed; of the review pack, only the diagrams-only mode is missing.

**Why it is this way.** Step 3 split in two: the seal surface first, then
`pack` and `sealmap-dense` (`docs/DESIGN.md:277`-`279`); `sealmap-dense` was
built on a branch and merged after the seal surface. The generated corpus
is rebuilt on demand and never trusted from disk (`docs/DESIGN.md:44`-`48`).

`sealmap-dense` came in under its estimate. The README now reports the
measured 0.17–0.29× source (`README.md:60`) where it quoted the research's
0.45×, and DESIGN records why the built projection is smaller: each callable
is expanded once and each signature printed once (`docs/DESIGN.md:139`-`144`).

**Debt (designed, not built):** `--review`, a pack of diagrams only, is in
the layer table (`docs/DESIGN.md:42`) but not in the binary
(`docs/DESIGN.md:181`, `crates/sealmap/src/main.rs:45`-`65`).

## DEL-02.2 The order of work and where the tree stands

```mermaid
sequenceDiagram
    autonumber
    participant W as the order of work
    W->>W: 1 repository, rename, dual licence,<br/>hardening, sealmap-extract,<br/>DONE (docs/DESIGN.md:291)
    W->>W: 2 ids and hashes, sym grammar, sig and body,<br/>injective ids, schema v2, DONE (docs/DESIGN.md:296)
    W->>W: 3 seal surface, retirement, sealmap-dense<br/>and pack DONE, published as 0.2.0<br/>(docs/DESIGN.md:298)
    W->>W: 4 E0-R on VisionClaw and agentbox<br/>(docs/DESIGN.md:304)
    W->>W: 5 sealmap skill, routing ADR (docs/DESIGN.md:305)
    W->>W: 6 dogfood, seal this corpus (docs/DESIGN.md:308)
    W->>W: 7 sealmap-ts, deferred (docs/DESIGN.md:310)
    W->>W: 8 review A/B (docs/DESIGN.md:312)
    W->>W: 9 migrate VisionFlow (docs/DESIGN.md:314)
```

**What it shows.** Three steps done in the tree, the third short of its
publish. The first
number the design is built to produce, topics flagged per commit file-level
against sealed, comes at step 4; `stale --since` is the tool it needs, and it
exists.

**Why it is this way.** The evidence endpoints are a fixed sequence, each
tested only if the previous one holds (`docs/DESIGN.md:262`-`263`).

**Open:** the endpoints are to be pre-registered in `PREREG.md` before any run
(`docs/DESIGN.md:263`), and the design cites its evidence as `research/01`
to `05` in a design-session scratchpad (`docs/DESIGN.md:4`-`5`); neither is in
the repository, so where do the pre-registration and the evidence live?

## DEL-02.3 Planned commands against present ones

```mermaid
sequenceDiagram
    autonumber
    participant P as the designed CLI<br/>docs/DESIGN.md:154-164
    participant N as the present CLI<br/>sealmap/src/main.rs:45-65
    Note over P: resolve, stale since rev, seal-check, verify,<br/>seal sign, pack, generate with --check, dense,<br/>export --scip, later
    P->>N: resolve, stale since rev, seal-check,<br/>verify, seal sign, pack
    P->>N: generate with --check, dense
    P-->>N: export --scip, not built
    Note over N: resolve, seal-check, verify, stale, seal sign, pack<br/>generate with --check, model, dense
```

**What it shows.** Every designed command except the later `export --scip`
exists, and `verify` now means the seal gate only; the old drift check is
`generate --check`.

**Why it is this way.** Git stays out of the libraries: `--since` and
`--diff` export the revision with `git archive` and read it as a second model
(`docs/DESIGN.md:166`-`169`, `crates/sealmap/src/main.rs:608`-`617`). `pack`
was built to its own flags rather than the planned ones: `--diff REV`
against the working tree instead of two named trees, and `--budget` instead
of `--max-bytes`. DESIGN records both (`docs/DESIGN.md:161`,
`docs/DESIGN.md:171`-`180`).

## DEL-02.4 A sealed topic's life, as built

```mermaid
sequenceDiagram
    autonumber
    participant S as seal sign
    participant SE as Sealed
    participant H as Holds
    participant B as Behaviour
    participant C as Contract
    participant A as Absent
    participant U as Unparsable
    participant P as ProseEdited
    participant L as LockFault
    S->>SE: seal sign writes the entry and the pointer
    alt ids resolve, sig and body match
        SE->>H: Holds
    else body changed
        SE->>B: Behaviour
        B->>SE: cheap cross-family review, sign
    else signature changed
        SE->>C: Contract
        C->>SE: ADR addendum, re-consolidate, review, sign
    else id gone, candidates listed
        SE->>A: Absent
        A->>SE: confirm rename, edit the citation, sign
    else a file that may hold it fails to parse
        SE->>U: Unparsable
    else topic_hash differs
        SE->>P: ProseEdited
        P->>SE: review the edit, sign
    else pointer and lock disagree, or lock not canonical
        SE->>L: LockFault
    end
    Note over H: the end state
```

**What it shows.** The lifecycle the design gives a sealed topic
(`docs/DESIGN.md:99`-`111`), now with every state reachable from the code;
the review between a failing state and the next seal is the skill's step, not
the crate's.

**Why it is this way.** Code, topic and lock land in one commit, ending the
two-commit "change then re-stamp" routine (`README.md:105`-`106`); `seal sign`
re-derives the entry from the code, so the lock is never hand-edited
(`docs/DESIGN.md:80`-`85`).

**Open:** reviewer family, evidence and signatures are left to a skill script,
`seal-gate.mjs`, next to `verify` (`docs/DESIGN.md:123`-`124`); `seal sign`
records whatever reviewer string it is given, so until that script exists
nothing enforces the cross-family rule.

## DEL-02.5 Crates planned against crates present

```mermaid
sequenceDiagram
    autonumber
    participant H as in the tree<br/>Cargo.toml:3
    participant P as planned<br/>docs/DESIGN.md:128-137
    Note over H: sealmap-model, sealmap-extract, sealmap-rust,<br/>sealmap-mermaid
    Note over H: sealmap-corpus, generate, write,<br/>the seal module and pack
    Note over H: sealmap-dense, skeletons, call trees,<br/>index, budgeted slices (sealmap/Cargo.toml:29)
    Note over H: sealmap facade, features cli, parallel<br/>(sealmap/Cargo.toml:17)
    Note over P: sealmap-ts on oxc, DEFERRED<br/>facade, sealmap-ts behind a default feature,<br/>oxc needs MSRV 1.97
    H->>P: what is still to come
```

**What it shows.** Seven crates exist, `sealmap-dense` the newest, and the
corpus crate has gained `pack`; one crate is deferred.

**Why it is this way.** The owner deferred `sealmap-ts` on 2026-10-05: it is
built only if E0-R on the Rust repositories shows the precise-staleness gain
is real (`docs/DESIGN.md:310`-`311`, `README.md:258`).

Closed in the 0.2.0 release: the workspace is at `0.2.0` (`Cargo.toml:6`),
so a lock signed by this tree names `sealmap` 0.2.0, the first release that
has the seal module (`crates/sealmap-corpus/src/seal/lock.rs:21`).

**Open:** `sealmap-ts` is deferred (`docs/DESIGN.md:133`), yet the crate
surface still plans it behind a *default* facade feature because oxc needs
MSRV 1.97 (`docs/DESIGN.md:137`); if it is ever built, does the facade's 1.85
MSRV survive a default feature that needs 1.97?

## DEL-02.6 Dogfooding the gate on this repository

```mermaid
sequenceDiagram
    autonumber
    participant T as 16 topics<br/>path:line citations
    participant V as sealmap verify<br/>check.rs:359
    T->>V: verify
    alt topics with no seal and no citation (check.rs:391-392)
        V-->>T: coverage only, never a failure
    else MER-02 cites a fixture id in prose (check.rs:390)
        V-->>T: an unsealed citation, exit 1
    end
    Note over V: no CI job runs verify yet (ci.yml:9-84)
```

**What it shows.** Run on this repository at `ae478d9`, `sealmap verify` treats
fifteen topics as legacy coverage and fails on one: MER-02 writes an example
id from a fixture crate (`shop`) as an inline code span, which reads as a
citation of a symbol nobody sealed.

**Why it is this way.** Legacy `path:line` topics must not fail the gate, or
a corpus could not migrate one topic at a time (`docs/DESIGN.md:119`-`121`);
a global-shaped id in a code span is always a citation, so a mistyped one
fails rather than vanishing (COR-04).

**Debt:** step 6 needs MER-02's example id moved into a fenced block or
reworded before `verify` can run in CI on this repository
(`crates/sealmap-corpus/src/seal/check.rs:390`, `docs/DESIGN.md:308`-`309`).
