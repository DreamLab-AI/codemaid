## X-01

**Silent omission of `Self::*` and match guard calls from call flows**
- Evidence: The model claims to construct honest, exact call flows representing the minimal "who talks to whom, in what order" (`crates/sealmap-model/src/flow.rs:4`-`8`), and that no calls are dropped or guessed silently (`README.md:304`-`306`, `crates/sealmap-rust/src/lib.rs:59`-`63`). In contradiction, `crates/sealmap-rust/src/resolve.rs:59` places `"Self"` into the `PRELUDE` list. At `crates/sealmap-rust/src/resolve.rs:815`, any path call whose first segment is an unshadowed prelude identifier is unconditionally dropped. As exposed in `DEN-01.6`, calls to `Self::...` (e.g. constructors like `SymbolId::global` or internal helper dispatches) show no calls at all. Furthermore, calls in match guards are inserted into arm labels and never walked (`crates/sealmap-rust/src/collect.rs:1215`-`1216`).
- Failure: Any production code using idiomatic Rust constructor patterns (`Self::new()`), recursive calls, or associated function helpers via `Self::` has those call edges silently purged from the model, sequence diagrams, and dense projections. Human reviewers and autonomous agents relying on generated flows or review packs will inspect an empty sequence, conclude no downstream callee is executed, and approve changes where critical execution paths and security/transaction boundaries are entirely invisible.
- Reviewer confidence: high

## X-02

**Moving or reordering items within a file breaks module seals despite stability claims**
- Evidence: `crates/sealmap/src/lib.rs` and `sealmap-extract/src/fingerprint.rs` claim:
  > "Every symbol carries `sig_hash` for its contract and `body_hash` for its implementation, both blind to formatting, comments and moves."
  > "Positions are never hashed either, so moving an item within a file or to another file changes neither of its fingerprints."
  
  However, in `crates/sealmap-rust/src/collect.rs` lines 88–108, while declarations (`use`, `mod x;`) are sorted before hashing, non-declaration module items are folded directly in source order:
  ```rust
  for item in items {
      match item {
          Item::Use(_) | Item::ExternCrate(_) | Item::Mod(syn::ItemMod { content: None, .. }) => { ... }
          _ => self.item(&path, item, &mut body),
      }
  }
  ```
  And `crates/sealmap-rust/tests/seal.rs` lines 198–200 explicitly shows:
  ```rust
  // Known limit, pinned: a module's body folds its members in source order,
  // so reordering items inside it changes the module's body hash.
  assert_eq!(class_of(&report, ACCOUNTS_MOD), [Class::Behaviour]);
  assert_eq!(failing(&report), BTreeSet::from([Class::Behaviour]));
  ```
- Failure: Any CI workflow guarding diagram veracity with `sealmap verify` will fail with `Class::Behaviour` if an engineer reorders two functions or types within a file whose module is cited by a topic (a common practice when organizing code). The lockfile rejects the clean change as a behavioural drift, forcing developers to repeatedly re-sign and re-review topics when no semantic or behavioural changes occurred, degrading trust in the verification gate.
- Reviewer confidence: high

## X-03

**Generation can follow an output symlink and overwrite a file outside the corpus**
- Evidence: `read_rec` accepts only regular files, so it ignores a symlink at an expected document path (`crates/sealmap-corpus/src/contract.rs:109`). `write` then treats that document as missing and calls `fs::write` on the joined path (`crates/sealmap-corpus/src/contract.rs:139`), which follows the symlink.
- Failure: A checkout or output directory contains a symlink at a generated document path pointing to another file writable by the process. Running generation overwrites that target.
- Reviewer confidence: high; the attack scenario is inferred from the shown filesystem calls.

## X-04

Merged from near-identical reports; each version follows.

**Canonical lock edits can bless unreviewed code**
- Evidence: The material says a hand-edited lock is refused, but describes `parse_canonical` as parsing and reserialising TOML, then comparing the bytes (`crates/sealmap-corpus/src/seal/lock.rs:227-235`). The lock stores fingerprints and opaque reviewer/model strings (`crates/sealmap-corpus/src/seal/lock.rs:57`, `:79-82`); no mechanism shown authenticates who set them.
- Failure: Someone changes code and updates the lock’s fingerprints while keeping valid canonical TOML. The check can then accept the new code as sealed without a new review. This assumes the lock’s integrity is meant to be enforced by sealmap itself.
- Reviewer confidence: medium; inferred from the described parser and lock format.

