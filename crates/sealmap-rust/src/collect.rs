//! Pass 1: parse one file with `syn` and collect unresolved raw data.

use std::collections::BTreeMap;

use proc_macro2::Span as PmSpan;
use sealmap_extract::fingerprint::Fingerprinter;
use sealmap_model::{CallKind, Fingerprint, MemberKind, SourcePath, Span, SymbolKind, Visibility};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{
    Attribute, Block, Expr, Fields, FnArg, GenericParam, Generics, ImplItem, Item, Pat, Stmt, TraitItem, Type,
    TypeParamBound, UseTree,
};

use crate::RustOptions;
use crate::fingerprint::{self, Canon};
use crate::layout::FileRole;
use crate::raw::*;
use crate::tidy::tokens;
use sealmap_extract::labels::{
    ARG_LITERAL_MAX, DOC_SUMMARY_MAX, LABEL_MAX, SIGNATURE_MAX as SIG_MAX, call_label, clip, condition_label, squeeze,
};
use sealmap_extract::raw::{last_call_mut, place_deferred, push_arms};

/// Expression nesting beyond which flow extraction stops descending. Real
/// code stays far below it; generated code (long `a + b + ...` or builder
/// chains, which syn nests one level per operator) can exceed any fixed
/// stack. Calls deeper than this are omitted from the flow.
pub(crate) const MAX_EXPR_DEPTH: u32 = 1024;

/// Parse `text` and collect everything pass 2 needs.
pub(crate) fn collect_file(path: &SourcePath, role: &FileRole, text: &str, opts: &RustOptions) -> RawFile {
    let mut raw = RawFile {
        path: path.clone(),
        role: role.clone(),
        text_lines: text.lines().count() as u32,
        hash: sealmap_model::ContentHash::of_text(text),
        modules: Vec::new(),
        items: Vec::new(),
        impls: Vec::new(),
        error: None,
    };
    let file = match syn::parse_file(text) {
        Ok(f) => f,
        Err(e) => {
            let msg = format!("{}:{}: {e}", e.span().start().line, e.span().start().column + 1);
            return failed_file(path, role, text, msg);
        }
    };
    let mut c = Collector { opts, raw: &mut raw };
    let span = Span::new(1, 1, c.raw.text_lines.max(1), 1);
    c.module(role.module.clone(), &file.attrs, None, &file.items, span, Visibility::Public);
    raw
}

/// The stand-in for a file that could not be collected (parse error or an
/// internal panic): just its module symbol, tagged `parse_error`, so the 1:1
/// contract still holds and the problem surfaces as a diagnostic.
pub(crate) fn failed_file(path: &SourcePath, role: &FileRole, text: &str, error: String) -> RawFile {
    let text_lines = text.lines().count() as u32;
    let (sig_hash, body_hash) = fingerprint::unparsable(role.module.last().map_or("", String::as_str), text);
    RawFile {
        path: path.clone(),
        role: role.clone(),
        text_lines,
        hash: sealmap_model::ContentHash::of_text(text),
        modules: vec![RawModule {
            path: role.module.clone(),
            span: Span::new(1, 1, text_lines.max(1), 1),
            doc: None,
            vis: Visibility::Public,
            uses: Vec::new(),
            tags: vec!["parse_error".into()],
            sig_hash,
            body_hash,
        }],
        items: Vec::new(),
        impls: Vec::new(),
        error: Some(error),
    }
}

struct Collector<'a> {
    opts: &'a RustOptions,
    raw: &'a mut RawFile,
}

