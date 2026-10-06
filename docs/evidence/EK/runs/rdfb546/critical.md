### F-01 — Command option injection and unvalidated archive extraction in git integration
- Topics: COR-05.2, COR-06.4
- Evidence: `stale --since` and `pack --diff` execute `archive REV:prefix piped into tar -x, temp dir` (`crates/sealmap/src/main.rs:614`–`622`). `REV` is taken from user/CLI input and passed into `git archive` and `tar`.
- Failure: Passing an attacker-controlled or branch-derived revision string (e.g. `--output=...` or leading dash flags) injects options into `git archive`. Piping unvalidated archive streams into `tar -x` inside a temporary directory risks arbitrary file overwrite or directory traversal if the tree contains malicious paths or symlinks.
- Confidence: High (inferred potential for flag injection depending on whether arguments are passed as argv arrays without `--` delimiters).
- Marked by authors: No

### F-02 — Non-atomic signing leaves repository in a permanently broken CI gate state
- Topics: COR-04.6, COR-05.2
- Evidence: `seal sign` performs two sequential mutations: it updates the topic Markdown file with a `sealed:` pointer (`crates/sealmap-corpus/src/seal/sign.rs:133`), then updates `seals.lock` (`crates/sealmap/src/main.rs:680`–`682`). The material acknowledges: "a failure between the two writes leaves a pointer with no entry, which verify reports as a lock fault, so an interrupted sign can never pass the gate" (`crates/sealmap/src/main.rs:680`–`682`, `crates/sealmap-corpus/src/seal/check.rs:383`).
- Failure: A crash, power loss, SIGINT, or disk exhaustion occurring between the Markdown file write and the lockfile write corrupts workspace integrity. The topic file has a pointer without a matching lock entry. Every subsequent run of `sealmap verify` immediately fails with `LockFault`, hard-blocking CI pipelines until an engineer manually edits or reverts files.
- Confidence: High
- Marked by authors: Yes (noted as an invariant, but unhandled as a crash recovery defect)

### F-03 — Destructive file deletion in `contract::write` without safe directory isolation
- Topics: COR-01.1, COR-03.1
- Evidence: `sealmap generate` brings a directory into compliance by writing and deleting files (`crates/sealmap/src/main.rs:323`). It deletes any file whose YAML front matter starts with `---` followed by `sealmap:` (`crates/sealmap-corpus/src/contract.rs:99`, `crates/sealmap-corpus/src/document.rs:10`–`12`, `153`–`155`).
- Failure: If a user specifies an output directory (`-o`) that points to an existing documentation root, a shared directory, or a directory containing files from another generator that uses `sealmap:` metadata, `sealmap` permanently deletes all files not present in its current in-memory corpus without a dry-run confirmation, recycle bin, or rollback mechanism.
- Confidence: High
- Marked by authors: No

### F-04 — MSRV guarantee broken for all downstream consumers on Rust 1.85
- Topics: DEL-01.2
- Evidence: `Cargo.toml:8` declares MSRV `1.85` for every crate. However, `Cargo.toml:35` declares a dependency requirement on `ignore` of `0.4.23`. `Cargo.lock:264` pins `ignore 0.4.29`, but downstream crates consuming `sealmap` as a library do not use its `Cargo.lock`. `ignore 0.4.30` declares no `rust-version`, so Cargo's MSRV resolver selects `0.4.30`, which fails to compile on Rust 1.85 (`ci.yml:36`–`38`, `Cargo.toml:32`–`34`).
- Failure: Any downstream consumer attempting to use the published crates on the stated MSRV of Rust 1.85 will experience build failures during `cargo build` unless they manually discover the comment in sealmap's manifest and pin `ignore` in their own lockfile.
- Confidence: High
- Marked by authors: Yes (Debt / known issue documented in manifest and CI comments)

### F-05 — Syn stack exhaustion causes uncatchable process abort in CI
- Topics: EXT-01.1, EXT-01.3
- Evidence: The parser isolates file collection inside a Rayon thread pool with 64 MiB stacks (`crates/sealmap-extract/src/isolate.rs:52`). However: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught" (`EXT-01`, `crates/sealmap-extract/src/isolate.rs:3`–`7`, `docs/DESIGN.md:232`).
- Failure: Processing deeply nested macros, complex trait generics, or machine-generated Rust files causes a process crash via stack overflow (`SIGSEGV` on the guard page). Because Rust cannot catch stack overflows with `catch_unwind`, the entire `sealmap` process dies immediately, aborting extraction and failing CI jobs with no diagnostics or partial results.
- Confidence: High
- Marked by authors: Yes (Debt / residual risk noted in EXT-01)

