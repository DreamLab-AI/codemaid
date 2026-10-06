# E0 pre-registration: deterministic maintenance, no LLM

Registered 2026-10-06, before any harness code was written or any endpoint
computed. Changes after this commit are amendments: dated, appended below, and
never applied silently. The question is the one in DESIGN §9 E0: **does
symbol-level staleness flag materially fewer topics per commit than today's
file-level staleness, without hiding real changes?**

## Inputs (pinned)

| Input | Revision |
|---|---|
| Corpus: VisionFlow `docs/diagrams`, areas `visionclaw/` (35 topics) and `agentbox/` (34 topics) | `cace76a09db2c98732a41cd8d798b9ef0a1f74e2` |
| VisionClaw (`../project` in `sources:`) | `9c8dcdc92d7ac50644f336bbd4e156f161219be0` |
| agentbox (`../project/agentbox`) | `b487c95fa9caf7096a3284f91138935a7f3c3531` |
| sealmap | the 0.2.0 crates (`v0.2.0`), default extraction options |

## Commit window

Per repository, walk first-parent history back from the pinned head. A commit
is **eligible** if it is not a merge and changes at least one file named in
some topic's `sources:` for that repository. Take the **100 most recent
eligible commits**. Commits that touch no cited file are counted (reported
separately) but excluded from the endpoints, since both methods flag zero
topics for them by construction.

## Mapping citations to symbols

Each topic is read once, at the pinned corpus revision. Every `path:line` (and
range start) in the topic is resolved at that topic's `verified_commit` to the
**innermost sealmap symbol whose span contains the line**. A citation is a
**fallback citation** if its file is not Rust, the line is inside no symbol, or
the file does not parse at the stamp; a fallback citation tracks its whole file.
A cited file with no line citation at all is also file-level.

## Per-commit counts (for commit C with parent P)

- **T_file:** topics with at least one `sources:` file changed between P and C.
- **T_sym:** topics where a cited symbol's `sig_hash` or `body_hash` differs
  between P and C, or the symbol exists at exactly one of P and C, or a
  fallback citation's file changed.
- **T_hop:** T_sym, plus topics where a one-hop callee (sealmap call edge,
  exact or inferred) of a cited symbol changed.
- A cited symbol absent at both P and C, with its file changed, counts as
  changed (conservative). Its share is reported.

## Endpoints

1. **Primary:** the reduction ratio R = ΣT_file / ΣT_sym over the 100 eligible
   VisionClaw commits. **Success: R ≥ 2.0.** The bootstrap 95% CI (10,000
   resamples of commits, seed 20261006) is reported alongside.
2. **Secondary, per repository and pooled:** median and p90 of T_file, T_sym and
   T_hop; R for T_hop; the same for agentbox (no success threshold, since most of
   its sources are not Rust).
3. **Coverage honesty:** the share of citations, and of topics, that are fully
   or partly fallback, per repository. Reported, not gated.
4. **Hidden-change precision:** from the (commit, topic) pairs flagged by
   T_file but not T_sym, draw 40 (seed 20261006, stratified 30 VisionClaw / 10
   agentbox, or all if fewer). A judge reads the topic text and the commit's
   diff restricted to the topic's sources and answers one question: *does this
   change make any statement in the topic false or incomplete?* The judge is a
   Claude subagent instructed per the protocol, with each "yes" re-checked by a
   second, independent judge. **Success: hidden-real-change rate ≤ 10%**, Wilson
   95% upper bound reported.

E0 **holds** only if endpoints 1 and 4 both succeed. If it holds, `sealmap-ts`
moves from deferred to scheduled. If R < 2.0, or the hidden rate exceeds 10%, it
stays deferred and the result is published as found.

## Reproducibility

The harness lives in this repository under `bench/e0/` (a `publish = false`
workspace member). Its output, `docs/evidence/E0/results.json` plus a
`RESULTS.md`, is generated from the pinned inputs and is byte-identical across
runs, except for the judge verdicts, which are stored with their prompts.

## Amendments

None.

The "None." above was true at registration. The entries below were written
while building the harness, **before the first endpoint run**; each records how
an ambiguous or impractical point was made concrete, and none changes an
endpoint, a threshold, the window, the seed or a mapping rule.

### 2026-10-06 #1: which citations are read

"Every `path:line` in the topic" is read with the corpus generator's own
citation rules (`scripts/diagram-index-gen.cjs` at the corpus pin): citations
inside mermaid blocks under an H2, bare `:N` bound to the same line's path, then
to the message's participant, then to the last path in the diagram; `:a,b,c`
expanded; a range contributes its start; host:port skipped; the path resolved
against `sources:` exact-first, then by suffix. Topic files are diagrams-only,
so prose citations are rare; their number is reported, and they are not read.
A citation that resolves to no single `sources:` entry (absent or ambiguous)
cannot be tied to a file and is excluded from both methods; its count is
reported.

### 2026-10-06 #2: timing is outside the byte-identical output

Wall-clock timing cannot be byte-identical across runs, so it is written to
`TIMING.md`, not to `results.json` or `RESULTS.md`. The byte-identity check
covers `results.json`, `RESULTS.md` and every file under `judge/` except
verdicts.

### 2026-10-06 #3: repository attribution

