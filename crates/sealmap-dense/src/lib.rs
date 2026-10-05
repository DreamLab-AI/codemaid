//! # sealmap-dense
//!
//! The compact **agent projection** of a [`sealmap_model::Codebase`]: what an
//! LLM agent reads instead of the source, or instead of a Mermaid corpus,
//! when it needs the shape of a codebase. Measured on sealmap's own tree it
//! costs well under half the bytes of the source it describes, while carrying
//! what source leaves implicit: the resolved call graph, with every edge
//! marked exact, inferred or external.
//!
//! The projection is a pure function of the model: the same codebase and
//! options give byte-identical text on every run and every machine.
//!
//! ## What it writes
//!
//! [`Dense::render`] produces two texts, normally written to
//! [`TEXT_FILE`] and [`INDEX_FILE`]:
//!
//! ```text
//! # sealmap-dense 1 warehouse: L<a>-<b> source lines; @name short names (see _index.txt); calls ~inferred ?external, ^ expanded above, ↺ cycle, … cut; tree roots: bare = entry point, … = continued, ↺ = cycle-only
//! # symbols
//! ## src/inventory.rs
//! pub trait StockStore { fn reserve(&mut self, sku: &str, qty: u32) -> Result<(), StockError>, fn lookup(&self, sku: &str) -> u32 } L3-9 @StockStore
//! pub struct InventoryLedger { levels: BTreeMap<String, u32>, journal: Vec<String> } : StockStore L17-20 @inventory/InventoryLedger
//! pub fn reserve(&mut self, sku: &str, qty: u32) -> Result<(), StockError> L33-40 @InventoryLedger.reserve
//! …
//! # calls
//! place
//!  orders/validate(&order)
//!   loop each via all
//!    inventory/validate(&line.sku)
//!  opt !validate(&order)
//!   return Ok(0)
//!  loop for line in &order.lines
//!   InventoryLedger.reserve(&line.sku, line.qty)?
//!    StockStore.available(sku)
//!    opt have<qty
//!     return Err(StockError::Short { sku: sku.into(), wanted:…
//!    InventoryLedger.record(_)
//!  notify(len()).await
//!   ?serde_json::to_string(&count)
//! ```
//!
//! * **Skeleton lines**, one per symbol, grouped by file and in source order:
//!   visibility, the signature as written (or the kind and name), fields or
//!   variants inline (field visibility is left out), implemented traits after
//!   ` : `, the line span `L<start>-<end>`, and the short name after `@`.
//! * **Call trees**, each call indented one space under its caller, with
//!   `alt`/`else`, `opt`, `loop`, `par`/`and` and early-exit lines from the
//!   flow. Every callable is expanded **once**: afterwards a call to it ends
//!   in `^`, a call back into a callable still being expanded ends in `↺`,
//!   and a call reached at the depth limit ends in `…`. So the trees grow
//!   with the number of call sites, not the number of paths, and every call
//!   site appears exactly once.
//! * **Tree headers** (unindented lines) say why the tree exists: a bare name
//!   is an *entry point*, which nothing in the codebase calls; `name …`
//!   continues a call cut above; `name ↺` is a callable reached only through
//!   a cycle.
//! * **Confidence marks** prefix a call: nothing for exact, `~` for inferred,
//!   `?` for external. A trailing `.await` or `?` is the call site's own.
//!   Internal callees are named by short name, others by their path.
//! * **The index** (`_index.txt`), one line per symbol ordered by short name:
//!   `<short> <sym:id> <path>:L<start>-<end>`. The short name is the first
//!   field and contains no whitespace; the location is the last field (`%`
//!   and whitespace in a path are percent-encoded); the `sym:` id is
//!   everything between.
//!
//! Short names are unique within an output and derived from the ids alone:
//! `Db.insert` for a method, `connect` for a function, `db/` for a module.
//! Where two symbols would share one, *both* are qualified, step by step:
//! the trait-impl block (`Db[Store].put`), the module path (`db/Db.put`),
//! the package (`shop:db/Db.put`), then a hash of the id.
//!
//! ## Slices
//!
//! [`Dense::slice`] is what a review pack embeds: the dense text for a set of
//! symbols plus their callers and callees to a depth, with its own index
//! section, byte-identical for identical inputs. With a byte budget it
//! **refuses** an oversized slice with [`SliceError::OverBudget`], naming the
//! overflow and the size of each seed alone, rather than truncating.
//!
//! ## Example
//!
//! ```
//! use sealmap_model::*;
//! use sealmap_dense::{Dense, DenseOptions, SliceError, SliceOptions};
//!
//! let pkg = Package::current("cargo", "shop").unwrap();
//! let root = SymbolId::package_root(pkg);
//! let db = root.child(Descriptor::r#type("Db")).unwrap();
//! let insert = db.child(Descriptor::method("insert")).unwrap();
//! let place = root.child(Descriptor::method("place")).unwrap();
//! let file = SourcePath::new("src/lib.rs").unwrap();
//!
//! let mut cb = Codebase::new("shop");
//! cb.add_symbol(Symbol::new(db.clone(), "Db", SymbolKind::Struct, file.clone()));
//! let mut m = Symbol::new(insert.clone(), "insert", SymbolKind::Method, file.clone());
//! m.signature = Some("pub fn insert(&mut self, id: u64)".into());
//! m.visibility = Visibility::Public;
//! m.span = Span::new(4, 5, 6, 6);
//! cb.add_symbol(m);
//! let mut f = Symbol::new(place.clone(), "place", SymbolKind::Function, file.clone());
//! f.signature = Some("fn place(db: &mut Db)".into());
//! f.span = Span::new(8, 1, 10, 2);
//! f.flow = Some(Flow::new(vec![Step::Call(Call::new(insert.clone(), "insert(7)", Confidence::Exact))]));
//! cb.add_symbol(f);
//!
//! let dense = Dense::new(&cb);
//! let out = dense.render(&DenseOptions::default());
//! assert!(out.text.contains("pub fn insert(&mut self, id: u64) L4-6 @Db.insert\n"));
//! assert!(out.text.ends_with("# calls\nplace\n Db.insert(7)\n"));
//! assert_eq!(dense.resolve("Db.insert"), Some(&insert));
//! assert!(out.index.contains("Db.insert sym:cargo shop . Db#insert(). src/lib.rs:L4-6\n"));
//!
//! // A slice around `insert`, refused when it does not fit.
//! let slice = dense.slice([&insert], &SliceOptions::new(1)).unwrap();
//! assert!(slice.contains("# callers\nDb.insert\n place\n"));
//! let err = dense.slice([&insert], &SliceOptions::new(1).max_bytes(64)).unwrap_err();
//! assert!(matches!(err, SliceError::OverBudget { budget: 64, .. }));
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod short;
mod skeleton;
mod tree;

