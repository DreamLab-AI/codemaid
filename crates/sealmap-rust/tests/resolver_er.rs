//! Resolver and model defects confirmed by the ER review experiment
//! (`docs/evidence/ER/`): each test reproduces one adjudicated finding.

use sealmap_model::{Codebase, Confidence, RelationKind, SourceSet, Step, SymbolId};
use sealmap_rust::{RustOptions, extract};

fn model(files: &[(&str, &str)]) -> Codebase {
    let mut src = SourceSet::new();
    for (p, t) in files {
        src.insert(p, t).unwrap();
    }
    let out = extract(&src, &RustOptions { name: "ledger".into(), ..Default::default() });
    assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    out.codebase
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

/// ER `gen X-05`: the binding of an `if let` or `while let` condition stayed
/// in the walker's environment after the statement, so a later call on the
/// shadowed outer name took the condition's type (a false Exact edge).
#[test]
fn if_let_and_while_let_bindings_end_with_their_statement() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        pub struct Db;
        impl Db { pub fn ping(&self) {} pub fn query(&self) {} }
        pub struct Other;
        impl Other { pub fn ping(&self) {} pub fn query(&self) {} }

        pub fn after_if(d: Db, o: Option<Other>) {
            if let Some(d) = o { d.ping(); }
            d.query();
        }
        pub fn in_else(d: Db, o: Option<Other>) {
            if let Some(d) = o { d.ping(); } else { d.query(); }
        }
        pub fn in_else_if(d: Db, o: Option<Other>, p: Option<Other>) {
            if let Some(d) = o { d.ping(); } else if let Some(_e) = p { d.query(); }
        }
        pub fn after_while(d: Db, o: Option<Other>) {
            while let Some(d) = o { d.ping(); break; }
            d.query();
        }
        "#,
    )]);
    let (db_ping, db_query) = ("sym:cargo ledger . Db#ping().", "sym:cargo ledger . Db#query().");
    let other_ping = "sym:cargo ledger . Other#ping().";
    assert_eq!(calls(&cb, "sym:cargo ledger . after_if()."), [exact(other_ping), exact(db_query)]);
    assert_eq!(calls(&cb, "sym:cargo ledger . in_else()."), [exact(other_ping), exact(db_query)]);
    assert_eq!(calls(&cb, "sym:cargo ledger . in_else_if()."), [exact(other_ping), exact(db_query)]);
    assert_eq!(calls(&cb, "sym:cargo ledger . after_while()."), [exact(other_ping), exact(db_query)]);
    // Inside its block the binding holds: no `ping` is drawn as Db's.
    assert!(
        cb.symbols.values().all(|s| s.flow.as_ref().is_none_or(|f| f.calls().all(|c| c.target.to_string() != db_ping)))
    );
}

/// ER `gen X-09` / `prefix X-10`: `#[cfg]` twins share one id, and the merge
/// kept only the first definition's flow, so the second twin's calls were
/// lost from the flow and from the call relations. The twins are now arms of
/// one branch, each labelled with its definition's line.
#[test]
fn cfg_twins_keep_every_definitions_flow() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        fn a() {}
        fn b() {}
        fn c() {}
        #[cfg(unix)]
        pub fn plat() { a(); }
        #[cfg(windows)]
        pub fn plat() { b(); }
        #[cfg(unix)]
        pub fn quiet() {}
        #[cfg(not(unix))]
        pub fn quiet() { c(); }
        #[cfg(unix)]
        pub fn same() { a(); }
        #[cfg(windows)]
        pub fn same() { a(); }
        "#,
    )]);
    let (a, b, c) = ("sym:cargo ledger . a().", "sym:cargo ledger . b().", "sym:cargo ledger . c().");
    assert_eq!(calls(&cb, "sym:cargo ledger . plat()."), [exact(a), exact(b)]);
    // The first twin calls nothing; the second's call still reaches the model.
    assert_eq!(calls(&cb, "sym:cargo ledger . quiet()."), [exact(c)]);
    // Identical twins stay one flat flow.
    assert_eq!(calls(&cb, "sym:cargo ledger . same()."), [exact(a)]);
    let flow = |s: &str| cb.symbol(&SymbolId::parse(s).unwrap()).unwrap().flow.clone().unwrap();
    assert_eq!(flow("sym:cargo ledger . same().").steps.len(), 1);
    let Step::Branch { arms } = &flow("sym:cargo ledger . plat().").steps[0] else { panic!("not a branch") };
    let labels: Vec<_> = arms.iter().map(|a| a.label.as_str()).collect();
    // A definition's span opens on its `#[cfg]` attribute.
    assert_eq!(labels, ["cfg twin at src/lib.rs:5", "cfg twin at src/lib.rs:7"]);
    let Step::Branch { arms } = &flow("sym:cargo ledger . quiet().").steps[0] else { panic!("not a branch") };
    assert!(arms[0].steps.is_empty() && arms[1].steps.len() == 1, "{arms:?}");
    // Call relations come from flows, so the second twin's edges are there.
    let to: Vec<String> = cb
        .relations
        .iter()
        .filter(|r| r.kind == RelationKind::Calls && r.from.to_string().ends_with(" quiet()."))
        .map(|r| r.to.to_string())
        .collect();
    assert_eq!(to, [c]);
}
