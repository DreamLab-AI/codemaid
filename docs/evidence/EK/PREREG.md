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

None.
