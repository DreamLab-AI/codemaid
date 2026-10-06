# ER pre-registration: review screen, corpus pack vs full source

Registered 2026-10-06, before any case was selected. Design from the external
consultation (`docs/evidence/consult/2026-10-06-codex.md`, round 2 §B), with
the owner's selection and adjudication replaced by models at the owner's
request. **Question: does the owner's actual workflow (a diagrams-only corpus
blob, reviewed in one shot) produce enough additional, confirmed, actionable
findings over reviewing the full source to justify building review-pack
tooling next?** Amendments are dated and appended below.

## Cases (selected by a Claude agent, oracle-checked)

Up to three historical fixes on sealmap `main`, each satisfying all of:

- its fix lands after `0d7c752` (the first authored corpus), so a corpus exists
  at the fix's parent P;
- a regression test added by the fix or immediately before it **fails at P for
  the defect's reason and passes at the fix**, confirmed by running it; tests
  that need more than trivial adaptation to run at P are rejected;
- preferably three separate episodes (distinct defects, distinct commits).

Selection stops after 60 minutes of agent wall-clock. If fewer than two cases
qualify, ER is reported as **not runnable** (an evidence-availability outcome,
not evidence against corpus value), and the decision rule's "otherwise" branch
applies.

Defects already disclosed by the corpus prose or register at P are recorded and
excluded from hidden-defect scoring.

## Arms (each case, once each, fresh calls)

- **A, the owner's workflow:** the authored corpus at P (`docs/diagrams` topic
  files), register markers stripped by `sealmap-review`'s `--register strip`;
  no source.
- **B, full source:** every first-party `.rs` file, `Cargo.toml` and test file
  at P, plus the corpus's `README.md` for orientation only.

Reviewer for both: Gemini 3.8 Flash, high thinking, temperature 0, through the
same script; prompt = the `critical` lens with the output limit changed from 15
to **three ranked, actionable findings**, each with location, triggering
condition and consequence. The fix diff, commit message and added test are never
shown.

## Adjudication (models, blind to arm)

All findings for a case are pooled, deduplicated and arm labels hidden. A fresh
Claude subagent with the repository checked out at P validates each finding
against code and tests: **confirmed** (real, actionable, reproducible or
evidenced at file:line), **unsupported** (wrong or unevidenced), or **unsure**.
Every "unsure", and a random 25% of the rest, goes to a second adjudicator of
another family (gpt-6-astra, Codex CLI, high reasoning); disagreements resolve
to the more conservative verdict (unsupported beats unsure beats confirmed).
Each finding is also checked for whether it identifies the case's hidden defect.
Adjudication cost (tokens, wall-clock) is recorded in place of owner triage
minutes.

## Decision rule (fixed now)

If arm A produces **at least two more confirmed findings than arm B, summed over
the cases, while detecting at least as many hidden defects as B and costing no
more adjudication**, build the smallest review-pack improvement that removes
observed work next. **Otherwise**, build the hunk-overlap shadow checker next
and defer further review-pack and sequence-flow features. The existing corpus
workflow stays available either way.

## Stated limits

Three cases cannot show general superiority. sealmap is small enough for full
source to fit, which removes the corpus's scale advantage, so a B win here does
not transfer to campaignbuilder-sized repositories; an A win is the
informative outcome.

## Amendments

None.

### 2026-10-06 #T1 (team lead): second reviewer, gpt-6-luna (written before any review output existed)

At the owner's request, every case and arm is also reviewed by **gpt-6-luna,
high reasoning, through the Codex CLI**, with the identical pack and lens. It is
an additional, pre-specified reviewer factor, not a replacement:

- The decision rule above is evaluated **for Gemini as registered** (primary)
  and reported separately **for Luna** (secondary). If they disagree, the
  primary governs and the disagreement is reported.
- Luna runs from an **empty working directory with a read-only sandbox**, so it
  can read nothing but the pack (arm A must stay diagrams-only). The pack is
  passed on stdin, never as a command-line argument.
- Luna's findings are pooled and adjudicated blind with Gemini's. Because Luna
  shares a vendor with the gpt-6-astra second adjudicator, Luna-originated
  findings that need a second opinion go to a **second independent Claude
  subagent** instead.
