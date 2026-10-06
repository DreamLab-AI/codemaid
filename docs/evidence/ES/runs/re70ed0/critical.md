### F-01 — Non-atomic signing leaves corrupted topic files on interruption
- Topics: COR-04.6, COR-05.3
- Evidence: `seal sign` writes the modified topic file first and the lock file second (`crates/sealmap/src/main.rs:680`-`682`). The text claims that because `verify` reports a pointer without an entry as a `LockFault`, "an interrupted sign can never pass the gate" (`crates/sealmap-corpus/src/seal/check.rs:383`).
- Failure: A crash, power loss, SIGKILL, or full disk occurring between writing the topic file and writing `seals.lock` leaves the topic file on disk permanently modified with a `sealed:` front-matter pointer that has no corresponding lock entry. The working tree is left in a dirty, broken state; automated pipelines fail with `LockFault`, and recovering requires manual Git intervention to revert the orphaned topic pointers.
- Confidence: high
- Marked by authors: no

### F-02 — Unsafe marker-only check causes destructive deletion of non-generated Markdown files
- Topics: COR-01.2, COR-03.3
- Evidence: `sealmap generate` (without `--check`) brings target directories into compliance by deleting files deemed orphaned (`crates/sealmap-corpus/src/contract.rs:99`). The invariant states that `write` deletes files opening with `---` whose first key is `sealmap: ` (`crates/sealmap-corpus/src/document.rs:10`-`12`, `crates/sealmap-corpus/src/document.rs:153`-`155`).
- Failure: Any hand-written Markdown documentation, template, note, or third-party file inside the output directory tree that begins with YAML front matter containing `sealmap: <value>` (for instance, documentation describing sealmap options or an uncommitted guide) is classified as `Orphaned` and unlinked from the filesystem without confirmation or recovery mechanisms.
- Confidence: high
- Marked by authors: no

### F-03 — Unpinned codebase names leak host filesystem paths into symbol IDs and break determinism
- Topics: DEL-01.3, EXT-01.2, MOD-01.2
- Evidence: DEL-01.3 admits that the codebase name is sourced from outside the code: "opt name pinned, else the checkout directory leaks into output (`main.rs:393`, `main.rs:419`-`424`)". EXT-01.2 confirms that when no manifest package directory matches, the crate name falls back to "named after the codebase (`layout.rs:143`)".
- Failure: If a user or automated harness runs `sealmap generate`, `sealmap verify`, or `sealmap pack` without passing an explicit `--name`, two checkouts in directories with different names (e.g., `/builds/agent-1/project` vs `/builds/agent-2/project`, or developer directory vs CI directory) generate different `SymbolId` strings (`sym:cargo <name> ...`). All symbol IDs, content hashes, review packs, and verification checks fail across distinct environments despite byte-identical source files.
- Confidence: high
- Marked by authors: no

### F-04 — Deeply nested syntax trees trigger uncatchable process aborts via stack overflow
- Topics: EXT-01.3
- Evidence: EXT-01 states that Rayon worker threads reserve 64 MiB stacks to avoid stack exhaustion (`crates/sealmap-extract/src/isolate.rs:52`), but admits: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught" (`docs/DESIGN.md:232`).
- Failure: A maliciously crafted or machine-generated Rust file containing deeply nested expressions, macros, or type parameters exhausts the 64 MiB stack during Syn parsing. Because Rust stack overflows trigger an uncatchable abort (SIGSEGV/guard page fault), the entire host process crashes immediately, terminating the language adapter, skipping diagnostics, and killing any agent or server running `sealmap`.
- Confidence: high
- Marked by authors: yes (Debt)

### F-05 — Canonical lock validation fails on Windows checkouts due to CRLF conversion
- Topics: COR-04.2
- Evidence: `parse` normalizes line endings (`crates/sealmap-corpus/src/seal/lock.rs:204`), but `parse_canonical` demands that `to_toml()` equals the raw file text byte-for-byte (`crates/sealmap-corpus/src/seal/lock.rs:231`-`234`). COR-04.2 explicitly notes that while `parse` accepts `\r\n`, "`verify` still reports it as not canonical (`crates/sealmap-corpus/src/seal/lock.rs:229`-`230`)".
- Failure: On Windows machines or Git configurations where `core.autocrlf` is active, checking out `seals.lock` converts line endings to `\r\n`. Running `sealmap verify` invokes `parse_canonical`, detects the carriage returns, flags `LockFault` (`NonCanonical`), and exits with code 1, breaking CI gates on Windows systems even though the lockfile was cleanly committed and unmodified.
- Confidence: high
- Marked by authors: no

