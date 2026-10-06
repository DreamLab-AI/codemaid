//! E0b region extraction (`docs/evidence/E0b/PREREG.md`, "The region rule").
//!
//! A Rust citation that E0 maps to symbol S is narrowed to the innermost of a
//! fixed list of syntax nodes inside S whose line span contains the cited
//! line. The region is named by its **node path** from S, a list of
//! (kind, ordinal) pairs where the ordinal counts earlier siblings of the same
//! kind under the same parent region, and fingerprinted with sealmap 0.2.0's
//! own body-token normalisation: the normaliser source is compiled in from
//! `sealmap-rust` (see `canon` in `main.rs`), never re-implemented here.
//!
//! Nodes are found on the original syntax tree (its spans are intact); a
//! node's hash is taken on a canonical copy built the way sealmap builds it,
//! so a region hashes the same tokens it contributes to S's `body_hash`.

use std::collections::BTreeMap;

use serde::Serialize;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::visit_mut::VisitMut;

use crate::canon::{Canon, canonical, feed};
use crate::model::Sym;
use sealmap_extract::fingerprint::Fingerprinter;

/// The node kinds the rule lists, in the order it lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A `match` arm.
    Arm,
    /// An `if` / `else if` / `else` branch block.
    Branch,
    /// A `while`, `loop` or `for` body.
    Loop,
    /// A closure body.
    Closure,
    /// An `async` block.
    Async,
    /// A statement directly inside a block.
    Stmt,
    /// A struct (or variant) field.
    Field,
    /// An enum variant.
    Variant,
    /// A trait item.
    TraitItem,
    /// An impl item.
    ImplItem,
    /// A top-level item of the file (only when S is the file's root module).
    Item,
}

impl Kind {
    /// Every kind, for tables.
    pub const ALL: [Kind; 11] = [
        Kind::Arm,
        Kind::Branch,
        Kind::Loop,
        Kind::Closure,
        Kind::Async,
        Kind::Stmt,
        Kind::Field,
        Kind::Variant,
        Kind::TraitItem,
        Kind::ImplItem,
        Kind::Item,
    ];

    /// Label used in paths and reports.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Arm => "arm",
            Kind::Branch => "branch",
            Kind::Loop => "loop",
            Kind::Closure => "closure",
            Kind::Async => "async",
            Kind::Stmt => "stmt",
            Kind::Field => "field",
            Kind::Variant => "variant",
            Kind::TraitItem => "trait_item",
            Kind::ImplItem => "impl_item",
            Kind::Item => "item",
        }
    }
}

/// A node path from S: empty means the region is S itself.
pub type RegionPath = Vec<(Kind, u32)>;

/// `stmt#3/branch#0/stmt#1`, or `(symbol)` for S itself.
pub fn path_label(p: &[(Kind, u32)]) -> String {
    if p.is_empty() {
        return "(symbol)".into();
    }
    p.iter().map(|(k, i)| format!("{}#{i}", k.label())).collect::<Vec<_>>().join("/")
}