impl Collector<'_> {
    /// Collect a module and everything in it. `syn_vis` is the declared
    /// visibility of an inline `mod` (`None` for a file's root module).
    /// Returns the module's (`sig_hash`, `body_hash`).
    fn module(
        &mut self,
        path: Segs,
        attrs: &[Attribute],
        syn_vis: Option<&syn::Visibility>,
        items: &[Item],
        span: Span,
        vis: Visibility,
    ) -> (Fingerprint, Fingerprint) {
        let mut sig = Fingerprinter::sig();
        sig.section("mod");
        fingerprint::attrs(&mut sig, attrs);
        sig.section("vis");
        if let Some(v) = syn_vis {
            fingerprint::feed(&mut sig, v);
        }
        sig.section("name");
        sig.ident(path.last().map_or("", String::as_str));
        let mut uses = Vec::new();
        for item in items {
            if let Item::Use(u) = item {
                flatten_use(&u.tree, &mut Vec::new(), &mut uses);
            }
        }
        self.raw.modules.push(RawModule {
            path: path.clone(),
            span,
            doc: doc_of(attrs),
            vis,
            uses,
            tags: tags_of(attrs),
            sig_hash: sig.finish(),
            body_hash: Fingerprint::default(),
        });
        let at = self.raw.modules.len() - 1;
        let mut body = Fingerprinter::body();
        body.section("mod");
        // rustfmt reorders `use`, `extern crate` and `mod x;` declarations,
        // and their order carries no meaning, so they are folded sorted.
        let mut declarations = Vec::new();
        for item in items {
            match item {
                Item::Use(_) | Item::ExternCrate(_) | Item::Mod(syn::ItemMod { content: None, .. }) => {
                    if !self.skip(item_attrs(item)) {
                        let mut d = Fingerprinter::body();
                        fingerprint::feed_canonical(&mut d, item, Canon::visit_item_mut);
                        declarations.push(d.finish());
                    }
                }
                _ => self.item(&path, item, &mut body),
            }
        }
        declarations.sort();
        body.section("declarations");
        for d in declarations {
            body.fingerprint(d);
        }
        let hashes = (sig.finish(), body.finish());
        self.raw.modules[at].body_hash = hashes.1;
        hashes
    }

    /// Add the `use` declarations inside a function body to `module`'s
    /// imports. Their scope is really the block; widening it to the module
    /// is the cheap approximation, and far better than not seeing them:
    /// `use crate::events::Metrics;` inside a handler would otherwise leave
    /// `Metrics` unresolved, and `use sha2::Sha256;` would leave `Sha256`
    /// looking like nothing in particular rather than a dependency type.
    fn local_uses(&mut self, module: &Segs, block: &Block) {
        struct Uses(Vec<RawUse>);
        impl<'ast> syn::visit::Visit<'ast> for Uses {
            fn visit_item_use(&mut self, u: &'ast syn::ItemUse) {
                flatten_use(&u.tree, &mut Vec::new(), &mut self.0);
            }
        }
        let mut found = Uses(Vec::new());
        syn::visit::Visit::visit_block(&mut found, block);
        if found.0.is_empty() {
            return;
        }
        if let Some(m) = self.raw.modules.iter_mut().rev().find(|m| m.path == *module) {
            m.uses.extend(found.0);
        }
    }

    /// Record a collected item and fold it into its module's body.
    fn push_item(&mut self, it: RawItem, fold: &mut Fingerprinter) {
        fold_member(fold, it.kind.keyword(), &it.name, it.sig_hash, it.body_hash);
        self.raw.items.push(it);
    }

    fn skip(&self, attrs: &[Attribute]) -> bool {
        !self.opts.include_tests && attrs.iter().any(is_test_attr)
    }

    /// Collect one item of `module`, folding what it contributes into the
    /// module's body fingerprint `fold`. Items left out of the model (tests
    /// when they are excluded) contribute nothing.
    fn item(&mut self, module: &Segs, item: &Item, fold: &mut Fingerprinter) {
        let base = |name: String, kind, vis: &syn::Visibility, attrs: &[Attribute], span: PmSpan| RawItem {
            module: module.clone(),
            name,
            kind,
            vis: vis_of(vis),
            span: span_of(span),
            signature: None,
            doc: doc_of(attrs),
            generics: Vec::new(),
            type_params: Vec::new(),
            tags: tags_of(attrs),
            members: Vec::new(),
            sig_refs: Vec::new(),
            supertraits: Vec::new(),
            methods: Vec::new(),
            flow: Vec::new(),
            sig_hash: Fingerprint::default(),
            body_hash: Fingerprint::default(),
        };
        match item {
            Item::Mod(m) => {
                if self.skip(&m.attrs) {
                    return;
                }
                // `mod x;` (no content) is folded by `module` with the other
                // declarations; the file it names is a module of its own.
                if let Some((_, items)) = &m.content {
                    let mut path = module.clone();
                    path.push(m.ident.to_string());
                    let (sig, body) =
                        self.module(path, &m.attrs, Some(&m.vis), items, span_of(m.span()), vis_of(&m.vis));
                    fold_member(fold, "mod", &m.ident.to_string(), sig, body);
                }
            }
            Item::Struct(s) => {
                if self.skip(&s.attrs) {
                    return;
                }
                let mut it = base(s.ident.to_string(), SymbolKind::Struct, &s.vis, &s.attrs, s.span());
                it.generics = generics_of(&s.generics);
                it.type_params = param_names(&s.generics);
                it.members = fields_of(&s.fields);
                (it.sig_hash, it.body_hash) = data_type(item, &s.generics, Shape::Fields(&s.fields));
                self.push_item(it, fold);
            }
            Item::Union(u) => {
                let mut it = base(u.ident.to_string(), SymbolKind::Union, &u.vis, &u.attrs, u.span());
                it.generics = generics_of(&u.generics);
                it.type_params = param_names(&u.generics);
                it.members = fields_of(&Fields::Named(u.fields.clone()));
                (it.sig_hash, it.body_hash) = data_type(item, &u.generics, Shape::Named(&u.fields));
                self.push_item(it, fold);
            }
            Item::Enum(e) => {
                if self.skip(&e.attrs) {
                    return;
                }
                let mut it = base(e.ident.to_string(), SymbolKind::Enum, &e.vis, &e.attrs, e.span());
                it.generics = generics_of(&e.generics);
                it.type_params = param_names(&e.generics);
                for v in &e.variants {
                    let mut refs = Vec::new();
                    for f in v.fields.iter() {
                        type_refs(&f.ty, &mut refs);
                    }
                    let ty = match &v.fields {
                        Fields::Unit => None,
                        Fields::Unnamed(u) => Some(squeeze(&tokens(u))),
                        Fields::Named(n) => Some(clip(&squeeze(&tokens(n)), LABEL_MAX)),
                    };
                    it.members.push(RawMember {
                        name: v.ident.to_string(),
                        kind: MemberKind::Variant,
                        ty,
                        vis: Visibility::Public,
                        span: span_of(v.span()),
                        refs,
                        type_params: Vec::new(),
                    });
                }
                (it.sig_hash, it.body_hash) = data_type(item, &e.generics, Shape::Variants(&e.variants));
                self.push_item(it, fold);
            }
            Item::Trait(t) => {
                if self.skip(&t.attrs) {
                    return;
                }
                let mut it = base(t.ident.to_string(), SymbolKind::Trait, &t.vis, &t.attrs, t.span());
                it.generics = generics_of(&t.generics);
                it.type_params = param_names(&t.generics);
                for b in &t.supertraits {
                    if let TypeParamBound::Trait(tb) = b {
                        it.supertraits.push(path_segs(&tb.path));
                    }
                }
                if t.unsafety.is_some() {
                    it.tags.push("unsafe".into());
                }
                for ti in &t.items {
                    match ti {
                        TraitItem::Fn(f) => {
                            let sig = clip(&tokens(&f.sig), SIG_MAX);
                            let mut refs = Vec::new();
                            sig_refs(&f.sig, &mut refs);
                            match &f.default {
                                Some(block) => {
                                    let params = params_of(&f.sig);
                                    let mut sig_fp = Fingerprinter::sig();
                                    sig_fp.section("trait");
                                    sig_fp.ident(&t.ident.to_string());
                                    fingerprint::feed(&mut sig_fp, &t.generics);
                                    fingerprint::feed(&mut sig_fp, &t.generics.where_clause);
                                    let (sig_hash, body_hash) = callable(sig_fp, &f.attrs, None, &f.sig, block);
                                    self.local_uses(module, block);
                                    it.methods.push(RawFn {
                                        sig_hash,
                                        body_hash,
                                        name: f.sig.ident.to_string(),
                                        vis: Visibility::Public,
                                        span: span_of(f.span()),
                                        signature: sig,
                                        doc: doc_of(&f.attrs),
                                        generics: generics_of(&f.sig.generics),
                                        type_params: param_names(&f.sig.generics),
                                        tags: fn_tags(&f.sig, &f.attrs),
                                        sig_refs: refs,
                                        flow: FlowWalker::new(params).block(block),
                                    });
                                }
                                None => it.members.push(RawMember {
                                    name: f.sig.ident.to_string(),
                                    kind: MemberKind::RequiredMethod,
                                    ty: Some(sig),
                                    vis: Visibility::Public,
                                    span: span_of(f.span()),
                                    refs,
                                    type_params: param_names(&f.sig.generics),
                                }),
                            }
                        }
                        TraitItem::Type(ty) => it.members.push(RawMember {
                            name: ty.ident.to_string(),
                            kind: MemberKind::AssocType,
                            ty: (!ty.bounds.is_empty()).then(|| squeeze(&tokens(&ty.bounds))),
                            vis: Visibility::Public,
                            span: span_of(ty.span()),
                            refs: Vec::new(),
                            type_params: Vec::new(),
                        }),
                        TraitItem::Const(k) => {
                            let mut refs = Vec::new();
                            type_refs(&k.ty, &mut refs);
                            it.members.push(RawMember {
                                name: k.ident.to_string(),
                                kind: MemberKind::AssocConst,
                                ty: Some(tokens(&k.ty)),
                                vis: Visibility::Public,
                                span: span_of(k.span()),
                                refs,
                                type_params: Vec::new(),
                            });
                        }
                        _ => {}
                    }
                }
                (it.sig_hash, it.body_hash) = trait_hashes(t);
                self.push_item(it, fold);
            }
            Item::Type(t) => {
                let mut it = base(t.ident.to_string(), SymbolKind::TypeAlias, &t.vis, &t.attrs, t.span());
                it.generics = generics_of(&t.generics);
                it.type_params = param_names(&t.generics);
                it.signature = Some(clip(&format!("type {} = {}", t.ident, tokens(&t.ty)), SIG_MAX));
                type_refs(&t.ty, &mut it.sig_refs);
                let mut sig = Fingerprinter::sig();
                sig.section("type");
                fingerprint::feed(&mut sig, item);
                let mut body = Fingerprinter::body();
                body.section("type");
                fingerprint::feed(&mut body, &t.generics);
                fingerprint::feed(&mut body, &t.generics.where_clause);
                fingerprint::feed(&mut body, &t.ty);
                (it.sig_hash, it.body_hash) = (sig.finish(), body.finish());
                self.push_item(it, fold);
            }
            Item::Fn(f) => {
                if self.skip(&f.attrs) {
                    return;
                }
                let mut it = base(f.sig.ident.to_string(), SymbolKind::Function, &f.vis, &f.attrs, f.span());
                it.generics = generics_of(&f.sig.generics);
                it.type_params = param_names(&f.sig.generics);
                it.signature = Some(clip(&format!("{}{}", vis_prefix(&f.vis), tokens(&f.sig)), SIG_MAX));
                it.tags.extend(fn_tags(&f.sig, &[]));
                sig_refs(&f.sig, &mut it.sig_refs);
                it.flow = FlowWalker::new(params_of(&f.sig)).block(&f.block);
                let mut sig = Fingerprinter::sig();
                sig.section("fn");
                (it.sig_hash, it.body_hash) = callable(sig, &f.attrs, Some(&f.vis), &f.sig, &f.block);
                self.local_uses(module, &f.block);
                self.push_item(it, fold);
            }
            Item::Const(k) => {
                let mut it = base(k.ident.to_string(), SymbolKind::Const, &k.vis, &k.attrs, k.span());
                it.signature = Some(clip(&format!("const {}: {}", k.ident, tokens(&k.ty)), SIG_MAX));
                type_refs(&k.ty, &mut it.sig_refs);
                (it.sig_hash, it.body_hash) = value("const", &k.attrs, &k.vis, None, &k.ident, &k.ty, &k.expr);
                self.push_item(it, fold);
            }
            Item::Static(s) => {
                let mut it = base(s.ident.to_string(), SymbolKind::Static, &s.vis, &s.attrs, s.span());
                it.signature = Some(clip(&format!("static {}: {}", s.ident, tokens(&s.ty)), SIG_MAX));
                type_refs(&s.ty, &mut it.sig_refs);
                (it.sig_hash, it.body_hash) =
                    value("static", &s.attrs, &s.vis, Some(&s.mutability), &s.ident, &s.ty, &s.expr);
                self.push_item(it, fold);
            }
            Item::Macro(m) => {
                if let Some(ident) = &m.ident {
                    let vis = if m.attrs.iter().any(|a| a.path().is_ident("macro_export")) {
                        syn::Visibility::Public(Default::default())
                    } else {
                        syn::Visibility::Inherited
                    };
                    let mut it = base(ident.to_string(), SymbolKind::Macro, &vis, &m.attrs, m.span());
                    let mut sig = Fingerprinter::sig();
                    sig.section("macro");
                    fingerprint::attrs(&mut sig, &m.attrs);
                    sig.ident(&ident.to_string());
                    let mut body = Fingerprinter::body();
                    body.section("macro");
                    fingerprint::feed(&mut body, &m.mac.tokens);
                    (it.sig_hash, it.body_hash) = (sig.finish(), body.finish());
                    self.push_item(it, fold);
                } else {
                    // A macro invocation at item level (`thread_local! { .. }`).
                    fingerprint::feed(fold, item);
                }
            }
            Item::Impl(i) => {
                if self.skip(&i.attrs) {
                    return;
                }
                let self_ty = first_path(&i.self_ty);
                let trait_ = i.trait_.as_ref().map(|(_, p, _)| (path_segs(p), trait_text(p)));
                let is_trait_impl = trait_.is_some();
                let mut methods = Vec::new();
                fold.section("impl");
                impl_header(fold, i);
                for ii in &i.items {
                    let ImplItem::Fn(f) = ii else {
                        // Associated types and constants belong to the impl.
                        fingerprint::feed(fold, ii);
                        continue;
                    };
                    if self.skip(&f.attrs) {
                        continue;
                    }
                    let mut sig_fp = Fingerprinter::sig();
                    impl_header(&mut sig_fp, i);
                    sig_fp.section("fn");
                    let (sig_hash, body_hash) = callable(sig_fp, &f.attrs, Some(&f.vis), &f.sig, &f.block);
                    fold_member(fold, "fn", &f.sig.ident.to_string(), sig_hash, body_hash);
                    self.local_uses(module, &f.block);
                    let mut refs = Vec::new();
                    sig_refs(&f.sig, &mut refs);
                    let vis = if is_trait_impl { Visibility::Public } else { vis_of(&f.vis) };
                    methods.push(RawFn {
                        name: f.sig.ident.to_string(),
                        span: span_of(f.span()),
                        signature: clip(&format!("{}{}", vis_prefix(&f.vis), tokens(&f.sig)), SIG_MAX),
                        doc: doc_of(&f.attrs),
                        generics: generics_of(&f.sig.generics),
                        type_params: param_names(&f.sig.generics),
                        tags: fn_tags(&f.sig, &f.attrs),
                        sig_refs: refs,
                        flow: FlowWalker::new(params_of(&f.sig)).block(&f.block),
                        vis,
                        sig_hash,
                        body_hash,
                    });
                }
                self.raw.impls.push(RawImpl {
                    module: module.clone(),
                    self_ty,
                    trait_,
                    type_params: param_names(&i.generics),
                    methods,
                });
            }
            // `use`, `extern crate`, foreign blocks, ...: part of the module.
            other => fingerprint::feed(fold, other),
        }
    }
}

