//! Pass 2: resolve raw paths against the whole workspace and build the model.

use std::collections::{BTreeMap, BTreeSet};

use codemaid_model::{
    Arm, Call, Codebase, Confidence, Exit, Flow, Member, Relation, RelationKind, SourceFile, Step, Symbol, SymbolId,
    SymbolKind,
};

use crate::raw::*;
use crate::{Diagnostic, ExternalCalls, RustOptions};

/// Crates whose items are never interesting as relation targets.
const STD_ROOTS: &[&str] = &["std", "core", "alloc"];

/// Names that resolve to the std prelude or primitives when not shadowed.
const PRELUDE: &[&str] = &[
    "Option",
    "Some",
    "None",
    "Result",
    "Ok",
    "Err",
    "Vec",
    "String",
    "Box",
    "ToString",
    "ToOwned",
    "Into",
    "From",
    "TryFrom",
    "TryInto",
    "Iterator",
    "IntoIterator",
    "DoubleEndedIterator",
    "ExactSizeIterator",
    "Extend",
    "Fn",
    "FnMut",
    "FnOnce",
    "Send",
    "Sync",
    "Sized",
    "Unpin",
    "Copy",
    "Clone",
    "Default",
    "Drop",
    "PartialEq",
    "Eq",
    "PartialOrd",
    "Ord",
    "AsRef",
    "AsMut",
    "Debug",
    "Display",
    "Hash",
    "Self",
    "self",
    "bool",
    "char",
    "str",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "usize",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "isize",
    "f32",
    "f64",
    "drop",
    "panic",
];

/// Method names so common in std that an unknown receiver must not be
/// matched to an internal method of the same name.
const COMMON_METHODS: &[&str] = &[
    "new",
    "default",
    "clone",
    "get",
    "get_mut",
    "insert",
    "remove",
    "push",
    "pop",
    "len",
    "is_empty",
    "iter",
    "iter_mut",
    "into_iter",
    "map",
    "and_then",
    "unwrap",
    "expect",
    "ok",
    "err",
    "collect",
    "to_string",
    "to_owned",
    "into",
    "from",
    "as_ref",
    "as_str",
    "contains",
    "extend",
    "join",
    "split",
    "parse",
    "write",
    "read",
    "send",
    "recv",
    "lock",
    "fmt",
    "eq",
    "cmp",
    "hash",
    "next",
    "take",
    "filter",
    "find",
    "first",
    "last",
    "clear",
    "entry",
    "keys",
    "values",
    "run",
    "start",
    "stop",
    "close",
    "open",
    "flush",
    "build",
    "update",
    "apply",
    "file_name",
    "extension",
    "path",
    "exists",
    "display",
    "to_str",
    "as_bytes",
    "trim",
    "lines",
    "chars",
    "bytes",
    "starts_with",
    "ends_with",
    "replace",
    "find_map",
    "position",
    "max",
    "min",
    "sum",
    "count",
    "rev",
    "zip",
    "skip",
    "chain",
    "flatten",
    "cloned",
    "copied",
    "sort",
    "dedup",
    "retain",
    "drain",
    "truncate",
    "with_capacity",
    "capacity",
    "reserve",
    "as_slice",
    "to_vec",
    "borrow",
    "borrow_mut",
    "set",
    "reset",
    "load",
    "store",
    "call",
    "handle",
    "process",
    "execute",
    "connect",
    "kind",
    "name",
    "id",
    "value",
    "key",
    "data",
    "size",
    "is_some",
    "is_none",
    "is_ok",
    "is_err",
    "map_err",
    "or_else",
    "unwrap_or",
    "unwrap_or_default",
    "unwrap_or_else",
    "as_mut",
];

