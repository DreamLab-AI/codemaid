### F-01 — External review packs expose proprietary source code due to unimplemented review mode
- Topics: DEL-02, COR-06
- Evidence: DESIGN §42 specifies review packs for "an outside reviewer who should not see code" (`docs/DESIGN.md:42`), and DEL-02 lists this as a core layer requirement. However, DEL-02.1 confirms: `pack.rs:274; diagrams-only --review NOT BUILT`. `sealmap pack` unconditionally embeds the verbatim topic text, dense code slices, and source code windows (`crates/sealmap-corpus/src/pack.rs:477`-`495`).
- Failure: An operator distributes review packs to an external contractor or auditor expecting the diagrams-only mode described in the design, inadvertently disclosing proprietary implementation code and raw source lines.
- Confidence: high
- Marked by authors: yes (catalogued as a gap in DEL-02)

### F-02 — Unbounded parser recursion triggers uncatchable stack overflow aborts
- Topics: EXT-01, EXT-03
- Evidence: EXT-01 acknowledges: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught." While AST expression walking imposes a 1,024 depth cap (`crates/sealmap-rust/src/collect.rs:1113`), `syn::parse_file` runs during collection before walking occurs.
- Failure: An automated PR or generated source file containing deeply nested expressions causes native stack exhaustion during `syn` parsing inside `collect_file`, producing an immediate process crash (`SIGSEGV`/abort) that terminates CI or worker daemons without diagnostics.
- Confidence: high
- Marked by authors: yes (marked as residual risk in EXT-01)

### F-03 — Destructive directory cleanup permanently deletes non-corpus Markdown files
- Topics: COR-01, COR-03
- Evidence: COR-03 states as an invariant that `write` deletes any Markdown file on disk if its YAML front matter opens with `---` and its first key is `sealmap: ` (`crates/sealmap-corpus/src/contract.rs:99`, `crates/sealmap-corpus/src/document.rs:153`-`155`), followed by deleting empty parent directories (`crates/sealmap-corpus/tests/contract.rs:138`).
- Failure: Running `sealmap generate -o <dir>` against a shared docs directory or project root containing existing documentation, templates, or notes that happen to begin with a `sealmap:` frontmatter key causes `sealmap` to classify them as "Orphaned" and permanently unlink them and their parent directories.
- Confidence: high
- Marked by authors: no

### F-04 — Non-atomic signing leaves repository in a corrupted gate-blocking state on interruption
- Topics: COR-04, COR-05
- Evidence: `seal sign` executes two discrete, non-atomic disk writes: first updating the topic file on disk (`crates/sealmap/src/main.rs:679`-`680`), then writing `seals.lock` (`crates/sealmap/src/main.rs:682`). The authors note: "a failure between the two writes leaves a pointer with no entry, which verify reports as a lock fault, so an interrupted sign can never pass the gate (`crates/sealmap/src/main.rs:680`-`682`, `crates/sealmap-corpus/src/seal/check.rs:383`)."
- Failure: A SIGKILL, container preemption, power loss, or disk-full event between line 680 and line 682 updates the topic with a new `sealed:` pointer without writing the corresponding lock entry. All subsequent CI runs executing `seal verify` fail with an unrecoverable lock fault, blocking deployment pipelines until manually repaired.
- Confidence: high
- Marked by authors: no (framed by authors as a safety invariant rather than an operational failure)

### F-05 — Lack of lockfile concurrency control causes silent lock clobbering and lost seals
- Topics: COR-04, COR-05
- Evidence: `seal sign` reads `seals.lock`, validates it in memory, and writes the entire file back canonically (`crates/sealmap/src/main.rs:662`-`663`, `crates/sealmap/src/main.rs:682`). No file locking (`flock`), atomic compare-and-swap, or transactional merging exists.
- Failure: When multiple engineers or parallel CI agents sign different topics concurrently, concurrent processes overwrite `seals.lock`. The last writer overwrites the entire lockfile, discarding entries added by earlier processes while leaving their topic files with invalid pointers, failing the CI verification gate.
- Confidence: high
- Marked by authors: no