/// The trait of an impl as written, which becomes part of its methods' ids
/// (`Db#[`From<String>`]from().`). Spacing comes from [`tokens`], and a
/// trailing comma in a generic or array list (`Handler<Msg,>`, which
/// rustfmt writes when it breaks the list) is dropped, so formatting never
/// changes an id. A comma before `)` is kept: `(T,)` is a tuple.
fn trait_text(p: &syn::Path) -> String {
    let mut out = tokens(p);
    for (from, to) in [(",>", ">"), (", >", ">"), (",]", "]"), (", ]", "]")] {
        while out.contains(from) {
            out = out.replace(from, to);
        }
    }
    out
}

// ----------------------------------------------------------- fingerprints

/// The outer attributes of the item kinds `module` folds as declarations.
fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Use(u) => &u.attrs,
        Item::ExternCrate(e) => &e.attrs,
        Item::Mod(m) => &m.attrs,
        _ => &[],
    }
}

/// Fold a member's identity and fingerprints into its container's body.
fn fold_member(fold: &mut Fingerprinter, keyword: &str, name: &str, sig: Fingerprint, body: Fingerprint) {
    fold.section(keyword);
    fold.ident(name);
    fold.fingerprint(sig);
    fold.fingerprint(body);
}

/// A function or method: `sig` already holds its context (the impl or trait
/// header, if any); the attributes, visibility and signature are added. The
/// body is the block alone, so the name never reaches it.
fn callable(
    mut sig: Fingerprinter,
    attrs: &[Attribute],
    vis: Option<&syn::Visibility>,
    signature: &syn::Signature,
    block: &Block,
) -> (Fingerprint, Fingerprint) {
    fingerprint::attrs(&mut sig, attrs);
    sig.section("vis");
    if let Some(v) = vis {
        fingerprint::feed(&mut sig, v);
    }
    sig.section("sig");
    fingerprint::feed(&mut sig, signature);
    let mut body = Fingerprinter::body();
    body.section("block");
    fingerprint::feed_canonical(&mut body, block, Canon::visit_block_mut);
    (sig.finish(), body.finish())
}