### F-06 — Seal lockfiles omit CLI extraction flags, causing false `Absent` failures under configuration drift
- Topics: COR-04.1, COR-05.1
- Evidence: COR-05 states: "Checking a seal with different settings from the ones it was signed with makes sealed functions look absent" (`crates/sealmap/src/main.rs:73`-`74`). However, the `Lock` and `TopicSeal` structs record only `version`, `algorithm`, `generator`, `topic_hash`, `reviewer`, `model`, `date`, and symbol fingerprints (`crates/sealmap-corpus/src/seal/lock.rs:57`-`79`), omitting the flags (`--tests`, `--repo`, `--name`) used during extraction.
- Failure: A topic signed with `--tests` or a specific codebase name is verified in CI using default flags. Because target configuration alters module roles (`crates/sealmap-rust/src/lib.rs:161`) and package prefixes, the symbol IDs no longer match the extracted model. The check reports symbols as `Absent` rather than diagnosing an option mismatch, rejecting a valid codebase.
- Confidence: high
- Marked by authors: no

### F-07 — Files exceeding 2 MiB or containing non-UTF-8 bytes are dropped silently without diagnostics
- Topics: MOD-03.2, EXT-01.4
- Evidence: `SourceSet::load_dir` walks the repository directory: if a file has an extension matching Rust but is 2 MiB or larger, or fails UTF-8 decoding, it is "skipped silently" (`crates/sealmap-model/src/source.rs:175`-`181`). This directly contradicts the invariant in EXT-01.4 that broken files produce a placeholder module symbol tagged `parse_error` and a diagnostic (`crates/sealmap-rust/src/collect.rs:57`-`59`, `crates/sealmap-rust/src/resolve.rs:218`-`222`).
- Failure: Large autogenerated source files (such as parser tables or protocol buffers) or source files containing invalid UTF-8 byte sequences are dropped without warning. Symbols defined inside them are omitted from the codebase. Any sealed topic citing functions in these files fails verification as `Absent`, and callers invoking them cannot resolve call edges.
- Confidence: high
- Marked by authors: no

### F-08 — Trait map overview projection silently discards relations beyond the edge budget
- Topics: COR-02.5
- Evidence: COR-02 claims: "A diagram over its message or edge budget is cut, with a note saying how much was left out and where the full list is, rather than rendered unreadably or not at all". However, COR-02.5 specifies for the overview trait map: "first max_edges relations in key order, no omitted count (`overview.rs:236`)".
- Failure: In repositories with dense trait hierarchies exceeding `max_edges` (default 300), the trait map in `_overview.md` silently truncates relations. Unlike the crate graph, module graph, and data model, no comment or warning is emitted indicating truncation occurred. Readers and downstream automated tools consume a partial trait implementation graph under the assumption that it is complete.
- Confidence: high
- Marked by authors: no

### F-09 — Untyped closure parameters synthesize false internal call edges
- Topics: EXT-05.6
- Evidence: EXT-05.6 states: "A closure's parameters are never entered into the walker's environment, so a method call on one is either a name guess (no outer binding) or resolved against the type of an unrelated outer variable with the same name" (`crates/sealmap-rust/src/collect.rs:1384`, `crates/sealmap-rust/tests/extract.rs:351`, `crates/sealmap-rust/tests/extract.rs:392`-`393`).
- Failure: Standard iterator chains (e.g. `items.iter().for_each(|x| x.save())`) treat `x` as `Untyped`. If `save` exists once among reachable crates, `resolve` binds `x.save()` as an `Inferred` call to an unrelated internal type's method (even if `x` is a primitive, third-party, or standard library type). Erroneous edges and phantom dependencies are injected into sequence diagrams, call graphs, and dense summaries.
- Confidence: high
- Marked by authors: yes (Debt)

### F-10 — Seal verification gate fails on sealmap's own repository, blocking CI integration
- Topics: DEL-02.6
- Evidence: DEL-02.6 documents that running `sealmap verify` on sealmap's repository fails: "MER-02 writes an example id from a fixture crate (shop) as an inline code span, which reads as a citation of a symbol nobody sealed... exit 1 (`check.rs:390`)". As a consequence, "no CI job runs verify yet (`ci.yml:9`-`84`)".
- Failure: The core product guarantee—enforcing diagram correctness via CI gates—cannot be dogfooded on the repository itself. Any inline backtick citation mentioning a fixture, third-party example, or documentation identifier matching the `sym:` grammar triggers an unsealed citation error. CI cannot enable `sealmap verify` without failing.
- Confidence: high
- Marked by authors: yes (Drift)

### F-11 — Workspace MSRV 1.85 guarantee fails for downstream consumers due to `ignore` dependency
- Topics: DEL-01.2
- Evidence: The workspace specifies MSRV 1.85 (`Cargo.toml:8`), but references `ignore >= 0.4.23` (`Cargo.toml:35`). `ignore 0.4.30` requires a compiler newer than 1.85 and declares no `rust-version`. While `Cargo.lock` pins `ignore 0.4.29` within the workspace (`Cargo.lock:264`), downstream consumers without the lockfile pull `0.4.30`.
- Failure: Integrating `sealmap-model` or `sealmap` as a library dependency in an environment locked to Rust 1.85 causes Cargo's resolver to fetch `ignore 0.4.30`, breaking compilation. The claimed MSRV compatibility holds only when building the internal workspace lockfile, not for downstream library adopters.
- Confidence: high
- Marked by authors: yes (Open)

