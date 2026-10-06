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
