# EH pre-registration: hybrid upkeep (batching plus cheap triage)

Registered 2026-10-06, after E0d's harness and label prompts were committed
(29fdd1d) and **before any E0d label existed**. If E0d fails, the remaining
question is not a smarter detector but fewer and cheaper re-checks: per-file
flags (full recall by construction), batched per topic over a window, then a cheap
model triaging each batched topic before an expensive re-author.

## Part 1: batching (deterministic, already computable)
From E0's per-commit `file` flag lists (`docs/evidence/E0/results.json`), report
the number of re-checks when each topic is re-checked once per window, for
windows of one commit (baseline), one UTC day, one ISO week and the whole window,
for VisionClaw and agentbox. No threshold: this is a measured property of the
commit history, recorded here so its definition is fixed.

## Part 2: cheap triage on E0d's labels
- **Gold:** E0d's blind labels (both labellers yes = stale), unchanged.
- **Population:** E0d's 60 labelled pairs. All are T_file flags, so triage sits
  exactly where the hybrid would put it.
- **Triage models** (each run once per pair, temperature 0 where settable):
  GLM-5.3-Flash via Claude Code on Z.AI (clean `env -i`, default effort);
  Claude Haiku 4.5; Claude Sonnet 5.5.
- **Triage input:** the E0d labeller prompt for that pair, with its instructions
  replaced by: "Answer stale: yes|no: would this change make any statement or
  diagram in this topic wrong? Answer yes when unsure." and a one-line reason.
  Same pair content as the labellers saw; no detector output.
- **Endpoints, per model:**
  1. **Recall vs gold ≥ 0.90** (Wilson 95% interval reported). The primary
     endpoint: a triage layer that drops real staleness is worse than none.
  2. **Rejection rate:** the share of pairs answered "no" (work saved), reported
     with precision.
  3. Cost: tokens and wall-clock per pair.
- **Decision:** a model qualifies as the triage layer if it meets endpoint 1. Of
  the qualifying models, recommend the one with the highest rejection rate per
  unit cost. If none qualifies, the hybrid is batching alone.

## Part 3: projected saving
For each qualifying model, report expected expensive re-authors over the E0
window = weekly-batched re-checks × (1 − rejection rate), against the
per-commit baseline. Labelled a projection: rejection rates are measured on
post-stamp T_file pairs, which may differ from batched topic-weeks.

## Amendments
None.
