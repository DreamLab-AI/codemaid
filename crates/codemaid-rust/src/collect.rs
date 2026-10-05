//! Pass 1: parse one file with `syn` and collect unresolved raw data.

use std::collections::BTreeMap;

use codemaid_model::{CallKind, MemberKind, SourcePath, Span, SymbolKind, Visibility};
use proc_macro2::Span as PmSpan;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Block, Expr, Fields, FnArg, GenericParam, Generics, ImplItem, Item, Pat, Stmt, TraitItem, Type,
    TypeParamBound, UseTree,
};

use crate::RustOptions;
use crate::layout::FileRole;
use crate::raw::*;
use crate::tidy::{clip, squeeze, tokens};

/// Max characters for labels on arrows and fragments.
const LABEL_MAX: usize = 56;
/// Max characters for a stored signature.
const SIG_MAX: usize = 200;

/// Parse `text` and collect everything pass 2 needs.
pub(crate) fn collect_file(path: &SourcePath, role: &FileRole, text: &str, opts: &RustOptions) -> RawFile {
    let mut raw = RawFile {
        path: path.clone(),
        role: role.clone(),
        text_lines: text.lines().count() as u32,
        hash: codemaid_model::ContentHash::of_text(text),
        modules: Vec::new(),
        items: Vec::new(),
        impls: Vec::new(),
        error: None,
    };
    let file = match syn::parse_file(text) {
        Ok(f) => f,
        Err(e) => {
            raw.error = Some(format!("{}:{}: {e}", e.span().start().line, e.span().start().column + 1));
            raw.modules.push(RawModule {
                path: role.module.clone(),
                span: Span::new(1, 1, raw.text_lines.max(1), 1),
                doc: None,
                vis: Visibility::Public,
                uses: Vec::new(),
                tags: vec!["parse_error".into()],
            });
            return raw;
        }
    };
    let mut c = Collector { opts, raw: &mut raw };
    let span = Span::new(1, 1, c.raw.text_lines.max(1), 1);
    c.module(role.module.clone(), &file.attrs, &file.items, span, Visibility::Public);
    raw
}

struct Collector<'a> {
    opts: &'a RustOptions,
    raw: &'a mut RawFile,
}

