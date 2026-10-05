# Decisions timeline

The register's sibling view: decisions and their reversals in time order,
drawn from `git log` on `main` (all of it on 2026-10-05) and from
`docs/DESIGN.md`. Times are commit times as recorded (UTC unless marked).
Hand-written; the generator does not produce it. A `docs/DESIGN.md:N` reference
in a row dated before step 3 is a line of the design at `af4b8b4`, the
revision that row describes.

## Before the design

| When | Commit | Decision | Later |
|---|---|---|---|
| 14:40 | `47d7dc2` | **codemaid v0.1**: five crates (model, zero-dependency Mermaid writers, syn frontend, 1:1 corpus with a verify/write contract, facade and CLI). The self-generated corpus is committed in `docs/codemaid` and verified in CI. | The committed corpus and its CI gate are retired by the design (`docs/DESIGN.md:44`) and removed at `d29bc8f`. |
| 15:42–16:12 | `d5ac063`, `08dfe4c`, `982323a` | Merged into the repository; repository URL points at `DreamLab-AI/codemaid`. | Repository renamed to `DreamLab-AI/sealmap` (`docs/DESIGN.md:251`). |

## The design is written and accepted

| When | Commit | Decision | Later |
|---|---|---|---|
| 19:10 | `f957346` | Estate README and governing design. The premise changes: from "committed 1:1 corpus with a drift gate" to a deterministic substrate under an LLM skill, with sealed, symbol-granular citations (`docs/DESIGN.md:3`-`4`, §1–§3). | — |
| 19:16 | `51ea4cf` | **Design accepted**; owner decisions recorded (§11): `topic_hash` yes, lock plus pointer yes, rename done, crate names reserved by a real release not placeholders, a model-routing ADR in agentbox, the Gemini key rotated and kept out of repositories. The open review prompt is dropped as the primary A/B arm (`docs/DESIGN.md:211`). | None of the seal surface is built yet (DEL-02). |

## Step 1 of the order of work

| When | Commit | Decision | Later |
|---|---|---|---|
| 19:20 | `8af26b3` | **Hardening** (§7): 64 MiB-stack collectors with a panic guard and an expression-depth cap; `.gitignore`-aware loading (934 files walked on VisionClaw instead of 47,229); glob imports through a table and a BFS; extract once in the CLI; corpus lookups indexed. Diagram ids: readable when plain, **FNV-1a suffix on collision**, plus a corpus-wide uniqueness assertion. An ignored regression test is added for the `Path::parent` lane fault. | The FNV suffix is removed at `337301a`; the ignored test is un-ignored at `5aac8a8`. |
| 19:22 | `34f8ea5` | **Rename codemaid → sealmap**; the CLI's default output directory becomes `.sealmap`; the self-corpus moves to `docs/sealmap`; CI pins `--name sealmap` so the checkout directory cannot leak into output. | `.sealmap` is described as gitignored but is not (DEL-02.6). |
| 19:22 | `0d1fe6b` | **Licence MIT OR Apache-2.0** for every crate. | — |
| 19:29 | `a02b381` | **Extract the shared core** as `sealmap-frontend` from `sealmap-rust`, proven byte-identical before and after on VisionClaw, tokio and the self tree. | Renamed `sealmap-extract` at `faf08e4`. |
| 19:32 | `d308faf` | Publication readiness: per-crate READMEs with doctests, docs.rs metadata, `#![deny(missing_docs)]`. | No crate from this tree is published yet. |

## Step 2 of the order of work

