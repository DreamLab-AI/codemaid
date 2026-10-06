### F-01 — Arbitrary path traversal and host file overwrite via `tar -x` during git revision export
- Topics: COR-05.2, COR-06.4
- Evidence: In COR-05.2, `stale --since` and `pack --diff` export older git revisions by piping git archive into tar: `GT->>GI: archive REV:prefix piped into tar -x, temp dir (main.rs:614-622)`.
- Failure: When run against untrusted revisions, pull requests, or branches containing symlinks or path traversal elements, raw `tar -x` can follow symlinks or write outside the temporary directory, overwriting arbitrary files on the host system running CI or agent workflows. If `REV` or `prefix` is interpolated into a shell pipeline without strict sanitization, command injection is also possible.
- Confidence: medium (inferred from the invocation of raw `tar -x` into a temporary directory)
- Marked by authors: no

### F-02 — Denial of service via uncatchable stack overflow during syntax tree parsing
- Topics: EXT-01.3
- Evidence: EXT-01.3 claims extraction isolation via `catch_unwind` and 64 MiB stacks (`isolate.rs:52`), but acknowledges: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught." Syn costs ~40 KiB per nesting level (`isolate.rs:3-7`).
- Failure: Highly nested syntax trees (auto-generated code, deep macro expansions, parser combinators, or malicious inputs) exhaust the thread stack during `syn` parsing. Because stack overflows in Rust trigger fatal process aborts (SIGSEGV/abort) rather than standard panics, `catch_unwind` cannot catch them, terminating the entire process and halting CI or server execution.
- Confidence: high
- Marked by authors: no (authors acknowledge it as a "residual risk", but do not mark it as debt or defect)

### F-03 — Permanent repository lock fault caused by non-atomic signing sequence
- Topics: COR-04.6, COR-05.2
- Evidence: COR-05.2 documents the invariant: "a failure between the two writes leaves a pointer with no entry, which verify reports as a lock fault, so an interrupted sign can never pass the gate (`main.rs:680-682`, `check.rs:383`)." The CLI modifies the topic Markdown file on disk before writing `seals.lock`.
- Failure: If `seal sign` is interrupted (process kill, disk quota exhaustion, OOM, power loss) after writing the topic Markdown file but before completing the write to `seals.lock`, the repository on disk is left corrupted. The topic contains a dangling `sealed:` pointer to a nonexistent lock entry, causing `sealmap verify` to permanently fail CI with `LockFault` until manual remediation.
- Confidence: high
- Marked by authors: no (authors framed the lack of atomicity as a safety invariant)

### F-04 — Topic file renaming deadlocks the seal signing workflow
- Topics: COR-04.4, COR-04.6
- Evidence: In COR-04.6, `sign` checks `id already sealed for another file? refuse (sign.rs:117-118)` before dropping entries for the target path (`sign.rs:135`). COR-04 enforces that `seals.lock` must never be hand-edited (`docs/DESIGN.md:80-85`) and non-canonical edits are rejected with `LockFault` (`check.rs:372-373`).
- Failure: Renaming or moving a topic file on disk (e.g. `topic-a.md` to `topic-b.md`) prevents `seal sign` from signing `topic-b.md`: the command refuses because the topic ID is already registered in `seals.lock` under `topic-a.md`. Because `topic-a.md` no longer exists on disk, it cannot be unsealed via CLI, and manual lock editing is rejected by `verify`, wedging the seal workflow.
- Confidence: high
- Marked by authors: no

### F-05 — Contradiction between truncation-free review pack guarantee and source window line clipping
- Topics: COR-06.1, COR-06.2
- Evidence: COR-06 guarantees: "Nothing is truncated (`pack.rs:278-282`)" and "It never quietly drops content (`README.md:42-45`)". Conversely, COR-06.2 specifies: `WI-->>SE: at most N lines from the first, label says when clipped (src/pack.rs:508-517)`.
- Failure: Reviewers auditing code changes via review packs receive truncated source excerpts for any function exceeding N lines. Security defects, contract changes, or subtle bugs appearing after line N are omitted from the pack despite explicit claims that packs contain complete, untruncated evidence.
- Confidence: high
- Marked by authors: no