impl Collector<'_> {
    fn module(&mut self, path: Segs, attrs: &[Attribute], items: &[Item], span: Span, vis: Visibility) {
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
        });
        for item in items {
            self.item(&path, item);
        }
    }

    fn skip(&self, attrs: &[Attribute]) -> bool {
        !self.opts.include_tests && attrs.iter().any(is_test_attr)
    }

    fn item(&mut self, module: &Segs, item: &Item) {
        let base = |name: String, kind, vis: &syn::Visibility, attrs: &[Attribute], span: PmSpan| RawItem {
            module: module.clone(),
            name,
            kind,
            vis: vis_of(vis),
            span: span_of(span),
            signature: None,
            doc: doc_of(attrs),
            generics: Vec::new(),
            tags: tags_of(attrs),
            members: Vec::new(),
            sig_refs: Vec::new(),
            supertraits: Vec::new(),
            methods: Vec::new(),
            flow: Vec::new(),
        };
        match item {
            Item::Mod(m) => {
                if self.skip(&m.attrs) {
                    return;
                }
                if let Some((_, items)) = &m.content {
                    let mut path = module.clone();
                    path.push(m.ident.to_string());
                    self.module(path, &m.attrs, items, span_of(m.span()), vis_of(&m.vis));
                }
            }
            Item::Struct(s) => {
                if self.skip(&s.attrs) {
                    return;
                }
                let mut it = base(s.ident.to_string(), SymbolKind::Struct, &s.vis, &s.attrs, s.span());
                it.generics = generics_of(&s.generics);
                it.members = fields_of(&s.fields);
                self.raw.items.push(it);
            }
            Item::Union(u) => {
                let mut it = base(u.ident.to_string(), SymbolKind::Union, &u.vis, &u.attrs, u.span());
                it.generics = generics_of(&u.generics);
                it.members = fields_of(&Fields::Named(u.fields.clone()));
                self.raw.items.push(it);
            }
            Item::Enum(e) => {
                if self.skip(&e.attrs) {
                    return;
                }
                let mut it = base(e.ident.to_string(), SymbolKind::Enum, &e.vis, &e.attrs, e.span());
                it.generics = generics_of(&e.generics);
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
                    });
                }
                self.raw.items.push(it);
            }
            Item::Trait(t) => {
                if self.skip(&t.attrs) {
                    return;
                }
                let mut it = base(t.ident.to_string(), SymbolKind::Trait, &t.vis, &t.attrs, t.span());
                it.generics = generics_of(&t.generics);
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
                                    it.methods.push(RawFn {
                                        name: f.sig.ident.to_string(),
                                        vis: Visibility::Public,
                                        span: span_of(f.span()),
                                        signature: sig,
                                        doc: doc_of(&f.attrs),
                                        generics: generics_of(&f.sig.generics),
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
                            });
                        }
                        _ => {}
                    }
                }
                self.raw.items.push(it);
            }
            Item::Type(t) => {
                let mut it = base(t.ident.to_string(), SymbolKind::TypeAlias, &t.vis, &t.attrs, t.span());
                it.generics = generics_of(&t.generics);
                it.signature = Some(clip(&format!("type {} = {}", t.ident, tokens(&t.ty)), SIG_MAX));
                type_refs(&t.ty, &mut it.sig_refs);
                self.raw.items.push(it);
            }
            Item::Fn(f) => {
                if self.skip(&f.attrs) {
                    return;
                }
                let mut it = base(f.sig.ident.to_string(), SymbolKind::Function, &f.vis, &f.attrs, f.span());
                it.generics = generics_of(&f.sig.generics);
                it.signature = Some(clip(&format!("{}{}", vis_prefix(&f.vis), tokens(&f.sig)), SIG_MAX));
                it.tags.extend(fn_tags(&f.sig, &[]));
                sig_refs(&f.sig, &mut it.sig_refs);
                it.flow = FlowWalker::new(params_of(&f.sig)).block(&f.block);
                self.raw.items.push(it);
            }
            Item::Const(k) => {
                let mut it = base(k.ident.to_string(), SymbolKind::Const, &k.vis, &k.attrs, k.span());
                it.signature = Some(clip(&format!("const {}: {}", k.ident, tokens(&k.ty)), SIG_MAX));
                type_refs(&k.ty, &mut it.sig_refs);
                self.raw.items.push(it);
            }
            Item::Static(s) => {
                let mut it = base(s.ident.to_string(), SymbolKind::Static, &s.vis, &s.attrs, s.span());
                it.signature = Some(clip(&format!("static {}: {}", s.ident, tokens(&s.ty)), SIG_MAX));
                type_refs(&s.ty, &mut it.sig_refs);
                self.raw.items.push(it);
            }
            Item::Macro(m) => {
                if let Some(ident) = &m.ident {
                    let vis = if m.attrs.iter().any(|a| a.path().is_ident("macro_export")) {
                        syn::Visibility::Public(Default::default())
                    } else {
                        syn::Visibility::Inherited
                    };
                    self.raw.items.push(base(ident.to_string(), SymbolKind::Macro, &vis, &m.attrs, m.span()));
                }
            }
            Item::Impl(i) => {
                if self.skip(&i.attrs) {
                    return;
                }
                let self_ty = first_path(&i.self_ty);
                let trait_ = i.trait_.as_ref().map(|(_, p, _)| (path_segs(p), tokens(p)));
                let is_trait_impl = trait_.is_some();
                let mut methods = Vec::new();
                for ii in &i.items {
                    if let ImplItem::Fn(f) = ii {
                        if self.skip(&f.attrs) {
                            continue;
                        }
                        let mut refs = Vec::new();
                        sig_refs(&f.sig, &mut refs);
                        let vis = if is_trait_impl { Visibility::Public } else { vis_of(&f.vis) };
                        methods.push(RawFn {
                            name: f.sig.ident.to_string(),
                            span: span_of(f.span()),
                            signature: clip(&format!("{}{}", vis_prefix(&f.vis), tokens(&f.sig)), SIG_MAX),
                            doc: doc_of(&f.attrs),
                            generics: generics_of(&f.sig.generics),
                            tags: fn_tags(&f.sig, &f.attrs),
                            sig_refs: refs,
                            flow: FlowWalker::new(params_of(&f.sig)).block(&f.block),
                            vis,
                        });
                    }
                }
                self.raw.impls.push(RawImpl { module: module.clone(), self_ty, trait_, methods });
            }
            _ => {}
        }
    }
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
    Some(clip(first, 160))
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
}

