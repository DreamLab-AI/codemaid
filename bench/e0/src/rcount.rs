//! E0b per-commit counts (PREREG "Per-commit counts"): T_region for one
//! topic, from the files changed between P and C, the models at both, and the
//! cited regions re-found by node path in each side's source.

use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

use serde::Serialize;

use crate::counting::{SymChange, compare};
use crate::git::Repo;
use crate::mapping::Tracking;
use crate::model::{Model, Sym};
use crate::region::{self, RegionPath};

/// File text at a revision.
pub trait Source {
    /// The UTF-8 text of `path` at `rev`, or `None` when absent or not UTF-8.
    fn text(&self, rev: &str, path: &str) -> Option<String>;
}

impl Source for Repo {
    fn text(&self, rev: &str, path: &str) -> Option<String> {
        self.show(rev, path).and_then(|b| String::from_utf8(b).ok())
    }
}

/// Parsed files and region hashes, memoised by revision.
pub struct Files<S: Source> {
    src: S,
    parsed: HashMap<(String, String), Option<Rc<syn::File>>>,
    hashes: HashMap<(String, String, RegionPath), Option<[u8; 16]>>,
    /// Files parsed (distinct revision and path).
    pub parses: usize,
    /// (revision, symbol) where the model has the symbol but its syntax node
    /// was not found in the parsed file (should stay empty).
    pub node_misses: BTreeSet<(String, String)>,
    /// (revision, path) a region was looked up in that did not parse.
    pub unparsed: BTreeSet<(String, String)>,
}

impl<S: Source> Files<S> {
    pub fn new(src: S) -> Self {
        Self {
            src,
            parsed: HashMap::new(),
            hashes: HashMap::new(),
            parses: 0,
            node_misses: BTreeSet::new(),
            unparsed: BTreeSet::new(),
        }
    }

    /// `path` at `rev`, parsed as sealmap parses it (`syn::parse_file`).
    pub fn file(&mut self, rev: &str, path: &str) -> Option<Rc<syn::File>> {
        let key = (rev.to_string(), path.to_string());
        if let Some(f) = self.parsed.get(&key) {
            return f.clone();
        }
        self.parses += 1;
        let f = self.src.text(rev, path).and_then(|t| syn::parse_file(&t).ok()).map(Rc::new);
        self.parsed.insert(key, f.clone());
        f
    }

    /// The region of a citation at `line` inside `sym` at `rev`; `None` when
    /// the symbol's node is not found there.
    pub fn locate(&mut self, rev: &str, sym: &Sym, line: u32) -> Option<RegionPath> {
        let f = self.file(rev, &sym.file)?;
        region::locate(&f, sym, line)
    }

    /// The hash of region `path` of symbol `id` (= `sym` at `rev`).
    pub fn hash(&mut self, rev: &str, id: &str, sym: &Sym, path: &RegionPath) -> Option<[u8; 16]> {
        let key = (rev.to_string(), id.to_string(), path.clone());
        if let Some(h) = self.hashes.get(&key) {
            return *h;
        }
        let h = match self.file(rev, &sym.file) {
            None => {
                self.unparsed.insert((rev.to_string(), sym.file.clone()));
                None
            }
            Some(f) if region::symbol_node(&f, sym).is_none() => {
                self.node_misses.insert((rev.to_string(), id.to_string()));
                None
            }
            Some(f) => region::hash_at(&f, sym, path),
        };
        self.hashes.insert(key, h);
        h
    }
}

/// A cited region: (cited file, symbol id, node path from the symbol).
pub type RegionKey = (String, String, RegionPath);

/// What one topic tracks in one repository under E0b: E0's whole-file
/// tracking unchanged, and cited regions in place of cited symbols.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RTracking {
    pub sources: BTreeSet<String>,
    pub fallback_files: BTreeSet<String>,
    pub uncited_files: BTreeSet<String>,
    pub regions: BTreeSet<RegionKey>,
}