use std::collections::BTreeSet;
use std::fmt;

use sealmap_model::{Codebase, SymbolId};

use crate::short::ShortNames;
use crate::skeleton::{one_line, order_key};
use crate::tree::{CallerWriter, Graph, TreeWriter};

/// Version of the dense text format, written in its first line.
///
/// Bumped whenever the meaning of a mark or the layout of a line changes, so
/// a tool reading the text can refuse a version it does not know.
pub const FORMAT_VERSION: u32 = 1;

/// File name for [`DenseOutput::text`] in a generated directory.
pub const TEXT_FILE: &str = "dense.txt";

/// File name for [`DenseOutput::index`] in a generated directory.
pub const INDEX_FILE: &str = "_index.txt";

/// The legend after the first line's codebase name.
const LEGEND: &str = "L<a>-<b> source lines; @name short names (see _index.txt); calls ~inferred ?external, ^ expanded above, ↺ cycle, … cut; tree roots: bare = entry point, … = continued, ↺ = cycle-only";

/// Options for the full projection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DenseOptions {
    /// Call levels expanded beneath a tree's root before a callee is cut
    /// (`…`) and given a tree of its own. At least 1; default 3.
    pub max_depth: usize,
}

impl Default for DenseOptions {
    fn default() -> Self {
        Self { max_depth: 3 }
    }
}

