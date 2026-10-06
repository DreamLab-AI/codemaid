### F-01 — Non-atomic multi-file write in `seal sign` corrupts repository state on interruption
- Topics: COR-04.4, COR-05.3
- Evidence: `seal sign` updates two separate files on disk: the topic file first (`main.rs:679`-`680`), and the `seals.lock` file last (`main.rs:682`). COR-05.3 explicitly notes: "a failure between the two writes leaves a pointer with no entry, which `verify` reports as a lock fault" (`main.rs:680`-`682`, `check.rs:383`).
- Failure: If the process is killed (OOM, SIGKILL, container preemption, or power failure) after the topic file is written but before the lockfile is updated, the topic points to a seal that does not exist in the lock. Subsequent CI runs fail with `LockFault`, halting deployment pipelines until an engineer manually diagnoses and repairs the desynchronized files.
- Confidence: high
- Marked by authors: yes (Marked as an invariant in COR-05.3, but lacks atomic write or rollback mitigation)

### F-02 — Subprocess execution of git archive and tar during `stale` and `pack` risks resource leaks and execution failures
- Topics: COR-05.2, COR-06.4
- Evidence: `GitTree::export` executes system subprocesses by piping `git archive REV:prefix` into `tar -x` within a temporary directory (`main.rs:608`-`622`). Cleanup relies entirely on Rust's `Drop` implementation on the temporary directory handle (`main.rs:644`-`646`).
- Failure: If the process terminates abruptly (such as an unhandled stack overflow, SIGKILL, or panic in surrounding routines), `Drop` does not run, leaking extracted code trees into temporary storage. In containerized environments with limited `/tmp` mounts, repeated runs will exhaust disk space.
- Confidence: high
- Marked by authors: no

### F-03 — Seal gate verification fails on its own codebase and is disabled in CI
- Topics: DEL-01.1, DEL-02.6
- Evidence: DEL-01.1 lists all CI jobs; none execute `sealmap verify` (`ci.yml:9`-`84`). DEL-02.6 confirms that running `sealmap verify` against the repository exits with code 1 because topic MER-02 contains an inline code span referencing a fixture identifier (`sym:`), which the parser misinterprets as an unsealed citation (`check.rs:390`).
- Failure: The core product capability—automated CI gating via `sealmap verify`—cannot pass on the project's own repository and is not exercised in CI. Any team that documents mock identifiers, syntax examples, or test fixtures in Markdown code spans will trigger false-positive CI build failures with exit code 1.
- Confidence: high
- Marked by authors: yes (Marked as an uncompleted dogfooding step in DEL-02.6)

### F-04 — Silent omission of large source files, non-UTF-8 files, and common directory names
- Topics: MOD-03.2
- Evidence: In `SourceSet::load_dir`, the directory walker silently skips files larger than 2 MiB (`source.rs:175`-`177`), silently skips non-UTF-8 files (`source.rs:181`), and hardcodes the exclusion of directories named `target`, `node_modules`, `vendor`, `dist`, `build`, and `out` below the root (`source.rs:163`).
- Failure: Legitimate Rust code residing in subdirectories named `build` or `out` (such as `crates/foo/src/build/codegen.rs`), large generated lookup tables exceeding 2 MiB, or files with non-UTF-8 test fixtures are completely omitted from the model without any warning, diagnostic message, or log entry. The resulting architecture model is silently incomplete.
- Confidence: high
- Marked by authors: no

### F-05 — Total omission of build scripts and non-target source files
- Topics: EXT-01.2
- Evidence: In `sealmap-rust/src/layout.rs:186`, `build.rs` and stray files receive "no target, never extracted".
- Failure: Build scripts (`build.rs`) in Rust handle code generation, native C/C++ library bindings, platform detection, and supply-chain steps. Because sealmap skips them entirely, any architecture review, security audit, or dependency analysis performed via `sealmap` has a blind spot regarding build-time execution and build dependencies.
- Confidence: high
- Marked by authors: no

