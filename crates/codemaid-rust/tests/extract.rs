//! Resolution and flow-shape tests for the Rust frontend.

use codemaid_model::{CallKind, Codebase, Confidence, RelationKind, SourceSet, Step, SymbolId, SymbolKind};
use codemaid_rust::{ExternalCalls, RustOptions, extract};

fn model(files: &[(&str, &str)]) -> Codebase {
    let mut src = SourceSet::new();
    for (p, t) in files {
        src.insert(p, t).unwrap();
    }
    let out = extract(&src, &RustOptions { name: "app".into(), ..Default::default() });
    assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    out.codebase
}

fn calls(cb: &Codebase, id: &str) -> Vec<(String, Confidence)> {
    cb.symbol(&SymbolId::new(id))
        .unwrap_or_else(|| panic!("no symbol {id}; have {:?}", cb.symbols.keys().collect::<Vec<_>>()))
        .flow
        .as_ref()
        .map(|f| f.calls().map(|c| (c.target.to_string(), c.confidence)).collect())
        .unwrap_or_default()
}

fn targets(cb: &Codebase, id: &str) -> Vec<String> {
    calls(cb, id).into_iter().map(|(t, _)| t).collect()
}

#[test]
fn resolves_paths_through_imports_renames_globs_and_reexports() {
    let cb = model(&[
        ("src/lib.rs", "pub mod a; pub mod b; pub use a::inner::deep as reexported;"),
        ("src/a/mod.rs", "pub mod inner; pub fn top() {}"),
        ("src/a/inner.rs", "pub fn deep() {} pub fn other() {} pub fn sib() { super::top(); crate::b::util(); }"),
        (
            "src/b.rs",
            "use crate::a::inner::{self, deep as d};\nuse crate::a::*;\npub fn util() {}\npub fn go() { d(); inner::other(); top(); crate::reexported(); }",
        ),
    ]);
    assert_eq!(
        targets(&cb, "app::b::go"),
        ["app::a::inner::deep", "app::a::inner::other", "app::a::top", "app::a::inner::deep"]
    );
    assert_eq!(targets(&cb, "app::a::inner::sib"), ["app::a::top", "app::b::util"]);
    assert!(calls(&cb, "app::b::go").iter().all(|(_, c)| *c == Confidence::Exact));
}

#[test]
fn resolves_methods_from_receiver_types() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        use std::sync::Arc;
        pub struct Db;
        impl Db { pub fn query(&self) {} }
        pub trait Store { fn put(&self); fn touch(&self) { self.put(); } }
        impl Store for Db { fn put(&self) {} }
        pub struct Svc { db: Arc<Db>, list: Vec<Db> }
        impl Svc {
            pub fn new() -> Self { Svc { db: Arc::new(Db), list: Vec::new() } }
            pub fn run(&self, other: &Db, s: impl Store) {
                self.db.query();          // field through Arc
                self.list.len();          // Vec method: std, dropped
                other.put();              // trait method via impl
                s.touch();                // trait-bounded param
                let local = Svc::new();   // constructed type
                local.helper();
                self.helper();
            }
            fn helper(&self) {}
        }
        "#,
    )]);
    assert_eq!(
        targets(&cb, "app::Svc::run"),
        [
            "app::Db::query",
            "app::Db::<Store>::put",
            "app::Store::touch",
            "app::Svc::new",
            "app::Svc::helper",
            "app::Svc::helper"
        ]
    );
    // Trait default method calls the required method on Self (the trait).
    assert_eq!(targets(&cb, "app::Store::touch"), ["app::Store::put"]);
    assert!(
        cb.relations.iter().any(|r| r.kind == RelationKind::Implements
            && r.from.as_str() == "app::Db"
            && r.to.as_str() == "app::Store")
    );
}

#[test]
fn drops_std_prelude_and_local_closures_by_default() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        pub fn f(xs: Vec<u8>) -> String {
            let g = |x: u8| x + 1;
            let v = Vec::new();
            drop(v);
            g(1);
            std::fs::read_to_string("x").ok();
            serde_json::to_string(&xs).unwrap();
            String::from("x")
        }
        "#,
    )]);
    assert_eq!(targets(&cb, "app::f"), ["serde_json::to_string"]);
    assert_eq!(calls(&cb, "app::f")[0].1, Confidence::External);
}