impl DenseOptions {
    /// Options with a given depth limit (clamped to at least 1).
    ///
    /// ```
    /// assert_eq!(sealmap_dense::DenseOptions::with_max_depth(0).max_depth, 1);
    /// ```
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self { max_depth: max_depth.max(1) }
    }
}

/// The two texts of a full projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseOutput {
    /// Skeletons and call trees ([`TEXT_FILE`]).
    pub text: String,
    /// One `<short> <sym:id> <path>:L<a>-<b>` line per symbol ([`INDEX_FILE`]).
    pub index: String,
}

impl DenseOutput {
    /// `(file name, contents)` pairs, in a fixed order, for writing to disk.
    ///
    /// ```
    /// let cb = sealmap_model::Codebase::new("empty");
    /// let out = sealmap_dense::render(&cb, &Default::default());
    /// let names: Vec<&str> = out.files().iter().map(|(n, _)| *n).collect();
    /// assert_eq!(names, ["_index.txt", "dense.txt"]);
    /// ```
    pub fn files(&self) -> [(&'static str, &str); 2] {
        [(INDEX_FILE, &self.index), (TEXT_FILE, &self.text)]
    }
}

/// Options for [`Dense::slice`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SliceOptions {
    /// Call levels followed from each seed, both towards callees and towards
    /// callers. 0 gives the seeds' skeleton lines only.
    pub depth: usize,
    /// Refuse a slice longer than this many bytes. `None`: no limit.
    pub max_bytes: Option<usize>,
}

impl SliceOptions {
    /// A slice to `depth` call levels, with no byte limit.
    pub fn new(depth: usize) -> Self {
        Self { depth, max_bytes: None }
    }

    /// The same options with a byte budget.
    pub fn max_bytes(mut self, max_bytes: usize) -> Self {
        self.max_bytes = Some(max_bytes);
        self
    }
}

impl Default for SliceOptions {
    fn default() -> Self {
        Self::new(1)
    }
}

/// Why [`Dense::slice`] refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SliceError {
    /// Seeds that are not symbols of the codebase, in id order.
    UnknownSymbols(Vec<SymbolId>),
    /// The slice does not fit the budget. Nothing was truncated.
    OverBudget {
        /// Size of the whole slice in bytes.
        bytes: usize,
        /// The budget it exceeded.
        budget: usize,
        /// Each seed with the size of its slice alone (same depth), in id
        /// order: what a caller needs to plan shards.
        seeds: Vec<(SymbolId, usize)>,
    },
}

impl fmt::Display for SliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSymbols(ids) => {
                write!(f, "{} seed id(s) are not symbols of the codebase:", ids.len())?;
                for id in ids {
                    write!(f, " {id};")?;
                }
                Ok(())
            }
            Self::OverBudget { bytes, budget, seeds } => {
                write!(
                    f,
                    "dense slice is {bytes} bytes, {} over the {budget}-byte budget; refusing rather than truncating",
                    bytes - budget
                )?;
                if let Some((id, size)) = seeds.iter().max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0))) {
                    write!(f, " (largest seed alone: {id} at {size} bytes)")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for SliceError {}

/// A codebase prepared for dense projection: short names and the call graph
/// are computed once, then any number of renders and slices share them.
#[derive(Debug, Clone)]
pub struct Dense<'a> {
    cb: &'a Codebase,
    shorts: ShortNames<'a>,
    graph: Graph<'a>,
}

impl<'a> Dense<'a> {
    /// Index `cb`: assign short names and read the call graph from the flows.
    pub fn new(cb: &'a Codebase) -> Self {
        Self { cb, shorts: ShortNames::new(cb), graph: Graph::new(cb) }
    }

