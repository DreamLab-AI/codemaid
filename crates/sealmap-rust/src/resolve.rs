//! Pass 2: resolve raw paths against the whole workspace and build the model.

use std::collections::{BTreeMap, BTreeSet};

use sealmap_extract::{aggregate_calls, ids, lower};
use sealmap_model::{
    Call, Codebase, Confidence, Flow, Member, Package, Relation, RelationKind, SourceFile, Suffix, Symbol, SymbolId,
    SymbolKind,
};

use crate::raw::*;
use crate::{Diagnostic, RustOptions};

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
            module: module_sym(&f.role.module),
            hash: f.hash.clone(),
            lines: f.text_lines,
        });
        for m in &f.modules {
            let id = module_sym(&m.path);
            let mut s =
                Symbol::new(id.clone(), m.path.last().cloned().unwrap_or_default(), SymbolKind::Module, f.path.clone());
            s.span = m.span;
            s.sig_hash = m.sig_hash;
            s.body_hash = m.body_hash;
            s.doc = m.doc.clone();
            s.visibility = m.vis.clone();
            s.tags = m.tags.clone();
            cb.add_symbol(s);
            for u in &m.uses {
                if u.alias == "*" {
                    if let Some(t) = r.resolve_internal(&id, &u.target, None) {
                        cb.add_relation(Relation::new(id.clone(), t, RelationKind::Imports, Confidence::Exact));
                    }
                    continue;
                }
                let (t, c) = r.resolve(&id, &u.target, None, Ns::Type);
                if keep_ref(&t) {
                    cb.add_relation(Relation::new(id.clone(), t, RelationKind::Imports, c));
                }
            }
        }
        for it in &f.items {
            let module = module_sym(&it.module);
            let id = ids::item_id(&module, it.kind, &it.name);
            let mut s = Symbol::new(id.clone(), &it.name, it.kind, f.path.clone());
            s.visibility = it.vis.clone();
            s.span = it.span;
            s.sig_hash = it.sig_hash;
            s.body_hash = it.body_hash;
            s.signature = it.signature.clone();
            s.doc = it.doc.clone();
            s.generics = it.generics.clone();
            s.tags = it.tags.clone();
            let scope = InScope::of(&it.type_params);
            s.members =
                it.members.iter().map(|m| r.member(&module, m, Some(&id), scope.with(&m.type_params))).collect();
            for m in &s.members {
                for t in &m.refs {
                    let c = if r.internal.contains(t) { Confidence::Exact } else { Confidence::External };
                    cb.add_relation(Relation::new(id.clone(), t.clone(), RelationKind::FieldType, c));
                }
            }
            for st in &it.supertraits {
                let (t, c) = r.resolve(&module, st, None, Ns::Type);
                if keep_ref(&t) {
                    cb.add_relation(Relation::new(id.clone(), t, RelationKind::Extends, c));
                }
            }
            r.add_uses(&mut cb, &id, &module, &it.sig_refs, Some(&id), scope);
            let ctx =
                FlowCtx { module: &module, self_ty: (it.kind == SymbolKind::Trait).then_some(&id), params: scope };
            if !it.flow.is_empty() {
                s.flow = r.flow(&ctx, &it.flow, opts);
            }
            for m in &it.methods {
                let mid = ids::method_id(&id, None, &m.name);
                let ms = r.method_symbol(&mid, &id, m, f, &ctx, opts, &mut cb);
                cb.add_symbol(ms);
            }
            cb.add_symbol(s);
        }
        for imp in &f.impls {
            let module = module_sym(&imp.module);
            let Some(self_segs) = &imp.self_ty else { continue };
            let (ty, _) = r.resolve(&module, self_segs, None, Ns::Type);
            if let Some((segs, _)) = &imp.trait_ {
                let (t, c) = r.resolve(&module, segs, Some(&ty), Ns::Type);
                if keep_ref(&t) || r.internal.contains(&t) {
                    cb.add_relation(Relation::new(ty.clone(), t, RelationKind::Implements, c));
                }
            }
            let ctx = FlowCtx { module: &module, self_ty: Some(&ty), params: InScope::of(&imp.type_params) };
            for m in &imp.methods {
                let mid = impl_method(&module, &ty, imp, &m.name);
                let mut ms = r.method_symbol(&mid, &ty, m, f, &ctx, opts, &mut cb);
                if let Some((_, disp)) = &imp.trait_ {
                    ms.tags.insert(0, format!("impl {disp}"));
                }
                cb.add_symbol(ms);
            }
        }
    }

    aggregate_calls(&mut cb);
    (cb, diags)
}