#[test]
fn external_calls_policy_all_keeps_std() {
    let mut src = SourceSet::new();
    src.insert("src/lib.rs", "pub fn f() { std::fs::read(\"x\").unwrap(); }").unwrap();
    let cb = extract(&src, &RustOptions { external_calls: ExternalCalls::All, ..Default::default() }).codebase;
    let ts: Vec<_> = cb
        .symbols
        .values()
        .filter_map(|s| s.flow.as_ref())
        .flat_map(|f| f.calls())
        .map(|c| c.target.to_string())
        .collect();
    assert!(ts.contains(&"std::fs::read".to_string()), "{ts:?}");
}

#[test]
fn flow_shapes_follow_control_flow() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        fn a() {} fn b() {} fn c() {} fn d() -> bool { true } fn e() {}
        async fn net() -> Result<(), ()> { Ok(()) }
        pub async fn flow(items: Vec<u8>, x: Option<u8>) -> Result<(), ()> {
            if d() { a(); } else if x.is_some() { b(); } else { return Err(()); }
            for _i in items { c(); }
            match x { Some(_) => e(), None => {} }
            net().await?;
            items2().iter().for_each(|_| a());
            tokio::spawn(async move { b(); });
            Ok(())
        }
        fn items2() -> Vec<u8> { vec![] }
        "#,
    )]);
    let f = cb.symbol(&SymbolId::new("app::flow")).unwrap().flow.clone().unwrap();
    let shape: Vec<&str> = f
        .steps
        .iter()
        .map(|s| match s {
            Step::Call(_) => "call",
            Step::Branch { .. } => "branch",
            Step::Loop { .. } => "loop",
            Step::Optional { .. } => "opt",
            Step::Parallel { .. } => "par",
            Step::Return(_) => "return",
        })
        .collect();
    assert_eq!(shape, ["call", "branch", "loop", "opt", "call", "call", "loop", "call", "par"]);
    let net = f.calls().find(|c| c.target.as_str() == "app::net").unwrap();
    assert!(net.awaited && net.fallible);
    assert_eq!(net.kind, CallKind::Function);
    // The else-branch early return survives as an exit inside the branch.
    if let Step::Branch { arms } = &f.steps[1] {
        assert_eq!(arms.len(), 3);
        assert_eq!(arms[1].label, "if x.is_some()");
        assert!(matches!(arms[2].steps[0], Step::Return(_)));
    } else {
        panic!("expected branch");
    }
    // `match` with a single arm that calls becomes an optional with the pattern.
    assert!(matches!(&f.steps[3], Step::Optional { label, .. } if label == "Some(_)"));
}

#[test]
fn workspace_crates_resolve_across_packages() {
    let cb = model(&[
        ("Cargo.toml", "[workspace]\nmembers = ['core', 'api']"),
        ("core/Cargo.toml", "[package]\nname = 'my-core'"),
        ("core/src/lib.rs", "pub struct Engine; impl Engine { pub fn start(&self) {} }"),
        ("api/Cargo.toml", "[package]\nname = 'api'"),
        ("api/src/lib.rs", "use my_core::Engine;\npub fn serve(e: &Engine) { e.start(); }"),
    ]);
    assert_eq!(calls(&cb, "api::serve"), [("my_core::Engine::start".to_string(), Confidence::Exact)]);
    assert!(cb.symbol(&SymbolId::new("my_core::Engine")).is_some_and(|s| s.kind == SymbolKind::Struct));
}

#[test]
fn tests_are_excluded_unless_requested() {
    let src_files = [("src/lib.rs", "pub fn real() {}\n#[cfg(test)] mod tests { #[test] fn t() { super::real(); } }")];
    let cb = model(&src_files);
    assert!(cb.symbol(&SymbolId::new("app::tests")).is_none());

    let mut src = SourceSet::new();
    src.insert(src_files[0].0, src_files[0].1).unwrap();
    let cb = extract(&src, &RustOptions { name: "app".into(), include_tests: true, ..Default::default() }).codebase;
    assert_eq!(targets(&cb, "app::tests::t"), ["app::real"]);
}

#[test]
fn unparsable_files_are_reported_but_still_present() {
    let mut src = SourceSet::new();
    src.insert("src/lib.rs", "pub mod broken;").unwrap();
    src.insert("src/broken.rs", "fn oops( {").unwrap();
    let out = extract(&src, &RustOptions { name: "app".into(), ..Default::default() });
    assert_eq!(out.diagnostics.len(), 1);
    assert_eq!(out.diagnostics[0].file.as_str(), "src/broken.rs");
    assert!(out.codebase.symbol(&SymbolId::new("app::broken")).is_some());
    assert_eq!(out.codebase.files.len(), 2);
}

