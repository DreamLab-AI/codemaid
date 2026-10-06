//! E0c's flow hash and T_flow (`docs/evidence/E0c/PREREG.md`, "The flow hash"
//! and amendment #1).
//!
//! A symbol's flow is its `sig_hash` followed by its outgoing calls in the order
//! sealmap 0.2.0's Rust adapter records them (`Flow::calls()`, depth-first
//! source order), each as its resolution kind and target id. Nothing here
//! resolves a call: the list is read off the adapter's own `Symbol::flow`.

use std::collections::{BTreeMap, BTreeSet};

use sealmap_model::{Codebase, Confidence, Fingerprint};
use serde::Serialize;

use crate::mapping::Tracking;
use crate::model::Model;

/// How the adapter resolved a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Res {
    Exact,
    Inferred,
    External,
}

impl Res {
    fn of(c: Confidence) -> Self {
        match c {
            Confidence::Exact => Res::Exact,
            Confidence::Inferred => Res::Inferred,
            Confidence::External => Res::External,
        }
    }

    /// The word used in the canonical serialisation.
    pub fn as_str(self) -> &'static str {
        match self {
            Res::Exact => "exact",
            Res::Inferred => "inferred",
            Res::External => "external",
        }
    }
}

/// One symbol's flow: its signature hash, its ordered calls and the flow hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowRec {
    pub sig: Fingerprint,
    pub calls: Vec<(Res, String)>,
    pub hash: Fingerprint,
}

/// The canonical serialisation: `sig <blake3-16:hex>\n`, then `<kind> <target>\n` per call.
pub fn canonical(sig: &Fingerprint, calls: &[(Res, String)]) -> String {
    let mut s = format!("sig {sig}\n");
    for (r, t) in calls {
        s.push_str(r.as_str());
        s.push(' ');
        s.push_str(t);
        s.push('\n');
    }
    s
}

/// BLAKE3-16 over the canonical serialisation.
pub fn flow_hash(sig: &Fingerprint, calls: &[(Res, String)]) -> Fingerprint {
    Fingerprint::from_blake3(&blake3::hash(canonical(sig, calls).as_bytes()))
}

/// Every symbol's flow. Symbols without a `Flow` (non-callables, modules,
/// callables that make no call) have an empty call list.
pub fn project(cb: &Codebase) -> BTreeMap<String, FlowRec> {
    cb.symbols
        .values()
        .map(|s| {
            let calls: Vec<(Res, String)> = s
                .flow
                .as_ref()
                .map(|f| f.calls().map(|c| (Res::of(c.confidence), c.target.to_string())).collect())
                .unwrap_or_default();
            let hash = flow_hash(&s.sig_hash, &calls);
            (s.id.to_string(), FlowRec { sig: s.sig_hash, calls, hash })
        })
        .collect()
}

/// How one sequence-cited symbol's flow compares between P and C.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowChange {
    Same,
    /// Present at both, flow hash differs.
    Hash,
    /// Present at exactly one side.
    OneSided,
}

/// Compare one symbol's flow between two models.
pub fn compare(id: &str, p: &Model, c: &Model) -> FlowChange {
    match (p.flows.get(id), c.flows.get(id)) {
        (Some(a), Some(b)) if a.hash != b.hash => FlowChange::Hash,
        (Some(_), None) | (None, Some(_)) => FlowChange::OneSided,
        _ => FlowChange::Same,
    }
}

/// One topic's T_flow for one commit, with its reasons.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowFlags {
    /// T_flow.
    pub flow: bool,
    /// A file holding a fallback sequence citation changed.
    pub by_fallback: bool,
    /// A sequence-cited symbol's flow changed or it exists at one side only.
    pub by_flow: bool,
    /// A sequence-cited symbol is absent at both sides and its file changed
    /// (E0's conservative rule; sensitivity only, never sets `flow`).
    pub absent_both: bool,
    /// Cited symbols whose flow changed, with how.
    pub changed: Vec<(String, FlowChange)>,
}

/// T_flow for one topic, from its sequence-citation tracking.
pub fn flag(t: &Tracking, changed: &BTreeSet<String>, p: &Model, c: &Model) -> FlowFlags {
    let mut f = FlowFlags { by_fallback: t.fallback_files.iter().any(|s| changed.contains(s)), ..FlowFlags::default() };
    for (id, file) in &t.symbols {
        let how = compare(id, p, c);
        if how != FlowChange::Same {
            f.by_flow = true;
            f.changed.push((id.clone(), how));
        } else if !p.flows.contains_key(id) && !c.flows.contains_key(id) && changed.contains(file) {
            f.absent_both = true;
        }
    }
    f.flow = f.by_fallback || f.by_flow;
    f
}

