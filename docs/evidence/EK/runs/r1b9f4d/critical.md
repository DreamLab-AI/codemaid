### F-01 — Command/argument injection and archive traversal via unvalidated git revision
- Topics: COR-05.2, COR-06.4, DEL-02.3
- Evidence: `crates/sealmap/src/main.rs:608`-`622` spawns `git archive REV:prefix` piped into `tar -x` inside a temporary directory to create comparative models (`GitTree export`). The `REV` argument comes directly from CLI input (`stale --since REV` or `pack --diff REV`) without documented sanitisation.
- Failure: In an automated CI webhook, PR review bot, or multi-tenant agent harness where `REV` is taken from untrusted branch names, commit refs, or PR parameters (e.g. `--output=/...`, `--remote=...`, or ref names containing flags), an attacker can inject options into `git archive` or unpack malicious archives (e.g. symlink traversal or path escapes via `tar -x`), leading to arbitrary file write or host compromise.
- Confidence: medium (inferred from process pipeline description in `main.rs:614`-`622` and lack of escaping specification)
- Marked by authors: no

---

### F-02 — Arbitrary file and directory deletion during directory reconciliation
- Topics: COR-03.3, COR-01.2
- Evidence: `crates/sealmap-corpus/src/contract.rs:99` and `crates/sealmap-corpus/src/document.rs:153`-`155` specify that `write` detects orphaned Markdown files and deletes them along with any empty parent directories. A file is deemed generated and subject to deletion solely if its front matter opens with `---` on the first line and the first key is `sealmap:`.
- Failure: If a user runs `sealmap generate -o <DIR>` and misconfigures `<DIR>` to point to a repository root, documentation folder, or existing documentation site that contains valid Markdown files starting with a `sealmap:` metadata key, `write` will permanently delete all such files not present in the new generation run, and recursively delete their parent directories.
- Confidence: high
- Marked by authors: no

---

### F-03 — Non-atomic signing writes leave corrupted, un-passable state on interruption
- Topics: COR-05.3, COR-04.6
- Evidence: `crates/sealmap/src/main.rs:679`-`682` performs two separate file writes during `seal sign`: first the topic file is overwritten on disk with the new `sealed:` pointer, and only then is `seals.lock` written canonically. COR-05.3 explicitly notes: "a failure between the two writes leaves a pointer with no entry, which verify reports as a lock fault".
- Failure: If the process is terminated (SIGKILL, container preemption, power loss, OOM) after updating the topic Markdown file but before writing `seals.lock`, the repository enters an unrecoverable corrupted state. `sealmap verify` fails closed with a `LockFault` on every subsequent CI run, requiring manual developer intervention to revert or re-seal.
- Confidence: high
- Marked by authors: no

---

### F-04 — Process crash on deep expression nesting due to uncatchable stack overflow
- Topics: EXT-01.3
- Evidence: `crates/sealmap-extract/src/isolate.rs:38`-`56` runs file parsing in a Rayon pool with 64 MiB stacks under `catch_unwind`. EXT-01 admits: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught." Furthermore, if the Rayon pool cannot be initialized, it falls back to a single scoped thread.
- Failure: Deeply nested macros, AST expressions, or recursion in generated or complex code bypasses Rust's `catch_unwind`, triggering an unrecoverable `SIGSEGV`/stack overflow abort that kills the entire host process or daemon, taking down the surrounding service or CI job.
- Confidence: high
- Marked by authors: yes (Residual risk in EXT-01)

---

### F-05 — Variable shadowing in closures resolves against outer type, generating false call edges
- Topics: EXT-05.6, EXT-03.4
- Evidence: `crates/sealmap-rust/src/collect.rs:1348`-`1351` clones the environment for closures without binding closure parameters (`crates/sealmap-rust/tests/extract.rs:392`-`393`: "A closure parameter is untyped, as before"). EXT-05.6 shows: "a parameter that shadows an outer typed name keeps the outer type" and calls are resolved against the outer type (`crates/sealmap-rust/tests/extract.rs:351`).
- Failure: When a closure parameter shadows an in-scope variable name (e.g. `let conn = DBConnection::new(); items.iter().for_each(|conn| conn.flush())`), method calls on the closure argument are resolved against the *outer* type (`DBConnection`) rather than remaining untyped or external. The generated call graphs, sequence diagrams, and architecture models report completely spurious internal interactions.
- Confidence: high
- Marked by authors: yes (Debt / noted in EXT-05.6)

---