A `sources:` path belongs to the repository chosen by the generator's
`REPO_PREFIXES` (longest prefix: `../project/agentbox/` is agentbox,
`../project/` VisionClaw; agentbox is a submodule of VisionClaw, and a
VisionClaw commit that only moves the gitlink touches no source). A topic is
counted in a repository's T_file/T_sym/T_hop if it has at least one source
there, so a cross-repository topic is counted in each. Its stamp in a repository
is the generator's `shaFor` (a plain sha applies to the area's own repository
only; a `{repo: sha}` map applies by key). Sources in other repositories
(VisionFlow, unmute, loom) are outside both windows.

### 2026-10-06 #4: citations with no usable stamp

A Rust citation whose topic declares no sha for that repository, or whose sha
names no commit there, cannot be resolved "at that topic's `verified_commit`".
It is a fallback citation (reasons `no_stamp`, `stamp_unknown`), the nearest
case to "the file does not parse at the stamp". A file absent at the stamp is
the same (`absent_at_stamp`). "Does not parse" means sealmap tagged the file's
module `parse_error`.

### 2026-10-06 #5: "innermost symbol"

The innermost symbol is the one with the smallest line span containing the
line, ties to the smallest id. sealmap gives every parsed file a root module
spanning the whole file, so a line outside every item (a top-level `use`, an
`impl` header, an excluded `#[cfg(test)]` module) maps to that module, whose
body hash folds in every item in it. This is the literal rule; such citations
are reported separately as *symbol (module)*. With the default options, files
under `tests/`, `examples/` and `benches/` are not extracted, so their lines are
inside no symbol (`no_symbol`).

### 2026-10-06 #6: one-hop callees

A cited symbol's one-hop callees are the targets of its `calls` relations with
confidence exact or inferred, taken from P and from C (union). A callee changed
if its `sig_hash` or `body_hash` differs or it exists at exactly one side.
Changed files are `git diff-tree --no-renames` between P and C (a rename is a
delete plus an add).

### 2026-10-06 #7: statistics

Median is the middle value (mean of the two middle values for an even count);
p90 is nearest rank (rank ⌈0.9n⌉). The bootstrap is the percentile method, the
250th and 9,750th of 10,000 sorted resampled ratios, ChaCha8
(`rand_chacha` 0.9, `seed_from_u64(20261006)`), indices drawn as `u32`; a
resample with ΣT_sym = 0 is +∞ and sorts last. The same bootstrap is reported,
ungated, for R (hop), agentbox and pooled. Coverage classes: a topic is
*symbol only* when every in-repository source is cited and every citation maps
to a symbol, *fully fallback* when no citation maps to a symbol, *partly*
otherwise.

### 2026-10-06 #8: drawing and judging endpoint 4

The pairs of each stratum are listed in window order (newest commit first),
then topic file order; one ChaCha8 generator seeded 20261006 draws VisionClaw
first, then agentbox, by a partial Fisher–Yates shuffle. A pair is a hidden real
change when the first judge and the independent second judge both answer yes.
The judge also records whether a yes rests only on moved `path:line` anchors.
That field is reported and not scored. The harness context has no subagent
tool, so it writes the sample and the prompts and stops; endpoint 4 stays
PENDING until judges run per `judge/PROTOCOL.md` and `e0 score` is run.

### 2026-10-06 #9: prose citations are read (written after the first run)

Amendment #1 called prose citations rare and left them unread. The first run
counted 338 `path:line` citations in prose outside the mermaid blocks (the
topics' **Invariant**, **Debt** and **Open** notes, on 155 lines), 2.9% of all
citations, which is not rare. The registered text says "every `path:line` …
in the topic", so the prose is read as well: the same parser runs over each H2
section's prose lines (outside every fence, headings excluded) as one more
block. Bare `:N` refs there bind to a path on the same line, then to the last
path earlier in that section. This replaces #1's scope, and #1 is not edited.
Because this choice was made after a result had been seen, the mermaid-only
reading is reported beside it as a sensitivity row, not as an endpoint. The
first run gave VisionClaw R = 1.19, CI 1.12–1.28, under the mermaid-only
reading.

### 2026-10-06 #10: an exploratory Rust-only row (written after the first run)

The first run showed that 289 of VisionClaw's 604 T_sym flags have no symbol
reason: they are set only by a changed file that is tracked whole, either
because a non-Rust citation fell back or because no citation names the source.
To show what symbol gating does on the code sealmap-rust can read, the report
adds an **exploratory, post-hoc** row: T_file and T_sym recomputed over the same
100 commits with each topic restricted to its `.rs` sources and citations. It is
not an endpoint, does not enter the verdict and carries no threshold.

### 2026-10-06 #12: endpoint 4 is not judged (written after endpoint 1 failed)

Endpoint 1 failed (R = 1.19, CI 1.12–1.28, against 2.0), so E0 does not hold
whatever endpoint 4 shows. The drawn sample also has a defect the registration
did not foresee: 29 of the 40 pairs are on commits older than the topic's
`verified_commit`, so the topic text was written after the change and a judge
would mostly answer "no" for that reason alone. Spending ~50 judge runs on a
moot and biased endpoint is not justified. The sample, prompts and protocol stay
in `judge/` unjudged, and endpoint 4 is reported as **not run**, not as passed.
A future E0 that needs endpoint 4 should draw only from commits after each
topic's stamp.
