//! Feeding syn nodes to the shared fingerprinter.
//!
//! Rust's token streams already exclude whitespace and ordinary comments.
//! Doc comments survive as `#[doc = "..."]` attributes, so they are removed
//! here, wherever they appear (on items, fields, variants, statements, or
//! inside bodies). Lint-level attributes (`allow`, `warn`, `deny`, `forbid`,
//! `expect`) and tool attributes (`rustfmt::…`, `clippy::…`) are removed
//! too: they change diagnostics, not the program, so adding an `#[allow]`
//! is not a contract change. Every other attribute (`cfg`, `derive`,
//! `must_use`, `inline`, `tokio::main`, ...) is hashed.
//!
//! Trailing commas are layout too (rustfmt adds them when it breaks a list
//! over lines), so a comma is dropped before a closing `}` or `]`, before a
//! `>` (generic lists) and at the end of a `where` clause. Before a closing
//! `)` it is dropped when the group is an argument or parameter list (it
//! follows a name that is not a keyword, another group, or the `>` closing
//! a generic list) or has another top-level comma; otherwise it is kept,
//! because `(T,)` is a one-element tuple and `(T)` is not.
//!
//! Some rewrites rustfmt makes are structural, so they are undone on a copy
//! of the syntax tree before its tokens are fed ([`canonical`]): a closure or
//! match-arm body that is a block holding a single expression is that
//! expression (`|x| { f(x) }` is `|x| f(x)`), every match arm carries its
//! comma, an or-pattern has no leading `|`, a block's final `return`,
//! `break` or `continue` carries its `;`, and the names in a `use` group are
//! sorted. A module's own `use` items are folded in sorted order (see
//! `collect.rs`). A macro invocation whose arguments parse as
//! comma-separated expressions (the only ones rustfmt reformats) has them
//! normalised the same way; any other macro body is normalised at the token
//! level only.

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use sealmap_frontend::fingerprint::{Delim, Fingerprinter};
use sealmap_model::Fingerprint;
use syn::visit_mut::{self, VisitMut};
use syn::{Attribute, Block, Expr, Stmt};

/// Attribute paths that never affect the program.
const IGNORED_ATTRS: &[&str] = &["doc", "allow", "warn", "deny", "forbid", "expect"];
/// Tool namespaces whose attributes never affect the program.
const IGNORED_TOOLS: &[&str] = &["rustfmt", "clippy"];

/// Feed a syn node's tokens.
pub(crate) fn feed(fp: &mut Fingerprinter, node: &impl ToTokens) {
    feed_stream(fp, node.to_token_stream());
}

/// Feed a node after undoing formatter rewrites on a copy of it.
pub(crate) fn feed_canonical<T: Clone + ToTokens>(fp: &mut Fingerprinter, node: &T, visit: fn(&mut Canon, &mut T)) {
    feed(fp, &canonical(node, visit));
}

/// A copy of `node` with formatter rewrites undone (see the module docs).
pub(crate) fn canonical<T: Clone>(node: &T, visit: fn(&mut Canon, &mut T)) -> T {
    let mut copy = node.clone();
    visit(&mut Canon, &mut copy);
    copy
}

/// The syntax-tree half of the normalisation.
pub(crate) struct Canon;

/// `{ e }` (no attributes, label or statements) → `e`.
fn unblock(e: &Expr) -> Option<Expr> {
    let Expr::Block(b) = e else { return None };
    if !b.attrs.is_empty() || b.label.is_some() || b.block.stmts.len() != 1 {
        return None;
    }
    match &b.block.stmts[0] {
        Stmt::Expr(inner, None) => Some(inner.clone()),
        // `{ return x; }`: the `;` after a diverging expression is layout
        // (the block rule below adds it), so the block is still `return x`.
        Stmt::Expr(inner @ (Expr::Return(_) | Expr::Break(_) | Expr::Continue(_)), Some(_)) => Some(inner.clone()),
        _ => None,
    }
}

