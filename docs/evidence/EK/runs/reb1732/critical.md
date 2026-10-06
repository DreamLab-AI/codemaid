### F-01 — Command and argument injection via untrusted git revisions
- Topics: COR-05, COR-06
- Evidence: COR-05 states that the CLI spawns subprocesses for `stale --since` and `pack --diff` to export git revisions into temporary directories (`crates/sealmap/src/main.rs:696`, `docs/DESIGN.md:166`-`169`), and COR-06 states that `--diff REV` takes an opaque revision string (`crates/sealmap-corpus/src/pack.rs:357`-`370`).
- Failure: If revision strings supplied by users, agent harnesses, or CI triggers contain git flag injections (such as `--upload-pack`, `--exec`, or option injection into checkout/archive), arbitrary commands can be executed in the host environment.
- Confidence: medium (inferred)
- Marked by authors: no

### F-02 — Uncatchable process abort via AST recursive stack overflow
- Topics: EXT-01, EXT-03
- Evidence: EXT-01 asserts that the parser "never fails a whole run because of one bad file: a file that does not parse, or that trips a bug, becomes a warning and a placeholder entry", but concedes: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught." EXT-03 claims recursion is bounded: "Invariant: walking stops below 1,024 levels of expression nesting, so generated code cannot overflow the collector's stack; deeper calls are omitted (`crates/sealmap-rust/src/collect.rs:1113`)."
- Failure: The 1,024 depth limit protects only the flow walker during collection, not the preceding recursive syn parsing step. A single file with deeply nested expressions, macros, or types causes a thread stack overflow during AST parsing, triggering an immediate OS-level process abort that bypasses thread panic isolation (`crates/sealmap-extract/src/isolate.rs:34`-`37`) and crashes the entire job.
- Confidence: high
- Marked by authors: no

### F-03 — Destructive file deletion in `generate` based on loose front-matter matching
- Topics: COR-01, COR-03
- Evidence: COR-01 and COR-03 claim that `write` is safe because of the invariant: "`write` deletes only Markdown files whose YAML front matter opens on the first line (`---`) and whose first key is the `sealmap: ` marker; a file that merely mentions the marker anywhere else is never touched (`crates/sealmap-corpus/src/contract.rs:99`, `crates/sealmap-corpus/src/document.rs:153`-`155`)."
- Failure: If `sealmap generate` is pointed to an existing directory (such as a documentation root or mixed notes folder), `verify_against` treats any markdown file containing an opening YAML `sealmap:` key as generator-managed. Any human-authored or external files that use this front-matter key are flagged as orphaned and permanently deleted from disk.
- Confidence: high
- Marked by authors: no

### F-04 — Non-atomic lock file writing leaves corrupted state on crash
- Topics: COR-04, COR-05
- Evidence: COR-05 documents that signing performs two writes, stating: "Invariant: a failure between the two writes leaves a pointer with no entry, which `verify` reports as a lock fault, so an interrupted sign can never pass the gate (`crates/sealmap/src/main.rs:680`-`682`, `crates/sealmap-corpus/src/seal/check.rs:383`)."
- Failure: While framed as a gate guarantee, an interrupted process (SIGKILL, container preemption, CI cancellation, power failure) between write one and write two corrupts the lockfile with dangling pointers. Because `verify` treats lock faults as fatal, every subsequent CI run is permanently broken until an engineer manually edits or reconstructs the lockfile text.
- Confidence: high
- Marked by authors: no

### F-05 — Call target re-binding bypasses the seal verification gate due to missing `flow_hash`
- Topics: MOD-02, EXT-06, EXT-05, COR-04
- Evidence: COR-04 relies on `sig_hash` and `body_hash` to prove a sealed function still `Holds` (`crates/sealmap-corpus/src/seal/check.rs:45`-`46`, `:104`-`105`). MOD-02 acknowledges: "The crate-surface table records that an optional `flow_hash` was considered and not built (`docs/DESIGN.md:130`); the model has only the two (`crates/sealmap-model/src/symbol.rs:209`-`212`)." EXT-06 confirms fingerprints are derived exclusively from the token stream of the definition itself (`crates/sealmap-extract/src/fingerprint.rs:4`-`24`, `crates/sealmap-rust/src/fingerprint.rs:1`-`30`).
- Failure: If a function body calls `process()`, and an import is changed from `use safe::process;` to `use malicious::process;` (or a trait impl is swapped), the function's syn token stream is identical, so its `sig_hash` and `body_hash` do not change. `sealmap verify` reports `Holds`, approving an altered execution graph without invalidating the seal.
- Confidence: high
- Marked by authors: yes (Open / dropped design feature in docs/DESIGN.md:130)

