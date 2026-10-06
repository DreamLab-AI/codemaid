# E0 harness (`publish = false`)

Runs the experiment pre-registered in
[`docs/evidence/E0/PREREG.md`](../../docs/evidence/E0/PREREG.md) and writes
`results.json`, `RESULTS.md`, `TIMING.md` and `judge/` next to it.

```sh
cargo run --release -p sealmap-bench-e0 -- run \
  --corpus ~/workspace/VisionFlow \
  --visionclaw ~/workspace/project \
  --agentbox ~/workspace/project/agentbox \
  --out docs/evidence/E0 --scratch /tmp/e0-scratch
```

The input repositories are only ever read at the pinned shas
(`git ls-tree`, `cat-file`, `diff-tree`, `show`). No checkout, ref, index or worktree
is touched, so another session can commit to them while the harness runs. For each
revision, the extractor's input (`.rs`, `Cargo.toml` and the ignore files) is
written to the scratch directory and extracted with
`sealmap_rust::extract_dir(.., &RustOptions::default())`. Models are memoised by a
BLAKE3 key over that input tree, so a commit that touches no Rust reuses its
parent's model. A full run takes about 90 s and is byte-identical across runs,
except `TIMING.md`.

Endpoint 4 needs independent judges. The harness writes `judge/<id>.prompt.md`,
`judge/sample.json` and `judge/PROTOCOL.md` and never produces a verdict. Once the
`judge/<id>.j1.json` files exist (plus `.j2.json` for every first `yes`), run:

```sh
cargo run --release -p sealmap-bench-e0 -- score --out docs/evidence/E0
```

That writes `endpoint4.json` and `ENDPOINT4.md`.

## E0b: region anchoring

`run-b` runs the experiment pre-registered in
[`docs/evidence/E0b/PREREG.md`](../../docs/evidence/E0b/PREREG.md) over the same
inputs, pins and window, and writes to `docs/evidence/E0b` (never to E0's directory).
`run` computes no regions and still reproduces E0 byte for byte.

```sh
cargo run --release -p sealmap-bench-e0 -- run-b \
  --corpus ~/workspace/VisionFlow \
  --visionclaw ~/workspace/project \
  --agentbox ~/workspace/project/agentbox \
  --out docs/evidence/E0b --scratch /tmp/e0b-scratch
```

Each Rust citation is narrowed to the innermost match arm, branch or loop body,
closure or `async` body, block statement, field, variant, trait or impl item (or, for
a line outside every item, the top-level item) inside its E0 symbol (`src/region.rs`).
The region is named by its node path from the symbol, and its hash runs through
sealmap-rust's own normaliser. `crates/sealmap-rust/src/fingerprint.rs` is compiled
into this harness unchanged through a `#[path]` module. T_region is counted in
`src/rcount.rs` and reported by `src/report_b.rs`. Endpoint 4 draws only pairs on
commits after the topic's stamp. Score it with `e0 score --out docs/evidence/E0b` once
verdicts exist. A run takes about two minutes.

## E0c

`e0 e0c` (same arguments, `--out docs/evidence/E0c`) runs the experiment in
[`docs/evidence/E0c/PREREG.md`](../../docs/evidence/E0c/PREREG.md). It reuses E0's
window and citation mapping unchanged, keeps only citations in `sequenceDiagram`
blocks, and compares T_file^seq with T_flow. T_flow is a BLAKE3-16 hash over a
symbol's `sig_hash` and its ordered calls as sealmap's Rust adapter resolves them
(`src/flow.rs`). The rest of the experiment lives in `src/e0c.rs`. Once verdicts
exist, `e0 e0c-score --out docs/evidence/E0c` scores endpoint 4.

## E0d

`e0 e0d` (same arguments, `--out docs/evidence/E0d`) runs the experiment in
[`docs/evidence/E0d/PREREG.md`](../../docs/evidence/E0d/PREREG.md): a
language-agnostic overlap rule, T_hunk(k) (`src/hunk.rs`). Each citation is
relocated from its topic's stamp to P through one `git diff -U0` between the two
blobs. The topic is flagged when a P→C hunk lies within k lines of a relocated
citation, when a cited file is deleted, or when a lost or ambiguous citation's
file changed. Every hunk-placing git option is pinned on the command line.
`src/e0d.rs` recomputes T_file, T_sym, T_hop (E0), T_region (E0b) and T_flow
(E0c) on the same commits with their own code. It draws 45 + 15 post-stamp
T_file pairs and writes blind label prompts to `labels/`. The detectors' flags
for those pairs go to `detectors.json`, outside `labels/`. A run takes about
three minutes. Once labels exist (`labels/<id>.l1.json`, plus `.l2.json` for
every first `yes`), `e0 e0d-score --out docs/evidence/E0d` scores recall and
precision for every detector and reads each ratio together with its recall.