    /// The short name of a symbol, or `None` if `id` is not in the codebase.
    pub fn short(&self, id: &SymbolId) -> Option<&str> {
        self.shorts.get(id)
    }

    /// The symbol a short name stands for.
    pub fn resolve(&self, short: &str) -> Option<&'a SymbolId> {
        self.shorts.resolve(short)
    }

    /// The full projection: [`text`](Self::text) and [`index`](Self::index).
    pub fn render(&self, options: &DenseOptions) -> DenseOutput {
        DenseOutput { text: self.text(options), index: self.index() }
    }

    /// The `_index.txt` text: every symbol, ordered by short name.
    pub fn index(&self) -> String {
        let mut out = String::new();
        for (short, id) in self.shorts.iter() {
            self.index_line(&mut out, short, id);
        }
        out
    }

    /// Skeleton lines for every symbol, then the call trees.
    pub fn text(&self, options: &DenseOptions) -> String {
        let mut out = self.header("");
        out.push_str("# symbols\n");
        skeleton::write_files(&mut out, self.cb, &self.shorts, self.cb.symbols.values());
        out.push_str("# calls\n");
        let mut writer = TreeWriter::new(self.cb, &self.shorts, &self.graph, options.max_depth.max(1), true);
        let callables = self.in_source_order(self.graph.flows.keys().copied());
        // Entry points first, then whatever only cycles reach.
        for &id in &callables {
            if !self.graph.callers.contains_key(id) {
                writer.root(id, "");
            }
        }
        for &id in &callables {
            if !writer.is_expanded(id) {
                writer.root(id, " ↺");
            }
        }
        out.push_str(&writer.out);
        out
    }

    /// The dense text for `seeds` plus their callees and callers to
    /// `options.depth` call levels: skeleton lines, call trees from each
    /// seed, caller trees up to each seed, and an index section for every
    /// symbol named. Seeds are a set: order and duplicates do not matter.
    ///
    /// # Errors
    ///
    /// [`SliceError::UnknownSymbols`] if a seed is not in the codebase, and
    /// [`SliceError::OverBudget`] if the text exceeds `options.max_bytes`.
    pub fn slice<'s, I>(&self, seeds: I, options: &SliceOptions) -> Result<String, SliceError>
    where
        I: IntoIterator<Item = &'s SymbolId>,
    {
        let mut known = BTreeSet::new();
        let mut unknown = BTreeSet::new();
        for id in seeds {
            match self.cb.symbols.get_key_value(id) {
                Some((k, _)) => {
                    known.insert(k);
                }
                None => {
                    unknown.insert(id.clone());
                }
            }
        }
        if !unknown.is_empty() {
            return Err(SliceError::UnknownSymbols(unknown.into_iter().collect()));
        }
        let text = self.slice_text(&known, options.depth);
        match options.max_bytes {
            Some(budget) if text.len() > budget => {
                let seeds = known
                    .iter()
                    .map(|id| ((*id).clone(), self.slice_text(&BTreeSet::from([*id]), options.depth).len()))
                    .collect();
                Err(SliceError::OverBudget { bytes: text.len(), budget, seeds })
            }
            _ => Ok(text),
        }
    }

    fn slice_text(&self, seeds: &BTreeSet<&'a SymbolId>, depth: usize) -> String {
        let callees = self.reach(seeds, depth, |id| self.graph.callees.get(id).map(|s| s.iter().copied().collect()));
        let callers = self.reach(seeds, depth, |id| self.graph.callers.get(id).map(|m| m.keys().copied().collect()));
        let members: BTreeSet<&'a SymbolId> = seeds.iter().chain(&callees).chain(&callers).copied().collect();

        let mut out = self.header(&format!(" slice depth {depth}"));
        out.push_str("# symbols\n");
        skeleton::write_files(&mut out, self.cb, &self.shorts, members.iter().filter_map(|id| self.cb.symbol(id)));
        let ordered = self.in_source_order(seeds.iter().copied());
        if depth > 0 {
            let mut calls = TreeWriter::new(self.cb, &self.shorts, &self.graph, depth, false);
            for &id in &ordered {
                calls.root(id, "");
            }
            if !calls.out.is_empty() {
                out.push_str("# calls\n");
                out.push_str(&calls.out);
            }
            let mut up = CallerWriter::new(self.cb, &self.shorts, &self.graph, depth);
            for &id in &ordered {
                up.root(id);
            }
            if !up.out.is_empty() {
                out.push_str("# callers\n");
                out.push_str(&up.out);
            }
        }
        out.push_str("# index\n");
        let mut named: Vec<(&str, &SymbolId)> =
            members.iter().filter_map(|id| self.shorts.get(id).map(|s| (s, *id))).collect();
        named.sort();
        for (short, id) in named {
            self.index_line(&mut out, short, id);
        }
        out
    }

    /// Every symbol within `depth` steps of `seeds` along `next`, seeds
    /// excluded.
    fn reach(
        &self,
        seeds: &BTreeSet<&'a SymbolId>,
        depth: usize,
        next: impl Fn(&SymbolId) -> Option<Vec<&'a SymbolId>>,
    ) -> BTreeSet<&'a SymbolId> {
        let mut seen: BTreeSet<&'a SymbolId> = seeds.clone();
        let mut frontier: Vec<&'a SymbolId> = seeds.iter().copied().collect();
        for _ in 0..depth {
            let mut grown = Vec::new();
            for id in frontier {
                for n in next(id).unwrap_or_default() {
                    if seen.insert(n) {
                        grown.push(n);
                    }
                }
            }
            frontier = grown;
        }
        seen.retain(|id| !seeds.contains(id));
        seen
    }

    fn in_source_order(&self, ids: impl Iterator<Item = &'a SymbolId>) -> Vec<&'a SymbolId> {
        let mut v: Vec<&'a SymbolId> = ids.collect();
        v.sort_by(|a, b| match (self.cb.symbol(a), self.cb.symbol(b)) {
            (Some(x), Some(y)) => order_key(x).cmp(&order_key(y)),
            _ => a.cmp(b),
        });
        v
    }

    fn header(&self, kind: &str) -> String {
        format!("# sealmap-dense {FORMAT_VERSION}{kind} {}: {LEGEND}\n", one_line(&self.cb.name))
    }

    fn index_line(&self, out: &mut String, short: &str, id: &SymbolId) {
        out.push_str(short);
        out.push(' ');
        out.push_str(id.as_str());
        if let Some(sym) = self.cb.symbol(id) {
            out.push(' ');
            encode_path(out, sym.file.as_str());
            out.push(':');
            skeleton::span(out, sym);
        }
        out.push('\n');
    }
}

/// The full projection of `codebase` in one call; see [`Dense::render`].
pub fn render(codebase: &Codebase, options: &DenseOptions) -> DenseOutput {
    Dense::new(codebase).render(options)
}

/// One slice of `codebase` in one call; see [`Dense::slice`]. Build a
/// [`Dense`] once instead when taking several slices of the same codebase.
///
/// # Errors
///
/// As [`Dense::slice`].
pub fn slice<'s, I>(codebase: &Codebase, seeds: I, options: &SliceOptions) -> Result<String, SliceError>
where
    I: IntoIterator<Item = &'s SymbolId>,
{
    Dense::new(codebase).slice(seeds, options)
}

/// A path with `%`, whitespace and control characters percent-encoded, so
/// the location stays the last whitespace-free field of an index line.
fn encode_path(out: &mut String, path: &str) {
    for c in path.chars() {
        if c == '%' || c.is_whitespace() || c.is_control() {
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        } else {
            out.push(c);
        }
    }
}

/// Compiles and runs the README examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_encoded_injectively() {
        let mut s = String::new();
        encode_path(&mut s, "a b/100%.rs");
        assert_eq!(s, "a%20b/100%25.rs");
    }
}