### F-06 — Silent Symbol ID collisions caused by fallback repair of invalid names
- Topics: MOD-01, EXT-02
- Evidence: MOD-01 asserts: "two distinct ids never print the same text, checked over 4,096 generated cases (`crates/sealmap-model/tests/sym_props.rs:85`)." EXT-02 contradicts this uniqueness guarantee: "Invariant: building an id never fails: a package name the grammar cannot hold is repaired to `_`, and an invalid manager falls back to `unknown _ .` (`crates/sealmap-extract/src/ids.rs:57`-`63`)."
- Failure: When multiple packages in a workspace have names outside the grammar's accepted alphabet, or when multiple managers are invalid, all of them are coerced to the identical identifier `_` and manager `unknown _ .`. Identically named modules and functions within those packages produce colliding `SymbolId`s, causing unrelated definitions to overwrite each other in the model and merge in generated seals.
- Confidence: high
- Marked by authors: no

### F-07 — Ambiguous method calls and external APIs are dropped, creating false empty flows
- Topics: EXT-04, EXT-05, DEN-01
- Evidence: EXT-05 states: "Two or more reachable candidates are a genuine ambiguity, and no edge is guessed; the call stays `sym:? name` (`crates/sealmap-rust/src/resolve.rs:954`-`968`)." EXT-04 states: "By default calls into the standard library and calls on values of unknown type are left out... Invariant: a symbol whose body makes no kept call has `flow: None`, so 'has a sequence diagram' and 'has a flow' are the same test (`crates/sealmap-extract/src/lower.rs:46`-`48`)."
- Failure: In idiomatic code where methods use common names (`handle()`, `execute()`, `dispatch()`) or interact heavily with external crates/std, calls resolve to `sym:? name` and are dropped by the default external policy. Critical orchestration functions end up with `flow: None`, producing no sequence diagrams and appearing as inert leaf nodes in dense projections, deceiving reviewers and agents into assuming they make no calls.
- Confidence: high
- Marked by authors: no

### F-08 — Macro-generated code is completely invisible to seals and sequence diagrams
- Topics: EXT-02, EXT-03, EXT-06
- Evidence: EXT-02 states: "Items that a macro generates are not seen, so they get no name and cannot be cited or sealed (`README.md:342`)." EXT-03 states: "A macro invocation is never drawn as a call of its own, only the calls inside its arguments, and only when those arguments look like ordinary expressions. Both are recorded below as debt."
- Failure: Codebases using declarative or procedural macros for routing (e.g. Axum, Actix), serialization, or dispatch cannot cite or seal generated entry points. Invocations of macros with non-standard syntax are omitted from call flows, leaving critical execution paths absent from generated diagrams and invisible to CI seal enforcement.
- Confidence: high
- Marked by authors: yes (Debt)

### F-09 — Non-injective lossy ID minting retained in diagram emitters
- Topics: MER-01, MER-02
- Evidence: MER-02 claims diagram IDs are "injective by construction: two different `SymbolId`s never produce the same diagram id, with no hash suffix and no collision handling (`crates/sealmap-mermaid/src/symbol.rs:10`-`12`)." However, MER-01 states: "Invariant: `Ident::new` is documented as lossy; only `Ident::from_symbol` is injective (`crates/sealmap-mermaid/src/escape.rs:43`-`48`, MER-02)." MER-02 admits that the topic draws "where ids are still minted the lossy way."
- Failure: Projections that continue to invoke `Ident::new` rather than `Ident::from_symbol` produce colliding node identifiers in Mermaid output. Separate components merge into a single node in rendered diagrams, misrepresenting dependencies and call lanes.
- Confidence: medium
- Marked by authors: yes (Debt / acknowledged residual in MER-02)

### F-10 — Closure execution sequencing inverts actual control flow
- Topics: EXT-03
- Evidence: EXT-03 records that "receivers and arguments are walked before the call that consumes them, and closure bodies passed as arguments are placed *after* the call... A closure saved in a variable is drawn where it is written, not where it is called."
- Failure: Saving a closure or callback in a variable before invoking or passing it causes the sequence diagram to depict the closure's operations at the assignment site rather than its execution site. For asynchronous pipelines and callback-based architectures, the generated sequence diagrams misrepresent the true order of execution.
- Confidence: high
- Marked by authors: yes (Debt)

