### F-01 — Arbitrary revision argument passed to shell pipe in `stale --since` and `pack --diff`
- Topics: COR-05.2, DEL-02.3
- Evidence: In COR-05.2, `stale --since REV` executes an external pipeline: `GT->>GI: archive REV:prefix piped into tar -x, temp dir (main.rs:614-622)`. DEL-02.3 confirms that `--since` and `--diff` export revisions by invoking `git archive` piped into tar (`main.rs:608-617`). The material claims git stays out of the libraries (`docs/DESIGN.md:166-169`), but the CLI takes raw user strings for `REV` and constructs command pipelines.
- Failure: In automated harnesses or CI pipelines where `REV` is supplied from an untrusted Git reference, branch name, or PR head, a revision string containing shell metacharacters or flags starting with `-` (flag injection into `git archive`) can execute arbitrary commands or manipulate archive destination paths in the runner environment.
- Confidence: medium (inferred pipeline invocation details from `main.rs:614-622`)
- Marked by authors: no

### F-02 — Destructive file and parent directory deletion during corpus generation
- Topics: COR-01.2, COR-03.3
- Evidence: COR-03.3 specifies that `write` "deletes it [orphaned file] and empty parents (`crates/sealmap-corpus/src/contract.rs:99`, `crates/sealmap-corpus/src/document.rs:153-155`)" for any Markdown file whose front matter begins with `sealmap: `. In COR-03.1, `generate PATH -o DIR` writes directly to any target path passed on the command line.
- Failure: If an operator points `-o DIR` to an existing documentation root, project directory, or shared directory containing Markdown documents with front matter starting with `--- \n sealmap: `, `sealmap generate` deletes all unmatched Markdown files as "orphaned" and recursively deletes empty parent directories up the filesystem tree.
- Confidence: high
- Marked by authors: no

### F-03 — Non-atomic two-phase write in `seal sign` corrupts repository state on crash
- Topics: COR-05.3, COR-04.4
- Evidence: COR-05.3 records that `seal sign` performs two separate sequential file writes: `CLI->>CLI: topic written only if the pointer changed it (main.rs:679-680)` followed by `CLI->>CLI: lock written canonically (main.rs:682)`. COR-05.3 documents as an invariant: "a failure between the two writes leaves a pointer with no entry, which `verify` reports as a lock fault (`crates/sealmap/src/main.rs:680-682`, `crates/sealmap-corpus/src/seal/check.rs:383`)".
- Failure: A crash, process kill, CI cancellation, or disk-full error occurring after the topic file is written but before `seals.lock` is flushed leaves a dangling `sealed:` pointer on disk. All subsequent executions of `sealmap verify` fail with a fatal `LockFault`, halting CI pipelines until a developer manually resolves or re-signs the lockfile.
- Confidence: high
- Marked by authors: no

### F-04 — Windows checkout CRLF causes byte-for-byte canonicality failure in `verify`
- Topics: COR-04.2, COR-04.4
- Evidence: COR-04.2 states that `parse` normalises line endings, but `parse_canonical` strictly requires that `to_toml equals the text byte for byte (lock.rs:233)`, otherwise returning `NonCanonical (lock.rs:234)`. COR-04.4 shows `VF->>VF: text not canonical, LockFault (check.rs:372-373)`.
- Failure: On Windows systems where Git checks out text files with `\r\n` line endings (`core.autocrlf = true`), `seals.lock` on disk contains CRLF. Because the internal writer produces `\n`, the byte-for-byte comparison in `parse_canonical` fails, causing `sealmap verify` to exit with status 1 on unmodified repositories.
- Confidence: high
- Marked by authors: no

### F-05 — Multi-repo flag `--repo` unconditionally rejected by `stale --since` and `pack --diff`
- Topics: COR-05.2, COR-03.5
- Evidence: COR-03.5 claims source flags are shared across all commands, explicitly noting: "Several repositories are loaded into one `SourceSet` under name prefixes... sealmap/src/lib.rs:112-113... CMD->>SITE: seal commands and pack (main.rs:156-165)". However, COR-05.2 states that `stale --since` (and `pack --diff`, which shares `model_at`) unconditionally aborts when `--repo` is passed: `ST->>ST: refuse with --repo (main.rs:461-462)`.
- Failure: Teams running multi-repository workspaces cannot use `sealmap stale --since REV` or `sealmap pack --diff REV`. Any automation or agent attempting change detection across multiple repositories crashes with an error.
- Confidence: high
- Marked by authors: no

