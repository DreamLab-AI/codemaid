### F-01 — Command option injection via unvalidated git revision string
- Topics: COR-05.2, COR-06.1
- Evidence: The CLI invokes `git archive REV:prefix piped into tar -x, temp dir` (`crates/sealmap/src/main.rs:608`-`622`, `crates/sealmap/src/main.rs:464`). The material claims git stays out of the library and is executed by the CLI (`docs/DESIGN.md:166`-`169`), but shows no validation or sanitisation on the user-supplied `REV` parameter from `--since REV` or `--diff REV`.
- Failure: If `REV` is supplied from an untrusted source (e.g. branch names in automated CI triggers or webhooks), arguments beginning with `--` (such as `--exec=<command>` or `--output=<path>`) can be passed directly to `git archive`, enabling arbitrary command execution or arbitrary file overwrite on the host running the CLI.
- Confidence: medium (inferred from subprocess invocation architecture in `main.rs:608`-`622`)
- Marked by authors: no

### F-02 — Destructive file and directory deletion outside generated boundaries
- Topics: COR-01.2, COR-03.3
- Evidence: `write` purges files deemed "orphaned" and deletes empty parent directories (`crates/sealmap-corpus/src/contract.rs:99`-`100`, `crates/sealmap-corpus/src/contract.rs:138`-`141`). While it checks that a Markdown file opens with `---` and a `sealmap: ` key (`crates/sealmap-corpus/src/document.rs:10`-`12`, `crates/sealmap-corpus/src/document.rs:153`-`155`), it unconditionally removes parent directories once emptied.
- Failure: If a user specifies an output path pointing to an existing directory containing project documentation or runs with `-o .`, any markdown file containing the front-matter prefix is deleted, and its parent directories are recursively removed from disk upon becoming empty, causing irreversible data loss in repository trees.
- Confidence: high
- Marked by authors: no

### F-03 — Non-atomic seal signing corrupts repository verification state on crash
- Topics: COR-04.4, COR-05.3
- Evidence: `seal sign` executes a non-atomic two-phase write: it writes the modified topic file first (`crates/sealmap/src/main.rs:679`-`680`), and only afterwards writes `seals.lock` canonically (`crates/sealmap/src/main.rs:682`). The authors note that an interrupted write leaves a pointer with no lock entry, causing `verify` to report `LockFault` (`crates/sealmap-corpus/src/seal/check.rs:383`).
- Failure: An interruption (SIGINT, process termination, disk exhaustion) between writing the topic and writing the lockfile leaves the repository in a corrupted state where the topic cites a seal that does not exist in `seals.lock`. All subsequent CI verification runs fail closed with `LockFault`, blocking deployment until manual file restoration occurs.
- Confidence: high
- Marked by authors: no

### F-04 — Windows CRLF checkouts break verification canonicality check
- Topics: COR-04.2, COR-04.4
- Evidence: `parse` normalises CRLF line endings, but `parse_canonical` requires that `to_toml` matches the file text byte-for-byte (`crates/sealmap-corpus/src/seal/lock.rs:229`-`235`). `to_toml` emits LF newlines (`crates/sealmap-corpus/src/seal/lock.rs:248`). When non-canonical, `verify` reports a `LockFault` (`crates/sealmap-corpus/src/seal/check.rs:372`-`373`).
- Failure: On Windows environments where Git checks out text files with `\r\n` (e.g. `core.autocrlf = true`), `seals.lock` contains `\r\n`. Even if the contents are otherwise unmodified, `parse_canonical` fails byte equality against LF-formatted `to_toml`, causing `sealmap verify` to fail CI across the entire repository.
- Confidence: high
- Marked by authors: no

### F-05 — Prose mentions of fixture identifiers trigger unresolvable verification failures
- Topics: COR-04.3, COR-04.6, DEL-02.6
- Evidence: Every CommonMark code span starting like a global id (`sym:<manager> `) is extracted as a formal symbol citation (`crates/sealmap-corpus/src/seal/topic.rs:284`-`331`). In `DEL-02.6`, MER-02 cites a fixture id from an external fixture crate in prose, which `sealmap verify` treats as an unsealed citation, causing exit code 1 (`crates/sealmap-corpus/src/seal/check.rs:390`). However, `seal sign` refuses any citation that cannot be resolved in the model (`crates/sealmap-corpus/src/seal/sign.rs:125`-`130`).
- Failure: Any topic that quotes an illustrative, external, or non-extracted `sym:` identifier in backticks cannot be verified (fails as `UnsealedCitation`) and cannot be signed (signing fails because the symbol does not resolve in the codebase). The repository verification gate permanently deadlocks unless prose formatting is altered.
- Confidence: high
- Marked by authors: no

