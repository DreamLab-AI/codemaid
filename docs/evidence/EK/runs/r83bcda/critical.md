### F-01 — Proprietary source code disclosure via missing diagrams-only review pack mode
- Topics: DEL-02, DEL-02.1, COR-06
- Evidence: `docs/DESIGN.md:42` specifies a review pack mode holding diagrams alone for outside reviewers who should not see source code. DEL-02.1 confirms that while other pack features were built, this mode was not: `diagrams-only --review NOT BUILT` (`crates/sealmap-corpus/src/pack.rs:274`). `pack` unconditionally extracts and embeds source code windows for every resolved citation (`crates/sealmap-corpus/src/pack.rs:490`–`517`).
- Failure: An operator distributing review packs to external auditors, contractors, or third-party AI models under the assumption that review packs can redact source code will inadvertently disclose proprietary raw source files.
- Confidence: high
- Marked by authors: yes (marked as "NOT BUILT" in DEL-02.1)

### F-02 — Process crash via uncatchable stack overflow on deeply nested source files
- Topics: EXT-01, EXT-03.2
- Evidence: EXT-01 claims extraction "never fails a whole run because of one bad file: a file that does not parse, or that trips a bug, becomes a warning and a placeholder entry" (`crates/sealmap-rust/src/lib.rs:153`–`155`). However, EXT-01 admits in its text: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught" (`crates/sealmap-extract/src/isolate.rs:34`–`37`).
- Failure: A malformed, adversarial, or machine-generated Rust file containing deeply nested expressions causes an uncatchable thread stack overflow during parsing, crashing the entire extraction CLI or CI worker and bypassing panic-isolation guards.
- Confidence: high
- Marked by authors: yes (marked as "residual risk" in EXT-01)

### F-03 — Arbitrary directory and markdown deletion via generated corpus cleanup
- Topics: COR-03, COR-03.2, COR-03.3
- Evidence: `sealmap generate` write reconciliation classifies any markdown file in the target directory that starts with `---` and `sealmap:` as an orphaned generated document if it is not expected in the fresh corpus (`crates/sealmap-corpus/src/contract.rs:99`). COR-03.3 specifies: `Orphaned --> [*]: write deletes it and empty parents`.
- Failure: If an operator misconfigures the output flag `-o` to point to a root documentation directory, a workspace folder, or a directory where authored topics contain copies of generated front matter, `sealmap generate` will permanently delete those markdown files and recursively delete any parent directories that become empty.
- Confidence: high
- Marked by authors: no

### F-04 — Non-atomic seal signing leaves repository in unrecoverable CI failure state
- Topics: COR-04.6, COR-05.3
- Evidence: `seal sign` executes a non-atomic two-phase disk write: the topic file is updated with a `sealed:` pointer first (`crates/sealmap/src/main.rs:679`–`680`), and the canonical lock file is written second (`crates/sealmap/src/main.rs:682`). COR-05.3 notes: "a failure between the two writes leaves a pointer with no entry, which verify reports as a lock fault, so an interrupted sign can never pass the gate".
- Failure: An interrupted process (SIGKILL, process timeout, full disk, or power loss) after writing the topic file but before updating `seals.lock` leaves a dangling pointer on disk. All subsequent CI runs executing `sealmap verify` fail immediately on a lock fault until manual git intervention or lockfile repair occurs.
- Confidence: high
- Marked by authors: no

### F-05 — Non-canonical locks silently overwritten despite claimed integrity refusal
- Topics: COR-04, COR-05.3
- Evidence: COR-04 claims as a core guarantee that "a lock edited by hand is refused" and asserts byte-for-byte roundtrip invariants (`crates/sealmap-corpus/tests/seal.rs:554`). In direct contradiction, COR-05.3 documents that during signing, the "lock parsed leniently, a non-canonical lock is rewritten canonically (`crates/sealmap/src/main.rs:662`–`663`)".
- Failure: An operator or automated script that makes an invalid, non-canonical, or out-of-order edit to `seals.lock` will not have the action refused by `seal sign`. The tool silently accepts the hand-edited lock, rewrites it, and persists the changes, defeating the tamper-evident validation guarantee.
- Confidence: high
- Marked by authors: no