| When | Commit | Decision | Later |
|---|---|---|---|
| 20:18 | `5973a4e` | **`sym:` id grammar** (SCIP-descriptor style, kind-explicit, no file path, `.` version). Methods sit under their owning type; trait impls add `[Trait]`; foreign-type impls anchor at `module/impl#[SelfTy][Trait]`. The resolver splits types from values. Mermaid ids "go through the existing hashed path mangling for now". | Mermaid ids rebuilt on the grammar at `337301a`. |
| 20:35 | `0a304ba` | **Per-symbol `sig_hash` and `body_hash`**, algorithm `sm1`, golden values pinned; rustfmt rewrites and lint attributes ignored. **`flow_hash` is not added** ("research 05 does not call for it"). | The design's crate table now records it as considered and not built (`6e0604c`). |
| 20:36 | `01f82b4` | **Schema v2** for `_model.json` and `_index.json`; `Codebase::from_json` refuses other versions; **no v1 reader**. | — |
| 20:43 | `337301a` | **Injective diagram ids** from the `sym:` id structure; the FNV-1a suffix and `Ident::from_path` removed as dead; the uniqueness assertion kept as a guard. | The design's hardening table corrected at `6e0604c` (MER-02.5). |
| 20:56 | `5aac8a8` | **Receiver provenance**: names bound by patterns inherit what is known about the scrutinee; a value derived from std or a dependency is never matched to an internal method by name; function-local `use` widened to the module. On VisionClaw: 76 name guesses withdrawn, 73 exact edges gained. | Untyped closure parameters still produce guesses (EXT-05.6). |
| 21:00 | `dc5e025` | **MSRV job** on Rust 1.85. It finds that `ignore` 0.4.30 needs Rust 1.88 without declaring it; the **lockfile pins 0.4.29** while the requirement stays `0.4.23`. | Holds inside the workspace only (DEL-01.2). |
| 21:02 | `33c9d3c` | Self-corpus regenerated for `sym:` ids and schema v2. | — |
| 21:06 | `faf08e4` | **`sealmap-frontend` renamed `sealmap-extract`**; per-language crates are "language adapters". The published `sealmap-frontend` 0.1.0 stays and is to be deprecated at 0.2. **`sealmap-ts` deferred** until E0-R shows the precise-staleness gain is real (`docs/DESIGN.md:174`-`177`). | The facade's planned default `sealmap-ts` feature is still in the design (DEL-02.5). |
| 21:08 | `3f22f3c` | README states what exists after step 2 and marks seals, staleness and packs as planned. | Resolved at step 3: `.sealmap/` gitignored, `--since` built, `pack` marked planned (`d29bc8f`, `6e0604c`). |
| 21:14 | `6c8a9b0` | **Dogfood fixes** found by reading the self-corpus: an undefined member of an internal type stays with the type (`Type#member().`); external path calls are no longer double-qualified. | The undefined-member id names a symbol the model does not hold (EXT-02.5). |
| 21:14 | `af4b8b4` | Self-corpus regenerated; `sealmap verify` clean. **This corpus was first stamped here.** | Re-stamped at `ae478d9` (all topics). |
| 22:22 | `5fab3bc` | The parse-error tag becomes `PARSE_ERROR_TAG` in the model, so the seal check can **fail closed from the model alone**, without adapter diagnostics. | — |
| 22:22 | `9e10f38` | **The seal module.** Canonical lock (re-serialisation is the canonicality check); `topic_hash` = BLAKE3-16 with the `sealed:` line removed; citations are code spans that start like a global id; the nine classes of DESIGN §3; legacy topics count as coverage, not failures; rename candidates restricted to the same kind. | Sealing a module is weak: its body follows member order (COR-04.5). |
| 22:22 | `702530e` | **The seal CLI.** `verify` becomes the seal gate; the 1:1 drift check becomes `generate --check`; `stale --since` exports a revision with `git archive` and reads it under the current codebase name. | Seals do not record extraction options (COR-05.1). |
| 22:23 | `d29bc8f` | **Generated corpus retired**: `docs/sealmap` deleted, `.sealmap/` gitignored; CI checks that two fresh generations are byte-identical and renders a fresh corpus. | — |
| 22:25 | `6e0604c` | README and DESIGN state the seal surface as built; `flow_hash` and the hash-suffix row corrected. | — |
| 22:26 | `ae478d9` | Reading the generated diagrams of the new code finds field doc comments leaking into enum-variant labels; fixed. **Every topic is re-stamped here**; COR-04 and COR-05 are new. | Generated diagrams still omit call-free match arms (COR-04.5). |

## Reversals at a glance

| Subject | First | Then | Now |
|---|---|---|---|
| Diagram ids | `::` → `__` mangling (0.1) | readable plus FNV-1a suffix (`8af26b3`) | injective encoding of the `sym:` id (`337301a`) |
| Shared extraction crate | inside `sealmap-rust` | `sealmap-frontend` (`a02b381`) | `sealmap-extract` (`faf08e4`) |
| The 1:1 corpus | committed, CI drift gate (`47d7dc2`) | retired in the design (`f957346`) | removed; CI checks determinism instead (`d29bc8f`) |
| `sealmap verify` | the 1:1 drift check (`47d7dc2`) | — | the seal gate; the drift check is `generate --check` (`702530e`) |
| TypeScript adapter | designed, 31–36 engineer-days (`docs/DESIGN.md:188`) | — | deferred (`faf08e4`) |
| `flow_hash` | optional in the design (`docs/DESIGN.md:101`) | not added (`0a304ba`) | recorded in the design as not built (`6e0604c`) |
| Primary review arm | the owner's open prompt | — | critical plus pre-mortem lenses (`docs/DESIGN.md:211`-`212`) |