### F-06 — Foreign type implementations violate invariance of method IDs under impl reorganization
- Topics: EXT-02.1, EXT-02.3
- Evidence: EXT-02 promises: "The rule that matters most is that a method sits under the type that owns it, never under its impl block... splitting an impl or moving it to another file keeps every method id (`ids.rs:20-28`, `README.md:177-179`)." In contrast, EXT-02.3 shows that when an owner is outside the codebase, `ID-->>IM: module/impl anchor (ids.rs:110)`.
- Failure: When implementing an internal trait for an external type (e.g. `impl MyTrait for String`), moving the `impl` block between files or modules alters the method ID. This invalidates all seals and citations, violating the core promise that refactoring `impl` blocks preserves method identities.
- Confidence: high
- Marked by authors: no

### F-07 — Output directory compliance failure caused by undeletable orphaned non-markdown files
- Topics: COR-01.1, COR-03.1
- Evidence: COR-03 defines the invariant: "`write` deletes only Markdown files whose YAML front matter opens on the first line (`---`) and whose first key is the `sealmap: ` marker (`contract.rs:99`, `document.rs:153-155`)." COR-01.1 shows `generate` outputs JSON files (`_model.json`, `_index.json`).
- Failure: Disabling optional model emission or removing obsolete files leaves JSON files on disk that `write` refuses to delete because they are not Markdown files with the `sealmap:` header. `verify_against` continues to report them as `orphaned`, causing `sealmap generate --check` to permanently exit 1.
- Confidence: high
- Marked by authors: no

### F-08 — Non-associative fingerprint folding produces unstable hashes for multi-variant `cfg` symbols
- Topics: MOD-02.3
- Evidence: MOD-02.3 shows `Fingerprint::merge` folds `sig_hash` and `body_hash` iteratively using pairwise sorted hashing: `FP->>FP: sort the pair, keyed hash of both (hash.rs:139)`.
- Failure: For symbols with three or more target configurations (e.g. Unix, Windows, macOS, WASM), pairwise hashing is non-associative: `hash(sort(hash(sort(A, B)), C)) != hash(sort(hash(sort(A, C)), B))`. File traversal or discovery order changes alter the final fingerprint without any change to the code tokens, triggering false-positive seal breakages in CI.
- Confidence: high
- Marked by authors: no

### F-09 — Inferred call graph edges silently deleted when unrelated workspace methods are introduced
- Topics: EXT-05.5, DEN-01.6
- Evidence: EXT-05.5 states: "Two reachable candidates are a genuine ambiguity, and no edge is guessed; the call stays `sym:? name` (`resolve.rs:954-968`, `tests/resolver_den01.rs:186`)."
- Failure: If an inferred call resolves because a method name is unique across reachable crates, adding a same-named method to any reachable crate creates an ambiguity. The resolver drops the call edge to external/unknown. If that was the function's only call, its sequence diagram and dense tree are deleted silently without warnings or errors.
- Confidence: high
- Marked by authors: no

### F-10 — Complete omission of seal gate verification from the project's own CI pipeline
- Topics: DEL-02, COR-06.4, MOD-02
- Evidence: DEL-02 states: "this repository has not sealed its own diagrams yet, so its CI does not run the seal gate (step 6)." COR-06.4 and MOD-02 corroborate that sealmap's own topic files have no seals.
- Failure: The core capability of the software—automated CI gating via `sealmap verify` to prevent diagram drift—is not dogfooded. Regressions in lock verification, classification logic, or canonical lock checking will not be caught by internal CI.
- Confidence: high
- Marked by authors: yes (authors marked this as an incomplete roadmap item: step 6)

### F-11 — Multi-repository configurations unsupported by differential analysis commands
- Topics: COR-05.2, COR-03.1
- Evidence: COR-05.2 states: `ST->>ST: refuse with --repo (main.rs:461-462)` for `stale --since`. In COR-06, `pack --diff` shares `Ctx::model_at` (`main.rs:460-466`), which rejects multi-repo options.
- Failure: Deployments managing codebases distributed across multiple repositories cannot run `sealmap stale --since` or `sealmap pack --diff`. The CLI commands abort immediately, rendering change-based CI gates unusable in multi-repo architectures.
- Confidence: high
- Marked by authors: no

