# sealmap design

Status: proposed for owner sign-off, 2026-10-05. Supersedes the v0.1 README's
"committed 1:1 corpus with a drift gate" premise. Evidence: `research/01`–`05` in
the design session scratchpad. Each section below names the report it summarises.

## 1. What sealmap is

sealmap is a deterministic substrate under an LLM diagram skill. It does not
write the diagrams people read. It does the bookkeeping that makes them cheap
to keep true:

- it extracts a code model;
- it gives every symbol a stable id and two content hashes;
- it checks that reviewed diagrams still describe the code they were sealed
  against;
- it builds bounded review packs on demand.

The LLM skill does the consolidation, the narratives, the register and the
review that produces a seal.

> **Panel line (draft):** *the seal and the distillation do the work, not a
> bigger model.*

### Crate rule

A capability goes into a crate only if all three hold:

1. it must be deterministic;
2. it is useful outside this estate;
3. it is cheap to keep general.

Everything else lives in the skills.

## 2. Three layers

| Layer | Artefact | Committed | Written by | Gated by |
|---|---|---|---|---|
| **Authored** | `docs/diagrams/<area>/NN-*.md`: consolidated topics, narratives, register; citations are `sym:` ids | yes | LLM skill, tiered (§6) | `sealmap verify` (ids resolve) + `diagram-index-gen.cjs --check --render` |
| **Sealed** | `docs/diagrams/seals.lock` (and `docs/adr/seals.lock`); each topic and ADR carries a static `sealed: seals.lock` line | yes | the skill's seal step | `sealmap verify`: **the CI gate** |
| **Generated** | model, index, dense projection, optional 1:1 Mermaid, under `.sealmap/` | **never** (gitignored) | `sealmap generate` | rebuilt in memory; never trusted from disk |
| **Review pack** | `sealmap pack`: pegged topics + dense slices + bounded source windows; `--review` = diagrams only | never | crate | Gemini external review, seal review, debugging |

**Retired from v0.1:** the committed `docs/codemaid/` corpus and its CI drift
gate. The 1:1 writer stays as an optional view, and `generate --check` becomes a
local consistency check.

## 3. The seal

A seal records that **topic T's claims were reviewed against these exact symbol
versions**.

```toml
version = 1
algorithm = "sm1"
generator = "sealmap 0.2.0"

[[topic]]
id = "CP-03"
file = "control-plane/03-the-scr-executor.md"
topic_hash = "blake3-16:…"     # binds the prose: an unreviewed edit fails verify
reviewer = "zai:glm-5.3"       # opaque to the crate
model = "claude:claude-sonnet-5"
date = "2026-10-05"
symbols = [{ id = "sym:ts:control-plane/src/scr/executor.ts#execute()", sig = "…", body = "…" }]
```

- **Lock plus pointer.** Hashes live in the lock, so re-seals never rewrite
  authored prose and reviewers read one file. The static `sealed:` pointer keeps
  any topic self-describing. `verify` fails if the pointer and the lock
  disagree.
- **Canonical form.** The lock is sorted TOML serialised by the crate. Merge
  conflicts stay per topic, and `seal.mjs sign` re-derives an entry
  mechanically.
- **Moves cost nothing.** `sym:` ids carry no line numbers, so a code move
  changes neither the lock nor the topic. The skill's index pass renders
  `path:line` at HEAD for human readers.

### What `verify` checks

| Condition | Class |
|---|---|
| id resolves, `sig` and `body` match | holds |
| `sig` same, `body` differs | behaviour |
| `sig` differs | contract |
| id gone; another id has the same `body` | absent (rename suspected) |
| file holding a sealed id fails to parse | unparsable, fail closed |
| `sym:` cited but not sealed, or a lock entry with no file | unsealed citation / orphan |
| `topic_hash` differs | prose edited since seal |
| pointer and lock disagree, or the lock is not canonical | lock fault |

Everything except **holds** exits 1.

Reviewer family, evidence and signatures are skill policy, enforced by
`seal-gate.mjs` next to `verify`. That keeps the crate estate-neutral.

## 4. Crate surface

