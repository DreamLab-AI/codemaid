# ES results: sequence-first rewrite authored by GLM-5.3-Flash

Run on 2026-10-06 under `PREREG.md` (c525d16), with two amendments dated before the rewrite
existed (`AMENDMENTS.md`: ES-1 the corpus revision, ES-2 the harness environment). Machine-readable
results are in `results.json`.

## Decision

**Does not hold. Mixed corpora stay.**

- **Endpoint 1 (primary) passes.** recall(R) − recall(A) = **+0.022 [−0.022, +0.056]**, and the CI
  lower bound is above the −0.05 margin. The rewritten corpus reviewed at least as well as the mixed
  original.
- **Endpoint 3 fails.** The blind fidelity judges found invented facts in **10 of 20 topics**,
  against an allowance of 2. The rewrite also dropped facts in 10 topics.

The review result is the interesting half. Rewriting recovered everything deletion lost: R − S =
**+0.115 [+0.078, +0.144]**, and R had no WRONG findings. But the rewrite that achieved this is not
faithful enough to be the corpus: 14 of the 77 rewritten diagrams now say something the
code-derived original did not. The PREREG makes faithfulness a condition, so the rule fails.

## Recall (sealmap, gold = 45 rows, EK's frozen list)

| Arm | Runs | Mean [95% CI] | Pack tokens | WRONG per run | NOVEL per run (REAL / PARTLY) |
|---|---|---|---|---|---|
| **R, GLM rewrite, all sequence** | 0.178, 0.267, 0.256 | **0.233** [0.178, 0.267] | 65,775 | **0.00** | 5.67 (1.33 / 4.33) |
| A, mixed original (EK) | 0.200, 0.233, 0.200 | 0.211 [0.200, 0.233] | 64,708 | 0.00 | 6.67 (0.00 / 6.67) |
| S, sequence only by deletion (EK) | 0.100, 0.122, 0.133 | 0.119 [0.100, 0.133] | 32,138 | 1.67 | 10.67 (0.33 / 8.67) |

| Endpoint | Estimate [95% CI] | Reading |
|---|---|---|
| 1. R − A (margin −0.05) | **+0.022** [−0.022, +0.056] | non-inferior; the CI lower bound is above the margin |
| 2. R − S | **+0.115** [+0.078, +0.144] | the rewrite recovers what deletion lost |
| 3. Fidelity | invented facts in **10/20** topics (17 facts); dropped facts in 10/20 topics (18 facts); citations: 0 dropped, 0 added in substance | **fails** (allowed ≤ 2 topics) |
| 4. WRONG per run | R 0.00 · A 0.00 · S 1.67 | the rewrite did not induce speculation |

The bootstrap follows EK Amendment 5 with sealmap as the single stratum: 10,000 resamples with
mulberry32(20261006), each drawing 3 replicate slots with replacement and applying them to both
arms. With k = 3 the CI shows only run-to-run noise. R's runs spread more than A's (0.178 to 0.267),
so one weak run and two strong ones produced the positive point estimate.

Per R run (FULL / PARTIAL / REG / NOVEL / REAL / PARTLY / WRONG): rfcf635 7/2/8/7/2/5/0, r32977d
12/0/10/5/0/5/0, re70ed0 11/1/10/5/2/3/0. Every run returned STOP with 15 parsed findings, and the
scorer classified all 15 in each.

## Endpoint 3: what the rewrite got wrong

Twenty fresh, blind Claude subagents each compared one original topic with its rewrite
(`fidelity-template.md`). They checked 1,077 facts in all. All 20 topics kept their prose byte for
byte (judged, and confirmed mechanically). The rewrite added no headings: 77 non-sequence blocks
became 77 sequence blocks.

