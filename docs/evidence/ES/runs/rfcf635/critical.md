### F-01 — Uncatchable stack overflow aborts process on deeply nested ASTs
- Topics: EXT-01.3, EXT-01
- Evidence: EXT-01.3 states that files are parsed inside a thread pool with 64 MiB stacks wrapped in `catch_unwind` (`isolate.rs:46`), and claims that panic isolation prevents one file from failing the run. However, EXT-01 ("For the business") concedes: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught."
- Failure: In production CI or an automated pipeline processing untrusted pull requests, generated code or pathological macros with deep expression/type recursion trigger a stack overflow. In Rust, stack overflow results in an uncatchable process abort (`SIGSEGV`/`SIGBUS`), killing the entire CI runner or service instance without generating diagnostics or placeholders.
- Confidence: high
- Marked by authors: yes (noted as residual risk in EXT-01)

### F-02 — Non-atomic signing leaves repository in broken `LockFault` state on interruption
- Topics: COR-05.3, COR-04.4
- Evidence: COR-05.3 shows that signing executes two sequential file writes: "topic written only if the pointer changed it (main.rs:679-680)", followed by "lock written canonically (main.rs:682)". The topic is modified on disk before `seals.lock` is written. COR-04.4 and COR-05.3 confirm that a topic file with a `sealed:` pointer lacking a corresponding lock entry produces a `LockFault` (`check.rs:383`, `main.rs:680-682`).
- Failure: If `seal sign` is killed (SIGINT, container timeout, system crash) after writing the topic file but before flushing `seals.lock`, the working copy is corrupted. Subsequent `sealmap verify` executions fail closed with `LockFault`, blocking CI until manual intervention repairs the Git working tree.
- Confidence: high
- Marked by authors: no (authors document this as an intended safety invariant, ignoring the lack of atomic file replacement)

### F-03 — Example and fixture symbol IDs in Markdown prose permanently fail CI verification
- Topics: COR-04.3, COR-04.6, DEL-02.6
- Evidence: COR-04.3 specifies that any inline code span in Markdown paragraphs starting like a global id (`sym:`, a manager, a space) is scanned as a citation (`topic.rs:182, 331`). COR-04.6 enforces that `sign` refuses any citation that cannot be resolved in the codebase (`sign.rs:125-130`). DEL-02.6 shows that in sealmap's own repository, `sealmap verify` fails with exit 1 because MER-02 uses an example fixture ID (`sym:cargo shop . ...`) in prose, which the checker treats as an `UnsealedCitation` that cannot be resolved or sealed.
- Failure: Any architectural documentation or tutorial that references hypothetical, foreign, or example symbols in inline backticks triggers an unresolvable `UnsealedCitation` failure in `sealmap verify`. The document cannot be sealed because the symbols do not exist in the code, permanently failing the CI gate.
- Confidence: high
- Marked by authors: yes (acknowledged as a bug in DEL-02.6, but untreated in the core citation grammar)

### F-04 — Extraction flags are omitted from `seals.lock`, causing silent verification failures
- Topics: COR-03.5, COR-04.1, COR-05.1
- Evidence: COR-03.5 states: "The source flags decide which symbols exist and what they are called, so a seal must be checked with the flags it was signed with (crates/sealmap/src/main.rs:73-74)". However, COR-04.1 shows the lock schema only records `version`, `algorithm`, `generator`, and topic records (`lock.rs:57`). Extraction flags (`--tests`, `--repo`, `--name`, `--external`) are not serialized into `seals.lock`.
- Failure: If an engineer seals a topic using `--tests` or a custom `--name`, but CI runs `sealmap verify` with default flags (or vice versa), symbols in tests or renamed packages appear `Absent`. CI fails false-positively without explaining that the CLI invocation flags diverged from signing time.
- Confidence: high
- Marked by authors: no

### F-05 — Multi-repository workspaces are rejected by `stale --since` and `pack --diff`
- Topics: COR-03.5, COR-05.2, COR-06.4
- Evidence: COR-03.5 advertises multi-repository support where repositories are loaded under name prefixes into a single `SourceSet` (`sealmap/src/lib.rs:112-113`). However, COR-05.2 documents that `stale --since` explicitly refuses execution when `--repo` is passed: `ST->>ST: refuse with --repo (main.rs:461-462)`.
- Failure: Teams running multi-repo or poly-repo workspaces cannot use change detection (`sealmap stale --since`) or incremental review pack generation (`sealmap pack --diff`). Any automation attempting to use diff-aware packaging on multi-repo layouts crashes with an error.
- Confidence: high
- Marked by authors: no

