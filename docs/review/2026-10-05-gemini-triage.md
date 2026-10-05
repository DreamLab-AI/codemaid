# Gemini review triage: sealmap corpus at 0d7c752, 2026-10-05

Intake per `build-with-quality` "Review findings intake". Source: `review-sealmap/findings.json` (33 findings: 15 critical, 3 premortem root causes, 15 premortem findings). Lenses: **critical** (code-fidelity critique), **premortem** (failure-story).

| Finding | Lens | Claim | Verdict | Evidence | Test / commit |
|---|---|---|---|---|---|
| critical:F-01 | critical | Stack overflow aborts the process, bypassing catch_unwind | known | DB-07 (EXT-01.3), isolate.rs:8 | none needed (register entry) |
| critical:F-02 | critical | One unreadable dir entry aborts load; non-UTF-8 files dropped silently | known | DB-05 (MOD-03.2), source.rs:170,181 | none needed (register entry) |
| critical:F-03 | critical | cfg twins keep only the first file, span, signature, doc | known | DB-02, DB-03 (MOD-02.3), codebase.rs:66-70 | none needed (register entry) |
| critical:F-04 | critical | Function-local use leaks into module-wide imports | known | DB-13 (EXT-03.7), collect.rs:155-160 | none needed (register entry) |
| critical:F-05 | critical | Overview crate graph uses lossy Ident::new, so nodes can collide | known | DB-22 (MER-02.3), overview.rs:109,119 | none needed (register entry) |
| critical:F-06 | critical | Trait-dispatched calls never stitch to a sequence fragment | rejected | Overstated. Concrete-receiver calls resolve to Type#[Trait]m() and default-method calls to Trait#m(); both expand. Only dyn calls to a required method target a bodiless declaration (resolve.rs:785-795). Pinned by trait_calls_link_to_impl_and_default_bodies | `trait_calls_link_to_impl_and_default_bodies` |
| critical:F-07 | critical | Untyped/shadowing closure params fabricate internal call edges | known | DB-16, DB-17, DB-18 (EXT-05.5/5.6) | none needed (register entry) |
| critical:F-08 | critical | Class::raw_member bypasses escape_type | known | DB-21 (MER-01.4), class.rs:96-99 | none needed (register entry) |
| critical:F-09 | critical | let-bound closures drawn at definition, not call site | known | DB-11 (EXT-03.4), collect.rs:1226-1231 | none needed (register entry) |
| critical:F-10 | critical | Derived-method ids are absent from Codebase.symbols | known | DB-09 (EXT-02.5), resolve.rs:528-530 | none needed (register entry) |
| critical:F-11 | critical | Eight-hop path walk limit silently turns internal into external | known | DB-15 (EXT-05.2), resolve.rs:555 | none needed (register entry) |
| critical:F-12 | critical | Lowering drops empty arms the model doc says are kept | known | DR-02 (EXT-04.2), lower.rs:71 vs flow.rs:109-111 | none needed (register entry) |
| critical:F-13 | critical | Codebase name from checkout dir breaks determinism | known | T-03 (DEL-01.3), main.rs:177-182 | none needed (register entry) |
| critical:F-14 | critical | MSRV 1.85 broken downstream by unpinned ignore 0.4.30 | known | T-02, DB-30 (DEL-01.2), Cargo.toml:8,31 | none needed (register entry) |
| critical:F-15 | critical | Generator version in _index.json causes false drift | known | DB-23 (COR-01.1), DB-28 (COR-03.2) | none needed (register entry) |
| premortem:R-1 | premortem | verify as CI gate collides with planned seals semantics | known | T-05 (DEL-02.3), DB-31, DB-32; README.md:225 | none needed (register entry) |
| premortem:R-2 | premortem | Closure dropping and empty-arm collapsing omit auth calls | known | DB-11, DR-02, DB-12 (EXT-03.4, EXT-04.2, EXT-03.6) | none needed (register entry) |
| premortem:R-3 | premortem | Non-UTF-8 source files silently excluded from the model | known | DB-05 (MOD-03.2), source.rs:181 | none needed (register entry) |
| premortem:F-01 | premortem | write deletes any .md starting with `sealmap: ` | rejected | Overstated. is_generated needs front matter (`---\n`) whose first key is the marker (document.rs:152-155); write deletes only those (contract.rs:99; I-34, I-37). Pinned by write_leaves_authored_markdown_that_mentions_the_marker | `write_leaves_authored_markdown_that_mentions_the_marker` |
| premortem:F-02 | premortem | Syn stack overflow aborts, bypassing the panic guard | known | DB-07 (EXT-01.3) | none needed (register entry) |
| premortem:F-03 | premortem | Checkout dir name leaks into codebase name, poisoning caches | known | T-03 (DEL-01.3) | none needed (register entry) |
| premortem:F-04 | premortem | cfg twin merge discards secondary file and span | known | DB-03 (MOD-02.3) | none needed (register entry) |
| premortem:F-05 | premortem | Untyped closure params bind to unrelated internal methods | known | DB-17 (EXT-05.6) | none needed (register entry) |
| premortem:F-06 | premortem | Shadowing closure param keeps the outer local type | known | DB-18 (EXT-05.6) | none needed (register entry) |
| premortem:F-07 | premortem | Macro calls never emitted, erasing checks from diagrams | known | DB-12 (EXT-03.6) | none needed (register entry) |
| premortem:F-08 | premortem | Eight-level recursion limit turns internal paths external | known | DB-15 (EXT-05.2) | none needed (register entry) |
| premortem:F-09 | premortem | Function-scoped use pollutes module namespace | known | DB-13 (EXT-03.7) | none needed (register entry) |
| premortem:F-10 | premortem | Trait map truncates unranked with no omitted count | known | DB-27 (COR-02.5), overview.rs:236 | none needed (register entry) |
| premortem:F-11 | premortem | Release bump rewrites _index.json, drifting every checkout | known | DB-23 (COR-01.1) | none needed (register entry) |
| premortem:F-12 | premortem | Unpinned ignore pulls let-chains into Rust 1.85 | known | T-02 (DEL-01.2) | none needed (register entry) |
| premortem:F-13 | premortem | Crate overview node collision from lossy Ident::new | known | DB-22 (MER-02.3) | none needed (register entry) |
| premortem:F-14 | premortem | Class::raw_member injects unescaped member text | known | DB-21 (MER-01.4) | none needed (register entry) |
| premortem:F-15 | premortem | Macro body tokenisation churns fingerprints on formatting | known | DB-19 (EXT-06.4) | none needed (register entry) |

