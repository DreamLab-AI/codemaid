//! `_overview.md`: cross-cutting diagrams for the whole codebase.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use sealmap_mermaid::{
    Cardinality, Class, ClassDiagram, ClassRelationKind, Direction, EdgeStyle, ErDiagram, Flowchart, Ident, NodeShape,
};
use sealmap_model::{Codebase, MemberKind, RelationKind, SymbolId, SymbolKind};

use crate::CorpusOptions;
use crate::naming::ident;

pub(crate) fn render(cb: &Codebase, opts: &CorpusOptions) -> String {
    let mut out = String::new();
    let stats = cb.stats();
    let _ = writeln!(out, "---\nsealmap: {}\nkind: overview\ncodebase: {}\n---", crate::CORPUS_SCHEMA_VERSION, cb.name);
    let _ = writeln!(
        out,
        "# {} overview\n{} files · {} symbols · {} relations · {} flows · {} calls",
        cb.name, stats.files, stats.symbols, stats.relations, stats.flows, stats.calls
    );

    let crates = crates_of(cb);
    if let Some(text) = crate_graph(cb, &crates, opts) {
        let _ = writeln!(out, "\n## crates\n```mermaid\n{text}```");
    }
    for krate in &crates {
        if let Some(text) = module_graph(cb, krate, opts) {
            let _ = writeln!(out, "\n## modules: {krate}\n```mermaid\n{text}```");
        }
    }
    for krate in &crates {
        if let Some(text) = data_model(cb, krate, opts) {
            let _ = writeln!(out, "\n## data: {krate}\n```mermaid\n{text}```");
        }
    }
    if let Some(text) = trait_map(cb, opts) {
        let _ = writeln!(out, "\n## traits\n```mermaid\n{text}```");
    }
    out
}

fn root(id: &SymbolId) -> &str {
    id.as_str().split("::").next().unwrap_or("")
}

fn crates_of(cb: &Codebase) -> BTreeSet<String> {
    cb.files.values().map(|f| root(&f.module).to_owned()).collect()
}

/// Module that physically contains a symbol (its file's module).
fn module_of(cb: &Codebase, id: &SymbolId) -> Option<SymbolId> {
    let s = cb.symbol(id)?;
    cb.file(&s.file).map(|f| f.module.clone())
}

const STRUCTURAL: [RelationKind; 6] = [
    RelationKind::Imports,
    RelationKind::Calls,
    RelationKind::Uses,
    RelationKind::FieldType,
    RelationKind::Implements,
    RelationKind::Extends,
];

/// Keep the `max` heaviest edges (ties broken by key order).
fn heaviest<K: Ord + Clone>(edges: BTreeMap<K, usize>, max: usize) -> (Vec<(K, usize)>, usize) {
    let total = edges.len();
    let mut v: Vec<(K, usize)> = edges.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(max);
    v.sort_by(|a, b| a.0.cmp(&b.0));
    (v, total.saturating_sub(max))
}

fn crate_graph(cb: &Codebase, crates: &BTreeSet<String>, opts: &CorpusOptions) -> Option<String> {
    let mut edges: BTreeMap<(String, String), usize> = BTreeMap::new();
    for r in cb.relations.iter().filter(|r| STRUCTURAL.contains(&r.kind)) {
        let (a, b) = (root(&r.from), root(&r.to));
        if a != b && crates.contains(a) && !b.is_empty() && b != "?" {
            *edges.entry((a.to_owned(), b.to_owned())).or_default() += 1;
        }
    }
    if crates.len() < 2 && edges.is_empty() {
        return None;
    }
    let (edges, dropped) = heaviest(edges, opts.max_edges);
    let externals: BTreeSet<&String> = edges.iter().map(|((_, b), _)| b).filter(|b| !crates.contains(*b)).collect();
    let mut f = Flowchart::new(Direction::LR);
    // Group crates by their top-level directory when there are several
    // (multi-repository inputs are laid out as `<repo>/...`).
    let mut groups: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    for k in crates {
        let top = cb
            .files
            .values()
            .find(|file| root(&file.module) == k)
            .map(|file| file.path.components().next().unwrap_or("").to_owned())
            .unwrap_or_default();
        groups.entry(top).or_default().push(k);
    }
    let multi = groups.len() > 1 && groups.keys().all(|g| !g.ends_with(".rs"));
    for (group, ks) in &groups {
        if multi {
            f.subgraph(Ident::new(&format!("repo_{group}")), group, None, |g| {
                for k in ks {
                    g.node(Ident::new(k), k, NodeShape::Stadium);
                }
            });
        } else {
            for k in ks {
                f.node(Ident::new(k), k, NodeShape::Stadium);
            }
        }
    }
    for e in &externals {
        f.node(Ident::new(e), e, NodeShape::Hexagon);
    }
    for ((a, b), n) in &edges {
        f.edge(&Ident::new(a), &Ident::new(b), EdgeStyle::Solid, Some(&n.to_string()));
    }
    let mut text = f.render();
    if dropped > 0 {
        text.push_str(&format!("%% {dropped} lighter edges omitted (max_edges)\n"));
    }
    Some(text)
}

