# ES2 results: sequence-first rewrite with a different-family fidelity gate

Run on 2026-10-06 under `PREREG.md` (fa3efd5). The amendments are in `AMENDMENTS.md`. ES2-1 to ES2-4
were written before any output existed. ES2-5 covers a key rotation that aborted the first author
session before it had edited anything. ES2-6 and ES2-7 were written after output was seen and change
nothing in the result. Machine-readable results are in `results.json` and `gate/final.json`.

## Decision

**Does not hold. Mixed corpora stay.**

| Endpoint | Result | Threshold | Verdict |
|---|---|---|---|
| 1. Recall, R2 − A | **−0.030 [−0.067, +0.011]** | ≥ −0.05 | point estimate passes; CI lower bound fails (ES's rule) |
| 2. Fidelity (primary) | invented or changed facts in **7 of 20 topics** (15 facts) | ≤ 2 topics | **fails** |
| 3. Conversion | **64 of 77** non-sequence diagrams end as sequence diagrams (**83.1%**) | ≥ 70% | passes |
| 4. WRONG per run | R2 0.67 · A 0.00 · S 1.67 · R (ES) 0.00 | reported | — |

The decision needs all of 1, 2 and 3. Endpoint 2 fails on its own, so the reading of endpoint 1
(ES2-7) does not matter.

The gate worked as a filter. ES's ungated rewrite had inventions in 10 of 20 topics; ES2 has them in
7. That is still far from ≤ 2. The gate and the judges disagree on one point that the template states
for both: whether an order imposed on an unordered fan-out counts. Fourteen of the 15 inventions are
an imposed order or concurrency, and 14 of the 15 sit in diagrams the Claude gate had **passed** in
round 1. Several
gate verdicts say outright that they read a sequence's order as "the diagram's form" (COR-04.3,
DEN-01.1, EXT-06.6, MOD-03.5). gpt-6-astra counted the same orderings as invented.

## Recall (sealmap, gold = 45 rows, EK's frozen list)

| Arm | Runs | Mean [95% CI] | Pack tokens | WRONG/run | NOVEL/run (REAL / PARTLY) |
|---|---|---|---|---|---|
| **R2, gated GLM rewrite** | 0.167, 0.167, 0.211 | **0.181** [0.167, 0.211] | 66,255 | **0.67** | 7.33 (1.33 / 5.33) |
| A, mixed original (EK) | 0.200, 0.233, 0.200 | 0.211 [0.200, 0.233] | 64,708 | 0.00 | 6.67 |
| S, sequence by deletion (EK) | 0.100, 0.122, 0.133 | 0.119 [0.100, 0.133] | 32,138 | 1.67 | 10.67 |
| R, ungated GLM rewrite (ES) | 0.178, 0.267, 0.256 | 0.233 [0.178, 0.267] | 65,775 | 0.00 | 5.67 |

Exploratory contrasts, not endpoints: R2 − S = +0.063 [+0.044, +0.078]; R2 − R = −0.052 [−0.100,
−0.011]. The bootstrap is ES's (EK Amendment 5, single stratum, 10,000 resamples, mulberry32(20261006),
3 replicate slots drawn with replacement and applied to both arms). With k = 3 the CI shows run noise
only. R and R2 are different GLM rewrites, so R2 − R mixes rewrite-to-rewrite variance with any effect
of the gate.

Per R2 run (FULL / PARTIAL / REG / NOVEL / REAL / PARTLY / WRONG): rc310a5 6/3/8/7/1/4/2, r805245
7/1/7/8/1/7/0, r8ab75c 9/1/8/7/2/5/0. Every run returned STOP with 15 parsed findings, and each
scorer classified all 15. The counts come from the score files, which `analyse-es2.cjs` deduplicates
by gold id. Two scorers' one-line replies counted slightly differently from their own files.

## Endpoint 2: what the judges found

There were 20 gpt-6-astra judges (Codex CLI 0.160.0, `model_reasoning_effort="high"`, `--sandbox
read-only`, cwd = a read-only 44fef36 snapshot), one per topic, all blind. They used ES's judge prompt,
with the ES2-3 harness changes. They checked R2 against the original topics.