**The seal gate does not authenticate who approved a seal**
- Evidence: A seal records reviewer and model as strings (`crates/sealmap-corpus/src/seal/lock.rs:68-85`), and `sign` accepts those values from its caller (`crates/sealmap-corpus/src/seal/sign.rs:12-20`). The check classifies seals by comparing symbol fingerprints (`crates/sealmap-corpus/src/seal/check.rs:121-147`); the shown lock and check have no authenticated signature or trusted reviewer key.
- Failure: If CI treats `verify` as proof of reviewer approval, a contributor who can change the code and lock can reseal the changed code while supplying a trusted reviewer’s name. The gate can pass without establishing that reviewer’s approval.
- Reviewer confidence: medium; the missing authentication is visible, while the production impact depends on how the calling skill and CI handle reviewer identity.

## X-05

**Source files can disappear from the model without warning**
- Evidence: The corpus contract says generation produces one document per source file (`crates/sealmap-corpus/src/lib.rs:11-57`), while the loader drops files over 2 MiB or with invalid UTF-8 (`crates/sealmap-model/src/source.rs:175-183`). The material says these files are skipped silently.
- Failure: A production repository contains a large source file with logic relevant to a review. Generation omits it without a diagnostic, so the generated view can appear complete while that logic is absent.
- Reviewer confidence: medium; the omission is explicit, and the impact on a particular repository is inferred.

## X-06

**One deeply nested file can terminate the whole extraction**
- Evidence: The material says a panic during per-file collection becomes a placeholder, but also says a stack overflow exhausts the large stack and ends the process. Collection uses 64 MiB stacks and catches panics (`crates/sealmap-extract/src/isolate.rs:38-56`); the flow walker has an expression-depth cap (`crates/sealmap-rust/src/collect.rs:30`).
- Failure: A file exceeds the stack limit during parsing or another uncapped operation. Instead of producing that file’s placeholder and continuing, extraction terminates, preventing the run from producing its model or corpus.
- Reviewer confidence: high.

## X-07

**Ambient checkout directory names leak into symbol IDs, invalidating seals across environments**
- Evidence: The core model promises strict byte-identical determinism on any machine governed by a contract explicitly forbidding ambient paths, usernames, or environment data (`crates/sealmap-model/src/lib.rs:18`-`34`). In contradiction, when `--name` is omitted, the CLI derives the codebase name from the ambient checkout directory path (`crates/sealmap/src/main.rs:351`, `377`-`382`). Under `crates/sealmap-rust/src/layout.rs:75`, fallback crates take this codebase name, and `crates/sealmap-extract/src/ids.rs:8` embeds it directly into package roots and symbol IDs (`sym:cargo <name> .`).
- Failure: When code is checked out into differently named directories across environments (e.g. `/home/dev/project` on a workstation versus `/runner/_work/1/repo` in CI or an isolated container in an agent harness), `SymbolId`s for fallback crates diverge. Seals signed locally will report all sealed symbols as `Absent` in CI, causing `sealmap verify` to fail. Similarly, `sealmap generate --check` will report false-positive drift across environments.
- Reviewer confidence: high

## X-08

**Two repositories with the same crate name can collapse into one model**
- Evidence: `load_repos` prefixes source paths with repository names (`crates/sealmap/src/lib.rs:113-123`), but layout derives the crate name from the package or library name (`crates/sealmap-rust/src/layout.rs:91-101`), and `module_sym` builds IDs from that crate name (`crates/sealmap-rust/src/resolve.rs:141-147`). When IDs collide, `Codebase::add_symbol` merges them, keeping the first definition’s scalar fields while combining fingerprints and members (`crates/sealmap-model/src/codebase.rs:71-85`).
- Failure: Combining two repositories that define the same crate name can merge their symbols. Generated diagrams, call resolution and seals may then describe a combined or wrong definition instead of either repository’s code.
- Reviewer confidence: high for repositories with colliding crate names; the production consequence is inferred.

## X-09

