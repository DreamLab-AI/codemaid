### F-01 — Command injection and path traversal risks in git revision export
- Topics: COR-05.2, COR-06.1
- Evidence: `stale --since REV` and `pack --diff REV` export trees using `git archive REV:prefix piped into tar -x, temp dir (main.rs:614-622)`.
- Failure: Passing untrusted, branch-derived, or user-supplied strings to `--since` or `--diff` allows injection of flags into `git archive`. Furthermore, piping archive streams directly into `tar -x` without verifying that paths remain confined to the temporary directory creates a path traversal risk that can overwrite files outside the target directory.
- Confidence: medium (inferred mechanism from description of pipeline execution in `main.rs:614-622`)
- Marked by authors: no

### F-02 — Destructive orphan purging in `generate` can delete hand-authored files
- Topics: COR-01.2, COR-03.2, COR-03.3
- Evidence: `write` deletes any Markdown file on disk that was not generated if its front matter begins with `---` followed by `sealmap: ` (`crates/sealmap-corpus/src/contract.rs:99`, `crates/sealmap-corpus/src/document.rs:153-155`).
- Failure: If a user configures `sealmap generate -o <DIR>` pointing to a directory that contains hand-authored documentation, design records, or topic files carrying a `sealmap: ` schema header, `contract.rs:80-100` classifies those files as `Orphaned` and permanently deletes them from disk without a recycle bin or confirmation prompt.
- Confidence: high
- Marked by authors: no

### F-03 — Non-atomic signing leaves repository corrupted on process interruption
- Topics: COR-04.6, COR-05.3
- Evidence: `seal sign` writes the modified topic file first (`main.rs:679-680`), then writes the updated `seals.lock` second (`main.rs:682`). The design acknowledges: "a failure between the two writes leaves a pointer with no entry, which verify reports as a lock fault" (`crates/sealmap/src/main.rs:680-682`, `crates/sealmap-corpus/src/seal/check.rs:383`).
- Failure: If `seal sign` is killed by SIGINT, SIGKILL, an out-of-memory event, or a system crash between the two disk writes, the topic file references a seal pointer missing from `seals.lock`. Subsequent CI runs and invocations of `sealmap verify` fail with `LockFault`, blocking pipelines until an engineer diagnoses and repairs the desynchronized files.
- Confidence: high
- Marked by authors: no

### F-04 — Shadowed closure parameters produce fabricated call-graph edges
- Topics: EXT-05.6
- Evidence: Closure parameters are never added to the walker's scope environment (`crates/sealmap-rust/src/collect.rs:1348-1351`). If a closure parameter has the same name as an in-scope outer variable, method calls on the parameter resolve against the outer variable's type (`crates/sealmap-rust/src/resolve.rs:866`, `crates/sealmap-rust/tests/extract.rs:351`, `crates/sealmap-rust/tests/extract.rs:392-393`).
- Failure: In common idioms like `let conn = DbConnection::new(); items.iter().map(|conn| conn.len())`, calls on the closure parameter `conn` are bound to methods on `DbConnection`. Sequence diagrams and architectural models show false, hallucinated internal method calls that do not exist in the code.
- Confidence: high
- Marked by authors: yes (Debt)

### F-05 — Default call-filtering policy silently drops unresolved method invocations
- Topics: EXT-04.3, EXT-04.4, DEN-01.6
- Evidence: Under the default `NonStd` policy, external calls are only retained if the callee root looks like a valid dependency, and calls with unknown receivers (`sym:?`) are dropped (`crates/sealmap-rust/src/lib.rs:64-68`, `crates/sealmap-extract/src/confidence.rs:51`).
- Failure: Any method call whose receiver type cannot be resolved (such as dynamic dispatch `dyn Trait`, complex chained returns, or un-inferred local variables) is silently erased from sequence diagrams and dense projections instead of being rendered with visual uncertainty markers. Reviewers and automated agents conclude that no invocation takes place.
- Confidence: high
- Marked by authors: no

