//! Per-commit flags (PREREG "Per-commit counts"): T_file, T_sym and T_hop for
//! one topic, from the files changed between P and C and the models at both.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::mapping::Tracking;
use crate::model::Model;

/// How one cited symbol compares between P and C.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymChange {
    /// Present at both with identical fingerprints, or absent at both with its file unchanged.
    Same,
    /// `sig_hash` or `body_hash` differs.
    Hash,
    /// Present at exactly one of P and C.
    OneSided,
    /// Absent at both, with its cited file changed: counted as changed (conservative).
    AbsentBothFileChanged,
}

pub(crate) fn compare(id: &str, p: &Model, c: &Model) -> SymChange {
    match (p.syms.get(id), c.syms.get(id)) {
        (Some(a), Some(b)) if a.sig != b.sig || a.body != b.body => SymChange::Hash,
        (Some(_), Some(_)) => SymChange::Same,
        (None, None) => SymChange::Same,
        _ => SymChange::OneSided,
    }
}

/// One topic's flags for one commit, with the reasons that set them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Flags {
    pub file: bool,
    pub sym: bool,
    pub hop: bool,
    /// A whole-file (fallback or uncited) file changed.
    pub by_file_level: bool,
    /// A file with a fallback citation changed.
    pub by_fallback_citation: bool,
    /// A source no citation names changed.
    pub by_uncited_source: bool,
    /// A cited symbol changed by hash or by existing at one side only.
    pub by_symbol: bool,
    /// A cited symbol absent at both sides had its file changed.
    pub by_absent_both: bool,
    /// Cited symbols that changed, with how.
    pub changed_symbols: Vec<(String, SymChange)>,
    /// One-hop callees that changed.
    pub changed_callees: BTreeSet<String>,
}

