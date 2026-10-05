//! The dense projection end to end, on Rust sources run through the real
//! language adapter, plus hand-built models for the edge cases.
//!
//! Golden files live in `tests/golden/`. After an intended format change,
//! regenerate them with `SEALMAP_DENSE_BLESS=1 cargo test -p sealmap-dense`
//! and review the diff.

use std::collections::BTreeSet;
use std::path::PathBuf;

use sealmap_dense::{Dense, DenseOptions, INDEX_FILE, SliceError, SliceOptions, TEXT_FILE, render};
use sealmap_model::{
    Call, Codebase, Confidence, Descriptor, Flow, Package, SourcePath, SourceSet, Span, Step, Symbol, SymbolId,
    SymbolKind,
};
use sealmap_rust::{RustOptions, extract};

/// A small but realistic crate: multi-letter type names, a trait with a
/// default method, two `From` impls on one type, a type and a function name
/// each defined in two modules, async calls, `?`, an external call, mutual
/// and self recursion, and a call chain deeper than the default depth.
const WAREHOUSE: &[(&str, &str)] = &[
    ("Cargo.toml", "[package]\nname = \"warehouse\"\n"),
    ("src/lib.rs", "//! Warehouse.\npub mod inventory;\npub mod ledger;\npub mod orders;\n"),
    (
        "src/inventory.rs",
        r#"use std::collections::BTreeMap;

pub trait StockStore {
    fn reserve(&mut self, sku: &str, qty: u32) -> Result<(), StockError>;
    fn available(&self, sku: &str) -> u32 {
        self.lookup(sku)
    }
    fn lookup(&self, sku: &str) -> u32;
}

#[derive(Debug)]
pub enum StockError {
    Missing(String),
    Short { sku: String, wanted: u32 },
}

pub struct InventoryLedger {
    levels: BTreeMap<String, u32>,
    journal: Vec<String>,
}

impl InventoryLedger {
    pub fn new() -> Self {
        Self { levels: BTreeMap::new(), journal: Vec::new() }
    }

    pub fn record(&mut self, line: String) {
        self.journal.push(line);
    }
}

impl StockStore for InventoryLedger {
    fn reserve(&mut self, sku: &str, qty: u32) -> Result<(), StockError> {
        let have = self.available(sku);
        if have < qty {
            return Err(StockError::Short { sku: sku.into(), wanted: qty });
        }
        self.record(format!("{sku}-{qty}"));
        Ok(())
    }

    fn lookup(&self, sku: &str) -> u32 {
        self.levels.get(sku).copied().unwrap_or(0)
    }
}

pub fn validate(sku: &str) -> bool {
    !sku.is_empty()
}
"#,
    ),
    (
        "src/orders.rs",
        r#"use crate::inventory::{InventoryLedger, StockError, StockStore};

pub struct PurchaseOrder {
    pub lines: Vec<OrderLine>,
}

pub struct OrderLine {
    pub sku: String,
    pub qty: u32,
}

impl From<OrderLine> for PurchaseOrder {
    fn from(line: OrderLine) -> Self {
        Self { lines: vec![line] }
    }
}

impl From<Vec<OrderLine>> for PurchaseOrder {
    fn from(lines: Vec<OrderLine>) -> Self {
        Self { lines }
    }
}

pub async fn place(ledger: &mut InventoryLedger, order: PurchaseOrder) -> Result<usize, StockError> {
    if !validate(&order) {
        return Ok(0);
    }
    for line in &order.lines {
        ledger.reserve(&line.sku, line.qty)?;
    }
    notify(order.lines.len()).await;
    Ok(order.lines.len())
}

async fn notify(count: usize) {
    let _ = serde_json::to_string(&count);
}

pub fn validate(order: &PurchaseOrder) -> bool {
    order.lines.iter().all(|line| crate::inventory::validate(&line.sku))
}
"#,
    ),
    (
        "src/ledger.rs",
        r#"pub struct InventoryLedger;

impl InventoryLedger {
    pub fn new() -> Self {
        InventoryLedger
    }
}

pub fn settle(depth: u32) -> u32 {
    if depth == 0 {
        return 0;
    }
    reconcile(depth - 1)
}

pub fn reconcile(depth: u32) -> u32 {
    settle(depth) + audit_trail(depth)
}

fn audit_trail(n: u32) -> u32 {
    if n > 10 { audit_trail(n - 1) } else { n }
}

pub fn chain_alpha() {
    chain_beta();
}

fn chain_beta() {
    chain_gamma();
}

fn chain_gamma() {
    chain_delta();
}

fn chain_delta() {
    chain_epsilon();
}

fn chain_epsilon() {
    settle(1);
}
"#,
    ),
];