### F-06 — Total macro blindness conceals structural changes from verification gates
- Topics: EXT-02, EXT-03, EXT-06
- Evidence: EXT-02 states: "items that a macro generates are not seen, so they get no name and cannot be cited or sealed (`README.md:342`)." EXT-03 states: "A macro invocation is never drawn as a call of its own, only the calls inside its arguments... Both are recorded below as debt." EXT-06 notes that macro arguments that are not plain expressions bypass AST normalisation.
- Failure: Codebases using macros to define public API endpoints, routing, data structures, or trait implementations can undergo breaking behavioural or contract changes without altering symbol IDs or fingerprints. `seal verify` reports all seals as holding despite underlying functional divergence.
- Confidence: high
- Marked by authors: yes (marked as Debt in EXT-03; documented limitation in EXT-02 and EXT-06)

### F-07 — Unrelated dependency additions trigger silent call graph edge deletion
- Topics: DEN-01, EXT-05
- Evidence: In EXT-05.4, name resolution for untyped receiver method calls (`by_name_only`, `crates/sealmap-rust/src/resolve.rs:954`-`968`) guesses an edge only if a candidate method name is unique across reachable workspace crates (`crates/sealmap-rust/src/layout.rs:135`). "Two or more reachable candidates are a genuine ambiguity, and no edge is guessed; the call stays sym:? name (`crates/sealmap-rust/src/resolve.rs:954`-`968`)."
- Failure: Adding a method with a common name inside a dependency crate creates a second candidate for an untyped call in an unrelated crate. The resolver shifts from an inferred edge to ambiguity, causing the call edge in the untouched crate to silently vanish from dense projections, call trees, and sequence diagrams.
- Confidence: high
- Marked by authors: no

### F-08 — Inverted sequence diagram flow for variable-bound closures misrepresents execution order
- Topics: EXT-03
- Evidence: EXT-03 notes as architectural debt: "A closure saved in a variable is drawn where it is written, not where it is called."
- Failure: In callback, builder, or asynchronous dispatch patterns where closures are defined and stored in variables before invocation, sequence diagrams display the closure's operations as executing prior to the caller invocation, presenting an inverted execution sequence to reviewers and verification tooling.
- Confidence: high
- Marked by authors: yes (marked as Debt in EXT-03)

### F-09 — Recursive expression cap silently omits function calls without diagnostics
- Topics: EXT-03
- Evidence: EXT-03.2 states: "Invariant: walking stops below 1,024 levels of expression nesting, so generated code cannot overflow the collector's stack; deeper calls are omitted (`crates/sealmap-rust/src/collect.rs:1113`)."
- Failure: Complex nested builder chains, match expressions, or recursive AST evaluations exceeding 1,024 expression levels have downstream function calls silently omitted from the raw IR and call graphs without emitting a diagnostic or warning, presenting incomplete interaction models.
- Confidence: high
- Marked by authors: no

### F-10 — Hardcoded smart pointer unwrapping misattributes custom and third-party `Deref` types
- Topics: EXT-05
- Evidence: EXT-05.4 states: "Smart pointers and lock guards are looked through, but any other wrapper (Vec, Option, Mutex) is itself the receiver (`crates/sealmap-rust/src/resolve.rs:917`-`919`)." The resolver explicitly unwraps only standard `Box`, `Arc`, `Rc`, and standard synchronization guards (`crates/sealmap-rust/src/resolve.rs:921`).
- Failure: Custom domain wrappers or third-party smart pointers implementing `Deref` (such as `parking_lot::MutexGuard` or `owning_ref`) are treated as terminal receivers rather than unwrapped, causing calls to inner type methods to fail resolution and degrade to `sym:? name` or get dropped entirely.
- Confidence: medium (inferred from resolver mechanics)
- Marked by authors: no

### F-11 — Visual diagram budget caps silently truncate sequence and overview projections
- Topics: COR-02
- Evidence: COR-02 states: "The caps are a deliberate trade: a diagram over its message or edge budget is cut, with a note saying how much was left out and where the full list is, rather than rendered unreadably or not at all. Long functions and hub crates are therefore always summarised in the diagrams; the complete call lists stay in the index."
- Failure: Functions with large call volumes have operations truncated in rendered Mermaid diagrams. Engineers or reviewing models relying on visual diagrams miss downstream operations (such as authentication or cleanup calls) that were dropped due to diagram budget limits.
- Confidence: high
- Marked by authors: yes (acknowledged as a deliberate trade in COR-02)

