## X-01

**`sealmap verify` audits topic lockfiles rather than the generated corpus, leaving generated diagram drift undetected in CI**
- Evidence: `docs/diagrams/README.md:12-14` claims: "This corpus is **not** `docs/sealmap/`. That directory is sealmap's own generated, one-document-per-source-file output, checked by `sealmap verify` in CI." In contrast, `crates/sealmap/src/main.rs:88-91` binds `Cmd::Verify(Corpus)` to `seal::verify(ctx.lock_text.as_deref(), seal::LOCK_FILE, &ctx.topics, &ctx.codebase)` at `crates/sealmap/src/main.rs:230-234`. `seal::verify` (`crates/sealmap-corpus/src/seal/check.rs:374-411`) only inspects hand-authored topic markdown files under `--diagrams` (`docs/diagrams/`) against `seals.lock`. Checking the generated 1:1 corpus requires `sealmap generate --check -o docs/sealmap` (`crates/sealmap/src/main.rs:59-62`, `246-258`).
- Failure: CI configurations set up according to `README.md` will execute `sealmap verify` expecting it to enforce the 1:1 contract on `docs/sealmap/`. Because `verify` exclusively inspects `docs/diagrams/seals.lock`, any drift, missing files, stale content, or hand edits in `docs/sealmap/` pass CI undetected. Developers and downstream orchestrators will consume drifted, obsolete architecture maps without any CI alert.
- Reviewer confidence: high

## X-02

Merged from near-identical reports; each version follows.

**Closure parameter scope leakage produces silent false call attribution**
- Evidence: EXT-04.3 and `README.md:234`-`236` claim that call resolution is strictly honest ("nothing is guessed silently") and that `Exact` confidence is reserved for calls resolved through explicit types, paths, or imports. In contradiction, EXT-05.6 establishes that closure parameters are never bound in the walker's scope environment (`crates/sealmap-rust/src/collect.rs:1342`-`1345`). If a closure parameter shares a name with an outer variable, method calls on the parameter silently inherit the outer variable's type (`crates/sealmap-rust/tests/extract.rs:392`), minting false `Exact` calls to unrelated types. If no outer variable matches, calls on untyped closure parameters fall back to single-candidate global name guessing (`crates/sealmap-rust/src/resolve.rs:848`, `crates/sealmap-rust/src/resolve.rs:927`), binding calls on arbitrary types (such as `u8`) to whichever internal struct uniquely shares that method name (`crates/sealmap-rust/tests/extract.rs:351`).
- Failure: In standard iterator and stream patterns (e.g., `events.iter().for_each(|client| client.flush())`), if an outer variable named `client` exists with another type (e.g., `DbClient`), calls inside the closure are attributed to `DbClient::flush` with `Exact` confidence. If no outer variable matches, a method call on an unmodelled closure parameter binds to an unrelated internal struct's method. Downstream agents and reviewers receive models and sequence diagrams containing fabricated call edges and false dependencies.
- Reviewer confidence: High. Pinned by existing regression assertions in `crates/sealmap-rust/tests/extract.rs:351` and `crates/sealmap-rust/tests/extract.rs:392`.

**Untyped closure calls can become false internal calls**
- Evidence: Closure parameters are not entered into the walker’s environment (`crates/sealmap-rust/src/collect.rs:1342`-`1345`). The material gives a concrete case where `conn.vacuum()` on a `u8` is matched to the codebase’s only `vacuum` method (`crates/sealmap-rust/tests/extract.rs:351`). The unique-name fallback labels that match Inferred (`crates/sealmap-rust/src/resolve.rs:927`-`932`), and the policy keeps Inferred calls (`crates/sealmap-extract/src/confidence.rs:48`).
- Failure: A generated diagram can show and expand an internal call that never occurs, leading a reviewer to follow a false dependency while missing the real behaviour of the closure.
- Reviewer confidence: high; the incorrect match is shown as a tested case.

## X-03

**Corpus writes can follow symlinks outside the output directory**
- Evidence: `read_rec` includes only entries whose `file_type` is a regular directory or file (`crates/sealmap-corpus/src/contract.rs:117-120`). But `write` joins the expected path and calls `fs::write` without checking for symlinks (`crates/sealmap-corpus/src/contract.rs:140,147-150`).
- Failure: If an expected output path is a symlink to a file outside the corpus directory, verification treats it as missing; writing the corpus follows the symlink and overwrites the target. This is inferred from the filesystem operations shown.
- Reviewer confidence: high; inferred

