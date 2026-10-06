# ES amendments (dated, written before the rewrite was produced)

## ES-1, 2026-10-06T09:01:40Z: the corpus revision

The PREREG names 28c62d8 as "exactly the copy EK reviewed". The two are not the same. EK's sealmap copy
(`EK/sealmap/corpus`, the A/S packs and the 45-row gold) matches 44fef36 (= fdd8207 for docs/diagrams)
byte for byte, 26/26 files, with 20 topics. 28c62d8 is ER's gen snapshot. It is an ancestor of 44fef36,
26 commits earlier, with 16 topics, and 21 of its 22 corpus files differ from EK's copy. The comparators
(A, S, gold) only make sense on EK's copy, so arm R is authored on EK's copy at 44fef36 (the source the
author sees is EK's `sealmap/code`, also at 44fef36). The "16 topics" in the PREREG becomes 20.
Endpoint 3's threshold stays at an absolute "more than 2 topics", which is stricter at 20.

## ES-2, 2026-10-06T09:01:40Z: harness environment

The author runs under `env -i` with only HOME (a private scratch dir), PATH, TERM, ANTHROPIC_BASE_URL,
ANTHROPIC_API_KEY and CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000, plus `--bare --strict-mcp-config`. The parent
session's environment carries CLAUDE_EFFORT=medium and CLAUDE_CONFIG_DIR (the operator's real config,
including the Explanatory output style and plugins). A plain `env HOME=... claude` inherits both, which
would lower GLM's effort and inject an operator output style. No thinking budget is set.
Tools: Read, Edit, Write, Glob, Grep, Bash. The brief is `author-brief.md`. It is the PREREG's brief verbatim plus
operating rules: edit only the mermaid fences of the 20 topics, and give any extra block its own new heading.
It also hands the author the structure-and-render gate command, so a gate failure is fixed by the author before review.