/// The header of an impl block (attributes, `unsafe`, generics, trait, self
/// type, `where`), part of each of its methods' contract.
fn impl_header(fp: &mut Fingerprinter, i: &syn::ItemImpl) {
    fp.section("impl");
    fingerprint::attrs(fp, &i.attrs);
    fingerprint::feed(fp, &i.defaultness);
    fingerprint::feed(fp, &i.unsafety);
    fingerprint::feed(fp, &i.generics);
    fp.section("trait");
    if let Some((bang, path, _)) = &i.trait_ {
        fingerprint::feed(fp, bang);
        fingerprint::feed(fp, path);
    }
    fp.section("self");
    fingerprint::feed(fp, &i.self_ty);
    fingerprint::feed(fp, &i.generics.where_clause);
}

/// The fields or variants of a data type, the body of its fingerprint.
enum Shape<'a> {
    Fields(&'a Fields),
    Named(&'a syn::FieldsNamed),
    Variants(&'a Punctuated<syn::Variant, syn::Token![,]>),
}

/// A struct, enum or union: the whole declaration is the contract; the
/// shape (generics and fields or variants, without the name) is the body.
/// Fields and variants are fed one by one, each with its own separator, so
/// the trailing comma of a field list is never part of the stream.
fn data_type(item: &Item, generics: &Generics, shape: Shape<'_>) -> (Fingerprint, Fingerprint) {
    let mut sig = Fingerprinter::sig();
    sig.section("data");
    fingerprint::feed(&mut sig, item);
    let mut body = Fingerprinter::body();
    body.section("data");
    fingerprint::feed(&mut body, generics);
    fingerprint::feed(&mut body, &generics.where_clause);
    let fields = |fp: &mut Fingerprinter, label: &str, fields: &mut dyn Iterator<Item = &syn::Field>| {
        fp.section(label);
        for f in fields {
            fp.section("field");
            fingerprint::feed(fp, f);
        }
    };
    match shape {
        Shape::Fields(Fields::Named(n)) | Shape::Named(n) => fields(&mut body, "named", &mut n.named.iter()),
        Shape::Fields(Fields::Unnamed(u)) => fields(&mut body, "unnamed", &mut u.unnamed.iter()),
        Shape::Fields(Fields::Unit) => body.section("unit"),
        Shape::Variants(vs) => {
            for v in vs {
                body.section("variant");
                fingerprint::attrs(&mut body, &v.attrs);
                body.ident(&v.ident.to_string());
                match &v.fields {
                    Fields::Named(n) => fields(&mut body, "named", &mut n.named.iter()),
                    Fields::Unnamed(u) => fields(&mut body, "unnamed", &mut u.unnamed.iter()),
                    Fields::Unit => body.section("unit"),
                }
                if let Some((_, discriminant)) = &v.discriminant {
                    body.section("discriminant");
                    fingerprint::feed_canonical(&mut body, discriminant, Canon::visit_expr_mut);
                }
            }
        }
    }
    (sig.finish(), body.finish())
}

/// A trait: the header plus every member's signature is the contract; every
/// member in full (default bodies included) is the body.
fn trait_hashes(t: &syn::ItemTrait) -> (Fingerprint, Fingerprint) {
    let mut sig = Fingerprinter::sig();
    sig.section("trait");
    fingerprint::attrs(&mut sig, &t.attrs);
    fingerprint::feed(&mut sig, &t.vis);
    fingerprint::feed(&mut sig, &t.unsafety);
    fingerprint::feed(&mut sig, &t.auto_token);
    sig.ident(&t.ident.to_string());
    fingerprint::feed(&mut sig, &t.generics);
    sig.section("supertraits");
    fingerprint::feed(&mut sig, &t.supertraits);
    fingerprint::feed(&mut sig, &t.generics.where_clause);
    let mut body = Fingerprinter::body();
    body.section("trait");
    fingerprint::feed(&mut body, &t.generics);
    fingerprint::feed(&mut body, &t.generics.where_clause);
    for ti in &t.items {
        sig.section("item");
        match ti {
            TraitItem::Fn(f) => {
                fingerprint::attrs(&mut sig, &f.attrs);
                fingerprint::feed(&mut sig, &f.sig);
            }
            TraitItem::Const(k) => {
                fingerprint::attrs(&mut sig, &k.attrs);
                sig.ident(&k.ident.to_string());
                fingerprint::feed(&mut sig, &k.generics);
                fingerprint::feed(&mut sig, &k.ty);
            }
            other => fingerprint::feed(&mut sig, other),
        }
        body.section("item");
        fingerprint::feed_canonical(&mut body, ti, Canon::visit_trait_item_mut);
    }
    (sig.finish(), body.finish())
}

/// A constant or static: everything but the value is the contract; the
/// value is the body.
fn value(
    keyword: &str,
    attrs: &[Attribute],
    vis: &syn::Visibility,
    mutability: Option<&syn::StaticMutability>,
    ident: &syn::Ident,
    ty: &Type,
    expr: &Expr,
) -> (Fingerprint, Fingerprint) {
    let mut sig = Fingerprinter::sig();
    sig.section(keyword);
    fingerprint::attrs(&mut sig, attrs);
    fingerprint::feed(&mut sig, vis);
    if let Some(m) = mutability {
        fingerprint::feed(&mut sig, m);
    }
    sig.ident(&ident.to_string());
    fingerprint::feed(&mut sig, ty);
    let mut body = Fingerprinter::body();
    body.section(keyword);
    fingerprint::feed_canonical(&mut body, expr, Canon::visit_expr_mut);
    (sig.finish(), body.finish())
}

// ---------------------------------------------------------------- helpers

fn span_of(s: PmSpan) -> Span {
    let (a, b) = (s.start(), s.end());
    Span::new(a.line as u32, a.column as u32 + 1, b.line as u32, b.column as u32 + 1)
}

fn vis_of(v: &syn::Visibility) -> Visibility {
    match v {
        syn::Visibility::Public(_) => Visibility::Public,
        syn::Visibility::Restricted(r) if r.path.is_ident("crate") => Visibility::Crate,
        syn::Visibility::Restricted(r) => Visibility::Restricted(tokens(&r.path)),
        syn::Visibility::Inherited => Visibility::Private,
    }
}

fn vis_prefix(v: &syn::Visibility) -> String {
    match v {
        syn::Visibility::Inherited => String::new(),
        v => format!("{} ", tokens(v)),
    }
}

fn is_test_attr(a: &Attribute) -> bool {
    let p = a.path();
    if p.is_ident("test") || p.segments.last().is_some_and(|s| s.ident == "test") {
        return true;
    }
    p.is_ident("cfg") && tokens(&a.meta).replace(' ', "") == "cfg(test)"
}

/// First sentence of the doc comment, at most 160 chars.
fn doc_of(attrs: &[Attribute]) -> Option<String> {
    let mut text = String::new();
    for a in attrs {
        if let syn::Meta::NameValue(nv) = &a.meta {
            if nv.path.is_ident("doc") {
                if let Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    let line = s.value();
                    let line = line.trim();
                    if line.is_empty() && !text.is_empty() {
                        break; // end of first paragraph
                    }
                    if line.starts_with('#') || line.starts_with("```") {
                        if text.is_empty() { continue } else { break }
                    }
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(line);
                }
            }
        }
    }
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let first = match text.find(". ") {
        Some(i) => &text[..=i],
        None => text,
    };
    Some(clip(first, DOC_SUMMARY_MAX))
}

/// Attributes worth keeping as tags.
fn tags_of(attrs: &[Attribute]) -> Vec<String> {
    const KEEP: &[&str] = &["derive", "cfg", "deprecated", "non_exhaustive", "repr", "must_use", "async_trait", "test"];
    let mut out = Vec::new();
    for a in attrs {
        let p = a.path();
        let multi = p.segments.len() > 1;
        if multi || KEEP.iter().any(|k| p.is_ident(k)) {
            let t = squeeze(&tokens(&a.meta));
            out.push(clip(&t, LABEL_MAX));
        }
    }
    out
}

fn fn_tags(sig: &syn::Signature, attrs: &[Attribute]) -> Vec<String> {
    let mut t = tags_of(attrs);
    if sig.constness.is_some() {
        t.push("const".into());
    }
    if sig.asyncness.is_some() {
        t.push("async".into());
    }
    if sig.unsafety.is_some() {
        t.push("unsafe".into());
    }
    t
}

fn generics_of(g: &Generics) -> Vec<String> {
    g.params
        .iter()
        .map(|p| match p {
            GenericParam::Type(t) => squeeze(&tokens(t)),
            GenericParam::Lifetime(l) => squeeze(&tokens(l)),
            GenericParam::Const(c) => squeeze(&tokens(c)),
        })
        .collect()
}

/// The names a generic list puts in scope as types or values: its type and
/// const parameters (lifetimes never appear in a resolved path). Bounds in a
/// `where` clause constrain these names but declare none, and an `impl
/// Trait` argument has no name: its paths are the trait's own.
fn param_names(g: &Generics) -> Vec<String> {
    g.params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(t) => Some(t.ident.to_string()),
            GenericParam::Const(c) => Some(c.ident.to_string()),
            GenericParam::Lifetime(_) => None,
        })
        .collect()
}

