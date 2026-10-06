## X-01

**`write` can replace a non-generated file at a corpus path**
- Evidence: An existing expected path that differs from generated output is classified as modified (`crates/sealmap-corpus/src/contract.rs:80-90`), and `write` rewrites modified files (`crates/sealmap-corpus/src/contract.rs:138-141`). The marker check is described for orphan deletion (`crates/sealmap-corpus/src/contract.rs:99-100`), but no marker check before rewriting an expected path appears in the material.
- Failure: A hand-authored file occupying an expected corpus path can be overwritten by `write`, even if it has no generator marker.
- Reviewer confidence: medium; inferred from the path-based comparison and rewrite behaviour.

## X-02

**Oversized source files disappear from generated checks**
- Evidence: `SourceSet::load_dir` drops files that exceed 2 MiB, and the diagram says skipped files are dropped silently (`crates/sealmap-model/src/source.rs:175-183`). `verify` regenerates from the loaded source set before comparing the corpus (`crates/sealmap/src/main.rs:107-146`).
- Failure: Once a large source file is absent from the corpus, later edits to it cannot affect the regenerated model or corpus; `verify` can pass without detecting those edits.
- Reviewer confidence: high; inferred from the loader and verification path.

## X-03

**Untyped closure parameters and lexical shadowing fabricate false call-graph edges in `_index.json`**
- Evidence: The model guarantees that call targets reflect true program flow ("who talks to whom, in what order", `crates/sealmap-model/src/flow.rs:4`-`8`), that "nothing is guessed silently" (`README.md:234`-`236`), and that index expansion links are direct lookups rather than guesses (`crates/sealmap-corpus/src/lib.rs:45`-`48`). In contrast, `EXT-05.6` shows that closure parameters are never bound in the collector's environment (`crates/sealmap-rust/src/collect.rs:1307`-`1310`) and are treated as `Untyped` (`crates/sealmap-rust/src/collect.rs:1343`). If a method is called on a closure parameter:
  1. If the parameter shadows an outer variable name, it resolves against the outer variable's type.
  2. If not, it falls back to `by_name_only` (`crates/sealmap-rust/src/resolve.rs:846`), which binds to any single internal method in the entire codebase sharing that name (e.g. `conn.vacuum()` binds to an internal `vacuum` method even when `conn` is a `u8`, `crates/sealmap-rust/tests/extract.rs:351`).
- Failure: Standard iterator chains (`items.iter().for_each(|entry| entry.execute())`) will either resolve method calls against unrelated shadowed outer variables or guess a completely unrelated internal function across different crates if its method name is unique in the workspace. In `_index.json`, the call's `expands` field (`crates/sealmap-corpus/src/index.rs:119`) links to the sequence fragment of the falsely matched function. Autonomous orchestrator agents following `expands` links will trace cross-file execution into unrelated subsystems, generating corrupt diffs and invalid architectural inferences.
- Reviewer confidence: high

## X-04

**Ambient checkout directory leaks into codebase identity, breaking determinism and CI verification**
- Evidence: `crates/sealmap-model/src/lib.rs:18`-`34` defines a strict determinism contract: identical inputs must produce byte-identical output across runs and machines, explicitly prohibiting "timestamps, absolute paths, user names or environment" (`crates/sealmap-model/src/lib.rs:33`). In contrast, `COR-03.5` and `DEL-01.3` show that the CLI falls back to the host machine's checkout directory name as the codebase name whenever the optional `--name` flag is omitted (`crates/sealmap/src/main.rs:159`-`161`).
- Failure: When run locally by developers and remotely in CI, local checkouts in different folder paths (e.g., `/Users/dev/work/project` versus `/runner/work/1/s`) derive different codebase names for the exact same source commit. The codebase name is embedded in `Codebase`, `_model.json`, `_index.json`, and overview diagrams. Running `sealmap verify` in CI will flag spurious drift between the local corpus and the CI run, exiting 1 (`crates/sealmap/src/main.rs:146`) and blocking deployment pipelines unless every caller explicitly passes `--name`.
- Reviewer confidence: high

## X-05

**Scoped `if let` and `while let` pattern bindings leak into outer function scope**
- Evidence: `FlowWalker::cond` (`crates/sealmap-rust/src/collect.rs:880-885`) executes `self.bind(&l.pat, Some(&l.expr))` for `Expr::Let` guards, mutating `self.env` directly. While `Expr::Match` (`crates/sealmap-rust/src/collect.rs:834-837`) and `Expr::ForLoop` (`crates/sealmap-rust/src/collect.rs:848-851`) clone and restore `self.env`, `Expr::If` (`crates/sealmap-rust/src/collect.rs:795-824`) and `Expr::While` (`crates/sealmap-rust/src/collect.rs:855-863`) invoke `self.cond` without snapshotting or restoring the outer environment.
- Failure: Pattern-bound identifiers from `if let` and `while let` guards leak into the enclosing function body and persist for all subsequent statements and `else` blocks. If an identifier with the same name is later used in an unannotated `let` binding, field access, or closure in the remainder of the function, the walker resolves its receiver to the leaked `Recv` from the earlier condition. Calls on the subsequent variable are erroneously resolved to methods of the earlier pattern-matched type, generating corrupt call edges and misleading sequence diagram participants.
- Reviewer confidence: high

## X-06