pub(crate) fn build(name: &str, files: Vec<RawFile>, opts: &RustOptions) -> (Codebase, Vec<Diagnostic>) {
    let r = Resolver::new(&files);
    let mut cb = Codebase::new(name);
    let mut diags = Vec::new();

    for f in &files {
        if let Some(e) = &f.error {
            diags.push(Diagnostic { file: f.path.clone(), message: e.clone() });
        }
    }

    // Modules and items first, so field types are resolved before flows.
    for f in &files {
        cb.add_file(SourceFile {
            path: f.path.clone(),
            language: "rust".into(),
            module: SymbolId::new(f.role.module.join("::")),
            hash: f.hash.clone(),
            lines: f.text_lines,
        });
        for m in &f.modules {
            let id = SymbolId::new(m.path.join("::"));
            let mut s =
                Symbol::new(id.clone(), m.path.last().cloned().unwrap_or_default(), SymbolKind::Module, f.path.clone());
            s.parent = (m.path.len() > 1).then(|| SymbolId::new(m.path[..m.path.len() - 1].join("::")));
            s.span = m.span;
            s.doc = m.doc.clone();
            s.visibility = m.vis.clone();
            s.tags = m.tags.clone();
            cb.add_symbol(s);
            let module = m.path.join("::");
            for u in &m.uses {
                if u.alias == "*" {
                    if let Some(t) = r.resolve_internal(&module, &u.target, None) {
                        cb.add_relation(Relation::new(id.clone(), t, RelationKind::Imports, Confidence::Exact));
                    }
                    continue;
                }
                let (t, c) = r.resolve(&module, &u.target, None);
                if keep_ref(&t) {
                    cb.add_relation(Relation::new(id.clone(), t, RelationKind::Imports, c));
                }
            }
        }
        for it in &f.items {
            let module = it.module.join("::");
            let id = SymbolId::new(format!("{module}::{}", it.name));
            let mut s = Symbol::new(id.clone(), &it.name, it.kind, f.path.clone());
            s.visibility = it.vis.clone();
            s.span = it.span;
            s.signature = it.signature.clone();
            s.doc = it.doc.clone();
            s.generics = it.generics.clone();
            s.tags = it.tags.clone();
            s.members = it.members.iter().map(|m| r.member(&module, m, Some(&id))).collect();
            for m in &s.members {
                for t in &m.refs {
                    let c = if cb_has(&r, t) { Confidence::Exact } else { Confidence::External };
                    cb.add_relation(Relation::new(id.clone(), t.clone(), RelationKind::FieldType, c));
                }
            }
            for st in &it.supertraits {
                let (t, c) = r.resolve(&module, st, None);
                if keep_ref(&t) {
                    cb.add_relation(Relation::new(id.clone(), t, RelationKind::Extends, c));
                }
            }
            r.add_uses(&mut cb, &id, &module, &it.sig_refs, Some(&id));
            let ctx = FlowCtx { module: &module, self_ty: (it.kind == SymbolKind::Trait).then_some(&id) };
            if !it.flow.is_empty() {
                s.flow = r.flow(&ctx, &it.flow, opts);
            }
            for m in &it.methods {
                let mid = id.child(&m.name);
                let ms = r.method_symbol(&mid, &id, m, f, &ctx, opts, &mut cb);
                cb.add_symbol(ms);
            }
            cb.add_symbol(s);
        }
        for imp in &f.impls {
            let module = imp.module.join("::");
            let Some(self_segs) = &imp.self_ty else { continue };
            let (ty, _) = r.resolve(&module, self_segs, None);
            let trait_seg = imp.trait_.as_ref().map(|(segs, disp)| {
                let (t, c) = r.resolve(&module, segs, Some(&ty));
                if keep_ref(&t) || r.internal.contains(&t) {
                    cb.add_relation(Relation::new(ty.clone(), t, RelationKind::Implements, c));
                }
                format!("<{}>", disp.replace("::", "."))
            });
            let ctx = FlowCtx { module: &module, self_ty: Some(&ty) };
            for m in &imp.methods {
                let mid = match &trait_seg {
                    Some(t) => ty.child(t).child(&m.name),
                    None => ty.child(&m.name),
                };
                let mut ms = r.method_symbol(&mid, &ty, m, f, &ctx, opts, &mut cb);
                if let Some((_, disp)) = &imp.trait_ {
                    ms.tags.insert(0, format!("impl {disp}"));
                }
                cb.add_symbol(ms);
            }
        }
    }

    // Aggregate call edges from flows.
    let mut calls: BTreeMap<(SymbolId, SymbolId), Confidence> = BTreeMap::new();
    for s in cb.symbols.values() {
        if let Some(flow) = &s.flow {
            for c in flow.calls() {
                let e = calls.entry((s.id.clone(), c.target.clone())).or_insert(c.confidence);
                *e = (*e).min(c.confidence);
            }
        }
    }
    for ((from, to), c) in calls {
        cb.add_relation(Relation::new(from, to, RelationKind::Calls, c));
    }
    (cb, diags)
}

