# E0d pre-registration: language-agnostic hunk overlap, the baseline to beat

Registered 2026-10-06 while E0b and E0c were running and before either's
results were seen. Origin: an external review (gpt-6-astra, high reasoning,
`docs/evidence/consult/2026-10-06-codex.md`) argued that per-file tracking is a
weak baseline and that a changed-line/citation overlap rule, which needs no
parsing and so covers every language, should be beaten before more extraction
machinery is built. Amendments are dated and appended below.

## Unchanged from E0

Inputs, pins, the same 100 eligible commits per repository, citation reading
(E0 amendments #1, #3, #4, #9), T_file and the bootstrap (seed 20261006). The
harness extends `bench/e0/`; E0's outputs stay byte-identical.

## The overlap rule

For each topic and commit P→C, take every citation's line or range in the
topic's cited file **at P**, relocated from the topic's stamp to P through the
intervening diffs (a citation whose line is deleted is **lost**). Compute
changed lines with a **zero-context** diff (`git diff -U0 P C`) per file. The
topic is flagged (**T_hunk(k)**) if any changed or deleted P-line lies within k
lines of a cited line, or a cited file is deleted or renamed away, or a
citation is lost or its relocation is ambiguous. Pure line movement never
flags.

**k = 5 is primary, fixed now.** k = 0 and k = 20 are reported as sensitivity
rows only; no k is chosen after seeing results.

## Endpoints

1. **Primary:** R_hunk = ΣT_file / ΣT_hunk(5) over VisionClaw. **≥ 2.0.**
2. **Co-primary:** the same pooled over VisionClaw and agentbox (overlap needs
   no parser, so agentbox's non-Rust citations are in scope). **≥ 2.0.**
3. **Secondary:** per repository, k = 0 and k = 20 rows, medians and p90s, and a
   side-by-side table against E0's R_sym, E0b's R_region and E0c's R_flow
   computed on the same commits.
4. **Staleness recall and precision, labelled independently of any detector.**
   Draw 60 topic/commit pairs uniformly from all pairs where the topic's cited
   files changed after its stamp (seed 20261006; 45 VisionClaw, 15 agentbox, or
   all if fewer). Each is labelled by a fresh Claude subagent, blind to every
   detector's output: **did this change make anything this topic states wrong
   or misleading?** Every "yes" is re-checked by a second blind judge; stale
   needs both. From these labels, report for each detector (file, sym, region,
   flow, hunk(5)): **recall** = stale pairs it flagged / all stale pairs, and
   **precision** = stale pairs among its flags, each with a Wilson 95% interval.
   Success for hunk(5): **recall ≥ 0.90** (lower bound reported).

**E0d holds** if endpoint 4 succeeds and endpoint 1 or 2 succeeds. Because
recall is now measured directly, any detector's ratio is read alongside its
recall; a high ratio with low recall counts as a fail for that detector.

## Amendments

None.

The "None." above was true at registration. The entries below are appended,
never edited.

### 2026-10-06 #1: making the registration concrete (written before any E0d code ran or any E0d number was seen)

Written while building the harness, before the first E0d run. E0b's and E0c's
results had been published by then (E0d itself was registered before them);
no E0d count, ratio or label existed. None of these changes an endpoint, a
threshold, k, the window, the seed or the sample sizes. Each picks the most
literal reading of a point the registration leaves open, and where two
readings are both defensible the other one is named as a **sensitivity row,
fixed now**, never an endpoint.

**Citations.** As E0 (amendments #1, #3, #4, #9): mermaid and prose
citations, resolved to one `sources:` file in the repository being walked;
unresolved or ambiguous ones are excluded. A range `a-b` is read whole
(lines `a..=b` at the stamp), where E0 kept only its start; `b < a` is read
as `a`. `:a,b,c` stays three citations. A topic's **cited files** for the
overlap rule are the files at least one of its citations names. A `sources:`
file that no citation names has no line to overlap, so it never sets
T_hunk (literal reading). Sensitivity row **hunk(5)+uncited**: T_hunk(5),
plus a flag whenever an uncited source changes (tracked whole, as E0 does).

**Relocation, stamp → P.** The stamp is E0's (`shaFor`, amendment #3). A
citation's file is relocated with the zero-context diff between the file's
blob at the stamp and its blob at P (`git diff -U0` over the two blob ids,
`--diff-algorithm=myers --indent-heuristic --no-textconv --no-ext-diff`,
pinned so no user configuration can change a hunk). "The intervening diffs"
is read as their net effect, one diff from the stamp to P. That needs no
ancestry, so it also relocates backwards when P precedes the stamp (most of
E0's window does). A line inside a hunk's old range is **deleted**; any other
line moves by the net line count of the hunks wholly above it. A range is
**lost** when its first or its last line is deleted; otherwise its span at P
runs from the relocated first line to the relocated last line. A citation is
lost as well when its file is absent at P. Its relocation is **ambiguous**
when it cannot be computed: the topic has no usable stamp (E0's `no_stamp`,
`stamp_unknown`), the file is absent at the stamp, the cited line is 0 or past
the file's last line at the stamp, or either blob is binary (git's own test).

**When a lost or ambiguous citation flags.** The registration flags a topic
whose citation "is lost or its relocation is ambiguous" without saying on
which commits. T_hunk is a per-commit change detector, so the primary reading
mirrors E0's conservative absent-at-both rule: a lost or ambiguous citation
flags the commit when its file changed between P and C (it cannot be
overlap-tested, so any change to its file counts). With this reading T_hunk ⊆
T_file, and the run checks that. The literal reading, where a citation lost or
ambiguous at P flags every commit whatever it touches, is the sensitivity row
**hunk(5) literal-lost**.

**Changed P-lines, P → C.** `git diff -U0` between the file's blobs at P and
C, pinned as above. A hunk that removes or replaces old lines `s..s+n-1`
changes those P-lines. A pure insertion after P-line `s` (`n = 0`) changes no
P-line but sits in the gap between `s` and `s+1`, and is within k of a cited
span `[a, b]` when `s+1 ≥ a−k` and `s ≤ b+k`. At k = 0 a line inserted directly
before or after a cited line therefore flags; a line inserted elsewhere, which
only moves the citation, does not ("pure line movement never flags"). A
replaced or deleted P-line `x` is within k when `a−k ≤ x ≤ b+k`. A cited file
present at P and absent at C is "deleted or renamed away" (`--no-renames`, as
E0 amendment #6). A cited file whose blob at P or C is binary and that changed
counts as ambiguous, so it flags. A path change with an identical blob (mode
only) changes no line.

**Ratios.** Endpoints 1 and 2 are ΣT_file / ΣT_hunk(5) over E0's same 100
(VisionClaw) and 100 + 100 (pooled) commits; bootstrap as E0 amendment #7,
pooled by concatenating the two repositories' commit rows. The k = 0, k = 20,
literal-lost and +uncited rows get the same ratios, with no threshold. The
side-by-side rows recompute E0's R_sym and R_hop and E0b's R_region over the
same commits with E0's own code (E0b's T_region over all sources and
citations, its endpoint 1). R_flow is E0c's, ΣT_file^seq / ΣT_flow, because
T_flow is defined only on sequence citations; its scope differs and the table
says so.

**Endpoint 4: the population and the draw.** "Pairs where the topic's cited
files changed after its stamp" uses E0's vocabulary, where a cited file is a
`sources:` file (E0 PREREG "Commit window"). The population is every (commit,
topic) pair in E0's window flagged by T_file whose change wholly postdates the
stamp: the stamp is P or an ancestor of P (E0b amendment #6, E0c amendment
#1). Pairs are listed in window order, then topic file order. One ChaCha8
generator seeded 20261006 draws 45 VisionClaw pairs, then 15 agentbox pairs,
by E0's partial Fisher–Yates (`stats::draw`), all of a stratum if it has
fewer. With this population T_file flags every pair, so its recall is 1 by
construction and its precision is the stale rate.

**Endpoint 4: blind labels.** The harness has no subagent tool and never
labels. It writes `labels/<id>.prompt.md`, `labels/sample.json` (ids, commits,
topics, diff files, no detector output) and `labels/PROTOCOL.md`. Each prompt
holds the whole topic text at the corpus pin and the commit's diff (E0's
format, default context) restricted to the topic's `sources:` files in that
repository, with the registered question verbatim: **Did this change make
anything this topic states wrong or misleading?** It also says, truthfully for
every pair, that the topic was verified against a revision at or before the
commit's parent. No prompt, file name, id or ordering carries or encodes any
detector's verdict. Every detector's flag for every drawn pair goes to
`detectors.json`, outside `labels/`, which the protocol forbids showing to a
labeller. A first `yes` goes, unchanged, to a second fresh labeller who does not
see the first verdict. A pair is **stale** when both say yes.

**Endpoint 4: scoring.** `e0 e0d-score` scores every detector on the same
labels: T_file, T_sym and T_hop (E0), T_region (E0b, endpoint-1 form), T_flow
(E0c, all sequence citations; a topic with no sequence citation is out of
E0c's scope and counts as not flagged), and T_hunk at k = 0, 5 and 20, literal-lost
and +uncited. Recall = stale pairs flagged / stale pairs; precision = stale
pairs among the detector's flagged pairs. Both are counted over the drawn
sample (unweighted; strata are also reported apart) and carry a Wilson 95%
interval (both bounds). Endpoint 4 succeeds when hunk(5)'s recall is ≥ 0.90.
With no stale pair, recall is undefined and endpoint 4 is not passed. The
labeller's `line_anchors_only` is recorded. A **sensitivity row**, not the
endpoint, drops from "stale" the pairs where both labellers mark their yes as
resting only on moved `path:line` anchors.

**Reading each detector.** The registration's "a high ratio with low recall
counts as a fail" is made concrete with the only recall threshold it states:
a detector's reading is PASS only when its ratio on VisionClaw clears 2.0
(for hunk(5), endpoint 1 or 2) **and** its recall is ≥ 0.90. A ratio that
clears 2.0 with recall below 0.90 reads **FAIL (ratio without recall)**. A
ratio below 2.0 reads **FAIL (ratio)**. These per-detector readings are
reported. Only hunk(5)'s enters the E0d verdict, which is the registered one.

**Timing and byte identity.** As E0 amendment #2: wall-clock goes to
`TIMING.md`. Two cold runs (fresh scratch) must be byte-identical across
`results.json`, `RESULTS.md`, `detectors.json` and every file under `labels/`
except labels. `git --version` is recorded in `results.json`, because hunk
placement depends on git's diff implementation. Range-end reading is added to
the shared citation parser without changing E0, E0b or E0c output; their
re-runs are checked byte for byte against the committed files.

### 2026-10-06 #L1 (team lead): labelling mechanism (after labels existed, before any scoring)
Labellers ran as fresh `claude -p --model sonnet` processes, one per prompt, with the prompt
file on stdin and nothing added, from an empty working directory with `--tools ""`,
`--setting-sources ""`, `--strict-mcp-config`, `--no-session-persistence` and
`--disable-slash-commands`. That satisfies PROTOCOL.md's "fresh subagent per prompt with no
repository access" more strictly than a tool-holding subagent would. The prompt set hashed
`0ba3cf6b0b749b79` before and after labelling. Outcome: 60 pairs, 29 first-round yes, 29
confirmed by an independent second labeller, 0 invalid. Helper: `labels/run-label.sh`.

### 2026-10-06 #L2 (team lead): correction to #L1 (after scoring; no label or score changed)
#L1 says the labellers ran with "nothing added". That is wrong in one respect: each `claude -p`
call also passed `--system-prompt "You are a careful labeller. Follow the user's instructions
exactly."` (see the runner `run-label.sh`). The prompt files themselves were unchanged (hash
`0ba3cf6b0b749b79`), and the system prompt is generic, naming no detector, label or outcome.
Also recorded here: `--model sonnet` resolved to `claude-sonnet-5-5` on Claude Code 2.1.289, so
the gold labels are Sonnet 5.5's. EH's Sonnet triage recall is therefore partly self-agreement;
see EH amendment #A2.