### F-06 — Example IDs in documentation prose trigger false CI verification failures
- Topics: COR-04.3, DEL-02.6
- Evidence: Any CommonMark code span outside fences starting with `sym:` is parsed as a formal citation (`crates/sealmap-corpus/src/seal/topic.rs:182`, `331`). In `DEL-02.6`, running `sealmap verify` on sealmap's own repository fails because an inline code span in `MER-02` referencing an illustrative example (`sym:cargo shop . ...`) is flagged as an `UnsealedCitation` (`crates/sealmap-corpus/src/seal/check.rs:390`).
- Failure: Teams documenting architectural patterns or providing illustrative symbol IDs in technical prose cannot run `sealmap verify` in CI without false exit-code 1 failures. This defect prevents sealmap from running its own seal gate in CI (`.github/workflows/ci.yml:9-84`).
- Confidence: high
- Marked by authors: no

### F-07 — Hardcoded exclusion rules and file-size caps silently drop source files
- Topics: MOD-03.2
- Evidence: `SourceSet::load_dir` unconditionally skips all directories named `target`, `node_modules`, `vendor`, `dist`, `build`, or `out` below the root (`crates/sealmap-model/src/source.rs:163`), and silently drops any file exceeding 2 MiB (`crates/sealmap-model/src/source.rs:175-177`).
- Failure: Legitimate Rust crates located in directories named `build` (such as code generators or workspace build utilities) or `vendor`, as well as large generated source files, are omitted without warnings or diagnostics. Calls to symbols in these files become broken or unresolved, and topics citing them fail as `Absent`.
- Confidence: high
- Marked by authors: no

### F-08 — Complete invisibility of macros, procedural macro expansions, and build scripts
- Topics: EXT-01.2, EXT-02.2, EXT-03.6
- Evidence: `build.rs` is never extracted (`crates/sealmap-rust/src/layout.rs:186`). Macro-generated items receive no symbol ID and cannot be cited or sealed (`README.md:342`). Macro invocations in function bodies that do not parse as simple comma-separated expressions or statements emit no call steps (`crates/sealmap-rust/src/collect.rs:1369-1376`).
- Failure: Codebases using declarative or procedural macro frameworks (`tokio::select!`, `actix_web`, `rocket`, `tracing`, `diesel`) produce hollow call graphs. Control flow across macro boundaries is lost, and macro-generated types cannot participate in seals or architectural verification.
- Confidence: high
- Marked by authors: yes (Debt)

### F-09 — Deep AST nesting causes uncatchable process aborts
- Topics: EXT-01.3
- Evidence: `syn` consumes ~40 KiB per nesting level in debug builds (`crates/sealmap-extract/src/isolate.rs:3-7`). The authors acknowledge: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught."
- Failure: Auto-generated serialization files, deeply nested type structures, or complex macro expansions can exceed the 64 MiB thread stack. Because stack overflows trigger `SIGSEGV` or abort rather than standard Rust panics, `catch_unwind` is bypassed, terminating the entire process and failing CI pipelines without diagnostic output.
- Confidence: high
- Marked by authors: yes (Debt)

### F-10 — Windows checkout line endings cause spurious `LockFault` failures
- Topics: COR-04.2
- Evidence: `parse` normalizes line endings, but `parse_canonical` requires `to_toml()` to match the file byte-for-byte (`crates/sealmap-corpus/src/seal/lock.rs:229-234`).
- Failure: In environments where Git checks out files with `\r\n` (standard on Windows hosts with default `core.autocrlf`), `seals.lock` contains CRLF line endings. While `seal sign` accepts the file, `sealmap verify` rejects it as `NonCanonical` and exits with `LockFault`, breaking multi-platform CI matrix builds and Windows developer environments.
- Confidence: high
- Marked by authors: no

### F-11 — Checkout directory name leaks into symbols, breaking determinism
- Topics: DEL-01.3, COR-03.5
- Evidence: The codebase name defaults to `--name`, but falls back to the directory name of the local checkout (`crates/sealmap/src/main.rs:393`, `419-424`), which leaks directly into emitted outputs (`DEL-01.3`).
- Failure: If `--name` is omitted, running `sealmap` across different environments (e.g., local checkout in `repo/` vs CI runner cloning into `/home/runner/work/1/s/`) produces mismatched symbol IDs, different model hashes, and divergent pack files, violating the stated promise of cross-machine byte-identical determinism.
- Confidence: high
- Marked by authors: no