fn cb_has(r: &Resolver, id: &SymbolId) -> bool {
    r.internal.contains(id)
}

/// Keep a resolved reference as a relation target? Drops std and prelude.
fn keep_ref(id: &SymbolId) -> bool {
    let first = id.as_str().split("::").next().unwrap_or("");
    !STD_ROOTS.contains(&first) && !PRELUDE.contains(&first)
}

struct FlowCtx<'a> {
    module: &'a str,
    self_ty: Option<&'a SymbolId>,
}

/// Lookup tables built from all raw files before any resolution.
struct Resolver {
    crates: BTreeSet<String>,
    /// module id → item name → id (items, submodules).
    items: BTreeMap<String, BTreeMap<String, SymbolId>>,
    /// module id → `use` entries.
    uses: BTreeMap<String, Vec<RawUse>>,
    /// Every internal symbol id that will exist (types, fns, modules, methods).
    internal: BTreeSet<SymbolId>,
    /// type id → field name → raw refs with defining module.
    fields: BTreeMap<SymbolId, (String, BTreeMap<String, Vec<Segs>>)>,
    /// type id → method name → method id (inherent first).
    methods: BTreeMap<SymbolId, BTreeMap<String, SymbolId>>,
    /// method name → ids, for unknown receivers.
    by_name: BTreeMap<String, BTreeSet<SymbolId>>,
    /// type id → implemented trait ids.
    impls: BTreeMap<SymbolId, BTreeSet<SymbolId>>,
    /// trait id → method names (required and provided).
    trait_methods: BTreeMap<SymbolId, BTreeSet<String>>,
}

impl Resolver {
    fn new(files: &[RawFile]) -> Self {
        let mut r = Resolver {
            crates: files.iter().map(|f| f.role.crate_name.clone()).collect(),
            items: BTreeMap::new(),
            uses: BTreeMap::new(),
            internal: BTreeSet::new(),
            fields: BTreeMap::new(),
            methods: BTreeMap::new(),
            by_name: BTreeMap::new(),
            impls: BTreeMap::new(),
            trait_methods: BTreeMap::new(),
        };
        for f in files {
            for m in &f.modules {
                let id = m.path.join("::");
                r.internal.insert(SymbolId::new(&id));
                r.uses.entry(id.clone()).or_default().extend(m.uses.iter().cloned());
                if m.path.len() > 1 {
                    let parent = m.path[..m.path.len() - 1].join("::");
                    r.items
                        .entry(parent)
                        .or_default()
                        .insert(m.path.last().cloned().unwrap_or_default(), SymbolId::new(&id));
                }
            }
            for it in &f.items {
                let module = it.module.join("::");
                let id = SymbolId::new(format!("{module}::{}", it.name));
                r.internal.insert(id.clone());
                r.items.entry(module.clone()).or_default().entry(it.name.clone()).or_insert(id.clone());
                if matches!(it.kind, SymbolKind::Struct | SymbolKind::Union | SymbolKind::Enum) {
                    let f = it.members.iter().map(|m| (m.name.clone(), m.refs.clone())).collect();
                    r.fields.insert(id.clone(), (module.clone(), f));
                }
                if it.kind == SymbolKind::Trait {
                    let names = r.trait_methods.entry(id.clone()).or_default();
                    for m in &it.members {
                        names.insert(m.name.clone());
                    }
                    for m in &it.methods {
                        names.insert(m.name.clone());
                        let mid = id.child(&m.name);
                        r.internal.insert(mid.clone());
                        r.methods.entry(id.clone()).or_default().insert(m.name.clone(), mid.clone());
                        r.by_name.entry(m.name.clone()).or_default().insert(mid);
                    }
                }
            }
        }
        // Impl methods need item tables to resolve self types; do inherent
        // impls before trait impls so inherent methods win name lookups.
        for pass_trait in [false, true] {
            for f in files {
                for imp in &f.impls {
                    if imp.trait_.is_some() != pass_trait {
                        continue;
                    }
                    let module = imp.module.join("::");
                    let Some(segs) = &imp.self_ty else { continue };
                    let (ty, _) = r.resolve(&module, segs, None);
                    let tseg = imp.trait_.as_ref().map(|(segs, disp)| {
                        let (t, _) = r.resolve(&module, segs, Some(&ty));
                        r.impls.entry(ty.clone()).or_default().insert(t);
                        format!("<{}>", disp.replace("::", "."))
                    });
                    for m in &imp.methods {
                        let mid = match &tseg {
                            Some(t) => ty.child(t).child(&m.name),
                            None => ty.child(&m.name),
                        };
                        r.internal.insert(mid.clone());
                        r.methods.entry(ty.clone()).or_default().entry(m.name.clone()).or_insert(mid.clone());
                        r.by_name.entry(m.name.clone()).or_default().insert(mid);
                    }
                }
            }
        }
        r
    }

