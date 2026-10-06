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
