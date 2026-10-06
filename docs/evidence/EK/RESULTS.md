# EK results: sequence-only corpora

Run on 2026-10-06 as pre-registered (`codemaid` fdd8207). Clarifications are dated in
`PREREG.md` (§Amendments, items 1–11). No arm, k, endpoint, threshold or seed was changed.
Raw data is in `results.json`.

## Decision

**The decision rule is not met. The mixed corpus stays, and this result is published as
found.**

- **S non-inferior to A?** Only on the point estimate, and only just. S−A is −0.044, against
  a margin of −0.05, so it is "non-inferior" under the pre-registered rule. The 95% CI is
  [−0.054, −0.032] and crosses the margin, so the evidence for non-inferiority is weak.
- **S ≥ M?** No. S−M is −0.037 [−0.057, −0.019]. Removing the same bytes at random hurt
  less than removing every non-sequence diagram, so sequence diagrams are not special. The
  kinds removed in S carried review value beyond their token count.
- **A vs P?** Diagrams add a little: A−P is +0.027 [+0.003, +0.046].

The whole effect comes from sealmap. On campaignbuilder every contrast sits inside run
noise, and recall is 2–3.5% in every arm.

| Contrast | Pooled (6 v 6) [95% CI] | campaignbuilder [CI] | sealmap [CI] |
|---|---|---|---|
| S − A (primary, margin −0.05) | −0.044 [−0.054, −0.032] | +0.004 [−0.001, +0.007] | −0.093 [−0.111, −0.067] |
| S − M | −0.037 [−0.057, −0.019] | +0.007 [−0.002, +0.014] | −0.081 [−0.122, −0.044] |
| A − P | +0.027 [+0.003, +0.046] | +0.003 [−0.008, +0.012] | +0.052 [0.000, +0.089] |

The bootstrap is paired and stratified by corpus, with 10,000 resamples and seed 20261006
(Amendment 5). With k = 3, the CIs show only run-to-run variation and are too narrow to
generalise beyond these two corpora.

## Recall per arm

Recall is (FULL + 0.5·PARTIAL) / |gold|. The gold lists are campaignbuilder 427 and
sealmap 45.

| Arm | Pooled mean [95% CI] | campaignbuilder runs | sealmap runs |
|---|---|---|---|
| A, all kinds | **0.118** [0.112, 0.129] | 0.028, 0.021, 0.027 | 0.200, 0.233, 0.200 |
| S, sequence only | **0.074** [0.065, 0.081] | 0.035, 0.027, 0.026 | 0.100, 0.122, 0.133 |
| M, matched random | **0.111** [0.101, 0.122] | 0.021, 0.016, 0.028 | 0.222, 0.200, 0.178 |
| P, prose only | **0.091** [0.078, 0.110] | 0.016, 0.029, 0.022 | 0.200, 0.144, 0.133 |

On sealmap, S came out below P: 0.119 against 0.159. A pack with only the 29 sequence
diagrams found fewer register issues than a pack with no diagrams at all. One reading consistent with
this (not tested): sealmap's Debt items mostly live in flowchart and class topics (data
shapes, caps, contracts), and in S those topics lose their *What it shows* / *Why*
paragraphs while keeping their narratives, which leaves a reviewer fragments to speculate on.

## Cost (endpoint 4)

Pack tokens are from Gemini `countTokens` on the pack alone. The lens adds 465 tokens.

| Arm | campaignbuilder | sealmap | Recall per 100k pack tokens (mean of the per-corpus ratios) |
|---|---|---|---|
| A | 549,008 | 64,708 | 0.165 |
| S | 348,995 (−36%) | 32,138 (−50%) | 0.189 |
| M | 350,778 | 32,150 | 0.314 |
| P | 275,479 (−50%) | 18,969 (−71%) | 0.424 |

The recall-per-token column is dominated by sealmap's small packs. M beats S at the same
token cost, which is endpoint 2 again.

Reviewer totals over 24 runs:
- 5.03M prompt tokens, of which 1.26M came from Gemini's implicit cache (M and P runs only).
- 379k thinking tokens and 100k output tokens.
- 64.7 min of summed call time, 19 min wall-clock at 4-way concurrency, 07:08–07:27Z.
- All 24 runs returned STOP with 15 parsed findings. No errors and no retries.

Mean seconds per run: A 176, S 190, M 140, P 141. No price was applied (Amendment 8).

