# ES pre-registration: sequence-first rewrite, authored by GLM-5.3-Flash

Registered 2026-10-06, before any rewrite was produced. Owner hypothesis, as
tested fairly: EK *deleted* non-sequence diagrams; ES asks whether a corpus
**rewritten** so that sequence diagrams carry the same facts (decisions as
`alt`/`opt`/`loop`/`break` blocks, structure as participants and notes) reviews
as well as the full mixed corpus.

## Material

- sealmap's authored corpus frozen at 28c62d8, exactly the copy EK reviewed
  (`docs/evidence/EK`), and EK's frozen gold (45 Tension/Debt/Drift/Open rows,
  hash recorded in EK's `gold-hashes.txt`).
- Arm **R** (new): the rewrite. Arms **A** and **S** are EK's existing runs,
  reused unchanged as comparators (same corpus, lens, reviewer and gold).

## Authoring (arm R)

- Author: **GLM-5.3-Flash through Claude Code on Z.AI** (the zai harness), at
  the model's default maximum reasoning effort (no thinking budget set; probed
  2026-10-06 that a budget only lowers effort), output cap 128K.
- Brief, fixed: rewrite every non-`sequenceDiagram` block in every topic as one
  or more `sequenceDiagram` blocks that carry the same facts and the same
  `path:line` citations; keep every narrative, register marker and invariant
  verbatim; add nothing not in the original topic. The author sees the corpus
  and the source at 28c62d8.
- Gate before review: the rewritten corpus must pass the corpus generator's
  `--check --render` (structure and mmdc). Citations are carried over, not
  re-derived; a rewrite that drops or invents a citation is counted (below),
  not repaired.
- One rewrite, no iteration on review results.

## Review and scoring

- Reviewer, lens, k and scoring exactly as EK: Gemini 3.8 Flash high effort,
  critical lens, k = 3, register-stripped pack, one blind Sonnet scorer per run
  against EK's gold, same rules.

## Endpoints

1. **Primary:** recall(R) − recall(A), mean over k, bootstrap 95% CI (seed
   20261006). Non-inferiority margin −0.05, as EK.
2. recall(R) − recall(S): does rewriting recover what deletion lost?
3. Fidelity of the rewrite, judged blind by a Claude subagent per topic
   against the original topic: facts dropped, facts invented, citations dropped
   or added. Reported as counts.
4. WRONG findings per run, as EK.

**Holds if** endpoint 1's CI lower bound is ≥ −0.05 **and** endpoint 3 shows
no invented facts in more than 2 of 16 topics. Otherwise mixed corpora stay.

## Known limits

Single corpus (sealmap), n = 16 topics, one author run, k = 3 at temperature 0
(narrow CIs, run noise only). A pass here licenses a larger test, not a
corpus-wide rewrite.

## Amendments

None.
