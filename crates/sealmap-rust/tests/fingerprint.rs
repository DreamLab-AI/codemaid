//! What `sig_hash` and `body_hash` respond to, and what they ignore.

use std::collections::BTreeMap;

use sealmap_model::{Codebase, Fingerprint, SourceSet, SymbolId};
use sealmap_rust::{RustOptions, extract};

fn model(files: &[(&str, &str)]) -> Codebase {
    let mut src = SourceSet::new();
    for (p, t) in files {
        src.insert(p, t).unwrap();
    }
    let out = extract(&src, &RustOptions { name: "app".into(), ..Default::default() });
    assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
    out.codebase
}

/// Every symbol's fingerprints, by id text.
fn hashes(cb: &Codebase) -> BTreeMap<String, (Fingerprint, Fingerprint)> {
    cb.symbols.values().map(|s| (s.id.to_string(), (s.sig_hash, s.body_hash))).collect()
}

/// Assert two fingerprint maps are equal, naming the symbols that differ.
fn assert_same(a: &BTreeMap<String, (Fingerprint, Fingerprint)>, b: &BTreeMap<String, (Fingerprint, Fingerprint)>) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    let differ: Vec<(&String, bool, bool)> =
        a.iter().filter(|(k, v)| b[*k] != **v).map(|(k, v)| (k, b[k].0 != v.0, b[k].1 != v.1)).collect();
    assert!(differ.is_empty(), "(id, sig differs, body differs): {differ:#?}");
}

fn of(cb: &Codebase, id: &str) -> (Fingerprint, Fingerprint) {
    let s = cb.symbol(&SymbolId::parse(id).unwrap()).unwrap_or_else(|| panic!("no {id}"));
    (s.sig_hash, s.body_hash)
}

const BASE: &str = r#"
use std::fmt;
pub mod inner { pub fn deep(x: u32) -> u32 { x * 2 } }
pub struct Db<T: Clone> { pub rows: Vec<T>, cap: usize }
pub enum Mode { Fast, Slow(u8) }
pub trait Store { fn put(&self, k: u8); fn touch(&self) { self.put(1); } }
impl<T: Clone> Db<T> {
    pub fn new(cap: usize) -> Self { Db { rows: Vec::new(), cap } }
    pub fn len(&self) -> usize { self.rows.len() + self.cap }
}
impl<T: Clone> Store for Db<T> { fn put(&self, k: u8) { let _ = k; } }
impl fmt::Display for Mode { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "m") } }
pub const LIMIT: u32 = 10;
pub static NAME: &str = "db";
pub type Rows = Vec<u8>;
macro_rules! twice { ($e:expr) => { $e + $e }; }
pub fn add(a: u32, b: u32) -> u32 { twice!(a) + b + LIMIT }
"#;

/// The same program reformatted, commented and documented throughout.
const REFORMATTED: &str = r#"
//! Crate docs.
use std::fmt;

/// Inner module.
pub mod inner {
    // a comment
    pub fn deep(x: u32) -> u32 {
        x * 2 // twice
    }
}

/// A database.
pub struct Db<T: Clone> {
    /// The rows.
    pub rows: Vec<T>,
    cap: usize, /* capacity */
}

pub enum Mode {
    /// Quick.
    Fast,
    Slow(u8),
}

pub trait Store {
    /// Store one key.
    fn put(&self, k: u8);
    fn touch(&self) {
        // default
        self.put(1);
    }
}