| Topic | Rewritten | Facts checked | Dropped | Invented |
|---|---|---|---|---|
| COR-01 | 3 | 33 | 0 | 0 |
| COR-02 | 4 | 42 | 0 | 0 |
| COR-03 | 4 | 49 | 1 | 0 |
| COR-04 | 4 | 84 | 0 | **2** |
| COR-05 | 2 | 19 | 0 | 0 |
| COR-06 | 2 | 32 | 1 | **1** |
| DEL-01 | 3 | 75 | 0 | **1** |
| DEL-02 | 6 | 87 | 1 | **1** |
| DEN-01 | 4 | 76 | 0 | 0 |
| EXT-01 | 3 | 36 | 1 | **1** |
| EXT-02 | 4 | 49 | 0 | **2** |
| EXT-03 | 6 | 81 | 0 | 0 |
| EXT-04 | 3 | 29 | 1 | 0 |
| EXT-05 | 4 | 68 | 2 | **4** |
| EXT-06 | 5 | 55 | 2 | **1** |
| MER-01 | 4 | 66 | 1 | 0 |
| MER-02 | 4 | 28 | 0 | **1** |
| MOD-01 | 4 | 62 | 3 | 0 |
| MOD-02 | 4 | 52 | 5 | **3** |
| MOD-03 | 4 | 54 | 0 | 0 |

The "Rewritten" column sums to 77, matching the mechanical count.

The inventions are mostly changes of meaning, not new subjects. Examples:
- **COR-04.5** words the ancestor test the wrong way round.
- **COR-06.1** emits shard output and then exits 1, where the original has two exclusive outcomes.
- **EXT-05.4** gives `by_name_only` the lookup-miss outcome and says non-internal derived calls are
  "dropped", although the topic says they become `sym:?`.
- **EXT-02.4** routes the kept-paren-comma case into "same text either way", against "(T,) and (T)
  stay distinct".
- **DEL-01.3** makes the codebase-name leak conditional ("opt name pinned"), where the original
  shows it as unconditional.

Some inventions come from the format itself, and a strict reader might not call them inventions. A
class diagram has no call order, so drawing one as messages invents an order (EXT-06.1). Class
associations become calls (MOD-02.1). An unconditional step gets wrapped in `opt` (MER-02.1). Even
with every topic whose inventions are only of that kind set aside, 7 topics remain, which still
fails the ≤ 2 rule. That sensitivity is post hoc and is reported only to show the failure is not
borderline.

**Citations.** The mechanical check (`mech-fidelity.cjs`, `mech-fidelity.json`) compared every
`path:line` inside the mermaid fences. It found no substantive change. In EXT-03.5, four citations
lost their crate prefix (`sealmap-extract/src/raw.rs:137` became `raw.rs:137`; same file, same
lines), which the EXT-03 judge counted as 4 citations dropped. In MOD-02, one citation
(`symbol.rs:244`) moved from the topic's prose into a diagram participant, so it is not new to the
topic. No citation was invented.

## Authoring (arm R)

- **Author:** GLM-5.3-Flash through Claude Code on Z.AI (`claude -p --bare --strict-mcp-config
  --dangerously-skip-permissions --tools Read,Edit,Write,Glob,Grep,Bash --model glm-5.3-flash`). It
  ran under `env -i` with a private HOME (ES-2) and no thinking budget, at the default maximum
  effort. `CLAUDE_CODE_MAX_OUTPUT_TOKENS` was 128000.
- **Brief:** `author-brief.md`, the PREREG brief verbatim plus operating rules (edit only mermaid
  fences, no renumbering, run the gate). One run, with no iteration on review results.
- **Cost:** **32.3 min wall-clock** (09:01:40–09:33:56Z), 146 turns: 43 Bash, 23 Read and 78 Edit
  calls, with one context compaction. Usage from `modelUsage`: 264,044 input tokens, 9,489,408
  cache-read tokens and 119,654 output tokens. Z.AI reports no separate thinking count. No price is
  applied.
- **Gate:** the author ran the gate itself and reported it green. I re-ran
  `diagram-index-gen.cjs docs/diagrams --check --render` independently: **exit 0**, 20 topics, 106
  diagrams, 107 register markers (unchanged from the original), 106/106 rendered by mmdc, none over
  4500 px (`gate.log`). Nothing outside the 20 topic files changed. The author did write scratch
  `.mmd` probes to `/tmp/mmdtest`, outside its directory, to test Mermaid syntax.
