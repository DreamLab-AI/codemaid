# ER results: review screen, corpus pack (A) vs full source (B)

Run on 2026-10-06 under `PREREG.md` (registered at 1c7b1a1, plus #T1 and #T2 from `main` at f5ddacd, plus #E1–#E13 written by the ER agent; the copy in this directory is the amended one). Machine-readable results are in `results.json`.

## Decision

**"Otherwise" branch: build the hunk-overlap shadow checker next and defer further review-pack and sequence-flow features.** The corpus workflow stays available.

- **Gemini (primary, as registered):** A had 4 confirmed findings and B had 5, so A did not reach B + 2. Hidden defects detected: 0 and 0, so A met "at least as many". A's adjudication cost was 297k tokens against B's 487k, so A met "no more". The rule fails on its first condition.
- **Luna (secondary, #T1):** A had 3 confirmed and B had 7. Hidden defects: 0 and 0. A's cost was 272k tokens against B's 82k. The rule fails on the first and third conditions. Luna agrees with Gemini.
- **Sensitivity (#E13), primary adjudicator only, without the second-opinion rubric wording:** Gemini A 6 vs B 6; Luna A 4 vs B 7. The outcome is unchanged.

Neither arm found any of the three hidden defects, under either reviewer.

## Cases and oracle evidence

Selection ran from 08:11 to 08:15 UTC, within the 60-minute budget. All five eligible candidates passed the oracle. Full output is in `oracle/ORACLE.md` and `oracle/*.log`, and the script is `oracle/run.sh`. The method: transplant the fix's test file into the snapshot, then run `cargo test -p sealmap-rust --test <file>` with a private `CARGO_HOME` and `CARGO_TARGET_DIR`.

| Case | Fix | Snapshot reviewed | Fails at P (assertion) | Passes at fix | Disclosed by corpus at P | Selected |
|---|---|---|---|---|---|---|
| gen | fec7aff (generic params by name shape) | 28c62d8 (#E2; P=1667cc4 is the test-only commit) | 2/3 fail: `left: []` vs `[A#bump(), geom/V2#normalise()]`; "sync(). bound a generic parameter to a concrete type: [Store#put(), Store#open()]" | 3/3 | no (only `docs/review/…-gemini-triage.md:48`, outside the corpus) | yes |
| var | ae478d9 (variant labels print field attrs) | 6e0604c | `left: Found "{ #[doc = \" Where.\"] #[serde(skip)] file: String…"` vs `"{ file: String, line: u32, }"` | 1/1 | no | yes |
| prefix | 926144c (leading path segment captured by same-named fn) | dd88e1f | `left: [extern …::audit::record (Inferred), …]` vs `[audit/record() Exact, audit/trail/flush() Exact]` | 1/1 | no (EXT-05 describes the fallback, not the defect) | yes |
| den | d227dd8 (`Self::`, match guards, struct-literal receivers) | 34b1296 | 4/5 fail, `left: []` | 5/5 | **yes**: DB-41, DB-42, O-12 | no (#E1) |
| reach | 613b9ba (name guesses across unreachable crates) | d227dd8 | 1/3 fail, missing `Entry#reconcile()` Inferred | 3/3 | **yes**: T-04 | no (#E1) |

6c8a9b0 predates the corpus (0d7c752) and was ineligible. The three selected cases are distinct defects from distinct commits and episodes. Each case's own regression test was absent from both packs (grep-checked). Packs at later snapshots contain earlier cases' tests, which is expected.

## Per-arm results

Each review returned exactly 3 findings: 36 raw findings, 33 adjudicated items after deduplication (#E12). The final verdict is the most conservative of the primary and any second opinion.

| Reviewer · arm | gen confirmed | var confirmed | prefix confirmed | **Confirmed** | Unsupported | Hidden defects | Adjudication tokens / s (#E9) | Review wall-clock / tokens (sum of 3) |
|---|---|---|---|---|---|---|---|---|
| Gemini · A (corpus) | 2/3 | 1/3 | 1/3 | **4** | 5 | 0/3 | 297,348 / 206 | 399 s; 156k prompt + 43.6k thinking |
| Gemini · B (source) | 2/3 | 1/3 | 2/3 | **5** | 4 | 0/3 | 487,468 / 261 | 1,183 s; 535k prompt + 176.8k thinking |
| Luna · A (corpus) | 1/3 | 1/3 | 1/3 | **3** | 6 | 0/3 | 271,821 / 196 | 583 s; 597k input (425k cached) + 40.3k output |
| Luna · B (source) | 3/3 | 2/3 | 2/3 | **7** | 2 | 0/3 | 81,711 / 137 | 1,091 s; 1,334k input (854k cached) + 69.9k output |

None of the 33 items was judged unsure. Primary-only confirmed counts were Gemini A 6 / B 6 and Luna A 4 / B 7.

What the confirmed findings were:

- **Gemini A:** closure-parameter shadowing that mints false Exact edges (gen; var merged with Luna A); `build`, `out` and `vendor` directories pruned silently at any depth (gen); the checkout-directory name feeding symbol ids for package-less files (prefix).
- **Gemini B:** `if let`/`while let` bindings that leak out of scope (reproduced); `constructed_type` mistyping static calls (reproduced); the README claim that `sealmap verify` checks `docs/sealmap/`, which is false; tuple and slice impls dropping their methods (reproduced); `#[cfg]` twins losing the second flow (reproduced).
- **Luna A:** `write` overwriting a non-generated file at an expected path; the closure-shadowing item merged with Gemini A; a stack-overflow abort (prefix; the var copy of the same item was overturned on second opinion).
- **Luna B:** symlink-following `write` (in all three cases; reproduced); same-name repos and crates collapsing (reproduced); `#[cfg]` twins losing their flow; stale `_model.json` left behind after `--no-model` (reproduced).

Closest to a hidden defect: gen X-07 (Gemini B) cited `is_generic_param` but argued that calls on real generics are dropped. That behaviour is documented, so the item was judged unsupported and only a **partial** match. No item scored "yes".

## Adjudication

- **Primary:** three fresh Claude subagents, one per case, blind to arm. gen took 100.5k tokens and 159 s (19 tool uses); var 97.1k tokens and 165 s; prefix 100.7k tokens and 173 s. Several verdicts rest on reproductions in temporary tests, deleted afterwards; every worktree ended clean. Each primary also reproduced its hidden defect at P, confirming it was live and findable.
- **Second opinion (#E7):** no item was unsure, so the 9 items are the seeded 25% sample (mulberry32 seeded 20261006). gpt-6-astra (high reasoning, cwd = snapshot) took 8 items across three calls, 562.6k tokens in all (423k cached) and 176 s. A second Claude subagent took the 2 Luna-origin items: 116.4k tokens and 35 s.
- **Agreement:** 6 of 10 opinion pairs agreed (60%). gpt-6-astra agreed with the primary on 5 of 8; the second Claude on 1 of 2. All four disagreements were primary "confirmed" against second "unsupported" because the behaviour is documented as intended: gen X-04 (directory-name default), var X-06 (stack-overflow abort; astra and Claude-2 both), prefix X-02 (module body hash is order-sensitive). See #E13 for the rubric-wording asymmetry behind these.
- **Total adjudication cost:** about 977k tokens and 708 s of agent wall-clock (orchestration excluded). Per-arm cost is dominated by which items the random 25% sample happened to hit. Primary-only cost is identical across arms (81.7k tokens each, since every arm contributed 9 item-credits), so the cost condition carries little signal here.

## Threats to validity

1. **n = 3 cases, 3 findings per review.** One or two verdict flips move an arm by the margin the rule tests. The rule's A + 2 condition failed by 3 (Gemini) and 6 (Luna) under the registered merge, and by 2 and 5 under primary-only.
2. **Hidden-defect detection is 0 everywhere**, so that condition does not discriminate. All three defects are narrow resolver or label bugs that a top-3 "production damage" lens is unlikely to rank. This is a ceiling-effect limit of the lens, not evidence for either arm.
3. **Lens asymmetry.** Arm B received the corpus lens verbatim, including "you do not have the source; the distillation is all you get" (#E3). Both reviewers still cited source directly.
4. **Rubric wording asymmetry (#E13).** The second-opinion prompts added "deliberately documented as intended" to "unsupported" after the primary verdicts had been read. It flipped 3 items: 2 crediting A, 1 crediting B. The decision is unchanged without it.
5. **Inconsistent verdicts across cases on the same defect.** The stack-overflow abort was confirmed in prefix (primary only, not sampled) but overturned in var (sampled). Out-of-sample items get only one adjudicator.
6. **Stale corpus.** prefix A-gemini F-01 reported `Self::` and match-guard drops from DEN-01.6 prose. d227dd8 had already fixed both at P, but the corpus at dd88e1f still describes them (and still lists DB-41/42). Register stripping did not remove it because the prose sits in a diagram section. This is a real cost of arm A: corpus drift becomes a false finding.
7. **Blinding is partial.** Topic ids and arm labels were withheld, but evidence text shows its kind (corpus prose versus quoted code), so an adjudicator could infer the arm.
8. **Transport change for Gemini B (#E11).** The baked script's fetch dies at undici's fixed 300 s headers timeout on 140–220k-token packs, before its own 600 s abort. B was re-sent over `node:https` with an identical request. This is also a defect in the baked script for any pack that takes more than 300 s.
9. **Parser gap (#E12).** The script's `parseFindings` missed bold-formatted fields (prefix-B-gemini). Adjudicators saw tolerantly extracted text; the counts were unaffected.
10. **Ambient tools.** Every Luna review tried a RuVector `memory_search` and was refused by the approval policy, so there was no contamination; no shell commands ran. Each astra run's `memory_search` succeeded but returned one unrelated entry. Adjudicators were told not to consult git history; no history commands appear in the astra logs. The Claude adjudicators' transcripts were not audited command by command.
11. **sealmap is small.** Full source fits in context (138–220k tokens), which removes the corpus's scale advantage. As the registration notes, a B win does not transfer to campaignbuilder-sized repositories.

## Amendments

All amendments are in `PREREG.md`, under "## Amendments":

- **#T1, #T2 (team lead):** add the Luna reviewer, at xhigh.
- **#E1–#E10, written before any review was sent:** selection rule and the three cases; the gen snapshot at 28c62d8; the lens copy; arm-B pack construction; Luna mechanics; deduplication; second-opinion routing; hidden-detection rule; cost attribution; per-reviewer decision.
- **#E11, written after review files existed but before any review text was read:** Gemini B transport retry.
- **#E12, written after reading reviews:** tolerant field extraction and the deduplication as applied.
- **#E13, written after adjudication, before scoring:** the rubric-wording deviation and adjudicator tool use.

## Files

- `PREREG.md` (amended)
- `oracle/`: `run.sh`, logs, `ORACLE.md`
- `lenses/critical-3.md`
- `b-review.cjs`, `luna-parse.cjs`, `pool.cjs`, `sample.cjs`, `score.cjs`, `run-reviews.sh`, `run-b-retry.sh`, `dedup.json`
- `reviews/<case>-<arm>-<reviewer>/`: pack, raw review, findings, manifest, and Luna event streams
- `adjudication/<case>/`: `blind.md`, `key.json`, `primary.json`, astra prompt and events, `claude2.json`; plus `second-selection.json` and cost files
- `results.json`

Worktrees and build directories were removed. The `reviews/` packs total about 4.7 MB; commit them or drop them as you prefer.

## Note on #T1/#T2 compliance

Every Luna review ran once only, at `model_reasoning_effort="xhigh"` (#T2). No "high" run was ever made, so nothing was discarded. Each run used an empty cwd (`er/luna-empty`), `--sandbox read-only`, and the pack plus lens on stdin via `-` (`< prompt.txt`, 154 KB–722 KB, beyond the argv limit). All six produced complete three-finding reviews, which confirms stdin works with `-`. No separate probe was run, so no amendment was needed. Luna-origin second opinions went to a second Claude subagent, never to gpt-6-astra. Per-review wall-clock and tokens for both reviewers are in the per-arm table and in `results.json` (`review_cost`).