| Crate | Owns |
|---|---|
| `sealmap-model` | language-neutral model; **`sym:` grammar** (SCIP-descriptor style, kind-explicit, no file path, version `.`); `sig_hash` / `body_hash` / optional `flow_hash` (BLAKE3 over comment- and whitespace-insensitive token streams, after rustc fingerprints); schema v2 |
| `sealmap-frontend` | shared by both frontends: raw flow IR, flow normalisation, call aggregation, confidence lattice, label rules, id builder and hashing. Extracted from `-rust` first, with no behaviour change. This is the parity lever |
| `sealmap-rust` | syn frontend, hardened (§7) |
| `sealmap-ts` | oxc frontend (§8) |
| `sealmap-mermaid` | typed writers, short participant aliases (−26.5 % tokens), **injective** ids with a uniqueness assertion |
| `sealmap-dense` | agent projection: indented call trees + skeletons with `L<start>-<end>` + `_index.txt`. 0.45× source, about 13 tokens per call edge against 42 |
| `sealmap-corpus` | generate, the `seal` module (lock parse and canonical write), `resolve`, `seal-check`, `stale`, `pack`, `verify` |
| `sealmap` | facade and CLI; `sealmap-ts` behind a default feature, because oxc needs MSRV 1.97 |

### CLI

| Command | Does |
|---|---|
| `resolve <sym:…>` | current span and hashes, or `absent` plus rename candidates |
| `stale [--since rev]` | topics whose sealed symbols changed, with class; exit 0 |
| `seal-check` | classify the given lock entries |
| `verify` | `seal-check` across the corpus, plus coverage and lock faults |
| `pack <topics\|--diff old new> [--review] [--max-bytes]` | deterministic blob. Refuses with a shard plan when over budget; never silently truncates; never includes the lock |
| `generate [--check]` | writes `.sealmap/` |
| `export --scip` (later) | interop |

Git stays out of the libraries. `--diff` takes two trees, and `--since` is CLI
sugar that shells out to `git`.

Licence `MIT OR Apache-2.0`. Every crate is published to crates.io with full
rustdoc, a README, and a clean `cargo doc --no-deps`. Skills pin the published
version, never a path dependency.

## 5. Skills

| Skill | State | Role |
|---|---|---|
| `sealmap-review` | **shipped** (agentbox `c4ea4efca`, `800d5494a`) | External mode: diagrams only, single-shot to Gemini 3.8 Flash, high thinking, `critical` + `premortem` lenses, pack-first ordering for cache reuse. Inline mode: Claude subagent checks topics against cited code. Findings go to build-with-quality intake as hypotheses (test first, then confirmed / rejected / known). Trigger set: train 12/12, held-out 6/8, zero false triggers |
| `sealmap` | to build with skill-builder | Read the generated substrate; the seal workflow (`seal.mjs propose\|sign\|rehome`, `seal-gate.mjs`, `cite-to-sym.mjs` migration); tiers (`tier.mjs`); `edge-check.mjs` (diagram-ir + `resolve` catches invented edges); the A/B bench |
| `diagrams-as-code` | small amendment | `diagram-index-gen.cjs` accepts `sym:` and `sealed:`; `--cite-check` kept for legacy `path:line` topics; the researcher role becomes `resolve` (zero tokens) |
| `build-with-quality` | small amendment | step 5a **Re-seal**: `stale` → behaviour (cheap cross-family review, sign) / contract (ADR addendum, re-consolidate, review, sign) / absent (confirm rename). Code, topic and lock land in **one commit**, ending today's two-commit routine |

When `sealmap pack --review` exists, `sealmap-review` swaps its own packer for
it. Until then the skill reads the corpus directly.

## 6. Model per step

Tiers come from the measured descent, not assumption. The skill names
`[model_routing]` activities and never names model ids.

| Step | Start | Escalate on |
|---|---|---|
| extraction, symbol → line, staleness, pack | **T0** sealmap (0 tokens) | — |
| diagram condensing, labels | **T1** Loom qwen3.8-27B / Haiku | render gate fails twice; diagram-ir finds an invented edge; > 25 % inferred calls |
| narratives, per-topic register | **T2** Sonnet | seal-review rejection; spans ≥ 3 areas |
| Tensions, ranking, cross-corpus synthesis | **T3** Opus | corpus too large → T4 Fable |
| seal review | **T3 cross-family** (GLM-5.3 if the author was Claude) | contract-class disagreement → T4 |
| per-commit maintenance | **T0** `stale`; behaviour → T2; contract → T3 | — |

De-escalation is sticky per corpus, in `docs/diagrams/.tiers.json`. It needs
four new activities (`diagram-polish`, `diagram-narrative`,
`diagram-synthesis`, `seal-review`) plus `loom` / `zai` hosts, so **an ADR**.

## 7. Rust hardening (done as a prototype; patch `research/03-rust-hardening.patch`)

| Fault | Cause | Fix |
|---|---|---|
| stack overflow on VisionClaw | deeply nested generics in vendored `typenum`, on rayon's 2 MiB stacks in debug | larger worker stacks, per-file panic guard, depth cap |
| 48 s and climbing | `.gitignore` ignored: 47,229 files walked, not 934 | respect ignore files |
| hang on older crates | exponential glob-import resolution (908 M calls on one file) | memoised table: 43 ms |
| id collision | `::` → `__` is not injective | readable ids + hash suffix on collision + corpus-wide assertion |
| wrong lane | `Path::parent()` drawn on `SymbolId` | resolver fix in the id work |