### F-06 — Untyped closure parameters synthesize spurious cross-crate call edges
- Topics: EXT-05.5, EXT-05.6
- Evidence: Closure parameters are never added to the walker environment and remain `Untyped` (`crates/sealmap-rust/src/collect.rs:1348`-`1351`, `crates/sealmap-rust/src/collect.rs:1384`). Method calls on untyped receivers fall back to `by_name_only` matching across all crates the caller can reach (`crates/sealmap-rust/src/resolve.rs:866`, `crates/sealmap-rust/src/resolve.rs:954`). If an outer variable shares the parameter name, the parameter silently inherits the outer variable's type.
- Failure: Standard iterator chains (e.g. `xs.iter().for_each(|conn| conn.vacuum())`) bind methods on primitive closure arguments (like `u8`) to completely unrelated workspace types that happen to have a unique `vacuum()` method (`crates/sealmap-rust/tests/extract.rs:351`), generating false dependency edges and misleading call flows in the codebase model.
- Confidence: high
- Marked by authors: yes (Debt)

### F-07 — Incomplete macro parsing silently drops call graph edges
- Topics: EXT-02, EXT-03.6, EXT-04.4
- Evidence: Macro bodies that do not parse as standard comma-separated expressions or statement lists are skipped without extracting calls (`crates/sealmap-rust/src/collect.rs:1366`-`1376`). The macro invocation itself is never recorded as a call step (`crates/sealmap-model/src/flow.rs:160`). Items generated by macro expansion are never indexed (`README.md:342`). Furthermore, prelude functions like `drop` are dropped before reaching external call policies (`crates/sealmap-rust/src/resolve.rs:833`).
- Failure: Callables that rely on common macro-based control flow (e.g., DSLs, routing tables, logging wrappers, `tokio::select!`) have their internal call edges omitted. Reviewers and models reading the resulting sequences or dense projections observe broken control flow graphs with missing execution paths.
- Confidence: high
- Marked by authors: yes (Debt)

### F-08 — Untracked files leak into model while pack revision claims clean state
- Topics: COR-06.1, MOD-03.2
- Evidence: `sealmap pack` sets the revision identifier to `HEAD`, appending `+dirty` only for modified *tracked* files (`crates/sealmap/src/main.rs:767`-`770`). Conversely, `SourceSet::load_dir` loads all `.rs` files under the root that pass `.gitignore` checks, regardless of Git tracking (`crates/sealmap-model/src/source.rs:154`-`183`).
- Failure: If a developer adds a new `.rs` file that is not yet tracked by Git, the generated review pack incorporates the new file's symbols and flows, but the pack header claims an un-dirty `HEAD` commit. Two runs against the same commit hash yield different output while claiming identical revision provenance.
- Confidence: high
- Marked by authors: no

### F-09 — Ambient directory path leaks into canonical symbol IDs and invalidates seals
- Topics: COR-05, DEL-01.3, EXT-01.2
- Evidence: When `--name` is omitted and no root Cargo package defines the codebase name, the name falls back to the directory name of the checkout (`crates/sealmap-rust/src/layout.rs:143`, `crates/sealmap/src/main.rs:393`). Symbol identifiers incorporate this name as the package component (`crates/sealmap-model/src/sym.rs:131`-`132`).
- Failure: When code is checked out in a directory with a name differing from the author's local directory (common in CI runner environments like `/home/runner/work/repo/repo`), all minted `SymbolId`s change. Sealed topics signed locally report all cited symbols as `Absent` in CI, causing verification to fail.
- Confidence: high
- Marked by authors: no

### F-10 — Uncatchable stack overflow on deep syn parsing crashes worker process
- Topics: EXT-01, EXT-01.3
- Evidence: Collection runs on worker threads with 64 MiB stacks under `catch_unwind` (`crates/sealmap-extract/src/isolate.rs:38`-`56`). The material acknowledges that stack overflow cannot be caught and terminates the process (`crates/sealmap-extract/src/isolate.rs:3`-`7`). Syn consumes ~40 KiB per expression nesting level in debug mode.
- Failure: A source file with deeply nested types or macros (~1,600 levels of nesting) causes an uncatchable stack overflow during syn parsing. The abort kills the host process immediately, defeating `catch_unwind` isolation and halting CI or server pipelines.
- Confidence: high
- Marked by authors: yes (Debt)