### F-06 — Untyped closure parameters fabricate false call edges and poison verification locks
- Topics: EXT-03.7, EXT-05.6
- Evidence: EXT-05.6 documents that closure parameters are never bound in the walker's environment (`collect.rs:1384`). When a method is called on a parameter, the resolver falls back to a workspace-wide name guess (`resolve.rs:866`), or if the parameter shadows an outer variable, it binds to the outer variable's type. In `tests/extract.rs:351`, a parameter named `conn` of type `u8` was matched to an internal database method `vacuum`.
- Failure: Hallucinated call edges are inserted into call graphs and sequence diagrams. Engineers or agents reviewing generated diagrams will approve and seal false architecture dependencies, committing invalid invariants into `seals.lock`.
- Confidence: high
- Marked by authors: yes (Marked as deliberate behavior / debt in EXT-05.6)

### F-07 — Fatal process abort on deeply nested syntax trees bypasses isolation guarantees
- Topics: EXT-01.1, EXT-01.3
- Evidence: `map_isolated` wraps collection in a 64 MiB stack with `catch_unwind` (`isolate.rs:38`-`56`), but EXT-01 acknowledges: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught." Syn costs ~40 KiB per nesting level in debug mode (`isolate.rs:3`-`7`).
- Failure: Highly nested expressions or recursive macro outputs trigger an operating-system stack overflow. Because stack overflow results in an uncatchable abort/SIGSEGV in Rust, the entire `sealmap` process crashes abruptly, killing the entire CI run without emitting diagnostics.
- Confidence: high
- Marked by authors: yes (Marked as a residual risk in EXT-01)

### F-08 — `stale --since` rejects multi-repository workspaces
- Topics: COR-03.5, COR-05.2
- Evidence: COR-03.5 documents that multi-repository workspaces are supported via `--repo` (`main.rs:76`-`86`, `sealmap/src/lib.rs:125`). However, COR-05.2 shows that `stale --since` explicitly refuses execution when `--repo` is passed (`main.rs:461`-`462`).
- Failure: Any organization using multi-repository configurations cannot use revision-based change detection (`sealmap stale --since REV` or `sealmap pack --diff REV`). CI pipelines tracking cross-repository changes fail immediately.
- Confidence: high
- Marked by authors: no

### F-09 — Trait map overview silently drops relations exceeding budget without warning
- Topics: COR-02.5, COR-06.1
- Evidence: In `_overview.md` generation, crate graphs, module graphs, and data models append comments indicating omitted elements when capped (`overview.rs:68, 125, 209`). However, the Trait Map takes the "first max_edges relations in key order, no omitted count" (`overview.rs:236`).
- Failure: In repositories with more than 300 trait implementations, trait relationships are silently omitted from the overview diagram with no marker, comment, or warning. This directly contradicts the design's stated guarantee that omissions are always explicitly noted rather than silently dropped (COR-02, COR-06.1).
- Confidence: high
- Marked by authors: no

### F-10 — Hardcoded path depth ceiling degrades deep module references to external stubs
- Topics: EXT-05.2
- Evidence: During path resolution in `resolve.rs:642`, if AST traversal exceeds a depth of 8 segments, the walker aborts resolution: "alt depth over 8 ... None, becomes a path id, External (resolve.rs:601)".
- Failure: In deeply nested modular codebases or schemas generated from nested namespaces (e.g., `crate::a::b::c::d::e::f::g::item`), valid internal calls are downgraded to unresolved `External` stubs. This distorts sequence diagrams, misattributes internal calls as third-party dependencies, and prevents accurate seal verification.
- Confidence: high
- Marked by authors: no

### F-11 — Downstream MSRV build failure via unpinned transitive dependency
- Topics: DEL-01.2
- Evidence: The workspace specifies `rust-version = 1.85` and declares `ignore = "0.4.23"` in `Cargo.toml:8, 35`. DEL-01.2 notes that `ignore 0.4.30` broke Rust 1.85 compatibility and lacks a `rust-version` field, causing Cargo's resolver to select 0.4.30 for downstream consumers building without `sealmap`'s lockfile.
- Failure: Any downstream consumer or integration environment that imports `sealmap-model` as a library on Rust 1.85 will fail to compile unless they manually pin `ignore` to 0.4.29.
- Confidence: high
- Marked by authors: yes (Marked as a caveat in DEL-01.2)

