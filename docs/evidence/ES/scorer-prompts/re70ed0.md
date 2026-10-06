You are a blind scorer for a review-recall experiment. Score ONE review run. Work alone; do not look at any other run, and do not read any file under the experiment directory other than the three named below.

Inputs (read-only):
- Issue list (the frozen gold list): /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/ek/sealmap/gold.json — a JSON object whose `items` are known issues (`id`, `kind`, `where`, `note`). There are 45 items.
- Findings of run re70ed0: /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/blind/re70ed0.json — a JSON array; each finding has `n`, `title`, `topics`, `evidence`, `failure`, `confidence`, `marked_by_authors`.
- The code at the pinned revision: /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/ek/sealmap/code (a plain directory, not a git checkout). Use it only to check NOVEL claims. Do not treat `docs/diagrams/` there as extra gold; the gold list is exactly the JSON file.

Task 1 — issues. For each gold issue decide:
- FULL: some finding states the same defect: the same mechanism and the same component or code path, so that an engineer acting on the finding would find and fix this issue.
- PARTIAL: some finding points at the same defect but is less specific, gets only part of it, or names the right component with a related but not identical mechanism.
- MISS: otherwise.
One finding may cover several issues; one issue may be covered by several findings (take the best). Be strict: same topic area alone is not PARTIAL. Search the gold notes for each finding's components, file names and mechanisms (grep the JSON) rather than reading all items linearly, then double-check each match.

Task 2 — findings. Classify each finding:
- REG: it matches at least one gold issue at FULL or PARTIAL.
- NOVEL: it makes a specific, checkable claim about a defect that matches no gold issue.
- VAGUE: too general to match or to check against code.
For every NOVEL finding, check the claim against the code under /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/ek/sealmap/code and give a verdict:
- REAL: the code shows the claimed defect.
- PARTLY: there is something real, but the claim is overstated, conditional, mitigated elsewhere, or a deliberate documented decision read as a defect.
- WRONG: the code contradicts the claim, or the mechanism it describes does not exist.
Cite at least one `path:line` you read for each verdict.

Output: write exactly one JSON file to /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/scores/re70ed0.json with this shape, and nothing else:
{
  "run_id": "re70ed0",
  "issues": [ { "id": "<gold id>", "score": "FULL" | "PARTIAL", "findings": [<n>, ...], "why": "<one line>" } ],
  "findings": [ { "n": <n>, "class": "REG" | "NOVEL" | "VAGUE", "matches": ["<gold id>", ...], "verdict": "REAL" | "PARTLY" | "WRONG" | null, "code": "<path:line, ...>" | null, "why": "<one line>" } ]
}
List in `issues` only the gold issues you score FULL or PARTIAL; every unlisted issue is a MISS. List every finding in `findings`. `verdict` and `code` are null unless class is NOVEL. Validate the file with `node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/scores/re70ed0.json` before you finish.

Reply with one line: counts of FULL, PARTIAL, REG, NOVEL, VAGUE, REAL, PARTLY, WRONG.