impl<T: Clone> Db<T> {
    /// Make one.
    pub fn new(cap: usize) -> Self {
        Db {
            rows: Vec::new(),
            cap,
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.rows.len()
            + self.cap
    }
}

impl<T: Clone> Store for Db<T> {
    fn put(&self, k: u8) {
        #[allow(unused)]
        let _ = k;
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m")
    }
}

/// The limit.
pub const LIMIT: u32 = 10;
pub static NAME: &str = "db";
pub type Rows = Vec<u8>;

macro_rules! twice {
    ($e:expr) => {
        $e + $e
    };
}

/// Adds.
pub fn add(a: u32, b: u32) -> u32 {
    twice!(a) + b + LIMIT
}
"#;

#[test]
fn reformatting_and_comments_change_nothing() {
    let a = hashes(&model(&[("src/lib.rs", BASE)]));
    let b = hashes(&model(&[("src/lib.rs", REFORMATTED)]));
    assert_eq!(a.len(), 16, "{:#?}", a.keys());
    assert_same(&a, &b);
    // Windows line endings, including inside a multi-line string literal.
    let crlf = BASE.replace('\n', "\r\n").replace(r#""db""#, "\"d\r\nb\"");
    let lf = BASE.replace(r#""db""#, "\"d\nb\"");
    assert_same(&hashes(&model(&[("src/lib.rs", &crlf)])), &hashes(&model(&[("src/lib.rs", &lf)])));
}

#[test]
fn no_symbol_is_left_unfingerprinted() {
    let cb = model(&[("src/lib.rs", BASE)]);
    for s in cb.symbols.values() {
        assert!(!s.sig_hash.is_unset() && !s.body_hash.is_unset(), "{}", s.id);
    }
}

#[test]
fn moving_items_changes_neither_hash() {
    let a = model(&[(
        "src/lib.rs",
        "pub struct Db;\npub fn f() -> u8 { 1 }\nimpl Db { pub fn get(&self) -> u8 { f() } }\n",
    )]);
    // Reordered within the file, and the impl block moved to another file.
    let b = model(&[
        ("src/lib.rs", "mod more;\n\n\npub fn f() -> u8 { 1 }\npub struct Db;\n"),
        ("src/more.rs", "use super::Db;\nuse super::f;\nimpl Db { pub fn get(&self) -> u8 { f() } }\n"),
    ]);
    for id in ["sym:cargo app . Db#", "sym:cargo app . f().", "sym:cargo app . Db#get()."] {
        assert_eq!(of(&a, id), of(&b, id), "{id}");
    }
    let file = |cb: &Codebase| cb.symbol(&SymbolId::parse("sym:cargo app . Db#get().").unwrap()).unwrap().file.clone();
    assert_ne!(file(&a), file(&b));
}

#[test]
fn a_body_edit_changes_only_the_body_hash() {
    let base = model(&[("src/lib.rs", BASE)]);
    for (from, to, id) in [
        ("x * 2", "x * 3", "sym:cargo app . inner/deep()."),
        ("self.rows.len() + self.cap", "self.cap", "sym:cargo app . Db#len()."),
        ("self.put(1)", "self.put(2)", "sym:cargo app . Store#touch()."),
        ("LIMIT: u32 = 10", "LIMIT: u32 = 11", "sym:cargo app . LIMIT."),
        ("$e + $e", "$e * 2", "sym:cargo app . twice!"),
    ] {
        let edited = model(&[("src/lib.rs", &BASE.replace(from, to))]);
        let (s0, b0) = of(&base, id);
        let (s1, b1) = of(&edited, id);
        assert_eq!(s0, s1, "{id}: signature moved on a body edit");
        assert_ne!(b0, b1, "{id}: body edit not seen");
    }
    // The enclosing module's body covers its members; its contract does not.
    let edited = model(&[("src/lib.rs", &BASE.replace("x * 2", "x * 3"))]);
    let (s0, b0) = of(&base, "sym:cargo app . inner/");
    let (s1, b1) = of(&edited, "sym:cargo app . inner/");
    assert_eq!(s0, s1);
    assert_ne!(b0, b1);
    assert_ne!(of(&base, "sym:cargo app .").1, of(&edited, "sym:cargo app .").1);
}

#[test]
fn a_signature_edit_changes_the_sig_hash() {
    let base = model(&[("src/lib.rs", BASE)]);
    for (from, to, id) in [
        ("pub fn add(a: u32, b: u32) -> u32", "pub fn add(a: u32, b: u64) -> u32", "sym:cargo app . add()."),
        ("pub fn add(a: u32, b: u32) -> u32", "pub fn add(a: u32, b: u32) -> u64", "sym:cargo app . add()."),
        ("pub fn add(a: u32, b: u32) -> u32", "pub(crate) fn add(a: u32, b: u32) -> u32", "sym:cargo app . add()."),
        (
            "pub fn add(a: u32, b: u32) -> u32",
            "#[must_use] pub fn add(a: u32, b: u32) -> u32",
            "sym:cargo app . add().",
        ),
        ("pub fn add(a: u32, b: u32) -> u32", "pub fn add(c: u32, b: u32) -> u32", "sym:cargo app . add()."),
        ("pub fn add(a: u32, b: u32) -> u32", "pub async fn add(a: u32, b: u32) -> u32", "sym:cargo app . add()."),
        ("impl<T: Clone> Db<T> {", "impl<T: Clone + Send> Db<T> {", "sym:cargo app . Db#len()."),
        (
            "pub struct Db<T: Clone> { pub rows: Vec<T>, cap: usize }",
            "pub struct Db<T: Clone> { pub rows: Vec<T>, cap: u64 }",
            "sym:cargo app . Db#",
        ),
        ("fn put(&self, k: u8); fn touch", "fn put(&self, k: u16); fn touch", "sym:cargo app . Store#"),
        ("pub const LIMIT: u32", "pub const LIMIT: u64", "sym:cargo app . LIMIT."),
    ] {
        let edited = model(&[("src/lib.rs", &BASE.replacen(from, to, 1))]);
        assert_ne!(of(&base, id).0, of(&edited, id).0, "{id}: `{to}` not seen as a contract change");
    }
}

#[test]
fn renaming_changes_the_id_and_keeps_the_body_hash() {
    let a = model(&[("src/lib.rs", "pub fn total(xs: &[u8]) -> u32 { xs.iter().map(|x| *x as u32).sum() }")]);
    let b = model(&[("src/lib.rs", "pub fn sum_all(xs: &[u8]) -> u32 { xs.iter().map(|x| *x as u32).sum() }")]);
    assert!(b.symbol(&SymbolId::parse("sym:cargo app . total().").unwrap()).is_none());
    let (sa, ba) = of(&a, "sym:cargo app . total().");
    let (sb, bb) = of(&b, "sym:cargo app . sum_all().");
    assert_eq!(ba, bb);
    assert_ne!(sa, sb);
    // Types too: the shape is the body.
    let a = model(&[("src/lib.rs", "pub struct Point { x: f32, y: f32 }")]);
    let b = model(&[("src/lib.rs", "pub struct Vec2 { x: f32, y: f32 }")]);
    assert_eq!(of(&a, "sym:cargo app . Point#").1, of(&b, "sym:cargo app . Vec2#").1);
}

/// Pinned values for a fixed input: any change to what the Rust language adapter
/// feeds, or to the shared encoding, fails here. Bump the algorithm id
/// (`sealmap_extract::fingerprint::ALGORITHM`) when that is intended.
#[test]
fn golden_values() {
    let cb = model(&[("src/lib.rs", "pub fn add(a: u8) -> u8 { a + 1 }\npub struct P { x: u8 }\n")]);
    let show = |id: &str| {
        let (s, b) = of(&cb, id);
        (s.to_string(), b.to_string())
    };
    assert_eq!(
        show("sym:cargo app . add()."),
        (
            "blake3-16:06e159eea9e92f696a2aff0b245c0a42".to_owned(),
            "blake3-16:c71884b2d4710c35ffb85bac8dbaae27".to_owned()
        )
    );
    assert_eq!(
        show("sym:cargo app . P#"),
        (
            "blake3-16:8b73eb696f330993d46d8b374148ff1f".to_owned(),
            "blake3-16:dfc66d810c0eee9bb8adac7d17ce048a".to_owned()
        )
    );
    assert_eq!(
        show("sym:cargo app ."),
        (
            "blake3-16:0a069927590b82d13b37d34115040801".to_owned(),
            "blake3-16:2a37c9c1af616b5d725672488b0806ea".to_owned()
        )
    );
}

#[test]
fn formatting_never_changes_an_id() {
    let a =
        model(&[("src/lib.rs", "pub struct S; pub trait H<M> { fn h(&self); }\nimpl H<u8> for S { fn h(&self) {} }")]);
    let b = model(&[(
        "src/lib.rs",
        "pub struct S;\npub trait H<M> {\n    fn h(&self);\n}\nimpl H<\n    u8,\n> for S {\n    fn h(&self) {}\n}\n",
    )]);
    assert_eq!(hashes(&a).keys().collect::<Vec<_>>(), hashes(&b).keys().collect::<Vec<_>>());
    assert!(a.symbol(&SymbolId::parse("sym:cargo app . S#[`H<u8>`]h().").unwrap()).is_some());
}
