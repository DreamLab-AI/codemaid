# Oracle evidence (selection 08:11:24–08:15 UTC)

Command per case/side (run.sh): transplant `git show <fix>:<test file>` into the snapshot worktree, then `CARGO_HOME=<scratchpad>/cargo-home-priv CARGO_TARGET_DIR=er/target/<case>-<side> cargo test -p sealmap-rust --test <file> -- <filter>`; the worktree file is restored afterwards.

## gen-S
```
test single_letter_generic_parameters_stay_unresolved ... ok
test single_capital_type_names_are_concrete_types_not_generics ... FAILED
test declared_multi_letter_generics_shadow_concrete_types ... FAILED
thread 'single_capital_type_names_are_concrete_types_not_generics' (2425764) panicked at crates/sealmap-rust/tests/extract.rs:437:5:
  left: []
 right: [("sym:cargo app . A#bump().", Exact), ("sym:cargo app . geom/V2#normalise().", Exact)]
thread 'declared_multi_letter_generics_shadow_concrete_types' (2425763) panicked at crates/sealmap-rust/tests/extract.rs:478:9:
sync(). bound a generic parameter to a concrete type: ["sym:cargo app . Store#put().", "sym:cargo app . Store#open()."]
test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.05s
exit=101
```
## gen-P
```
test single_letter_generic_parameters_stay_unresolved ... ok
test single_capital_type_names_are_concrete_types_not_generics ... FAILED
test declared_multi_letter_generics_shadow_concrete_types ... FAILED
thread 'single_capital_type_names_are_concrete_types_not_generics' (2396424) panicked at crates/sealmap-rust/tests/extract.rs:437:5:
  left: []
 right: [("sym:cargo app . A#bump().", Exact), ("sym:cargo app . geom/V2#normalise().", Exact)]
thread 'declared_multi_letter_generics_shadow_concrete_types' (2396421) panicked at crates/sealmap-rust/tests/extract.rs:478:9:
sync(). bound a generic parameter to a concrete type: ["sym:cargo app . Store#put().", "sym:cargo app . Store#open()."]
test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.07s
exit=101
```
## gen-F
```
test single_capital_type_names_are_concrete_types_not_generics ... ok
test single_letter_generic_parameters_stay_unresolved ... ok
test declared_multi_letter_generics_shadow_concrete_types ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.02s
exit=0
```
## var-P
```
test variant_labels_leave_out_field_attributes ... FAILED
thread 'variant_labels_leave_out_field_attributes' (2394171) panicked at crates/sealmap-rust/tests/extract.rs:522:5:
  left: [("Found", Some("{ #[doc = \" Where.\"] #[serde(skip)] file: String, line:…")), ("Gone", Some("(#[doc = \"why\"] String)")), ("None", None)]
 right: [("Found", Some("{ file: String, line: u32, }")), ("Gone", Some("(String)")), ("None", None)]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.19s
exit=101
```
## var-F
```
test variant_labels_leave_out_field_attributes ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.01s
exit=0
```
## prefix-P
```
test a_path_prefix_never_resolves_to_a_function ... FAILED
thread 'a_path_prefix_never_resolves_to_a_function' (2396198) panicked at crates/sealmap-rust/tests/resolver_den01.rs:250:5:
  left: [("sym:extern ledger::cli::audit::record", Inferred), ("sym:extern ledger::cli::trail::flush", Inferred)]
 right: [("sym:cargo ledger . audit/record().", Exact), ("sym:cargo ledger . audit/trail/flush().", Exact)]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.04s
exit=101
```
## prefix-F
```
test a_path_prefix_never_resolves_to_a_function ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s
exit=0
```
## den-P
```
test self_path_without_a_self_type_is_dropped ... ok
test struct_literal_receiver_is_typed ... FAILED
test self_module_path_calls_resolve_to_the_current_module ... FAILED
test self_path_calls_resolve_to_the_impl_self_type ... FAILED
test match_guard_calls_are_walked ... FAILED
thread 'struct_literal_receiver_is_typed' (2394929) panicked at crates/sealmap-rust/tests/resolver_den01.rs:136:5:
  left: []
 right: [("sym:cargo ledger . Statement#render().", Exact)]
thread 'self_module_path_calls_resolve_to_the_current_module' (2394926) panicked at crates/sealmap-rust/tests/resolver_den01.rs:82:5:
  left: []
 right: [("sym:cargo ledger . audit/stamp().", Exact)]
thread 'self_path_calls_resolve_to_the_impl_self_type' (2394927) panicked at crates/sealmap-rust/tests/resolver_den01.rs:63:5:
  left: []
 right: [("sym:cargo ledger . Account#validated().", Exact)]
thread 'match_guard_calls_are_walked' (2394925) panicked at crates/sealmap-rust/tests/resolver_den01.rs:112:5:
  left: []
 right: [("sym:cargo ledger . Posting#is_reversal().", Exact), ("sym:cargo ledger . is_shared().", Exact)]
test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
exit=101
```
## den-F
```
test self_path_without_a_self_type_is_dropped ... ok
test match_guard_calls_are_walked ... ok
test self_module_path_calls_resolve_to_the_current_module ... ok
test struct_literal_receiver_is_typed ... ok
test self_path_calls_resolve_to_the_impl_self_type ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
exit=0
```
## reach-P
```
test binaries_and_target_dependencies_reach_their_libraries ... ok
test ambiguous_guesses_among_reachable_crates_emit_no_edge ... ok
test guessed_edges_ignore_crates_the_caller_cannot_reach ... FAILED
thread 'guessed_edges_ignore_crates_the_caller_cannot_reach' (2395356) panicked at crates/sealmap-rust/tests/resolver_den01.rs:180:5:
  left: [("sym:cargo ledger_core . Journal#first().", Exact)]
 right: [("sym:cargo ledger_core . Journal#first().", Exact), ("sym:cargo ledger_core . Entry#reconcile().", Inferred)]
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.16s
exit=101
```
## reach-F
```
test binaries_and_target_dependencies_reach_their_libraries ... ok
test ambiguous_guesses_among_reachable_crates_emit_no_edge ... ok
test guessed_edges_ignore_crates_the_caller_cannot_reach ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.04s
exit=0
```