/// What kind of change set a symbol's T_flow flag (exploratory driver analysis).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Driver {
    /// Present at one side only (added, removed, renamed or moved).
    OneSided,
    /// `sig_hash` changed.
    Signature,
    /// The flow changed but the symbol's `sig_hash` and `body_hash` did not:
    /// a call's resolution moved because of an edit elsewhere.
    Drift,
    /// Same calls, same kinds, different order.
    Reordered,
    /// Same targets in the same order; only a resolution kind changed.
    ResolutionOnly,
    /// Calls added, removed or retargeted.
    Calls,
}

/// Classify a changed symbol. `None` when its flow did not change.
pub fn driver(id: &str, p: &Model, c: &Model) -> Option<Driver> {
    let (a, b) = match (p.flows.get(id), c.flows.get(id)) {
        (Some(a), Some(b)) if a.hash != b.hash => (a, b),
        (Some(_), None) | (None, Some(_)) => return Some(Driver::OneSided),
        _ => return None,
    };
    if a.sig != b.sig {
        return Some(Driver::Signature);
    }
    if let (Some(x), Some(y)) = (p.syms.get(id), c.syms.get(id)) {
        if x.sig == y.sig && x.body == y.body {
            return Some(Driver::Drift);
        }
    }
    let targets = |v: &[(Res, String)]| v.iter().map(|(_, t)| t.clone()).collect::<Vec<_>>();
    if targets(&a.calls) == targets(&b.calls) {
        return Some(Driver::ResolutionOnly);
    }
    let mut sa = a.calls.clone();
    let mut sb = b.calls.clone();
    sa.sort();
    sb.sort();
    Some(if sa == sb { Driver::Reordered } else { Driver::Calls })
}