### F-06 — Local closures evaluated at definition site, corrupting diagram sequence and deleting call sites
- Topics: EXT-03.4, EXT-03.2
- Evidence: `crates/sealmap-rust/src/collect.rs:965` and `collect.rs:1267`-`1270` walk a closure assigned to a local variable immediately at the `let` binding, emitting its calls as an `Optional` block. Subsequent invocations of the closure variable are dropped via `crates/sealmap-rust/src/resolve.rs:839`-`840`.
- Failure: If a function initializes a closure early but invokes it conditionally, in a loop, or after other side effects, sequence diagrams show the closure's operations executing at variable declaration. The actual invocation sites are deleted. Reviewers and auditing tools receive an inverted and structurally inaccurate model of the execution flow.
- Confidence: high
- Marked by authors: yes (Debt / noted in EXT-03.4)

---

### F-07 — Prose documentation examples formatted as `sym:` IDs break CI verification
- Topics: COR-04.3, COR-04.4, DEL-02.6
- Evidence: `crates/sealmap-corpus/src/seal/topic.rs:182` and `topic.rs:331` treat any CommonMark code span starting with `sym:`, a manager, and a space as an active citation. DEL-02.6 confirms that running `sealmap verify` on sealmap's own codebase failed with exit 1 because topic MER-02 used an illustrative fixture ID (`` `sym:cargo shop . store/` ``) in inline backticks, which was classified as an `UnsealedCitation`.
- Failure: Any documentation or ADR explaining symbol naming conventions, syntax, or external examples using standard Markdown code formatting immediately breaks CI verification. Teams are forced to either distort documentation syntax or leave CI gates disabled (which sealmap itself currently does, per `ci.yml:9`-`84`).
- Confidence: high
- Marked by authors: no

---

### F-08 — False-positive rename detection on functions with identical trivial bodies
- Topics: COR-04.5, MOD-02.5
- Evidence: `crates/sealmap-corpus/src/seal/check.rs:158` and `check.rs:191`-`199` match vanished symbols (`Absent`) to candidates of the same kind whose `body_hash` equals the sealed `body_hash`. MOD-02.5 confirms `body_hash` excludes the symbol's name.
- Failure: Trivial functions (e.g. `fn is_valid(&self) -> bool { true }`, empty constructors, forwarding methods, or stub implementations) share identical `body_hash` values across a crate. When one such function is deleted or refactored, the seal checker reports unrelated existing functions as rename candidates, misinforming reviewers and hiding the actual deletion.
- Confidence: high
- Marked by authors: no

---

### F-09 — Loose name-guessing binds arbitrary untyped receivers to unrelated workspace methods
- Topics: EXT-05.4, EXT-05.5, DEN-01.6
- Evidence: `crates/sealmap-rust/src/resolve.rs:866` and `resolve.rs:954`-`968` fall back to `by_name_only` on untyped receivers. If a method name is not on a blacklist of ~125 common names and appears exactly once across reachable workspace crates, it is inferred as an internal call.
- Failure: Any untyped local variable (e.g. from dynamic data, third-party libraries without explicit types, or deserialized values) calling a domain method (e.g. `.dispatch()`, `.authenticate()`, `.cleanup()`) is bound to an arbitrary internal struct that happens to implement that method name. Inferred edges and sequence diagrams will depict calls between components that have zero runtime relationship.
- Confidence: high
- Marked by authors: no

---

### F-10 — Total omission of macro-generated code and custom macro call paths
- Topics: EXT-02.2, EXT-03.6, EXT-06.3
- Evidence: EXT-02 notes that macro-generated items are invisible (`README.md:342`), receiving no ID and being impossible to cite or seal. `crates/sealmap-rust/src/collect.rs:1369`-`1376` skips macro bodies entirely unless they parse as comma-separated expressions or statements, and the macro invocation itself is never recorded as a call.
- Failure: In modern Rust codebases heavily reliant on macros (e.g. `tokio::select!`, `tracing::instrument`, RPC frameworks, custom derive/domain DSLs), major execution flows, spawned tasks, and generated endpoints are silently omitted. The generated models and review packs represent only hand-written syntactic constructs, creating a false sense of comprehensive architectural auditing.
- Confidence: high
- Marked by authors: yes (Debt / noted in EXT-03.6)

---

### F-11 — Multi-repository workspaces cannot use staleness detection or review diffing
- Topics: COR-05.2, COR-06.4, DEL-02.3
- Evidence: `crates/sealmap/src/main.rs:461`-`462` explicitly rejects runs combining `--since` with `--repo`: "refuse with --repo". `sealmap pack --diff` delegates directly to `model_at` / `stale` logic (`main.rs:705`).
- Failure: Any enterprise or monorepo environment utilizing sealmap's multi-repository loading feature (`--repo`) is blocked from running `sealmap stale --since` and `sealmap pack --diff`. CI pipelines cannot detect stale seals or generate PR-scoped review packs across repository boundaries.
- Confidence: high
- Marked by authors: no

