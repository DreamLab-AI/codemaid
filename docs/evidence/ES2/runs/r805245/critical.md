### F-01 — Process abort via uncatchable stack overflow on deeply nested syntax
- Topics: EXT-01.3, EXT-01, EXT-03.2
- Evidence: `EXT-01.3` notes that rayon threads run with 64 MiB stacks, but syn costs ~40 KiB per nesting level in debug builds (`crates/sealmap-extract/src/isolate.rs:3-7`), and admits that deeply nested code "still ends the process, because a stack overflow cannot be caught" (`crates/sealmap-rust/src/lib.rs:14-30`). While `EXT-03.2` sets a 1,024-level expression recursion cap (`crates/sealmap-rust/src/collect.rs:1113`), that cap is checked during AST walking *after* syn has already parsed the file (`crates/sealmap-rust/src/collect.rs:33`).
- Failure: A maliciously crafted or machine-generated source file with deeply nested types, macros, or delimiters causes `syn::parse_file` to overflow the thread stack during the initial parse. Because stack overflows cannot be caught by `catch_unwind` (`isolate.rs:46`), the process aborts immediately, crashing CI or the host service with no diagnostic.
- Confidence: High
- Marked by authors: Yes (residual risk noted in EXT-01)

### F-02 — Potential argument injection in git export pipeline
- Topics: COR-05.2, DEL-02.3
- Evidence: `stale --since REV` and `pack --diff REV` execute a shell-level pipeline: `GT->>GI: archive REV:prefix piped into tar -x, temp dir (main.rs:614-622)`. The opaque revision string passed on the command line (`main.rs:464`) is directly interpolated into the `git archive` invocation (`main.rs:614`).
- Failure: If an untrusted revision string is supplied (for example, by an automated PR worker, an agent skill, or a CI webhook reading branch names like `--output=/path/to/target`), `git archive` interprets the revision as command-line options or writes files outside the temporary directory via tar extraction, leading to arbitrary file write or command execution.
- Confidence: Medium (inferred from pipeline description in `main.rs:614-622`)
- Marked by authors: No

### F-03 — Non-atomic signing leaves repository in a broken state on crash
- Topics: COR-05.3, COR-04.4
- Evidence: `seal sign` executes a two-phase disk write: the topic file is written first (`main.rs:679-680`), followed by writing `seals.lock` (`main.rs:682`). The authors frame this as an invariant: "a failure between the two writes leaves a pointer with no entry, which `verify` reports as a lock fault" (`main.rs:680-682`, `check.rs:383`).
- Failure: If the process is terminated (SIGKILL, power failure, or container preemption) after updating the Markdown topic but before committing the lockfile, the topic file on disk retains a new `sealed:` pointer that has no corresponding entry in `seals.lock`. Subsequent CI runs fail with `LockFault`, and the developer cannot cleanly revert without manual surgery on front-matter pointers.
- Confidence: High
- Marked by authors: Yes (acknowledged as an invariant failure mode in COR-05.3)

### F-04 — Seals claim diagrams are verified but only check AST tokens, missing call graph and dependency changes
- Topics: COR-04.1, MOD-02.5, COR-04, DEL-02.4
- Evidence: `COR-04` claims a seal proves that "a hand-written topic was reviewed against exact versions of the code it cites" and that CI can mechanically declare diagrams "still true". However, `TopicSeal` records only `sig_hash` and `body_hash` (`crates/sealmap-corpus/src/seal/lock.rs:57`). `flow_hash` was explicitly rejected and not built (`crates/sealmap-model/src/symbol.rs:209-212`, `docs/DESIGN.md:130`).
- Failure: If a function `foo()` calls `bar()`, and `bar()`'s implementation changes completely, or if an external dependency changes the resolution of `foo`'s calls, `foo`'s own syntax tokens remain identical. `sealmap verify` evaluates `foo`'s fingerprints as unchanged and reports `Holds`. CI passes, certifying as valid a sequence diagram depicting interactions that no longer occur in the code.
- Confidence: High
- Marked by authors: Yes (authors note `flow_hash` was dropped in MOD-02.5, but do not mark the resulting contradiction with verification guarantees)

### F-05 — Resolver binds shadowed closure parameters to outer types, hallucinating call edges
- Topics: EXT-05.6, EXT-03.7
- Evidence: `EXT-05.6` documents that closure parameters are never added to the walker environment (`crates/sealmap-rust/src/collect.rs:1348-1351`). If a closure parameter shadows an outer variable name, "a parameter that shadows an outer typed name keeps the outer type" (`collect.rs:1384`, `tests/extract.rs:392-393`).
- Failure: In code such as `let conn = Db::connect(); xs.iter().for_each(|conn| conn.clear());` where `conn` inside the closure is an integer or standard collection, any method called on the parameter is resolved against `Db`. The resolver fabricates calls from the enclosing function to `Db::clear`, injecting spurious arrows into sequence diagrams and corrupting the call index.
- Confidence: High
- Marked by authors: Yes (Debt noted in EXT-05.6)

