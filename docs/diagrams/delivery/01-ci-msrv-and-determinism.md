---
id: DEL-01
title: CI, the MSRV job and the determinism guarantees
area: delivery
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - .github/workflows/ci.yml
  - Cargo.toml
  - Cargo.lock
  - rustfmt.toml
  - tools/validate-mermaid.mjs
  - crates/sealmap-model/src/lib.rs
  - crates/sealmap-model/Cargo.toml
  - crates/sealmap-rust/src/lib.rs
  - crates/sealmap-rust/tests/extract.rs
  - crates/sealmap-corpus/tests/contract.rs
  - crates/sealmap-corpus/tests/seal.rs
  - crates/sealmap-corpus/tests/lock_props.rs
  - crates/sealmap/src/main.rs
  - docs/DESIGN.md
  - README.md
verified_commit: ae478d90d2b910101a1aa065cb4655e7defea332
---
## For developers

sealmap's delivery surface is one workflow with four jobs, a workspace
manifest that every crate inherits its metadata, lints and MSRV from, and a
handful of guarantees the code makes about its own output. This topic draws
the jobs, the MSRV problem the `msrv` job found the day it was added, the
chain of mechanisms behind "byte-identical on every machine", and how the
crates are prepared for crates.io.

The relevant history is short: the `msrv` job came in `dc5e025`; per-crate
READMEs, docs.rs metadata and `deny(missing_docs)` in `d308faf`; the dual
licence in `0d1fe6b`; the committed generated corpus and its drift gate were
retired in step 3, when CI started checking determinism instead. None of the
0.2 crates in this tree is published yet; the README's roadmap puts the first
0.2 crates.io release after `pack`, the last open item of step 3
(`README.md:349`-`352`).

## For the business

Three promises here are what make sealmap cheap to adopt in an agent harness.
It builds on a stated, tested minimum Rust (1.85), so a harness image does not
have to chase the newest toolchain. Its output is byte-identical for the same
sources and options on any machine, so results can be cached, diffed and
compared in CI without noise. And every crate is held to full documentation
and warning-free docs, which is the bar for a library others depend on.

There is one live caveat on the first promise. A dependency (`ignore` 0.4.30)
needs a newer Rust than it declares; inside this repository the lockfile
avoids it, but a project that depends on the published crates without that
lock can still pull it in on Rust 1.85 and fail to build.

## DEL-01.1 The four CI jobs

```mermaid
flowchart TB
    subgraph RUST["rust job, stable, ci.yml:9"]
        R1["fmt check<br/>ci.yml:17"]
        R2["clippy, all targets, warnings denied<br/>ci.yml:18"]
        R3["tests, default and no-default features<br/>ci.yml:19-20"]
        R4["docs with warnings denied<br/>ci.yml:21-23"]
        R5["determinism: two fresh generations,<br/>name pinned, compared with diff<br/>ci.yml:26-30"]
        R1 --> R2 --> R3 --> R4 --> R5
    end
    subgraph MSRV["msrv job, Rust 1.85, ci.yml:35"]
        M1["build, locked<br/>ci.yml:42"]
        M2["test, locked<br/>ci.yml:43"]
        M1 --> M2
    end
    subgraph MER["mermaid job, Node 22, ci.yml:45"]
        X1["install mermaid 12 and puppeteer<br/>ci.yml:54"]
        X2["generate a fresh corpus into .sealmap<br/>ci.yml:56"]
        X3["render every block, MODE render<br/>ci.yml:59-61"]
        X1 --> X2 --> X3
    end
    subgraph DIA["diagrams job, full history, ci.yml:66"]
        D1["structure and every path:line citation<br/>at its topic's stamp, strict<br/>ci.yml:79"]
        D2["mmdc render with the width ceiling<br/>ci.yml:80"]
        D1 --> D2
    end
    RUST --> MSRV --> MER --> DIA
```

**What it shows.** Stable gets the full lint, test and doc gate and a
determinism check; the declared MSRV gets build and test against the
lockfile; a freshly generated corpus is rendered by real Mermaid in headless
Chromium; and the hand-authored corpus is checked and rendered by its own
generator.

