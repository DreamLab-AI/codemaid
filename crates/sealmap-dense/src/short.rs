//! Short names: the compact, per-output handles the dense text uses in place
//! of full `sym:` ids.
//!
//! A short name is derived from the id alone, at the lowest *level* that
//! makes it unique across the codebase:
//!
//! | Level | Shape | Example |
//! |---|---|---|
//! | 0 | owner and name | `Db.put`, `connect`, `db/` |
//! | 1 | … with the trait-impl block | `Db[Store].put` |
//! | 2 | … with the module path | `db/Db[Store].put` |
//! | 3 | … with the package | `shop:db/Db[Store].put` |
//! | 4 | … with 8 hex digits of the id's BLAKE3 | `shop:db/Db.put'1a2b3c4d` |
//! | 5 | … with the whole digest | `shop:db/Db.put'1a2b…` |
//!
//! Every member of a colliding group moves up together, so no symbol is
//! favoured by the order it was found in, and a name depends only on the ids
//! in the codebase. Level 5 is injective unless BLAKE3 collides.

use std::collections::BTreeMap;

use sealmap_model::{Codebase, ContentHash, DescriptorKind, DescriptorView, IdView, SymbolId};

/// The highest level [`candidate`] knows.
const MAX_LEVEL: u8 = 5;

/// The short name of every symbol in a codebase, and its inverse.
#[derive(Debug, Clone)]
pub(crate) struct ShortNames<'a> {
    by_id: BTreeMap<&'a SymbolId, String>,
    by_short: BTreeMap<String, &'a SymbolId>,
}

impl<'a> ShortNames<'a> {
    /// Assign a unique short name to every symbol of `cb`.
    pub(crate) fn new(cb: &'a Codebase) -> Self {
        let ids: Vec<&'a SymbolId> = cb.symbols.keys().collect();
        let mut level = vec![0u8; ids.len()];
        let mut names: Vec<String> = ids.iter().map(|id| candidate(id, 0)).collect();
        loop {
            let groups = collisions(&names);
            if groups.is_empty() {
                break;
            }
            let mut progressed = false;
            for i in groups.into_iter().flatten() {
                while level[i] < MAX_LEVEL {
                    level[i] += 1;
                    let next = candidate(ids[i], level[i]);
                    if next != names[i] {
                        names[i] = next;
                        progressed = true;
                        break;
                    }
                }
            }
            if !progressed {
                // Only a full BLAKE3 collision gets here. Keep the promise of
                // uniqueness anyway: number the members in id order.
                for group in collisions(&names) {
                    for (k, i) in group.into_iter().enumerate().skip(1) {
                        names[i] = format!("{}'{k}", names[i]);
                    }
                }
                break;
            }
        }
        let mut by_id = BTreeMap::new();
        let mut by_short = BTreeMap::new();
        for (id, name) in ids.into_iter().zip(names) {
            by_short.insert(name.clone(), id);
            by_id.insert(id, name);
        }
        debug_assert_eq!(by_id.len(), by_short.len(), "short names must be unique");
        Self { by_id, by_short }
    }

    /// The short name of `id`, if it is a symbol of the codebase.
    pub(crate) fn get(&self, id: &SymbolId) -> Option<&str> {
        self.by_id.get(id).map(String::as_str)
    }

    /// The symbol a short name stands for.
    pub(crate) fn resolve(&self, short: &str) -> Option<&'a SymbolId> {
        self.by_short.get(short).copied()
    }

    /// Every `(short, id)` pair, ordered by short name.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &'a SymbolId)> + '_ {
        self.by_short.iter().map(|(s, id)| (s.as_str(), *id))
    }
}

/// Groups of indices whose names are equal, each group in index (id) order.
fn collisions(names: &[String]) -> Vec<Vec<usize>> {
    let mut seen: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, n) in names.iter().enumerate() {
        seen.entry(n.as_str()).or_default().push(i);
    }
    seen.into_values().filter(|g| g.len() > 1).collect()
}