---

### F-12 — Monolithic topics over budget cause total sharding failure and block review generation
- Topics: COR-06.3, COR-06.1
- Evidence: `crates/sealmap-corpus/src/pack.rs:279` and `pack.rs:308`-`318` implement `pack --shard` by allocating whole topic sections into files. If a single topic's section exceeds the budget on its own, `shard` aborts with `TopicOverBudget` (`crates/sealmap/src/main.rs:736`-`743`).
- Failure: When a complex topic citing many symbols or large source spans exceeds the review budget, `pack --shard` fails completely and outputs zero packs. Because the tool refuses to truncate or sub-shard individual topics, automated CI packaging or review generation fails until the budget is manually increased or the topic is rewritten.
- Confidence: high
- Marked by authors: no

---

### F-13 — Non-deterministic codebase naming breaks cross-environment hashing and caching
- Topics: DEL-01.3, EXT-01.2, COR-03.5
- Evidence: DEL-01.3 shows that determinism is broken by default naming: "NM -.->|leaks into output| BY; codebase name: --name, else the checkout directory's name (main.rs:393, main.rs:419-424)". The codebase name forms the root of symbol IDs (`sym:cargo <name> .`) and fallback crate names.
- Failure: The core marketing promise of byte-identical output across environments fails unless `--name` is explicitly passed. In CI environments using variable directory paths (e.g. GitHub Actions runner paths `/home/runner/work/repo/repo` vs local developer `/home/user/code/repo`), generated IDs, index files, and content hashes differ, invalidating remote caches and causing false drift alarms.
- Confidence: high
- Marked by authors: no

---

### F-14 — Missing dependency upper bound breaks MSRV 1.85 for downstream consumers
- Topics: DEL-01.2, Cargo.toml
- Evidence: `Cargo.toml:35` specifies `ignore = "0.4.23"`. `ignore 0.4.30` requires a newer Rust compiler but does not declare `rust-version`. While internal CI passes because `Cargo.lock` pins `ignore 0.4.29` (`.github/workflows/ci.yml:46`), downstream consumers building without a lockfile pull `0.4.30` on Rust 1.85.
- Failure: Any downstream application incorporating `sealmap` crates as dependencies and compiling against the advertised MSRV of Rust 1.85 fails to compile out of the box during dependency resolution.
- Confidence: high
- Marked by authors: yes (Marked as "The ignore 0.4.30 problem" in DEL-01.2)

---

### F-15 — Strict schema version gating with zero backwards/forwards compatibility
- Topics: MOD-03.4, MOD-03.1
- Evidence: `crates/sealmap-model/src/codebase.rs:47`-`58` probes `schema_version`. If the version is not strictly `2`, it returns `ModelJsonError.Version` and aborts. `crates/sealmap-model/src/lib.rs:106`-`111` notes that schema v1 has no reader. Furthermore, every `SymbolId` is strictly parsed against the canonical parser during JSON deserialization.
- Failure: Stored `_model.json` artifacts from previous schema versions cannot be loaded, migrated, or compared against current versions. Upgrading sealmap instantly invalidates all cached artifacts and external integrations relying on older JSON models.
- Confidence: high
- Marked by authors: no

---

## Not judgeable from this material

1. **Subprocess escaping in `git archive` execution**: Whether `crates/sealmap/src/main.rs:608`-`622` passes `REV` via a direct shell string or uses safe `std::process::Command` argv vectors cannot be verified without inspecting the implementation in `main.rs`.
2. **Memory consumption and OOM behavior on massive codebases**: Whether the BFS glob-resolution tables (`resolve.rs:485`-`555`) and Rayon thread pool with 64 MiB stacks trigger out-of-memory aborts on codebases exceeding 50,000 files cannot be determined from the benchmarks cited.
3. **Thread safety and re-entrancy of the library facade**: Whether `sealmap::Options` and extraction pipelines are safe to run concurrently across multiple threads in a long-lived service daemon, or if they rely on process-level globals/current working directory state.
4. **Behavior on symlink loops and sparse git checkouts**: How `SourceSet::load_dir` (`source.rs:154`-`183`) and `GitTree export` (`main.rs:608`) behave in the presence of circular filesystem symlinks or Git sparse-checkout configurations.
5. **Node/Puppeteer rendering sandbox vulnerabilities**: The security posture and resource consumption of the headless Chromium validator script (`tools/validate-mermaid.mjs`) when fed untrusted, adversarial Mermaid diagram inputs in production CI.