### F-12 — Deferred closure variables invert sequence order and drop execution call sites
- Topics: EXT-03.4
- Evidence: When a closure is assigned to a variable (`let g = || ...`), its body is walked at the `let` binding and rendered as an optional fragment there (`collect.rs:1267`, `1270`). When `g()` is subsequently invoked, the resolver drops the call step as an unknown local closure (`resolve.rs:839`-`840`).
- Failure: Sequence diagrams report closure logic as occurring at initialization rather than execution, while the actual call site shows no outgoing execution arrow. This inverts the temporal order of operations in sequence diagrams for event handlers, callbacks, and deferred tasks.
- Confidence: high
- Marked by authors: yes (Marked as debt in EXT-03)

### F-13 — Inability to extract, cite, or seal macro-generated symbols
- Topics: EXT-02.2, EXT-03.6
- Evidence: Items generated by macros are not expanded or extracted into symbols (`README.md:342`, `collect.rs:428`). Furthermore, macro invocations inside function bodies are never recorded as call steps (`flow.rs:160`), and calls inside macro bodies are only analyzed if the body happens to parse as standard Rust expressions or statements (`collect.rs:1366`-`1376`).
- Failure: In codebases built on macro-heavy frameworks (e.g., Tokio, Axum, Tonic, Diesel), generated endpoints, structs, and implementations cannot be cited, verified, or sealed. Calls dispatched via macros do not appear in flows, producing hollow diagrams for modern Rust services.
- Confidence: high
- Marked by authors: yes (Marked as a documented limitation in EXT-02 and EXT-03)

### F-14 — Relational model silently drops self-recursive function calls
- Topics: MOD-03.1
- Evidence: `Codebase::add_relation` enforces the invariant: "a relation from a symbol to itself is never stored" (`codebase.rs:91`).
- Failure: Recursive calls are stripped from the model's relation graph. Any external tooling, query, or architectural analysis relying on `Codebase.relations` to identify self-recursion, recursive loops, or execution cycles cannot detect self-recursive functions.
- Confidence: high
- Marked by authors: no

### F-15 — Zero-tolerance budget cliff causes hard failures in automated review workflows
- Topics: COR-06.3, DEN-01.5
- Evidence: Both `sealmap pack` and `Dense.slice` enforce strict binary size limits: "at exactly the budget the pack is returned, one byte under it is refused" (`tests/pack.rs:181`, `src/pack.rs:279`, `lib.rs:368`-`373`). There is no option for soft degradation or truncation.
- Failure: A trivial upstream commit that adds a single character or doc line pushing a pack 1 byte over `--budget` causes the command to abort with exit code 1. Automated PR review bots or CI pipelines will hard-fail, halting reviews until a human manually recalculates and increases budget parameters.
- Confidence: high
- Marked by authors: no

---

## Not judgeable from this material

1. **Subprocess injection safety in `GitTree::export`**: The material does not show whether `main.rs:608`-`622` constructs `git archive` and `tar -x` commands using argument vectors or passes raw strings through a system shell, leaving command injection and path traversal risks unverifiable.
2. **Concurrency and file-locking behavior**: The material does not disclose whether concurrent invocations of `sealmap seal sign` or `sealmap generate` against the same directory perform file locking, or if overlapping runs will corrupt `seals.lock`.
3. **Memory ceiling during full AST collection**: The material confirms Rayon stack sizing (64 MiB), but does not reveal heap allocation limits during parallel parsing of large workspaces, leaving OOM vulnerability under multi-gigabyte codebases unknown.
4. **Resilience of front-matter parsing**: The exact parser implementation for extracting `id:` and `sealed:` front-matter fields in `read_topics` (`main.rs:472`-`496`) is not shown; it is impossible to verify whether malformed YAML or crafted Markdown headers can cause panics.
5. **Collision risk of 128-bit fingerprints at scale**: The material confirms fingerprints are truncated to 16 bytes (128 bits) via BLAKE3 (`hash.rs:97`), but lacks empirical collision metrics across enterprise mono-repositories containing millions of distinct callables.