### F-06 — Non-associative CFG-twin fingerprint merging causes platform- and order-dependent hashes
- Topics: MOD-02.3, EXT-06
- Evidence: When multiple definitions share a `SymbolId` across `#[cfg]` gates, MOD-02.3 states that fingerprints are combined sequentially via pairwise folding: `FP->>FP: sort the pair, keyed hash of both (hash.rs:139)`. For $N \ge 3$ variants, sequential folding computes $H(\text{sort}(H(\text{sort}(A, B)), C))$. Keyed BLAKE3 hashing is not associative ($H(\text{sort}(H(\text{sort}(A, B)), C)) \ne H(\text{sort}(H(\text{sort}(B, C)), A))$).
- Failure: If target definitions arrive in different orders (e.g. across different files or reordered in source), the resulting `sig_hash` and `body_hash` diverge despite the code content being identical. This breaks byte-level determinism across platforms and triggers spurious `Contract` or `Behaviour` drift findings.
- Confidence: high (inferred from cryptographic properties of pairwise BLAKE3 folding)
- Marked by authors: no

### F-07 — Closure parameter shadowing attributes calls to unrelated outer variable types
- Topics: EXT-05.6
- Evidence: EXT-05.6 documents the walker's handling of closure parameters: "AR->>AR: keeps the outer type" when a closure parameter shadows an outer typed name.
- Failure: If an outer variable `conn` has type `DatabaseConnection`, and a closure declares a parameter `|conn|` of type `u32` (e.g. an ID), any call `conn.foo()` inside the closure is resolved against `DatabaseConnection`. The sequence diagram draws an exact call to `DatabaseConnection::foo` rather than identifying the method on the closure argument, injecting severe architectural hallucinations into the model.
- Confidence: high
- Marked by authors: yes (author acknowledged in EXT-05.6 diagram notes)

### F-08 — Untyped closure parameters synthesize spurious calls via workspace-wide name guessing
- Topics: EXT-05.6, EXT-05.5
- Evidence: EXT-05.6 states that closure parameters without outer shadowing are `Untyped`. EXT-05.5 states that untyped variables undergo `by_name_only` matching. EXT-05.6 explicitly highlights the consequence: "`vacuum` matched to the only internal vacuum, Inferred, though `conn` is a u8 (tests/extract.rs:351)".
- Failure: Any method called on an untyped closure parameter that happens to match a unique method name in the workspace (or its dependencies) is drawn as a real call (`Inferred`). Non-internal or third-party method calls on primitives, iterators, or builders synthesize phantom architectural dependencies.
- Confidence: high
- Marked by authors: yes (marked as acknowledged debt in EXT-03 and EXT-05.6)

### F-09 — Closures stored in `let` variables are omitted from sequence diagram call sites
- Topics: EXT-03.4, EXT-03
- Evidence: EXT-03.4 shows that when a closure is assigned to a local variable via `let g = ...`, its body is walked at the point of declaration as an `Optional` fragment (`collect.rs:1270`). When `g()` is subsequently invoked, "call dropped as a local closure (resolve.rs:840)".
- Failure: If a closure is invoked multiple times, inside loops, or under conditional branches, none of those invocations appear in sequence diagrams. The diagram depicts the closure's operations as executing unconditionally at the `let` binding site, producing an invalid timeline of execution for any codebase utilizing higher-order functions or callbacks.
- Confidence: high
- Marked by authors: yes (marked as debt in EXT-03)

### F-10 — Unbounded dense slices cause review pack generation to abort on large topics
- Topics: COR-06.2, COR-06.3, DEN-01.5
- Evidence: COR-06.2 states: "Invariant: the dense slice is taken without a byte limit, so it cannot fail on size. The only budget is the pack's (crates/sealmap-corpus/src/pack.rs:479-481)." COR-06.3 shows that during sharding, if a single topic's section exceeds the budget on its own: `S->>OUT: TopicOverBudget names the topic (src/pack.rs:318)`, causing `main.rs:736-743` to exit 1.
- Failure: If a topic cites a central or hub function with a large transitive call graph, its dense slice expands without limit. If the resulting section exceeds `--budget`, sharding cannot split the topic. `sealmap pack` terminates with exit 1 and produces zero output, making automated review pack generation unworkable for hub modules without manual budget adjustments.
- Confidence: high
- Marked by authors: no

### F-11 — Unrelated method additions in dependencies silently delete inferred call edges
- Topics: EXT-05.5, DEN-01.6
- Evidence: EXT-05.5 specifies that `by_name_only` resolution infers an edge if and only if the method name is unique across all reachable crates (`resolve.rs:958-965`). If two or more reachable candidates exist, "Two reachable candidates give no edge rather than a pick... stays sym:? name (crates/sealmap-rust/src/resolve.rs:954-968)".
- Failure: When an upstream dependency or sibling workspace crate introduces a method whose name collides with an existing inferred internal call, the resolver stops inferring the edge. The call arrow silently disappears from generated sequence diagrams and dense trees across the codebase without any compiler error, warning, or code change in the caller's repository.
- Confidence: high
- Marked by authors: no