Scoring took 24 Sonnet subagents, about 1.97M subagent tokens and 22 min of summed
duration.

## Precision (endpoint 5)

| Arm | NOVEL per run | REAL | PARTLY | **WRONG per run (scorer)** | WRONG per run after audit |
|---|---|---|---|---|---|
| A | 6.00 | 0.17 | 5.50 | **0.33** | 0.33 |
| S | 7.50 | 0.17 | 6.50 | **0.83** | 0.83 |
| M | 6.33 | 0.33 | 5.67 | **0.33** | 0.17 |
| P | 7.17 | 0.67 | 6.00 | **0.50** | 0.33 |

Across all runs the WRONG rate is 12/360 findings (3.3%) by the scorers, and 10/360 after
the audit. S had the most WRONG findings: 5 in total, all on sealmap, and all 5 upheld by the audit.
S also had the most NOVEL findings. With less to go on, the
reviewer speculated more.

No run had a VAGUE finding. Most NOVEL findings were PARTLY: real, but overstated or a
documented decision read as a defect, as in the pilots.

**Audit (exploratory, Amendment 9).** A second blind Sonnet agent re-checked all 12 WRONG
verdicts and agreed with 10. It overturned w3 and w9 (one P run, one M run) to REAL: both
point at the stale "not published yet" sentence in
`docs/diagrams/delivery/01-ci-msrv-and-determinism.md:38-40`, which the first scorers said
was absent.

## Threats to validity

1. **campaignbuilder sits at the floor.** 15 findings against 427 gold items caps recall at
   about 0.035 even with perfect matching, and every arm landed in 0.016–0.035. Its
   contrasts (±0.007) are inside run-to-run spread, so the pooled result is effectively a
   sealmap result. The 45-item sealmap list carries the signal. Pooling the per-run means
   weights it about 8 times, by scale.
2. **n = 3 at temperature 0.** The CIs come from 3 replicate slots per corpus. They describe
   the service's residual non-determinism, not variance across corpora or reviewers.
   Treat the primary endpoint's borderline result (−0.044 against −0.05) as unresolved
   rather than as a pass.
3. **The removal unit is literal (Amendment 1).** Invariant paragraphs and stray prose in
   diagram sections were kept in S, M and P. Removing whole sections would have taken more
   from S and P and would likely have widened both gaps.
4. **Scorer depth.** The sealmap scorers spent about 30 s and 4–7 tool calls each, so
   REAL/PARTLY/WRONG verdicts rest on shallow code reads. The audit overturned 2 of 12
   WRONG verdicts. Usage per scorer is in `scorer-usage.tsv`.
5. **Single scorer per run, no inter-rater check on recall.** FULL versus PARTIAL is a
   judgement call. Different Sonnet instances scored replicates of the same arm, so scorer
   variance is mixed into the replicate variance.
6. **Arm leakage through content.** Findings cite diagram ids and diagram content that only
   exist in some arms, so a scorer could in principle infer an arm. The scorers never saw
   labels, filenames or pack hints, and a keyword scan of the findings found no mention of
   missing diagrams.
7. **Gold hashes were not committed to the repository.** They were written to
   `gold-hashes.txt` at 07:06Z, before the first review was sent at 07:08Z, because the
   repository tree was in use by another engineer.

## Files

| File | Contents |
|---|---|
| `PREREG.md` | copy of the pre-registration with dated amendments |
| `gold-hashes.txt`, `<corpus>/gold.json`, `extract-gold.cjs` | frozen issue lists and their hashes |
| `<corpus>/base/` | `external-review.cjs --dry-run` output (arm A pack and manifest) |
| `<corpus>/arms/pack-{A,S,M,P}.txt`, `arms.json`, `build-arms.cjs` | the arms; M's selected ids, byte counts and sha256 per pack |
| `tokens.jsonl` | countTokens per arm |
| `run.cjs`, `runall.sh`, `logs/` | reviewer runner and run log |
| `runs/<id>/` | critical.md, findings.json and manifest.json per run |
| `mapping.json` | opaque id → corpus, arm and replicate (kept out of scorer inputs) |
| `blind/`, `scorer-prompt.md`, `scorer-prompts/`, `scores/`, `scorer-usage.tsv` | blind scoring |
| `audit-wrong-*.json` | the WRONG-verdict audit |
| `analyse.cjs`, `results.json` | endpoints, bootstrap, per run, per arm, per corpus and pooled |