/// A region node, borrowed from a parsed file.
#[derive(Clone, Copy)]
pub enum NodeRef<'a> {
    Arm(&'a syn::Arm),
    /// A branch or loop body.
    Block(&'a syn::Block),
    /// The closure whose body is the region.
    Closure(&'a syn::ExprClosure),
    Async(&'a syn::ExprAsync),
    /// Statement `.1` of block `.0`.
    Stmt(&'a syn::Block, usize),
    Field(&'a syn::Field),
    Variant(&'a syn::Variant),
    TraitItem(&'a syn::TraitItem),
    ImplItem(&'a syn::ImplItem),
    Item(&'a syn::Item),
}

impl NodeRef<'_> {
    /// First and last line of the node (attributes and doc comments included,
    /// as in sealmap's symbol spans).
    pub fn lines(&self) -> (u32, u32) {
        let s = match self {
            NodeRef::Arm(a) => a.span(),
            NodeRef::Block(b) => b.span(),
            NodeRef::Closure(c) => c.body.span(),
            NodeRef::Async(a) => a.span(),
            NodeRef::Stmt(b, i) => b.stmts[*i].span(),
            NodeRef::Field(f) => f.span(),
            NodeRef::Variant(v) => v.span(),
            NodeRef::TraitItem(t) => t.span(),
            NodeRef::ImplItem(i) => i.span(),
            NodeRef::Item(i) => i.span(),
        };
        (s.start().line as u32, s.end().line as u32)
    }

    /// BLAKE3-16 region hash: the node's tokens through sealmap's body
    /// normalisation. Context-dependent canonical rewrites are applied the
    /// way they apply inside S: a closure body is taken from the canonical
    /// closure (`|x| { f(x) }` is `|x| f(x)`), a statement from its canonical
    /// block (a final `return` carries its `;`).
    pub fn hash(&self) -> [u8; 16] {
        let mut fp = Fingerprinter::body();
        match self {
            NodeRef::Arm(a) => feed(&mut fp, &canonical(*a, Canon::visit_arm_mut)),
            NodeRef::Block(b) => feed(&mut fp, &canonical(*b, Canon::visit_block_mut)),
            NodeRef::Closure(c) => feed(&mut fp, &canonical(*c, Canon::visit_expr_closure_mut).body),
            NodeRef::Async(a) => feed(&mut fp, &canonical(*a, Canon::visit_expr_async_mut)),
            NodeRef::Stmt(b, i) => feed(&mut fp, &canonical(*b, Canon::visit_block_mut).stmts[*i]),
            NodeRef::Field(f) => feed(&mut fp, &canonical(*f, Canon::visit_field_mut)),
            NodeRef::Variant(v) => feed(&mut fp, &canonical(*v, Canon::visit_variant_mut)),
            NodeRef::TraitItem(t) => feed(&mut fp, &canonical(*t, Canon::visit_trait_item_mut)),
            NodeRef::ImplItem(i) => feed(&mut fp, &canonical(*i, Canon::visit_impl_item_mut)),
            NodeRef::Item(i) => feed(&mut fp, &canonical(*i, Canon::visit_item_mut)),
        }
        *fp.finish().as_bytes()
    }
}

/// The syntax node of a sealmap symbol.
#[derive(Clone, Copy)]
pub enum SymNode<'a> {
    /// A file's root module.
    File(&'a syn::File),
    Item(&'a syn::Item),
    ImplItem(&'a syn::ImplItem),
    TraitItem(&'a syn::TraitItem),
    Field(&'a syn::Field),
    Variant(&'a syn::Variant),
}

/// sealmap's span convention (`collect.rs` `span_of`): 1-based lines and columns.
fn span4(s: proc_macro2::Span) -> (u32, u32, u32, u32) {
    let (a, b) = (s.start(), s.end());
    (a.line as u32, a.column as u32 + 1, b.line as u32, b.column as u32 + 1)
}

/// Find S's node in `file` by its exact span (lines and columns). A file's
/// root module spans `1:1` to `last line:1` and is the file itself; every
/// other symbol is an item, impl item, trait item, field or variant whose
/// joined span is the symbol's span. Ties go to the first node in pre-order.
pub fn symbol_node<'a>(file: &'a syn::File, s: &Sym) -> Option<SymNode<'a>> {
    if s.module && (s.start, s.start_col, s.end_col) == (1, 1, 1) {
        return Some(SymNode::File(file));
    }
    let want = (s.start, s.start_col, s.end, s.end_col);
    find_in_items(&file.items, want)
}

fn find_fields(fields: &syn::Fields, want: (u32, u32, u32, u32)) -> Option<SymNode<'_>> {
    fields.iter().find(|f| span4(f.span()) == want).map(SymNode::Field)
}

fn find_in_items(items: &[syn::Item], want: (u32, u32, u32, u32)) -> Option<SymNode<'_>> {
    for it in items {
        if span4(it.span()) == want {
            return Some(SymNode::Item(it));
        }
        let inner = match it {
            syn::Item::Mod(m) => m.content.as_ref().and_then(|(_, items)| find_in_items(items, want)),
            syn::Item::Impl(i) => i.items.iter().find(|ii| span4(ii.span()) == want).map(SymNode::ImplItem),
            syn::Item::Trait(t) => t.items.iter().find(|ti| span4(ti.span()) == want).map(SymNode::TraitItem),
            syn::Item::Struct(s) => find_fields(&s.fields, want),
            syn::Item::Union(u) => u.fields.named.iter().find(|f| span4(f.span()) == want).map(SymNode::Field),
            syn::Item::Enum(e) => e.variants.iter().find_map(|v| {
                if span4(v.span()) == want { Some(SymNode::Variant(v)) } else { find_fields(&v.fields, want) }
            }),
            _ => None,
        };
        if inner.is_some() {
            return inner;
        }
    }
    None
}

/// If sealmap's canonical form drops the block around a match-arm or closure
/// body (`=> { e }` is `=> e`, `|x| { e }` is `|x| e`), the expression inside
/// it: the walk then goes straight to that expression, so the block's single
/// statement is not a region, and node paths agree across that formatter
/// rewrite exactly as the hashes do. `canon_body` is the body after sealmap's
/// own canonicalisation of the arm or closure, so the decision is sealmap's.
fn collapsed(body: &syn::Expr, canon_body: impl FnOnce() -> Box<syn::Expr>) -> Option<&syn::Expr> {
    let syn::Expr::Block(b) = body else { return None };
    let [syn::Stmt::Expr(inner, _)] = b.block.stmts.as_slice() else { return None };
    (!matches!(*canon_body(), syn::Expr::Block(_))).then_some(inner)
}

/// The callback of [`walk`]: a region node and its path.
pub type OnNode<'a, 'f> = dyn FnMut(&[(Kind, u32)], NodeRef<'a>) + 'f;

/// Pre-order walk over the region nodes strictly inside a symbol node.
struct Walker<'a, 'f> {
    counters: Vec<BTreeMap<Kind, u32>>,
    path: RegionPath,
    f: &'f mut OnNode<'a, 'f>,
}

impl<'a> Walker<'a, '_> {
    fn enter(&mut self, kind: Kind, node: NodeRef<'a>, inner: impl FnOnce(&mut Self)) {
        let level = self.counters.last_mut().expect("a counter level per open node");
        let n = level.entry(kind).or_insert(0);
        let ordinal = *n;
        *n += 1;
        self.path.push((kind, ordinal));
        (self.f)(&self.path, node);
        self.counters.push(BTreeMap::new());
        inner(self);
        self.counters.pop();
        self.path.pop();
    }
}

impl<'a> Visit<'a> for Walker<'a, '_> {
    fn visit_arm(&mut self, a: &'a syn::Arm) {
        self.enter(Kind::Arm, NodeRef::Arm(a), |w| {
            w.visit_pat(&a.pat);
            if let Some((_, g)) = &a.guard {
                w.visit_expr(g);
            }
            match collapsed(&a.body, || canonical(a, Canon::visit_arm_mut).body) {
                Some(inner) => w.visit_expr(inner),
                None => w.visit_expr(&a.body),
            }
        });
    }

    fn visit_expr_if(&mut self, i: &'a syn::ExprIf) {
        self.visit_expr(&i.cond);
        self.enter(Kind::Branch, NodeRef::Block(&i.then_branch), |w| w.visit_block(&i.then_branch));
        if let Some((_, e)) = &i.else_branch {
            match &**e {
                // `else { .. }`: the block is the branch.
                syn::Expr::Block(b) if b.attrs.is_empty() && b.label.is_none() => {
                    self.enter(Kind::Branch, NodeRef::Block(&b.block), |w| w.visit_block(&b.block));
                }
                // `else if ..`: its branches are siblings of this one.
                other => self.visit_expr(other),
            }
        }
    }

    fn visit_expr_while(&mut self, x: &'a syn::ExprWhile) {
        self.visit_expr(&x.cond);
        self.enter(Kind::Loop, NodeRef::Block(&x.body), |w| w.visit_block(&x.body));
    }

    fn visit_expr_loop(&mut self, x: &'a syn::ExprLoop) {
        self.enter(Kind::Loop, NodeRef::Block(&x.body), |w| w.visit_block(&x.body));
    }

    fn visit_expr_for_loop(&mut self, x: &'a syn::ExprForLoop) {
        self.visit_pat(&x.pat);
        self.visit_expr(&x.expr);
        self.enter(Kind::Loop, NodeRef::Block(&x.body), |w| w.visit_block(&x.body));
    }

    fn visit_expr_closure(&mut self, c: &'a syn::ExprClosure) {
        self.enter(Kind::Closure, NodeRef::Closure(c), |w| {
            match collapsed(&c.body, || canonical(c, Canon::visit_expr_closure_mut).body) {
                Some(inner) => w.visit_expr(inner),
                None => w.visit_expr(&c.body),
            }
        });
    }

    fn visit_expr_async(&mut self, a: &'a syn::ExprAsync) {
        self.enter(Kind::Async, NodeRef::Async(a), |w| w.visit_block(&a.block));
    }

    fn visit_block(&mut self, b: &'a syn::Block) {
        for (i, s) in b.stmts.iter().enumerate() {
            self.enter(Kind::Stmt, NodeRef::Stmt(b, i), |w| w.visit_stmt(s));
        }
    }

    fn visit_field(&mut self, f: &'a syn::Field) {
        self.enter(Kind::Field, NodeRef::Field(f), |w| visit::visit_field(w, f));
    }

    fn visit_variant(&mut self, v: &'a syn::Variant) {
        self.enter(Kind::Variant, NodeRef::Variant(v), |w| visit::visit_variant(w, v));
    }

    fn visit_trait_item(&mut self, t: &'a syn::TraitItem) {
        self.enter(Kind::TraitItem, NodeRef::TraitItem(t), |w| visit::visit_trait_item(w, t));
    }

    fn visit_impl_item(&mut self, i: &'a syn::ImplItem) {
        self.enter(Kind::ImplItem, NodeRef::ImplItem(i), |w| visit::visit_impl_item(w, i));
    }
}

/// Call `f` on every region node strictly inside `node`, in pre-order, with
/// its path. S itself is never offered, even when its own kind is listed
/// (an impl item, a field): the walk starts at S's children.
pub fn walk<'a, 'f>(node: SymNode<'a>, f: &'f mut OnNode<'a, 'f>) {
    let mut w = Walker { counters: vec![BTreeMap::new()], path: Vec::new(), f };
    match node {
        SymNode::File(file) => {
            for it in &file.items {
                w.enter(Kind::Item, NodeRef::Item(it), |w| w.visit_item(it));
            }
        }
        SymNode::Item(i) => visit::visit_item(&mut w, i),
        SymNode::ImplItem(i) => visit::visit_impl_item(&mut w, i),
        SymNode::TraitItem(t) => visit::visit_trait_item(&mut w, t),
        SymNode::Field(fd) => visit::visit_field(&mut w, fd),
        SymNode::Variant(v) => visit::visit_variant(&mut w, v),
    }
}

/// The region of a citation at `line` inside symbol `s`: the deepest region
/// node whose line span contains the line; among equally deep ones, the
/// smallest line span, then the first in source order. `Some(vec![])` when
/// no node qualifies (the region is S); `None` when S's node is not in
/// `file`.
pub fn locate(file: &syn::File, s: &Sym, line: u32) -> Option<RegionPath> {
    let node = symbol_node(file, s)?;
    let mut best: Option<(usize, u32, RegionPath)> = None;
    walk(node, &mut |path, n| {
        let (a, b) = n.lines();
        if a <= line && line <= b {
            let better = match &best {
                None => true,
                Some((d, w, _)) => path.len() > *d || (path.len() == *d && b - a < *w),
            };
            if better {
                best = Some((path.len(), b - a, path.to_vec()));
            }
        }
    });
    Some(best.map(|b| b.2).unwrap_or_default())
}

/// The region hash at `path` inside symbol `s`, or `None` when S's node or
/// the path is not found in `file`.
pub fn hash_at(file: &syn::File, s: &Sym, path: &[(Kind, u32)]) -> Option<[u8; 16]> {
    let node = symbol_node(file, s)?;
    let mut out = None;
    walk(node, &mut |p, n| {
        if out.is_none() && p == path {
            out = Some(n.hash());
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Model;
    use sealmap_model::SourceSet;
    use sealmap_rust::{RustOptions, extract};

    /// A trailing item keeps the fixture's first item from spanning the
    /// whole file (E0's innermost-symbol tie would then pick the module).
    const TAIL: &str = "\npub fn tail() {}\n";

    fn model(src: &str) -> Model {
        let mut s = SourceSet::new();
        s.insert("Cargo.toml", "[package]\nname = \"shop\"").unwrap();
        s.insert("src/lib.rs", src).unwrap();
        Model::from_extraction(&extract(&s, &RustOptions::default()))
    }

    /// The region label at `line` of `src` (with [`TAIL`] appended).
    fn at(src: &str, line: u32) -> String {
        let src = &format!("{src}{TAIL}");
        let m = model(src);
        let (_, s) = m.innermost("src/lib.rs", line).expect("a symbol");
        let file = syn::parse_file(src).unwrap();
        path_label(&locate(&file, s, line).expect("symbol node found"))
    }

    const MATCH: &str = "\
pub fn f(x: u8) -> u8 {
    let y = 1;
    match x {
        0 => {
            g(y)
        }
        1 => 2,
        _ => {
            let z = 3;
            z
        }
    }
}
";

    #[test]
    fn match_arms_and_statements_inside_them() {
        assert_eq!(at(MATCH, 2), "stmt#0");
        assert_eq!(at(MATCH, 3), "stmt#1");
        // `0 => { g(y) }` is `0 => g(y)` canonically: the arm itself.
        assert_eq!(at(MATCH, 5), "stmt#1/arm#0");
        assert_eq!(at(MATCH, 7), "stmt#1/arm#1");
        assert_eq!(at(MATCH, 9), "stmt#1/arm#2/stmt#0");
        assert_eq!(at(MATCH, 1), "(symbol)");
    }

    #[test]
    fn branches_loops_closures_and_async_blocks() {
        let src = "\
pub async fn f(xs: Vec<u8>) {
    if xs.is_empty() {
        a();
    } else if xs.len() > 1 {
        b();
    } else {
        c();
    }
    for x in &xs {
        d(*x);
    }
    while e() {
        f2();
    }
    loop {
        break;
    }
    let g = |v: u8| {
        h(v);
        v
    };
    let k = async {
        m().await
    };
}
";
        assert_eq!(at(src, 3), "stmt#0/branch#0/stmt#0");
        assert_eq!(at(src, 5), "stmt#0/branch#1/stmt#0");
        assert_eq!(at(src, 7), "stmt#0/branch#2/stmt#0");
        assert_eq!(at(src, 10), "stmt#1/loop#0/stmt#0");
        assert_eq!(at(src, 13), "stmt#2/loop#0/stmt#0");
        assert_eq!(at(src, 16), "stmt#3/loop#0/stmt#0");
        assert_eq!(at(src, 19), "stmt#4/closure#0/stmt#0");
        assert_eq!(at(src, 23), "stmt#5/async#0/stmt#0");
    }

    #[test]
    fn fields_variants_trait_and_impl_items_and_top_level_items() {
        let src = "\
pub struct S {
    pub a: u8,
    pub b: u16,
}

pub enum E {
    One,
    Two { x: u8 },
}

use std::fmt;

impl S {
    pub const N: u8 = 1;
}
";
        // Fields and variants are not sealmap symbols: a field line maps to
        // the struct, and narrows to the field.
        assert_eq!(at(src, 3), "field#1");
        assert_eq!(at(src, 7), "variant#0");
        // `Two { x: u8 },` on one line: the innermost node is the field.
        assert_eq!(at(src, 8), "variant#1/field#0");
        // A line on the struct header is inside the struct, outside every field.
        assert_eq!(at(src, 1), "(symbol)");
        // A `use` at the top level is inside only the root module: the top-level item.
        assert_eq!(at(src, 11), "item#2");
        // An impl header line: the top-level impl.
        assert_eq!(at(src, 13), "item#3");
        // A line between items in the root module: no region, the module itself.
        assert_eq!(at(src, 12), "(symbol)");
    }

    #[test]
    fn a_variant_narrows_to_its_fields() {
        let src = "pub enum E {\n    Two {\n        x: u8,\n        y: u8,\n    },\n}\n";
        assert_eq!(at(src, 4), "variant#0/field#1");
        assert_eq!(at(src, 2), "variant#0");
    }

    #[test]
    fn trait_items_inside_a_trait() {
        let src = "pub trait T {\n    type A;\n    fn f(&self) {\n        g();\n    }\n}\n";
        assert_eq!(at(src, 4), "stmt#0");
        // The associated type is a symbol of its own or the trait narrows to it.
        let l2 = at(src, 2);
        assert!(l2 == "(symbol)" || l2 == "trait_item#0", "{l2}");
    }

    #[test]
    fn region_hashes_ignore_formatting_and_comments_like_body_hash() {
        let a = "pub fn f(x: u8) -> u8 {\n    match x {\n        0 => { g(x) }\n        _ => 2,\n    }\n}\n";
        let b =
            "pub fn f(x: u8) -> u8 {\n    match x {\n        // why\n        0 => g(x),\n        _ => 2,\n    }\n}\n";
        let (a, b) = (&format!("{a}{TAIL}"), &format!("{b}{TAIL}"));
        let (ma, mb) = (model(a), model(b));
        let (fa, fb) = (syn::parse_file(a).unwrap(), syn::parse_file(b).unwrap());
        let sa = ma.innermost("src/lib.rs", 3).unwrap().1;
        let sb = mb.innermost("src/lib.rs", 4).unwrap().1;
        let path = locate(&fa, sa, 3).unwrap();
        // `=> { g(x) }` is `=> g(x)` canonically: the arm, not a statement in it.
        assert_eq!(path_label(&path), "stmt#0/arm#0");
        assert_eq!(hash_at(&fa, sa, &path), hash_at(&fb, sb, &path));
        // The symbol's body hash agrees that nothing changed.
        assert_eq!(sa.body, sb.body);
        // A missing path is not found.
        assert_eq!(hash_at(&fa, sa, &[(Kind::Stmt, 7)]), None);
    }
}
