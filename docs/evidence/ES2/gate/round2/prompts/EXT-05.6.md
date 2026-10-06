You are a fidelity gate for ONE rewritten diagram. Work alone. Read only the two topic files named below and the source files that the diagram cites.

- ORIGINAL topic: /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/original-diagrams/extract/05-workspace-name-resolution.md
- REWRITE of the same topic: /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/repaired/docs/diagrams/extract/05-workspace-name-resolution.md
- Source code at the pinned revision (sealmap at 44fef36, a read-only plain directory): /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/snap. A citation such as `sealmap-extract/src/raw.rs:137` or `raw.rs:137` names a file under `crates/` or elsewhere in that tree; find it there.

The diagram under test is the ```mermaid block under the heading `## EXT-05.6 Closure parameters are untyped` in the REWRITE. It replaces the original diagram under the same heading in the ORIGINAL (a `flowchart`). The rewrite was meant to express that original diagram as a `sequenceDiagram` that carries the same facts and the same `path:line` citations, and to add nothing that is not in the original topic.

1. List each fact the diagram under test states: a component, a call or message, an order, a condition or branch, a repetition, a value, cap or constant, a type, field or variant, a state or transition, an outcome, or a relationship.
2. Classify each fact:
   - supported: the original diagram or the original topic's prose states it. Restated, merged or moved into a note is fine.
   - unsupported: the original topic does not state it anywhere. Sequence scaffolding, such as a "caller" participant or an activation, is not a fact. But an arrow between participants claims a call or message, in that order; an `alt`/`opt`/`break` claims a condition, and a `loop` claims repetition. If the original makes no such claim, the claim is unsupported.
   - changed: the original states it differently, for example a reversed order, a different condition, a different value, the wrong outcome, a step made conditional that the original shows as unconditional (or the reverse), or two exclusive outcomes drawn as both happening.
   Where it is unclear whether the rewrite's wording means the same as the original's, read the cited lines in /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/snap to decide. A rewrite fact that the cited code contradicts is changed.
3. Also record, for information only: facts of the original diagram that the diagram under test no longer states (dropped), and citations dropped or added.

Verdict: "fail" if there is at least one unsupported or changed fact, otherwise "pass". Be exact and conservative, and quote short text for every item.

Write exactly one JSON file to /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/gate/round2/EXT-05.6.json:
{"id": "EXT-05.6", "verdict": "pass"|"fail", "facts_checked": <n>, "unsupported": [{"fact": "<short quote from the rewrite>", "why": "<one line>"}], "changed": [{"fact": "<short quote from the rewrite>", "original": "<short quote from the original>", "why": "<one line>"}], "dropped": [{"fact": "<short quote from the original>"}], "citations_dropped": ["<cite>"], "citations_added": ["<cite>"]}
Validate it with node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/gate/round2/EXT-05.6.json. Reply with one line: the verdict and the counts of facts_checked, unsupported, changed, dropped.