/// Calls at C not at P and at P not at C, as multisets (for the report).
pub fn call_diff(a: &[(Res, String)], b: &[(Res, String)]) -> (Vec<String>, Vec<String>) {
    let count = |v: &[(Res, String)]| {
        let mut m: BTreeMap<String, i64> = BTreeMap::new();
        for (r, t) in v {
            *m.entry(format!("{} {t}", r.as_str())).or_default() += 1;
        }
        m
    };
    let (ca, cb) = (count(a), count(b));
    let keys: BTreeSet<&String> = ca.keys().chain(cb.keys()).collect();
    let (mut added, mut removed) = (Vec::new(), Vec::new());
    for k in keys {
        let d = cb.get(k).copied().unwrap_or(0) - ca.get(k).copied().unwrap_or(0);
        for _ in 0..d.max(0) {
            added.push(k.clone());
        }
        for _ in 0..(-d).max(0) {
            removed.push(k.clone());
        }
    }
    (added, removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counting;
    use crate::mapping::{RepoCitation, Stamp, map_citation, tracking};
    use sealmap_model::SourceSet;
    use sealmap_rust::{RustOptions, extract};

    fn model(lib: &str) -> Model {
        let mut s = SourceSet::new();
        s.insert("Cargo.toml", "[package]\nname = \"shop\"").unwrap();
        s.insert("src/lib.rs", lib).unwrap();
        Model::from_extraction(&extract(&s, &RustOptions::default()))
    }

    fn set(v: &[&str]) -> BTreeSet<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn track(m: &Model, line: u32) -> Tracking {
        let rc = [RepoCitation {
            file: "src/lib.rs".into(),
            mapped: map_citation("src/lib.rs", line, &Stamp::Model { model: m, file_exists: true }),
        }];
        tracking(&set(&["src/lib.rs"]), &rc)
    }

    /// `run` (lines 1–5) calls `load` then `save`.
    const V1: &str = "pub fn run(x: u8) -> u8 {\n    let a = load(x);\n    let b = a + 1;\n    save(b)\n}\n\npub fn load(x: u8) -> u8 {\n    x\n}\n\npub fn save(x: u8) -> u8 {\n    x\n}\n\npub fn audit(x: u8) -> u8 {\n    x\n}\n";
    const RUN: &str = "sym:cargo shop . run().";

    /// T_file^seq, T_sym^seq and T_flow for an edit of V1 into `v2`, citing `run`.
    fn flags(v2: &str) -> (bool, bool, FlowFlags) {
        let (p, c) = (model(V1), model(v2));
        let t = track(&p, 2);
        assert_eq!(t.symbols.iter().next().unwrap().0, RUN);
        let changed = set(&["src/lib.rs"]);
        let e0 = counting::flag(&t, &changed, &p, &c);
        (e0.file, e0.sym, flag(&t, &changed, &p, &c))
    }

    #[test]
    fn the_flow_lists_calls_in_source_order_with_their_resolution() {
        let m = model(V1);
        let f = &m.flows[RUN];
        assert_eq!(
            f.calls,
            [
                (Res::Exact, "sym:cargo shop . load().".to_string()),
                (Res::Exact, "sym:cargo shop . save().".to_string())
            ]
        );
        assert_eq!(f.hash, flow_hash(&f.sig, &f.calls));
        assert!(canonical(&f.sig, &f.calls).starts_with("sig blake3-16:"));
        assert!(m.flows["sym:cargo shop . audit()."].calls.is_empty());
    }

    #[test]
    fn a_body_edit_that_changes_no_call_does_not_flag() {
        let (file, sym, f) = flags(&V1.replace("a + 1", "a * 2 + 7"));
        assert!(file && sym, "the body changed, so T_sym^seq flags");
        assert!(!f.flow && f.changed.is_empty(), "{f:?}");
    }

    #[test]
    fn a_reordered_call_flags() {
        let v2 = V1
            .replace("    let a = load(x);\n    let b = a + 1;\n    save(b)\n", "    let b = save(x);\n    load(b)\n");
        let (_, _, f) = flags(&v2);
        assert!(f.flow && f.by_flow);
        let (p, c) = (model(V1), model(&v2));
        assert_eq!(driver(RUN, &p, &c), Some(Driver::Reordered));
    }

    #[test]
    fn an_added_call_flags() {
        let v2 = V1.replace("    let b = a + 1;\n", "    let b = audit(a);\n");
        let (_, _, f) = flags(&v2);
        assert_eq!(f.changed, [(RUN.to_string(), FlowChange::Hash)]);
        let (p, c) = (model(V1), model(&v2));
        assert_eq!(driver(RUN, &p, &c), Some(Driver::Calls));
        let (added, removed) = call_diff(&p.flows[RUN].calls, &c.flows[RUN].calls);
        assert_eq!(added, ["exact sym:cargo shop . audit()."]);
        assert!(removed.is_empty());
    }

    #[test]
    fn a_changed_callee_flags() {
        let v2 = V1.replace("    save(b)\n", "    audit(b)\n");
        let (_, _, f) = flags(&v2);
        assert!(f.flow);
        let (p, c) = (model(V1), model(&v2));
        let (added, removed) = call_diff(&p.flows[RUN].calls, &c.flows[RUN].calls);
        assert_eq!((added.len(), removed.len()), (1, 1));
    }

    #[test]
    fn a_signature_change_flags() {
        let v2 = V1.replace("pub fn run(x: u8) -> u8", "pub fn run(x: u8, _y: bool) -> u8");
        let (_, _, f) = flags(&v2);
        assert!(f.flow);
        assert_eq!(driver(RUN, &model(V1), &model(&v2)), Some(Driver::Signature));
    }

    #[test]
    fn a_removed_symbol_flags() {
        let v2 = V1.replace("pub fn run(", "pub fn go(");
        let (_, _, f) = flags(&v2);
        assert_eq!(f.changed, [(RUN.to_string(), FlowChange::OneSided)]);
        assert_eq!(driver(RUN, &model(V1), &model(&v2)), Some(Driver::OneSided));
    }

    #[test]
    fn a_callee_body_edit_is_not_a_flow_change() {
        // The callee's own body changes; the caller's calls do not.
        let (_, sym, f) =
            flags(&V1.replace("pub fn load(x: u8) -> u8 {\n    x\n", "pub fn load(x: u8) -> u8 {\n    x + 1\n"));
        assert!(!sym && !f.flow);
    }

    #[test]
    fn a_fallback_file_flags_and_absent_at_both_is_sensitivity_only() {
        let p = model(V1);
        let rc = [
            RepoCitation {
                file: "flake.nix".into(),
                mapped: map_citation("flake.nix", 3, &Stamp::Model { model: &p, file_exists: true }),
            },
            RepoCitation {
                file: "src/lib.rs".into(),
                mapped: crate::mapping::Mapped::Symbol { id: "sym:gone".into(), module: false },
            },
        ];
        let t = tracking(&set(&["flake.nix", "src/lib.rs"]), &rc);
        let f = flag(&t, &set(&["flake.nix"]), &p, &p);
        assert!(f.flow && f.by_fallback && !f.by_flow);
        let g = flag(&t, &set(&["src/lib.rs"]), &p, &p);
        assert!(!g.flow && g.absent_both);
    }

    #[test]
    fn drift_is_a_flow_change_with_unchanged_fingerprints() {
        // `load` is defined in the crate at P and gone at C, so the call in
        // `run` resolves differently while `run`'s own text is unchanged.
        let v2 = V1.replace("pub fn load(x: u8) -> u8 {\n    x\n}\n", "");
        let (p, c) = (model(V1), model(&v2));
        assert_ne!(p.flows[RUN].hash, c.flows[RUN].hash);
        assert_eq!(driver(RUN, &p, &c), Some(Driver::Drift));
    }
}