fn module_graph(cb: &Codebase, krate: &str, opts: &CorpusOptions) -> Option<String> {
    let modules: BTreeSet<SymbolId> =
        cb.files.values().map(|f| f.module.clone()).filter(|m| root(m) == krate).collect();
    if modules.len() < 2 {
        return None;
    }
    let mut edges: BTreeMap<(SymbolId, SymbolId), usize> = BTreeMap::new();
    for r in cb.relations.iter().filter(|r| STRUCTURAL.contains(&r.kind) && root(&r.from) == krate) {
        let Some(a) = module_of(cb, &r.from) else { continue };
        let b = match module_of(cb, &r.to) {
            Some(b) => b,
            // Imports of a module target the module itself.
            None => continue,
        };
        if a != b && modules.contains(&a) && modules.contains(&b) {
            *edges.entry((a, b)).or_default() += 1;
        }
    }
    let (edges, dropped) = heaviest(edges, opts.max_edges);
    let mut f = Flowchart::new(Direction::LR);
    for m in &modules {
        let label = m.as_str().strip_prefix(&format!("{krate}::")).unwrap_or(m.as_str());
        f.node(ident(m), label, NodeShape::Rect);
    }
    for ((a, b), n) in &edges {
        f.edge(&ident(a), &ident(b), EdgeStyle::Solid, Some(&n.to_string()));
    }
    let mut text = f.render();
    if dropped > 0 {
        text.push_str(&format!("%% {dropped} lighter edges omitted (max_edges)\n"));
    }
    Some(text)
}

fn data_model(cb: &Codebase, krate: &str, opts: &CorpusOptions) -> Option<String> {
    let is_data = |k: SymbolKind| matches!(k, SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union);
    let types: BTreeSet<SymbolId> =
        cb.symbols.values().filter(|s| is_data(s.kind) && root(&s.id) == krate).map(|s| s.id.clone()).collect();
    // Field edges between data types of this crate.
    let mut rels: Vec<(SymbolId, String, SymbolId, Cardinality, bool)> = Vec::new();
    let mut degree: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for id in &types {
        let s = &cb.symbols[id];
        for m in s.members.iter().filter(|m| m.kind == MemberKind::Field || m.kind == MemberKind::Variant) {
            for t in m.refs.iter().filter(|t| types.contains(*t) && *t != id) {
                let ty = m.ty.as_deref().unwrap_or("");
                let (card, owned) = cardinality(ty);
                rels.push((id.clone(), m.name.clone(), t.clone(), card, owned));
                *degree.entry(id.clone()).or_default() += 1;
                *degree.entry(t.clone()).or_default() += 1;
            }
        }
    }
    if rels.is_empty() {
        return None;
    }
    let mut ranked: Vec<(&SymbolId, &usize)> = degree.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    let keep: BTreeSet<SymbolId> = ranked.into_iter().take(opts.max_entities).map(|(id, _)| id.clone()).collect();

    let mut er = ErDiagram::new();
    for id in &keep {
        let s = &cb.symbols[id];
        let e = ident(id);
        er.entity(e.clone(), &s.name);
        for m in s.members.iter().filter(|m| m.kind == MemberKind::Field) {
            er.attr(&e, m.ty.as_deref().unwrap_or("_"), &m.name, None);
        }
    }
    for (from, field, to, card, owned) in &rels {
        if keep.contains(from) && keep.contains(to) {
            er.relation(&ident(from), Cardinality::One, &ident(to), *card, *owned, field);
        }
    }
    let mut text = er.render();
    let omitted = degree.len().saturating_sub(keep.len());
    if omitted > 0 {
        text.push_str(&format!("%% {omitted} lower-degree entities omitted (max_entities)\n"));
    }
    Some(text)
}

/// Cardinality and ownership implied by a field type.
fn cardinality(ty: &str) -> (Cardinality, bool) {
    let t = ty.trim_start_matches('&').trim_start_matches("mut ");
    let many = ["Vec<", "VecDeque<", "HashMap<", "BTreeMap<", "HashSet<", "BTreeSet<", "IndexMap<", "SmallVec<", "["]
        .iter()
        .any(|w| t.starts_with(w));
    let shared = ty.starts_with('&') || ["Arc<", "Rc<", "Weak<"].iter().any(|w| t.contains(w));
    if many {
        (Cardinality::Many, !shared)
    } else if t.starts_with("Option<") {
        (Cardinality::ZeroOrOne, !shared)
    } else {
        (Cardinality::One, !shared)
    }
}

fn trait_map(cb: &Codebase, opts: &CorpusOptions) -> Option<String> {
    let traits: BTreeSet<&SymbolId> = cb.symbols_of_kind(SymbolKind::Trait).map(|s| &s.id).collect();
    let impls: Vec<_> = cb
        .relations_of_kind(RelationKind::Implements)
        .chain(cb.relations_of_kind(RelationKind::Extends))
        .filter(|r| traits.contains(&r.to))
        .take(opts.max_edges)
        .collect();
    if impls.is_empty() {
        return None;
    }
    let mut d = ClassDiagram::new(Direction::LR);
    for t in &traits {
        if impls.iter().any(|r| &r.to == *t || &r.from == *t) {
            let mut c = Class::new(ident(t), &cb.symbols[*t].name);
            c.annotation("trait");
            d.class(c);
        }
    }
    for r in &impls {
        if !d.has_class(&ident(&r.from)) {
            d.class(Class::new(ident(&r.from), r.from.name()));
        }
        let kind = if r.kind == RelationKind::Extends {
            ClassRelationKind::Inheritance
        } else {
            ClassRelationKind::Realization
        };
        d.relation(&ident(&r.from), &ident(&r.to), kind, None);
    }
    Some(d.render())
}
