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

The "None." above was true at registration. The entries below were written
while building the harness, **before the first E0b run and before any E0b
endpoint was computed**. Each records how an ambiguous point was made
concrete; none changes the region rule's list of nodes, an endpoint, a
threshold, the window, the seed or the sampling.

### 2026-10-06 #1: node paths follow sealmap's canonical tree

sealmap's normalisation treats `=> { e }` as `=> e` and `|x| { e }` as
`|x| e`. On the raw tree the first has one more node (the statement `e` inside
the block), so a brace-only rustfmt rewrite would move a citation's node path
and count as a change, although the region hash, and the symbol's
`body_hash`, are unchanged. Node identity is therefore read on the same
canonical form the hash uses: a match-arm or closure body block that sealmap's
own `Canon` collapses is transparent, its single statement is not a region,
and the walk continues into the expression. Whether a body collapses is
decided by running sealmap's `Canon` on the arm or closure, not by a second
copy of the rule.

### 2026-10-06 #2: kinds, ordinals and the top-level-item case

The node kinds are `arm`, `branch`, `loop`, `closure`, `async`, `stmt`,
`field`, `variant`, `trait_item`, `impl_item` and `item`, following the rule's
list. A path step's ordinal counts the earlier siblings **of the same kind**
under the same parent region (or under S). The branches of one
`if … else if … else` chain are siblings, because an `else if` is not itself a
region. S is never its own region, even when its kind is listed. The
top-level-item case applies when E0 mapped the line to a file's root module.
By E0 amendment #5 that happens when the line is inside no item symbol. It
does not apply to an inline `mod` symbol. Struct fields and enum variants are
not sealmap symbols, so a citation on a field line maps to its type and
narrows to `field#n`.

### 2026-10-06 #3: "innermost" among overlapping nodes

The innermost qualifying node is the deepest one (the longest node path)
whose line span contains the cited line. Among equally deep nodes that share
the line, it is the one with the smallest line span, and then the first in
source order. Line spans include attributes and doc comments, as sealmap's
symbol spans do.

### 2026-10-06 #4: comparing a region between P and C

A citation whose region is S itself is compared exactly as E0 compares a
symbol: `sig_hash` and `body_hash`, presence at one side only, and E0's
conservative absent-at-both rule. A narrower region is compared by its hash.
The hash is `Fingerprinter::body()` over the node's tokens through sealmap's
`feed` and `Canon`, with no section label. A closure body is taken from the
canonical closure and a statement from its canonical block, so
context-dependent rewrites apply as they do inside S. A narrower region
counts as changed in these cases:

- the hashes differ;
- the path is found at exactly one side;
- S is at both sides and the path is found at neither (conservative, since
  the file changed);
- S is absent at both sides and the cited file changed (E0's rule).

Each side reads S's file as named by that side's model. A region whose
symbol's file is byte-identical at P and C is unchanged without parsing.

### 2026-10-06 #5: finding S's syntax node

S's node is the item, impl item, trait item, field or variant whose span
matches S's sealmap span exactly, lines and columns, under sealmap's own
convention. Ties go to the first node in pre-order. The root module is the
whole file. If S's node cannot be found at the stamp, the citation's region
is S, and the number of such citations is reported.

### 2026-10-06 #6: "after the topic's stamp" and the 20-pair floor

A (commit C, parent P) pair counts as after the topic's stamp in that
repository when the stamp is an ancestor of P or equal to it. In other words,
C was made after the topic text was verified. C equal to the stamp does not
count. A topic with no usable stamp has no eligible pairs. "Fewer than 20
eligible pairs" is counted over both repositories pooled, and the per-repo
counts are reported. The draw follows E0 amendment #8: window order, then
topic order, one ChaCha8 generator seeded 20261006, VisionClaw then agentbox.

### 2026-10-06 #7: where the normaliser comes from

sealmap-rust's normaliser (`crates/sealmap-rust/src/fingerprint.rs`) is
crate-private. The harness compiles that file unchanged into `bench/e0` with a
`#[path]` module, so the published crates' API does not change and no second
copy of the normalisation exists. Endpoint 2's `.rs` restriction is E0
amendment #10's: each topic's `.rs` sources and citations.

### 2026-10-06 #8: two lookup checks; the run as found (written after the first run)

The first E0b run gave endpoint 1 R_region = 1.20 (CI 1.13–1.29) and
endpoint 2 R_region = 1.37 (CI 1.26–1.54), both below 2.0. That run counted
565 VisionClaw region events where S was present at P and C but the cited
path was found at neither side. To rule out a harness bug behind that number,
two checks were added after the run. One counts symbols that the model holds
but whose syntax node was not found at P or C. The other counts region files
that did not parse at P or C. Both are 0 in both repositories, so the
events are the path drift that amendment #4's conservative rule counts. The
counters change no return value, the re-run printed the same endpoint
figures, and no rule was changed.

That rule also explains the 16 VisionClaw (commit, topic) flags set by
T_region but not T_sym. In each, S's body hash is equal at P and C, but the
node path taken at a later stamp exists at neither side. They are reported
in RESULTS "Checks" and left in the counts.

Endpoint 4 has 36 eligible pairs (35 VisionClaw, 1 agentbox), which is at
least 20, so the endpoint is not underpowered. The draw takes 30 + 1, because
agentbox has fewer than 10 ("or all if fewer"). Endpoints 1 and 2 both
failed, so E0b does not hold whatever endpoint 4 shows. The sample and
prompts are in `judge/`, unjudged, and endpoint 4 is reported as PENDING.
