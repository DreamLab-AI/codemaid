# E0 pre-registration: deterministic maintenance, no LLM

Registered 2026-10-06, before any harness code was written or any endpoint
computed. Changes after this commit are amendments: dated, appended below, and
never applied silently. The question is the one in DESIGN §9 E0: **does
symbol-level staleness flag materially fewer topics per commit than today's
file-level staleness, without hiding real changes?**

## Inputs (pinned)

| Input | Revision |
|---|---|
| Corpus: VisionFlow `docs/diagrams`, areas `visionclaw/` (35 topics) and `agentbox/` (34 topics) | `cace76a09db2c98732a41cd8d798b9ef0a1f74e2` |
| VisionClaw (`../project` in `sources:`) | `9c8dcdc92d7ac50644f336bbd4e156f161219be0` |
| agentbox (`../project/agentbox`) | `b487c95fa9caf7096a3284f91138935a7f3c3531` |
| sealmap | the 0.2.0 crates (`v0.2.0`), default extraction options |

## Commit window

Per repository, walk first-parent history back from the pinned head. A commit
is **eligible** if it is not a merge and changes at least one file named in
some topic's `sources:` for that repository. Take the **100 most recent
eligible commits**. Commits that touch no cited file are counted (reported
separately) but excluded from the endpoints, since both methods flag zero
topics for them by construction.

## Mapping citations to symbols

Each topic is read once, at the pinned corpus revision. Every `path:line` (and
range start) in the topic is resolved at that topic's `verified_commit` to the
**innermost sealmap symbol whose span contains the line**. A citation is a
**fallback citation** if its file is not Rust, the line is inside no symbol, or
the file does not parse at the stamp; a fallback citation tracks its whole file.
A cited file with no line citation at all is also file-level.

## Per-commit counts (for commit C with parent P)

- **T_file:** topics with at least one `sources:` file changed between P and C.
- **T_sym:** topics where a cited symbol's `sig_hash` or `body_hash` differs
  between P and C, or the symbol exists at exactly one of P and C, or a
  fallback citation's file changed.
- **T_hop:** T_sym, plus topics where a one-hop callee (sealmap call edge,
  exact or inferred) of a cited symbol changed.
- A cited symbol absent at both P and C, with its file changed, counts as
  changed (conservative). Its share is reported.

## Endpoints

1. **Primary:** the reduction ratio R = ΣT_file / ΣT_sym over the 100 eligible
   VisionClaw commits. **Success: R ≥ 2.0.** The bootstrap 95% CI (10,000
   resamples of commits, seed 20261006) is reported alongside.
2. **Secondary, per repository and pooled:** median and p90 of T_file, T_sym and
   T_hop; R for T_hop; the same for agentbox (no success threshold, since most of
   its sources are not Rust).
3. **Coverage honesty:** the share of citations, and of topics, that are fully
   or partly fallback, per repository. Reported, not gated.
4. **Hidden-change precision:** from the (commit, topic) pairs flagged by
   T_file but not T_sym, draw 40 (seed 20261006, stratified 30 VisionClaw / 10
   agentbox, or all if fewer). A judge reads the topic text and the commit's
   diff restricted to the topic's sources and answers one question: *does this
   change make any statement in the topic false or incomplete?* The judge is a
   Claude subagent instructed per the protocol, with each "yes" re-checked by a
   second, independent judge. **Success: hidden-real-change rate ≤ 10%**, Wilson
   95% upper bound reported.

E0 **holds** only if endpoints 1 and 4 both succeed. If it holds, `sealmap-ts`
moves from deferred to scheduled. If R < 2.0, or the hidden rate exceeds 10%, it
stays deferred and the result is published as found.

## Reproducibility

The harness lives in this repository under `bench/e0/` (a `publish = false`
workspace member). Its output, `docs/evidence/E0/results.json` plus a
`RESULTS.md`, is generated from the pinned inputs and is byte-identical across
runs, except for the judge verdicts, which are stored with their prompts.

## Amendments

None.