/// The id of the module at `segments` (crate name first), in the `cargo`
/// package named by the crate.
fn module_sym(segments: &[String]) -> SymbolId {
    let (krate, rest) = segments.split_first().map_or(("", &[][..]), |(k, r)| (k.as_str(), r));
    ids::module_id(&crate_package(krate), rest)
}

fn crate_package(krate: &str) -> Package {
    ids::package("cargo", krate)
}

/// The id of method `name` from impl block `imp` whose self type resolved
/// to `ty`.
fn impl_method(module: &SymbolId, ty: &SymbolId, imp: &RawImpl, name: &str) -> SymbolId {
    let self_ty = imp.self_ty.as_ref().map(|s| s.join("::")).unwrap_or_default();
    let trait_ = imp.trait_.as_ref().map(|(_, disp)| disp.as_str());
    ids::impl_method_id(module, ty, &self_ty, trait_, name)
}

/// The id of member `name` of `ty` when the codebase does not define it: a
/// method under a global type, or the path extended by the name.
fn undefined_member(ty: &SymbolId, name: &str) -> SymbolId {
    ids::method_id(ty, None, name)
}

/// Keep a resolved reference as a relation target? Drops std and prelude.
fn keep_ref(id: &SymbolId) -> bool {
    let root = id.root();
    let first = root.as_deref().unwrap_or("?");
    !STD_ROOTS.contains(&first) && !PRELUDE.contains(&first)
}

/// Which namespace a path's final segment is looked up in first. Rust keeps
/// types (and modules) apart from values (functions, constants), so a module
/// `config` and a function `config` in one module are both reachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ns {
    Type,
    Value,
}

/// The definitions one name has in one module, per namespace.
#[derive(Debug, Clone, Default)]
struct Slots {
    ty: Option<SymbolId>,
    value: Option<SymbolId>,
}

impl Slots {
    fn insert(&mut self, kind: SymbolKind, id: SymbolId) {
        let slot = match kind {
            SymbolKind::Function | SymbolKind::Method | SymbolKind::Const | SymbolKind::Static => &mut self.value,
            SymbolKind::Macro => return,
            _ => &mut self.ty,
        };
        if slot.is_none() {
            *slot = Some(id);
        }
    }

    fn get(&self, ns: Ns) -> Option<&SymbolId> {
        match ns {
            Ns::Type => self.ty.as_ref().or(self.value.as_ref()),
            Ns::Value => self.value.as_ref().or(self.ty.as_ref()),
        }
    }
}

struct FlowCtx<'a> {
    module: &'a SymbolId,
    self_ty: Option<&'a SymbolId>,
    /// Generic parameters in scope: the item's own, or the impl's or
    /// trait's (a method's own are added by [`Resolver::method_symbol`]).
    params: InScope<'a>,
}

/// The generic type and const parameters in scope at one point, as declared:
/// an item's own, or an impl's or trait's plus one method's. A path whose
/// first segment is one of them names a parameter, never a same-named item,
/// whatever its spelling (`T`, `V2`, `Store`).
///
/// Scope is always known: every path the resolver sees comes from an item,
/// impl or trait whose generics the collector recorded, and items nested in
/// function bodies are not collected, so there is no name-shape fallback.
#[derive(Debug, Clone, Copy, Default)]
struct InScope<'a> {
    enclosing: &'a [String],
    own: &'a [String],
}

impl<'a> InScope<'a> {
    /// The parameters of a top-level item, impl or trait.
    fn of(own: &'a [String]) -> Self {
        InScope { enclosing: &[], own }
    }

    /// This scope plus a member's own parameters (a method in an impl or
    /// trait). Generics nest one level deep: members never nest further.
    fn with(self, own: &'a [String]) -> Self {
        debug_assert!(self.enclosing.is_empty(), "generic scopes nest one level");
        InScope { enclosing: self.own, own }
    }

