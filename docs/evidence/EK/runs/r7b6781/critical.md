### F-01 — Unsanitized git revision execution and missing multi-repo diffing
- Topics: COR-05.2, COR-06.4, COR-03.5
- Evidence: In `main.rs:608-622`, `stale --since REV` and `pack --diff REV` export revisions using `archive REV:prefix piped into tar -x, temp dir`. The `REV` argument comes directly from CLI input. Furthermore, `main.rs:461-462` explicitly specifies: `ST->>ST: refuse with --repo`, while `main.rs:76-86` advertises `--repo` as a shared flag across all commands.
- Failure: Passing untrusted revision strings (such as in automated CI bot triggers) risks argument injection into `git archive`. Additionally, unpacking unchecked archives with `tar -x` into temporary directories risks path traversal via malicious symlinks. In multi-repo workspaces (a primary documented feature), running `sealmap stale --since` or `sealmap pack --diff` immediately crashes/refuses execution.
- Confidence: high
- Marked by authors: no

### F-02 — Destructive file and directory deletion during corpus generation
- Topics: COR-01.2, COR-03.3, COR-03.4
- Evidence: `crates/sealmap-corpus/src/document.rs:10-12` and `contract.rs:99, 138-141` state that `write` deletes any Markdown file whose YAML front matter opens with `---` and `sealmap:`, and subsequently deletes empty parent directories (`write deletes it and empty parents`).
- Failure: If `sealmap generate -o <DIR>` is pointed at an existing directory containing non-generated documentation or design notes that use `sealmap:` metadata, or if an orphaned file is cleaned up inside a directory tree, `write` purges those files and recursively removes their parent directories from disk without confirmation.
- Confidence: high
- Marked by authors: no

### F-03 — Deadlock on topic file renaming with no deletion mechanism
- Topics: COR-04.4, COR-04.6, COR-05.1, COR-05.3
- Evidence: COR-04.6 claims: "Replacing by file as well as by id means a renumbered topic moves its seal rather than leaving the old entry orphaned (`tests/seal.rs:543`)". However, the signing pipeline in `sign.rs:117-118` enforces: `SG->>LK: id already sealed for another file? refuse`. Meanwhile, `check.rs:259-260` halts on any file missing from disk: `topic file missing, Orphan, stop`.
- Failure: If an engineer renames a topic file (e.g., `01-architecture.md` to `02-architecture.md`) without changing its topic ID, `verify` immediately fails because `01-architecture.md` is an `Orphan`. Running `sealmap seal sign 02-architecture.md` is refused because the ID is already sealed under the old filename. Because the CLI lacks any command to remove or unseal entries (`COR-05.1`), and hand-editing `seals.lock` is rejected by `parse_canonical` (`lock.rs:234`), the workspace is permanently locked in a failing gate.
- Confidence: high
- Marked by authors: no

### F-04 — Prose mentions of symbol IDs break CI verification with no escape hatch
- Topics: COR-04.3, COR-04.4, DEL-02.6
- Evidence: COR-04.3 states that any Markdown code span starting with `sym:`, a manager, and a space is treated as a code citation (`topic.rs:331`). DEL-02.6 documents that running `sealmap verify` on sealmap's own codebase fails because `MER-02 writes an example id from a fixture crate (shop) as an inline code span, which reads as a citation of a symbol nobody sealed`, forcing the project to disable `verify` in CI (`ci.yml:9-84`).
- Failure: Any topic that documents example IDs, external IDs, or fixture symbols in backticks is classified as an `UnsealedCitation`, causing `sealmap verify` to exit 1. Because CommonMark backtick spans have no escaping mechanism, documentation cannot reference example IDs without breaking the CI seal gate.
- Confidence: high
- Marked by authors: yes (Marked in DEL-02.6)

### F-05 — Ambient checkout directory leakage corrupts symbol IDs and determinism
- Topics: DEL-01.3, EXT-01.2, MOD-01.2, COR-05
- Evidence: DEL-01.3 notes that codebase name resolution (`main.rs:393, 419-424`) falls back to the checkout directory name. In EXT-01.2, fallback crates without an explicit package manifest take this name (`layout.rs:143`), which forms the package root in global symbol IDs (`MOD-01.2`). COR-05 notes that checking seals with different settings makes symbols appear `Absent`.
- Failure: When `--name` is omitted, cloning a repository into differently named directories (standard in CI runners, Docker containers, and developer workstations) produces divergent symbol IDs. `sealmap verify` reports sealed symbols as `Absent`, failing builds on identical Git commits.
- Confidence: high
- Marked by authors: no