    /// Resolve `segs` as written in `module`. Returns the canonical id and how
    /// sure we are. Unresolvable paths come back as their literal text with
    /// [`Confidence::External`].
    fn resolve(&self, module: &str, segs: &[String], self_ty: Option<&SymbolId>) -> (SymbolId, Confidence) {
        match self.walk(module, segs, self_ty, 0) {
            Some(id) if self.internal.contains(&id) => (id, Confidence::Exact),
            Some(id) => {
                let root = id.as_str().split("::").next().unwrap_or("");
                if self.crates.contains(root) { (id, Confidence::Inferred) } else { (id, Confidence::External) }
            }
            None => (SymbolId::new(segs.join("::")), Confidence::External),
        }
    }

    fn resolve_internal(&self, module: &str, segs: &[String], self_ty: Option<&SymbolId>) -> Option<SymbolId> {
        self.walk(module, segs, self_ty, 0).filter(|id| self.internal.contains(id))
    }

    fn walk(&self, module: &str, segs: &[String], self_ty: Option<&SymbolId>, depth: u8) -> Option<SymbolId> {
        if depth > 8 || segs.is_empty() {
            return None;
        }
        let first = segs[0].as_str();
        let mut rest = &segs[1..];
        let base: SymbolId = match first {
            "crate" => SymbolId::new(module.split("::").next().unwrap_or(module)),
            "self" => SymbolId::new(module),
            "super" => {
                let mut m = SymbolId::new(module).parent().unwrap_or_else(|| SymbolId::new(module));
                while rest.first().map(String::as_str) == Some("super") {
                    m = m.parent().unwrap_or(m);
                    rest = &rest[1..];
                }
                m
            }
            "Self" => self_ty?.clone(),
            name => {
                if let Some(id) = self.items.get(module).and_then(|m| m.get(name)) {
                    id.clone()
                } else if let Some(u) = self.uses.get(module).and_then(|us| us.iter().find(|u| u.alias == name)) {
                    // Re-resolve the import target, then append the rest.
                    let mut full = u.target.clone();
                    full.extend(rest.iter().cloned());
                    return self
                        .walk(module, &full, self_ty, depth + 1)
                        .or_else(|| Some(SymbolId::new(full.join("::"))));
                } else if let Some(id) = self.glob(module, name, depth) {
                    id
                } else if self.crates.contains(name) {
                    SymbolId::new(name)
                } else {
                    return Some(SymbolId::new(segs.join("::")));
                }
            }
        };
        let mut cur = base;
        for (i, seg) in rest.iter().enumerate() {
            let key = cur.as_str().to_owned();
            if let Some(id) = self.items.get(&key).and_then(|m| m.get(seg)) {
                cur = id.clone();
            } else if let Some(id) = self.methods.get(&cur).and_then(|m| m.get(seg)) {
                cur = id.clone();
            } else if self.uses.get(&key).is_some_and(|us| us.iter().any(|u| &u.alias == seg)) {
                // `pub use` re-export inside an internal module.
                return self.walk(&key, &rest[i..], self_ty, depth + 1);
            } else {
                cur = cur.child(seg);
            }
        }
        Some(cur)
    }