### F-06 — Contradiction on published status and omission of seal verification from sealmap's own CI
- Topics: DEL-01, DEL-02
- Evidence: DEL-01 claims: "None of the 0.2 crates in this tree is published yet; the README's roadmap puts the first 0.2 crates.io release after step 3" (`README.md:364`–`367`). In contrast, DEL-02 claims: "Step 3 closed with the 0.2.0 publish (`docs/DESIGN.md:302`-`303`)... Closed in the 0.2.0 release: the workspace is at 0.2.0 (`Cargo.toml:6`)". Furthermore, DEL-02 admits: "this repository has not sealed its own diagrams yet, so its CI does not run the seal gate (step 6)."
- Failure: The release state of the project is contradictory in documentation. Crucially, the core product promise—gating CI on cryptographic topic seals (`sealmap verify`)—is not dogfooded in the repository's own delivery pipeline. Production adopters are running a gate mechanism that the authors do not validate on themselves.
- Confidence: High
- Marked by authors: Yes (Drift / unfinished roadmap step)

### F-07 — Spooky action at a distance: unrelated code changes silently break call graphs without seal invalidation
- Topics: EXT-05.5, DEN-01.6, MOD-02.3
- Evidence: The resolver makes an inferred call connection for unknown receivers only if the method name is unique among reachable workspace crates (`crates/sealmap-rust/src/resolve.rs:954`–`968`). If a second method of the same name is added to any reachable crate, candidate count exceeds one and the edge is dropped to `sym:? name` (`crates/sealmap-rust/src/resolve.rs:968`). Furthermore, `flow_hash` was deliberately omitted from symbol fingerprints (`docs/DESIGN.md:130`, `crates/sealmap-model/src/symbol.rs:209`–`212`).
- Failure: Adding an unrelated method in a dependency or sibling crate causes an existing call edge in a separate crate to disappear. Because symbol fingerprints only hash signatures and bodies (`sig_hash`, `body_hash`), this silent loss of call graph connectivity does *not* invalidate the caller's seal or trigger `sealmap verify` or `stale`. Sequence diagrams and dense projections silently degrade without alerting reviewers.
- Confidence: High
- Marked by authors: No

### F-08 — Blindness to macros causes silent omissions in call graphs, flows, and seals
- Topics: EXT-02, EXT-03, EXT-06
- Evidence: "items that a macro generates are not seen, so they get no name and cannot be cited or sealed (`README.md:342`)". "A macro invocation is never drawn as a call of its own, only the calls inside its arguments, and only when those arguments look like ordinary expressions" (`EXT-03`). Formatter rewrites inside macro arguments change fingerprints because token normalization is bypassed (`crates/sealmap-rust/src/fingerprint.rs:1`–`30`).
- Failure: In codebases relying on standard Rust macros (`tokio::select!`, `tracing::instrument`, RPC route macros, builder macros), control flow and invoked calls are omitted from sequence diagrams and dense projections. Items produced by macros cannot be referenced by `SymbolId` or sealed in `seals.lock`, creating massive blind spots in architecture enforcement.
- Confidence: High
- Marked by authors: Yes (Debt noted in EXT-02, EXT-03, EXT-06)

### F-09 — Strict byte-for-byte canonicality makes git conflict resolution in `seals.lock` fail CI
- Topics: COR-04.4, COR-05.1
- Evidence: `verify` checks whether `seals.lock` matches its canonical formatting byte-for-byte; if not canonical, it issues a `LockFault` (`crates/sealmap-corpus/src/seal/check.rs:372`–`373`, `crates/sealmap-corpus/tests/seal.rs:387`). "a lock edited by hand is refused" (`COR-04`).
- Failure: When multiple developers branch and seal topics concurrently, merging branches creates git conflicts in `seals.lock`. Standard manual git conflict resolution will almost never match the private, whitespace-exact canonical ordering of `sealmap_corpus::seal`. Every merged branch with resolved conflicts causes `sealmap verify` to fail with `LockFault`, breaking CI until `seal sign` is re-run for all topics.
- Confidence: High
- Marked by authors: No

### F-10 — Extraction flags omitted from `seals.lock` cause false-positive verification failures
- Topics: COR-05, COR-05.1
- Evidence: `SymbolId` values depend on extraction parameters: "whether tests were included, what the codebase is called when no manifest names it. Checking a seal with different settings from the ones it was signed with makes sealed functions look absent" (`COR-05`, `crates/sealmap/src/main.rs:439`). `seals.lock` stores generator version, topic hash, and symbol hashes, but does not record the extraction flags (`--tests`, `--repo`, `--name`).
- Failure: If a topic is signed in a development environment that included `--tests` or an implicit `--name`, and CI runs `sealmap verify` with default settings (or vice versa), the cited symbols fail to resolve as global IDs. `verify` reports them as `Absent` or `UnsealedCitation`, failing the CI gate despite zero code changes.
- Confidence: High
- Marked by authors: Yes (Tension / caution noted in COR-05)

