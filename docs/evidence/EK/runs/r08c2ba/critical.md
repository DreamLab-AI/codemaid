### F-01 — Unbounded directory and file deletion outside target boundaries during write convergence
- Topics: COR-01, COR-03.2, COR-03.3
- Evidence: The material claims `write` strictly deletes only generated Markdown files opening with the `sealmap: ` front-matter marker (`crates/sealmap-corpus/src/document.rs:10`-`12`, `crates/sealmap-corpus/src/contract.rs:99`). However, COR-03.3 specifies that when cleaning up orphaned files, `write` "deletes it and empty parents (`crates/sealmap-corpus/src/contract.rs:138`-`141`)".
- Failure: If `sealmap generate -o <DIR>` is invoked against an existing directory tree, a root directory, or a symlinked path where parent directories become empty after orphan deletion, `write` prunes parent directories up the directory tree without documented boundary checks. An attacker or misconfiguration placing a header-marked file in a shared or root path can trigger recursive deletion of parent directories.
- Confidence: high
- Marked by authors: no

### F-02 — Non-atomic multi-file write in `seal sign` permanently corrupts repository state upon interruption
- Topics: COR-04.6, COR-05.3
- Evidence: In COR-05.3, `seal sign` mutates the topic file on disk first (`main.rs:679`-`680`) and then writes `seals.lock` second (`main.rs:682`). The authors frame the result as a safety invariant: "a failure between the two writes leaves a pointer with no entry, which verify reports as a lock fault, so an interrupted sign can never pass the gate (`crates/sealmap/src/main.rs:680`-`682`, `crates/sealmap-corpus/src/seal/check.rs:383`)".
- Failure: If `seal sign` is killed, suffers a power outage, encounters an I/O failure, or exhausts disk space between writing the topic file and writing `seals.lock`, the working tree is left in a corrupted state: the topic has a modified `sealed:` pointer while `seals.lock` lacks the corresponding entry. All subsequent CI runs executing `sealmap verify` immediately fail with a lock fault until someone manually diagnoses and repairs the desynchronized files.
- Confidence: high
- Marked by authors: no

### F-03 — Fenced code blocks are omitted from citation extraction, leaving Mermaid diagrams unverified by seals
- Topics: COR-04, COR-04.3
- Evidence: COR-04 claims a seal mechanically binds a human or model review to the exact version of the cited code to keep "a diagram true" (`docs/DESIGN.md:50`-`121`, COR-04). However, COR-04.3 specifies the scanner implementation: "fenced blocks skipped, so Mermaid is never scanned (`crates/sealmap-corpus/src/seal/topic.rs:246`, `topic.rs:252`)".
- Failure: The core artifacts being reviewed and sealed are Mermaid diagrams contained within fenced Markdown code blocks (` ```mermaid ... ``` `). Because fenced blocks are ignored by citation parsing, citations or symbols referenced directly within the diagram syntax are never extracted, hashed, or stored in `seals.lock`. If the code depicted in the diagram undergoes breaking structural, signature, or behavioural changes, `sealmap verify` continues to report `Holds` as long as the surrounding prose was not edited.
- Confidence: high
- Marked by authors: no

### F-04 — Syn recursive descent parsing induces uncatchable stack overflow, violating panic isolation guarantees
- Topics: EXT-01, EXT-01.1, EXT-01.4, EXT-03
- Evidence: EXT-01 guarantees that extraction "never fails a whole run because of one bad file: a file that does not parse, or that trips a bug, becomes a warning and a placeholder entry" via isolated threads (`crates/sealmap-extract/src/isolate.rs:38`, `crates/sealmap-rust/src/lib.rs:180`). EXT-03 notes expression nesting is capped at 1,024 (`crates/sealmap-rust/src/collect.rs:1113`). Yet EXT-01 concedes: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught."
- Failure: `syn` parses the file inside `collect_file` before the internal 1,024 AST walker limit is ever reached. Deeply nested syntax constructs (e.g. nested macro invocations, complex type definitions, or large generated expressions) cause `syn`'s recursive descent parser to overflow the thread stack. Because stack overflow terminates the process immediately without triggering Rust's `catch_unwind` panic handlers, a single file aborts the entire CLI run or CI job.
- Confidence: high
- Marked by authors: yes (residual risk)