fn fields_of(fields: &Fields) -> Vec<RawMember> {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let mut refs = Vec::new();
            type_refs(&f.ty, &mut refs);
            RawMember {
                name: f.ident.as_ref().map_or_else(|| i.to_string(), ToString::to_string),
                kind: MemberKind::Field,
                ty: Some(tokens(&f.ty)),
                vis: vis_of(&f.vis),
                span: span_of(f.span()),
                refs,
                type_params: Vec::new(),
            }
        })
        .collect()
}

pub(crate) fn path_segs(p: &syn::Path) -> Segs {
    p.segments.iter().map(|s| s.ident.to_string()).collect()
}

/// Every path mentioned in a type, outermost first.
fn type_refs(ty: &Type, out: &mut Vec<Segs>) {
    struct V<'a>(&'a mut Vec<Segs>);
    impl<'ast> syn::visit::Visit<'ast> for V<'_> {
        fn visit_path(&mut self, p: &'ast syn::Path) {
            let segs = path_segs(p);
            if !self.0.contains(&segs) {
                self.0.push(segs);
            }
            syn::visit::visit_path(self, p);
        }
    }
    syn::visit::Visit::visit_type(&mut V(out), ty);
}

fn sig_refs(sig: &syn::Signature, out: &mut Vec<Segs>) {
    for input in &sig.inputs {
        if let FnArg::Typed(t) = input {
            type_refs(&t.ty, out);
        }
    }
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        type_refs(ty, out);
    }
}

/// Outermost path of a type, looking through references and parens.
fn first_path(ty: &Type) -> Option<Segs> {
    match ty {
        Type::Path(p) => Some(path_segs(&p.path)),
        Type::Reference(r) => first_path(&r.elem),
        Type::Paren(p) => first_path(&p.elem),
        Type::Group(g) => first_path(&g.elem),
        _ => None,
    }
}

fn params_of(sig: &syn::Signature) -> BTreeMap<String, Recv> {
    let mut env = BTreeMap::new();
    for input in &sig.inputs {
        match input {
            FnArg::Receiver(_) => {
                env.insert("self".to_owned(), Recv::SelfValue);
            }
            FnArg::Typed(t) => {
                if let Pat::Ident(id) = &*t.pat {
                    let mut refs = Vec::new();
                    type_refs(&t.ty, &mut refs);
                    env.insert(id.ident.to_string(), Recv::Typed(refs));
                }
            }
        }
    }
    env
}

fn flatten_use(tree: &UseTree, prefix: &mut Segs, out: &mut Vec<RawUse>) {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            flatten_use(&p.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(n) => {
            let name = n.ident.to_string();
            if name == "self" {
                if let Some(last) = prefix.last() {
                    out.push(RawUse { alias: last.clone(), target: prefix.clone() });
                }
            } else {
                let mut t = prefix.clone();
                t.push(name.clone());
                out.push(RawUse { alias: name, target: t });
            }
        }
        UseTree::Rename(r) => {
            let mut t = prefix.clone();
            if r.ident != "self" {
                t.push(r.ident.to_string());
            }
            out.push(RawUse { alias: r.rename.to_string(), target: t });
        }
        UseTree::Glob(_) => out.push(RawUse { alias: "*".into(), target: prefix.clone() }),
        UseTree::Group(g) => {
            for t in &g.items {
                flatten_use(t, prefix, out);
            }
        }
    }
}

// ---------------------------------------------------------------- flows

/// Methods whose closure argument runs once per element.
const LOOPING: &[&str] = &[
    "for_each",
    "try_for_each",
    "filter",
    "filter_map",
    "flat_map",
    "fold",
    "try_fold",
    "any",
    "all",
    "find",
    "find_map",
    "position",
    "inspect",
    "retain",
    "scan",
    "take_while",
    "skip_while",
    "sort_by",
    "sort_by_key",
    "sort_unstable_by",
    "sort_unstable_by_key",
    "map_while",
    "par_iter",
];

struct FlowWalker {
    env: BTreeMap<String, Recv>,
    /// Current `expr` recursion depth, see [`MAX_EXPR_DEPTH`].
    depth: u32,
}

impl FlowWalker {
    fn new(env: BTreeMap<String, Recv>) -> Self {
        Self { env, depth: 0 }
    }

    fn block(&mut self, b: &Block) -> Vec<RawStep> {
        let mut out = Vec::new();
        self.stmts(&b.stmts, &mut out);
        out
    }