**Result:** VisionClaw generates in **1.0 s**, tokio in 0.24 s, with
byte-identical output across runs. Tests, clippy, fmt and docs are clean.

## 8. TypeScript frontend

- **Targets:** co-created / campaignbuilder (about 700 `.ts` / `.tsx`).
- **Parser:** oxc parser + semantic + resolver, versions pinned exactly.
- **Ids:** the same `sym:` grammar. Route handlers get synthesised ids
  (``index/`POST /api/provision`().``). Without them, the 52 anonymous handlers
  in `index.ts` stay file-granular, and E0's gain disappears.
- **Disambiguation:** two packages are both named `campaignbuilder`, so the
  package part carries a disambiguator.
- **Parity:** a shared conformance crate of about 40 dual-language cases, with
  a parity gate in CI.
- **Effort:** 31–36 engineer-days, about 4 weeks elapsed with Rust and TS in
  parallel after the `sealmap-frontend` split.

## 9. Evidence plan

The endpoints are a fixed sequence. Each is tested only if the previous one
holds, and all are pre-registered in `PREREG.md` before any endpoint run.

1. **E0, deterministic maintenance (no LLM).** Replay the last 100 code
   commits. Count topics invalidated per commit:
   - **today:** file-level, the corpus's `sources:`;
   - **sealed:** symbol-level, plus a one-hop-callee variant.
   - **Baseline measured:** median 10/45, p90 40/45 on campaignbuilder, driven
     by `index.ts` (39 topics) and `compose.pod.yaml` (31).
   - **Reported:** the reduction ratio, coverage honesty (share falling back
     to file level), and a precision sample for real changes hidden by
     symbol-level gating.
   - **Order:** Rust first (VisionClaw, agentbox); TS after `sealmap-ts`.
2. **External review yield.**
   - **Measure:** diagrams-only critical + pre-mortem review, recall of a
     frozen issue list per 100k pack tokens, against handing over the code.
   - **Pilot (n = 1):** blind critical review rediscovered 7 of 30 Tensions
     plus 10 known register items. New claims: 1 real, 5 overstated, 0 wrong.
   - **Changed:** the owner's open prompt was dropped as the primary arm (it
     flatters); critical + pre-mortem are baked in.
3. **Non-inferiority.** The substrate does not cost review recall (δ = 0.05).
4. **Cheaper upkeep.** Maintenance tokens per commit, today against sealed and
   tiered, lower bound > 1×.

Planted faults in **code**, distilled into diagrams, test the distillation end
to end. **Dogfood:** sealmap's own repository, sealed, is round 0.

## 10. Order of work

1. **Repository.** Rename `DreamLab-AI/codemaid` → `sealmap`, rename the
   crates, change the licence to dual. Apply the hardening patch. Extract
   `sealmap-frontend`.
2. **Ids and hashes.** `sym:` grammar, `sig` / `body` hashes, injective
   Mermaid ids, schema v2.
3. **Seal surface.** `seal` module plus `resolve` / `stale` / `seal-check` /
   `verify` / `pack`, and `sealmap-dense`. Retire the committed corpus.
   **First crates.io publish (0.2).**
4. **E0-R** on VisionClaw and agentbox. This is the first headline number.
5. **`sealmap` skill** (skill-builder): seal workflow, tiers, edge-check,
   bench. Amend diagrams-as-code and build-with-quality. Write the routing ADR.
   Bake via `lib/sealmap.nix`.
6. **Dogfood.** Seal sealmap's own corpus. `verify` runs non-blocking in CI for
   2 weeks, then blocking.
7. **`sealmap-ts`** (parallel from step 3), then E0-T on campaignbuilder.
8. **Review A/B.** Calibrate, freeze `PREREG.md`, run the endpoint window
   (≥ 30 commits).
9. **Migrate VisionFlow** area by area: `cite-to-sym`, then review, then seal,
   with `verify` running non-blocking first.

## 11. Decisions for the owner

1. **`topic_hash`: recommend yes.** Without it, prose can change after review
   and the seal still passes. The cost is a zero-token re-seal for cosmetic
   edits.
2. **Lock plus pointer (§3): recommend yes.**
3. **Repository rename and crate-name reservation at step 1:** both are public
   and can't be undone.
4. **The model-routing ADR** (§6): four activities and two hosts.
5. **Rotate the Gemini key**, then move it to agentbox `.env` as
   `GEMINI_API_KEY`.
