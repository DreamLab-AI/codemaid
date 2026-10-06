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

The "None." above was true at registration. The entries below are appended,
never edited.

### 2026-10-06 #A1: making the registration concrete (written before any triage output was seen)

Written 2026-10-06T10:35:22Z, after two-line smoke tests ("Reply with exactly: stale: no") on each model and before any triage input reached any model. No endpoint, threshold or decision rule changes.

**Part 1.** Dates are E0's `commits[].date` (which carry UTC offsets), converted to UTC. A UTC day and an ISO week (ISO 8601 year-week of the UTC date) are calendar buckets; "the whole window" is all 100 eligible commits. Re-checks in a window = the number of distinct topics flagged by T_file on any commit in it, summed over windows. The per-commit baseline is ΣT_file (719 VisionClaw, 1224 agentbox), which equals E0d's endpoint-4 population totals, a cross-check.

**Triage input.** `build_triage.py` reads only `E0d/labels/<id>.prompt.md`. The instruction section is the text from `Answer exactly one question:` up to the `## TOPIC (` heading (the question, the JSON reply format and the `statements` note). It is replaced by the registered sentence verbatim, followed by the reply format the registration names (`stale: yes|no` and a one-line `reason:`). The header, the context paragraph that says what TOPIC and DIFF are (and that the topic was verified at or before the parent) and the TOPIC and DIFF are byte-identical to the labeller's. 60 inputs, concatenated sha256 prefix `1556696fb4e4aa49`, reproduced on a second build.

**Invocation.** One fresh `claude -p` (Claude Code 2.1.289) per (model, pair); triage input on stdin; cwd an empty directory; flags `--tools "" --setting-sources "" --strict-mcp-config --no-session-persistence --disable-slash-commands --output-format json`. `--output-format json` only wraps the reply with usage and timing; it does not change the model input. No `--system-prompt` is passed: the team lead's specified command has none (E0d's `run-label.sh` passed a one-line "careful labeller" system prompt, a difference recorded here). Model ids: `claude-haiku-4-5`, `claude-sonnet-5-5`, and `glm-5.3-flash` on Z.AI. Environment via `env -i`: PATH, HOME, LANG/LC_CTYPE=C.UTF-8; Claude models authenticate through HOME's Claude Code credentials; GLM adds ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic, ANTHROPIC_AUTH_TOKEN from `ZAI_ANTHROPIC_API_KEY` (else `ZAI_API_KEY`) read fresh from agentbox/.env, ANTHROPIC_API_KEY empty. Default effort for every model. **Temperature is not settable** through Claude Code's CLI, so it is each endpoint's default. At most 6 calls run at once (models run one after another, 6 workers each).

**Once per pair.** A call that fails at the transport level (non-zero exit, no JSON, `is_error`, no result text) is not a model answer; it is retried up to twice and every failed attempt is recorded. A returned answer is never re-asked.

**Parsing.** The reply is lower-cased and markdown emphasis/backticks stripped. The verdict is the first match of `stale\s*[:=]\s*"?(yes|no)\b`; failing that, a reply whose first word is exactly `yes` or `no`. Anything else is **unparsable and counts as yes** (registered "yes when unsure"), and is listed.

**Scoring.** Gold stale = both E0d labels `yes` (29/60). Recall = triage-yes among stale / 29, Wilson 95% (z = 1.96). Rejection rate = triage-no / 60. Precision = stale among triage-yes. Cost: input, cache and output tokens, Claude Code's reported `total_cost_usd` (for GLM this is Claude Code's price table applied to a non-Anthropic model and is not Z.AI's bill; tokens are the comparable unit) and wall-clock per call. "Rejection rate per unit cost", for the decision among qualifiers, is rejection rate / mean total tokens per pair (input + cache read + cache creation + output); USD is shown beside it.

**Part 3.** Expected expensive re-authors = weekly re-checks × (1 − rejection rate), per repository and pooled, beside the per-commit baseline; the pooled sample rejection rate is applied to both repositories (no per-stratum rate, n = 15 for agentbox being too small), and per-stratum rates are shown as a sensitivity line only.

### 2026-10-06 #A2: an observation on the gold (written after the triage output was seen)

This changes no endpoint, threshold, parse rule or the decision; it is a caveat on reading endpoint 1. E0d's labellers ran `claude -p --model sonnet` (#L1). On the Claude Code that ran EH (2.1.289) the `sonnet` alias resolves to `claude-sonnet-5-5`, checked after the run with the same flags (`modelUsage` key). E0d's labelling ran the same day, so the gold was very probably written by the same model as the Sonnet 5.5 triage arm. Sonnet 5.5's recall is therefore partly agreement with itself, and is likely biased upwards against an independent gold. Haiku 4.5 and GLM-5.3-Flash share no such link. The other differences from the labellers' runs remain: a different question, a two-line reply in place of JSON, no system prompt, and one call in place of two.