impl VisitMut for Canon {
    fn visit_expr_closure_mut(&mut self, c: &mut syn::ExprClosure) {
        visit_mut::visit_expr_closure_mut(self, c);
        // A closure with a return type needs its block.
        if matches!(c.output, syn::ReturnType::Default) {
            if let Some(e) = unblock(&c.body) {
                *c.body = e;
            }
        }
    }

    fn visit_arm_mut(&mut self, a: &mut syn::Arm) {
        visit_mut::visit_arm_mut(self, a);
        if let Some(e) = unblock(&a.body) {
            *a.body = e;
        }
        a.comma = Some(Default::default());
    }

    fn visit_pat_or_mut(&mut self, p: &mut syn::PatOr) {
        visit_mut::visit_pat_or_mut(self, p);
        p.leading_vert = None;
    }

    fn visit_block_mut(&mut self, b: &mut Block) {
        visit_mut::visit_block_mut(self, b);
        if let Some(Stmt::Expr(Expr::Return(_) | Expr::Break(_) | Expr::Continue(_), semi @ None)) = b.stmts.last_mut()
        {
            *semi = Some(Default::default());
        }
    }

    fn visit_macro_mut(&mut self, m: &mut syn::Macro) {
        let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        if let Ok(args) = m.parse_body_with(parser) {
            let mut out = TokenStream::new();
            for (i, mut e) in args.into_iter().enumerate() {
                self.visit_expr_mut(&mut e);
                if i > 0 {
                    <syn::Token![,]>::default().to_tokens(&mut out);
                }
                e.to_tokens(&mut out);
            }
            m.tokens = out;
        }
    }

    fn visit_use_group_mut(&mut self, g: &mut syn::UseGroup) {
        visit_mut::visit_use_group_mut(self, g);
        let mut items: Vec<syn::UseTree> = std::mem::take(&mut g.items).into_iter().collect();
        items.sort_by_cached_key(|t| t.to_token_stream().to_string());
        g.items = items.into_iter().collect();
    }
}

/// Feed every attribute that matters.
pub(crate) fn attrs(fp: &mut Fingerprinter, attrs: &[Attribute]) {
    fp.section("attrs");
    for a in attrs {
        if !ignored_path(a.path()) {
            feed(fp, a);
        }
    }
}

/// The fingerprints of a module whose file did not parse: its name in the
/// contract, its raw text as one literal in the body, so any edit shows.
pub(crate) fn unparsable(name: &str, text: &str) -> (Fingerprint, Fingerprint) {
    let mut sig = Fingerprinter::sig();
    sig.section("unparsable");
    sig.ident(name);
    let mut body = Fingerprinter::body();
    body.section("unparsable");
    body.literal(text);
    (sig.finish(), body.finish())
}

fn ignored_path(p: &syn::Path) -> bool {
    match p.segments.first() {
        Some(first) if p.segments.len() == 1 => IGNORED_ATTRS.iter().any(|n| first.ident == n),
        Some(first) => IGNORED_TOOLS.iter().any(|n| first.ident == n),
        None => false,
    }
}

/// Keywords that can stand directly before a parenthesised tuple.
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "box", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self",
    "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "yield",
];

fn feed_stream(fp: &mut Fingerprinter, ts: TokenStream) {
    feed_trees(fp, ts, Delimiter::None, false);
}

/// Does a parenthesised group after `prev` (and `prev2` before it) hold an
/// argument or parameter list rather than a tuple?
fn is_argument_list(prev2: Option<&TokenTree>, prev: Option<&TokenTree>) -> bool {
    match prev {
        Some(TokenTree::Ident(id)) => !KEYWORDS.iter().any(|k| id == k),
        Some(TokenTree::Group(g)) => g.delimiter() != Delimiter::Brace,
        // `f::<T>(..)`, `fn f<T>(..)`; not `->` or `=>`.
        Some(t) if is_punct(t, '>') => !prev2.is_some_and(|p| is_punct(p, '-') || is_punct(p, '=')),
        _ => false,
    }
}