### F-06 — Documentation code spans trigger false `UnsealedCitation` failures in `verify`
- Topics: COR-04.3, DEL-02.6
- Evidence: DESIGN §3 states that legacy or unsealed topics serve as coverage and do not cause check failures (`docs/DESIGN.md:119-121`, `COR-04.4`). However, COR-04.3 scans any code span outside code fences that starts with `sym:`, a manager, or a space as a citation (`topic.rs:331`, `topic.rs:182`). DEL-02.6 documents that running `sealmap verify` against its own documentation failed: "MER-02 writes an example id from a fixture crate (`shop`) as an inline code span, which reads as a citation of a symbol nobody sealed... V-->>T: an unsealed citation, exit 1".
- Failure: Any topic file that includes an illustrative `sym:` identifier in backtick code spans causes `sealmap verify` to fail with status 1. Documentation explaining syntax cannot co-exist with the seal gate unless examples are intentionally malformed or backed by real definitions in the source code.
- Confidence: high
- Marked by authors: yes (dogfood triage in DEL-02.6)

### F-07 — Downstream MSRV build failure on Rust 1.85 due to unpinned `ignore 0.4.30`
- Topics: DEL-01.2, DEL-01
- Evidence: DEL-01 guarantees a minimum supported Rust version of 1.85 (`ci.yml:39`, `Cargo.toml:8`). However, DEL-01.2 documents that the workspace dependency specification is `ignore = ">=0.4.23"` (`Cargo.toml:35`). `ignore 0.4.30` requires a newer toolchain than 1.85 and publishes no `rust-version`.
- Failure: While the repository builds via its pinned `Cargo.lock`, any external crate depending on published `sealmap` libraries on Rust 1.85 without that lockfile resolves `ignore 0.4.30` and fails during compilation.
- Confidence: high
- Marked by authors: yes (caveat in DEL-01.2)

### F-08 — Silent edge truncation without omitted count comment in Overview Trait Map
- Topics: COR-02.5, COR-02
- Evidence: COR-02 asserts that "a diagram over its message or edge budget is cut, with a note saying how much was left out and where the full list is, rather than rendered unreadably or not at all". In contrast, COR-02.5 states for trait maps: `TT: first max_edges relations in key order, no omitted count overview.rs:236`.
- Failure: In repositories with more than 300 trait relations, `overview.rs` silently drops excess relations in key order without leaving an omitted count or comment. Users and models examining the trait map receive an incomplete architecture map with no indication that relations were truncated.
- Confidence: high
- Marked by authors: no

### F-09 — Unpinned codebase name leaks checkout path into fallback crate symbol IDs
- Topics: DEL-01.3, EXT-01.2, COR-03.1
- Evidence: DEL-01.3 guarantees byte-identical models and corpus output across machines (`crates/sealmap-model/src/lib.rs:18-34`). However, DEL-01.3 admits that the codebase name "comes from outside the source bytes unless it is pinned... leaks into output (main.rs:393, main.rs:419-424)". EXT-01.2 shows that source files outside a Cargo package fall back to a crate named after the codebase (`layout.rs:143`).
- Failure: When `--name` is omitted on the CLI, cloning a repository into differently named directories (e.g. CI ephemeral runners vs local checkouts) produces different `SymbolId` values for fallback crates. Generated corpus files, hashes, and seal locks diverge across machines.
- Confidence: high
- Marked by authors: no

### F-10 — Source files exceeding 2 MiB silently dropped without placeholder or diagnostic
- Topics: MOD-03.2, EXT-01.4
- Evidence: EXT-01 claims that extraction "never fails a whole run because of one bad file: a file that does not parse, or that trips a bug, becomes a warning and a placeholder entry... Keeping a module symbol for a failed file keeps the 1:1 corpus contract (`crates/sealmap-rust/src/collect.rs:57-59`)". However, MOD-03.2 shows that `SourceSet::load_dir` skips large files completely: `alt extension wanted and under 2 MiB (source.rs:175-177)... else extension not wanted or too big: skipped silently`.
- Failure: Any source file larger than 2 MiB (such as generated code, embedded lookup tables, or large AST definitions) is discarded without producing a diagnostic or placeholder module. Symbols inside those files vanish from the model, and seal checks flag them as `Absent` rather than `Unparsable`.
- Confidence: high
- Marked by authors: no

### F-11 — Process abort via uncatchable stack overflow on deeply nested code
- Topics: EXT-01.3, EXT-01
- Evidence: EXT-01 states that syn parsing requires approximately 40 KiB of stack per nesting level in debug mode (`crates/sealmap-extract/src/isolate.rs:3-7`). Although thread stacks are expanded to 64 MiB (`isolate.rs:52`), EXT-01 explicitly acknowledges: "The residual risk is a file nested deeply enough to exhaust even the large stack: that still ends the process, because a stack overflow cannot be caught".
- Failure: Deeply nested type expressions or macro-expanded structures cause a thread stack overflow during parsing. Because Rust stack overflows abort the process via guard pages without unwinding, the panic isolation in `map_isolated` is bypassed and the entire CLI process terminates immediately.
- Confidence: high
- Marked by authors: yes (residual risk in EXT-01)