#[test]
fn extraction_is_deterministic_regardless_of_insertion_order() {
    let files = [
        ("src/lib.rs", "pub mod x; pub mod y; pub fn root() { x::a(); y::b(); }"),
        ("src/x.rs", "pub fn a() { crate::y::b(); }"),
        ("src/y.rs", "pub fn b() {}"),
    ];
    let mut fwd = SourceSet::new();
    let mut rev = SourceSet::new();
    for (p, t) in files {
        fwd.insert(p, t).unwrap();
    }
    for (p, t) in files.iter().rev() {
        rev.insert(p, t).unwrap();
    }
    let opts = RustOptions { name: "app".into(), ..Default::default() };
    assert_eq!(extract(&fwd, &opts).codebase, extract(&rev, &opts).codebase);
}

/// Generated code nests far deeper than hand-written code. Each shape below
/// overflowed a 2 MiB rayon worker (debug build) before collection moved to
/// big-stack threads and the flow walker got a depth guard.
#[test]
fn deep_nesting_does_not_overflow() {
    let mut ty = String::from("B0");
    for _ in 0..200 {
        ty = format!("UInt<{ty}, B1>");
    }
    let shapes = [
        format!("pub type U = {ty};\n"),
        format!("pub fn f() {{ let x = {}1; }}\n", "g() + ".repeat(5000)),
        format!("pub fn f() {{ x{}; }}\n", ".a()".repeat(5000)),
        format!("pub fn f() {{ {}g(){} }}\n", "if c { ".repeat(300), " }".repeat(300)),
        format!("pub fn f() {{ {}x{}; }}\n", "g(".repeat(300), ")".repeat(300)),
    ];
    // Two files, so rayon actually hands work to its workers.
    for text in shapes {
        let mut src = SourceSet::new();
        src.insert("src/lib.rs", &text).unwrap();
        src.insert("src/other.rs", "pub fn h() {}").unwrap();
        let out = extract(&src, &RustOptions { name: "deep".into(), ..Default::default() });
        assert!(out.codebase.symbol(&SymbolId::new("deep")).is_some());
    }
}

/// `use a::*` cycles and many globs per module used to make path resolution
/// exponential (sysctl 0.6: 908 M `walk` calls for one file, never finished).
#[test]
fn glob_cycles_resolve_quickly() {
    let mut src = SourceSet::new();
    let mut lib = String::new();
    for i in 0..12 {
        lib.push_str(&format!("mod m{i};\npub use m{i}::*;\n"));
    }
    src.insert("src/lib.rs", &lib).unwrap();
    for i in 0..12 {
        let mut m = String::new();
        // Edition-2015 style crate-relative globs that do not resolve from
        // the module, plus a real cycle back to the crate root.
        for j in 0..12 {
            m.push_str(&format!("use m{j}::*;\n"));
        }
        m.push_str("use crate::*;\n");
        m.push_str(&format!("pub struct T{i};\npub fn f{i}() {{ missing(); T{i}::new(); }}\n"));
        src.insert(format!("src/m{i}.rs").as_str(), &m).unwrap();
    }
    let t = std::time::Instant::now();
    let out = extract(&src, &RustOptions { name: "g".into(), ..Default::default() });
    assert!(t.elapsed() < std::time::Duration::from_secs(5), "took {:?}", t.elapsed());
    assert!(out.codebase.symbol(&SymbolId::new("g::m3::T3")).is_some());
}

/// Known resolver fault, fixed in the id work (DESIGN §7, step 2): a method
/// call on a receiver of unknown type (`dir`, bound from an `Option<&Path>`)
/// is matched by name to the only in-crate method called `parent`, so
/// `Path::parent()` is drawn, as inferred, on the `Id` lane (seen in
/// `contract::remove_empty_parents`). The right answer is to drop the call (std) or
/// keep it external, never to bind it to `Id::parent`.
#[test]
#[ignore = "resolver fix belongs to step 2 (sym: ids)"]
fn std_receiver_method_is_not_bound_to_same_named_internal_method() {
    let cb = model(&[(
        "src/lib.rs",
        r#"
        use std::path::Path;
        pub struct Id;
        impl Id { pub fn parent(&self) -> Option<Id> { None } }
        pub fn walk(file: &Path) {
            let mut cur = file.parent();
            while let Some(dir) = cur { cur = dir.parent(); }
        }
        "#,
    )]);
    let t = targets(&cb, "app::walk");
    assert!(!t.iter().any(|t| t == "app::Id::parent"), "Path::parent bound to Id::parent: {t:?}");
}