    fn glob(&self, module: &str, name: &str, depth: u8) -> Option<SymbolId> {
        for u in self.uses.get(module)?.iter().filter(|u| u.alias == "*") {
            let target = self.walk(module, &u.target, None, depth + 1)?;
            if let Some(id) = self.items.get(target.as_str()).and_then(|m| m.get(name)) {
                return Some(id.clone());
            }
        }
        None
    }

    /// Resolve a list of raw type refs to kept relation targets.
    fn refs(&self, module: &str, raw: &[Segs], self_ty: Option<&SymbolId>) -> Vec<SymbolId> {
        let mut out = Vec::new();
        for segs in raw {
            // Associated types (`Self::Error`, `T::Output`) are not symbols.
            if segs.len() > 1 && (segs[0] == "Self" || is_generic_param(&segs[0])) {
                continue;
            }
            if segs.len() == 1
                && (PRELUDE.contains(&segs[0].as_str()) || is_generic_param(&segs[0]))
                && !self.local(module, &segs[0])
            {
                continue;
            }
            let (id, _) = self.resolve(module, segs, self_ty);
            if (keep_ref(&id) || self.internal.contains(&id)) && Some(&id) != self_ty && !out.contains(&id) {
                out.push(id);
            }
        }
        out
    }

    /// `true` if `name` is defined or imported in `module`.
    fn local(&self, module: &str, name: &str) -> bool {
        self.items.get(module).is_some_and(|m| m.contains_key(name))
            || self.uses.get(module).is_some_and(|us| us.iter().any(|u| u.alias == name))
    }

    fn member(&self, module: &str, m: &RawMember, owner: Option<&SymbolId>) -> Member {
        Member {
            name: m.name.clone(),
            kind: m.kind,
            ty: m.ty.clone(),
            visibility: m.vis.clone(),
            span: m.span,
            refs: self.refs(module, &m.refs, owner),
        }
    }

