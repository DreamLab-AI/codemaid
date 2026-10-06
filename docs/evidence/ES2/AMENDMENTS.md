# ES2 amendments

Dated and appended. Each says whether it was written before or after any ES2 output was seen.
Endpoints and thresholds are unchanged by all of them.

## ES2-1, 2026-10-06T10:06Z, before any output: author harness

GLM-5.3-Flash runs exactly as ES-2 (`env -i` with only HOME, PATH, TERM, ANTHROPIC_BASE_URL,
ANTHROPIC_API_KEY, CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000; `--bare --strict-mcp-config`; tools Read, Edit,
Write, Glob, Grep, Bash; no thinking budget, so default maximum effort), with a fresh private HOME under
`es2/` and, added, `TMPDIR` pointing inside its working directory. The working directory is a plain
`git archive` copy of 44fef36 (no `.git`). The author prompt is ES's `author-brief.md` byte for byte plus
one operating rule, inserted before the gate rule: "Keep any scratch files (Mermaid probes, notes) under
`.scratch/` in the working directory; `TMPDIR` points there. Never write outside the working directory."
In ES the author wrote probes to `/tmp/mmdtest`; the team lead's harness rule is that its scratch stays
inside its own directory. `.scratch/` is excluded from every comparison and from the pack.

## ES2-2, 2026-10-06T10:06Z, before any output: gate and repair mechanics

The PREREG fixes the gate's inputs and verdict, not its wording; the wording is fixed here, before any
rewrite exists, in `gate-template.md`. Unit: every `sequenceDiagram` block in the rewrite that is not
byte-identical to the original block under the same heading (an added heading's block is its own unit).
Verdict: fail if the gate finds any fact the original topic does not state (unsupported) or states
differently (changed); dropped facts and citation changes are recorded but do not fail a diagram. The
gate's definitions of a fact and of an invention follow ES's fidelity-judge prompt, so the gate and the
endpoint-2 judges test the same thing. Repair: one fresh GLM session (same harness as ES2-1) receives
every failing diagram with its gate list (`repair-brief.md`) and a read-only copy of the original
corpus under `.original/`; each failing diagram is repaired once. Each repaired block is re-gated by a
new Claude Opus subagent with the same template. A block that still fails, or that the repair left
unchanged or turned into a non-sequence diagram, is replaced by the original block under its heading
(an added section that still fails is deleted, and its parent original is restored).

## ES2-3, 2026-10-06T10:06Z, before any output: the Codex read-only sandbox cannot start here

A probe (three `codex exec --sandbox read-only` calls, gpt-6-astra, cwd = the 44fef36 snapshot) showed
that every shell command fails with `bwrap: Failed to make / slave: Operation not permitted`: the
container runs under no-new-privileges, and `~/.codex/config.toml` already records that bwrap is
unavailable. A read-only judge therefore can neither read its two input files nor write its JSON. The
registered conditions are kept (gpt-6-astra, high effort, `--sandbox read-only`, cwd = a read-only
44fef36 snapshot, prompt on stdin), and the ES judge prompt is kept verbatim with two harness changes:
(1) the full text of the ORIGINAL and REWRITE files is appended after the prompt, each in a fence headed
by its path, with the line "The shell is unavailable in this sandbox; the two files' contents follow.";
(2) the closing instruction to write and validate a file at {{OUT}} and reply with one line is replaced
by "Reply with exactly that JSON object as your whole final message, and nothing else." The harness
writes the final agent message to {{OUT}} and validates it. The judging task, definitions and JSON
schema are unchanged.

## ES2-4, 2026-10-06T10:06Z, before any output: Gemini runner fallbacks (known bugs)

The review runner is ES's `run.cjs` unchanged. Two known faults of the sealmap-review script are handled
as follows, and reported if they trigger: (1) Node's `fetch` (undici) drops a response after 300 s of
headers wait regardless of the AbortSignal; a run that fails that way is re-sent once with identical
request bytes over `node:https` (no timeout below 600 s). (2) `parseFindings` leaves a field null when its
label is bold (`- **Evidence**:`); if any parsed finding has a null Topics, Evidence or Failure field,
or the parsed count is below the number of `F-NN` headings in `critical.md`, findings are re-extracted with a tolerant parser that accepts plain or bold labels, and
the scorer sees the tolerant extraction.

## ES2-5, 2026-10-06T10:24Z, after an aborted author session (no rewrite output existed)

The first author session (started 10:06:15Z) ended at turn 29 with `API Error: 401 Authentication
Failed`: the Z.AI key in the session environment was rotated in `agentbox/.env` at 10:18:50Z. The
session had only read files and written one Mermaid probe under `.scratch/`; it made no edit to any
topic (0 lines added). Its logs are kept in `aborted/`. The author run was restarted from a fresh
`git archive` copy and a fresh HOME, identical in every respect except that the key is now read from
`agentbox/.env` at launch (never printed or stored). This is the one and only authoring run.

## ES2-6, 2026-10-06T11:00Z, after the rewrite and the round-1 gate were seen: render temp directory

The author could not run the render gate with `TMPDIR` inside its working directory: Chromium's Unix
socket path (108-byte limit) does not fit under the long scratchpad path. It therefore ran mmdc with
`TMPDIR=/tmp/mmdc-tmp`, outside its directory, as ES's author did with `/tmp/mmdtest`. Its other scratch
stayed under `.scratch/`. This changes nothing about the rewrite. The repair brief names the one
exception explicitly (`TMPDIR=/tmp/es2-glm` for the gate command only). The repair brief
(`repair-brief.md`) is otherwise written from ES2-2: the author brief's fixed rewrite rule, the
original corpus under `.original/`, the gate's definitions in one paragraph, and each failing
diagram's unsupported and changed facts with the gate's one-line reasons, verbatim.

## ES2-7, 2026-10-06T11:35Z, after all output was seen: how endpoint 1 is read (no change)

The PREREG states endpoint 1 as "R2 − A ≥ −0.05 (bootstrap 95% CI reported)" and says ES2 reuses ES
otherwise. ES applied the margin to the CI lower bound. `analyse-es2.cjs` applies ES's rule, as written
before any output. The result sits between the two readings: the point estimate passes, the lower
bound does not. RESULTS.md reports both. The decision does not depend on the reading, because
endpoint 2 fails on its own. Nothing was re-run or changed.