### F-06 — Unescaped Markdown code spans in prose trigger false citation failures in CI
- Topics: COR-04.3, DEL-02.6, COR-04.4
- Evidence: `COR-04.3` specifies that any CommonMark code span outside fenced blocks that starts like a global symbol ID (`sym:`, cargo manager, space) is treated as a code citation (`crates/sealmap-corpus/src/seal/topic.rs:182, 331`). In `DEL-02.6`, `sealmap verify` failed on the project's own documentation because topic `MER-02` mentioned an example ID (`shop`) in prose, which the checker evaluated as an unsealed citation (`crates/sealmap-corpus/src/seal/check.rs:390`).
- Failure: An engineer writing explanatory documentation or architectural ADRs who quotes an illustrative symbol ID, a syntax grammar example, or a foreign reference within backticks causes `sealmap verify` to classify it as an `UnsealedCitation`, exiting with code 1 and failing CI.
- Confidence: High
- Marked by authors: Yes (documented as dogfood triage in DEL-02.6)

### F-07 — CRLF line endings on Windows checkouts cause false `LockFault` failures in `verify`
- Topics: COR-04.2, COR-04.4
- Evidence: `Lock::parse` normalizes line endings (`crates/sealmap-corpus/src/seal/lock.rs:204`), but `parse_canonical` requires that `to_toml` matches the file on disk byte for byte (`lock.rs:231-234`). `to_toml` generates LF endings. `COR-04.2` notes that "parse alone normalises line endings... while verify still reports it as not canonical" (`lock.rs:229-230`).
- Failure: When a developer or CI runner checks out the repository on Windows with Git's default `core.autocrlf = true`, `seals.lock` is written to disk with CRLF. Running `sealmap verify` executes `parse_canonical`, detects the byte mismatch caused by `\r`, outputs a `LockFault`, and exits with code 1, despite the lock's content being completely intact.
- Confidence: High
- Marked by authors: No

### F-08 — Unpinned codebase name leaks ambient directory paths into symbol IDs, invalidating seals
- Topics: DEL-01.3, COR-03.1, COR-05
- Evidence: `DEL-01.3` admits that the codebase name leaks into the output (`main.rs:393, 419-424`) unless explicitly pinned with `--name` (`.github/workflows/ci.yml:29`). The codebase name is derived as a fallback from the checkout directory name (`main.rs:393`).
- Failure: If a developer seals a topic locally in a directory named `sealmap-dev`, and CI checks out the branch into a directory named `workspace` or `repo` without passing `--name`, all global symbol IDs change (e.g. `sym:cargo sealmap-dev .` vs `sym:cargo workspace .`). `sealmap verify` reports every sealed symbol as `Absent`, failing the build.
- Confidence: High
- Marked by authors: No

### F-09 — Unpinned dependency `ignore 0.4.30` breaks downstream compilation on the declared MSRV
- Topics: DEL-01.2, DEL-01
- Evidence: Workspace crates declare `rust-version = "1.85"` (`Cargo.toml:8`) and depend on `ignore = "0.4.23"` (`Cargo.toml:35`). `Cargo.lock` pins `ignore` to 0.4.29, but downstream consumers compiling without the workspace lockfile resolve `ignore 0.4.30`, which does not declare a `rust-version` and fails to compile on Rust 1.85 (`ci.yml:36-38`).
- Failure: Any downstream application specifying an MSRV of Rust 1.85 that adds `sealmap` or `sealmap-model` as a dependency will fail to build on 1.85 unless they manually downgrade and pin transitive dependencies in their own lockfile.
- Confidence: High
- Marked by authors: Yes (noted as an MSRV caveat in DEL-01.2)

### F-10 — Unrelated method additions in dependencies silently drop call graph edges
- Topics: EXT-05.5, DEN-01.6
- Evidence: When resolving method calls on untyped receivers, the resolver applies a "by-name guess" only if the method name exists exactly once among reachable workspace crates (`crates/sealmap-rust/src/resolve.rs:954-968`). If there are two or more candidates, it emits no edge and leaves the call as `sym:? name` (`resolve.rs:968`).
- Failure: If an internal module relies on a guessed call to a unique helper `format_data()`, and an unrelated workspace dependency adds a struct with a method also named `format_data()`, the candidate count increases from 1 to 2. The edge is instantly and silently removed from the caller's flow and sequence diagram, altering architecture diagrams without any code change in the caller.
- Confidence: High
- Marked by authors: No (the scoping rule in `layout.rs:135` restricted the search space, but did not eliminate ambiguity-induced edge drops)