    fn add_uses(&self, cb: &mut Codebase, from: &SymbolId, module: &str, raw: &[Segs], self_ty: Option<&SymbolId>) {
        for t in self.refs(module, raw, self_ty) {
            let c = if self.internal.contains(&t) { Confidence::Exact } else { Confidence::External };
            cb.add_relation(Relation::new(from.clone(), t, RelationKind::Uses, c));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn method_symbol(
        &self,
        id: &SymbolId,
        parent: &SymbolId,
        m: &RawFn,
        f: &RawFile,
        ctx: &FlowCtx<'_>,
        opts: &RustOptions,
        cb: &mut Codebase,
    ) -> Symbol {
        let mut s = Symbol::new(id.clone(), &m.name, SymbolKind::Method, f.path.clone());
        s.parent = Some(parent.clone());
        s.visibility = m.vis.clone();
        s.span = m.span;
        s.signature = Some(m.signature.clone());
        s.doc = m.doc.clone();
        s.generics = m.generics.clone();
        s.tags = m.tags.clone();
        self.add_uses(cb, id, ctx.module, &m.sig_refs, ctx.self_ty);
        if !m.flow.is_empty() {
            s.flow = self.flow(ctx, &m.flow, opts);
        }
        s
    }

    fn flow(&self, ctx: &FlowCtx<'_>, raw: &[RawStep], opts: &RustOptions) -> Option<Flow> {
        let mut steps = self.steps(ctx, raw, opts);
        // A trailing top-level `return` is just the end of the function.
        if matches!(steps.last(), Some(Step::Return(_))) {
            steps.pop();
        }
        // A flow that only returns says nothing about calls.
        if steps.iter().all(|s| matches!(s, Step::Return(_))) {
            return None;
        }
        Some(Flow::new(steps))
    }

    fn steps(&self, ctx: &FlowCtx<'_>, raw: &[RawStep], opts: &RustOptions) -> Vec<Step> {
        let mut out = Vec::new();
        for s in raw {
            match s {
                RawStep::Call(c) => {
                    if let Some(call) = self.call(ctx, c, opts) {
                        out.push(Step::Call(call));
                    }
                }
                RawStep::Branch(raw_arms) => {
                    let first_label = raw_arms.first().map(|(l, _)| l.clone()).unwrap_or_default();
                    let all: Vec<(usize, Arm)> = raw_arms
                        .iter()
                        .enumerate()
                        .map(|(i, (label, steps))| {
                            (i, Arm { label: label.clone(), steps: self.steps(ctx, steps, opts) })
                        })
                        .filter(|(_, a)| !a.steps.is_empty())
                        .collect();
                    match all.len() {
                        0 => {}
                        1 => {
                            // One arm left: say which condition guards it.
                            let (i, arm) = all.into_iter().next().expect("one arm");
                            let label = if i > 0 && arm.label.is_empty() {
                                format!("not {first_label}")
                            } else {
                                arm.label.trim_start_matches("if ").to_owned()
                            };
                            out.push(Step::Optional { label, body: arm.steps });
                        }
                        _ => out.push(Step::Branch { arms: all.into_iter().map(|(_, a)| a).collect() }),
                    }
                }
                RawStep::Parallel(arms) => {
                    let arms = self.arms(ctx, arms, opts);
                    if arms.iter().any(|a| !a.steps.is_empty()) {
                        out.push(Step::Parallel { arms });
                    }
                }
                RawStep::Loop(label, body) => {
                    let body = self.steps(ctx, body, opts);
                    if !body.is_empty() {
                        out.push(Step::Loop { label: label.clone(), body });
                    }
                }
                RawStep::Optional(label, body) => {
                    let body = self.steps(ctx, body, opts);
                    if !body.is_empty() {
                        out.push(Step::Optional { label: label.clone(), body });
                    }
                }
                RawStep::Return(label, line) => out.push(Step::Return(Exit { label: label.clone(), line: *line })),
            }
        }
        out
    }

    fn arms(&self, ctx: &FlowCtx<'_>, arms: &[(String, Vec<RawStep>)], opts: &RustOptions) -> Vec<Arm> {
        arms.iter()
            .map(|(label, steps)| Arm { label: label.clone(), steps: self.steps(ctx, steps, opts) })
            .filter(|a| !a.steps.is_empty())
            .collect()
    }

    fn call(&self, ctx: &FlowCtx<'_>, c: &RawCall, opts: &RustOptions) -> Option<Call> {
        let (target, confidence) = match &c.callee {
            Callee::Path(segs) => {
                // `drop(x)`, `Vec::new()`, `String::from(..)`: prelude, not code.
                if PRELUDE.contains(&segs[0].as_str()) && !self.local(ctx.module, &segs[0]) {
                    return None;
                }
                let (id, c) = self.resolve(ctx.module, segs, ctx.self_ty);
                // A bare name that is neither defined, imported nor a crate is
                // a local closure or function pointer: not a symbol.
                if segs.len() == 1 && c == Confidence::External && !self.local(ctx.module, &segs[0]) {
                    return None;
                }
                (id, c)
            }
            Callee::Method { recv, name } => self.method(ctx, recv, name)?,
        };
        let keep = match confidence {
            Confidence::Exact | Confidence::Inferred => true,
            Confidence::External => match opts.external_calls {
                ExternalCalls::All => true,
                ExternalCalls::NonStd => {
                    let root = target.as_str().split("::").next().unwrap_or("");
                    // External roots that look like crates (lower-case);
                    // unresolved type names are not dependencies.
                    keep_ref(&target)
                        && root.starts_with(|c: char| c.is_ascii_lowercase())
                        && !self.crates.contains(root)
                }
                ExternalCalls::None => false,
            },
        };
        keep.then(|| Call {
            target,
            label: c.label.clone(),
            kind: c.kind,
            confidence,
            awaited: c.awaited,
            fallible: c.fallible,
            line: c.line,
        })
    }

    /// Resolve a method call. Returns `None` only when the call should be
    /// dropped outright.
    fn method(&self, ctx: &FlowCtx<'_>, recv: &Recv, name: &str) -> Option<(SymbolId, Confidence)> {
        let ty = match recv {
            Recv::SelfValue => ctx.self_ty.cloned(),
            Recv::SelfField(field) => {
                let found = ctx.self_ty.and_then(|ty| self.fields.get(ty)).and_then(|(m, f)| Some((m, f.get(field)?)));
                match found {
                    Some((module, refs)) => self.receiver_type(module, refs, None),
                    None => return Some(self.by_name_only(name)),
                }
            }
            Recv::Typed(refs) => self.receiver_type(ctx.module, refs, ctx.self_ty),
            Recv::Untyped => return Some(self.by_name_only(name)),
            Recv::Unknown => None,
        };
        let Some(ty) = ty else { return Some((SymbolId::new(format!("?::{name}")), Confidence::External)) };
        if let Some(id) = self.methods.get(&ty).and_then(|m| m.get(name)) {
            return Some((id.clone(), Confidence::Exact));
        }
        if self.trait_methods.get(&ty).is_some_and(|n| n.contains(name)) {
            return Some((ty.child(name), Confidence::Exact));
        }
        for tr in self.impls.get(&ty).into_iter().flatten() {
            if self.trait_methods.get(tr).is_some_and(|n| n.contains(name)) {
                return Some((tr.child(name), Confidence::Exact));
            }
        }
        // Known type, method not defined in the codebase: a derived or std
        // trait method on an internal type, or a dependency's method.
        Some((ty.child(name), Confidence::External))
    }

    /// The type a method is called on, from the declared type's paths
    /// (outermost first). Smart pointers are looked through; any other
    /// wrapper (`Vec`, `Option`, `Mutex`, ...) *is* the receiver.
    fn receiver_type(&self, module: &str, refs: &[Segs], self_ty: Option<&SymbolId>) -> Option<SymbolId> {
        const DEREF: &[&str] =
            &["Box", "Arc", "Rc", "Cow", "Pin", "Ref", "RefMut", "MutexGuard", "RwLockReadGuard", "RwLockWriteGuard"];
        for segs in refs {
            let last = segs.last()?.as_str();
            if segs.len() == 1 && is_generic_param(last) {
                return None;
            }
            let (id, _) = self.resolve(module, segs, self_ty);
            if DEREF.contains(&last) && !self.internal.contains(&id) {
                continue;
            }
            return Some(id);
        }
        None
    }

    /// Unknown receiver: accept a unique, distinctive internal method name.
    fn by_name_only(&self, name: &str) -> (SymbolId, Confidence) {
        if !COMMON_METHODS.contains(&name) {
            if let Some(ids) = self.by_name.get(name) {
                if ids.len() == 1 {
                    let id = ids.iter().next().cloned().unwrap_or_else(|| SymbolId::new(name));
                    return (id, Confidence::Inferred);
                }
            }
        }
        (SymbolId::new(format!("?::{name}")), Confidence::External)
    }
}

/// Single upper-case letters (and `T1`-style names) are almost always generic
/// parameters.
fn is_generic_param(s: &str) -> bool {
    let mut cs = s.chars();
    matches!(cs.next(), Some(c) if c.is_ascii_uppercase()) && cs.all(|c| c.is_ascii_digit())
}