**Non-path trait implementations are silently discarded during resolution**
- Evidence: In `crates/sealmap-rust/src/resolve.rs` lines 258–265 and `collect.rs` lines 867–875:
  ```rust
  for imp in &f.impls {
      let module = module_sym(&imp.module);
      let Some(self_segs) = &imp.self_ty else { continue };
      let (ty, _) = r.resolve(&module, self_segs, None, Ns::Type);
  ```
  `first_path` in `collect.rs` resolves `self_ty` by inspecting `syn::Type`:
  ```rust
  fn first_path(ty: &Type) -> Option<Segs> {
      match ty {
          Type::Path(p) => Some(path_segs(&p.path)),
          Type::Reference(r) => first_path(&r.elem),
          Type::Paren(p) => first_path(&p.elem),
          Type::Group(g) => first_path(&g.elem),
          _ => None,
      }
  }
  ```
  When an impl block defines a trait on a slice (e.g., `impl Trait for [u8]` or `&[u8]`), tuple (e.g., `impl Trait for (A, B)`), array, bare function pointer (`fn()`), or dynamic trait object (`dyn Trait`), `first_path` returns `None`. Consequently, `self_segs` is `None`, and `resolve.rs` hits `continue`.
- Failure: Entire `impl` blocks for non-path types are skipped without warning. None of their methods are registered in `cb.symbols`, none of their calls or flows enter `cb.relations`, and their implementation relations (`RelationKind::Implements`) are never created. If an agent or engineer relies on sealmap to trace execution flow through standard patterns like tuple-based handlers (common in Axum/Actix), byte-slice parsers, or trait objects, sealmap renders those calls as dead ends or unresolved external calls.
- Reviewer confidence: high

## X-10

**Silent dropping of function flows and metadata for multiple definitions sharing an ID**
- Evidence: In `crates/sealmap-model/src/codebase.rs` lines 61–78:
  ```rust
  pub fn add_symbol(&mut self, symbol: Symbol) {
      match self.symbols.get_mut(&symbol.id) {
          Some(existing) => {
              existing.sig_hash = existing.sig_hash.merge(symbol.sig_hash);
              existing.body_hash = existing.body_hash.merge(symbol.body_hash);
              existing.members.extend(symbol.members);
              for tag in symbol.tags {
                  if !existing.tags.contains(&tag) { existing.tags.push(tag); }
              }
          }
          None => { self.symbols.insert(symbol.id.clone(), symbol); }
      }
  }
  ```
  `add_symbol` merges the `sig_hash` and `body_hash` across duplicate IDs (such as multi-target or `#[cfg]` twins: Windows vs. Unix, feature-gated implementations), but it completely ignores `symbol.flow`, `symbol.signature`, and `symbol.visibility`.
- Failure: Any codebase that uses target-specific or feature-gated functions under the same path/name will have only the first parsed variant's control-flow tree retained in the model. Calls and critical side effects in alternate implementations (e.g., Windows OS syscalls, fallback network logic, feature-flagged security checks) are silently dropped from the generated sequence diagrams, the call graph, and `_index.json`. Downstream LLM agents relying on `.sealmap/` or `dense.txt` will see an incomplete and misleading call graph, leading them to assume these functions execute no calls or only the calls of the first platform parsed.
- Reviewer confidence: high

## X-11

**Over-eager citation parsing breaks the seal verification gate on prose examples**
- Evidence: `sealmap verify` is documented as the production CI gate that ensures reviewed topics remain true to code (`crates/sealmap-corpus/src/seal/check.rs:359`, `docs/DESIGN.md:50`-`121`). However, the citation extractor treats any CommonMark inline code span outside fences starting like a global ID (`sym:`, a manager, a space) as a live citation (`crates/sealmap-corpus/src/seal/topic.rs:331`). At `crates/sealmap-corpus/src/seal/check.rs:390`, any unsealed citation in a topic is flagged as `UnsealedCitation`, and a report passes only when every finding is `Holds` (`crates/sealmap-corpus/src/seal/check.rs:104`-`105`). In `DEL-02.6`, the authors admit that running `sealmap verify` on this repository fails with exit code 1 because topic `MER-02` references an illustrative fixture ID in an inline code span. Consequently, no CI workflow runs `sealmap verify` (`.github/workflows/ci.yml:9`-`80`).
- Failure: Any engineering team that includes illustrative symbol IDs or grammar examples in Markdown prose will trigger an `UnsealedCitation` failure that cannot pass the seal check. Adopters cannot turn on the primary verification gate (`sealmap verify`) in CI pipelines without either stripping symbol documentation or seeing their CI gate permanently fail with exit code 1.
- Reviewer confidence: high