/// E0's tracking with its symbols replaced by `regions` (only those whose
/// file the base tracks a symbol in, so a restricted base restricts them too).
pub fn rtracking(base: &Tracking, regions: &[RegionKey]) -> RTracking {
    let files: BTreeSet<&String> = base.symbols.iter().map(|(_, f)| f).collect();
    RTracking {
        sources: base.sources.clone(),
        fallback_files: base.fallback_files.clone(),
        uncited_files: base.uncited_files.clone(),
        regions: regions.iter().filter(|(f, _, _)| files.contains(f)).cloned().collect(),
    }
}

/// How one cited region compares between P and C.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RChange {
    /// Identical hash at both, or (whole-symbol region) as E0's `same`.
    Same,
    /// The region hash differs (or, for a whole-symbol region, `sig_hash` or `body_hash`).
    Hash,
    /// The region (or its symbol) is found at exactly one side.
    OneSided,
    /// The symbol is at both sides, its file changed, and the path is found at neither.
    Missing,
    /// The symbol is absent at both, with the cited file changed (E0's conservative rule).
    AbsentBothFileChanged,
}

/// One topic's E0b flags for one commit, with the reasons that set them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RFlags {
    pub file: bool,
    pub region: bool,
    pub by_fallback_citation: bool,
    pub by_uncited_source: bool,
    /// Fallback-citation files that changed.
    pub fallback_hit: Vec<String>,
    /// Uncited sources that changed.
    pub uncited_hit: Vec<String>,
    /// A cited region (narrower than its symbol) changed.
    pub by_region: bool,
    /// A citation whose region is its whole symbol changed.
    pub by_whole_symbol: bool,
    /// Cited regions that changed, with how.
    pub changed: Vec<(RegionKey, RChange)>,
}

/// One side of a commit: its sha and its model.
pub type Side<'a> = (&'a str, &'a Model);

fn region_change<S: Source>(
    key: &RegionKey,
    changed: &BTreeSet<String>,
    (p_rev, p): Side<'_>,
    (c_rev, c): Side<'_>,
    files: &mut Files<S>,
) -> RChange {
    let (file, id, path) = key;
    if path.is_empty() {
        let how = compare(id, p, c);
        return match how {
            SymChange::Same if !p.syms.contains_key(id) && !c.syms.contains_key(id) && changed.contains(file) => {
                RChange::AbsentBothFileChanged
            }
            SymChange::Same => RChange::Same,
            SymChange::Hash => RChange::Hash,
            SymChange::OneSided => RChange::OneSided,
            SymChange::AbsentBothFileChanged => RChange::AbsentBothFileChanged,
        };
    }
    match (p.syms.get(id), c.syms.get(id)) {
        (None, None) if changed.contains(file) => RChange::AbsentBothFileChanged,
        (None, None) => RChange::Same,
        // The symbol's file is identical at both sides: so is every region in it.
        (Some(a), Some(b)) if a.file == b.file && !changed.contains(&a.file) => RChange::Same,
        (ps, cs) => {
            let hp = ps.and_then(|s| files.hash(p_rev, id, s, path));
            let hc = cs.and_then(|s| files.hash(c_rev, id, s, path));
            match (hp, hc) {
                (Some(x), Some(y)) if x == y => RChange::Same,
                (Some(_), Some(_)) => RChange::Hash,
                (Some(_), None) | (None, Some(_)) => RChange::OneSided,
                (None, None) => RChange::Missing,
            }
        }
    }
}

