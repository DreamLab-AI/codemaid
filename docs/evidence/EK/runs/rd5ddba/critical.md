### F-01 — Uncatchable process termination via AST parsing stack overflow
- Topics: EXT-01, EXT-03
- Evidence: EXT-01 claims: "It never fails a whole run because of one bad file: a file that does not parse, or that trips a bug, becomes a warning and a placeholder entry" and cites "panic-isolated big-stack collection", yet admits: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught." EXT-03 asserts that "walking stops below 1,024 levels of expression nesting, so generated code cannot overflow the collector's stack (`crates/sealmap-rust/src/collect.rs:1113`)".
- Failure: The 1,024 expression nesting limit only takes effect during flow walking, which runs *after* `syn` has already parsed the file into an AST during the collection pass. If an analysed repository contains generated or deeply nested syntax that exceeds thread stack limits during syn's recursive descent parse, the operating system raises a SIGSEGV / stack overflow. Because stack overflows in Rust cannot be caught by `catch_unwind`, the process crashes instantly, taking down CI pipelines or backend analysis workers without emitting diagnostics.
- Confidence: high
- Marked by authors: yes (residual risk noted in EXT-01)

### F-02 — Arbitrary parameter injection into external `git` subshell calls
- Topics: COR-05, COR-06
- Evidence: COR-05 states: "The only processes the CLI spawns are for `stale --since` and `pack --diff`, which export a git revision into a temporary directory so the library can compare two models, and `git rev-parse` and `git status` for a pack's revision line; git stays out of the libraries (`docs/DESIGN.md:166-169`)." COR-06 states: "`--diff REV` adds every topic whose sealed or cited symbols changed since a revision."
- Failure: If `sealmap pack --diff <REV>` or `sealmap stale --since <REV>` is invoked in automated CI or agent pipelines using untrusted inputs (e.g. PR branch names, user-provided ref strings, or webhook metadata), unescaped revision flags (e.g. `--output=...`, `--exec=...`) can be interpreted as git options rather than revision names, executing unintended actions or reading unauthorized files.
- Confidence: high (inferred from external process execution of user-supplied revision strings)
- Marked by authors: no

### F-03 — Destructive file deletion in `generate` based on loose front matter matching
- Topics: COR-01, COR-03
- Evidence: COR-01 states: "`write` only ever deletes files whose front matter starts that way (`crates/sealmap-corpus/src/document.rs:10-12`)." COR-03 states: "Invariant: `write` deletes only Markdown files whose YAML front matter opens on the first line (`---`) and whose first key is the `sealmap: ` marker; a file that merely mentions the marker anywhere else is never touched (`crates/sealmap-corpus/src/contract.rs:99`, `crates/sealmap-corpus/src/document.rs:153-155`)."
- Failure: If an adopter maintains human-written documentation, architectural notes, or ADRs that include metadata like `sealmap: reviewed`, `sealmap: false`, or any custom configuration under the `sealmap: ` key in their initial front matter lines, running `sealmap generate` (without `--check`) treats these files as orphan corpus outputs and permanently removes them from disk.
- Confidence: high
- Marked by authors: no

### F-04 — Non-atomic two-phase writes leave permanent lockfile corruption on interruption
- Topics: COR-05
- Evidence: COR-05 states: "Invariant: a failure between the two writes leaves a pointer with no entry, which `verify` reports as a lock fault, so an interrupted sign can never pass the gate (`crates/sealmap/src/main.rs:680-682`, `crates/sealmap-corpus/src/seal/check.rs:383`)."
- Failure: Signing a topic requires two separate disk writes: updating the topic file's front matter pointer and appending or rewriting the lockfile. If the process is terminated between the two operations (via CI job cancellation, SIGKILL, OOM termination, or host crash), the repository is left with a dangling pointer and missing lock record. All subsequent invocations of `sealmap verify` report a fatal lock fault, failing all CI runs across all branches until someone manually inspects and repairs the lockfile.
- Confidence: high
- Marked by authors: no (documented as an invariant rather than addressed as a fault)

### F-05 — Contradictory byte-budget invariant between review packs and the dense projection
- Topics: COR-06, DEN-01
- Evidence: DEN-01 asserts: "A slice over its byte budget is refused, never truncated (`crates/sealmap-dense/src/lib.rs:368-373`)." and "Invariant: at exactly the budget, the slice is returned unchanged. One byte under the budget, it is refused, with the overflow named (`crates/sealmap-dense/tests/dense.rs:304`)." In direct contradiction, COR-06 asserts: "Invariant: the dense slice is taken without a byte limit, so it cannot fail on size. The only budget is the pack's (`crates/sealmap-corpus/src/pack.rs:479-481`)."
- Failure: Slices generated for review packs bypass the slice-level sizing contract of `sealmap-dense`. An unexpectedly large topic slice will allocate unbounded memory and can only be rejected at the aggregate review pack boundary, defeating the fail-fast per-topic budgeting guarantees that agents and review harnesses depend on.
- Confidence: high
- Marked by authors: no (direct contradiction between topic specifications)