## X-04

**Markdown citations vanish from verification when a file has no topic ID**
- Evidence: `read_topics_rec` adds a Markdown file to the topics map only when `topic_id` is present (`crates/sealmap/src/main.rs:343`); the verify command checks the topics map loaded by `Ctx` (`crates/sealmap/src/main.rs:202-205`).
- Failure: A new or edited topic with `sym:` citations but a missing front-matter `id:` is omitted from the map, so its citations are not checked for seals and verification can pass. This is inferred from the filtering and call path.
- Reviewer confidence: high; inferred

## X-05

**Default codebase name resolution leaks ambient checkout directory into outputs, breaking determinism**
- Evidence: `crates/sealmap-model/src/lib.rs:18`-`34` defines a strict determinism contract: "no timestamps, absolute paths, user names or environment", guaranteeing that corpus generation and JSON models are byte-identical across runs and environments (`crates/sealmap-corpus/tests/contract.rs:93`). In contradiction, DEL-01.3 and `crates/sealmap/src/main.rs:159`-`161` reveal that if `--name` is omitted (the CLI default), the codebase name is extracted from the local checkout directory name. This ambient string is written directly into `_model.json`, `_index.json` (`crates/sealmap-corpus/src/index.rs:87`), fallback crate roots (`crates/sealmap-rust/src/layout.rs:75`), and overview diagrams.
- Failure: A developer runs `sealmap generate` in a checkout directory named `service-dev` and commits the output. When CI checks out the branch into a standard runner directory (such as `/home/runner/work/service/service` or `/workspace`) and runs `sealmap verify`, the in-memory generated corpus uses the runner's path name. `verify` reports `Modified` drift on `_index.json` and `_model.json` and exits 1, failing CI on clean code. Content-addressed caching across different machines or containers fails due to diverging BLAKE3 digests.
- Reviewer confidence: High. Diagrammed explicitly in DEL-01.3 (`NM -.->|leaks into output| BY`) and implemented in `crates/sealmap/src/main.rs:159`-`161`.

## X-06

Merged from near-identical reports; each version follows.

**Uncatchable stack overflow abort on deeply nested ASTs breaks the fault-isolation invariant**
- Evidence: EXT-01 and EXT-01.1 state as an invariant: "Extraction never fails: unparsable files surface in Extraction::diagnostics (`crates/sealmap-rust/src/lib.rs:148`-`150`)" and "It never fails a whole run because of one bad file: a file that does not parse, or that trips a bug, becomes a warning and a placeholder entry". In contradiction, worker threads isolate panics using `catch_unwind` (`crates/sealmap-extract/src/isolate.rs:46`), but stack overflows in Rust trigger an uncatchable process abort. While expression walking enforces a 1,024 depth cap (`crates/sealmap-rust/src/collect.rs:1113`), `syn::parse_file` runs *before* the flow walker (`crates/sealmap-rust/src/collect.rs:33`) without a recursion limit. Syn consumes ~40 KiB per nesting level in debug builds (`crates/sealmap-extract/src/isolate.rs:3`-`7`).
- Failure: Analyzing a codebase containing deeply nested recursive types, large macro expansions, or machine-generated expressions exhausts the 64 MiB stack during syn's recursive-descent parsing. The operating system aborts the process via SIGSEGV/SIGBUS. The entire CI run or indexing job crashes immediately, producing no diagnostics, warnings, or partial codebase models.
- Reviewer confidence: High.

**A stack overflow can abort the whole extraction run**
- Evidence: The material says one bad file does not fail the whole run and describes per-file panic handling (`crates/sealmap-extract/src/isolate.rs:46`-`47`), but also says a stack overflow cannot be caught and can end the process. The 64 MiB stack and fallback thread do not remove that stated limit (`crates/sealmap-extract/src/isolate.rs:52`-`56`); the expression-depth cap is a separate guard (`crates/sealmap-rust/src/collect.rs:30`).
- Failure: A sufficiently deeply nested file can terminate the process, losing the entire corpus run rather than producing a per-file diagnostic.
- Reviewer confidence: high; inferred production scenario from the stated uncaught stack-overflow limit.

## X-07

