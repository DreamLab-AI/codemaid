You are the author of a diagrams-as-code corpus rewrite. The working directory is a plain copy of the sealmap repository (source code and docs) at revision 44fef36. The corpus is `docs/diagrams/`: 20 topic files under `docs/diagrams/<area>/NN-*.md`. Every other file in this directory is source code you may read but must not change.

Brief (fixed):

Rewrite every non-`sequenceDiagram` block in every topic as one or more `sequenceDiagram` blocks that carry the same facts and the same `path:line` citations. Express decisions as `alt`/`opt`/`loop`/`break` blocks, and structure as participants and notes. Keep every narrative, register marker and invariant verbatim. Add nothing that is not in the original topic. Existing `sequenceDiagram` blocks stay as they are.

Operating rules:
- Edit only the 20 topic files under `docs/diagrams/<area>/`. Do not edit REGISTER.md, README.md, COVERAGE.md, DECISIONS-TIMELINE.md, the config or the tools, and do not touch anything outside `docs/diagrams/`.
- Keep each topic's frontmatter, its `## For developers` and `## For the business` sections, every `## <ID>.<n>` heading, and every paragraph under it (`**What it shows**`, `**Why it is this way**`, `**Invariant:**`, register markers such as `**Debt:**`, `**Tension:**`, `**Drift:**`, `**Open:**`) byte for byte. Only the contents of the ```mermaid fences change.
- If one original diagram needs more than one sequenceDiagram, add the extra blocks as new sections with new headings that continue that topic's numbering (`## <ID>.<next n> <title>`), each with one mermaid block. Do not renumber existing headings.
- Carry every `path:line` citation from an original block into its replacement. Do not invent or re-derive citations.
- Keep any scratch files (Mermaid probes, notes) under `.scratch/` in the working directory; `TMPDIR` points there. Never write outside the working directory.
- Before finishing, the corpus must pass this gate from the working directory (structure and Mermaid grammar, renders no wider than 4500px):
    node docs/diagrams/tools/diagram-index-gen.cjs docs/diagrams --check --render --jobs 8
  Fix any error it reports, then run it again until it exits 0.
- Finish with a short summary: topics rewritten, blocks replaced, blocks added, and anything you could not express as a sequence.