fn feed_trees(fp: &mut Fingerprinter, ts: TokenStream, delim: Delimiter, arguments: bool) {
    let trees: Vec<TokenTree> = ts.into_iter().collect();
    let commas = trees.iter().filter(|t| is_punct(t, ',')).count();
    let mut in_where = false;
    let mut i = 0;
    while i < trees.len() {
        if let Some(n) = ignored_attr_len(&trees[i..]) {
            i += n;
            continue;
        }
        match &trees[i] {
            TokenTree::Ident(id) => {
                let text = id.to_string();
                in_where |= text == "where";
                fp.ident(&text);
            }
            TokenTree::Punct(p)
                if p.as_char() == ',' && is_trailing(trees.get(i + 1), delim, commas, in_where, arguments) => {}
            TokenTree::Punct(p) => {
                in_where &= p.as_char() != ';';
                fp.punct(p.as_char());
            }
            TokenTree::Literal(l) => fp.literal(&l.to_string()),
            TokenTree::Group(g) => {
                in_where &= g.delimiter() != Delimiter::Brace;
                let d = match g.delimiter() {
                    Delimiter::Parenthesis => Delim::Paren,
                    Delimiter::Bracket => Delim::Bracket,
                    Delimiter::Brace => Delim::Brace,
                    Delimiter::None => Delim::None,
                };
                let args = g.delimiter() == Delimiter::Parenthesis
                    && is_argument_list(
                        i.checked_sub(2).and_then(|j| trees.get(j)),
                        i.checked_sub(1).and_then(|j| trees.get(j)),
                    );
                fp.open(d);
                feed_trees(fp, g.stream(), g.delimiter(), args);
                fp.close(d);
            }
        }
        i += 1;
    }
}

fn is_punct(t: &TokenTree, c: char) -> bool {
    matches!(t, TokenTree::Punct(p) if p.as_char() == c)
}

/// Is a comma followed by `next` (in a group delimited by `delim` holding
/// `commas` top-level commas) a trailing comma that layout alone decides?
fn is_trailing(next: Option<&TokenTree>, delim: Delimiter, commas: usize, in_where: bool, arguments: bool) -> bool {
    match next {
        None => delim != Delimiter::Parenthesis || arguments || commas > 1,
        Some(t) if is_punct(t, '>') => true,
        Some(TokenTree::Group(g)) if in_where => g.delimiter() == Delimiter::Brace,
        Some(t) if in_where => is_punct(t, ';'),
        Some(_) => false,
    }
}