### F-11 — Verification gate is entirely un-dogfooded in repository CI
- Topics: DEL-01.1, DEL-02.6, DEL-02.2
- Evidence: `DEL-01.1` documents the four CI jobs (`ci.yml:9-84`), none of which execute `sealmap verify`. CI runs the legacy 0.1 `path:line` citation check (`ci.yml:83`), while `DEL-02.6` acknowledges that running `sealmap verify` on the current tree fails on `MER-02`. `DEL-02.2` lists dogfooding the seal gate as deferred to step 6 (`docs/DESIGN.md:308`).
- Failure: The central feature of the 0.2 architecture—gating CI on cryptographic seals (`seals.lock`)—is not running on the repository that produces it. Breaking changes in `sealmap verify` or `seal sign` can pass all existing CI checks undetected.
- Confidence: High
- Marked by authors: Yes (Open / deferred in DEL-02.2)

### F-12 — Incremental analysis and diff commands crash when given multi-repository source sets
- Topics: COR-05.2, COR-03.5
- Evidence: `sealmap` supports multi-repository codebases via `--repo` (`crates/sealmap/src/lib.rs:125`). However, `stale --since` explicitly refuses to run when `--repo` is specified (`crates/sealmap/src/main.rs:461-462`).
- Failure: Teams running multi-repository workspaces loaded under name prefixes cannot run `sealmap stale --since REV` or `sealmap pack --diff REV` in their review pipelines; the commands exit with code 2, forcing teams to run unbounded full regenerations or forego staleness checks altogether.
- Confidence: High
- Marked by authors: No

### F-13 — Trait implementation methods are unconditionally marked public, leaking private APIs
- Topics: EXT-02.3, COR-03.5
- Evidence: `EXT-02.3` establishes as an invariant: "a trait-impl method is recorded as public whatever its written visibility, because trait methods are as visible as the trait" (`crates/sealmap-rust/src/collect.rs:458`).
- Failure: When an internal or `pub(crate)` trait is implemented on an internal struct, its methods are tagged `Public` in the model. If a user runs `sealmap generate --public-only` (`main.rs:89-119`) to produce an external-facing corpus for third-party consumers, private implementation details leak into the generated output.
- Confidence: High
- Marked by authors: No

### F-14 — Trait map overview truncates at 300 relations with no omission marker
- Topics: COR-02.5
- Evidence: While crate and module graphs cap edges at `max_edges` and emit a comment stating the omitted count (`crates/sealmap-corpus/src/overview.rs:68, 125`), the trait map caps relations by taking the "first max_edges relations in key order, no omitted count" (`overview.rs:236`).
- Failure: In large codebases exceeding 300 trait implementations, `_overview.md` displays a truncated trait hierarchy. Because no comment or truncation indicator is generated, downstream LLM agents parsing the overview assume the trait map is complete and hallucinate that required types do not implement expected traits.
- Confidence: High
- Marked by authors: No

### F-15 — Rejection of unknown keys in `seals.lock` prevents forward compatibility
- Topics: COR-04.2
- Evidence: `Lock::parse` deserializes TOML into raw structs that strictly refuse unknown keys (`crates/sealmap-corpus/src/seal/lock.rs:142`).
- Failure: If a future patch or an external reviewer skill introduces an auxiliary field into `seals.lock` (such as review metadata, signatures, or extended hashes), any existing or pinned CLI binary reading that lock fails with `LockError`. This forces coordinated lockstep upgrades of all local developer tooling, CI environments, and automated agents.
- Confidence: High
- Marked by authors: No

---

## Not judgeable from this material

1. **Safety of `tar -x` extraction in `GitTree` (`main.rs:614-622`)**: The text notes that `archive REV:prefix piped into tar -x` is executed into a temporary directory, but does not provide the code that invokes the subshell, leaving unverified whether malicious tar archives with absolute paths or symlinks can escape the temporary directory.
2. **File locking on `seals.lock` during concurrent signing**: The material shows `seal sign` writing directly to `seals.lock` (`main.rs:682`), but does not reveal whether file locks (`flock`) are acquired. Concurrent agent executions could clobber entries or produce corrupted TOML.
3. **Ascent limits during orphan deletion in `write` (`contract.rs:99`)**: The contract states that `write` deletes orphaned generated files and their empty parents. Without seeing the directory traversal implementation, it is impossible to verify whether directory removal stops strictly at the specified output directory root or ascends further up the filesystem.
4. **Memory limits under Rayon 64 MiB stacks in constrained containers**: Extraction builds a Rayon thread pool allocating 64 MiB of virtual stack per thread (`isolate.rs:52`). In containerized environments with strict virtual memory (RLIMIT_AS) or cgroup limits, it is unjudgeable whether this allocation triggers premature OOM-kills.
5. **Handling of cycle resolution in macro expansion and syntax trees**: The walker stops at 1,024 expression nesting levels (`collect.rs:1113`), but parsing occurs before the walker. The material does not show whether self-referential or cyclic macro constructs can induce non-terminating loops in `syn` or the token stream normalizer prior to extraction.