    fn contains(&self, name: &str) -> bool {
        self.own.iter().chain(self.enclosing).any(|p| p == name)
    }

    /// Does `segs` start at a generic parameter (`T`, `T::Output`)?
    fn heads(&self, segs: &[String]) -> bool {
        segs.first().is_some_and(|s| self.contains(s))
    }
}

/// A data type's fields as [`Resolver::fields`] keeps them.
struct Fields {
    module: SymbolId,
    params: Vec<String>,
    refs: BTreeMap<String, Vec<Segs>>,
}

impl Fields {
    /// The module, generic scope and raw refs of field `name`.
    fn field(&self, name: &str) -> Option<(&SymbolId, InScope<'_>, &[Segs])> {
        Some((&self.module, InScope::of(&self.params), self.refs.get(name)?))
    }
}

/// Lookup tables built from all raw files before any resolution.
struct Resolver {
    crates: BTreeSet<String>,
    /// module id → item name → definitions (items, submodules).
    items: BTreeMap<SymbolId, BTreeMap<String, Slots>>,
    /// module id → `use` entries.
    uses: BTreeMap<SymbolId, Vec<RawUse>>,
    /// Every internal symbol id that will exist (types, fns, modules, methods).
    internal: BTreeSet<SymbolId>,
    /// type id → its defining module, its generic parameters and, per field
    /// name, the field's raw refs.
    fields: BTreeMap<SymbolId, Fields>,
    /// type id → method name → method id (inherent first).
    methods: BTreeMap<SymbolId, BTreeMap<String, SymbolId>>,
    /// method name → ids, for unknown receivers.
    by_name: BTreeMap<String, BTreeSet<SymbolId>>,
    /// type id → implemented trait ids.
    impls: BTreeMap<SymbolId, BTreeSet<SymbolId>>,
    /// trait id → method names (required and provided).
    trait_methods: BTreeMap<SymbolId, BTreeSet<String>>,
    /// module id → internal modules it glob-imports (`use a::b::*`), in
    /// source order. Resolved once, without consulting globs, so a name
    /// lookup never re-enters glob resolution.
    globs: BTreeMap<SymbolId, Vec<SymbolId>>,
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
            globs: BTreeMap::new(),
        };
        for f in files {
            for m in &f.modules {
                let id = module_sym(&m.path);
                r.internal.insert(id.clone());
                r.uses.entry(id.clone()).or_default().extend(m.uses.iter().cloned());
                if let Some(parent) = id.parent() {
                    let name = m.path.last().cloned().unwrap_or_default();
                    r.items.entry(parent).or_default().entry(name).or_default().insert(SymbolKind::Module, id);
                }
            }
            for it in &f.items {
                let module = module_sym(&it.module);
                let id = ids::item_id(&module, it.kind, &it.name);
                r.internal.insert(id.clone());
                r.items
                    .entry(module.clone())
                    .or_default()
                    .entry(it.name.clone())
                    .or_default()
                    .insert(it.kind, id.clone());
                if matches!(it.kind, SymbolKind::Struct | SymbolKind::Union | SymbolKind::Enum) {
                    let refs = it.members.iter().map(|m| (m.name.clone(), m.refs.clone())).collect();
                    r.fields
                        .insert(id.clone(), Fields { module: module.clone(), params: it.type_params.clone(), refs });
                }
                if it.kind == SymbolKind::Trait {
                    let names = r.trait_methods.entry(id.clone()).or_default();
                    for m in &it.members {
                        names.insert(m.name.clone());
                    }
                    for m in &it.methods {
                        names.insert(m.name.clone());
                        let mid = ids::method_id(&id, None, &m.name);
                        r.internal.insert(mid.clone());
                        r.methods.entry(id.clone()).or_default().insert(m.name.clone(), mid.clone());
                        r.by_name.entry(m.name.clone()).or_default().insert(mid);
                    }
                }
            }
        }
        // Glob targets are resolved up front into a table, so a name lookup
        // is a bounded graph search (`glob`) instead of a re-entrant `walk`,
        // which was exponential in the number of globs per module. Round 0
        // resolves targets without globs; round 1 lets a glob target itself
        // be found through round-0 globs (`use super::*; use inner::*;`).
        for round in 0..2 {
            let mut globs: BTreeMap<SymbolId, Vec<SymbolId>> = BTreeMap::new();
            for (module, uses) in &r.uses {
                for u in uses.iter().filter(|u| u.alias == "*") {
                    if let Some(t) = r.walk(module, &u.target, None, Ns::Type, 0, round > 0) {
                        if r.items.contains_key(&t) && !globs.get(module).is_some_and(|v| v.contains(&t)) {
                            globs.entry(module.clone()).or_default().push(t);
                        }
                    }
                }
            }
            r.globs = globs;
        }
        // Impl methods need item tables to resolve self types; do inherent
        // impls before trait impls so inherent methods win name lookups.
        for pass_trait in [false, true] {
            for f in files {
                for imp in &f.impls {
                    if imp.trait_.is_some() != pass_trait {
                        continue;
                    }
                    let module = module_sym(&imp.module);
                    let Some(segs) = &imp.self_ty else { continue };
                    let (ty, _) = r.resolve(&module, segs, None, Ns::Type);
                    if let Some((segs, _)) = &imp.trait_ {
                        let (t, _) = r.resolve(&module, segs, Some(&ty), Ns::Type);
                        r.impls.entry(ty.clone()).or_default().insert(t);
                    }
                    for m in &imp.methods {
                        let mid = impl_method(&module, &ty, imp, &m.name);
                        r.internal.insert(mid.clone());
                        r.methods.entry(ty.clone()).or_default().entry(m.name.clone()).or_insert(mid.clone());
                        r.by_name.entry(m.name.clone()).or_default().insert(mid);
                    }
                }
            }
        }
        r
    }

    /// Resolve `segs` as written in `module`, looking the final segment up
    /// in `ns` first. Returns the canonical id and how sure we are.
    /// Unresolvable paths come back as a path id with
    /// [`Confidence::External`].
    fn resolve(
        &self,
        module: &SymbolId,
        segs: &[String],
        self_ty: Option<&SymbolId>,
        ns: Ns,
    ) -> (SymbolId, Confidence) {
        match self.walk(module, segs, self_ty, ns, 0, true) {
            Some(id) if self.internal.contains(&id) => (id, Confidence::Exact),
            Some(id) => {
                let in_workspace = id.root().is_some_and(|root| self.crates.contains(root.as_ref()));
                (id, if in_workspace { Confidence::Inferred } else { Confidence::External })
            }
            None => (ids::path_id(segs).unwrap_or_else(|| ids::unresolved_method_id("")), Confidence::External),
        }
    }

    /// [`Self::resolve`] for a path written where the generic parameters
    /// `params` are in scope. A path through a parameter (`T::new`,
    /// `Store::Key`) is a bare external path, never walked into an item that
    /// happens to share the parameter's name.
    fn resolve_in(
        &self,
        module: &SymbolId,
        segs: &[String],
        self_ty: Option<&SymbolId>,
        ns: Ns,
        params: InScope<'_>,
    ) -> (SymbolId, Confidence) {
        if params.heads(segs) {
            return (ids::path_id(segs).unwrap_or_else(|| ids::unresolved_method_id("")), Confidence::External);
        }
        self.resolve(module, segs, self_ty, ns)
    }

    fn resolve_internal(&self, module: &SymbolId, segs: &[String], self_ty: Option<&SymbolId>) -> Option<SymbolId> {
        self.walk(module, segs, self_ty, Ns::Type, 0, true).filter(|id| self.internal.contains(id))
    }

    /// The definition of `name` in module `module`, `ns` first.
    fn item(&self, module: &SymbolId, name: &str, ns: Ns) -> Option<&SymbolId> {
        self.items.get(module).and_then(|m| m.get(name)).and_then(|s| s.get(ns))
    }

    /// `use_globs` is false only while the glob table itself is being built.
    fn walk(
        &self,
        module: &SymbolId,
        segs: &[String],
        self_ty: Option<&SymbolId>,
        ns: Ns,
        depth: u8,
        use_globs: bool,
    ) -> Option<SymbolId> {
        if depth > 8 || segs.is_empty() {
            return None;
        }
        let first = segs[0].as_str();
        let mut rest = &segs[1..];
        // Intermediate segments name modules and types; only the last one is
        // looked up in `ns` first.
        let ns_at = |i: usize| if i + 1 == segs.len() { ns } else { Ns::Type };
        let base: SymbolId = match first {
            "crate" => module_sym(&[module.root().unwrap_or_default().into_owned()]),
            "self" => module.clone(),
            "super" => {
                let mut m = module.parent().unwrap_or_else(|| module.clone());
                while rest.first().map(String::as_str) == Some("super") {
                    m = m.parent().unwrap_or(m);
                    rest = &rest[1..];
                }
                m
            }
            "Self" => self_ty?.clone(),
            name => {
                if let Some(id) = self.item(module, name, ns_at(0)) {
                    id.clone()
                } else if let Some(u) = self
                    .uses
                    .get(module)
                    .and_then(|us| us.iter().find(|u| u.alias == name && u.target.as_slice() != [name]))
                {
                    // Re-resolve the import target, then append the rest.
                    // (`use foo;` — target == alias — would only re-enter
                    // itself; it falls through to the crate check below.)
                    let mut full = u.target.clone();
                    full.extend(rest.iter().cloned());
                    return self.walk(module, &full, self_ty, ns, depth + 1, use_globs).or_else(|| ids::path_id(&full));
                } else if let Some(id) = use_globs.then(|| self.glob(module, name, ns_at(0))).flatten() {
                    id
                } else if self.crates.contains(name) {
                    module_sym(&[name.to_owned()])
                } else {
                    return ids::path_id(segs);
                }
            }
        };
        let offset = segs.len() - rest.len();
        let mut cur = base;
        for (i, seg) in rest.iter().enumerate() {
            if let Some(id) = self.item(&cur, seg, ns_at(offset + i)) {
                cur = id.clone();
            } else if let Some(id) = self.methods.get(&cur).and_then(|m| m.get(seg)) {
                cur = id.clone();
            } else if self.uses.get(&cur).is_some_and(|us| us.iter().any(|u| &u.alias == seg)) {
                // `pub use` re-export inside an internal module.
                return self.walk(&cur, &rest[i..], self_ty, ns, depth + 1, use_globs);
            } else if i + 1 == rest.len()
                && self.internal.contains(&cur)
                && cur.last().is_some_and(|d| *d.suffix() == Suffix::Type)
            {
                // A member of an internal type that the code does not define
                // (`Fingerprint::default()` from a derive, an associated item
                // from a trait): it still belongs to that type, so it is
                // `Type#member().` (or `Type#Member#` in type position), never
                // a kind-free path that would cut it loose from the type.
                cur = match ns {
                    Ns::Value => ids::method_id(&cur, None, seg),
                    Ns::Type => ids::item_id(&cur, SymbolKind::TypeAlias, seg),
                };
            } else {
                cur = cur.extend_path(seg);
            }
        }
        Some(cur)
    }

    /// Find `name` through `module`'s glob imports, following glob chains
    /// (`pub use inner::*` re-exports) breadth-first. Each module is visited
    /// at most once, so cyclic globs (`a: use b::*`, `b: use a::*`) are
    /// harmless and the cost is linear in the glob graph.
    fn glob(&self, module: &SymbolId, name: &str, ns: Ns) -> Option<SymbolId> {
        let mut queue: std::collections::VecDeque<&SymbolId> = self.globs.get(module)?.iter().collect();
        let mut seen: BTreeSet<&SymbolId> = BTreeSet::from([module]);
        while let Some(m) = queue.pop_front() {
            if !seen.insert(m) {
                continue;
            }
            if let Some(id) = self.item(m, name, ns) {
                return Some(id.clone());
            }
            if let Some(next) = self.globs.get(m) {
                queue.extend(next.iter());
            }
        }
        None
    }

    /// Resolve a list of raw type refs to kept relation targets.
    fn refs(&self, module: &SymbolId, raw: &[Segs], self_ty: Option<&SymbolId>, params: InScope<'_>) -> Vec<SymbolId> {
        let mut out = Vec::new();
        for segs in raw {
            // Generic parameters and their associated types (`T`,
            // `T::Output`, `Self::Error`) are not symbols.
            if params.heads(segs) || (segs.len() > 1 && segs[0] == "Self") {
                continue;
            }
            if segs.len() == 1 && PRELUDE.contains(&segs[0].as_str()) && !self.local(module, &segs[0]) {
                continue;
            }
            let (id, _) = self.resolve(module, segs, self_ty, Ns::Type);
            if (keep_ref(&id) || self.internal.contains(&id)) && Some(&id) != self_ty && !out.contains(&id) {
                out.push(id);
            }
        }
        out
    }

    /// `true` if `name` is defined or imported in `module`.
    fn local(&self, module: &SymbolId, name: &str) -> bool {
        self.items.get(module).is_some_and(|m| m.contains_key(name))
            || self.uses.get(module).is_some_and(|us| us.iter().any(|u| u.alias == name))
    }

    fn member(&self, module: &SymbolId, m: &RawMember, owner: Option<&SymbolId>, params: InScope<'_>) -> Member {
        Member {
            name: m.name.clone(),
            kind: m.kind,
            ty: m.ty.clone(),
            visibility: m.vis.clone(),
            span: m.span,
            refs: self.refs(module, &m.refs, owner, params),
        }
    }

    fn add_uses(
        &self,
        cb: &mut Codebase,
        from: &SymbolId,
        module: &SymbolId,
        raw: &[Segs],
        self_ty: Option<&SymbolId>,
        params: InScope<'_>,
    ) {
        for t in self.refs(module, raw, self_ty, params) {
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
        s.sig_hash = m.sig_hash;
        s.body_hash = m.body_hash;
        s.signature = Some(m.signature.clone());
        s.doc = m.doc.clone();
        s.generics = m.generics.clone();
        s.tags = m.tags.clone();
        let ctx = FlowCtx { params: ctx.params.with(&m.type_params), ..*ctx };
        self.add_uses(cb, id, ctx.module, &m.sig_refs, ctx.self_ty, ctx.params);
        if !m.flow.is_empty() {
            s.flow = self.flow(&ctx, &m.flow, opts);
        }
        s
    }

    fn flow(&self, ctx: &FlowCtx<'_>, raw: &[RawStep], opts: &RustOptions) -> Option<Flow> {
        lower::lower_flow(raw, &mut |c: &RawCall| self.call(ctx, c, opts))
    }

    fn call(&self, ctx: &FlowCtx<'_>, c: &RawCall, opts: &RustOptions) -> Option<Call> {
        let (target, confidence) = match &c.callee {
            Callee::Path(segs) => {
                match segs[0].as_str() {
                    // `Self` and `self` are on the prelude list for type
                    // references, but in a call path they are not the
                    // prelude. `Self::f(..)` names the enclosing impl's self
                    // type (the trait, in a provided method); outside both it
                    // names nothing. `self::f(..)` is the current module.
                    "Self" if ctx.self_ty.is_none() => return None,
                    "Self" | "self" => {}
                    // `drop(x)`, `Vec::new()`, `String::from(..)`: prelude, not code.
                    first if PRELUDE.contains(&first) && !self.local(ctx.module, first) => return None,
                    _ => {}
                }
                let (id, c) = self.resolve_in(ctx.module, segs, ctx.self_ty, Ns::Value, ctx.params);
                // A bare name that is neither defined, imported nor a crate is
                // a local closure or function pointer: not a symbol.
                if segs.len() == 1 && c == Confidence::External && !self.local(ctx.module, &segs[0]) {
                    return None;
                }
                (id, c)
            }
            Callee::Method { recv, name } => self.method(ctx, recv, name)?,
        };
        let keep = opts.external_calls.keeps(confidence, || {
            let root = target.root();
            let root = root.as_deref().unwrap_or("");
            // External roots that look like crates (lower-case); unresolved
            // type names are not dependencies.
            keep_ref(&target) && root.starts_with(|c: char| c.is_ascii_lowercase()) && !self.crates.contains(root)
        });
        keep.then(|| c.to_call(target, confidence))
    }

    /// Resolve a method call. Returns `None` only when the call should be
    /// dropped outright.
    fn method(&self, ctx: &FlowCtx<'_>, recv: &Recv, name: &str) -> Option<(SymbolId, Confidence)> {
        let ty = match recv {
            Recv::SelfValue => ctx.self_ty.cloned(),
            Recv::SelfField(field) => match ctx.self_ty.and_then(|ty| self.fields.get(ty)?.field(field)) {
                Some((module, params, refs)) => self.receiver_type(module, refs, None, params),
                None => return Some(self.by_name_only(name)),
            },
            Recv::Typed(refs) => self.receiver_type(ctx.module, refs, ctx.self_ty, ctx.params),
            Recv::Untyped => return Some(self.by_name_only(name)),
            Recv::Derived(origin) => {
                return Some(if self.internal_origin(ctx, origin) {
                    self.by_name_only(name)
                } else {
                    (ids::unresolved_method_id(name), Confidence::External)
                });
            }
            Recv::Returned(_) | Recv::Computed(_) | Recv::Unknown => None,
        };
        let Some(ty) = ty else { return Some((ids::unresolved_method_id(name), Confidence::External)) };
        if let Some(id) = self.methods.get(&ty).and_then(|m| m.get(name)) {
            return Some((id.clone(), Confidence::Exact));
        }
        if self.trait_methods.get(&ty).is_some_and(|n| n.contains(name)) {
            return Some((ids::method_id(&ty, None, name), Confidence::Exact));
        }
        for tr in self.impls.get(&ty).into_iter().flatten() {
            if self.trait_methods.get(tr).is_some_and(|n| n.contains(name)) {
                return Some((ids::method_id(tr, None, name), Confidence::Exact));
            }
        }
        // Known type, method not defined in the codebase: a derived or std
        // trait method on an internal type, or a dependency's method.
        Some((undefined_member(&ty, name), Confidence::External))
    }

    /// Does a [`Recv::Derived`] value come from code in the analysed
    /// codebase? `self`, a field or typed value whose type mentions an
    /// internal type, an internal function's result, or a plain variable of
    /// unevident type do; a value of `std` or third-party type, a third-party
    /// function's result and anything unknown do not.
    fn internal_origin(&self, ctx: &FlowCtx<'_>, origin: &Recv) -> bool {
        let any_internal = |module: &SymbolId, refs: &[Segs], self_ty: Option<&SymbolId>, params: InScope<'_>| {
            refs.iter().any(|segs| self.internal.contains(&self.resolve_in(module, segs, self_ty, Ns::Type, params).0))
        };
        match origin {
            Recv::SelfValue | Recv::Untyped => true,
            Recv::SelfField(field) => match ctx.self_ty.and_then(|ty| self.fields.get(ty)?.field(field)) {
                Some((module, params, refs)) => any_internal(module, refs, None, params),
                None => true,
            },
            Recv::Typed(refs) => any_internal(ctx.module, refs, ctx.self_ty, ctx.params),
            Recv::Returned(segs) => {
                self.internal.contains(&self.resolve_in(ctx.module, segs, ctx.self_ty, Ns::Value, ctx.params).0)
            }
            Recv::Derived(inner) | Recv::Computed(inner) => self.internal_origin(ctx, inner),
            Recv::Unknown => false,
        }
    }

    /// The type a method is called on, from the declared type's paths
    /// (outermost first). Smart pointers are looked through; any other
    /// wrapper (`Vec`, `Option`, `Mutex`, ...) *is* the receiver. A generic
    /// parameter has no concrete type.
    fn receiver_type(
        &self,
        module: &SymbolId,
        refs: &[Segs],
        self_ty: Option<&SymbolId>,
        params: InScope<'_>,
    ) -> Option<SymbolId> {
        const DEREF: &[&str] =
            &["Box", "Arc", "Rc", "Cow", "Pin", "Ref", "RefMut", "MutexGuard", "RwLockReadGuard", "RwLockWriteGuard"];
        for segs in refs {
            let last = segs.last()?.as_str();
            if segs.len() == 1 && params.contains(last) {
                return None;
            }
            let (id, _) = self.resolve_in(module, segs, self_ty, Ns::Type, params);
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
                    if let Some(id) = ids.iter().next() {
                        return (id.clone(), Confidence::Inferred);
                    }
                }
            }
        }
        (ids::unresolved_method_id(name), Confidence::External)
    }
}