### F-06 — Injective ID guarantee broken by fallback sanitization of invalid package and manager names
- Topics: EXT-02, MOD-01, MER-02
- Evidence: MOD-01 promises: "two distinct ids never print the same text, checked over 4,096 generated cases (`crates/sealmap-model/tests/sym_props.rs:85`)." MER-02 promises: "`Ident::from_symbol` maps one to the other by an encoding that is injective by construction: two different `SymbolId`s never produce the same diagram id". However, EXT-02 states: "Invariant: building an id never fails: a package name the grammar cannot hold is repaired to `_`, and an invalid manager falls back to `unknown _ .` (`crates/sealmap-extract/src/ids.rs:57-63`)."
- Failure: When multiple packages with invalid or unsupported naming characters are analyzed within the same workspace or across dependencies, their package names are clamped to `_` and manager names to `unknown _ .`. Symbols sharing the same internal item path across those packages receive identical `SymbolId` strings and identical Mermaid diagram identifiers, violating the injectivity guarantees of MOD-01 and MER-02 and silently cross-linking unrelated functions.
- Confidence: high
- Marked by authors: no

### F-07 — Residual use of lossy `Ident::new` permits silent node collisions in diagrams
- Topics: MER-01, MER-02
- Evidence: MER-02 states that injective IDs prevent Mermaid from "silently draw[ing] them as one box" and closes "a class of silent error rather than reducing its odds (`crates/sealmap-mermaid/src/symbol.rs:10-12`)." However, MER-01 states: "Invariant: `Ident::new` is documented as lossy; only `Ident::from_symbol` is injective (`crates/sealmap-mermaid/src/escape.rs:43-48`)", and MER-02 admits that the topic records "where ids are still minted the lossy way."
- Failure: Non-symbol diagram nodes (such as module graph nodes, overview entity boxes, or flowchart items) minted via `Ident::new` are subject to the lossy sanitization rules. If two entities differ only by characters normalized away by `Ident::new`, Mermaid merges them into a single diagram node, creating the exact false-connectivity defect MER-02 claims was eradicated.
- Confidence: high
- Marked by authors: no

### F-08 — Ambiguous reachable methods are dropped, silently omitting valid internal call edges
- Topics: DEN-01, EXT-04, EXT-05
- Evidence: DEN-01 (Closed `613b9ba`) and EXT-05 state: "Two or more reachable candidates are a genuine ambiguity, and no edge is guessed; the call stays `sym:? name` (`crates/sealmap-rust/src/resolve.rs:954-968`)." EXT-04 states: "By default calls into the standard library and calls on values of unknown type are left out, which keeps diagrams about *your* code and its dependencies."
- Failure: If an unresolved receiver calls a method name implemented by two or more reachable workspace crates (for instance, common methods like `execute()`, `process()`, or `validate()`), the resolver refuses to guess and tags it `sym:? name`. Because the default external policy discards calls on values of unknown type, the call is completely omitted from sequence diagrams, flows, and dense projections. Critical internal calls between workspace crates silently vanish from reviewed diagrams.
- Confidence: high
- Marked by authors: no

### F-09 — Macro-generated code and control flow are invisible to seals and call graphs
- Topics: EXT-02, EXT-03, EXT-06
- Evidence: EXT-02 states: "items that a macro generates are not seen, so they get no name and cannot be cited or sealed (`README.md:342`)." EXT-03 states: "A macro invocation is never drawn as a call of its own, only the calls inside its arguments... Both are recorded below as debt."
- Failure: Codebases using macros for routing, RPC dispatch, async concurrency (`tokio::select!`), or domain DSLs cannot cite or seal macro-generated endpoints. Furthermore, when a function invokes a macro that handles control flow or delegates work, the invocation is not drawn as a call. Architectural topics can be sealed and pass CI indefinitely while critical logic inside macros changes without detection.
- Confidence: high
- Marked by authors: yes (debt in EXT-03, limitation in EXT-02)

### F-10 — Diagram truncation directly contradicts the core "nothing is truncated" guarantee
- Topics: COR-02, COR-06, EXT-03
- Evidence: COR-06 claims as an absolute guarantee: "Nothing is truncated... It never quietly drops content." In direct opposition, COR-02 states: "overview... each capped so Mermaid can render it... The caps are a deliberate trade: a diagram over its message or edge budget is cut... Long functions and hub crates are therefore always summarised in the diagrams; the complete call lists stay in the index." In addition, EXT-03 states: "walking stops below 1,024 levels of expression nesting... deeper calls are omitted (`crates/sealmap-rust/src/collect.rs:1113`)."
- Failure: Review packs and sequence diagrams present an incomplete representation of the codebase for hub crates and complex functions. An agent or reviewer relying on Mermaid diagrams to verify security invariants or call chains is reviewing a partial representation without knowing which specific call branches were excluded.
- Confidence: high
- Marked by authors: no (direct contradiction between subsystem claims)