### F-12 — Local closure calls are dropped, fragmenting call graph flows
- Topics: EXT-03.4
- Evidence: In EXT-03.4, when a closure is assigned to a variable via `let`, its body is walked at the point of definition as an optional fragment, but subsequent invocations of the variable are "dropped as a local closure (`resolve.rs:840`)".
- Failure: Functions that instantiate closures or function pointers and execute them later (such as helper routines, dispatch tables, or deferred cleanups) omit all call sites where the closure is invoked. Diagrams depict the closure logic at the definition line but disconnect it from the control flow and call graph at its actual point of execution.
- Confidence: high
- Marked by authors: yes (Debt)

### F-13 — Arbitrary depth limit in path resolution turns internal calls into unresolved external paths
- Topics: EXT-05.2
- Evidence: In EXT-05.2, `Resolver::walk` inspects path depth: "alt depth over 8 (`resolve.rs:642`) -> None, becomes a path id, External (`resolve.rs:601`)".
- Failure: In deeply nested module hierarchies or re-export chains exceeding 8 segments (e.g. `crate::module::sub::layer::domain::aggregate::entity::method`), path resolution terminates prematurely. The internal call is misclassified as `External` confidence, causing the resolver to omit the internal relation or route it to an external lane.
- Confidence: high
- Marked by authors: no

### F-14 — Method signature fingerprints include parent `impl` headers, triggering widespread seal invalidation
- Topics: EXT-06.2, EXT-02.3
- Evidence: EXT-02 promises that methods sit under their type rather than an `impl` block so that reorganizing `impl` blocks changes no name (`README.md:177`-`179`). However, EXT-06.2 specifies that a method's signature hasher consumes the `impl_header` (`crates/sealmap-rust/src/collect.rs:451`), including `attrs, unsafety, generics, trait, self type, where`.
- Failure: Editing a single generic parameter, where clause, or non-doc attribute on an `impl` block modifies the `sig_hash` of every method defined within that block. Even if 40 methods undergo zero signature or body changes, all 40 are flagged with `Contract` drift by `sealmap verify`, invalidating seals across unrelated topics.
- Confidence: high
- Marked by authors: no

### F-15 — Contradiction regarding publication status and version stability across documentation
- Topics: DEL-01, DEL-02.2, DEL-02.5
- Evidence: DEL-01 explicitly states: "None of the 0.2 crates in this tree is published yet; the README's roadmap puts the first 0.2 crates.io release after step 3... (`README.md:364`-`367`)". DEL-02.2 and DEL-02.5 contradict this by claiming: "published as 0.2.0, docs/DESIGN.md:298", and "Closed in the 0.2.0 release: the workspace is at 0.2.0 (`Cargo.toml:6`), so a lock signed by this tree names sealmap 0.2.0". DEL-02.2 also claims: "Three steps done in the tree, the third short of its publish".
- Failure: Deploying this repository into automated toolchains creates ambiguity regarding crate identity and reproducibility. Seals generated by the tool stamp `generator = "sealmap 0.2.0"` in `seals.lock`, but this version may correspond to an unpublished git commit rather than an immutable crates.io release, preventing external auditors from replicating seal validation results.
- Confidence: high
- Marked by authors: no

---

## Not judgeable from this material

1. **Subprocess injection safety in Git commands:** `stale --since` and `pack --diff` pass revision arguments to `git archive` and `tar -x` via command execution (`crates/sealmap/src/main.rs:608`-`622`). The material does not show whether revisions containing shell metacharacters or leading hyphens (e.g., `--output`) are sanitized against command injection.
2. **Determinism of `Fingerprint::merge` across three or more `#[cfg]` variants:** `Fingerprint::merge` sorts and hashes pairs of fingerprints (`crates/sealmap-model/src/hash.rs:138`). It cannot be determined whether iterating three or more `#[cfg]` twins under non-deterministic parallel AST collection produces identical merged hashes regardless of insertion sequence.
3. **Memory bounds during parallel extraction:** `collect_all` runs across Rayon worker threads with 64 MiB stacks (`crates/sealmap-extract/src/isolate.rs:52`). On high-core count systems (e.g. 64–128 threads), virtual and resident memory consumption during concurrent extraction of large workspaces cannot be verified from the diagrams.
4. **Behavior under macro-generated AST items:** Macro expansions are explicitly unexpanded by Syn collection (`crates/sealmap-rust/src/collect.rs:410`-`428`). How heavily macro-based frameworks (such as `tokio::select!`, `axum`, or procedural attribute derivations) impact graph connectivity and symbol resolution cannot be determined without empirical coverage data.
5. **Multi-tenant concurrency and file locking:** Seal operations (`seal sign`) and file writes (`sealmap generate`) perform directory reading and file rewriting without explicit file locks. Whether concurrent invocations across CI jobs or agents within the same directory tree cause write collisions or race conditions cannot be determined.