### F-12 — Single oversized topic causes unrecoverable `pack` failure
- Topics: COR-06.3
- Evidence: `pack` refuses silent truncation (`docs/DESIGN.md:161`). When splitting by topic via `shard`, if any single topic with its dense slice exceeds `--budget`, execution fails immediately with `TopicOverBudget` (`crates/sealmap-corpus/src/pack.rs:316-318`).
- Failure: If an authored topic cites several large symbols whose dense slice and source window exceed an LLM context window budget, `sealmap pack --shard` aborts with exit code 1. It cannot subdivide the topic or trim the window, blocking automated review pipelines that operate under strict token budgets.
- Confidence: high
- Marked by authors: no

### F-13 — `stale --since` and `pack --diff` fail on multi-repository projects
- Topics: COR-05.2
- Evidence: `stale --since` rejects invocations using `--repo` (`crates/sealmap/src/main.rs:461-462`).
- Failure: Workspaces configured across multiple repositories (supported elsewhere via `--repo` and `SourceSet` name prefixes) cannot use `sealmap stale --since` or `sealmap pack --diff`. Change-detection and incremental review pack generation are unavailable for multi-repo architectures.
- Confidence: high
- Marked by authors: no

### F-14 — Trait map overview silently truncates relations without omitted count
- Topics: COR-02.5
- Evidence: In `_overview.md`, crate graphs and module graphs log omitted edges as comments (`overview.rs:68`, `125`), and ER models record omitted entities (`overview.rs:190`, `209`). In contrast, the trait map truncates at `max_edges` with "no omitted count" (`crates/sealmap-corpus/src/overview.rs:236`).
- Failure: Downstream agents and human reviewers inspecting `_overview.md` for interface relationships receive a diagram that cuts off silently after 300 relations. Because there is no warning or omitted count, consumers assume the diagram is complete and miss critical trait implementations.
- Confidence: high
- Marked by authors: no

### F-15 — Unpinned workspace dependency breaks downstream builds on declared MSRV
- Topics: DEL-01.2, DEL-01.4
- Evidence: The workspace specifies `rust-version = "1.85"` (`Cargo.toml:8`) and depends on `ignore = "0.4.23"` (`Cargo.toml:35`). `ignore 0.4.30` requires a newer compiler but declares no `rust-version`, so Cargo's MSRV resolver selects it (`DEL-01.2`).
- Failure: While internal builds pass due to `Cargo.lock` pinning `ignore 0.4.29`, external downstream applications consuming `sealmap-model` or `sealmap` on Rust 1.85 without a lockfile pull in `ignore 0.4.30` and fail during compilation, violating the advertised MSRV.
- Confidence: high
- Marked by authors: yes (Debt)

## Not judgeable from this material

1. **Subprocess execution safety in `GitTree` (`main.rs:608-646`):** The material indicates that `git archive` and `tar -x` are invoked, but does not show whether argument arrays bypass shell interpretation or whether path validation occurs during extraction to prevent arbitrary filesystem writes.
2. **Concurrency safety during parallel directory traversal:** While rayon is configured with 64 MiB worker thread stacks in `sealmap-extract::isolate`, the behavior and memory consumption when running on systems with high core counts (e.g., 64–128 threads reserving 4–8 GiB of virtual stack space) cannot be evaluated without inspecting `isolate.rs`.
3. **Behavior on non-UTF-8 source files and paths:** `SourceSet::load_dir` skips non-UTF-8 files silently (`source.rs:181`), but the material does not reveal whether file path normalization (`crates/sealmap-model/src/path.rs`) panics or aborts when encountering platform-specific non-Unicode filenames on disk.
4. **Memory overhead of uncompressed in-memory corpus representations:** `sealmap_corpus::generate` holds all Markdown files, JSON indexes, and diagram representations in an in-memory map before writing (`crates/sealmap-corpus/src/lib.rs:12-58`). Whether this causes memory exhaustion on repositories with tens of thousands of source files cannot be determined from the text.
5. **Macro expansion fallback completeness:** The material states that macro invocations that parse as comma-separated expressions or statements are walked, but does not show what proportion of common macro patterns parse successfully under `syn`'s expression/statement parsers without prior expansion.