    fn stmts(&mut self, stmts: &[Stmt], out: &mut Vec<RawStep>) {
        for s in stmts {
            match s {
                Stmt::Local(l) => {
                    if let Some(init) = &l.init {
                        self.expr(&init.expr, out);
                        if let Some((_, diverge)) = &init.diverge {
                            let steps = self.sub(diverge);
                            if !steps.is_empty() {
                                out.push(RawStep::Optional("let-else".into(), steps));
                            }
                        }
                    }
                    self.bind(&l.pat, l.init.as_ref().map(|i| &*i.expr));
                }
                Stmt::Expr(e, _) => self.expr(e, out),
                Stmt::Macro(m) => self.mac(&m.mac, out),
                Stmt::Item(_) => {}
            }
        }
    }

    /// Record the type of a `let` binding when it is evident.
    fn bind(&mut self, pat: &Pat, init: Option<&Expr>) {
        let (name, declared) = match pat {
            Pat::Type(pt) => match &*pt.pat {
                Pat::Ident(id) => {
                    let mut refs = Vec::new();
                    type_refs(&pt.ty, &mut refs);
                    (id.ident.to_string(), Some(refs))
                }
                _ => return,
            },
            Pat::Ident(id) => (id.ident.to_string(), None),
            // `let Some(x) = init else { .. }`, `let (a, b) = init`.
            other => {
                match init {
                    Some(e) => self.bind_destructured(other, e, false),
                    None => self.forget(other),
                }
                return;
            }
        };
        let inferred = declared.or_else(|| init.and_then(constructed_type).map(|t| vec![t]));
        match inferred {
            Some(refs) => {
                self.env.insert(name, Recv::Typed(refs));
            }
            // Bound to something we could not type (a call chain, a
            // computation): calls on it are not guessed by name, but where
            // it came from is kept for the parts a pattern takes from it.
            None if init.is_some() => {
                let origin = init.map_or(Recv::Unknown, |e| self.origin(e));
                self.env.insert(name, Recv::Computed(Box::new(origin)));
            }
            None => {
                self.env.remove(&name);
            }
        }
    }

    /// Bind the names a destructuring pattern (`Some(x)`, `(a, b)`,
    /// `Foo { bar, .. }`) takes from `scrutinee`; `iterated` for the pattern
    /// of a `for` loop, which takes elements rather than the value itself.
    ///
    /// * A typed scrutinee is looked into where the shape says which type
    ///   the one bound name gets: `Some(x)` / `Ok(x)` take the first type
    ///   argument (`opt: Option<Db>` makes `x` a `Db`), `Err(e)` the second,
    ///   and `for x in` the element of a one-parameter collection
    ///   (`Vec<Db>`, `HashSet<Db>`, `Option<Db>`).
    /// * Anything else becomes [`Recv::Derived`] from the scrutinee's
    ///   origin, so a later call on it is matched by name only when that
    ///   origin is internal code.
    fn bind_destructured(&mut self, pat: &Pat, scrutinee: &Expr, iterated: bool) {
        let mut names = Vec::new();
        pattern_names(pat, &mut names);
        let origin = self.origin(scrutinee);
        let class = match (&origin, names.len()) {
            (Recv::Typed(refs), 1) => peel(pat, refs, iterated).map(Recv::Typed),
            _ => None,
        }
        .unwrap_or_else(|| Recv::Derived(Box::new(origin)));
        for name in names {
            self.env.insert(name, class.clone());
        }
    }

    /// Where the value of `e` comes from, for [`Recv::Derived`]: the
    /// receiver at the root of its method chain, field accesses, `?`,
    /// `.await` and borrows, or the path it calls.
    fn origin(&self, e: &Expr) -> Recv {
        match e {
            Expr::Path(p) if p.path.segments.len() == 1 => {
                self.env.get(&p.path.segments[0].ident.to_string()).cloned().unwrap_or(Recv::Untyped)
            }
            Expr::Field(f) => match (&*f.base, &f.member) {
                (Expr::Path(p), syn::Member::Named(n)) if p.path.is_ident("self") => Recv::SelfField(n.to_string()),
                _ => self.origin(&f.base),
            },
            // A turbofish names the result's type: `x.try_into::<Config>()`,
            // `any.downcast_ref::<Metrics>()`, `iter.collect::<Vec<Job>>()`.
            Expr::MethodCall(m) => match m.turbofish.as_ref().and_then(|t| turbofish_refs(&t.args)) {
                Some(refs) => Recv::Typed(refs),
                None => self.origin(&m.receiver),
            },
            Expr::Call(c) => match &*c.func {
                Expr::Path(p) => match constructed_type(e) {
                    Some(t) => Recv::Typed(vec![t]),
                    None => {
                        // `serde_json::from_str::<Store>(..)`.
                        let named = p.path.segments.last().and_then(|seg| match &seg.arguments {
                            syn::PathArguments::AngleBracketed(a) => turbofish_refs(&a.args),
                            _ => None,
                        });
                        named.map_or_else(|| Recv::Returned(path_segs(&p.path)), Recv::Typed)
                    }
                },
                _ => Recv::Unknown,
            },
            Expr::Try(t) => self.origin(&t.expr),
            Expr::Await(a) => self.origin(&a.base),
            Expr::Paren(p) => self.origin(&p.expr),
            Expr::Reference(r) => self.origin(&r.expr),
            Expr::Unary(u) => self.origin(&u.expr),
            Expr::Index(i) => self.origin(&i.expr),
            _ => Recv::Unknown,
        }
    }

    /// Drop what is known about the names `pat` binds (a shadowing binding
    /// without a value).
    fn forget(&mut self, pat: &Pat) {
        let mut names = Vec::new();
        pattern_names(pat, &mut names);
        for name in names {
            self.env.remove(&name);
        }
    }

    fn sub(&mut self, e: &Expr) -> Vec<RawStep> {
        let mut out = Vec::new();
        self.expr(e, &mut out);
        out
    }

    fn sub_block(&mut self, b: &Block) -> Vec<RawStep> {
        let saved = self.env.clone();
        let out = self.block(b);
        self.env = saved;
        out
    }

    fn expr(&mut self, e: &Expr, out: &mut Vec<RawStep>) {
        if self.depth >= MAX_EXPR_DEPTH {
            return;
        }
        self.depth += 1;
        self.expr_inner(e, out);
        self.depth -= 1;
    }