| Topic | Dropped | Invented | Where the inventions are |
|---|---|---|---|
| COR-01 | 1 | 1 | COR-01.5 (round-1 pass): file and parent links put in an order |
| COR-03 | 1 | 1 | COR-03.5 (round-1 pass): prefix `NAME` for `NAME/` |
| COR-04 | 2 | 3 | COR-04.3 (round-1 pass): order imposed on the front-matter outputs |
| DEL-01 | 0 | 1 | DEL-01.3 (round-1 pass): order imposed on determinism mechanisms |
| DEL-02 | 0 | 1 | DEL-02.6 (repaired): `par` asserts concurrency |
| DEN-01 | 1 | 3 | DEN-01.1 (round-1 pass): orders on independent branches |
| MER-01 | 1 | 5 | MER-01.4 (round-1 pass): order imposed on independent writer rules |
| EXT-06, MOD-02, MOD-03 | 4, 5, 6 | 0 | — |
| other 10 topics | 0 | 0 | — |

Totals: 15 facts invented in 7 topics and 21 dropped in 8 topics. One citation was added
(`structure.rs:178`, inside the original's `178-190`) and one dropped (`hash.rs:89`, MOD-02.1). The
judges flagged no restored diagram. Post-hoc sensitivity, reported only to show the failure is not
borderline: setting aside every invention that is purely an imposed order or concurrency leaves 1 topic
(COR-03), which would pass. So the outcome depends on whether a sequence diagram's order counts as a
claim. The registered judge prompt says it does ("An explicit claim of a call, order ... that the
original does not make is invented"), and the gate template says the same. In practice the Claude gate
applied that rule less strictly than gpt-6-astra.

The mechanical check (`mech-fidelity.json`) found prose verbatim in 20 of 20 topics, no headings added,
and 106 blocks before and after, 13 of them non-sequence (the restored ones).

## Gate and repair

| Stage | Units | Pass | Rate |
|---|---|---|---|
| Round 1 (GLM rewrite, fresh Opus gate per diagram) | 77 | 41 | **53.2%** |
| Round 2 (one GLM repair of the 36 failures, new Opus gate) | 36 | 23 | **63.9%** |
| Final: rewritten diagrams kept | 77 | 64 | 83.1% |

The 13 diagrams restored to their original form are COR-02.5, COR-03.4, DEN-01.6, EXT-01.2, EXT-02.1,
EXT-02.4, EXT-03.7, EXT-04.5, EXT-05.6, EXT-06.1, MER-01.2, MOD-01.1 and MOD-01.4. Five of the 13
failed in round 2 because the repair replaced an imposed order with a `par` block. That claims
concurrency, which the gate also counted as unsupported.

Round-1 failures fell into four groups:
- class or type relations drawn as messages;
- exclusive outcomes drawn as all happening;
- orders imposed on fan-outs;
- mis-attributed senders or labels, for example a reversed call direction in MER-01.2 and reversed
  dependency arrows in DEL-01.4.

The repair touched exactly the 36 failing diagrams and added no sections. R2 passed the corpus gate:
`diagram-index-gen.cjs docs/diagrams --check --render` gave **exit 0**, with 20 topics, 106 diagrams,
107 register markers (unchanged) and 106/106 rendered (`gate.log`).

## Costs

| Stage | Wall-clock | Usage |
|---|---|---|
| GLM author (one run) | 33.0 min (10:22:26–10:55:25Z), 136 turns | 245,875 in · 4,059,648 cache-read · 111,198 out |
| GLM repair (one run) | 19.8 min (10:59:50–11:19:40Z), 120 turns | 156,735 in · 6,627,264 cache-read · 70,419 out |
| GLM aborted session (ES2-5, discarded) | 15.3 min, 29 turns, no edits | 97,508 in · 629,696 cache-read · 50,884 out |
| Gate round 1, 77 Opus subagents | under 4.5 min elapsed (between author end and repair start); 1,685 s summed | 4,604,174 tokens |
| Gate round 2, 36 Opus subagents | about 3 min elapsed; 701 s summed | 2,147,115 tokens |
| Gemini reviews, 3 runs | 2.9 min (161–174 s each) | 66,720 prompt · 17.0k–18.4k thinking · 3.8k–3.9k out per run |
| Fidelity judges, 20 Codex runs | 3.5 min (concurrency 5); 919 s summed | 525,036 in (240,256 cached) · 26,254 out (23,265 reasoning) |
| Scorers, 3 Sonnet subagents | about 35 s; 94 s summed | 211,135 tokens |

Z.AI reports no separate thinking count for GLM, and no price is applied. Per-agent rows are in
`agent-usage.tsv`.

## Procedure as run

1. **Rewrite.** GLM-5.3-Flash ran through Claude Code on Z.AI under `env -i` (ES2-1), with a fresh HOME
   and default maximum effort. It used ES's brief plus one scratch-file rule (`author-brief.md`) and
   rewrote 77 blocks into 77 sequence diagrams with no sections added. Chromium's socket-path limit
   forced it to render with `TMPDIR=/tmp/mmdc-tmp` (ES2-6).
2. **Gate.** Each rewritten diagram went to one fresh Claude Opus subagent with only
   `gate-template.md` (ES2-2), the original and rewritten topics and the read-only 44fef36 source.
3. **Repair.** One fresh GLM session (`repair-brief.md`) received the 36 failing diagrams with the
   gate's findings verbatim. Every repaired block went to a new Opus subagent. `finalise.cjs` kept 23
   and restored 13 originals.
4. **Review.** The pack is `external-review.cjs R2/corpus/docs/diagrams --register strip --lens
   critical --dry-run`. That command reproduces EK's arm-A pack byte for byte (sha256 83a8bb35…). The
   R2 pack is 216,706 bytes and 66,255 tokens, sha256 7535b834…. It has no register leak: the 4 extra
   keyword hits over A are "drift state" and "ex*tension*". ES's `run.cjs` ran unchanged: Gemini 3.8
   Flash, high, temperature 0, critical lens, k = 3. All three runs were concurrent and none retried.
   **Neither known script bug triggered (ES2-4)**: every response arrived in under 300 s, and
   `parseFindings` filled every field, so neither the `node:https` re-send nor the tolerant parser was
   used.
5. **Scoring.** One blind Sonnet subagent per run used ES's scorer template unchanged against EK's
   gold (sha256 04141c43…, unchanged) and EK's code copy. Run ids are opaque (`mapping.json`).
6. **Fidelity.** One gpt-6-astra judge per topic, as above. Codex's read-only sandbox cannot start a
   shell here (bwrap under no-new-privileges, ES2-3). The topic texts were therefore appended to the
   unchanged ES prompt, and the JSON came back as the final message. No judge attempted a command,
   and none needed a retry.

## Threats to validity

1. **Gate and judge calibration differ.** The gate and the judges were told the same rule about order,
   but the Opus gates often waived it for fan-outs and the gpt-6-astra judges did not. A gate prompt
   that forbids every unstated order, `par` included, might have closed most of the gap. That is a
   hypothesis for a new registration, not something this run shows.
2. **The CI shows run noise only.** There is a single corpus, a single rewrite and k = 3. Recall fell
   from ES's R (0.233) to R2 (0.181) on a different GLM rewrite, so this run cannot attribute the drop
   to the gate.
3. **There is one judge per topic** and no inter-rater check. The judges compare against the original
   topic, not the code.
4. **The scorers differ from EK's** (EK threat 5), as in ES.
5. **The author saw the render gate** (as in ES). The repair saw the fidelity gate's findings, as the
   PREREG intends.

## Files

| Path | Contents |
|---|---|
| `PREREG.md`, `AMENDMENTS.md` | registration and ES2-1 to ES2-7 |
| `author-brief.md`, `repair-brief.md`, `run-glm.sh`, `logs/author.*`, `logs/repair.*`, `transcripts/` | GLM sessions |
| `aborted/` | the key-rotation session (ES2-5): logs only |
| `rewrite1/`, `repaired/`, `R2/corpus/` | corpus after the rewrite, after the repair, and final |
| `gate-template.md`, `units.cjs`, `mkgate.cjs`, `finalise.cjs`, `gate/` | gate prompts, verdicts per round, `final.json` |
| `gate.log`, `mech-fidelity.json` | corpus gate and mechanical check of R2 |
| `R2/base/`, `tokens.jsonl`, `run.cjs`, `run-https.cjs`, `runall.sh`, `queue.tsv`, `logs/status.log`, `runs/` | pack and reviews |
| `blind.cjs`, `blind/`, `mapping.json`, `scorer-prompts/`, `scores/` | blind scoring |
| `judge.cjs`, `fidelity/` (`prompts/`, `raw/`, `judge-manifest.json`, one JSON per topic) | endpoint-2 judges |
| `agent-usage.tsv`, `analyse-es2.cjs`, `results.json` | costs, endpoints and decision |