### F-06 — CI seal invalidation caused by checkout directory name variance
- Topics: COR-01, COR-05, EXT-01.1
- Evidence: Global symbol IDs embed the codebase name (`sym:cargo <name> . ...`). EXT-01.1 states that the codebase name is derived from `--name`, the dir, or the repo names (`crates/sealmap/src/main.rs:393`). COR-05 cautions: "a seal remembers functions by id, and the ids depend on how the code was read (whether tests were included, what the codebase is called when no manifest names it)".
- Failure: In repositories where no manifest specifies the codebase name, signing locally under a directory named `my-repo` produces symbols prefixed with `sym:cargo my-repo`. When checked out in a CI runner into `/workspace` or `/runner/work/repo/repo`, extraction defaults to the directory name `workspace`. Every cited symbol fails verification as `Absent`, causing catastrophic build failures across all seals.
- Confidence: high
- Marked by authors: no

### F-07 — Inherent method preference produces incorrect call resolution for trait dispatches
- Topics: EXT-05, EXT-05.1
- Evidence: EXT-05.1 enforces: "Invariant: inherent impls are registered before trait impls, so an inherent method wins a name lookup over a trait method (`crates/sealmap-rust/src/resolve.rs:557`–`559`)".
- Failure: When a type implements a trait method that shares a name with an inherent method, or in contexts where Rust trait scoping dictates that a trait method is called, `sealmap` unconditionally binds to the inherent method without running type checking. The resulting sequence diagrams and call graphs record false internal interactions.
- Confidence: high
- Marked by authors: no

### F-08 — Spurious call inferences generated by single-candidate workspace reachability
- Topics: EXT-05, EXT-05.4
- Evidence: When resolving method calls on untyped receivers (`resolve.rs:866`), the resolver falls back to `by_name_only` (`resolve.rs:954`). If a method name appears exactly once across all reachable workspace crates, it binds an `inferred` call edge (`crates/sealmap-rust/src/resolve.rs:954`–`968`).
- Failure: A method invoked on an untyped variable (e.g. from an external dependency or an unmodeled struct) that happens to share a name with a single reachable workspace function (e.g., `execute`, `cleanup`, `render`) will be bound as an internal call. The system synthesizes nonexistent dependencies and sequence arrows between unrelated components.
- Confidence: high
- Marked by authors: no

### F-09 — Total omission of macro-generated items and invocations from diagrams and seals
- Topics: EXT-02, EXT-03.6, EXT-06
- Evidence: EXT-02 establishes: "items that a macro generates are not seen, so they get no name and cannot be cited or sealed (`README.md:342`)". EXT-03.6 notes: "a macro whose body is neither an expression list nor a statement block contributes nothing; the macro invocation itself is never a call step (`crates/sealmap-model/src/flow.rs:160`, `crates/sealmap-rust/src/collect.rs:1366`–`1376`)".
- Failure: Frameworks relying heavily on declarative or procedural macros (e.g., web routing in Axum/Actix, RPC dispatchers, serialization wrappers) have their generated functions omitted from the model. CI cannot seal these critical pathways, and generated sequence diagrams omit their execution flows entirely.
- Confidence: high
- Marked by authors: yes (marked as "debt" in EXT-03)

### F-10 — Inverted execution order for stored closures and asynchronous callbacks
- Topics: EXT-03, EXT-03.2
- Evidence: EXT-03 documents that closure arguments are moved after calls, but warns: "A closure saved in a variable is drawn where it is written, not where it is called... Adopters should know the shapes it gets wrong, because a reviewer reading a diagram cannot see them".
- Failure: Sequence diagrams for event loops, deferred handlers, or task queues display callbacks executing sequentially during setup rather than when invoked later. Anyone reviewing the diagram for concurrency, state mutations, or lock contention is misled regarding the actual order of execution.
- Confidence: high
- Marked by authors: yes (marked as "debt" in EXT-03)

### F-11 — Default suppression of standard library and unknown receiver calls hides critical operations
- Topics: EXT-04, EXT-05.4
- Evidence: EXT-04 states: "By default calls into the standard library and calls on values of unknown type are left out... The cost of that density is that a reviewer never sees those calls (`crates/sealmap-extract/src/confidence.rs:1`–`7`)". EXT-05.4 confirms computed, returned, or unknown receiver calls are dropped (`crates/sealmap-rust/src/resolve.rs:874`).
- Failure: Operations critical to security and system stability—such as OS command execution (`std::process::Command`), filesystem manipulation (`std::fs`), network calls, and synchronization locking (`std::sync::Mutex::lock`)—are completely invisible on generated diagrams, creating an inaccurate impression that code performs no external I/O.
- Confidence: high
- Marked by authors: no