**`constructed_type` assigns constructor receiver types to arbitrary static method returns**
- Evidence: `crates/sealmap-rust/src/collect.rs:987-997` claims to recognize constructors (`Foo::new(..) -> Foo`), but its implementation matches any path call with at least two segments whose penultimate segment starts with an ASCII uppercase letter (`Expr::Path(p) if p.path.segments.len() >= 2 => ty.starts_with(uppercase)`). It unconditionally extracts that segment as the constructed type for the variable binding (`crates/sealmap-rust/src/collect.rs:720`).
- Failure: Any non-constructor associated function or static method on a type (e.g., `let ok = Response::is_success(&resp);`, `let count = Metrics::total();`, `let guard = Mutex::lock(&m);`, or `let id = Uuid::nil();`) falsely tags the bound variable (`ok`, `count`, `guard`, `id`) as having the type of the enclosing type (`Response`, `Metrics`, `Mutex`, `Uuid`). Subsequent method calls on those variables (such as `guard.unlock()` or `id.to_string()`) are resolved against the wrong type, creating spurious `Exact` relations to unrelated internal methods or minting phantom `undefined_member` external endpoints in class and sequence diagrams.
- Reviewer confidence: high

## X-07

**Method calls on generic parameters are silently dropped from all flows and diagrams**
- Evidence: EXT-04 claims all internal calls are preserved with confidence lattice semantics and `ExternalCalls::NonStd` only drops `std`/`core`/`alloc` and unresolved external calls (`crates/sealmap-extract/src/confidence.rs:36-47`). In `crates/sealmap-rust/src/resolve.rs:753-755`, `receiver_type` explicitly returns `None` for any single-letter or digit-suffixed generic type name (`is_generic_param`), forcing `method` (`crates/sealmap-rust/src/resolve.rs:708`) to fall back to `ids::unresolved_method_id(name)` (`sym:? <name>`) with `Confidence::External`. Under default options (`ExternalCalls::NonStd`), `crates/sealmap-rust/src/resolve.rs:671-678` queries `is_dependency`, which returns `false` because `target.root()` on an unresolved method is empty.
- Failure: In any generic or trait-heavy Rust codebase (e.g., functions of the form `fn process<W: Writer>(w: &mut W)` or `fn handle<S: Service>(s: S)` calling `w.flush()` or `s.call()`), all method invocations on generic type parameters are silently discarded during lowering. They never appear in `Flow`, produce no `RelationKind::Calls` relations, emit no sequence diagram messages, and vanish from `_index.json`. Downstream agent orchestrators relying on the index for inter-procedural flow receive disconnected call graphs.
- Reviewer confidence: high

## X-08

**Repositories with the same name silently overwrite each other**
- Evidence: `load_repos` prefixes each repository’s files with its supplied name and inserts them into one `SourceSet` (`crates/sealmap/src/lib.rs:105–116`). `SourceSet::insert` replaces an existing file at the same path (`crates/sealmap-model/src/source.rs:99–104`).
- Failure: Two repo arguments use the same name. Files with matching relative paths map to the same prefixed path, and the later repository replaces the earlier one before extraction.
- Reviewer confidence: high; inferred from the path construction and replacement behaviour

## X-09

**`#[cfg]` definitions can lose all but the first call flow**
- Evidence: `add_symbol` says it handles `#[cfg]` duplicates, but its duplicate branch merges fingerprints, members and tags without merging `flow` (`crates/sealmap-model/src/codebase.rs:64–83`). The existing symbol therefore keeps the first definition’s flow.
- Failure: A function with platform-specific `#[cfg]` definitions has different calls on Unix and Windows. The generated model retains only the first collected flow, so the other platform’s calls are absent.
- Reviewer confidence: high; inferred from the duplicate-merge branch

## X-10

**The designed seal gate is not implemented**
- Evidence: The design’s seal layer is labelled “NOT BUILT”, and its planned `verify` is a seal check (`docs/DESIGN.md:44-46`, `docs/DESIGN.md:117`). The current CLI offers `generate`, `verify` and `model`; current `verify` compares generated corpus files (`crates/sealmap/src/main.rs:28-36`, `crates/sealmap/src/main.rs:119-146`).
- Failure: An adopter using today’s `verify` as the planned reviewed-topic gate gets only a corpus drift check; it cannot check whether reviewed topics still match the code.
- Reviewer confidence: high; the missing implementation is explicit, and the production consequence is inferred.

## X-11

**Silent omission of legitimate code directories matching hardcoded directory names**
- Evidence: `EXT-01.1` and `COR-01.2` state that generation yields "exactly one Markdown document per source file" (`crates/sealmap-corpus/src/lib.rs:7`-`53`), that "extraction never fails" and records unparsable files as diagnostics (`crates/sealmap-rust/src/lib.rs:148`-`150`). However, `MOD-03.2` shows that `SourceSet::load_dir` hardcodes an exclusion filter that prunes any directory named `target`, `node_modules`, `vendor`, `dist`, `build`, or `out` anywhere below the root (`crates/sealmap-model/src/source.rs:163`), skipping all matching files silently without diagnostics or fallback representations (`crates/sealmap-model/src/source.rs:175`-`177`, `crates/sealmap-model/src/source.rs:183`).
- Failure: In repositories containing subcrates, code generators, build-helper crates, or internal modules under standard directory names such as `build/` (e.g. `crates/build/` or `src/build/`), `out/`, or in-tree dependencies in `vendor/`, all Rust source files within those paths are dropped from extraction. No document, symbol, relation, or diagnostic is produced. Downstream agent harnesses and reviewers querying the generated model or index will see an incomplete codebase with no indication that code was omitted.
- Reviewer confidence: high

## X-12

**`write` can overwrite files through symlinks**
- Evidence: `read_rec` reads only entries for which `file_type().is_file()` is true (`crates/sealmap-corpus/src/contract.rs:121–123`). But `write` joins the expected path and calls `fs::write` without checking for symlinks (`crates/sealmap-corpus/src/contract.rs:141–151`).
- Failure: An expected document path in the output directory is a symlink to another writable file. The read pass treats the document as missing; generation follows the symlink and overwrites its target.
- Reviewer confidence: high
