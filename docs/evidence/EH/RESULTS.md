# EH results: hybrid upkeep (batching plus cheap triage)

Pre-registered in `PREREG.md` (commit 03a588e, before any E0d label existed). Amendment #A1 was written before any triage output was seen; #A2 was written after it and changes nothing scored. Triage ran 2026-10-06 10:35–10:54 UTC: 180 calls, 0 transport failures, 0 retries, 0 unparsable answers.

## Verdict

- **Only Sonnet 5.5 meets endpoint 1** (recall 0.93, 27 of 29, Wilson 95% 0.78–0.98). GLM-5.3-Flash reaches 0.86 (25/29, 0.69–0.95) and Haiku 4.5 reaches 0.38 (11/29, 0.23–0.56); both fail.
- **Decision, as registered: Sonnet 5.5 is the triage layer.** It is the only model that qualifies, so the rejection-per-cost tiebreak does not come into play.
- **Caveat (#A2):** the gold labels came from `claude -p --model sonnet`, which resolves to `claude-sonnet-5-5`. Sonnet's recall is therefore partly agreement with itself. Its lower bound (0.78) also sits below 0.90; the registration states the threshold on the point estimate.
- **Projected saving:** about 118 expensive re-authors over the E0 window, against 1943 per-commit re-checks (−94%) and 229 weekly-batched re-checks (−48%). Batching alone does most of the work.

## Part 1: batching (E0 window, T_file flags)

A re-check is one topic re-checked once in a window in which T_file flagged it at least once.

| Window | VisionClaw re-checks | ×fewer | agentbox re-checks | ×fewer | Pooled |
|---|---|---|---|---|---|
| per commit (baseline) | 719 (100 windows) | 1 | 1224 (100) | 1 | 1943 |
| UTC day | 362 (21 days) | 1.99 | 164 (6 days) | 7.46 | 526 |
| ISO week | 162 (6 weeks) | 4.44 | 67 (2 weeks) | 18.27 | 229 |
| whole window | 36 | 19.97 | 37 | 33.08 | 73 |

The baselines equal E0d's endpoint-4 population totals (719, 1224), which cross-checks the computation. agentbox's 100 commits fall in 6 days, so batching collapses its flags much further than VisionClaw's.

## Part 2: triage on E0d's 60 labelled pairs (29 stale)

| Model | recall (Wilson 95%) | rejection rate (Wilson 95%) | precision (Wilson 95%) | missed stale | endpoint 1 |
|---|---|---|---|---|---|
| GLM-5.3-Flash | 0.86 (0.69–0.95) | 0.58 (0.46–0.70) | 1.00 (0.87–1) | ab-02, vc-14, vc-15, vc-23 | **FAIL** |
| Haiku 4.5 | 0.38 (0.23–0.56) | 0.82 (0.70–0.89) | 1.00 (0.74–1) | 18 pairs | **FAIL** |
| Sonnet 5.5 | 0.93 (0.78–0.98) | 0.48 (0.36–0.61) | 0.87 (0.71–0.95) | vc-09, vc-14 | **PASS** |

The gold stale rate is 0.48, so a perfect triage rejects 52%. Sonnet rejects 48% and says yes to 4 pairs the gold calls fresh (vc-02, vc-10, vc-12, vc-16). Haiku and GLM never say yes to a fresh pair, but they miss stale ones. The registered "yes when unsure" instruction did not make Haiku cautious. vc-14 is missed by all three models.

By stratum, recall / rejection rate:

| Model | VisionClaw (n 45, stale 25) | agentbox (n 15, stale 4) |
|---|---|---|
| GLM-5.3-Flash | 0.88 / 0.51 | 0.75 / 0.80 |
| Haiku 4.5 | 0.36 / 0.80 | 0.50 / 0.87 |
| Sonnet 5.5 | 0.92 / 0.40 | 1.00 / 0.73 |

### Cost per pair (endpoint 3)

| Model | mean total tokens (input side / output) | mean wall-clock (median, max) | Claude Code reported USD, mean (sum of 60) |
|---|---|---|---|
| GLM-5.3-Flash | 16,554 (13,303 / 3,251) | 65.6 s (54.2, 253.3) | 0.115 (6.90)* |
| Haiku 4.5 | 23,757 (20,956 / 2,801) | 31.1 s (24.6, 127.8) | 0.044 (2.65) |
| Sonnet 5.5 | 22,009 (21,769 / 240) | 4.9 s (4.4, 10.3) | 0.081 (4.88) |

\*Claude Code prices GLM with its own table, so this figure is not Z.AI's bill. GLM's token counts come from a different tokeniser. Haiku and GLM spend thousands of output tokens per answer (thinking); Sonnet spends about 240. Rejection rate per 1k tokens: GLM 0.035, Haiku 0.034, Sonnet 0.022, reported only, since a single model qualified.

## Part 3: projection (expensive re-authors over the E0 window)

Expected re-authors = weekly re-checks × (1 − rejection rate). The pooled sample rejection rate is applied to both repositories. This is a **projection**: the rejection rates were measured on single-commit, post-stamp T_file pairs, and a batched topic-week carries a larger, mixed diff that may be rejected less often.

| | VisionClaw | agentbox | pooled | vs per-commit | vs weekly batching |
|---|---|---|---|---|---|
| per-commit re-checks | 719 | 1224 | 1943 | — | — |
| weekly batching alone | 162 | 67 | 229 | −88.2% | — |
| + Sonnet 5.5 triage (qualifies) | 83.7 | 34.6 | **118.3** | −93.9% | −48.3% |
| + GLM-5.3-Flash (does not qualify; for reference) | 67.5 | 27.9 | 95.4 | −95.1% | −58.3% |
| + Haiku 4.5 (does not qualify; for reference) | 29.7 | 12.3 | 42.0 | −97.8% | −81.7% |

Sensitivity (per-stratum rejection rates in place of the pooled one): Sonnet 115.1 pooled (97.2 VisionClaw, 17.9 agentbox).

Triage is not free: Sonnet makes one call per weekly re-check, 229 calls at about 22k tokens each, which saves 111 re-authors. Triage pays only if a re-author costs well over about 2× a triage call (229 / 111).

## Method and files

- `tools/batching.py`: Part 1, from `docs/evidence/E0/results.json` (writes `tools/batching.json`).
- `tools/build_triage.py`: builds each triage input from `E0d/labels/<id>.prompt.md` alone. It replaces the instruction section (from `Answer exactly one question:` to `## TOPIC (`) with the registered sentence and a two-line reply format, leaving every other byte unchanged. The 60 inputs hash to `1556696fb4e4aa49` (sha256 prefix, reproduced). The inputs are not committed; rebuild them with the script.
- `tools/run_triage.py`: one fresh `claude -p` per (model, pair) under `env -i`, prompt on stdin, empty cwd, `--tools "" --setting-sources "" --strict-mcp-config --no-session-persistence --disable-slash-commands --output-format json`, at most 6 at once. Z.AI's key is read fresh from `.env` and is never stored; the files were checked for it.
- `tools/score.py`: parsing, scoring and the projection; it is the only step that reads labels.
- `runs/<model>/<id>.json`: every raw answer with usage, cost and wall-clock.
- `results.json`: all numbers, per-pair rows included.