### F-12 — Inflexible review pack budget limits halt review automation on boundary overflows
- Topics: COR-06
- Evidence: COR-06 states: "Nothing is truncated. Over budget, pack refuses with the header's size and each topic's (`crates/sealmap-corpus/src/pack.rs:278`-`282`)." Invariant: "at exactly the budget the pack is returned, one byte under it is refused (`crates/sealmap-corpus/tests/pack.rs:181`)."
- Failure: When an automated workflow runs `sealmap pack` on topics referencing functions with large call trees, exceeding the configured byte ceiling by a single byte triggers a hard abort rather than returning a degraded or truncated excerpt, halting automated review pipelines.
- Confidence: high
- Marked by authors: no

### F-13 — Divergent CLI extraction flags cause false absent-symbol gate rejections in CI
- Topics: COR-05, DEN-01
- Evidence: COR-05 documents: "a seal remembers functions by id, and the ids depend on how the code was read (whether tests were included, what the codebase is called when no manifest names it). Checking a seal with different settings from the ones it was signed with makes sealed functions look absent."
- Failure: A developer signs a topic locally where target settings or naming fallbacks differ from the CI pipeline's invocation. CI runs `seal verify` with different arguments, causing valid sealed IDs to be flagged as absent, rejecting un-drifted pull requests at the CI gate.
- Confidence: high
- Marked by authors: yes (documented as a caution in COR-05)

### F-14 — Transitive dependency version mismatch breaks downstream compilation on MSRV 1.85
- Topics: DEL-01
- Evidence: DEL-01 states: "A dependency (`ignore` 0.4.30) needs a newer Rust than it declares; inside this repository the lockfile avoids it, but a project that depends on the published crates without that lock can still pull it in on Rust 1.85 and fail to build."
- Failure: Downstream consumers integrating published `sealmap` crates on their declared MSRV (Rust 1.85) experience build failures upon updating dependencies without an explicitly pinned lockfile, breaking downstream CI pipelines.
- Confidence: high
- Marked by authors: yes (noted as a live caveat in DEL-01)

### F-15 — Repository CI does not dogfood or enforce the seal verification gate
- Topics: DEL-01, DEL-02
- Evidence: DEL-02 states: "this repository has not sealed its own diagrams yet, so its CI does not run the seal gate (step 6)." DEL-01 confirms that CI only checks generation determinism rather than the seal verification gate.
- Failure: The primary operational feature (`seal verify`) is not exercised against this repository's own authored topics in CI, allowing regressions in git state evaluation, exit codes, or platform path handling to remain undetected prior to production deployment.
- Confidence: high
- Marked by authors: yes (tracked as step 6 in DEL-02)

---

## Not judgeable from this material

1. **Child process execution safety**: Whether `stale --since` and `pack --diff` properly escape arbitrary revision strings passed to git subprocesses, handle missing `git` binaries in minimal container runtimes, or clean up temporary directories on unexpected termination.
2. **Date calculation correctness in `seal sign`**: Whether the homebrewed UTC calendar calculation (`crates/sealmap/src/main.rs:793`-`800`) correctly handles leap years, month boundaries, and epoch rollovers without producing invalid dates that violate `sign.rs:107`-`111`.
3. **Extraction resource consumption under scale**: Memory usage, CPU scaling, and isolation stability when running parallel collection and sequential resolution across workspaces containing millions of lines of code or deeply nested module trees.
4. **Advanced trait resolution handling**: How the name resolver handles complex Rust type systems such as blanket implementations, associated type projections, higher-ranked trait bounds, and conditional generic implementations.
5. **Token normalisation collisions in fingerprints**: Whether token-stream-based fingerprinting (`sm1`) in `crates/sealmap-rust/src/fingerprint.rs` generates hash collisions or false equivalence across distinct macro expansions.