### F-12 — Silent truncation of sequence diagrams conflicts with completeness guarantees
- Topics: COR-02, COR-02.1, COR-06
- Evidence: COR-02 states that sequence diagrams enforce hard budgets: "A diagram over its message or edge budget is cut, with a note saying how much was left out and where the full list is... Long functions and hub crates are therefore always summarised (`crates/sealmap-corpus/src/sequence.rs:40`)". This contradicts the general guarantee promoted in COR-06 that the system "never quietly drops content".
- Failure: High-complexity functions (such as centralized state machine routers or core dispatch loops) have their calls truncated midway. Reviewers or AI agents analyzing the rendered Mermaid sequence diagrams miss trailing security checks, error recovery branches, or cleanup routines.
- Confidence: high
- Marked by authors: yes (marked as a "deliberate trade" in COR-02)

### F-13 — Unyielding byte-budget refusals cause automated CI and review pack pipeline outages
- Topics: DEN-01, COR-06
- Evidence: DEN-01 and COR-06 enforce zero-tolerance byte limits: "A slice over its byte budget is refused, never truncated (`crates/sealmap-dense/src/lib.rs:368`–`373`)"; "at exactly the budget the pack is returned, one byte under it is refused... Over budget, pack refuses (`crates/sealmap-corpus/src/pack.rs:278`–`282`)".
- Failure: Routine code additions that increase a function size by a single byte cause downstream `sealmap pack` or dense slicing operations to fail with hard errors rather than providing a truncated excerpt or warning. Automated review generation pipelines will crash unexpectedly until operators intervene to bump size thresholds.
- Confidence: high
- Marked by authors: no

### F-14 — Unverified production readiness due to lack of self-dogfooding on the seal gate
- Topics: DEL-02, DEL-02.2
- Evidence: DEL-02 documents that although the seal mechanism is implemented, "this repository has not sealed its own diagrams yet, so its CI does not run the seal gate (step 6) (`docs/DESIGN.md:308`)".
- Failure: The core production capability—preventing architectural drift in CI via `sealmap verify`—is not running on the repository that produces it. End-to-end edge cases involving git status, branch changes, and seal validation remain unexercised in production CI workflows.
- Confidence: high
- Marked by authors: yes (marked as "step 6" backlog in DEL-02.2)

### F-15 — Downstream MSRV compilation failure caused by unpinned transitive dependency
- Topics: DEL-01
- Evidence: DEL-01 states that while the repository builds on Rust 1.85 using its checked-in `Cargo.lock`, "A dependency (`ignore` 0.4.30) needs a newer Rust than it declares; inside this repository the lockfile avoids it, but a project that depends on the published crates without that lock can still pull it in on Rust 1.85 and fail to build".
- Failure: Any downstream consumer integrating `sealmap` library crates into a production pipeline running on the documented MSRV of Rust 1.85 will experience build failures as soon as standard dependency resolution selects `ignore` 0.4.30.
- Confidence: high
- Marked by authors: yes (marked as a "live caveat" in DEL-01)

---

## Not judgeable from this material

1. **Git workspace race conditions**: Whether concurrent runs of `sealmap pack --diff` or `sealmap stale --since` collide on `.git/index.lock` or interfere with untracked files when spawning git processes in shared build environments.
2. **Behavior on symlink loops and non-standard filesystems**: How path normalisation in `crates/sealmap-model/src/path.rs` behaves when encountering symbolic link cycles, junctions, or network filesystem latencies.
3. **Memory consumption under massive codebases**: Peak heap allocation when `sealmap-dense` or `sealmap-rust` builds whole-workspace AST tables and in-memory call trees on repositories with tens of thousands of files.
4. **Collision rate of truncated 128-bit fingerprints**: Whether merging `#[cfg]` variants via 16-byte BLAKE3 hashes creates practical collision risks across large multi-target codebases.
5. **Recovery semantics of partial write failures in directory generation**: Whether `sealmap generate` leaves orphaned temporary files or half-written markdown files if aborted during a batch write of hundreds of files.