### F-05 — Method calls on returned, computed, and ambiguous receivers are silently dropped from call graphs
- Topics: EXT-04, EXT-05.4, DEN-01
- Evidence: The system claims "Nothing is guessed silently (`README.md:318`-`320`)" and presents its call graphs as an honest substrate. However, EXT-05.4 states that for method receivers, "returned, computed or unknown: dropped (`resolve.rs:874`)" and "values the walker could only describe as computed or returned are not resolved at all". In addition, when two or more reachable candidates exist for an unknown receiver, "no edge is guessed; the call stays `sym:? name` (`resolve.rs:954`-`968`)", which DEN-01 reveals is dropped by the default external policy (`crates/sealmap-rust/src/resolve.rs:830`-`833`).
- Failure: Idiomatic Rust patterns such as builder patterns (`Builder::new().build()`), method chains (`iter().filter().collect()`), and factory calls produce `Recv::Returned` or `Recv::Computed`. All such calls, as well as any method call matching multiple traits or types across reachable dependencies, are silently omitted from generated sequence diagrams and dense projections without a placeholder or notice. Automated review agents make decisions based on an incomplete call graph.
- Confidence: high
- Marked by authors: no

### F-06 — Command-line flag divergence between `seal sign` and `seal verify` causes total seal invalidation in CI
- Topics: COR-05, MOD-01.2, EXT-02
- Evidence: COR-05 notes: "Checking a seal with different settings from the ones it was signed with makes sealed functions look absent." Symbol IDs depend on the codebase name (which defaults to directory name or `_` if unconfigured; `crates/sealmap-extract/src/ids.rs:57`-`63`) and flags like `--tests`, `--name`, and `--repo` (`crates/sealmap/src/main.rs:309`, `393`).
- Failure: If an engineer executes `seal sign` locally where `--tests` is passed or the root checkout directory has a local name, but the CI pipeline runs `sealmap verify` without matching flags or in a checkout with a generic directory path, all generated `SymbolId`s change. `sealmap verify` reports every cited symbol as `Absent` (`crates/sealmap-corpus/src/seal/check.rs:158`) and breaks the CI gate across the entire repository.
- Confidence: high
- Marked by authors: yes (Open / caution)

### F-07 — Repository CI does not dogfood its own seal verification gate
- Topics: DEL-01, DEL-02, DEL-02.2
- Evidence: DEL-02 states: "this repository has not sealed its own diagrams yet, so its CI does not run the seal gate (step 6)." DEL-02.2 shows step 6 ("6 dogfood: seal this corpus") is incomplete. DEL-01 shows CI only checks formatting, MSRV, tests, and generation determinism.
- Failure: The central feature of the 0.2 product—the `sealmap verify` gating mechanism enforcing `seals.lock`—is completely unexercised in production CI by the project itself. Regressions in lock validation, drift calculation, and CI exit codes can pass all workflow checks and ship unnoticed.
- Confidence: high
- Marked by authors: yes (uncompleted roadmap step)

### F-08 — Diagram ID collision during generation triggers an unhandled panic in production
- Topics: MER-02, MER-02.3
- Evidence: MER-02.3 states that during `generate`, `assert_unique_idents` checks every symbol ID and relation endpoint: "same diagram id seen for another sym id, panic (`naming.rs:28`)".
- Failure: Although the authors assert that `Ident::from_symbol` is injective by construction, any unforeseen encoding gap, unhandled unicode pattern, or compiler-specific mangle collision triggers an immediate, unhandled `panic` in `assert_unique_idents`. In a production CI run or automated agent tool invocation, `sealmap generate` crashes with an unhandled panic rather than emitting an error diagnostic.
- Confidence: high
- Marked by authors: no

