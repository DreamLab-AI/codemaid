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