    fn expr_inner(&mut self, e: &Expr, out: &mut Vec<RawStep>) {
        match e {
            Expr::Call(c) => {
                let mut deferred = Vec::new();
                for a in &c.args {
                    self.arg(a, out, &mut deferred);
                }
                match &*c.func {
                    Expr::Path(p) => {
                        let segs = path_segs(&p.path);
                        let name = segs.last().cloned().unwrap_or_default();
                        // Tuple-struct / variant constructors (`Some(x)`,
                        // `Foo(x)`) are data, not calls.
                        if !name.starts_with(|ch: char| ch.is_uppercase()) {
                            let shown = if segs.len() >= 2 {
                                format!("{}::{}", segs[segs.len() - 2], name)
                            } else {
                                name.clone()
                            };
                            out.push(call(Callee::Path(segs), &shown, &c.args, CallKind::Function, c.span()));
                            place_deferred(&name, deferred, LOOPING, out);
                            return;
                        }
                    }
                    other => self.expr(other, out),
                }
                place_deferred("", deferred, LOOPING, out);
            }
            Expr::MethodCall(m) => {
                self.expr(&m.receiver, out);
                let mut deferred = Vec::new();
                for a in &m.args {
                    self.arg(a, out, &mut deferred);
                }
                let recv = self.recv(&m.receiver);
                let name = m.method.to_string();
                out.push(call(
                    Callee::Method { recv, name: name.clone() },
                    &name,
                    &m.args,
                    CallKind::Method,
                    m.method.span(),
                ));
                place_deferred(&name, deferred, LOOPING, out);
            }
            Expr::Await(a) => {
                self.expr(&a.base, out);
                if is_call(&a.base) {
                    if let Some(RawStep::Call(c)) = last_call_mut(out) {
                        c.awaited = true;
                    }
                }
            }
            Expr::Try(t) => {
                self.expr(&t.expr, out);
                if is_call(&t.expr) {
                    if let Some(RawStep::Call(c)) = last_call_mut(out) {
                        c.fallible = true;
                    }
                }
            }
            Expr::If(i) => {
                let mut arms = Vec::new();
                let mut cur = Some(i);
                let mut first = true;
                while let Some(ifx) = cur.take() {
                    let label = cond_label(&ifx.cond);
                    let mut cond_steps = Vec::new();
                    self.cond(&ifx.cond, &mut cond_steps);
                    let body = self.sub_block(&ifx.then_branch);
                    if first {
                        out.append(&mut cond_steps);
                        arms.push((label, body));
                    } else {
                        cond_steps.extend(body);
                        arms.push((format!("if {label}"), cond_steps));
                    }
                    first = false;
                    match ifx.else_branch.as_ref().map(|(_, e)| &**e) {
                        Some(Expr::If(next)) => cur = Some(next),
                        Some(Expr::Block(b)) => arms.push((String::new(), self.sub_block(&b.block))),
                        Some(other) => arms.push((String::new(), self.sub(other))),
                        None => {}
                    }
                }
                push_arms(arms, out);
            }
            Expr::Match(m) => {
                self.expr(&m.expr, out);
                let arms = m
                    .arms
                    .iter()
                    .map(|a| {
                        let mut label = squeeze(&tokens(&a.pat));
                        if let Some((_, g)) = &a.guard {
                            label = format!("{label} if {}", squeeze(&tokens(g)));
                        }
                        let saved = self.env.clone();
                        self.bind_destructured(&a.pat, &m.expr, false);
                        let steps = self.sub(&a.body);
                        self.env = saved;
                        (clip(&label, LABEL_MAX), steps)
                    })
                    .collect::<Vec<_>>();
                if arms.iter().any(|(_, s)| !s.is_empty()) {
                    out.push(RawStep::Branch(arms));
                }
            }
            Expr::ForLoop(f) => {
                self.expr(&f.expr, out);
                let label =
                    clip(&format!("for {} in {}", squeeze(&tokens(&f.pat)), squeeze(&tokens(&f.expr))), LABEL_MAX);
                let saved = self.env.clone();
                self.bind_destructured(&f.pat, &f.expr, true);
                let body = self.sub_block(&f.body);
                self.env = saved;
                if !body.is_empty() {
                    out.push(RawStep::Loop(label, body));
                }
            }
            Expr::While(w) => {
                let label = clip(&format!("while {}", cond_label(&w.cond)), LABEL_MAX);
                let mut body = Vec::new();
                self.cond(&w.cond, &mut body);
                body.extend(self.sub_block(&w.body));
                if !body.is_empty() {
                    out.push(RawStep::Loop(label, body));
                }
            }
            Expr::Loop(l) => {
                let body = self.sub_block(&l.body);
                if !body.is_empty() {
                    out.push(RawStep::Loop("loop".into(), body));
                }
            }
            Expr::Block(b) => self.stmts(&b.block.stmts, out),
            Expr::Unsafe(b) => self.stmts(&b.block.stmts, out),
            Expr::Async(b) => self.stmts(&b.block.stmts, out),
            Expr::TryBlock(b) => self.stmts(&b.block.stmts, out),
            Expr::Const(b) => self.stmts(&b.block.stmts, out),
            Expr::Closure(c) => {
                let steps = self.sub(&c.body);
                if !steps.is_empty() {
                    out.push(RawStep::Optional("closure".into(), steps));
                }
            }
            Expr::Return(r) => {
                if let Some(e) = &r.expr {
                    self.expr(e, out);
                }
                let label = r.expr.as_ref().map_or_else(
                    || "return".to_owned(),
                    |e| clip(&format!("return {}", squeeze(&tokens(e))), LABEL_MAX),
                );
                out.push(RawStep::Return(label, r.span().start().line as u32));
            }
            Expr::Macro(m) => self.mac(&m.mac, out),
            Expr::Binary(b) => {
                self.expr(&b.left, out);
                self.expr(&b.right, out);
            }
            Expr::Assign(a) => {
                self.expr(&a.right, out);
                self.expr(&a.left, out);
            }
            Expr::Unary(u) => self.expr(&u.expr, out),
            Expr::Paren(p) => self.expr(&p.expr, out),
            Expr::Group(g) => self.expr(&g.expr, out),
            Expr::Reference(r) => self.expr(&r.expr, out),
            Expr::Field(f) => self.expr(&f.base, out),
            Expr::Index(i) => {
                self.expr(&i.expr, out);
                self.expr(&i.index, out);
            }
            Expr::Cast(c) => self.expr(&c.expr, out),
            Expr::Let(l) => self.expr(&l.expr, out),
            Expr::Tuple(t) => t.elems.iter().for_each(|e| self.expr(e, out)),
            Expr::Array(a) => a.elems.iter().for_each(|e| self.expr(e, out)),
            Expr::Repeat(r) => self.expr(&r.expr, out),
            Expr::Range(r) => {
                if let Some(s) = &r.start {
                    self.expr(s, out);
                }
                if let Some(e) = &r.end {
                    self.expr(e, out);
                }
            }
            Expr::Struct(s) => {
                for f in &s.fields {
                    self.expr(&f.expr, out);
                }
                if let Some(rest) = &s.rest {
                    self.expr(rest, out);
                }
            }
            Expr::Break(b) => {
                if let Some(e) = &b.expr {
                    self.expr(e, out);
                }
            }
            Expr::Yield(y) => {
                if let Some(e) = &y.expr {
                    self.expr(e, out);
                }
            }
            _ => {}
        }
    }

    /// Condition of `if` / `while`: `let` scrutinee or boolean expression.
    fn cond(&mut self, cond: &Expr, out: &mut Vec<RawStep>) {
        self.expr(cond, out);
        if let Expr::Let(l) = cond {
            self.bind(&l.pat, Some(&l.expr));
        }
    }

    /// Walk a call argument. Closures and async blocks are deferred: their
    /// bodies run during (not before) the call, so they render after it.
    fn arg(&mut self, a: &Expr, out: &mut Vec<RawStep>, deferred: &mut Vec<Vec<RawStep>>) {
        match a {
            Expr::Closure(c) => {
                let saved = self.env.clone();
                let steps = self.sub(&c.body);
                self.env = saved;
                if !steps.is_empty() {
                    deferred.push(steps);
                }
            }
            Expr::Async(b) => {
                let steps = self.sub_block(&b.block);
                if !steps.is_empty() {
                    deferred.push(steps);
                }
            }
            other => self.expr(other, out),
        }
    }