### F-06 — Review pack source windows silently truncate code while guaranteeing completeness
- Topics: COR-06, COR-06.2, DEN-01
- Evidence: COR-06 claims: "Nothing is truncated... It never quietly drops content." DEN-01 claims: "It never receives a silently shortened excerpt that looks complete." However, COR-06.2 reveals: `WI-->>SE: at most N lines from the first, label says when clipped (src/pack.rs:508-517)` via `--source-window` (`main.rs:206-232`).
- Failure: When generating review packs for audits or LLM ingestion, functions exceeding the line window are cut off at N lines. An auditor or model reviews an incomplete function body under the premise that sealmap review packs never truncate content.
- Confidence: high
- Marked by authors: no

### F-07 — Closure parameter shadowing binds calls to outer variable types
- Topics: EXT-05.6, EXT-03.7
- Evidence: EXT-05.6 documents that "a parameter that shadows an outer typed name keeps the outer type (`collect.rs:1348-1351`)". In addition, any unshadowed parameter is `Untyped`, and method calls on it fall back to workspace-wide `by_name_only` guessing (`tests/extract.rs:351`).
- Failure: In common Rust patterns where a closure parameter shadows an outer variable name (`let file = ...; items.iter().map(|file| file.read())`), the resolver binds `file.read()` to the outer variable's type rather than the inner parameter's type. For unshadowed parameters, method calls match unrelated single-instance methods across reachable crates (e.g. matching `u8` calls to `Database::vacuum`). This fabricates false call edges and misdirects review seals.
- Confidence: high
- Marked by authors: yes (Marked as debt in EXT-05.6)

### F-08 — Refactoring foreign trait implementations invalidates seals
- Topics: EXT-02, EXT-02.1
- Evidence: EXT-02 promises: "Reorganising methods across impl blocks or files, the most common refactor in a growing Rust codebase, changes no name and therefore invalidates no reviewed diagram." However, EXT-02.1 defines IDs for foreign implementations (`impl Trait for ForeignType`) as `module/impl#[SelfTy][Trait]m().` (`ids.rs:105`), which are anchored in the specific module containing the `impl`.
- Failure: Moving a foreign trait implementation or extension trait from one file to another changes its symbol ID. Existing seals citing that method are reported as `Absent`, failing CI builds despite the design promise that file reorganizations preserve IDs.
- Confidence: high
- Marked by authors: no

### F-09 — Silent omission of local closures and macro invocations from call flows
- Topics: EXT-03.4, EXT-03.6
- Evidence: EXT-03.4 states that calling a local closure or function pointer (`let g = ...; g(1)`) is "dropped as a local closure (`resolve.rs:840`)". EXT-03.6 states that a macro invocation is "never a call step (`flow.rs:160`)", and macros whose arguments do not parse as expression lists contribute nothing.
- Failure: Core application logic relying on callbacks, dispatch tables, handler closures, or routing macros (e.g. HTTP routing, custom logging, telemetry) is silently omitted from generated sequence diagrams and dense projections, producing false representations of system flow.
- Confidence: high
- Marked by authors: yes (Marked as debt in EXT-03)

### F-10 — Hardcoded path depth limit silently reclassifies internal calls as external dependencies
- Topics: EXT-05.2
- Evidence: EXT-05.2 specifies that if path segments exceed 8: `depth over 8? (resolve.rs:642) -> yes -> None: becomes a path id, External (resolve.rs:601)`.
- Failure: Code organized in module trees deeper than 8 segments (e.g., `crate::subsystem::subdomain::module::component::layer::handler::call()`) is silently aborted by `resolve.rs:642` and classified as an external dependency (`sym:extern ...`). This falsifies architecture boundaries and creates incorrect external lanes in sequence diagrams.
- Confidence: high
- Marked by authors: no

