# ES2 pre-registration: sequence-first rewrite with a fidelity gate

Registered 2026-10-06, after ES (d160292) and before any ES2 output exists.
ES showed a GLM-5.3-Flash sequence-only rewrite kept review recall (R − A = +0.022)
but invented facts in 10 of 20 topics. ES2 tests whether a different-family
fidelity gate fixes that without losing recall.

## Material
- Corpus: EK's sealmap copy at 44fef36 (20 topics), identical to ES.
- Code for checking: the sealmap tree at 44fef36.
- Gold, lens, reviewer (Gemini 3.8 Flash, high, temperature 0), k = 3, scorer
  method and comparator runs (EK arm A, EK arm S, ES arm R): unchanged from ES.

## Procedure
1. **Rewrite.** GLM-5.3-Flash via Claude Code on Z.AI, default (max) effort, run
   under `env -i` with only the variables it needs, rewrites each non-sequence
   diagram into a sequence diagram with the same author prompt as ES. Prose is
   not edited.
2. **Gate.** For every rewritten diagram, a fresh Claude Opus subagent (different
   family from the author) compares it with the original diagram and the cited
   code at 44fef36 and returns pass / fail with each unsupported or changed fact.
3. **One repair.** A failing diagram goes back to GLM once, with the gate's list.
   The repair is re-gated by a new Claude subagent. If it still fails, the
   original non-sequence diagram is restored (the topic stays mixed).
4. The result (arm R2) must pass the corpus gate `--check --render`.

## Endpoints
1. **Recall:** R2 − A ≥ −0.05 (bootstrap 95% CI reported, seed 20261006).
2. **Fidelity (primary):** independent fidelity judges find invented or changed
   facts in **at most 2 of 20 topics**. Judges are gpt-6-astra via the Codex CLI
   at high effort, read-only sandbox, in a snapshot of the tree at 44fef36: a
   third family, not the author's or the gate's. Same judge prompt as ES.
3. **Conversion:** at least **70%** of non-sequence diagrams end as sequence
   diagrams (not restored). Below that, the result describes a mostly mixed
   corpus and does not count as sequence-first.
4. Reported, no threshold: WRONG findings per run; gate pass rates before and
   after repair; GLM and gate tokens and wall-clock.

## Decision
ES2 holds only if endpoints 1, 2 and 3 all pass. Then sequence-first authoring
with a different-family fidelity gate becomes the recommended corpus style in the
diagrams-as-code skill. Otherwise mixed corpora stay.

## Amendments
None.
