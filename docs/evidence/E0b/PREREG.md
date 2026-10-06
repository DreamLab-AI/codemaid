# E0b pre-registration: region-level anchoring inside large symbols

Registered 2026-10-06, after E0 failed (R = 1.19) and before any E0b code was
written. E0 showed that cited symbols are often too coarse (`main()`,
`AppState::new`, module symbols): a body hash over hundreds of lines changes on
most commits. **Question: does anchoring each citation to the smallest
enclosing code region, not the whole symbol, flag materially fewer topics per
commit without hiding real changes?** Amendments are dated and appended below.

## Unchanged from E0

Inputs, pins, the commit window (the same 100 eligible commits per repository),
citation reading (E0 amendments #1, #3, #4, #9) and T_file are exactly E0's, so
E0b is directly comparable. The harness extends `bench/e0/`.

## The region rule

For a Rust citation at line L that E0 maps to symbol S, the **region** is the
innermost of these syntax nodes inside S whose span contains L:

- a `match` arm;
- an `if`/`else if`/`else` branch block, or a `while`, `loop` or `for` body;
- a closure body, or an `async` block;
- a statement directly inside any block (`let`, an expression statement, or an
  item statement);
- a struct field, enum variant, trait item or impl item;
- for a line outside every item in a file (a `use`, an `impl` header): that
  top-level item.

If no node qualifies, the region is S itself. The region is identified by its
node path from S (kind and ordinal at each level), and its **region hash** is
BLAKE3-16 over the node's tokens, normalised exactly as sealmap 0.2.0 normalises
body tokens (whitespace, comments and lint attributes ignored). Region identity
across P and C uses the node path inside S. A region that cannot be found at one
side counts as changed.

## Per-commit counts

- **T_file**, **T_sym**: as E0.
- **T_region:** topics where a cited region's hash differs between P and C, or
  the region exists at exactly one side, or a fallback citation's file changed
  (fallback exactly as E0).

## Endpoints

1. **Primary:** R_region = ΣT_file / ΣT_region over VisionClaw's 100 commits.
   **Success: R_region ≥ 2.0.** Bootstrap 95% CI as E0 (seed 20261006).
2. **Co-primary, now pre-registered (post hoc in E0):** the same ratio with each
   topic restricted to its `.rs` sources and citations. **Success: ≥ 2.0.** This
   isolates what anchoring can do where sealmap can read the code.
3. **Secondary:** R_region for agentbox and pooled; R_sym repeated for
   reference; medians and p90s; the share of Rust citations whose region is the
   whole symbol (no narrower node found).
4. **Hidden-change precision**, fixing E0's defect: pairs flagged by T_file but
   not T_region are drawn **only from commits after the topic's stamp in that
   repository**. 40 pairs (30 VisionClaw, 10 agentbox, or all if fewer; seed
   20261006). Each is judged by a fresh Claude subagent per E0's
   `judge/PROTOCOL.md`, with every "yes" re-checked by a second independent
   judge. A hidden real change needs both judges to say yes. **Success: rate ≤
   10%**, Wilson 95% upper bound reported. If fewer than 20 eligible pairs exist,
   endpoint 4 is reported as underpowered, not passed.

**E0b holds** if endpoint 4 succeeds **and** either endpoint 1 or endpoint 2
succeeds. If it holds through endpoint 2 only, the result reads: region anchoring
works on Rust, and the estate's non-Rust citations are the remaining limit.
Region hashing moves into the sealmap crates only if E0b holds.

## Amendments

None.