Totals: known 31, rejected 2, confirmed 0.

## Notes

- Every `known` row is a defect the authors had already registered; the reviewer restated the register as findings. Several carried `marked_by_authors: yes`, which the register confirms.
- Both rejections come from reading the diagrams without the code. The two regression tests live in `crates/sealmap-corpus/tests/contract.rs` and pass; they were written to fail if the claim were true.
- No reviewer finding was confirmed. The one code change (and the citation restamp of the six `extract/` topics to `fec7aff`) comes from the defect found while probing, recorded below.
- Probe caution (now fixed, see below): a single-letter type name (`A`) was treated as a generic parameter (`resolve.rs:is_generic_param`), so a first reproduction of F-06 with `struct A` dropped every call.

## Defects found during triage

| Defect | Verdict | Evidence | Test / commit |
|---|---|---|---|
| triage:G-1 resolver took any one-capital name (`A`, `V2`) for a generic parameter and no multi-letter one (`Store`) | confirmed, fixed | `is_generic_param` checked spelling, not declared scope; now `InScope` (resolve.rs:407, collect.rs:788). VisionClaw model byte-identical before and after | `single_capital_type_names_are_concrete_types_not_generics`, `declared_multi_letter_generics_shadow_concrete_types`, `single_letter_generic_parameters_stay_unresolved`; tests `1667cc4`, fix `fec7aff` |
