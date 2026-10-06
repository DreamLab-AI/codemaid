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

/// ER `prefix X-09`: an impl whose self type is not a path (a tuple, slice,
/// array, reference to one, function pointer or trait object) was skipped,
/// with every method and call in it. Such methods are anchored like methods
/// of any type the codebase does not define: under `impl#[<type as written>]`
/// in the impl's module.
#[test]
fn impls_on_non_path_types_keep_their_methods() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        fn helper() {}
        pub trait T { fn go(&self); }
        impl T for (u8, u8) { fn go(&self) { helper(); } }
        impl T for [u8] { fn go(&self) { helper(); } }
        impl T for &[u8] { fn go(&self) { helper(); } }
        impl T for [u8; 4] { fn go(&self) { helper(); } }
        impl T for fn(u8) { fn go(&self) { helper(); } }
        impl dyn T { pub fn twice(&self) { self.go(); helper(); } }
        "#,
    )]);
    let helper = exact("sym:cargo ledger . helper().");
    let methods: Vec<String> = cb.symbols.keys().map(|k| k.to_string()).filter(|k| k.contains(" impl#")).collect();
    assert_eq!(
        methods,
        [
            "sym:cargo ledger . impl#[`&[u8]`][T]go().",
            "sym:cargo ledger . impl#[`(u8, u8)`][T]go().",
            "sym:cargo ledger . impl#[`[u8; 4]`][T]go().",
            "sym:cargo ledger . impl#[`[u8]`][T]go().",
            "sym:cargo ledger . impl#[`dyn T`]twice().",
            "sym:cargo ledger . impl#[`fn(u8)`][T]go().",
        ]
    );
    for m in &methods {
        assert!(calls(&cb, m).contains(&helper), "{m} lost its call");
        let s = cb.symbol(&SymbolId::parse(m).unwrap()).unwrap();
        assert!(s.parent.is_some(), "{m} has no parent");
    }
    // Each self type keeps its own id: `[u8]` and `&[u8]` are different impls.
}

/// ER `prefix X-08`: crate ids carry the crate name only, so two packages
/// with one name became one crate and their symbols merged. A crate name
/// claimed by more than one directory is now qualified with the directory,
/// and a name reference picks the package the caller's manifest depends on.
#[test]
fn same_named_crates_stay_apart() {
    let cb = model(&[
        ("a/Cargo.toml", "[package]\nname = \"core1\""),
        ("a/src/lib.rs", "pub fn run() { alpha() }\nfn alpha() {}"),
        ("b/Cargo.toml", "[package]\nname = \"core1\""),
        ("b/src/lib.rs", "pub fn run() { beta() }\nfn beta() {}"),
        ("b/src/main.rs", "fn main() { core1::run() }"),
        ("app/Cargo.toml", "[package]\nname = \"app\"\n[dependencies]\ncore1 = { path = \"../a\" }"),
        ("app/src/lib.rs", "pub fn go() { core1::run() }"),
        ("ws/Cargo.toml", "[workspace]\nmembers = [\"svc\"]\n[workspace.dependencies]\ncore1 = { path = \"../b\" }"),
        ("ws/svc/Cargo.toml", "[package]\nname = \"svc\"\n[dependencies]\ncore1.workspace = true"),
        ("ws/svc/src/lib.rs", "pub fn go() { core1::run() }"),
        ("any/Cargo.toml", "[package]\nname = \"any\"\n[dependencies]\ncore1 = \"1\""),
        ("any/src/lib.rs", "pub fn go() { core1::run() }"),
    ]);
    assert_eq!(calls(&cb, "sym:cargo a/core1 . run()."), [exact("sym:cargo a/core1 . alpha().")]);
    assert_eq!(calls(&cb, "sym:cargo b/core1 . run()."), [exact("sym:cargo b/core1 . beta().")]);
    // A binary names its own package's library.
    assert_eq!(calls(&cb, "sym:cargo core1_main . main()."), [exact("sym:cargo b/core1 . run().")]);
    // A path dependency, directly or through the workspace table, picks one.
    assert_eq!(calls(&cb, "sym:cargo app . go()."), [exact("sym:cargo a/core1 . run().")]);
    assert_eq!(calls(&cb, "sym:cargo svc . go()."), [exact("sym:cargo b/core1 . run().")]);
    // A dependency that names either is ambiguous: no internal edge is drawn.
    assert!(calls(&cb, "sym:cargo any . go().").iter().all(|(t, _)| !t.contains("core1 . run")));
    // Unique names stay plain.
    assert!(cb.symbols.keys().any(|k| k.to_string() == "sym:cargo app ."));
}