/// Flag one topic for one commit.
pub fn flag(t: &Tracking, changed: &BTreeSet<String>, p: &Model, c: &Model) -> Flags {
    let mut f = Flags { file: t.sources.iter().any(|s| changed.contains(s)), ..Flags::default() };
    f.by_fallback_citation = t.fallback_files.iter().any(|s| changed.contains(s));
    f.by_uncited_source = t.uncited_files.iter().any(|s| changed.contains(s));
    f.by_file_level = f.by_fallback_citation || f.by_uncited_source;
    for (id, file) in &t.symbols {
        let mut how = compare(id, p, c);
        if how == SymChange::Same && !p.syms.contains_key(id) && !c.syms.contains_key(id) && changed.contains(file) {
            how = SymChange::AbsentBothFileChanged;
        }
        match how {
            SymChange::Same => {}
            SymChange::AbsentBothFileChanged => f.by_absent_both = true,
            _ => f.by_symbol = true,
        }
        if how != SymChange::Same {
            f.changed_symbols.push((id.clone(), how));
        }
        let none = BTreeSet::new();
        let callees = p.calls.get(id).unwrap_or(&none).union(c.calls.get(id).unwrap_or(&none));
        for callee in callees {
            if compare(callee, p, c) != SymChange::Same {
                f.changed_callees.insert(callee.clone());
            }
        }
    }
    f.sym = f.by_file_level || f.by_symbol || f.by_absent_both;
    f.hop = f.sym || !f.changed_callees.is_empty();
    f
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::{Fallback, Mapped, RepoCitation, Stamp, map_citation, tracking};
    use sealmap_model::SourceSet;
    use sealmap_rust::{RustOptions, extract};

    const CARGO: &str = "[package]\nname = \"shop\"";

    fn model(files: &[(&str, &str)]) -> Model {
        let mut s = SourceSet::new();
        s.insert("Cargo.toml", CARGO).unwrap();
        for (p, t) in files {
            s.insert(p, t).unwrap();
        }
        Model::from_extraction(&extract(&s, &RustOptions::default()))
    }

    fn set(v: &[&str]) -> BTreeSet<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    /// Track `file:line` citations at stamp `m`, over `sources`.
    fn track(m: &Model, sources: &[&str], cites: &[(&str, u32)]) -> Tracking {
        let rc: Vec<RepoCitation> = cites
            .iter()
            .map(|(f, l)| RepoCitation {
                file: f.to_string(),
                mapped: map_citation(f, *l, &Stamp::Model { model: m, file_exists: true }),
            })
            .collect();
        tracking(&set(sources), &rc)
    }

    const V1: &str = "pub fn a() -> u8 {\n    b()\n}\n\npub fn b() -> u8 {\n    1\n}\n\npub fn c() -> u8 {\n    3\n}\n";

    #[test]
    fn an_edit_outside_the_cited_fn_flags_file_but_not_sym() {
        let p = model(&[("src/lib.rs", V1)]);
        let c = model(&[("src/lib.rs", &V1.replace("    3\n", "    4\n"))]);
        let t = track(&p, &["src/lib.rs"], &[("src/lib.rs", 2)]);
        assert_eq!(t.symbols.iter().next().unwrap().0, "sym:cargo shop . a().");
        let f = flag(&t, &set(&["src/lib.rs"]), &p, &c);
        assert!(f.file && !f.sym && !f.hop, "{f:?}");
    }

    #[test]
    fn a_one_hop_callee_change_flags_hop_only() {
        let p = model(&[("src/lib.rs", V1)]);
        let c = model(&[("src/lib.rs", &V1.replace("    1\n", "    2\n"))]);
        let t = track(&p, &["src/lib.rs"], &[("src/lib.rs", 2)]);
        let f = flag(&t, &set(&["src/lib.rs"]), &p, &c);
        assert!(f.file && !f.sym && f.hop, "{f:?}");
        assert_eq!(f.changed_callees, set(&["sym:cargo shop . b()."]));
    }

    #[test]
    fn a_symbol_that_disappears_is_changed() {
        let p = model(&[("src/lib.rs", V1)]);
        let c = model(&[("src/lib.rs", &V1.replace("pub fn c()", "pub fn d()"))]);
        let t = track(&p, &["src/lib.rs"], &[("src/lib.rs", 10)]);
        let f = flag(&t, &set(&["src/lib.rs"]), &p, &c);
        assert!(f.file && f.sym && f.by_symbol && !f.by_absent_both);
        assert_eq!(f.changed_symbols, [("sym:cargo shop . c().".to_string(), SymChange::OneSided)]);
    }

    #[test]
    fn absent_at_both_with_file_changed_counts_conservatively() {
        let stamp = model(&[("src/lib.rs", V1)]);
        let gone = V1.replace("pub fn c()", "pub fn e()");
        let p = model(&[("src/lib.rs", &gone)]);
        let c = model(&[("src/lib.rs", &format!("{gone}\n// edit\n"))]);
        let t = track(&stamp, &["src/lib.rs"], &[("src/lib.rs", 10)]);
        let f = flag(&t, &set(&["src/lib.rs"]), &p, &c);
        assert!(f.sym && f.by_absent_both && !f.by_symbol);
        // The same with the file untouched does not count.
        assert!(!flag(&t, &set(&[]), &p, &p).sym);
    }

    #[test]
    fn a_fallback_file_and_an_uncited_source_track_the_whole_file() {
        let p = model(&[("src/lib.rs", V1)]);
        let t = track(&p, &["src/lib.rs", "flake.nix", "README.md"], &[("src/lib.rs", 2), ("flake.nix", 4)]);
        assert_eq!(
            map_citation("flake.nix", 4, &Stamp::Model { model: &p, file_exists: true }),
            Mapped::Fallback(Fallback::NotRust)
        );
        let f = flag(&t, &set(&["flake.nix"]), &p, &p);
        assert!(f.file && f.sym && f.by_fallback_citation && !f.by_uncited_source);
        let g = flag(&t, &set(&["README.md"]), &p, &p);
        assert!(g.file && g.sym && g.by_uncited_source && !g.by_fallback_citation);
        let h = flag(&t, &set(&["other.rs"]), &p, &p);
        assert!(!h.file && !h.sym && !h.hop);
    }

    #[test]
    fn comment_only_edits_inside_the_cited_fn_do_not_flag_sym() {
        let p = model(&[("src/lib.rs", V1)]);
        let c = model(&[("src/lib.rs", &V1.replace("    b()\n", "    // why\n    b()\n"))]);
        let t = track(&p, &["src/lib.rs"], &[("src/lib.rs", 2)]);
        let f = flag(&t, &set(&["src/lib.rs"]), &p, &c);
        assert!(f.file && !f.sym && !f.hop, "{f:?}");
    }
}