### F-11 — sealmap does not dogfood its own seal gate in CI
- Topics: DEL-02, COR-04, COR-05
- Evidence: DEL-02 states: "And this repository has not sealed its own diagrams yet, so its CI does not run the seal gate (step 6)." COR-04 and COR-05 describe `sealmap verify` as the primary mechanism to protect against diagram drift in production pipelines.
- Failure: The primary gate mechanism offered to adopters is not run on the product's own codebase in CI. Regressions and corner cases in the seal verification workflow can pass through sealmap's own delivery pipeline undetected.
- Confidence: high
- Marked by authors: yes (Open / step 6)

### F-12 — Undocumented MSRV failure for downstream consumers via `ignore` dependency
- Topics: DEL-01
- Evidence: DEL-01 guarantees: "It builds on a stated, tested minimum Rust (1.85), so a harness image does not have to chase the newest toolchain." It then acknowledges: "A dependency (`ignore` 0.4.30) needs a newer Rust than it declares; inside this repository the lockfile avoids it, but a project that depends on the published crates without that lock can still pull it in on Rust 1.85 and fail to build."
- Failure: Downstream crates pulling in `sealmap` libraries on Rust 1.85 without cargo lockfile pinning will resolve `ignore` 0.4.30 during dependency resolution, causing builds to fail on the declared MSRV.
- Confidence: high
- Marked by authors: yes (Caveat in DEL-01)

### F-13 — Symbol IDs depend on CLI execution flags, causing false CI verification failures
- Topics: COR-05, DEN-01
- Evidence: COR-05 warns: "Checking a seal with different settings from the ones it was signed with makes sealed functions look absent." DEN-01 notes that extraction depends on flags such as `--repo`, `--name`, and `--tests` (`crates/sealmap/src/main.rs:51`, `:341`-`348`).
- Failure: If an engineer seals topics in an environment where test harnesses are enabled or where workspace flags differ from the default CI runner invocation, `SymbolId` paths diverge. In CI, `sealmap verify` reports all cited symbols as absent, failing builds on valid code.
- Confidence: high
- Marked by authors: no

### F-14 — Inflexible byte budget enforcement causes hard refusals in review pack pipelines
- Topics: COR-06, DEN-01
- Evidence: COR-06 states: "Invariant: at exactly the budget the pack is returned, one byte under it is refused, and the refusal's sizes add up to the pack's length (`crates/sealmap-corpus/tests/pack.rs:181`)." DEN-01 states: "A slice over its byte budget is refused, never truncated (`crates/sealmap-dense/src/lib.rs:368`-`373`)."
- Failure: Review pack creation has zero tolerance for budget limits. If a pack exceeds a configured context limit by a single byte, it hard-fails and emits nothing. Minor edits in cited source files will abort automated review workflows rather than providing degraded or partial output.
- Confidence: high
- Marked by authors: no

### F-15 — Contradiction between delivery topics on crates.io 0.2.0 publication
- Topics: DEL-01, DEL-02
- Evidence: DEL-01 states: "None of the 0.2 crates in this tree is published yet; the README's roadmap puts the first 0.2 crates.io release after step 3, which is now done in the tree (`README.md:364`-`367`)." DEL-02 contradicts this directly: "Step 3 closed with the 0.2.0 publish (`docs/DESIGN.md:302`-`303`). Closed in the 0.2.0 release: the workspace is at `0.2.0` (`Cargo.toml:6`), so a lock signed by this tree names `sealmap` 0.2.0, the first release that has the seal module (`crates/sealmap-corpus/src/seal/lock.rs:21`)."
- Failure: Release state is tracked inconsistently across delivery documentation, making it indeterminate whether the codebase represents published artifacts or an unreleased state, risking botched packaging and dependency mismatches.
- Confidence: high
- Marked by authors: no

---

## Not judgeable from this material

1. **Subprocess argument sanitization in CLI execution**: Whether `REV` arguments passed to `git rev-parse`, `git status`, `stale --since`, and `pack --diff` in `crates/sealmap/src/main.rs:696` are sanitized against option injection (e.g. arguments starting with `--`).
2. **Specific usages of lossy `Ident::new`**: The exact call sites in `sealmap-corpus` or `sealmap-mermaid` where lossy diagram IDs are still minted instead of injective symbol IDs.
3. **Multi-process concurrency in `.sealmap` directories**: Whether concurrent executions of `sealmap dense`, `generate`, or `pack` in the same working tree collide on files or temporary directories.
4. **Memory exhaustion on large workspaces**: The memory growth profile of the breadth-first search and precomputed glob tables in `crates/sealmap-rust/src/resolve.rs:456`-`482` when resolving workspaces with tens of thousands of files.
5. **Divergence across verified commits**: All subsystem topics cite commit `4ed7a51`, whereas delivery topics cite commit `6cefaf4`; the material does not reveal what changes were introduced across the workspace between those two revisions.