fn warehouse() -> Codebase {
    let mut src = SourceSet::new();
    for (path, text) in WAREHOUSE {
        src.insert(path, text).unwrap();
    }
    extract(&src, &RustOptions { name: "warehouse".into(), ..Default::default() }).codebase
}

/// Compare with `tests/golden/<name>`, or rewrite it under
/// `SEALMAP_DENSE_BLESS=1`.
fn golden(name: &str, actual: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(name);
    if std::env::var_os("SEALMAP_DENSE_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (run with SEALMAP_DENSE_BLESS=1 to create it)", path.display()));
    assert!(
        expected == actual,
        "{name} differs from the golden file; rerun with SEALMAP_DENSE_BLESS=1 and review the diff\n--- actual ---\n{actual}"
    );
}

fn id(text: &str) -> SymbolId {
    SymbolId::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// The lines of the `# calls` section.
fn call_section(text: &str) -> Vec<&str> {
    let start = text.find("\n# calls\n").expect("a calls section") + "\n# calls\n".len();
    text[start..].lines().take_while(|l| !l.starts_with("# ")).collect()
}

const FRAGMENTS: &[&str] = &["alt", "else", "opt", "loop", "par", "and", "return", "break", "continue"];

fn is_fragment(line: &str) -> bool {
    let word = line.trim_start().split([' ', '(']).next().unwrap_or("");
    FRAGMENTS.contains(&word)
}

#[test]
fn golden_warehouse_text_and_index() {
    let out = render(&warehouse(), &DenseOptions::default());
    golden("warehouse.dense.txt", &out.text);
    golden("warehouse._index.txt", &out.index);
}

#[test]
fn golden_warehouse_slice() {
    let cb = warehouse();
    let dense = Dense::new(&cb);
    let reserve = id("sym:cargo warehouse . inventory/InventoryLedger#[StockStore]reserve().");
    golden("warehouse.slice-reserve.txt", &dense.slice([&reserve], &SliceOptions::new(2)).unwrap());
}

#[test]
fn two_runs_are_byte_identical() {
    let a = render(&warehouse(), &DenseOptions::default());
    let b = render(&warehouse(), &DenseOptions::default());
    assert_eq!(a, b);
    for depth in 1..6 {
        let opts = DenseOptions::with_max_depth(depth);
        assert_eq!(render(&warehouse(), &opts), render(&warehouse(), &opts), "depth {depth}");
    }
}

#[test]
fn slices_ignore_seed_order_and_duplicates() {
    let cb = warehouse();
    let dense = Dense::new(&cb);
    let place = id("sym:cargo warehouse . orders/place().");
    let settle = id("sym:cargo warehouse . ledger/settle().");
    let opts = SliceOptions::new(2);
    let forward = dense.slice([&place, &settle], &opts).unwrap();
    let backward = dense.slice([&settle, &place, &settle], &opts).unwrap();
    assert_eq!(forward, backward);
    // A fresh `Dense` over a fresh extraction gives the same bytes.
    let again = warehouse();
    assert_eq!(Dense::new(&again).slice([&place, &settle], &opts).unwrap(), forward);
}

#[test]
fn every_call_site_appears_exactly_once_at_any_depth() {
    let cb = warehouse();
    let sites = cb.stats().calls;
    for depth in 1..6 {
        let text = render(&cb, &DenseOptions::with_max_depth(depth)).text;
        let calls = call_section(&text).into_iter().filter(|l| l.starts_with(' ') && !is_fragment(l)).count();
        assert_eq!(calls, sites, "depth {depth}");
    }
}

#[test]
fn every_callable_is_expanded_exactly_once() {
    let cb = warehouse();
    let dense = Dense::new(&cb);
    let text = render(&cb, &DenseOptions::with_max_depth(2)).text;
    let lines = call_section(&text);
    for sym in cb.symbols.values().filter(|s| s.flow.as_ref().is_some_and(|f| !f.is_empty())) {
        let short = dense.short(&sym.id).unwrap();
        let expanded = lines
            .iter()
            .filter(|l| {
                let body = l.trim_start().trim_start_matches(['~', '?']);
                let named =
                    body.strip_prefix(short).is_some_and(|rest| rest.is_empty() || rest.starts_with(['(', ' ']));
                // A tree header (unindented) is an expansion whatever its mark.
                named && (!l.starts_with(' ') || !(l.ends_with(" ^") || l.ends_with(" ↺") || l.ends_with(" …")))
            })
            .count();
        assert_eq!(expanded, 1, "{short} expanded {expanded} times\n{text}");
    }
}

#[test]
fn cycles_are_marked_not_followed() {
    let text = render(&warehouse(), &DenseOptions::default()).text;
    let lines = call_section(&text);
    // Self recursion.
    assert!(lines.iter().any(|l| l.trim_start() == "audit_trail(_) ↺"), "{text}");
    // Cut at the default depth of 3, then given its own tree.
    assert!(lines.iter().any(|l| l.trim_start() == "chain_delta() …"), "{text}");
    assert!(lines.contains(&"chain_delta …"), "{text}");
    // Mutual recursion (settle -> reconcile -> settle) once nothing cuts it.
    let deep = render(&warehouse(), &DenseOptions::with_max_depth(10)).text;
    let lines = call_section(&deep);
    let settle = lines.iter().position(|l| l.trim_start() == "settle(1)").expect(&deep);
    assert_eq!(lines[settle + 3..settle + 5], ["      reconcile(_)", "       settle(depth) ↺"], "{deep}");
}

#[test]
fn over_budget_slices_are_refused_with_the_overflow_named() {
    let cb = warehouse();
    let dense = Dense::new(&cb);
    let place = id("sym:cargo warehouse . orders/place().");
    let settle = id("sym:cargo warehouse . ledger/settle().");
    let full = dense.slice([&place, &settle], &SliceOptions::new(2)).unwrap();
    // Exactly at the budget: accepted, unchanged.
    assert_eq!(dense.slice([&place, &settle], &SliceOptions::new(2).max_bytes(full.len())).unwrap(), full);
    // One byte under: refused, not truncated.
    let err = dense.slice([&place, &settle], &SliceOptions::new(2).max_bytes(full.len() - 1)).unwrap_err();
    let SliceError::OverBudget { bytes, budget, seeds } = &err else { panic!("{err:?}") };
    assert_eq!((*bytes, *budget), (full.len(), full.len() - 1));
    let alone: Vec<usize> =
        [&settle, &place].iter().map(|s| dense.slice([*s], &SliceOptions::new(2)).unwrap().len()).collect();
    assert_eq!(seeds, &vec![(settle.clone(), alone[0]), (place.clone(), alone[1])]);
    let message = err.to_string();
    assert!(message.contains(&format!("{} bytes, 1 over the {}-byte budget", full.len(), full.len() - 1)), "{message}");
    assert!(message.contains("refusing rather than truncating"), "{message}");
}

#[test]
fn unknown_seeds_are_refused_by_name() {
    let cb = warehouse();
    let ghost = id("sym:cargo warehouse . ghost().");
    let place = id("sym:cargo warehouse . orders/place().");
    let err = sealmap_dense::slice(&cb, [&place, &ghost], &SliceOptions::default()).unwrap_err();
    assert_eq!(err, SliceError::UnknownSymbols(vec![ghost.clone()]));
    assert!(err.to_string().contains("sym:cargo warehouse . ghost()."));
}

#[test]
fn depth_zero_slices_are_skeletons_and_index_only() {
    let cb = warehouse();
    let place = id("sym:cargo warehouse . orders/place().");
    let text = sealmap_dense::slice(&cb, [&place], &SliceOptions::new(0)).unwrap();
    assert!(!text.contains("# calls") && !text.contains("# callers"), "{text}");
    assert!(text.ends_with("# index\nplace sym:cargo warehouse . orders/place(). src/orders.rs:L24-33\n"), "{text}");
}

#[test]
fn index_lines_resolve_both_ways() {
    let cb = warehouse();
    let dense = Dense::new(&cb);
    let index = dense.index();
    assert_eq!(index.lines().count(), cb.symbols.len());
    for line in index.lines() {
        let (short, rest) = line.split_once(' ').unwrap();
        let (sym, location) = rest.rsplit_once(' ').unwrap();
        let sym = id(sym);
        assert_eq!(dense.short(&sym), Some(short));
        assert_eq!(dense.resolve(short), Some(&sym));
        let s = cb.symbol(&sym).unwrap();
        assert_eq!(location, format!("{}:L{}-{}", s.file, s.span.start_line, s.span.end_line));
    }
}

#[test]
fn output_files_have_fixed_names() {
    let out = render(&warehouse(), &DenseOptions::default());
    assert_eq!(out.files().map(|(n, _)| n), [INDEX_FILE, TEXT_FILE]);
}

/// A hand-built codebase where nearly every base name collides.
fn collisions() -> Codebase {
    let mut cb = Codebase::new("clash");
    let file = SourcePath::new("src/lib.rs").unwrap();
    let mut line = 0;
    let mut add = |cb: &mut Codebase, text: &str, kind: SymbolKind| {
        let sym = id(text);
        line += 1;
        let mut s = Symbol::new(sym.clone(), sym.name().into_owned(), kind, file.clone());
        s.span = Span::new(line, 1, line, 2);
        cb.add_symbol(s);
    };
    for (text, kind) in [
        // Same function name in two modules.
        ("sym:cargo alpha . orders/validate().", SymbolKind::Function),
        ("sym:cargo alpha . stock/validate().", SymbolKind::Function),
        // Same type name in two modules, each with a method `new`.
        ("sym:cargo alpha . orders/Ledger#", SymbolKind::Struct),
        ("sym:cargo alpha . stock/Ledger#", SymbolKind::Struct),
        ("sym:cargo alpha . orders/Ledger#new().", SymbolKind::Method),
        ("sym:cargo alpha . stock/Ledger#new().", SymbolKind::Method),
        // Two trait impls with the same method on one type.
        ("sym:cargo alpha . Order#[`From<Line>`]from().", SymbolKind::Method),
        ("sym:cargo alpha . Order#[`From<Vec<Line>>`]from().", SymbolKind::Method),
        ("sym:cargo alpha . Order#", SymbolKind::Struct),
        // The same path in two packages.
        ("sym:cargo alpha . net/Client#connect().", SymbolKind::Method),
        ("sym:cargo beta . net/Client#connect().", SymbolKind::Method),
        // Identical but for the method disambiguator: only a hash separates them.
        ("sym:cargo alpha . Codec#encode(1).", SymbolKind::Method),
        ("sym:cargo alpha . Codec#encode(2).", SymbolKind::Method),
        // Modules with the same last name, and a function named like one.
        ("sym:cargo alpha . orders/net/", SymbolKind::Module),
        ("sym:cargo alpha . net/", SymbolKind::Module),
        ("sym:cargo alpha . net().", SymbolKind::Function),
        // A short that a qualified name could collide with.
        ("sym:cargo alpha . Ledger#new().", SymbolKind::Method),
        // Package roots.
        ("sym:cargo alpha .", SymbolKind::Module),
        ("sym:cargo beta .", SymbolKind::Module),
    ] {
        add(&mut cb, text, kind);
    }
    cb
}

#[test]
fn short_names_are_unique_under_collisions() {
    let cb = collisions();
    let dense = Dense::new(&cb);
    let shorts: BTreeSet<&str> = cb.symbols.keys().map(|id| dense.short(id).unwrap()).collect();
    assert_eq!(shorts.len(), cb.symbols.len());
    for sym in cb.symbols.keys() {
        let short = dense.short(sym).unwrap();
        assert!(!short.is_empty() && !short.contains(char::is_whitespace), "{short:?}");
        assert_eq!(dense.resolve(short), Some(sym));
    }
    let short = |text: &str| dense.short(&id(text)).unwrap().to_owned();
    assert_eq!(short("sym:cargo alpha . orders/validate()."), "orders/validate");
    assert_eq!(short("sym:cargo alpha . stock/validate()."), "stock/validate");
    assert_eq!(short("sym:cargo alpha . orders/Ledger#"), "orders/Ledger");
    assert_eq!(short("sym:cargo alpha . stock/Ledger#new()."), "stock/Ledger.new");
    // All three `Ledger.new`s move up together: none keeps the bare name.
    assert_eq!(short("sym:cargo alpha . Ledger#new()."), "alpha:Ledger.new");
    assert_eq!(short("sym:cargo alpha . Order#[`From<Line>`]from()."), "Order[From<Line>].from");
    assert_eq!(short("sym:cargo alpha . Order#[`From<Vec<Line>>`]from()."), "Order[From<Vec<Line>>].from");
    assert_eq!(short("sym:cargo alpha . Order#"), "Order");
    assert_eq!(short("sym:cargo alpha . net/Client#connect()."), "alpha:net/Client.connect");
    assert_eq!(short("sym:cargo beta . net/Client#connect()."), "beta:net/Client.connect");
    assert!(short("sym:cargo alpha . Codec#encode(1).").starts_with("alpha:Codec.encode'"));
    assert_eq!(short("sym:cargo alpha . orders/net/"), "orders/net/");
    assert_eq!(short("sym:cargo alpha . net/"), "alpha:net/");
    assert_eq!(short("sym:cargo alpha . net()."), "net");
    assert_eq!(short("sym:cargo alpha ."), "alpha/");
    assert_eq!(short("sym:cargo beta ."), "beta/");
}

#[test]
fn short_names_do_not_depend_on_insertion_order() {
    let forward = collisions();
    let mut reversed = Codebase::new("clash");
    for sym in forward.symbols.values().rev() {
        reversed.add_symbol(sym.clone());
    }
    assert_eq!(Dense::new(&forward).index(), Dense::new(&reversed).index());
}

/// A cycle with no entry point still gets a tree, and its calls are listed.
#[test]
fn a_pure_cycle_gets_a_tree() {
    let pkg = Package::current("cargo", "ring").unwrap();
    let root = SymbolId::package_root(pkg);
    let file = SourcePath::new("src/lib.rs").unwrap();
    let ids: Vec<SymbolId> =
        ["ping", "pong", "pang"].iter().map(|n| root.child(Descriptor::method(*n)).unwrap()).collect();
    let mut cb = Codebase::new("ring");
    for (i, sym) in ids.iter().enumerate() {
        let next = &ids[(i + 1) % ids.len()];
        let mut s = Symbol::new(sym.clone(), sym.name().into_owned(), SymbolKind::Function, file.clone());
        s.span = Span::new(i as u32 * 3 + 1, 1, i as u32 * 3 + 3, 2);
        s.flow =
            Some(Flow::new(vec![Step::Call(Call::new(next.clone(), format!("{}()", next.name()), Confidence::Exact))]));
        cb.add_symbol(s);
    }
    let text = render(&cb, &DenseOptions::default()).text;
    assert_eq!(call_section(&text), ["ping ↺", " pong()", "  pang()", "   ping() ↺"]);
}