impl FlowWalker {
    fn new(env: BTreeMap<String, Recv>) -> Self {
        Self { env }
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
            _ => return,
        };
        let inferred = declared.or_else(|| init.and_then(constructed_type).map(|t| vec![t]));
        match inferred {
            Some(refs) => {
                self.env.insert(name, Recv::Typed(refs));
            }
            // Bound to something we could not type (a call chain, a
            // computation): calls on it are not guessed by name.
            None if init.is_some() => {
                self.env.insert(name, Recv::Unknown);
            }
            None => {
                self.env.remove(&name);
            }
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
                            self.flush_deferred(&name, deferred, out);
                            return;
                        }
                    }
                    other => self.expr(other, out),
                }
                self.flush_deferred("", deferred, out);
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
                self.flush_deferred(&name, deferred, out);
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
                let body = self.sub_block(&f.body);
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

    fn flush_deferred(&self, callee: &str, deferred: Vec<Vec<RawStep>>, out: &mut Vec<RawStep>) {
        for steps in deferred {
            if callee.contains("spawn") {
                out.push(RawStep::Parallel(vec![(format!("{callee}ed task"), steps)]));
            } else if LOOPING.contains(&callee) {
                out.push(RawStep::Loop(format!("each via {callee}"), steps));
            } else {
                let label = if callee.is_empty() { "closure".to_owned() } else { format!("via {callee}") };
                out.push(RawStep::Optional(label, steps));
            }
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

fn call(callee: Callee, name: &str, args: &Punctuated<Expr, syn::Token![,]>, kind: CallKind, span: PmSpan) -> RawStep {
    let sketch: Vec<String> = args.iter().map(arg_sketch).collect();
    let label = clip(&format!("{name}({})", sketch.join(", ")), LABEL_MAX);
    RawStep::Call(RawCall { callee, label, kind, awaited: false, fallible: false, line: span.start().line as u32 })
}

/// A compact stand-in for an argument: identifiers and short literals are
/// kept, everything else becomes `_`.
fn arg_sketch(e: &Expr) -> String {
    match e {
        Expr::Path(p) => p.path.segments.last().map_or_else(|| "_".into(), |s| s.ident.to_string()),
        Expr::Lit(l) => clip(&tokens(l), 14),
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
    clip(&format!("if {text}"), LABEL_MAX).trim_start_matches("if ").to_owned()
}

fn push_arms(arms: Vec<(String, Vec<RawStep>)>, out: &mut Vec<RawStep>) {
    if arms.iter().all(|(_, s)| s.is_empty()) {
        return;
    }
    if arms.len() == 1 {
        let (label, steps) = arms.into_iter().next().unwrap_or_default();
        out.push(RawStep::Optional(label, steps));
    } else {
        // Empty arms are kept here; pass 2 drops them after resolution and
        // uses their labels to describe what remains.
        out.push(RawStep::Branch(arms));
    }
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

/// The last call pushed at this level (fragments pushed after it, such as a
/// deferred closure, are skipped over).
fn last_call_mut(out: &mut [RawStep]) -> Option<&mut RawStep> {
    out.iter_mut().rev().find(|s| matches!(s, RawStep::Call(_)))
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