/// If `trees` starts with an ignored attribute (`#[doc = ..]`,
/// `#![allow(..)]`, `#[clippy::x]`), the number of trees it spans.
fn ignored_attr_len(trees: &[TokenTree]) -> Option<usize> {
    let TokenTree::Punct(hash) = trees.first()? else { return None };
    if hash.as_char() != '#' {
        return None;
    }
    let inner = matches!(trees.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '!');
    let at = if inner { 2 } else { 1 };
    let TokenTree::Group(g) = trees.get(at)? else { return None };
    if g.delimiter() != Delimiter::Bracket {
        return None;
    }
    let mut inside = g.stream().into_iter();
    let TokenTree::Ident(first) = inside.next()? else { return None };
    let path_continues = matches!(inside.next(), Some(TokenTree::Punct(p)) if p.as_char() == ':');
    let ignored = if path_continues {
        IGNORED_TOOLS.iter().any(|n| first == n)
    } else {
        IGNORED_ATTRS.iter().any(|n| first == n)
    };
    ignored.then_some(at + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(src: &str) -> Fingerprint {
        let block: syn::Block = syn::parse_str(src).unwrap();
        let mut fp = Fingerprinter::body();
        feed(&mut fp, &block);
        fp.finish()
    }

    #[test]
    fn strips_docs_and_lints_but_keeps_other_attributes() {
        let plain = body("{ fn f() {} let x = 1; }");
        assert_eq!(plain, body("{ /// doc\n fn f() {} #[allow(unused)] let x = 1; }"));
        assert_eq!(plain, body("{ #[doc = \"d\"] fn f() {} #[clippy::skip] let x = 1; }"));
        assert_eq!(plain, body("{ fn f() { #![allow(dead_code)] } let x = 1; }"));
        assert_ne!(plain, body("{ #[inline] fn f() {} let x = 1; }"));
        assert_ne!(plain, body("{ #[cfg(unix)] fn f() {} let x = 1; }"));
    }

    fn canon_body(src: &str) -> Fingerprint {
        let block: syn::Block = syn::parse_str(src).unwrap();
        let mut fp = Fingerprinter::body();
        feed_canonical(&mut fp, &block, Canon::visit_block_mut);
        fp.finish()
    }

    #[test]
    fn formatter_rewrites_are_undone() {
        assert_eq!(canon_body("{ xs.map(|x| f(x)) }"), canon_body("{ xs.map(|x| { f(x) }) }"));
        assert_eq!(
            canon_body("{ match a { A => f(), B => { g() } } }"),
            canon_body("{ match a { A => { f() } B => g(), } }")
        );
        assert_eq!(canon_body("{ match a { A | B => 1 } }"), canon_body("{ match a { | A | B => 1 } }"));
        assert_eq!(canon_body("{ if a { return 1 } }"), canon_body("{ if a { return 1; } }"));
        assert_eq!(
            canon_body("{ match a { A => return 1, B => 2 } }"),
            canon_body("{ match a { A => { return 1 } B => 2 } }")
        );
        assert_eq!(canon_body("{ use a::{c, b}; }"), canon_body("{ use a::{b, c}; }"));
        assert_eq!(canon_body("{ assert!(xs.map(|x| f(x))); }"), canon_body("{ assert!(xs.map(|x| { f(x) }),); }"));
        // Not formatting: a closure with a return type keeps its block, and
        // a block with statements is not an expression.
        assert_ne!(canon_body("{ xs.map(|x| { g(); f(x) }) }"), canon_body("{ xs.map(|x| f(x)) }"));
        assert_ne!(canon_body("{ xs.map(|x| { f(x) }.y) }"), canon_body("{ xs.map(|x| f(x).y) }"));
    }

    #[test]
    fn trailing_commas_are_layout_except_in_one_tuples() {
        assert_eq!(body("{ f(a, b); }"), body("{ f(a, b,); }"));
        assert_eq!(body("{ let v = [1, 2]; }"), body("{ let v = [1, 2,]; }"));
        assert_eq!(body("{ match x { A => 1, B => 2 } }"), body("{ match x { A => 1, B => 2, } }"));
        assert_eq!(body("{ let s = S { a }; }"), body("{ let s = S { a, }; }"));
        assert_eq!(body("{ fn f<A, B>() {} }"), body("{ fn f<A, B,>() {} }"));
        assert_eq!(body("{ fn f<T>() where T: A, T: B {} }"), body("{ fn f<T>() where T: A, T: B, {} }"));
        assert_ne!(body("{ let t: (u8,) = x; }"), body("{ let t: (u8) = x; }"));
        assert_ne!(body("{ return (x,); }"), body("{ return (x); }"));
        assert_ne!(body("{ fn f() -> (u8,) {} }"), body("{ fn f() -> (u8) {} }"));
        assert_eq!(body("{ f(x); }"), body("{ f(x,); }"));
        assert_eq!(body("{ Self(x); g::<T>(x); h()(x); }"), body("{ Self(x,); g::<T>(x,); h()(x,); }"));
        assert_eq!(body("{ fn f<T>(x: T) {} }"), body("{ fn f<T>(x: T,) {} }"));
        assert_ne!(body("{ f(a, {b}); }"), body("{ f(a {b}); }"));
    }
}