### F-12 — Minimum call threshold in sequence projections breaks expansion links in the index
- Topics: COR-01.4, COR-02.1
- Evidence: COR-02.1 states that if a callable makes fewer calls than `min_calls`, no sequence diagram is generated (`sequence.rs:19`). COR-01.4 states that `link_expansions` populates the `expands` field in `_index.json` only if the callee target has a sequence diagram fragment (`index.rs:113, 119`).
- Failure: When `min_calls` is set $> 1$ (or $> 0$ for functions that make no internal calls), functions below the threshold generate no sequence fragment. Orchestrating agents traversing `_index.json` via the `expands` merge map find `expands: None` on calls to these functions, falsely assuming they are external or dead ends and failing to expand the call graph.
- Confidence: high
- Marked by authors: no

### F-13 — Directory synchronization fails to clean up orphaned reserved files like `_model.json`
- Topics: COR-01.1, COR-03.2, COR-03.3
- Evidence: COR-03.3 establishes the invariant: "`write` deletes only Markdown files whose YAML front matter opens on the first line (`---`) and whose first key is the `sealmap: ` marker". COR-01.1 and COR-03.2 note that non-markdown reserved files starting with `_` (`_model.json`, `_index.json`) do not have Markdown YAML front matter.
- Failure: If a repository toggles options (e.g. running with `--no-model` after previously generating `_model.json`), `_model.json` is not deleted by `sealmap generate` or `write`. The stale JSON model file remains in the directory indefinitely, leading downstream tools or agents to consume out-of-date codebase models.
- Confidence: high
- Marked by authors: no

### F-14 — Source files exceeding 2 MiB are silently omitted from the model
- Topics: MOD-03.2
- Evidence: MOD-03.2 shows the loading filter in `load_dir`: "alt extension wanted and under 2 MiB (source.rs:175-177)... else unwanted: Note over WK: skipped silently".
- Failure: Any large source file (such as generated parsers, table mappings, or amalgamated files exceeding 2 MiB) is dropped silently without logging an error or diagnostic. The symbols in the file do not exist in the `Codebase`, their calls are omitted, and citations to definitions in those files report false `Absent` errors during verification.
- Confidence: high
- Marked by authors: no

### F-15 — Trait map overview truncates relations without an omission indicator
- Topics: COR-02.5
- Evidence: COR-02.5 compares the overview cap behaviors: the crate graph, module graph, and data model all record an omitted count as a comment when edges or entities exceed caps (`overview.rs:68, 125, 190`). In contrast, for the trait map: "first max_edges relations in key order, no omitted count (overview.rs:236)".
- Failure: In large codebases where trait relations exceed `max_edges` (default 300), the trait map in `_overview.md` silently truncates relations. Because truncation order is alphabetical by key and emits no warning comment, agents and developers review an incomplete trait hierarchy while assuming the diagram depicts the full system.
- Confidence: high
- Marked by authors: no

---

## Not judgeable from this material

1. **Process execution safety in `GitTree` (`main.rs:608-646`):** The material shows `git archive REV:prefix piped into tar -x` being executed via shell/subprocesses for `stale --since` and `pack --diff`. It cannot be determined whether `REV` or path prefixes are sanitized against arbitrary argument injection, or whether `tar -x` prevents path traversal vulnerabilities (e.g., symlink or `../` attacks) inside the temporary directory.
2. **Memory consumption of 64 MiB Rayon thread stacks under heavy concurrency:** The material notes that 64 MiB virtual address space is reserved per thread. Whether this causes out-of-memory crashes on memory-cgroup-constrained CI containers or 32-bit platforms cannot be evaluated without runtime memory telemetry.
3. **Concurrency and race conditions on `.sealmap` and `seals.lock`:** The material does not document file locking on `seals.lock` or the `.sealmap/` directory. It is impossible to verify whether concurrent invocations of `seal sign`, `sealmap generate`, and `sealmap verify` corrupt files during simultaneous execution in CI pipelines.
4. **Behavior on malformed non-UTF-8 source files:** MOD-03.2 notes that non-UTF-8 files are "skipped silently". It is unjudgeable whether a single invalid UTF-8 byte in an otherwise valid Rust file causes the entire file (and all its symbols) to vanish silently, or if encoding diagnostics are surfaced.
5. **Macro expansion invisibility impact on callers:** The material confirms that macro-generated items are not minted as symbols (EXT-02, EXT-03.6), but does not reveal how call sites targeting macro-generated methods or types are classified—specifically, whether they collapse to `sym:?`, produce broken `Inferred` guesses, or drop silently.