### F-11 — `stale` command unconditionally exits with code 0 on failure
- Topics: COR-05.2, COR-05.4
- Evidence: `stale` is explicitly hardcoded to exit 0 on all runs (`crates/sealmap/src/main.rs:586`, `crates/sealmap/src/main.rs:599`), even when Git export fails or errors occur (`crates/sealmap/src/main.rs:630`-`634`).
- Failure: Automated CI scripts or maintenance bots running `sealmap stale --since <rev>` to detect out-of-date diagrams cannot detect failures via exit status. If `git archive` fails, an invalid revision is passed, or diffing fails, the tool exits 0 with empty or partial results, silently bypassing automated checks.
- Confidence: high
- Marked by authors: no

### F-12 — Unpinned `ignore` dependency breaks declared MSRV for downstream consumers
- Topics: DEL-01, DEL-01.2
- Evidence: The workspace specifies MSRV 1.85 (`Cargo.toml:8`), while depending on `ignore = ">= 0.4.23"` (`Cargo.toml:35`). The repository lockfile pins `ignore` to 0.4.29, but downstream library consumers without a lockfile resolve `ignore` 0.4.30 (`Cargo.lock:263`-`264`). `ignore` 0.4.30 lacks a `rust-version` field and fails to compile on Rust 1.85.
- Failure: Any downstream application declaring a dependency on `sealmap` or `sealmap-model` that builds under the documented MSRV of Rust 1.85 fails to compile during dependency resolution unless they manually identify and pin `ignore`.
- Confidence: high
- Marked by authors: yes (Tension)

### F-13 — Verification gate is unenforced in CI workflow
- Topics: DEL-01.1, DEL-02.6
- Evidence: The CI pipeline executes fmt, clippy, unit tests, determinism diffs, Mermaid headless rendering, and legacy citation checks (`.github/workflows/ci.yml:9`-`84`). `DEL-02.6` notes: "no CI job runs verify yet (ci.yml:9-84)."
- Failure: The headline guarantee—that diagrams in the repository are validated against code changes via `sealmap verify`—is not running in automated integration. Code changes that break diagram assertions can be merged without failing CI.
- Confidence: high
- Marked by authors: no

### F-14 — Review pack sharding aborts entirely if a single topic exceeds budget
- Topics: COR-06.1, COR-06.3
- Evidence: `sealmap pack --shard` evaluates topics one by one. If an individual topic's rendered section exceeds the `--budget` on its own, `shard` halts and returns `TopicOverBudget` with exit code 1 (`crates/sealmap-corpus/src/pack.rs:295`-`331`, `crates/sealmap/src/main.rs:736`-`743`). Dense slices for topics are generated without byte limits (`crates/sealmap-corpus/src/pack.rs:479`-`481`).
- Failure: In large systems where an architectural topic cites high-degree hub symbols, that topic's dense slice and source window will easily exceed a typical review budget. Instead of isolating the oversized topic or packing the remaining topics, the entire packing command aborts with no output files generated.
- Confidence: high
- Marked by authors: no

### F-15 — Unindexed linear search in corpus expansion linking creates quadratic bottleneck
- Topics: COR-01.3, COR-01.4
- Evidence: While symbol lookup was hardened to avoid $O(\text{files} \times \text{symbols})$ scans (`crates/sealmap-corpus/src/lib.rs:227`), `Index::fragment(id)` remains an unindexed linear scan over all documents and fragments (`crates/sealmap-corpus/src/index.rs:133`). In `link_expansions`, the generator iterates over every call of every fragment and resolves its target using this linear search (`crates/sealmap-corpus/src/index.rs:112`-`119`).
- Failure: In repositories with thousands of files and tens of thousands of calls, `link_expansions` exhibits $O(\text{calls} \times \text{fragments})$ quadratic scaling, causing severe CPU and generation latency spikes during full corpus or index generation.
- Confidence: high
- Marked by authors: no

---

## Not judgeable from this material

1. **Subprocess argument escaping in `main.rs:608`-`646`**: The exact implementation of how `REV` and `prefix` are passed to `git rev-parse` and `git archive` (whether using raw `std::process::Command` argument arrays or shell string interpolation) cannot be confirmed without the source.
2. **System clock date calculation in `main.rs:793`-`800`**: The custom 7-line UTC date calculation algorithm cannot be verified for leap-year, leap-second, or month-boundary edge cases that could format invalid dates and break `seals.lock` validation.
3. **Atomic file write implementation in `contract.rs` and `main.rs`**: It cannot be determined whether `seals.lock` and Markdown outputs are written using `tempfile` and atomic rename or truncated in place, which dictates resilience against corruption during unexpected termination.
4. **Peak heap memory during whole-repository dense graph slicing**: The memory overhead of building and retaining full `Dense` call graphs and token streams across multi-thousand-file codebases cannot be measured from the documentation alone.
5. **Concurrent execution safety**: It cannot be determined whether running multiple concurrent `sealmap` commands against the same target directory or `.sealmap/` folder uses file locking or risks race conditions and data corruption.