- Also reported per reviewer: wall-clock and tokens per review.

### 2026-10-06 #T2 (team lead): Luna at extra-high reasoning (written before any review output existed)

At the owner's request, amendment #T1's reviewer runs at **extra-high
reasoning** (`model_reasoning_effort = "xhigh"`, accepted by gpt-6-luna in a
probe) instead of high. Nothing else in #T1 changes.

### 2026-10-06 #E1–#E10 (ER agent; all written at 08:15 UTC, before any review request was sent and before any review output existed)

Base text: this copy was taken from `main` at `f5ddacd` (the 1c7b1a1 registration plus #T1, #T2). The main working tree is off limits to the ER agent, so these amendments are written in `er/PREREG.md` for the team lead to commit.

- **#E1 Case selection rule.** All five eligible candidates passed the oracle (fail at P for the defect's reason, pass at the fix): fec7aff, ae478d9, d227dd8, 613b9ba, 926144c (6c8a9b0 predates 0d7c752, ineligible). The corpus at P discloses d227dd8's three defects (REGISTER DB-41 `Self::f`, DB-42 match guards, O-12 struct-literal receiver; DEN-01.6) and 613b9ba's (T-04, by-name guess unique across the workspace). Since a disclosed defect cannot score as hidden, the three undisclosed ones are selected: **gen** = fec7aff, **var** = ae478d9, **prefix** = 926144c. They are three distinct defects in three distinct commits and episodes (triage-found; label bug found reading a generated diagram; DEN-01-era reading of `sealmap pack`).
- **#E2 Review snapshot for gen.** fec7aff's parent 1667cc4 is the test-only commit that adds the regression tests; the registration forbids showing the added test. Both arms and the adjudicators therefore use **28c62d8** (1667cc4 minus that test). The tests, transplanted, fail at 28c62d8 exactly as at 1667cc4 (2 of 3 fail, same assertions) and pass at fec7aff. For var and prefix the snapshot is P itself (6e0604c, dd88e1f); the added tests are not present there.
- **#E3 Lens.** The script loads lenses only from its own `assets/lenses`, so the lens copy `er/lenses/critical-3.md` (count 15→3, nothing else changed) is byte-identical to `loadLens('critical', 3)`, and the baked script is run unmodified as `--lens critical --count 3 --register strip`. The same lens text goes to arm B, including its "diagrams-only / you do not have the source" preamble (literal reading of "same lens"; recorded as a threat). The lens's existing Topics/Evidence/Failure shape is kept as the "location, triggering condition, consequence" fields.
- **#E4 Arm B pack.** The script cannot pack source. `er/b-review.cjs` builds it with the script's `=== FILE: rel ===` format and sends it with the script's request parameters (countTokens, then generateContent, pack and lens as two parts of one user turn, gemini-3.8-flash, thinkingLevel high, temperature 0), reusing the script's `stripRegister`, `loadLens` and `parseFindings`. Files: `docs/diagrams/README.md` first (register-stripped, as `--register strip` would), then every git-tracked file outside `docs/` that is `*.rs`, a `Cargo.toml`, or under a `tests/` directory (golden fixtures included), sorted.
- **#E5 Luna (#T1/#T2) mechanics.** Prompt = the same pack text, a blank line, then the same lens text, on stdin: `codex exec --json --skip-git-repo-check --model gpt-6-luna -c model_reasoning_effort='"xhigh"' --sandbox read-only -C <empty dir> -`. Findings parsed with the script's `parseFindings`. Any command Luna executes is logged from the JSON events and reported (a read-only sandbox does not stop reads outside cwd).
- **#E6 Deduplication.** Per case, the orchestrator merges two findings only when they name the same defect at the same code location; the mapping is recorded. A merged item credits every arm and reviewer that produced it. Pooled items are shuffled with a mulberry32 PRNG seeded 20261006 and relabelled X-01…; origins are hidden from all adjudicators.
- **#E7 Second-opinion routing.** Every "unsure" plus a seeded 25% (ceil) sample of the rest, sampled with the same PRNG after shuffling. An item with a Gemini origin goes to gpt-6-astra; an item with a Luna origin goes to a second independent Claude subagent; an item with both goes to both. Resolution: most conservative verdict wins (unsupported > unsure > confirmed). Final "unsure" counts as not confirmed.
- **#E8 Hidden-defect detection.** An arm detects a case's hidden defect if any of its items is judged (by the final resolution) to identify it and is not "unsupported".
- **#E9 Adjudication cost per arm.** Primary adjudicator cost per case (tokens, wall-clock from the Agent notification) is divided equally over the case's pooled items; each second-opinion call's cost goes to its item. An arm's cost is the sum over items credited to it (a merged item is charged in full to each crediting arm).
- **#E10 Decision rule application.** Evaluated per reviewer (Gemini primary, Luna secondary) on Gemini-only and Luna-only item credits respectively; confirmed counts are summed over the three cases.

### 2026-10-06 #E11 (ER agent; written 08:25 UTC, after the arm-A and Luna review files existed on disk but before any review text was read; only exit codes, wall-clock and the gen-A finding count had been seen)

All three Gemini arm-B calls failed with `fetch failed` at 305–307 s: Node 22's built-in fetch (undici) has a fixed 300 s headers timeout that fires before the script's own 600 s abort signal, and a 150–200k-token full-source pack with high thinking takes longer than 300 s to answer. No response was received, so no arm-B review exists yet. Arm B is re-sent once per case with the identical URL, headers, body (pack, lens, model, thinkingLevel high, temperature 0) and retry policy, over `node:https` with a 900 s timeout (`er/b-review.cjs`, `post()`); nothing the model sees changes. Arm A ran through the unmodified script and is not re-run. Recorded as a defect in the baked script for large packs.

### 2026-10-06 #E12 (ER agent; written 08:33 UTC, AFTER all twelve review outputs were read, before any adjudication)

- **Field extraction.** The script's `parseFindings` returned null Evidence/Failure for all three prefix-B-gemini findings, whose reply wrote `- **Evidence**:` instead of `- Evidence:`. The finding counts (3 per review, 36 in all) are unaffected. The blind adjudication text for all 36 is taken from each raw `critical.md` with a tolerant extractor (`er/pool.cjs`, `blocks()`), carrying title, Evidence, Failure and the reviewer's confidence. Topics and "Marked by authors" are left out because topic ids would reveal the arm.
- **Deduplication as applied (rule #E6).** gen: no merges (12 items). var: A-gemini F-01 with A-luna F-01 (closure parameters untyped, `collect.rs:1342`), and A-gemini F-03 with A-luna F-03 (uncatchable stack overflow, `isolate.rs`), giving 10 items. prefix: A-luna F-01 with B-luna F-02 (seal lock records unauthenticated, `seal/lock.rs`), giving 11 items. Same-defect findings in *different* cases are not merged, since cases are scored separately.
- **No change to any verdict rule, routing or the decision rule.**

### 2026-10-06 #E13 (ER agent; written 08:39 UTC, AFTER primary and second adjudications were seen, before scoring)

- **Second-opinion rubric wording (deviation, disclosed).** The primary adjudicator prompts defined "unsupported" as "wrong, already handled, not present at this revision, or not evidenced". The gpt-6-astra and second-Claude prompts, written after the primary verdicts had been read, added "**or deliberately documented as intended**". Primary adjudicators had already applied "actionable" this way (gen X-02, X-07, X-10; var X-04, X-05, X-09; prefix X-04, X-05, X-11), but the wording is not identical. Under the conservative merge it flipped three primary "confirmed" verdicts to unsupported: gen X-04 (checkout-name default), var X-06 (stack-overflow abort, also register DB-07 at P) and prefix X-02 (module body hash order-sensitive). RESULTS reports the registered conservative merge as primary and a primary-only sensitivity alongside it. The decision outcome is reported under both.
- **Tool use by adjudicators.** Each astra run made one RuVector `memory_search` (namespace `tasks`), which returned one unrelated entry (a 2026-08 report review). No adjudicator ran git commands against other revisions. Every worktree was clean afterwards.
