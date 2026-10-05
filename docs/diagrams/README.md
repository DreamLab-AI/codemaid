# sealmap diagram corpus

A hand-authored, citation-verified map of the sealmap workspace: one topic
file per subsystem, each explained twice (for whoever inherits the code, and
for the agent-harness owners who adopt sealmap), around Mermaid diagrams whose
every mechanism claim cites `path:line` at the commit stamped in the topic.

This corpus is **not** `docs/sealmap/`. That directory is sealmap's own
generated, one-document-per-source-file output, checked by `sealmap verify`
in CI. This one is written by hand with the `diagrams-as-code` method and
checked by `tools/diagram-index-gen.cjs`. It catalogues the state of play and
proposes no fixes; the register is the input to later work, not a plan.

All topics are verified against `af4b8b4` (the tip of `main` when the corpus
was written). Governing documents: [`docs/DESIGN.md`](../DESIGN.md) (accepted
design) and the root [`README.md`](../../README.md). There are no ADRs.

## Doors

- **New to the code?** Read in order: MOD-01 (ids) → MOD-03 (the model) →
  EXT-01 (the adapter pipeline) → COR-01 (the generated corpus).
- **Deciding whether to adopt sealmap in a harness?** Read the "For the
  business" sections of MOD-02, EXT-05, DEL-01 and DEL-02, then
  [REGISTER.md](REGISTER.md).
- **Changing the resolver or the walker?** EXT-03, EXT-04, EXT-05, and the
  Debt rows from them in the register.
- **Feeding a codebase to an agent, or building `pack`?** DEN-01: the dense
  format, its marks, and the slice that refuses rather than truncates.
- **Planning 0.2 work?** DEL-02 is the gap between `docs/DESIGN.md` and the
  code; [DECISIONS-TIMELINE.md](DECISIONS-TIMELINE.md) is how it got here.

## Areas

| Area | Prefix | Covers |
|---|---|---|
| model | MOD | `sealmap-model`: the `sym:` grammar, hashes and fingerprints, the `Codebase`, sources, schema v2 |
| extract | EXT | `sealmap-extract` plus `sealmap-rust`: the adapter pipeline, ids, flow walker, lowering and confidence, resolution, fingerprinting |
| mermaid | MER | `sealmap-mermaid`: typed writers, escaping, injective diagram ids |
| corpus | COR | `sealmap-corpus` plus the facade and CLI: generate, the index, projections, verify and write |
| delivery | DEL | CI, the MSRV job, determinism, publication, and design versus code |
| dense | DEN | `sealmap-dense`: the agent projection, short names, call trees, budgeted slices; and what reading it revealed about the adapter |

## Checking and regenerating

From the repository root:

```sh
node docs/diagrams/tools/diagram-index-gen.cjs docs/diagrams --check --cite-check --strict-citations
node docs/diagrams/tools/diagram-index-gen.cjs docs/diagrams --check --render   # needs mmdc
node docs/diagrams/tools/diagram-index-gen.cjs docs/diagrams                    # rewrites the block below, COVERAGE.md, REGISTER.md
```

`rendered/` is gitignored. When code changes, re-read the cited symbols at the
new commit, correct the lines that moved, and bump that topic's
`verified_commit`; leave topics whose sources did not change on their stamp.

<!-- BEGIN GENERATED DIAGRAM INDEX -->
_17 topic files, 92 diagrams. Regenerate with_ `node docs/diagrams/tools/diagram-index-gen.cjs docs/diagrams`.

### model

| ID | Topic | Diagrams | Kinds | Governing | ADRs |
|----|-------|----------|-------|-----------|------|
| MOD-01 | [The sym id grammar](model/01-the-sym-id-grammar.md) | 6 | classDiagram, flowchart, sequenceDiagram | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| MOD-02 | [Content hashes and per-symbol fingerprints](model/02-content-hashes-and-fingerprints.md) | 5 | classDiagram, flowchart, sequenceDiagram, stateDiagram-v2 | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| MOD-03 | [The codebase model, its sources and schema v2](model/03-the-codebase-model-and-schema-v2.md) | 5 | classDiagram, flowchart, sequenceDiagram | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |

### extract

| ID | Topic | Diagrams | Kinds | Governing | ADRs |
|----|-------|----------|-------|-----------|------|
| EXT-01 | [The language-adapter pipeline](extract/01-the-adapter-pipeline.md) | 5 | sequenceDiagram, flowchart, stateDiagram-v2 | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| EXT-02 | [Minting ids for Rust definitions](extract/02-minting-ids-for-definitions.md) | 5 | flowchart, sequenceDiagram | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| EXT-03 | [The flow walker and the raw flow IR](extract/03-the-flow-walker-and-raw-ir.md) | 7 | classDiagram, sequenceDiagram, flowchart, stateDiagram-v2 | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| EXT-04 | [Lowering, confidence and labels](extract/04-lowering-confidence-and-labels.md) | 5 | sequenceDiagram, flowchart | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| EXT-05 | [Workspace-wide name resolution](extract/05-workspace-name-resolution.md) | 6 | classDiagram, flowchart, sequenceDiagram | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| EXT-06 | [Signature and body fingerprints from token streams](extract/06-signature-and-body-fingerprints.md) | 6 | classDiagram, sequenceDiagram, flowchart | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |

### mermaid

| ID | Topic | Diagrams | Kinds | Governing | ADRs |
|----|-------|----------|-------|-----------|------|
| MER-01 | [Typed Mermaid writers and escaping](mermaid/01-typed-writers-and-escaping.md) | 5 | classDiagram, flowchart, sequenceDiagram | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| MER-02 | [Injective diagram ids from symbol ids](mermaid/02-injective-diagram-ids.md) | 5 | flowchart, sequenceDiagram | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |

### corpus

| ID | Topic | Diagrams | Kinds | Governing | ADRs |
|----|-------|----------|-------|-----------|------|
| COR-01 | [Generating the 1:1 corpus and its index](corpus/01-generating-the-corpus-and-index.md) | 5 | sequenceDiagram, flowchart, classDiagram | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| COR-02 | [Structure, sequence and overview projections](corpus/02-structure-sequence-and-overview-projections.md) | 5 | sequenceDiagram, flowchart | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| COR-03 | [Verify, write and the command line](corpus/03-verify-write-and-the-cli.md) | 5 | sequenceDiagram, flowchart, stateDiagram-v2 | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |

### delivery

| ID | Topic | Diagrams | Kinds | Governing | ADRs |
|----|-------|----------|-------|-----------|------|
| DEL-01 | [CI, the MSRV job and the determinism guarantees](delivery/01-ci-msrv-and-determinism.md) | 5 | flowchart, sequenceDiagram | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
| DEL-02 | [Design versus code, what 0.2 plans and the tree does not yet do](delivery/02-design-versus-code.md) | 6 | flowchart, stateDiagram-v2 | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |

### dense

| ID | Topic | Diagrams | Kinds | Governing | ADRs |
|----|-------|----------|-------|-----------|------|
| DEN-01 | [The dense agent projection](dense/01-the-dense-agent-projection.md) | 6 | flowchart, sequenceDiagram | [DESIGN.md](../../docs/DESIGN.md), [README.md](../../README.md) |  |
<!-- END GENERATED DIAGRAM INDEX -->