### F-11 — Trait map silently truncates relationships beyond budget without notice
- Topics: COR-02.5
- Evidence: COR-02.5 specifies that while crate graphs, module graphs, and data models emit comments indicating the count of omitted entities when caps are exceeded, the trait map (`overview.rs:230`) outputs the `first max_edges relations in key order, no omitted count (`overview.rs:236`)`.
- Failure: In large codebases exceeding 300 trait relationships, the overview diagram drops the remainder in alphabetical key order without any comment or indicator. Engineers and agents reviewing `_overview.md` make architectural decisions on an incomplete trait map assuming it is exhaustive.
- Confidence: high
- Marked by authors: no

### F-12 — Substring matching on "spawn" misclassifies synchronous calls as parallel execution
- Topics: EXT-03.3
- Evidence: EXT-03.3 specifies deferred closure classification via: `callee name contains spawn? (labels.rs:140) -> yes -> Parallel arm: spawned task (labels.rs:153)`.
- Failure: Calling methods such as `entity.despawn(|| ...)`, `worker.check_spawn(|| ...)`, or synchronous factory helpers matches `contains("spawn")` and renders the call inside a Mermaid `par` block. This misrepresents single-threaded synchronous code as multi-threaded concurrency in review diagrams.
- Confidence: high
- Marked by authors: no

### F-13 — Non-atomic two-phase signing leaves repository in unverified LockFault state
- Topics: COR-05.3, COR-04.4
- Evidence: In COR-05.3, `seal sign` writes the topic Markdown file first (`main.rs:679-680`) and `seals.lock` second (`main.rs:682`). In COR-04.4, `check.rs:383` checks: `pointer without entry, LockFault`.
- Failure: A process interruption (SIGINT, crash, or disk full) between the topic write and the lock write leaves the topic pointing to a nonexistent lock entry. Subsequent runs of `sealmap verify` report a fatal `LockFault`. There is no rollback, atomic directory transaction, or recovery mechanism provided.
- Confidence: high
- Marked by authors: no

### F-14 — Workspace manifest maintains unpinned dependency breaking downstream MSRV
- Topics: DEL-01.2, Cargo.toml
- Evidence: DEL-01.2 notes that `ignore 0.4.30` broke Rust 1.85 MSRV compliance. While `Cargo.lock` pins `0.4.29` internally, the published manifest `Cargo.toml:35` leaves the requirement at `ignore = "0.4.23"`.
- Failure: Downstream crates on Rust 1.85 importing published `sealmap` crates without using sealmap's lockfile will resolve `ignore 0.4.30` during standard dependency resolution, causing immediate compilation failure on the documented MSRV toolchain.
- Confidence: high
- Marked by authors: yes (Marked in DEL-01.2)

### F-15 — TOML parse errors suppress all code drift and security classification
- Topics: COR-04.4
- Evidence: COR-04.4 specifies: `VF->>LK: parse (check.rs:360); alt the lock does not parse -> VF-->>CL: one LockFault and nothing else (check.rs:363-367)`.
- Failure: While non-canonical formatting is inspected in full, any syntax error in `seals.lock` (such as Git merge conflict markers or corrupted TOML) causes `verify` to halt immediately and return only a single `LockFault`. All subsequent checks for broken contracts, absent symbols, unsealed citations, and code drift are suppressed.
- Confidence: high
- Marked by authors: no

---

## Not judgeable from this material

1. **AST Token Normalization Equivalence:** Whether `feed_trees` and `canonical` in `crates/sealmap-rust/src/fingerprint.rs` correctly prevent false contract changes across differing Rust editions, complex macro expansions, and all `rustfmt` configurations.
2. **Concurrent File Access and Locking:** Whether concurrent invocations of `sealmap` (e.g., parallel CI runners or concurrent agent processes) against `.sealmap/` or `seals.lock` perform OS-level file locking or suffer from write-clobbering and race conditions.
3. **Symlink Traversal and Security Boundaries:** Whether `SourcePath` normalization in `crates/sealmap-model/src/path.rs` validates against filesystem symlinks pointing outside the workspace root during physical directory traversal in `SourceSet::load_dir`.
4. **Stack Exhaustion on Deep Macro Recursion:** Whether the 64 MiB worker thread stack and the 1,024 expression depth cap in `crates/sealmap-rust/src/collect.rs` are sufficient to prevent uncatchable process aborts when parsing malicious or deeply generative macros.
5. **Real-world Context Budget Consumption:** The empirical token costs and failure rates of review pack generation (`crates/sealmap-corpus/src/pack.rs`) when processing large enterprise workspaces with thousands of cross-crate dependencies.