### F-09 — Explicit contradiction regarding the crates.io release status of 0.2.0
- Topics: DEL-01, DEL-02, DEL-02.2
- Evidence: DEL-01 claims: "None of the 0.2 crates in this tree is published yet; the README's roadmap puts the first 0.2 crates.io release after step 3, which is now done in the tree (`README.md:364`-`367`)." DEL-02 contradicts this in the text: "Step 3 closed with the 0.2.0 publish (`docs/DESIGN.md:302`-`303`)". DEL-02.2 then contradicts DEL-02 by stating: "Three steps done in the tree, the third short of its publish," while the diagram in DEL-02.2 simultaneously asserts: "published as 0.2.0 (`docs/DESIGN.md:298`)".
- Failure: Downstream automation and adopters cannot determine whether the crates are published or installable from crates.io. If dependencies or tooling rely on crates.io for published packages, builds will fail if the tree is actually unpublished, or pull mismatched 0.1 versions.
- Confidence: high
- Marked by authors: no

### F-10 — Unpinned transitive dependency breaks the stated Rust 1.85 MSRV contract
- Topics: DEL-01
- Evidence: DEL-01 claims an MSRV of Rust 1.85 tested in CI (`.github/workflows/ci.yml`). However, DEL-01 reveals: "A dependency (`ignore` 0.4.30) needs a newer Rust than it declares; inside this repository the lockfile avoids it, but a project that depends on the published crates without that lock can still pull it in on Rust 1.85 and fail to build."
- Failure: Any consumer adding `sealmap-model`, `sealmap-corpus`, or `sealmap-dense` to an external workspace without sealmap's internal lockfile on Rust 1.85 will resolve `ignore` 0.4.30 and fail compilation. The advertised MSRV contract is false for external consumers.
- Confidence: high
- Marked by authors: yes (live caveat)

### F-11 — Hardcoded closure sequencing in flow walking inverts actual concurrent and deferred execution order
- Topics: EXT-03, EXT-03.2
- Evidence: EXT-03 states: "closure bodies passed as arguments are placed *after* the call, because they run during it (`crates/sealmap-rust/src/collect.rs:1353`, `crates/sealmap-extract/src/raw.rs:160`)". EXT-03 also notes: "A closure saved in a variable is drawn where it is written, not where it is called."
- Failure: In asynchronous runtimes, thread spawns (`thread::spawn(|| ...)`), task pools, or deferred callbacks, closures do not run sequentially immediately after the invoking function. In sequence diagrams, parallel or background work is rendered as an immediate, synchronous step of the calling thread. Furthermore, closures assigned to local variables display their entire execution body at the declaration site, inverting causality and misleading reviewers.
- Confidence: high
- Marked by authors: yes (Debt)

### F-12 — Complete macro blindness prevents citation and causes false fingerprint invalidation
- Topics: EXT-02, EXT-03.6, EXT-06
- Evidence: EXT-02 notes: "items that a macro generates are not seen, so they get no name and cannot be cited or sealed (`README.md:342`)." EXT-03.6 states that macros produce no call nodes (`CallKind::Macro` is omitted), and macro invocations whose arguments are not plain expressions are skipped entirely. EXT-06 notes that token-level normalisation inside macro arguments can cause formatting changes to alter fingerprints.
- Failure: In repositories heavily using macros (`tracing::instrument`, route registration, `tokio::select!`, or custom derive macros), items produced by macro expansion cannot be cited in architectural topics or protected by seals. If an existing function is refactored into a macro invocation, the call disappears from the model, causing `sealmap verify` to report false absent symbols or altered contracts.
- Confidence: high
- Marked by authors: yes (Debt / limit)

