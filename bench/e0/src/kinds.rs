//! Exploratory (EK): which diagram kinds a flag comes through. Each tracked
//! unit is attributed to the kinds of the citations that created it.
//!
//! - **T_file:** a changed source is attributed to the kinds of every citation
//!   of that file, or to `uncited` if no citation names it.
//! - **T_sym:** a changed cited symbol is attributed to the kinds of the
//!   citations that mapped to it; a changed fallback-citation file to the kinds
//!   of its fallback citations; a changed uncited source to `uncited`.
//!
//! A topic is flagged *only through* kind K when K is the one kind across all
//! of the units that set the flag. Nothing here changes a flag.

use std::collections::{BTreeMap, BTreeSet};

use crate::counting::Flags;
use crate::mapping::{Mapped, Tracking};
use crate::topics::Citation;

/// The bucket for sources no citation names.
pub const UNCITED: &str = "uncited";

/// Kinds per tracked unit of one topic in one repository.
#[derive(Debug, Clone, Default)]
pub struct KindIndex {
    source: BTreeMap<String, BTreeSet<String>>,
    symbol: BTreeMap<String, BTreeSet<String>>,
    fallback_file: BTreeMap<String, BTreeSet<String>>,
}

/// Index a topic's mapped citations (citation, repository-relative file, mapping).
pub fn index(cites: &[(Citation, String, Mapped)]) -> KindIndex {
    let mut k = KindIndex::default();
    for (c, file, m) in cites {
        k.source.entry(file.clone()).or_default().insert(c.kind.clone());
        match m {
            Mapped::Symbol { id, .. } => k.symbol.entry(id.clone()).or_default().insert(c.kind.clone()),
            Mapped::Fallback(_) => k.fallback_file.entry(file.clone()).or_default().insert(c.kind.clone()),
        };
    }
    k
}

/// The kinds a topic's T_file and T_sym flags came through for one commit
/// (empty when the method did not flag the topic).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attr {
    pub file: BTreeSet<String>,
    pub sym: BTreeSet<String>,
}

fn uncited() -> BTreeSet<String> {
    [UNCITED.to_string()].into()
}

/// Attribute one topic's flags for one commit.
pub fn attribute(t: &Tracking, idx: &KindIndex, changed: &BTreeSet<String>, f: &Flags) -> Attr {
    let mut a = Attr::default();
    for s in t.sources.iter().filter(|s| changed.contains(*s)) {
        a.file.extend(idx.source.get(s).cloned().unwrap_or_else(uncited));
    }
    if !f.sym {
        return a;
    }
    for (id, _) in &f.changed_symbols {
        a.sym.extend(idx.symbol.get(id).cloned().unwrap_or_default());
    }
    for s in t.fallback_files.iter().filter(|s| changed.contains(*s)) {
        a.sym.extend(idx.fallback_file.get(s).cloned().unwrap_or_default());
    }
    if t.uncited_files.iter().any(|s| changed.contains(s)) {
        a.sym.insert(UNCITED.into());
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counting::flag;
    use crate::mapping::{Fallback, RepoCitation, tracking};
    use crate::model::Model;
    use crate::topics::Resolved;

    fn cite(kind: &str, file: &str, m: Mapped) -> (Citation, String, Mapped) {
        let c = Citation {
            diagram: "X-01.1".into(),
            prose: kind == "prose",
            kind: kind.into(),
            cited: file.into(),
            line: 1,
            end: None,
            resolved: Resolved::Source(file.into()),
        };
        (c, file.into(), m)
    }

    #[test]
    fn flags_are_attributed_to_the_kinds_that_created_them() {
        let sym = Mapped::Symbol { id: "sym:a".into(), module: false };
        let cites = vec![
            cite("sequenceDiagram", "a.rs", sym.clone()),
            cite("flowchart", "a.rs", sym),
            cite("classDiagram", "b.nix", Mapped::Fallback(Fallback::NotRust)),
        ];
        let sources: BTreeSet<String> = ["a.rs", "b.nix", "c.md"].map(String::from).into();
        let rc: Vec<RepoCitation> =
            cites.iter().map(|(_, f, m)| RepoCitation { file: f.clone(), mapped: m.clone() }).collect();
        let t = tracking(&sources, &rc);
        let idx = index(&cites);
        let empty = Model::default();
        let set = |v: &[&str]| -> BTreeSet<String> { v.iter().map(|s| s.to_string()).collect() };

        // Only the class diagram's fallback file changed: both methods, one kind.
        let changed = set(&["b.nix"]);
        let a = attribute(&t, &idx, &changed, &flag(&t, &changed, &empty, &empty));
        assert_eq!((a.file, a.sym), (set(&["classDiagram"]), set(&["classDiagram"])));

        // a.rs changed but its symbol did not (absent at both, file changed counts): two kinds.
        let changed = set(&["a.rs", "c.md"]);
        let a = attribute(&t, &idx, &changed, &flag(&t, &changed, &empty, &empty));
        assert_eq!(a.file, set(&["sequenceDiagram", "flowchart", UNCITED]));
        assert_eq!(a.sym, set(&["sequenceDiagram", "flowchart", UNCITED]));
    }
}
