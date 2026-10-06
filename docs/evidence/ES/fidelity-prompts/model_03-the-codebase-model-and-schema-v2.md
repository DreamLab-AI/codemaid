You are a blind fidelity judge for a diagram rewrite. Work alone and read only the two files named below.

- ORIGINAL topic: /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/fidelity-inputs/model_03-the-codebase-model-and-schema-v2/original.md
- REWRITE of the same topic: /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/fidelity-inputs/model_03-the-codebase-model-and-schema-v2/rewrite.md

Both are Markdown topic files from a diagrams-as-code corpus: prose sections plus ```mermaid diagrams that cite source code as `path:line`. The rewrite was supposed to turn every non-sequence diagram (flowchart, classDiagram, stateDiagram) into one or more `sequenceDiagram` blocks that carry the same facts and the same `path:line` citations, keep all prose verbatim, and add nothing that is not in the original topic. Extra diagram sections with new headings are allowed.

Compare the DIAGRAMS. For every original diagram that the rewrite changed, list each fact it states: a component, a call or message, an order, a condition or branch, a value, cap or constant, a type, field or variant, a state or transition, an outcome, or a relationship. Then check whether the rewrite's diagrams still state it, in any form (a participant, message, note, alt/opt/loop/break block). Then list each fact the rewrite's diagrams state that appears nowhere in the ORIGINAL topic, in its diagrams or its prose.

- facts_dropped: facts in an original diagram that no rewrite diagram states. Restated, merged or moved into a note does not count as dropped.
- facts_invented: facts in a rewrite diagram that the original topic does not state anywhere. Sequence scaffolding, such as a "caller" participant or an activation, is not a fact. An explicit claim of a call, order, condition, value, outcome or relationship that the original does not make is invented. So is a change of meaning, such as a reversed order, a different condition or the wrong outcome.
- citations_dropped: each `path:line` (or `path:a-b`, or a bare `file.rs:N`) present in an original diagram but absent from every rewrite diagram.
- citations_added: each citation in a rewrite diagram that appears nowhere in the original topic.
- prose_changed: true if any text outside the mermaid fences differs, apart from added headings and their new diagram sections.

Be exact and conservative. Quote the original or rewrite text for each fact (short). Do not judge whether the facts are true of the code.

Write exactly one JSON file to /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/fidelity/model_03-the-codebase-model-and-schema-v2.json:
{"topic": "model/03-the-codebase-model-and-schema-v2.md", "diagrams_rewritten": <n>, "facts_checked": <n>, "facts_dropped": [{"diagram": "<id>", "fact": "<short quote>"}], "facts_invented": [{"diagram": "<id>", "fact": "<short quote>", "why": "<one line>"}], "citations_dropped": ["<cite>"], "citations_added": ["<cite>"], "prose_changed": true|false, "notes": "<one line>"}
Validate it with node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/fidelity/model_03-the-codebase-model-and-schema-v2.json. Reply with one line: counts of facts_checked, facts_dropped, facts_invented, citations_dropped, citations_added.