### F-13 — In-memory corpus generation risks out-of-memory crashes on large codebases
- Topics: COR-01, COR-03.1
- Evidence: COR-01 notes that `sealmap_corpus::generate` projects the entire codebase into an in-memory map from path to text (`crates/sealmap-corpus/src/lib.rs:12`-`58`), and COR-03.1 confirms that both `generate` and `generate --check` "regenerate the whole corpus in memory first (`main.rs:310`)". COR-01 confirms that the corpus is "about the same size as the source (`README.md:59`)".
- Failure: On codebases containing millions of lines of code across thousands of files, the system maintains the source files, ASTs, intermediate representation, and the entire set of generated Markdown strings in RAM simultaneously. In containerized CI runners or memory-constrained developer environments, the process triggers an OOM kill before any disk operations or checks can complete.
- Confidence: high (inferred from in-memory architecture)
- Marked by authors: no

### F-14 — Review pack generation fails completely under byte budget without fallback or partial degradation
- Topics: COR-06, COR-06.2, DEN-01
- Evidence: COR-06 guarantees that "Nothing is truncated. Over budget, pack refuses with the header's size and each topic's (`crates/sealmap-corpus/src/pack.rs:278`-`282`)." In DEN-01 and COR-06.2, `Dense.slice` is taken with "no byte limit (`crates/sealmap-corpus/src/pack.rs:479`-`481`)". COR-06 notes that `shard` "refuses only a topic too large to fit alone (`crates/sealmap-corpus/src/pack.rs:295`-`331`)".
- Failure: If an authored topic cites several hub symbols whose dense slices and source windows exceed the requested byte ceiling (e.g. LLM context window boundaries), `sealmap pack` fails completely. `sealmap pack --shard` also hard-fails on that topic. Because there is no progressive truncation of source windows or degradation mode, automated review pipelines halt completely until limits are raised or topics manually restructured.
- Confidence: high
- Marked by authors: no

### F-15 — Lossy identifier generation in overview diagrams causes node collision and corrupted graph topologies
- Topics: MER-01, MER-01.5, MER-02, COR-02
- Evidence: MER-01 states that `Ident::new` is "lossy" and replaces all characters outside letters, digits, and underscores with an underscore (`escape.rs:64`). MER-02 acknowledges that while symbol IDs use injective mapping, overview diagrams (`crates/sealmap-corpus/src/overview.rs:14`) still mint identifiers the lossy way.
- Failure: In overview diagrams (module graphs, crate dependency diagrams, and ER diagrams), distinct entities whose names differ only by non-alphanumeric separators (such as `module-v1` vs `module_v1` or nested module paths) are assigned identical Mermaid node IDs. Mermaid silently merges these distinct modules into a single diagram node, cross-connecting unrelated edges and generating an inaccurate architecture map.
- Confidence: high (inferred from MER-01 and MER-02 specifications)
- Marked by authors: yes (partially acknowledged in MER-02)

---

## Not judgeable from this material

1. **Path traversal sanitization during directory cleanup:** The source implementation of `crates/sealmap-corpus/src/contract.rs` is required to verify whether the deletion of orphaned files and empty parent directories is strictly clamped within the specified output directory root.
2. **Subprocess execution and command injection:** The implementation of `stale --since` and `pack --diff` in `crates/sealmap/src/main.rs` is required to determine whether git revision strings are passed safely without shell interpolation and whether temporary checkout directories are guaranteed to be cleaned up on crash or abort.
3. **Lockfile concurrency and race conditions:** The behavior of `seals.lock` when multiple `seal sign` operations run concurrently across different branches or processes cannot be evaluated without seeing lock contention and merge conflict resolution logic.
4. **Symlink handling and cycle detection:** The layout scanner implementation in `crates/sealmap-rust/src/layout.rs` is required to assess how the directory walker handles circular directory symlinks, broken links, or permissions errors.
5. **Real-world downstream token consumption and error rates:** The empirical impact of `dense.txt` on agent reasoning and whether the omission of unresolvable calls misleads downstream models cannot be evaluated without the unexecuted step 4 (E0-R evaluation) test results.
