# E0c pre-registration: call-flow staleness for sequence diagrams

Registered 2026-10-06, before any E0c code was written and while E0b was running
(E0b's results were not seen). Owner direction: the deterministic layer should
maintain **sequence diagrams only**, anchored per call; other diagram kinds stay
LLM-maintained at a lower cadence. **Question: if a topic's sequence diagrams are
flagged only when the calls they depict change (not when any body text
changes), are materially fewer flagged per commit, without hiding real
changes?** Amendments are dated and appended below.

## Unchanged from E0

Inputs and pins (VisionFlow cace76a…, VisionClaw 9c8dcdc…, agentbox
b487c95…), the same 100 eligible commits per repository, citation reading (E0
amendments #1, #3, #4, #9), the innermost-symbol mapping, and the fallback rule
for citations sealmap cannot read. The harness extends `bench/e0/`; E0's outputs
must stay byte-identical.

## Scope: sequence citations only

A **sequence citation** is a citation inside a `sequenceDiagram` block (the
generator's first-token rule). Both sides of every comparison use only these;
citations in other diagram kinds and in prose are excluded from E0c entirely,
because under the owner's split they are LLM-maintained on their own cadence.
Topics with no sequence citation are excluded.

## The flow hash

For a Rust symbol S, its **flow** is the ordered list of S's outgoing calls as
sealmap 0.2.0's Rust adapter extracts them (source order; each call recorded as
its resolution kind and target: the callee `sym:` id for exact and inferred
calls, the canonical external name for external calls), preceded by S's
signature hash. The **flow hash** is BLAKE3-16 over that list's canonical
serialisation. A body edit that changes no call, call order, callee or signature
leaves the flow hash unchanged.

## Per-commit counts

- **T_file^seq:** topics where a file holding one of its sequence citations
  changed between P and C.
- **T_sym^seq:** as E0's T_sym, restricted to sequence citations (reference).
- **T_flow:** topics where a sequence-cited symbol's flow hash differs, or the
  symbol exists at exactly one side, or a fallback sequence citation's file
  changed.

## Endpoints

1. **Primary:** R_flow = ΣT_file^seq / ΣT_flow over VisionClaw's 100 commits.
   **Success: ≥ 2.0.** Bootstrap 95% CI (seed 20261006).
2. **Co-primary:** the same with only `.rs` sequence citations. **Success: ≥
   2.0.**
3. **Secondary:** agentbox and pooled; R_sym^seq alongside; medians and p90s;
   the share of sequence citations that fall back; the number of topics in
   scope.
4. **Hidden-change precision:** pairs flagged by T_file^seq but not T_flow,
   drawn only from commits after the topic's stamp in that repository; 40 pairs
   (30 VisionClaw, 10 agentbox, or all if fewer; seed 20261006). Each is judged
   by a fresh Claude subagent per E0's `judge/PROTOCOL.md`, with the question
   narrowed to: **does this change make any sequence diagram in this topic
   wrong or misleading** (a call, branch condition, argument or return a
   diagram shows)? Every "yes" is re-checked by a second independent judge; a
   hidden change needs both. **Success: ≤ 10%**, Wilson 95% upper bound
   reported. Fewer than 20 eligible pairs → underpowered, not passed.

**E0c holds** if endpoint 4 succeeds and endpoint 1 or 2 succeeds. If it holds,
sealmap is restructured around `flows` (check, stale, draft); if not, the
generated-corpus direction is not rescued by narrowing to sequence diagrams,
and that is reported as found.

## Known risk, stated in advance

The flow hash ignores conditions and arguments that change without changing a
call. Sequence diagrams often show those (`alt` guards, message labels), so
endpoint 4 is where E0c can fail even if the ratio passes.

## Amendments

None.

The "None." above was true at registration. The entries below are appended,
never edited.

### 2026-10-06 #1: making the registration concrete (written before any E0c code ran or any number was seen)

Written while building the harness, before the first E0c run and before any
endpoint number existed. None of these changes an endpoint, a threshold, the
window, the seed or a mapping rule. Each one picks the most literal reading of
a point the registration leaves open.

- **Which topics are in scope.** A topic is in a repository's scope when at
  least one of its sequence citations resolves to a `sources:` file in that
  repository (E0 amendments #1 and #3). The window is still E0's 100 eligible
  commits, so a commit that touches none of a repository's sequence-cited files
  contributes 0 to every count.
- **What T_file^seq tracks.** It tracks the files that hold the topic's
  sequence citations. A `sources:` entry that no sequence citation names is not
  tracked, because both sides use only sequence citations. E0 excludes
  citations that resolve to no single source, and E0c excludes them too.
- **Callables only have flows.** sealmap 0.2.0 records a `Flow` only for
  callables. A sequence citation that maps to a non-callable or to a file's
  module (E0 amendment #5) therefore has a flow made of its signature hash and
  an empty call list. That is the literal definition. The share of such
  citations is reported.
- **Order and serialisation.** The call list is `Flow::calls()`, which is
  sealmap's own depth-first, source-order walk. Branches and loops are
  flattened, and their labels are not part of the flow. Each call is recorded
  as `exact`, `inferred` or `external`, followed by the text of its target
  `SymbolId`. For an external call that text is the canonical external name.
  The call list holds whatever external calls the adapter keeps under
  `RustOptions::default()`. The canonical serialisation is UTF-8, one
  `\n`-terminated line per item: first `sig <blake3-16:hex>`, then one
  `<kind> <target>` per call. The flow hash is the first 16 bytes of BLAKE3 over
  those bytes (`Fingerprint::from_blake3`).
- **A symbol absent on both sides.** T_flow's definition does not carry over
  E0's conservative rule (a cited symbol absent at P and at C, with its file
  changed, counts as changed). Read literally, such a symbol does not flag
  T_flow. A sensitivity row with the rule applied is reported. It is not an
  endpoint. T_sym^seq keeps the rule, because it is "as E0's T_sym".
- **Endpoint 2.** Both sides keep only the sequence citations whose file ends
  in `.rs`. Topics with no such citation are out of scope for that row.
- **"After the topic's stamp" (endpoint 4).** A (commit C, topic) pair is
  eligible when the topic's stamp in that repository is P (C's parent) or an
  ancestor of P, so that the whole change from P to C postdates the stamp. A
  topic with no usable stamp in that repository (`no_stamp`, `stamp_unknown`)
  contributes no pairs. Pairs are listed and drawn as in E0 amendment #8: window
  order, then topic file order, one ChaCha8 generator seeded with 20261006,
  VisionClaw first, then agentbox. Each prompt carries the whole topic text and
  the commit's diff, restricted to the topic's sequence-cited files in that
  repository. The harness has no subagent tool, so endpoint 4 stays PENDING
  until judges run per `judge/PROTOCOL.md`.
- **Timing and byte identity.** These follow E0 amendment #2. Timing goes to
  `TIMING.md`. Byte identity covers `results.json`, `RESULTS.md` and every
  file under `judge/` except verdicts.
- **What drives the remaining flags (exploratory, not an endpoint).** For each
  T_flow flag set by a symbol, the change is classified as follows: signature
  changed, calls added or removed, calls reordered only, resolution kind changed
  only, flow changed while `sig_hash` and `body_hash` are both unchanged
  (resolution drift caused by edits elsewhere), or symbol present on one side
  only.

### 2026-10-06 #2: what "eligible" counts in the power rule (written before the first E0c run, no number seen)

"Fewer than 20 eligible pairs → underpowered" is read as the **pooled
population** of eligible pairs: flagged by T_file^seq and not T_flow, and
after the stamp, summed over VisionClaw and agentbox. It is not read as the
drawn sample size. The drawn size is reported beside it. If the population is
under 20, endpoint 4 is reported as UNDERPOWERED (not passed), and E0c does not
hold, because it needs endpoint 4 to succeed. This verdict is written whether
or not any pair is judged.

### 2026-10-06 #3: a per-commit driver table (written after the first run, endpoint numbers seen)

The first run showed 134 of VisionClaw's 309 symbol-change events were
`one_sided`. To show where those come from, the exploratory driver section
gained a table of the commits with the most symbol-change events, with their
subjects. The table is post hoc and exploratory. It changes no count, endpoint
or verdict, and it was added after R_flow = 1.53 (CI 1.32–1.87) and
R_flow(.rs) = 2.05 (CI 1.59–2.94) had been seen.