### F-11 — Seal validity silently breaks across environments due to unpersisted CLI extraction flags
- Topics: COR-05
- Evidence: COR-05 states: "A seal remembers functions by id, and the ids depend on how the code was read (whether tests were included, what the codebase is called when no manifest names it). Checking a seal with different settings from the ones it was signed with makes sealed functions look absent."
- Failure: Unlike `CorpusOptions` in COR-01, seal lockfiles do not record or validate the CLI extraction flags (e.g. `--tests`, `--name`, `--repo`) under which the seals were signed. If an engineer signs a seal locally using default settings, but CI runs `sealmap verify` with `--tests` enabled, all sealed functions fail resolution and appear absent, breaking CI without any change to the underlying code.
- Confidence: high
- Marked by authors: no (noted only as "One caution for adopters")

### F-12 — Undeclared transitive dependency breakage violates published MSRV 1.85 promise
- Topics: DEL-01
- Evidence: DEL-01 promises: "It builds on a stated, tested minimum Rust (1.85)... There is one live caveat on the first promise. A dependency (`ignore` 0.4.30) needs a newer Rust than it declares; inside this repository the lockfile avoids it, but a project that depends on the published crates without that lock can still pull it in on Rust 1.85 and fail to build."
- Failure: Any downstream consumer adding `sealmap` or its subcrates as dependencies into an existing workspace on Rust 1.85 without a pinned Cargo.lock will resolve `ignore` 0.4.30 by default. Compilation immediately fails due to compiler incompatibility, breaking builds in production CI pipelines that enforce MSRV 1.85.
- Confidence: high
- Marked by authors: yes (caveat in DEL-01)

### F-13 — Production seal gating is completely unexercised on sealmap's own repository
- Topics: DEL-01, DEL-02
- Evidence: DEL-02 reveals: "And this repository has not sealed its own diagrams yet, so its CI does not run the seal gate (step 6)." DEL-01 confirms that CI only checks "that two fresh generations agree (DEL-01)."
- Failure: The foundational workflow of sealmap—using `sealmap verify` as an automated pull-request gate to enforce diagram validity—has never been run as a live gate within sealmap's own development cycle. Real-world edge cases involving CI exit codes, lockfile contention, concurrent branch merges, and git diff resolution remain unvalidated by dogfooding.
- Confidence: high
- Marked by authors: yes (unimplemented step 6 in DEL-02)

### F-14 — Hard external dependency on `git` breaks operation in containerized and non-git environments
- Topics: COR-05, EXT-01
- Evidence: EXT-01 claims: "It never compiles anything, so it runs on any checkout, including one that does not build, and needs no toolchain at run time." COR-05 contradicts this: "The only processes the CLI spawns are for `stale --since` and `pack --diff`, which export a git revision into a temporary directory so the library can compare two models, and `git rev-parse` and `git status` for a pack's revision line".
- Failure: In minimal production containers, distroless runners, or source trees unpacked from tarballs (e.g., Docker build contexts without `.git` directories), running `sealmap stale --since` or `sealmap pack --diff` fails immediately with executable-not-found or missing-repository errors from git subprocess invocations.
- Confidence: high
- Marked by authors: no (contradiction between claims of zero runtime toolchain and hard git dependency)

### F-15 — Direct contradiction regarding crates.io publication status of 0.2.0 crates
- Topics: DEL-01, DEL-02
- Evidence: DEL-01 explicitly states: "None of the 0.2 crates in this tree is published yet; the README's roadmap puts the first 0.2 crates.io release after step 3, which is now done in the tree (`README.md:364-367`)." However, DEL-02 contradicts this in two separate places: "Step 3 closed with the 0.2.0 publish (`docs/DESIGN.md:302-303`)." and "Closed in the 0.2.0 release: the workspace is at `0.2.0` (`Cargo.toml:6`), so a lock signed by this tree names `sealmap` 0.2.0, the first release that has the seal module (`crates/sealmap-corpus/src/seal/lock.rs:21`)."
- Failure: The release state of the codebase is contradictory across delivery documentation. If crates are not published, downstream CI and adopters referencing `0.2.0` in lockfiles cannot pull published artifacts. If they were published, DEL-01's verification state is out of date.
- Confidence: high
- Marked by authors: no (direct contradiction between DEL-01 and DEL-02)

---

## Not judgeable from this material

1. **Lockfile concurrency and file locking:** Whether concurrent invocations of `sealmap sign` or `sealmap verify` from multiple CI jobs or processes corrupt the lockfile or topic files when executed concurrently.
2. **Memory limits under large multi-crate workspaces:** Whether building in-memory models, call graphs, and dense representations across thousands of files triggers OOM termination on memory-constrained CI runners.
3. **Symlink handling and directory traversal:** Whether the source loader traverses symlinks pointing outside the repository root, potentially leaking external host files into generated models and review packs.
4. **Git command isolation and temporary directory cleanup:** How `sealmap` cleans up temporary directories created during `stale --since` and `pack --diff` when the process receives SIGTERM or crashes.
5. **Exact exit code specification across CLI subcommands:** Whether command failure modes (such as lock faults, drift, missing files, and syntax errors) map to distinct exit codes that CI pipelines can reliably branch on.