### F-12 — Downstream MSRV compilation failure on Rust 1.85 due to unpinned `ignore` dependency
- Topics: DEL-01.2
- Evidence: DEL-01.2 details that `ignore` 0.4.30 fails to build on Rust 1.85, while `Cargo.toml` specifies `ignore = "0.4.23"`. The workspace avoids failure only via `Cargo.lock` pinning 0.4.29 (`ci.yml:46`).
- Failure: Any downstream consumer incorporating sealmap crates into their project on the advertised MSRV of Rust 1.85 will resolve `ignore` 0.4.30 and fail to compile during standard `cargo build`.
- Confidence: high
- Marked by authors: yes (authors marked this as "The ignore 0.4.30 problem")

### F-13 — Unparsed descriptor appending in `child()` risks corrupted model serialization
- Topics: MOD-01.3, MOD-01.5, MOD-03.4
- Evidence: MOD-01.5 states: "`child()` of Db with method get... text plus written descriptor (`sym.rs:583`). Neither runs the parser." MOD-01.3 dictates that canonical representation requires passing round-trip parsing (`sym.rs:441`). MOD-03.4 notes `from_json` strictly verifies canonical IDs (`codebase.rs:58`).
- Failure: Constructing a child ID with an invalid, unescaped, or non-canonical descriptor appends text directly without validation. When persisted to `_model.json` or `_index.json`, subsequent deserialization by `sealmap-model::from_json` fails on the unparseable ID, corrupting the generated model cache.
- Confidence: medium (inferred)
- Marked by authors: no

### F-14 — Unrecorded extraction settings produce silent seal verification failures
- Topics: COR-04.6, COR-05.1
- Evidence: COR-05 notes: "Checking a seal with different settings from the ones it was signed with makes sealed functions look absent." COR-04.6 reveals `seals.lock` records symbol hashes, topic hash, and reviewer, but records no metadata regarding `--tests`, `--name`, or repo options (`sign.rs:137-140`).
- Failure: If a seal is signed in an environment that included tests or specified a custom codebase name, but CI executes `sealmap verify` with default flags, all sealed symbols fail resolution and are classified as `Absent`. CI fails unconditionally with misleading drift reports.
- Confidence: high
- Marked by authors: no

### F-15 — Inconsistent release and publication state across documentation topics
- Topics: DEL-01.1, DEL-02.1
- Evidence: DEL-01 claims: "None of the 0.2 crates in this tree is published yet (`README.md:364-367`)." DEL-02 contradicts this: "Step 3 closed with the 0.2.0 publish (`docs/DESIGN.md:302-303`)" and "Closed in the 0.2.0 release: the workspace is at 0.2.0 (`Cargo.toml:6`), so a lock signed by this tree names sealmap 0.2.0... (`lock.rs:21`)."
- Failure: Pipeline maintainers cannot rely on version string semantics; `seals.lock` stamps generator as `sealmap 0.2.0`, yet downstream consumers cannot pull published `0.2.0` crates from crates.io, creating build and lock verification confusion across environments.
- Confidence: high
- Marked by authors: no

## Not judgeable from this material

1. **Path and argument sanitization in `GitTree::export` (`crates/sealmap/src/main.rs:614-622`):** Whether `REV` and `prefix` arguments are passed directly as array arguments to `std::process::Command` or interpolated into a shell invocation (`sh -c`), and whether `tar -x` includes options to prevent path traversal, follow symlinks, or write through existing links.
2. **Syntax tree recursion limits in `crates/sealmap-rust/src/collect.rs`:** Whether `syn` is configured with parse depth limits before building ASTs, or if deeply nested expressions can force an uncatchable thread stack overflow before reaching the flow walker's internal 1,024 depth cap.
3. **Line ending enforcement in `crates/sealmap-corpus/src/seal/lock.rs`:** Whether lock canonicality checking normalizes CRLF line endings to LF, or whether a git checkout with Windows line endings unconditionally causes `sealmap verify` to report `LockFault`.
4. **File locking on `seals.lock` (`crates/sealmap/src/main.rs:680`):** Whether concurrent invocations of `seal sign` use advisory filesystem locks, or whether concurrent signing runs can race and clobber entries.
5. **Inherent versus trait method resolution in `crates/sealmap-rust/src/resolve.rs`:** Whether the resolver's preference for inherent methods over trait methods accounts for trait methods brought into scope via glob imports across distinct packages without a full type-checker.