### F-11 — Pairwise CFG twin folding produces order-dependent fingerprints
- Topics: MOD-02.3
- Evidence: When multiple definitions share an ID, `add_symbol` folds them via `Fingerprint::merge`: "sort the pair, keyed hash of both" (`crates/sealmap-model/src/codebase.rs:74`–`75`, `crates/sealmap-model/src/hash.rs:138`–`139`).
- Failure: For symbols with three or more conditional compilation configurations (e.g., `#[cfg(unix)]`, `#[cfg(windows)]`, `#[cfg(target_arch = "wasm32")]`), pairwise iterative folding (`fold(fold(A, B), C)`) is non-associative. If file discovery order or path sorting differs across platforms or runs, the resulting merged `sig_hash` and `body_hash` change, breaking cross-platform determinism and generating spurious seal invalidations.
- Confidence: Medium (inferred from pairwise merge semantics in `codebase.rs:74` and `hash.rs:139`).
- Marked by authors: No

### F-12 — Zero-tolerance budget enforcement causes hard failures in review pack pipelines
- Topics: COR-06, DEN-01.5
- Evidence: In `sealmap-dense` and `pack`, slices and packs over budget are strictly refused: "A slice over its byte budget is refused, never truncated (`crates/sealmap-dense/src/lib.rs:368`-`373`)". "at exactly the budget the pack is returned, one byte under it is refused" (`crates/sealmap-corpus/tests/pack.rs:181`). `shard` refuses any topic too large to fit in a single shard (`crates/sealmap-corpus/src/pack.rs:295`–`331`).
- Failure: In automated workflows generating review packs for LLMs or external reviewers, adding a single line of code or a single call to a cited function that pushes the payload one byte past the limit results in hard process termination (`OverBudget`) rather than graceful summarization, depth reduction, or truncation. Orchestration pipelines crash without an automated mitigation path.
- Confidence: High
- Marked by authors: Yes (Design choice, but an operational brittleness defect)

### F-13 — Closure binding and deferred execution misrepresent temporal causality in sequences
- Topics: EXT-03, EXT-03.2
- Evidence: "A closure saved in a variable is drawn where it is written, not where it is called" (`EXT-03`). Closures passed as arguments are held back and drawn *after* the method call completes (`crates/sealmap-rust/src/collect.rs:1344`–`1353`, `crates/sealmap-extract/src/raw.rs:160`).
- Failure: Sequence diagrams invert or misplace execution order. A closure bound via `let f = || ...;` and invoked lines later appears to execute before preceding logic. An inline closure passed to an iterator or runner appears to execute after the host function returns. Reviewers or LLM agents relying on Mermaid sequences receive false causality information about state mutations and network calls.
- Confidence: High
- Marked by authors: Yes (Debt noted in EXT-03)

### F-14 — Artificially elevated limits in Mermaid CI validator hide client-side rendering crashes
- Topics: DEL-01.5, COR-02
- Evidence: `tools/validate-mermaid.mjs:19` validates generated diagrams by launching Chromium and explicitly raising the internal edge and text limits (`DEL-01.5`, `ci.yml:63`–`65`).
- Failure: Standard production Markdown viewers (GitHub web UI, GitLab, VS Code preview, IDE plugins, Notion) enforce default Mermaid edge and string size caps. Diagrams that pass sealmap's CI validation exceed default client limits and crash or fail to render when viewed by human engineers in standard tools.
- Confidence: High
- Marked by authors: No

### F-15 — Lack of `--repo` support breaks staleness analysis in multi-crate workspaces
- Topics: COR-05.2
- Evidence: In `stale --since`, the CLI explicitly refuses execution if `--repo` is passed: "refuse with --repo (`main.rs:461`-`462`)" (`COR-05.2`).
- Failure: In complex multi-repository checkouts or setups where source paths require explicit `--repo` scoping, developers cannot run `sealmap stale --since REV` or `sealmap pack --diff REV`. Differential analysis and staleness tracking are unusable in multi-repo topologies.
- Confidence: High
- Marked by authors: No

---

## Not judgeable from this material

1. **Argument sanitization in `main.rs:608–646`:** Whether `REV` is passed to `git archive` using strict argument vectors (`Command::arg`) with an explicit `--` separator, or whether options can be injected via leading hyphens.
2. **Associativity of `Fingerprint::merge`:** The exact mathematical formulation in `crates/sealmap-model/src/hash.rs:138` when more than two definitions share an ID.
3. **Behavior on filesystem symlink loops:** Whether `crates/sealmap-model/src/path.rs` and the loader follow symlinks or handle cyclical filesystem references without panicking.
4. **Line-ending normalization during lock canonicality checks:** Whether `verify` canonical checks reject `seals.lock` due to CRLF versus LF conversions caused by `git checkout` on Windows.
5. **Memory residency during concurrent Rayon parsing:** Peak memory consumption when parsing massive multi-thousand-file codebases concurrently before the sequential resolve pass.