**Extra reserved files can survive while verification reports clean**
- Evidence: Extra files are classified as orphaned only when they are Markdown files with the generated marker (`crates/sealmap-corpus/src/contract.rs:96-99`). `write` acts only on the resulting report (`crates/sealmap-corpus/src/contract.rs:138-150`). The CLI can omit `_model.json` with `--no-model` (`crates/sealmap/src/main.rs:217`).
- Failure: Generate once with `_model.json`, then regenerate into the same directory with `--no-model`. The old model is an unrecognised extra, so verification can report clean and leave stale model data available to consumers.
- Reviewer confidence: high

## X-08

**Chained, returned, and computed method calls are silently dropped from sequence flows and call graphs**
- Evidence: `crates/sealmap-extract/src/confidence.rs:30-33` states that under `ExternalCalls::All`, extraction will "Keep every external call, including `std` and unresolved method calls (`sym:? name`)." `crates/sealmap/src/lib.rs:17-18` advertises "sequence diagrams for every function's ordered calls, with `alt`/`opt`/`loop`/`par` fragments for control flow". However, `crates/sealmap-rust/src/resolve.rs:732` resolves method calls via `Callee::Method { recv, name } => self.method(ctx, recv, name)?`. At `crates/sealmap-rust/src/resolve.rs:753`, receiver handling explicitly drops values of unknown type or origin: `Recv::Returned(_) | Recv::Computed(_) | Recv::Unknown => None`. Because `self.method` returns `None`, the `?` operator aborts `call()`, returning `None` before `opts.external_calls.keeps(...)` (`resolve.rs:737`) is ever evaluated.
- Failure: In idiomatic Rust, builder patterns, factory calls, and chained expressions (`let client = make_client(); client.send()`, `conn_pool.get().query()`, or `builder.build().start()`) bind variables to `Recv::Computed` or evaluate receivers as `Recv::Unknown`. All method calls on these receivers are silently dropped from the generated sequence diagrams and call graph relations. Autonomous harness agents reading these diagrams will conclude that the calling functions execute no calls and have no side effects, concealing critical operations (e.g., authorization checks, database updates, metrics publishing) from the agent's context.
- Reviewer confidence: high

## X-09

**The corpus omits some repository Rust source**
- Evidence: COR-01 describes one Markdown document per source file (`crates/sealmap-corpus/src/lib.rs:7`-`53`), but the adapter drops test, example and bench targets unless requested (`crates/sealmap-rust/src/lib.rs:155`). The layout rules say `build.rs` and stray files are never extracted (`crates/sealmap-rust/src/layout.rs:113`-`118`); the CLI exposes a `--tests` flag (`crates/sealmap/src/main.rs:38`-`73`).
- Failure: A production corpus generated with default options can omit tests and build-script code. Changes to omitted files will not appear in its diagrams or corpus drift check, even though build scripts can affect production builds.
- Reviewer confidence: high for the omissions; inferred for the operational impact.

## X-10

**Corpus verification misclassifies all files as stale and misses orphaned files on CRLF checkouts**
- Evidence: `crates/sealmap-model/src/lib.rs:25-28` claims byte-identical determinism and portable content hashes achieved through "normalised paths and newlines". However, `crates/sealmap-corpus/src/contract.rs:84-106` reads disk files in `read_dir_corpus` using standard `fs::read_to_string` without line-ending normalisation. In `crates/sealmap-corpus/src/document.rs:134-144`, `front_matter_hash` hardcodes `text.strip_prefix("---\n")?` and `fm.find("\n---")?`, while `is_generated` hardcodes `text.strip_prefix("---\n")`. On Windows or systems where git checks out files with CRLF (`\r\n`), `strip_prefix("---\n")` fails on `"---\r\n"`, returning `None`. In `crates/sealmap-corpus/src/contract.rs:91-105`, `front_matter_hash(have)` evaluates to `None`, routing every valid file to `_ => Drift::Stale`. Furthermore, `is_generated` returns `false`, causing `verify_against` to skip all unreferenced files instead of classifying them as `Drift::Orphaned`.
- Failure: Any pipeline running on Windows runners or repositories with `core.autocrlf=true` will fail `sealmap generate --check` with false-positive stale reports across 100% of the corpus. Conversely, executing `sealmap generate` (without `--check`) on such checkouts will fail to detect or purge orphaned generated files, leaving discarded documents in the corpus and invalidating 1:1 contract guarantees.
- Reviewer confidence: high