**Why it is this way.** Nothing generated is committed any more, so the gate
is that two generations agree byte for byte (`.github/workflows/ci.yml:24`-`25`),
not that a committed copy matches. Testing with `--no-default-features`
exercises the sequential collection path, which must give the same output as
the parallel one (`crates/sealmap-rust/src/lib.rs:67`-`70`). The Mermaid job
is the proof that typed writers produce valid diagrams (`README.md:307`-`310`).

**Debt:** the MSRV job runs only `build` and `test` with default features
(`.github/workflows/ci.yml:42`-`43`); clippy, docs and the
`--no-default-features` configuration are checked on stable only.

**Open:** no job runs `sealmap verify` yet: this repository's topics are not
sealed (DESIGN step 6 adds it, non-blocking first), so the seal gate is
exercised only by its tests.

## DEL-01.2 The ignore 0.4.30 problem

```mermaid
sequenceDiagram
    autonumber
    participant DV as a 1.85 toolchain
    participant MF as workspace manifest<br/>Cargo.toml:35
    participant LK as Cargo.lock<br/>Cargo.lock:263
    participant CI as msrv job<br/>ci.yml:35
    participant DS as downstream crate, no lock
    DV->>MF: requirement ignore 0.4.23 or later (Cargo.toml:35)
    Note over MF: rust-version 1.85 for every crate, Cargo.toml:8
    DV->>LK: --locked build (ci.yml:42)
    LK-->>DV: ignore 0.4.29, builds on 1.85 (Cargo.lock:264)
    CI-->>DV: green
    DS->>MF: resolves the newest compatible ignore
    MF-->>DS: 0.4.30 declares no rust-version, resolver cannot avoid it
    Note over DS: DEBT: 0.4.30 uses let-chains, needs Rust 1.88,<br/>Cargo.toml:32
```

**What it shows.** The repository builds on 1.85 because the lockfile pins
`ignore` 0.4.29; the requirement stays at `0.4.23` so downstream graphs can
unify on newer releases, which is exactly what exposes them to 0.4.30.

**Why it is this way.** The MSRV-aware resolver skips releases that declare a
newer `rust-version`, but 0.4.30 declares none, so it is chosen and fails; the
comment gives the manual remedy for a 1.85 user outside the workspace
(`Cargo.toml:32`-`34`, `.github/workflows/ci.yml:32`-`34`). `ignore` is a
dependency because the model crate's loader honours ignore files
(`crates/sealmap-model/Cargo.toml:21`).

**Tension (MSRV vs dependency):** `ignore` 0.4.30 needs Rust 1.88 without
declaring it (`Cargo.toml:32`-`33`), against the declared MSRV 1.85
(`Cargo.toml:8`); only the lockfile pin to 0.4.29 (`Cargo.lock:264`) keeps the
MSRV true, and a published crate ships without that lock.

## DEL-01.3 Where byte-identical output comes from

```mermaid
flowchart TB
    O["ordered collections only<br/>sealmap-model/src/lib.rs:24"]
    P["normalised relative paths<br/>sealmap-model/src/lib.rs:27"]
    H["BLAKE3, never a seeded hasher<br/>sealmap-model/src/lib.rs:30"]
    A["no timestamps, absolute paths,<br/>user names or environment<br/>sealmap-model/src/lib.rs:33"]
    PO["parallel results in input order<br/>sealmap-rust/src/lib.rs:67"]
    BY["byte-identical model, corpus and index"]
    T1["extraction deterministic regardless of<br/>insertion order, tests/extract.rs:231"]
    T2["generation byte-identical across runs<br/>and input order, tests/contract.rs:93"]
    T3["CI: two fresh generations compared<br/>ci.yml:29-30"]
    T4["lock writer: order-independent signing,<br/>byte-stable round trip, tests/seal.rs:554"]
    NM["codebase name: --name, else the<br/>checkout directory's name<br/>main.rs:351, main.rs:377-382"]
    O --> BY
    P --> BY
    H --> BY
    A --> BY
    PO --> BY
    BY --> T1
    BY --> T2
    BY --> T3
    BY --> T4
    NM -.->|leaks into output| BY
```