/// The short name of `id` at `level` (see the module docs). Never contains
/// whitespace and is never empty.
pub(crate) fn candidate(id: &SymbolId, level: u8) -> String {
    let mut out = match id.view() {
        IdView::Global(g) => {
            let split = g.descriptors.iter().position(|d| d.kind != DescriptorKind::Namespace);
            let (ns, items) = g.descriptors.split_at(split.unwrap_or(g.descriptors.len()));
            let module = || match ns.last() {
                Some(last) => format!("{}/", last.name),
                None => format!("{}/", g.package),
            };
            let qualified = || {
                let mut s: String = ns.iter().map(|d| format!("{}/", d.name)).collect();
                s.push_str(&render(items));
                if s.is_empty() { format!("{}/", g.package) } else { s }
            };
            match level {
                0 | 1 if items.is_empty() => module(),
                0 => render(tail(items, false)),
                1 => render(tail(items, true)),
                2 => qualified(),
                _ => format!("{}:{}", g.package, qualified()),
            }
        }
        IdView::Path(_) | IdView::Unresolved(_) => id.display_path(),
    };
    out.retain(|c| !c.is_whitespace());
    if out.is_empty() {
        out.push('_');
    }
    if level >= 4 {
        let hash = ContentHash::of_text(id.as_str());
        out.push('\'');
        out.push_str(hash.short(if level == 4 { 8 } else { 64 }));
    }
    out
}

/// The last two items that are not trait-impl blocks; with `blocks`, the
/// blocks between them are kept.
fn tail<'v, 'a>(items: &'v [DescriptorView<'a>], blocks: bool) -> Vec<&'v DescriptorView<'a>> {
    let real: Vec<usize> =
        items.iter().enumerate().filter(|(_, d)| d.kind != DescriptorKind::TypeParameter).map(|(i, _)| i).collect();
    let from = real.len().saturating_sub(2);
    match real.get(from) {
        Some(&start) => items[start..].iter().filter(|d| blocks || d.kind != DescriptorKind::TypeParameter).collect(),
        // Only trait-impl blocks: keep them all.
        None => items.iter().collect(),
    }
}

/// Items joined in a Rust-like reading order: `Db[Store].put`.
fn render<'v, 'a: 'v, I>(items: I) -> String
where
    I: IntoIterator<Item = &'v DescriptorView<'a>>,
{
    let mut s = String::new();
    let mut after_namespace = true;
    for d in items {
        if !s.is_empty() && !after_namespace && d.kind != DescriptorKind::TypeParameter {
            s.push('.');
        }
        after_namespace = false;
        match d.kind {
            DescriptorKind::TypeParameter => {
                s.push('[');
                s.push_str(&d.name);
                s.push(']');
            }
            DescriptorKind::Parameter => {
                s.push('(');
                s.push_str(&d.name);
                s.push(')');
            }
            DescriptorKind::Macro => {
                s.push_str(&d.name);
                s.push('!');
            }
            DescriptorKind::Meta => {
                s.push_str(&d.name);
                s.push(':');
            }
            DescriptorKind::Namespace => {
                s.push_str(&d.name);
                s.push('/');
                after_namespace = true;
            }
            DescriptorKind::Type | DescriptorKind::Method | DescriptorKind::Term => s.push_str(&d.name),
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> SymbolId {
        SymbolId::parse(s).unwrap()
    }

    #[test]
    fn levels_grow_one_qualifier_at_a_time() {
        let put = id("sym:cargo shop . db/Db#[Store]put().");
        let names: Vec<String> = (0..=3).map(|l| candidate(&put, l)).collect();
        assert_eq!(names, ["Db.put", "Db[Store].put", "db/Db[Store].put", "shop:db/Db[Store].put"]);
        assert!(candidate(&put, 4).starts_with("shop:db/Db[Store].put'"));
        assert_eq!(candidate(&put, 4).len(), "shop:db/Db[Store].put'".len() + 8);
    }

    #[test]
    fn modules_and_roots_end_in_a_slash() {
        assert_eq!(candidate(&id("sym:cargo shop . db/"), 0), "db/");
        assert_eq!(candidate(&id("sym:cargo shop . a/b/"), 2), "a/b/");
        assert_eq!(candidate(&id("sym:cargo shop ."), 0), "shop/");
        assert_eq!(candidate(&id("sym:cargo shop ."), 3), "shop:shop/");
    }

    #[test]
    fn kinds_keep_their_marks_and_whitespace_goes() {
        assert_eq!(candidate(&id("sym:cargo shop . log!"), 0), "log!");
        assert_eq!(candidate(&id("sym:cargo shop . MAX."), 0), "MAX");
        let from = id("sym:cargo shop . Db#[`Iterator<Item = u8>`]next().");
        assert_eq!(candidate(&from, 1), "Db[Iterator<Item=u8>].next");
    }
}