    /// Calls inside macro arguments (`vec![f()]`, `assert!(g())`,
    /// `format!("{}", h())`). Macros that do not parse as comma-separated
    /// expressions are skipped.
    fn mac(&mut self, m: &syn::Macro, out: &mut Vec<RawStep>) {
        let parser = Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        if let Ok(args) = m.parse_body_with(parser) {
            for a in &args {
                self.expr(a, out);
            }
        } else if let Ok(stmts) = m.parse_body_with(Block::parse_within) {
            self.stmts(&stmts, out);
        }
    }

    fn recv(&self, e: &Expr) -> Recv {
        match e {
            Expr::Path(p) if p.path.segments.len() == 1 => {
                let name = p.path.segments[0].ident.to_string();
                self.env.get(&name).cloned().unwrap_or(Recv::Untyped)
            }
            Expr::Field(f) => match (&*f.base, &f.member) {
                (Expr::Path(p), syn::Member::Named(n)) if p.path.is_ident("self") => Recv::SelfField(n.to_string()),
                (Expr::Path(_) | Expr::Field(_), syn::Member::Named(_)) => Recv::Untyped,
                _ => Recv::Unknown,
            },
            Expr::Paren(p) => self.recv(&p.expr),
            Expr::Reference(r) => self.recv(&r.expr),
            Expr::Unary(u) => self.recv(&u.expr),
            _ => Recv::Unknown,
        }
    }
}

/// The type the single name bound by `pat` gets from a scrutinee of
/// declared type `refs` (outermost first), when the shape says so.
fn peel(pat: &Pat, refs: &[Segs], iterated: bool) -> Option<Vec<Segs>> {
    const ONE_PARAMETER: &[&str] =
        &["Vec", "VecDeque", "HashSet", "BTreeSet", "IndexSet", "Option", "BinaryHeap", "LinkedList"];
    let outer = refs.first()?.last()?.as_str();
    let arg = |i: usize| refs.get(i).map(|r| vec![r.clone()]);
    if iterated {
        return if ONE_PARAMETER.contains(&outer) && refs.len() == 2 { arg(1) } else { None };
    }
    let Pat::TupleStruct(ts) = pat else { return None };
    match ts.path.segments.last()?.ident.to_string().as_str() {
        "Some" if outer == "Option" => arg(1),
        "Ok" if outer == "Result" => arg(1),
        "Err" if outer == "Result" => arg(2),
        // A value known only by its own type, as a constructor or turbofish
        // gives it (`Ok(store) = RoleStore::new(..).await`,
        // `Some(m) = any.downcast_ref::<Metrics>()`): the wrapper is implied,
        // the payload is that type.
        "Some" | "Ok" if refs.len() == 1 && !["Option", "Result"].contains(&outer) => arg(0),
        _ => None,
    }
}

/// The type paths of a turbofish with exactly one type argument
/// (`::<Config>`, `::<Vec<Job>>`).
fn turbofish_refs(args: &Punctuated<syn::GenericArgument, syn::Token![,]>) -> Option<Vec<Segs>> {
    let mut types = args.iter().filter_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    });
    let (Some(ty), None) = (types.next(), types.next()) else { return None };
    let mut refs = Vec::new();
    type_refs(ty, &mut refs);
    (!refs.is_empty()).then_some(refs)
}

/// The names a pattern binds, in source order.
fn pattern_names(pat: &Pat, out: &mut Vec<String>) {
    struct Names<'a>(&'a mut Vec<String>);
    impl<'ast> syn::visit::Visit<'ast> for Names<'_> {
        fn visit_pat_ident(&mut self, p: &'ast syn::PatIdent) {
            self.0.push(p.ident.to_string());
            syn::visit::visit_pat_ident(self, p);
        }
        // Paths inside patterns (`Foo::Bar`, consts) bind nothing.
        fn visit_expr(&mut self, _: &'ast Expr) {}
    }
    syn::visit::Visit::visit_pat(&mut Names(out), pat);
}

fn call(callee: Callee, name: &str, args: &Punctuated<Expr, syn::Token![,]>, kind: CallKind, span: PmSpan) -> RawStep {
    let sketch: Vec<String> = args.iter().map(arg_sketch).collect();
    RawStep::Call(RawCall::new(callee, call_label(name, &sketch), kind, span.start().line as u32))
}

/// A compact stand-in for an argument: identifiers and short literals are
/// kept, everything else becomes `_`.
fn arg_sketch(e: &Expr) -> String {
    match e {
        Expr::Path(p) => p.path.segments.last().map_or_else(|| "_".into(), |s| s.ident.to_string()),
        Expr::Lit(l) => clip(&tokens(l), ARG_LITERAL_MAX),
        Expr::Reference(r) => format!("&{}", arg_sketch(&r.expr)),
        Expr::Field(f) => match &f.member {
            syn::Member::Named(n) => format!("{}.{}", arg_sketch(&f.base), n),
            syn::Member::Unnamed(i) => format!("{}.{}", arg_sketch(&f.base), i.index),
        },
        Expr::Closure(_) => "|..|".into(),
        Expr::Async(_) => "async{..}".into(),
        Expr::MethodCall(m) => format!("{}()", m.method),
        Expr::Call(c) => match &*c.func {
            Expr::Path(p) => p.path.segments.last().map_or_else(|| "_".into(), |s| format!("{}()", s.ident)),
            _ => "_".into(),
        },
        _ => "_".into(),
    }
}

fn cond_label(cond: &Expr) -> String {
    let text = match cond {
        Expr::Let(l) => format!("let {} = {}", squeeze(&tokens(&l.pat)), squeeze(&tokens(&l.expr))),
        other => squeeze(&tokens(other)),
    };
    condition_label(&text)
}

fn is_call(e: &Expr) -> bool {
    match e {
        Expr::Call(_) | Expr::MethodCall(_) => true,
        Expr::Paren(p) => is_call(&p.expr),
        Expr::Await(a) => is_call(&a.base),
        Expr::Try(t) => is_call(&t.expr),
        _ => false,
    }
}

/// `Foo::new(..)`, `Foo { .. }`, `Foo::new(..)?`, `Foo::open(..).await` → `Foo`.
fn constructed_type(e: &Expr) -> Option<Segs> {
    match e {
        Expr::Call(c) => match &*c.func {
            Expr::Path(p) if p.path.segments.len() >= 2 => {
                let segs = path_segs(&p.path);
                let ty = &segs[segs.len() - 2];
                ty.starts_with(|ch: char| ch.is_uppercase()).then(|| segs[..segs.len() - 1].to_vec())
            }
            _ => None,
        },
        Expr::Struct(s) => Some(path_segs(&s.path)),
        Expr::Try(t) => constructed_type(&t.expr),
        Expr::Await(a) => constructed_type(&a.base),
        Expr::Paren(p) => constructed_type(&p.expr),
        Expr::Reference(r) => constructed_type(&r.expr),
        _ => None,
    }
}