/// Flag one topic for one commit under T_region.
pub fn flag_regions<S: Source>(
    t: &RTracking,
    changed: &BTreeSet<String>,
    p: Side<'_>,
    c: Side<'_>,
    files: &mut Files<S>,
) -> RFlags {
    let mut f = RFlags { file: t.sources.iter().any(|s| changed.contains(s)), ..RFlags::default() };
    f.fallback_hit = t.fallback_files.iter().filter(|s| changed.contains(*s)).cloned().collect();
    f.uncited_hit = t.uncited_files.iter().filter(|s| changed.contains(*s)).cloned().collect();
    f.by_fallback_citation = !f.fallback_hit.is_empty();
    f.by_uncited_source = !f.uncited_hit.is_empty();
    for key in &t.regions {
        let how = region_change(key, changed, p, c, files);
        if how == RChange::Same {
            continue;
        }
        if key.2.is_empty() {
            f.by_whole_symbol = true;
        } else {
            f.by_region = true;
        }
        f.changed.push((key.clone(), how));
    }
    f.region = f.by_fallback_citation || f.by_uncited_source || f.by_region || f.by_whole_symbol;
    f
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::{RepoCitation, Stamp, map_citation, tracking};
    use crate::region::path_label;
    use sealmap_model::SourceSet;
    use sealmap_rust::{RustOptions, extract};
    use std::collections::BTreeMap;

    /// In-memory revisions: rev → path → text.
    struct Mem(BTreeMap<&'static str, BTreeMap<&'static str, String>>);

    impl Source for &Mem {
        fn text(&self, rev: &str, path: &str) -> Option<String> {
            self.0.get(rev)?.get(path).cloned()
        }
    }

    fn model(files: &BTreeMap<&'static str, String>) -> Model {
        let mut s = SourceSet::new();
        s.insert("Cargo.toml", "[package]\nname = \"shop\"").unwrap();
        for (p, t) in files {
            s.insert(p, t).unwrap();
        }
        Model::from_extraction(&extract(&s, &RustOptions::default()))
    }

    fn set(v: &[&str]) -> BTreeSet<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    /// Cite `file:line` at revision "stamp", then flag the change from "p" to "c".
    /// A trailing item keeps a fixture's first item from spanning the whole
    /// file (E0's innermost-symbol tie would then pick the module).
    fn run(stamp: &str, p: &str, c: &str, line: u32) -> (String, RFlags) {
        let tail = |s: &str| format!("{s}\npub fn tail() {{}}\n");
        let (stamp, p, c) = (tail(stamp), tail(p), tail(c));
        let mem = Mem(BTreeMap::from([
            ("stamp", BTreeMap::from([("src/lib.rs", stamp.to_string())])),
            ("p", BTreeMap::from([("src/lib.rs", p.to_string())])),
            ("c", BTreeMap::from([("src/lib.rs", c.to_string())])),
        ]));
        let ms = model(&mem.0["stamp"]);
        let mp = model(&mem.0["p"]);
        let mc = model(&mem.0["c"]);
        let mut files = Files::new(&mem);
        let mapped = map_citation("src/lib.rs", line, &Stamp::Model { model: &ms, file_exists: true });
        let base =
            tracking(&set(&["src/lib.rs"]), &[RepoCitation { file: "src/lib.rs".into(), mapped: mapped.clone() }]);
        let crate::mapping::Mapped::Symbol { id, .. } = mapped else { panic!("not a symbol") };
        let path = files.locate("stamp", &ms.syms[&id], line).expect("located");
        let label = path_label(&path);
        let t = rtracking(&base, &[("src/lib.rs".into(), id, path)]);
        let f = flag_regions(&t, &set(&["src/lib.rs"]), ("p", &mp), ("c", &mc), &mut files);
        (label, f)
    }

    const V1: &str = "\
pub fn route(x: u8) -> u8 {
    let base = 10;
    match x {
        0 => {
            base + 1
        }
        1 => {
            base + 2
        }
        _ => 0,
    }
}
";

    #[test]
    fn a_citation_in_an_unchanged_arm_of_a_changed_match_does_not_flag() {
        let c = V1.replace("base + 2", "base * 2");
        let (label, f) = run(V1, V1, &c, 5);
        assert_eq!(label, "stmt#1/arm#0");
        assert!(f.file && !f.region, "{f:?}");
    }

    #[test]
    fn a_citation_in_the_changed_arm_flags() {
        let c = V1.replace("base + 2", "base * 2");
        let (label, f) = run(V1, V1, &c, 8);
        assert_eq!(label, "stmt#1/arm#1");
        assert!(f.file && f.region && f.by_region && !f.by_whole_symbol, "{f:?}");
        assert_eq!(f.changed[0].1, RChange::Hash);
    }

    #[test]
    fn a_citation_on_the_match_line_covers_every_arm() {
        let c = V1.replace("base + 2", "base * 2");
        let (label, f) = run(V1, V1, &c, 3);
        assert_eq!(label, "stmt#1");
        assert!(f.region);
    }

    #[test]
    fn an_inserted_statement_shifts_ordinals_and_flags() {
        // Node paths use ordinals: a new statement before the cited one moves
        // it, and the old path now names a different node.
        let c = V1.replace("    let base = 10;\n", "    let base = 10;\n    log();\n");
        let (_, f) = run(V1, V1, &c, 3);
        assert!(f.region, "{f:?}");
    }

    #[test]
    fn a_vanished_region_counts_as_changed() {
        let c = V1.replace("        _ => 0,\n", "");
        let (label, f) = run(V1, V1, &c, 10);
        assert_eq!(label, "stmt#1/arm#2");
        assert_eq!(f.changed[0].1, RChange::OneSided);
    }

    #[test]
    fn a_line_outside_items_tracks_its_top_level_item() {
        let src = "use std::fmt;\n\npub struct A;\n\nimpl A {\n    pub fn f(&self) {}\n}\n";
        // An edit to the struct changes the root module's body hash (E0 would
        // flag the `use` citation) but not the `use` item.
        let c = src.replace("pub struct A;", "pub struct A(u8);");
        let (label, f) = run(src, src, &c, 1);
        assert_eq!(label, "item#0");
        assert!(f.file && !f.region, "{f:?}");
        // Editing the `use` itself flags.
        let d = src.replace("use std::fmt;", "use std::fmt::Debug;");
        let (_, g) = run(src, src, &d, 1);
        assert!(g.region);
        // The impl header line: the top-level impl, which a method edit changes.
        let e = src.replace("pub fn f(&self) {}", "pub fn f(&self) { g() }");
        let (label, h) = run(src, src, &e, 5);
        assert_eq!(label, "item#2");
        assert!(h.region);
    }

    #[test]
    fn a_whole_symbol_region_behaves_as_e0() {
        let src = "pub fn f() -> u8 {\n    1\n}\n\npub fn g() -> u8 {\n    2\n}\n";
        // Line 1 is the signature: no narrower node, the region is `f` itself.
        let c = src.replace("pub fn f() -> u8", "pub fn f() -> u16");
        let (label, f) = run(src, src, &c, 1);
        assert_eq!(label, "(symbol)");
        assert!(f.region && f.by_whole_symbol);
        let d = src.replace("    2\n", "    3\n");
        let (_, g) = run(src, src, &d, 1);
        assert!(g.file && !g.region);
    }

    #[test]
    fn comments_and_formatting_inside_the_region_do_not_flag() {
        let c =
            V1.replace("        1 => {\n            base + 2\n        }\n", "        // two\n        1 => base + 2,\n");
        let (_, f) = run(V1, V1, &c, 8);
        assert!(f.file && !f.region, "{f:?}");
    }

    #[test]
    fn fallback_and_uncited_files_are_as_e0() {
        let mut t = RTracking { sources: set(&["a.rs", "b.nix"]), ..RTracking::default() };
        t.fallback_files = set(&["b.nix"]);
        let m = Model::default();
        let mem = Mem(BTreeMap::new());
        let mut files = Files::new(&mem);
        let f = flag_regions(&t, &set(&["b.nix"]), ("p", &m), ("c", &m), &mut files);
        assert!(f.file && f.region && f.by_fallback_citation);
        let g = flag_regions(&t, &set(&["a.rs"]), ("p", &m), ("c", &m), &mut files);
        assert!(g.file && !g.region);
    }
}
