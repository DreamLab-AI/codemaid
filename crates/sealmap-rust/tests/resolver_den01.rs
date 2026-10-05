//! Resolver defects found by reading sealmap's own dense projection
//! (DEN-01.6): edges the model lost or that depended on unrelated code.

use sealmap_model::{Codebase, Confidence, SourceSet, SymbolId};
use sealmap_rust::{ExternalCalls, RustOptions, extract};

fn model_with(files: &[(&str, &str)], external_calls: ExternalCalls) -> Codebase {
    let mut src = SourceSet::new();
    for (p, t) in files {
        src.insert(p, t).unwrap();
    }
    let out = extract(&src, &RustOptions { name: "ledger".into(), external_calls, ..Default::default() });
    assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    out.codebase
}

fn model(files: &[(&str, &str)]) -> Codebase {
    model_with(files, ExternalCalls::NonStd)
}

fn calls(cb: &Codebase, sym: &str) -> Vec<(String, Confidence)> {
    let id = SymbolId::parse(sym).unwrap_or_else(|e| panic!("{sym}: {e}"));
    cb.symbol(&id)
        .unwrap_or_else(|| {
            panic!("no symbol {sym}; have {:?}", cb.symbols.keys().map(|k| k.to_string()).collect::<Vec<_>>())
        })
        .flow
        .as_ref()
        .map(|f| f.calls().map(|c| (c.target.to_string(), c.confidence)).collect())
        .unwrap_or_default()
}

fn exact(s: &str) -> (String, Confidence) {
    (s.to_owned(), Confidence::Exact)
}

/// DEN-01.6 (a): `Self::f(..)` was dropped because `Self` sat on the prelude
/// list, checked before resolution. `Self` names the enclosing impl's self
/// type (or the trait, in a provided method).
#[test]
fn self_path_calls_resolve_to_the_impl_self_type() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        pub struct Account { cents: i64 }
        impl Account {
            pub fn opening(cents: i64) -> Self { Self::validated(cents) }
            fn validated(cents: i64) -> Self { Self { cents } }
            pub fn reopened(&self) -> Self { Self::opening(self.cents) }
        }
        pub trait Ledger {
            fn post(&mut self, cents: i64);
            fn post_twice(&mut self, cents: i64) { Self::post(self, cents); Ledger::post(self, cents); }
        }
        impl Ledger for Account {
            fn post(&mut self, cents: i64) { self.cents += cents; }
        }
        impl Account {
            pub fn transfer(&mut self, cents: i64) { Self::post(self, -cents); }
        }
        "#,
    )]);
    assert_eq!(calls(&cb, "sym:cargo ledger . Account#opening()."), [exact("sym:cargo ledger . Account#validated().")]);
    assert_eq!(calls(&cb, "sym:cargo ledger . Account#reopened()."), [exact("sym:cargo ledger . Account#opening().")]);
    // In a provided trait method, `Self::post` is exactly `Ledger::post`.
    let twice = calls(&cb, "sym:cargo ledger . Ledger#post_twice().");
    assert_eq!(twice.len(), 2, "{twice:?}");
    assert_eq!(twice[0], twice[1]);
    assert_eq!(twice[0].0, "sym:cargo ledger . Ledger#post().");
    // `Self::post` on the impl's type finds the trait-impl method.
    assert_eq!(
        calls(&cb, "sym:cargo ledger . Account#transfer()."),
        [exact("sym:cargo ledger . Account#[Ledger]post().")]
    );
}

/// `self::f(..)` is a path into the current module, not the `self` value:
/// it was dropped by the same prelude check.
#[test]
fn self_module_path_calls_resolve_to_the_current_module() {
    let cb = model(&[("src/lib.rs", "pub mod audit { pub fn record() { self::stamp(); } fn stamp() {} }")]);
    assert_eq!(calls(&cb, "sym:cargo ledger . audit/record()."), [exact("sym:cargo ledger . audit/stamp().")]);
}

/// A `Self::` call outside any impl or trait has nothing to resolve to and
/// stays out of the flow, even when every external call is kept.
#[test]
fn self_path_without_a_self_type_is_dropped() {
    let cb = model_with(&[("src/lib.rs", "pub fn orphan() { Self::nowhere(); }")], ExternalCalls::All);
    assert!(calls(&cb, "sym:cargo ledger . orphan().").is_empty());
}

/// DEN-01.6 (b): calls in a match-arm guard were only formatted into the arm
/// label and never walked. A guard sees the arm's bindings.
#[test]
fn match_guard_calls_are_walked() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        pub struct Posting { cents: i64 }
        impl Posting { pub fn is_reversal(&self) -> bool { self.cents < 0 } }
        pub fn is_shared(name: &str) -> bool { name.starts_with("shared") }
        pub fn settle(name: &str, posting: Option<Posting>) -> u8 {
            match posting {
                Some(p) if p.is_reversal() => 1,
                Some(_) if is_shared(name) => 2,
                _ => 0,
            }
        }
        "#,
    )]);
    assert_eq!(
        calls(&cb, "sym:cargo ledger . settle()."),
        [exact("sym:cargo ledger . Posting#is_reversal()."), exact("sym:cargo ledger . is_shared().")]
    );
}

/// DEN-01.6 (c): a method called on a struct literal (`Parser { .. }.id()`)
/// was lost: the walker classed the receiver as unknown. A struct literal
/// names its own type.
#[test]
fn struct_literal_receiver_is_typed() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        pub struct Statement { lines: Vec<String> }
        impl Statement { pub fn render(&self) -> String { self.lines.join("\n") } }
        impl Statement {
            pub fn blank(&self) -> String { (Self { lines: Vec::new() }).render() }
        }
        pub fn print(lines: Vec<String>) -> String { Statement { lines }.render() }
        pub fn print_ref(lines: Vec<String>) -> String { (&Statement { lines }).render() }
        "#,
    )]);
    let render = exact("sym:cargo ledger . Statement#render().");
    assert_eq!(calls(&cb, "sym:cargo ledger . print()."), std::slice::from_ref(&render));
    assert_eq!(calls(&cb, "sym:cargo ledger . print_ref()."), std::slice::from_ref(&render));
    assert_eq!(calls(&cb, "sym:cargo ledger . Statement#blank()."), [render]);
}