### F-12 — Execution order inversion and dropped calls for let-bound closures in Flow Walker
- Topics: EXT-03.4, EXT-03
- Evidence: EXT-03 claims sequence diagrams depict actual execution order. However, EXT-03.4 documents that `Stmt::Local` walks closure bodies at the definition site (`collect.rs:1267`) and emits an optional fragment there, while `Resolver::call` recognizes subsequent calls through the variable as local closures and drops them (`resolve.rs:839-840`). The authors note: "A closure saved in a variable is drawn where it is written, not where it is called... Both are recorded below as debt."
- Failure: Sequence diagrams for functions that assign closures to local variables and call them later (such as deferred execution or cleanup routines) display those actions during variable initialization and omit the call at the actual execution site, presenting an inverted timeline of control flow.
- Confidence: high
- Marked by authors: yes (debt in EXT-03)

### F-13 — False call edges and incorrect scopes generated from untyped closure parameters
- Topics: EXT-05.6, EXT-05
- Evidence: EXT-05 claims calls are resolved accurately or mapped to `sym:?` without silent guesses. However, EXT-05.6 documents: `closure argument walked with the environment cloned, parameters never bound (collect.rs:1348-1351)`. A method call on a closure parameter falls back to a name guess (`resolve.rs:866`), causing `vacuum` called on a `u8` parameter to resolve to an internal database `vacuum` method (`tests/extract.rs:351`). Furthermore, `a parameter that shadows an outer typed name keeps the outer type`.
- Failure: Method calls on closure parameters (e.g. iterating over collections via `.for_each(|x| x.process())`) either attach to unrelated single-candidate methods in other crates or resolve against shadowed variables in outer scopes, generating false architectural dependencies and incorrect sequence diagram arrows.
- Confidence: high
- Marked by authors: yes (debt in EXT-05)

### F-14 — Binary crate renaming upon addition of `lib.rs` invalidates existing symbol IDs
- Topics: EXT-01.2, EXT-02
- Evidence: EXT-02 claims definition IDs remain stable across code reorganizations (`crates/sealmap-extract/src/ids.rs:20-28`, `README.md:177-179`). However, EXT-01.2 defines the layout rule: `MAIN: src/main.rs? -> BIN: binary crate; pkg_main when a lib exists (layout.rs:170)`.
- Failure: When a project with only `src/main.rs` adds a `src/lib.rs`, the binary crate name changes from `pkg` to `pkg_main`. Every `SymbolId` in `main.rs` changes from `sym:cargo pkg . ...` to `sym:cargo pkg_main . ...`. All existing seals, review packs, and citations referencing `main.rs` are invalidated as `Absent`.
- Confidence: high
- Marked by authors: no

### F-15 — Single over-budget topic causes total pack generation abort in `pack --shard`
- Topics: COR-06.3, COR-06.2
- Evidence: COR-06 states: "shard instead fills numbered packs with whole topics and refuses only a topic too large to fit alone (`crates/sealmap-corpus/src/pack.rs:295-331`)". COR-06.2 notes that `Dense.slice` is taken without a byte limit (`pack.rs:479-481`).
- Failure: If a single topic cites a hub function whose callers, callees, and dense skeleton exceed the pack `--budget`, `shard` terminates with `TopicOverBudget (src/pack.rs:318)` and `main.rs:736-743` exits with status 1 without outputting any files. The packaging workflow fails completely rather than isolating the oversized topic.
- Confidence: high
- Marked by authors: no

## Not judgeable from this material

1. Command execution safety in `sealmap/src/main.rs:608-622`: whether arguments passed to `git archive` and `tar -x` are escaped or passed as direct argv vectors without shell evaluation.
2. Directory traversal boundaries in `crates/sealmap-corpus/src/contract.rs:99-141`: whether file and empty parent directory deletion is strictly confined within the canonicalized `-o DIR` path.
3. Concurrency handling and file locking in `crates/sealmap-corpus/src/seal/sign.rs`: whether concurrent executions of `seal sign` on separate topics lock `seals.lock` or risk clobbering each other's entries.
4. Parsing behavior on cyclic symlinks in `SourceSet::load_dir` (`crates/sealmap-model/src/source.rs:154-183`): whether the `ignore` walker safely handles circular directory links without hanging.
5. Inherent method precedence implementation in `crates/sealmap-rust/src/resolve.rs:557-559`: whether a private inherent method can shadow an in-scope public trait method of the same name.