**What it shows.** Five mechanisms feed the guarantee, two tests and a CI step
hold it for generated output, and a test plus a property test
(`crates/sealmap-corpus/tests/lock_props.rs:46`) hold it for the seal lock;
one input, the codebase name, comes from outside the source bytes unless it is
pinned.

**Why it is this way.** The model crate states the four-clause contract that
downstream crates rely on (`crates/sealmap-model/src/lib.rs:18`-`34`). CI pins
`--name sealmap` so the checkout directory cannot leak into the output
(`.github/workflows/ci.yml:29`).

**Tension (determinism contract vs CLI):** "no ambient data"
(`crates/sealmap-model/src/lib.rs:33`-`34`) holds for the model's types, but the
CLI names the codebase after the canonicalised checkout directory when no
name is given (`crates/sealmap/src/main.rs:351`, `crates/sealmap/src/main.rs:377`-`382`),
and that name reaches the index and overview, so two clones in differently
named directories produce different bytes. For seals the name matters only
where no `Cargo.toml` names the package, which is why `stale --since` reads
the older tree under the current tree's name.

## DEL-01.4 Crates, dependencies and publication

```mermaid
flowchart TB
    WS["workspace: version 0.1.0, edition 2024,<br/>rust-version 1.85, MIT OR Apache-2.0<br/>Cargo.toml:5-9"]
    LN["lints: unsafe forbidden, missing docs<br/>warned, clippy all<br/>Cargo.toml:39-44"]
    MO["sealmap-model"]
    MM["sealmap-mermaid, model optional"]
    XT["sealmap-extract"]
    RU["sealmap-rust"]
    CO["sealmap-corpus"]
    DE["sealmap-dense"]
    FA["sealmap facade and CLI"]
    DT["every crate: README doctests<br/>via include_str, deny missing_docs<br/>sealmap-model/src/lib.rs:114-117"]
    REL["release: thin LTO, one codegen unit<br/>Cargo.toml:46-48"]
    WS --> LN
    MO --> MM
    MO --> XT --> RU
    MO --> CO
    MM --> CO
    RU --> FA
    CO --> FA
    MO --> DE --> FA
    LN --> DT
    WS --> REL
```

**What it shows.** Seven crates inherit version, edition, MSRV, licence,
repository and lints from one manifest; the dependency graph runs model →
extract → rust, model/mermaid → corpus and model → dense, with the facade on
top.

**Why it is this way.** Workspace path dependencies carry a version too
(`Cargo.toml:17`-`22`), so the crates can be published without editing
manifests; the README examples of every crate compile as doctests, so the
crates.io page cannot drift from the API (`crates/sealmap-model/src/lib.rs:114`-`117`).

**Open:** the workspace lint level for missing docs is `warn`
(`Cargo.toml:41`) while every crate root raises it to `deny`
(`crates/sealmap-model/src/lib.rs:78`); nothing records which level a new
crate is meant to start from.

## DEL-01.5 The Mermaid validator

```mermaid
sequenceDiagram
    autonumber
    participant CI as mermaid job<br/>ci.yml:59
    participant VM as validate-mermaid.mjs<br/>validate-mermaid.mjs:7
    participant BR as headless Chromium with mermaid
    CI->>VM: every md under the fresh .sealmap, MODE render (ci.yml:61)
    VM->>VM: collect fenced mermaid blocks (validate-mermaid.mjs:11)
    VM->>BR: launch, load mermaid.min.js, raise text and edge limits (validate-mermaid.mjs:14)
    loop chunks of 100 blocks
        VM->>BR: render, or parse when MODE is unset (validate-mermaid.mjs:28)
        BR-->>VM: null or the error text
    end
    VM-->>CI: count failed, exit 1 if any (validate-mermaid.mjs:41)
```

**What it shows.** The validator extracts every fenced block and asks real
Mermaid to lay it out, in batches, in one browser page.

**Why it is this way.** Rendering in the real library is the only check that
matches what a reader's renderer does; the edge and text limits are raised so
large generated diagrams are judged on grammar and layout, not size
(`tools/validate-mermaid.mjs:19`). The script defaults to `parse`
(`tools/validate-mermaid.mjs:15`); CI asks for a full render because some
faults only surface at layout (`.github/workflows/ci.yml:57`-`58`).
