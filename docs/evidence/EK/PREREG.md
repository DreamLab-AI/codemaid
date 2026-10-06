# EK pre-registration: are sequence diagrams the only kind worth keeping?

Registered 2026-10-06, before any arm was built or reviewed. Owner hypothesis:
**a corpus that keeps only `sequenceDiagram` blocks loses no review value and
costs fewer tokens.** Amendments are dated and appended below, never applied
silently.

## Corpora (pinned)

| Corpus | Revision | Frozen issue list |
|---|---|---|
| campaignbuilder `docs/diagrams` | `2af6a25288e55ead63757eb827c883197d41ab98` | `REGISTER.md` Tension + Debt + Drift entries at that sha |
| sealmap `docs/diagrams` | `44fef36` | `REGISTER.md` Tension + Debt + Drift entries at that sha |

Each issue list is hashed (sha256) and the hash committed before any review
output is read. Open and Invariant entries are excluded: the stripped-review
pilots showed Opens are invisible from diagrams, and Invariants are not defects.

## Arms

All arms start from the **register-stripped** pack (`external-review.cjs
--register strip`), so the reviewer has to find issues itself.

| Arm | Content |
|---|---|
| **A** all kinds | the stripped corpus as is (control) |
| **S** sequence only | every non-`sequenceDiagram` Mermaid block removed, together with its `## <id>.<n>` heading and its *What it shows* / *Why it is this way* paragraphs; topic narratives kept |
| **M** matched removal | the same number of bytes as S removes, taken from a random selection of diagrams of **any** kind (seed 20261006), with their headings and paragraphs; separates "fewer tokens" from "fewer of this kind" |
| **P** prose only | every diagram removed, with its heading and paragraphs; topic narratives kept. The floor: do diagrams add anything? |

## Review and scoring

- **Reviewer:** Gemini 3.8 Flash, high thinking, single shot, the baked
  `critical` lens unchanged, **k = 3** independent runs per arm per corpus
  (24 runs).
- **Scorer:** a Claude subagent per run, given the frozen issue list and the
  findings with all arm labels removed, scoring each issue FULL / PARTIAL /
  MISS and each finding REG / NOVEL / VAGUE, with NOVEL claims checked against
  the code as REAL / PARTLY / WRONG.
- **Recall:** FULL + 0.5 × PARTIAL, over the issue list, per run.

## Endpoints

1. **Primary, non-inferiority of S against A:** mean recall(S) − mean
   recall(A) over the 6 runs per arm pooled across corpora. **S is
   non-inferior if the difference is > −0.05.** A paired bootstrap 95% CI
   (10,000 resamples, seed 20261006) is reported.
2. **Kind against volume:** recall(S) − recall(M). If S ≥ M, the kind matters
   beyond token count; if S < M, sequence diagrams are not special.
3. **Value of diagrams at all:** recall(A) − recall(P).
4. **Cost:** pack tokens per arm, and recall per 100k pack tokens.
5. **Precision:** WRONG findings per run, per arm.

## Exploratory (not endpoints)

From E0's VisionClaw data: the share of each diagram kind's citations that
fall back to file level, and how often each kind's topics go stale, as a
measure of which kinds sealmap can keep honest cheaply.

## Decision rule

If S is non-inferior to A **and** S ≥ M, the diagrams-as-code skill moves to
sequence-first authoring, with other kinds allowed only where a topic
documents why a sequence cannot express the fact (state machines and data
shapes being the expected cases). Otherwise, the mixed corpus stays and the
result is published as found.

## Amendments

### 2026-10-06: execution clarifications (no arm, k, endpoint, threshold or seed changed)

This copy lives in the EK output directory. The repository copy at `fdd8207` is unchanged
because another engineer is working in that tree.

1. **Removal unit, literal reading.** In S, M and P, each removed diagram takes exactly four
   things: its `## <ID>.<n>` heading line, its mermaid block, its `**What it shows**`
   paragraph and its `**Why it is this way**` paragraph. A paragraph here is a run of
   non-blank lines, and each removed span takes one following blank line with it. Other
   paragraphs inside a diagram section are **kept**. Most of these are `**Invariant:**`
   paragraphs (49 in sealmap, about 860 in campaignbuilder); a few are extra prose or tables.
   They are kept because the PREREG names only the heading, the block and the two
   paragraphs, and because `--register strip` keeps Invariants too. Once the heading is
   gone, they sit under the preceding heading.
2. **Arms are built on the pack, not on the corpus.** Arm A's pack was produced by
   `external-review.cjs <corpus> --register strip --lens critical --dry-run`, and
   `build-arms.cjs` derived S, M and P from that pack. The script only accepts a corpus
   directory, so `run.cjs` sent every review instead. It imports the script's own
   `loadLens` and `parseFindings` and sends the identical request: contents [pack, critical
   lens with COUNT=15], `thinkingLevel: high`, `temperature: 0`, `gemini-3.8-flash`, and
   the same retry policy.
3. **How M selects diagrams.** All diagram units go through a Fisher–Yates shuffle with
   mulberry32(20261006). Units are taken in shuffled order until the removed bytes reach
   S's removed bytes, and the last unit is dropped if that lands closer. M−S is +327 bytes
   for sealmap and −1,844 bytes for campaignbuilder, each less than one unit.
4. **The gold list is the REGISTER.md table rows** under `## Tensions`, `## Debt` and
   `## Drifts` at the pinned sha. Counts are Tension/Debt/Drift: campaignbuilder 427
   (30/285/112) and sealmap 45 (4/40/1). Their sha256 hashes are in `gold-hashes.txt`,
   written at 07:06Z, before the first review was sent at 07:08Z. They were not committed
   to the repository (see the note at the top of this section).
5. **How the "paired bootstrap" is run.** Runs are independent, so pairs are formed by
   (corpus, replicate slot). Each of the 10,000 resamples (mulberry32(20261006)) draws 3
   replicate slots with replacement per corpus and applies the same slots to every arm. The
   statistic is the pooled mean over the 6 drawn runs. Corpus is a fixed stratum, so the CI
   reflects run-to-run variation only.
6. **Temperature 0 and n = 3.** The baked lens call fixes temperature 0. Runs still varied
   (thinking ran from 8.5k to 29k tokens, and findings differed), but a 3-replicate
   bootstrap understates the uncertainty.
7. **Scoring returns non-MISS issues only.** Each scorer lists every issue it scores FULL or
   PARTIAL, and every unlisted issue is a MISS. Each run had its own Sonnet subagent. Every
   scorer was blind: it saw only an opaque run id and the parsed findings fields, and the
   run order was shuffled independently of arm. Of the 24 scorers, 22 were launched
   together and 2 after the harness's 20-concurrent-agent limit freed up.
8. **Cost is reported as tokens** (prompt, cached, thinking, output) plus wall-clock time.
   No price is applied.
9. **Scorer audit (exploratory, added after scoring).** The sealmap scorers finished in
   about 30 s with 4–7 tool calls each, so their code checks were shallow. A second blind
   Sonnet agent re-checked all 12 WRONG verdicts against the code. The results are reported
   beside endpoint 5 and do not replace it.
10. **The exploratory E0 analysis was not run** (VisionClaw citation fallback and staleness
    by kind). It is out of scope for this run.
11. **Gemini's implicit prefix cache** served part of the prompt for some M and P runs
    (1.26M cached tokens in total). The service does this on its own. It affects cost, not
    the request, so runs stay independent in content.