- **Pack:** `external-review.cjs R/corpus/docs/diagrams --register strip --lens critical --dry-run`,
  the same command EK used. The same script reproduces EK's arm-A pack byte for byte
  (sha256 83a8bb35…). The R pack is 214,662 bytes and 65,775 tokens, +1.6% on A. Its sha256 is
  8a78cfb9…. A keyword scan found the same 13 Tension/Debt/Drift/TODO mentions in R as in A, so no
  register leaked.

## Review and scoring (as EK)

- **Reviews:** 3 runs of EK's `run.cjs`, unchanged (Gemini 3.8 Flash, thinkingLevel high,
  temperature 0, critical lens with COUNT = 15), all concurrent. Started 09:35:20Z, ended 09:37:24Z.
  Each run took 117–123 s, with 66,240 prompt tokens, 0 cached, 9.3k–11.7k thinking and 3.3k–3.9k
  output. There were no errors and no retries. The key was loaded into the one runner's environment
  only, as in EK's `runall.sh`.
- **Scoring:** one fresh, blind Sonnet subagent per run, using EK's `scorer-prompt.md` unchanged
  against EK's gold (sha256 04141c43…, unchanged) and EK's code copy. Run ids are opaque. The
  scorers took 207,860 tokens and 192 s in all.
- **Fidelity judges:** 20 Claude subagents took 1,267,620 tokens and 1,041 s summed. Usage per agent
  is in `agent-usage.tsv`.

## Threats to validity

1. **Single author run, single corpus, k = 3.** The R − A estimate rests on 3 runs, and the R runs
   spread widely. "Non-inferior" here licenses at most a larger test. The PREREG says so too.
2. **The judges were told that a change of meaning counts as invention** (`fidelity-template.md`).
   That is a reasonable reading of "add nothing not in the original topic", but it is stricter than
   "new subject matter". The post-hoc sensitivity above shows the verdict does not depend on it.
3. **One judge per topic, no inter-rater check.** The judges checked against the original topic,
   not the code. A rewrite fact the original does not state, but that is true of the code, still
   counts as invented by design.
4. **The scorers differ from EK's.** A and S were scored by EK's Sonnet instances on the same day
   with the same prompt. Scorer variance is mixed into the R − A contrast (EK threat 5).
5. **The author saw the gate.** The brief gave GLM the structure-and-render gate command (ES-2), so
   gate failures were fixed before review. This is allowed, because the PREREG forbids only
   iteration on review results. It means the gate does not measure first-pass validity.
6. **Corpus revision (ES-1).** The rewrite is of EK's actual corpus (44fef36, 20 topics), not of
   28c62d8 (16 topics). The PREREG names 28c62d8, but its comparators only exist at 44fef36.

## Files

| Path | Contents |
|---|---|
| `PREREG.md`, `AMENDMENTS.md` | registration (unchanged) and ES-1/ES-2 |
| `author-brief.md`, `run-author.sh`, `author.json`, `author-transcript.jsonl.gz`, `author.start`/`.end` | the authoring run |
| `R/corpus/docs/diagrams/` | the rewritten corpus (frontmatter still says verified_commit 4ed7a51…/6cefaf4…, as in the original) |
| `gate.log`, `mech-fidelity.cjs`, `mech-fidelity.json` | gate re-run and mechanical citation/prose check |
| `R/base/` (`pack.txt`, `manifest.json`), `tokens.jsonl` | the register-stripped R pack and its token count |
| `run.cjs`, `runall.sh`, `queue.tsv`, `logs/`, `runs/<id>/` | EK's runner and the 3 reviews |
| `mapping.json`, `blind/`, `scorer-template.md`, `scorer-prompts/`, `scores/` | blind scoring |
| `fidelity-template.md`, `fidelity-prompts/`, `fidelity/` | endpoint 3 judges |
| `agent-usage.tsv` | tokens and seconds per Claude subagent |
| `analyse-es.cjs`, `results.json` | endpoints and decision |
