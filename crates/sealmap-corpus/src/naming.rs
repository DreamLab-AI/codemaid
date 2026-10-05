//! Stable ids, short labels and sequence lanes.

use sealmap_mermaid::Ident;
use sealmap_model::{Codebase, SymbolId, SymbolKind};

use crate::{CorpusOptions, ExternalLanes};

/// Diagram id for a symbol (stable across all documents).
pub(crate) fn ident(id: &SymbolId) -> Ident {
    Ident::from_path(id.as_str())
}

/// Every symbol and relation endpoint must get its own diagram id: a shared
/// id would silently merge two nodes in every diagram both appear in, and
/// break merge-by-id across documents. `Ident::from_path` is injective by
/// construction (short of a 64-bit hash collision); this checks it on the
/// actual codebase.
///
/// # Panics
///
/// If two distinct ids map to the same diagram id.
pub(crate) fn assert_unique_idents(cb: &Codebase) {
    let mut seen: std::collections::BTreeMap<Ident, &SymbolId> = std::collections::BTreeMap::new();
    let ids = cb.symbols.keys().chain(cb.relations.iter().flat_map(|r| [&r.from, &r.to]));
    for id in ids {
        if let Some(prev) = seen.insert(ident(id), id) {
            assert!(prev == id, "diagram id collision: `{prev}` and `{id}` both map to `{}`", ident(id));
        }
    }
}

/// The lane (participant) a call target belongs to, its alias, and the
/// prefix to show on the message when the lane is coarser than the owner.
pub(crate) struct Lane {
    pub id: SymbolId,
    pub alias: String,
    pub prefix: String,
    pub external: bool,
}

pub(crate) fn lane_of(cb: &Codebase, target: &SymbolId, opts: &CorpusOptions) -> Lane {
    if cb.is_internal(target) {
        let owner = cb.owner_of(target).unwrap_or_else(|| target.clone());
        return Lane { alias: alias_for(cb, &owner), id: owner, prefix: String::new(), external: false };
    }
    let s = target.as_str();
    if s.starts_with("?::") {
        return Lane { id: SymbolId::new("unresolved"), alias: "?".into(), prefix: String::new(), external: true };
    }
    // An internal type reached with an unknown method (inferred): owner is
    // the parent, which is internal.
    if let Some(parent) = target.parent() {
        if cb.is_internal(&parent) {
            return Lane { alias: alias_for(cb, &parent), id: parent, prefix: String::new(), external: false };
        }
    }
    let segs: Vec<&str> = s.split("::").collect();
    match opts.external_lanes {
        ExternalLanes::CrateRoot => {
            let root = segs[0];
            let prefix = if segs.len() > 2 { format!("{}::", segs[segs.len() - 2]) } else { String::new() };
            Lane { id: SymbolId::new(root), alias: root.to_owned(), prefix, external: true }
        }
        ExternalLanes::Owner => {
            let owner = target.parent().unwrap_or_else(|| target.clone());
            Lane { alias: owner.name().to_owned(), id: owner, prefix: String::new(), external: true }
        }
    }
}

/// Alias for an internal lane: type name, or module name for modules.
pub(crate) fn alias_for(cb: &Codebase, id: &SymbolId) -> String {
    match cb.symbol(id) {
        Some(s) if s.kind == SymbolKind::Module => format!("{} mod", s.name),
        Some(s) => s.name.clone(),
        None => id.name().to_owned(),
    }
}
