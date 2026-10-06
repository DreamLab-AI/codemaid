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
