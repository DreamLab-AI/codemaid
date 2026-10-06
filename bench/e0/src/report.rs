//! `results.json` and `RESULTS.md`. Both are pure functions of the run, so a
//! re-run on the same pins writes the same bytes.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::counting::Flags;
use crate::judge::Sample;
use crate::mapping::{Fallback, Mapped};
use crate::stats::{self, Bootstrap};
use crate::topics::{Resolved, Topic};
use crate::{RepoRun, change_label, fallback_label};

/// The pre-registered primary threshold.
pub const R_THRESHOLD: f64 = 2.0;
/// The pre-registered hidden-change ceiling.
pub const HIDDEN_CEILING: f64 = 0.10;

fn round(x: f64) -> Value {
    if x.is_finite() { json!((x * 10_000.0).round() / 10_000.0) } else { Value::Null }
}

fn opt(x: Option<f64>) -> Value {
    x.map_or(Value::Null, round)
}

fn share(n: usize, d: usize) -> Value {
    if d == 0 { Value::Null } else { round(n as f64 / d as f64) }
}

fn boot(b: &Bootstrap) -> Value {
    json!({"resamples": b.resamples, "seed": b.seed, "ci95": [round(b.lo), round(b.hi)], "infinite_resamples": b.infinite,
           "method": "percentile, nearest rank (250th and 9750th of 10000 sorted)"})
}

fn summary_block(file: &[u32], sym: &[u32], hop: &[u32]) -> Value {
    let sum = |v: &[u32]| v.iter().map(|&x| u64::from(x)).sum::<u64>();
    json!({
        "commits": file.len(),
        "sum": {"t_file": sum(file), "t_sym": sum(sym), "t_hop": sum(hop)},
        "median": {"t_file": round(stats::median(file)), "t_sym": round(stats::median(sym)), "t_hop": round(stats::median(hop))},
        "p90": {"t_file": stats::p90(file), "t_sym": stats::p90(sym), "t_hop": stats::p90(hop)},
        "r_sym": opt(stats::ratio(file, sym)),
        "r_hop": opt(stats::ratio(file, hop)),
        "bootstrap_r_sym": boot(&stats::bootstrap(file, sym)),
        "bootstrap_r_hop": boot(&stats::bootstrap(file, hop)),
    })
}

fn series_of<'a>(rows: impl Iterator<Item = &'a [(usize, Flags)]>) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut f = Vec::new();
    let mut s = Vec::new();
    let mut h = Vec::new();
    for flags in rows {
        f.push(flags.iter().filter(|(_, x)| x.file).count() as u32);
        s.push(flags.iter().filter(|(_, x)| x.sym).count() as u32);
        h.push(flags.iter().filter(|(_, x)| x.hop).count() as u32);
    }
    (f, s, h)
}

fn series(r: &RepoRun) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    series_of(r.rows.iter().map(|x| x.flags.as_slice()))
}

fn series_mermaid(r: &RepoRun) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    series_of(r.rows.iter().map(|x| x.flags_mermaid.as_slice()))
}

fn series_rust(r: &RepoRun) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    series_of(r.rows.iter().map(|x| x.flags_rust.as_slice()))
}

/// Why each T_sym flag was set. A flag can have several reasons; `only_*`
/// counts flags with exactly that one.
fn reasons(r: &RepoRun) -> Value {
    let (mut n, mut sym, mut fb, mut unc, mut abs) = (0, 0, 0, 0, 0);
    let (mut only_sym, mut only_fb, mut only_unc) = (0, 0, 0);
    let mut file_not_sym = 0;
    for row in &r.rows {
        for (_, x) in &row.flags {
            if x.file && !x.sym {
                file_not_sym += 1;
            }
            if !x.sym {
                continue;
            }
            n += 1;
            sym += usize::from(x.by_symbol);
            fb += usize::from(x.by_fallback_citation);
            unc += usize::from(x.by_uncited_source);
            abs += usize::from(x.by_absent_both);
            let k = [x.by_symbol || x.by_absent_both, x.by_fallback_citation, x.by_uncited_source];
            if k.iter().filter(|b| **b).count() == 1 {
                only_sym += usize::from(k[0]);
                only_fb += usize::from(k[1]);
                only_unc += usize::from(k[2]);
            }
        }
    }
    json!({
        "t_sym_flags": n, "t_file_not_t_sym": file_not_sym,
        "with_reason": {"cited_symbol_changed": sym, "absent_at_both": abs, "fallback_citation_file_changed": fb, "uncited_source_changed": unc},
        "only_reason": {"cited_symbol": only_sym, "fallback_citation_file": only_fb, "uncited_source": only_unc},
    })
}

fn coverage(all: &[Topic], r: &RepoRun) -> Value {
    let mut total = 0;
    let mut item = 0;
    let mut module = 0;
    let mut by_reason: BTreeMap<&str, usize> = BTreeMap::new();
    let (mut sym_only, mut partly, mut fully) = (0, 0, 0);
    let mut uncited_sources = 0;
    let mut topics = Vec::new();
    let mut mostly = Vec::new();
    for t in &r.tracked {
        let n = t.cites.len();
        let fb = t.cites.iter().filter(|(_, _, m)| crate::is_fallback(m)).count();
        let modu = t.cites.iter().filter(|(_, _, m)| matches!(m, Mapped::Symbol { module: true, .. })).count();
        total += n;
        module += modu;
        item += n - fb - modu;
        let mut reasons: BTreeMap<&str, usize> = BTreeMap::new();
        for (_, _, m) in &t.cites {
            if let Mapped::Fallback(f) = m {
                *by_reason.entry(fallback_label(*f)).or_default() += 1;
                *reasons.entry(fallback_label(*f)).or_default() += 1;
            }
        }
        let uncited = t.tracking.sources.iter().filter(|s| !t.cites.iter().any(|(_, f, _)| f == *s)).count();
        uncited_sources += uncited;
        let class = if t.tracking.symbols.is_empty() {
            fully += 1;
            "fully_fallback"
        } else if t.tracking.files().is_empty() {
            sym_only += 1;
            "symbol_only"
        } else {
            partly += 1;
            "partly_fallback"
        };
        let id = &all[t.topic].id;
        if n > 0 && fb * 2 > n {
            mostly.push(json!({"topic": id, "citations": n, "fallback": fb}));
        }
        topics.push(json!({
            "topic": id, "class": class, "stamp": t.stamp, "sources": t.tracking.sources.len(), "uncited_sources": uncited,
            "citations": n, "symbol_item": n - fb - modu, "symbol_module": modu, "fallback": fb, "fallback_by_reason": reasons,
            "cited_symbols": t.tracking.symbols.len(), "whole_files_fallback": t.tracking.fallback_files.len(), "whole_files_uncited": t.tracking.uncited_files.len(),
        }));
    }
    let nt = r.tracked.len();
    let fallback: usize = by_reason.values().sum();
    json!({
        "citations": {
            "total": total, "symbol_item": item, "symbol_module": module, "fallback": fallback,
            "fallback_share": share(fallback, total), "module_share": share(module, total),
            "fallback_by_reason": by_reason,
        },
        "topics": {
            "tracked": nt, "symbol_only": sym_only, "partly_fallback": partly, "fully_fallback": fully,
            "share_partly_or_fully": share(partly + fully, nt), "share_fully": share(fully, nt),
        },
        "uncited_sources": uncited_sources,
        "topics_mostly_fallback": mostly,
        "per_topic": topics,
        "_unmapped": {
            "unresolvable_citations_in_area": all.iter().filter(|t| t.area == r.key.as_str()).flat_map(|t| &t.citations)
                .filter(|c| !matches!(c.resolved, Resolved::Source(_))).count(),
            "citations_to_other_repositories_in_area": r.other_repo_citations,
        },
    })
}

fn repo_json(all: &[Topic], r: &RepoRun) -> Value {
    let (f, s, h) = series(r);
    let mermaid = {
        let (a, b, c) = series_mermaid(r);
        summary_block(&a, &b, &c)
    };
    let rust = {
        let (a, b, c) = series_rust(r);
        summary_block(&a, &b, &c)
    };
    let id = |ti: usize| all[ti].id.clone();
    let mut sym_flags = 0;
    let mut only_absent = 0;
    let mut events: BTreeMap<&str, usize> = BTreeMap::new();
    let mut sym_not_file = Vec::new();
    let mut hop_not_file = 0;
    let mut commits = Vec::new();
    let mut file_flags = 0;
    let mut precedes = 0;
    for row in &r.rows {
        let mut ff = Vec::new();
        let mut sf = Vec::new();
        let mut hf = Vec::new();
        for (ti, x) in &row.flags {
            if x.file {
                ff.push(id(*ti));
                file_flags += 1;
                if r.precedes_stamp.get(&(row.sha.clone(), *ti)).copied().unwrap_or(false) {
                    precedes += 1;
                }
            }
            if x.sym {
                sf.push(id(*ti));
                sym_flags += 1;
                if x.by_absent_both && !x.by_symbol && !x.by_file_level {
                    only_absent += 1;
                }
                if !x.file {
                    sym_not_file.push(json!({
                        "commit": row.sha, "topic": id(*ti),
                        "changed_symbols": x.changed_symbols.iter().map(|(i, c)| json!([i, change_label(*c)])).collect::<Vec<_>>(),
                    }));
                }
            }
            if x.hop {
                hf.push(id(*ti));
                if !x.file {
                    hop_not_file += 1;
                }
            }
            for (_, c) in &x.changed_symbols {
                *events.entry(change_label(*c)).or_default() += 1;
            }
        }
        // Why each T_hop-flagged topic was flagged: enough to audit any flag by hand.
        let why: serde_json::Map<String, Value> = row
            .flags
            .iter()
            .filter(|(_, x)| x.hop)
            .map(|(ti, x)| {
                let mut reasons = Vec::new();
                if x.by_fallback_citation {
                    reasons.push("fallback_citation_file");
                }
                if x.by_uncited_source {
                    reasons.push("uncited_source");
                }
                if x.by_symbol {
                    reasons.push("cited_symbol");
                }
                if x.by_absent_both {
                    reasons.push("absent_at_both");
                }
                if !x.sym {
                    reasons.push("callee_only");
                }
                (
                    id(*ti),
                    json!({
                        "reasons": reasons,
                        "symbols": x.changed_symbols.iter().map(|(i, c)| json!([i, change_label(*c)])).collect::<Vec<_>>(),
                        "callees": x.changed_callees.len(),
                    }),
                )
            })
            .collect();
        commits.push(json!({
            "sha": row.sha, "parent": row.parent, "date": row.date, "subject": row.subject,
            "t_file": ff.len(), "t_sym": sf.len(), "t_hop": hf.len(), "file": ff, "sym": sf, "hop": hf, "why": why,
        }));
    }
    let newest = r.rows.first().map(|x| json!({"sha": x.sha, "date": x.date}));
    let oldest = r.rows.last().map(|x| json!({"sha": x.sha, "date": x.date}));
    json!({
        "head": r.head,
        "window": {
            "eligible": r.rows.len(), "walked_first_parent": r.walked, "ineligible_touch_no_cited_file": r.ineligible,
            "merges_skipped": r.merges, "roots_skipped": r.roots, "newest": newest, "oldest": oldest,
        },
        "endpoints": summary_block(&f, &s, &h),
        "t_sym_reasons": reasons(r),
        "sensitivity_mermaid_only": mermaid,
        "exploratory_rust_only": rust,
        "exploratory_ek_by_kind": ek_block(r),
        "coverage": coverage(all, r),
        "conservative_absent_at_both": {
            "t_sym_flags": sym_flags, "flags_set_only_by_absent_at_both": only_absent,
            "share_of_t_sym_flags": share(only_absent, sym_flags), "symbol_change_events": events,
        },
        "checks": {
            "t_sym_without_t_file": sym_not_file, "t_hop_without_t_file": hop_not_file,
            "t_file_flags_on_commits_preceding_the_topic_stamp": precedes, "t_file_flags": file_flags,
        },
        "commits": commits,
    })
}

/// Display order of the kinds; any other kind follows, then `uncited`.
const KIND_ORDER: [&str; 6] = ["sequenceDiagram", "flowchart", "classDiagram", "stateDiagram-v2", "erDiagram", "prose"];

fn kind_rank(k: &str) -> (usize, String) {
    let i = KIND_ORDER.iter().position(|x| *x == k).unwrap_or(if k == crate::kinds::UNCITED { 99 } else { 50 });
    (i, k.to_string())
}

/// Exploratory (EK): citations and flags per diagram kind for one repository.
fn ek_block(r: &RepoRun) -> Value {
    let mut cites: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    let mut topics_with: BTreeMap<String, usize> = BTreeMap::new();
    for t in &r.tracked {
        let mut seen = std::collections::BTreeSet::new();
        for (c, _, m) in &t.cites {
            let e = cites.entry(c.kind.clone()).or_default();
            e.0 += 1;
            e.1 += usize::from(crate::is_fallback(m));
            e.2 += usize::from(matches!(m, Mapped::Symbol { module: true, .. }));
            seen.insert(c.kind.clone());
        }
        for k in seen {
            *topics_with.entry(k).or_default() += 1;
        }
    }
    // Per kind, per commit: topics flagged only through that kind.
    let mut kinds: Vec<String> = cites.keys().cloned().chain([crate::kinds::UNCITED.to_string()]).collect();
    kinds.sort_by_key(|k| kind_rank(k));
    let n = r.rows.len();
    let mut only_file: BTreeMap<String, Vec<u32>> = kinds.iter().map(|k| (k.clone(), vec![0; n])).collect();
    let mut only_sym = only_file.clone();
    let (mut mixed_file, mut mixed_sym) = (0usize, 0usize);
    for (i, row) in r.rows.iter().enumerate() {
        for (_, a) in &row.kind_attr {
            if a.file.len() == 1 {
                only_file.get_mut(a.file.iter().next().unwrap()).unwrap()[i] += 1;
            } else if a.file.len() > 1 {
                mixed_file += 1;
            }
            if a.sym.len() == 1 {
                only_sym.get_mut(a.sym.iter().next().unwrap()).unwrap()[i] += 1;
            } else if a.sym.len() > 1 {
                mixed_sym += 1;
            }
        }
    }
    let sum = |v: &[u32]| v.iter().map(|&x| u64::from(x)).sum::<u64>();
    let per_kind: Vec<Value> = kinds
        .iter()
        .map(|k| {
            let (c, fb, md) = cites.get(k).copied().unwrap_or_default();
            let (f, s) = (&only_file[k], &only_sym[k]);
            json!({
                "kind": k, "topics_citing": topics_with.get(k).copied().unwrap_or(0),
                "citations": c, "fallback": fb, "fallback_share": share(fb, c), "symbol_module": md,
                "flagged_only_through_kind": {
                    "t_file": {"sum": sum(f), "median": round(stats::median(f)), "p90": stats::p90(f)},
                    "t_sym": {"sum": sum(s), "median": round(stats::median(s)), "p90": stats::p90(s)},
                    "r": opt(stats::ratio(f, s)),
                },
            })
        })
        .collect();
    json!({
        "label": "Exploratory (EK): not an E0 endpoint; registered in docs/evidence/EK/PREREG.md (fdd8207)",
        "per_kind": per_kind,
        "flagged_through_several_kinds": {"t_file": mixed_file, "t_sym": mixed_sym},
    })
}

/// The whole results document.
pub fn results_json(all: &[Topic], runs: &[RepoRun], sample: &Sample) -> Value {
    let mut repos = serde_json::Map::new();
    let (mut pf, mut ps, mut ph) = (Vec::new(), Vec::new(), Vec::new());
    let (mut mf, mut ms, mut mh) = (Vec::new(), Vec::new(), Vec::new());
    for r in runs {
        repos.insert(r.key.as_str().into(), repo_json(all, r));
        let (f, s, h) = series(r);
        pf.extend(f);
        ps.extend(s);
        ph.extend(h);
        let (f, s, h) = series_mermaid(r);
        mf.extend(f);
        ms.extend(s);
        mh.extend(h);
    }
    let vc = &runs[0];
    let (vf, vs, _) = series(vc);
    let r = stats::ratio(&vf, &vs);
    let e1 = r.is_some_and(|x| x >= R_THRESHOLD);
    let e1_text = if e1 { "PASS" } else { "FAIL" };
    let overall = if e1 { "UNDETERMINED until endpoint 4 is judged" } else { "DOES NOT HOLD (endpoint 1 failed)" };
    json!({
        "experiment": "E0",
        "prereg": "docs/evidence/E0/PREREG.md",
        "pins": {"corpus": crate::CORPUS_SHA, "visionclaw": crate::VISIONCLAW_SHA, "agentbox": crate::AGENTBOX_SHA,
                 "sealmap": "0.2.0 crates in-tree, sealmap_rust::extract_dir with RustOptions::default()"},
        "corpus": {
            "topics": all.len(),
            "by_area": crate::AREAS.iter().map(|a| (a.to_string(), json!(all.iter().filter(|t| t.area == *a).count()))).collect::<serde_json::Map<_, _>>(),
            "citations": all.iter().map(|t| t.citations.len()).sum::<usize>(),
            "prose_citations": all.iter().flat_map(|t| &t.citations).filter(|c| c.prose).count(),
            "bare_refs_unbound": all.iter().map(|t| t.unbound_bare).sum::<usize>(),
        },
        "repos": repos,
        "pooled": summary_block(&pf, &ps, &ph),
        "pooled_mermaid_only": summary_block(&mf, &ms, &mh),
        "endpoint4": sample.to_json(),
        "verdict": {
            "endpoint1": {"r": opt(r), "threshold": R_THRESHOLD, "result": e1_text},
            "endpoint4": {"threshold": HIDDEN_CEILING, "result": "PENDING"},
            "e0": overall,
        },
    })
}

fn g<'a>(v: &'a Value, path: &[&str]) -> &'a Value {
    path.iter().fold(v, |acc, k| &acc[*k])
}

fn num(v: &Value) -> String {
    match v {
        Value::Null => "n/a".into(),
        Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.fract() == 0.0 { format!("{f:.0}") } else { format!("{f:.2}") }
        }
        other => other.to_string(),
    }
}

/// A short console summary.
pub fn summary(v: &Value) -> String {
    let mut s = String::new();
    for k in ["visionclaw", "agentbox"] {
        let e = g(v, &["repos", k, "endpoints"]);
        s.push_str(&format!(
            "{k}: R={} CI95=[{}, {}] R_hop={} median file/sym/hop={}/{}/{} p90={}/{}/{}\n",
            num(&e["r_sym"]),
            num(&e["bootstrap_r_sym"]["ci95"][0]),
            num(&e["bootstrap_r_sym"]["ci95"][1]),
            num(&e["r_hop"]),
            num(&e["median"]["t_file"]),
            num(&e["median"]["t_sym"]),
            num(&e["median"]["t_hop"]),
            num(&e["p90"]["t_file"]),
            num(&e["p90"]["t_sym"]),
            num(&e["p90"]["t_hop"]),
        ));
    }
    s.push_str(&format!("verdict: {}\n", v["verdict"]["e0"].as_str().unwrap_or("")));
    s
}

/// `RESULTS.md`, with the PREREG's amendments copied in verbatim.
pub fn results_md(v: &Value, prereg: &str) -> String {
    let mut o = String::new();
    o.push_str("<!-- GENERATED by `cargo run -p sealmap-bench-e0 -- run` — do not edit by hand -->\n");
    o.push_str("# E0 results: file-level against symbol-level staleness\n\n");
    o.push_str("Pre-registration: [PREREG.md](PREREG.md). Machine-readable: [results.json](results.json). ");
    o.push_str("Wall-clock: [TIMING.md](TIMING.md) (not part of the reproducible output).\n\n");
    o.push_str("| Input | Revision |\n|---|---|\n");
    for k in ["corpus", "visionclaw", "agentbox", "sealmap"] {
        o.push_str(&format!("| {k} | `{}` |\n", v["pins"][k].as_str().unwrap_or("")));
    }
    let ver = &v["verdict"];
    o.push_str("\n## Verdict\n\n");
    o.push_str(&format!(
        "- **Endpoint 1 (primary):** R = ΣT_file / ΣT_sym over the {} eligible VisionClaw commits = **{}** (bootstrap 95% CI {}–{}); threshold ≥ {}: **{}**.\n",
        num(&v["repos"]["visionclaw"]["endpoints"]["commits"]),
        num(&ver["endpoint1"]["r"]),
        num(&v["repos"]["visionclaw"]["endpoints"]["bootstrap_r_sym"]["ci95"][0]),
        num(&v["repos"]["visionclaw"]["endpoints"]["bootstrap_r_sym"]["ci95"][1]),
        num(&ver["endpoint1"]["threshold"]),
        ver["endpoint1"]["result"].as_str().unwrap_or(""),
    ));
    let e4 = &v["endpoint4"];
    o.push_str(&format!(
        "- **Endpoint 4 (hidden-change precision):** **PENDING**. {} pairs drawn ({} VisionClaw, {} agentbox) from {} + {} eligible pairs; prompts are in [`judge/`](judge/). No verdict has been produced in the context that drew the sample.\n",
        num(&e4["drawn"]["total"]),
        num(&e4["drawn"]["visionclaw"]),
        num(&e4["drawn"]["agentbox"]),
        num(&e4["population"]["visionclaw"]),
        num(&e4["population"]["agentbox"]),
    ));
    o.push_str(&format!("- **E0:** {}.\n", ver["e0"].as_str().unwrap_or("")));

    o.push_str("\n## Endpoints 1–2: topics flagged per commit\n\n");
    o.push_str("| | commits | ΣT_file | ΣT_sym | ΣT_hop | median T_file | median T_sym | median T_hop | p90 T_file | p90 T_sym | p90 T_hop | R (sym) | 95% CI | R (hop) | 95% CI |\n");
    o.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    let rows: [(&str, &Value); 8] = [
        ("VisionClaw", &v["repos"]["visionclaw"]["endpoints"]),
        ("agentbox", &v["repos"]["agentbox"]["endpoints"]),
        ("pooled", &v["pooled"]),
        ("VisionClaw, mermaid citations only", &v["repos"]["visionclaw"]["sensitivity_mermaid_only"]),
        ("agentbox, mermaid citations only", &v["repos"]["agentbox"]["sensitivity_mermaid_only"]),
        ("pooled, mermaid citations only", &v["pooled_mermaid_only"]),
        ("VisionClaw, Rust sources only (exploratory)", &v["repos"]["visionclaw"]["exploratory_rust_only"]),
        ("agentbox, Rust sources only (exploratory)", &v["repos"]["agentbox"]["exploratory_rust_only"]),
    ];
    for (name, e) in rows {
        o.push_str(&format!(
            "| {name} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {}–{} | {} | {}–{} |\n",
            num(&e["commits"]),
            num(&e["sum"]["t_file"]),
            num(&e["sum"]["t_sym"]),
            num(&e["sum"]["t_hop"]),
            num(&e["median"]["t_file"]),
            num(&e["median"]["t_sym"]),
            num(&e["median"]["t_hop"]),
            num(&e["p90"]["t_file"]),
            num(&e["p90"]["t_sym"]),
            num(&e["p90"]["t_hop"]),
            num(&e["r_sym"]),
            num(&e["bootstrap_r_sym"]["ci95"][0]),
            num(&e["bootstrap_r_sym"]["ci95"][1]),
            num(&e["r_hop"]),
            num(&e["bootstrap_r_hop"]["ci95"][0]),
            num(&e["bootstrap_r_hop"]["ci95"][1]),
        ));
    }
    o.push_str(
        "\nMedians are the middle value (mean of the two middle values for an even count); p90 is nearest rank. ",
    );
    o.push_str("Only VisionClaw's R carries a threshold. agentbox and pooled CIs are reported, not gated. ");
    o.push_str("The *mermaid citations only* rows are the sensitivity reading of amendment #9 (prose citations ignored); they are not an endpoint. The *Rust sources only* rows are exploratory and post hoc (amendment #10): both methods restricted to each topic's `.rs` sources and citations, over the same commits, to separate what symbol gating does on code sealmap-rust can read from what file-level tracking of non-Rust sources contributes.\n");
    o.push_str("\n### Why T_sym flags a topic\n\nA flag can have more than one reason; *only* counts flags with exactly one.\n\n");
    o.push_str("| | T_sym flags | cited symbol changed (hash or one-sided) | cited symbol absent at both, file changed | fallback-citation file changed | uncited source changed | only cited symbol | only fallback citation | only uncited source | T_file but not T_sym |\n|---|---|---|---|---|---|---|---|---|---|\n");
    for k in ["visionclaw", "agentbox"] {
        let r = &v["repos"][k]["t_sym_reasons"];
        o.push_str(&format!(
            "| {k} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            num(&r["t_sym_flags"]),
            num(&r["with_reason"]["cited_symbol_changed"]),
            num(&r["with_reason"]["absent_at_both"]),
            num(&r["with_reason"]["fallback_citation_file_changed"]),
            num(&r["with_reason"]["uncited_source_changed"]),
            num(&r["only_reason"]["cited_symbol"]),
            num(&r["only_reason"]["fallback_citation_file"]),
            num(&r["only_reason"]["uncited_source"]),
            num(&r["t_file_not_t_sym"]),
        ));
    }

    o.push_str("\n## Commit window\n\n| | eligible | walked (first parent) | ineligible: touch no cited file | merges skipped | newest | oldest |\n|---|---|---|---|---|---|---|\n");
    for k in ["visionclaw", "agentbox"] {
        let w = &v["repos"][k]["window"];
        o.push_str(&format!(
            "| {k} | {} | {} | {} | {} | `{}` {} | `{}` {} |\n",
            num(&w["eligible"]),
            num(&w["walked_first_parent"]),
            num(&w["ineligible_touch_no_cited_file"]),
            num(&w["merges_skipped"]),
            &w["newest"]["sha"].as_str().unwrap_or("")[..10.min(w["newest"]["sha"].as_str().unwrap_or("").len())],
            w["newest"]["date"].as_str().unwrap_or(""),
            &w["oldest"]["sha"].as_str().unwrap_or("")[..10.min(w["oldest"]["sha"].as_str().unwrap_or("").len())],
            w["oldest"]["date"].as_str().unwrap_or(""),
        ));
    }

    o.push_str("\n## Endpoint 3: coverage honesty\n\n");
    o.push_str("A citation is *symbol (item)* when its innermost symbol is a function, method, type, trait, const or the like; *symbol (module)* when the only symbol containing the line is a module (a file's root module spans the whole file, and its body hash folds in every item, so these behave close to file level); *fallback* per the PREREG. Topic classes: *symbol only* (every in-repo source is cited and every citation maps to a symbol), *fully fallback* (no citation maps to a symbol), *partly* (the rest).\n\n");
    o.push_str("| | citations | symbol (item) | symbol (module) | fallback | fallback share | topics tracked | symbol only | partly | fully fallback | share partly or fully | uncited sources |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for k in ["visionclaw", "agentbox"] {
        let c = &v["repos"][k]["coverage"];
        o.push_str(&format!(
            "| {k} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            num(&c["citations"]["total"]),
            num(&c["citations"]["symbol_item"]),
            num(&c["citations"]["symbol_module"]),
            num(&c["citations"]["fallback"]),
            num(&c["citations"]["fallback_share"]),
            num(&c["topics"]["tracked"]),
            num(&c["topics"]["symbol_only"]),
            num(&c["topics"]["partly_fallback"]),
            num(&c["topics"]["fully_fallback"]),
            num(&c["topics"]["share_partly_or_fully"]),
            num(&c["uncited_sources"]),
        ));
    }
    o.push_str("\nFallback citations by reason:\n\n| | ");
    let reasons = [
        Fallback::NotRust,
        Fallback::NoStamp,
        Fallback::StampUnknown,
        Fallback::AbsentAtStamp,
        Fallback::ParseError,
        Fallback::NoSymbol,
    ];
    o.push_str(&reasons.iter().map(|r| fallback_label(*r)).collect::<Vec<_>>().join(" | "));
    o.push_str(" |\n|---|");
    o.push_str(&"---|".repeat(reasons.len()));
    o.push('\n');
    for k in ["visionclaw", "agentbox"] {
        let c = &v["repos"][k]["coverage"]["citations"]["fallback_by_reason"];
        o.push_str(&format!(
            "| {k} | {} |\n",
            reasons.iter().map(|r| num(c.get(fallback_label(*r)).unwrap_or(&json!(0)))).collect::<Vec<_>>().join(" | ")
        ));
    }
    for k in ["visionclaw", "agentbox"] {
        let m = v["repos"][k]["coverage"]["topics_mostly_fallback"].as_array().cloned().unwrap_or_default();
        if !m.is_empty() {
            o.push_str(&format!("\n{k}: topics whose citations are mostly (> 50%) fallback: "));
            o.push_str(
                &m.iter()
                    .map(|x| {
                        format!(
                            "{} ({}/{})",
                            x["topic"].as_str().unwrap_or(""),
                            num(&x["fallback"]),
                            num(&x["citations"])
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            o.push_str(".\n");
        }
    }
    let corpus = &v["corpus"];
    o.push_str(&format!(
        "\nCorpus: {} topics, {} citations read, {} of them in prose outside the mermaid blocks (amendment #9); {} bare `:N` refs had no bindable path and were not read.\n",
        num(&corpus["topics"]),
        num(&corpus["citations"]),
        num(&corpus["prose_citations"]),
        num(&corpus["bare_refs_unbound"]),
    ));
    for k in ["visionclaw", "agentbox"] {
        let u = &v["repos"][k]["coverage"]["_unmapped"];
        o.push_str(&format!(
            "{k}-area topics: {} citations resolve to no single `sources:` entry (excluded from both methods); {} resolve to sources in another repository.\n",
            num(&u["unresolvable_citations_in_area"]),
            num(&u["citations_to_other_repositories_in_area"]),
        ));
    }

    o.push_str("\n## Conservative rule and consistency checks\n\n| | T_sym flags | set only by absent-at-both | share | T_sym without T_file | T_hop without T_file | T_file flags on commits that precede the topic's stamp |\n|---|---|---|---|---|---|---|\n");
    for k in ["visionclaw", "agentbox"] {
        let c = &v["repos"][k]["conservative_absent_at_both"];
        let ch = &v["repos"][k]["checks"];
        o.push_str(&format!(
            "| {k} | {} | {} | {} | {} | {} | {} of {} |\n",
            num(&c["t_sym_flags"]),
            num(&c["flags_set_only_by_absent_at_both"]),
            num(&c["share_of_t_sym_flags"]),
            ch["t_sym_without_t_file"].as_array().map_or(0, Vec::len),
            num(&ch["t_hop_without_t_file"]),
            num(&ch["t_file_flags_on_commits_preceding_the_topic_stamp"]),
            num(&ch["t_file_flags"]),
        ));
    }
    o.push_str("\nT_sym without T_file can only arise when a cited symbol's definition now lives in, or moved through, a file that is not in the topic's `sources:`; each case is listed in `results.json` under `checks`.\n");

    o.push_str("\n## Endpoint 4: hidden-change sample\n\n");
    o.push_str("Population: (commit, topic) pairs flagged by T_file and not by T_sym. Draw: ChaCha8 seeded 20261006, VisionClaw stratum first, then agentbox, partial Fisher–Yates over the pairs in window order. Each prompt carries the topic text at the corpus pin and the commit's diff restricted to the topic's sources in that repository. Scoring: `e0 score --out docs/evidence/E0` once `judge/<id>.j1.json` (and `.j2.json` for every first-judge yes) exist; see [judge/PROTOCOL.md](judge/PROTOCOL.md).\n\n");
    o.push_str("| id | repo | commit | topic | precedes stamp | diff lines |\n|---|---|---|---|---|---|\n");
    for s in v["endpoint4"]["sample"].as_array().cloned().unwrap_or_default() {
        o.push_str(&format!(
            "| {} | {} | `{}` | {} | {} | {} |\n",
            s["id"].as_str().unwrap_or(""),
            s["repo"].as_str().unwrap_or(""),
            &s["commit"].as_str().unwrap_or("")[..10],
            s["topic"].as_str().unwrap_or(""),
            s["commit_precedes_stamp"],
            num(&s["diff_lines"]),
        ));
    }

    o.push_str("\n## Exploratory (EK): by diagram kind\n\n");
    o.push_str("Not an E0 endpoint, and no E0 number above depends on it. It was registered as exploratory in `docs/evidence/EK/PREREG.md` (commit `fdd8207`). Each citation takes the kind of the mermaid block it sits in (the generator's rule: the first token of the block; `graph` counts as `flowchart`), or `prose` outside every block. ");
    o.push_str("A topic's flag is attributed to the kinds of the citations behind the units that set it. **T_file:** each changed source counts under the kinds of every citation of that file, or under `uncited` if no citation names it. **T_sym:** a changed cited symbol counts under the kinds of the citations that mapped to it, a changed fallback file under the kinds of its fallback citations, and a changed uncited source under `uncited`. ");
    o.push_str("*Only through K* counts (commit, topic) flags whose units all trace to K alone; flags that trace to several kinds are counted apart. Per-commit medians and p90s are over the 100 commits.\n");
    for k in ["visionclaw", "agentbox"] {
        let e = &v["repos"][k]["exploratory_ek_by_kind"];
        o.push_str(&format!("\n### {k}\n\n| kind | topics citing | citations | fallback | fallback share | module | only through kind: ΣT_file | ΣT_sym | R | median T_file / T_sym | p90 T_file / T_sym |\n|---|---|---|---|---|---|---|---|---|---|---|\n"));
        for x in e["per_kind"].as_array().cloned().unwrap_or_default() {
            let f = &x["flagged_only_through_kind"];
            o.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} / {} | {} / {} |\n",
                x["kind"].as_str().unwrap_or(""),
                num(&x["topics_citing"]),
                num(&x["citations"]),
                num(&x["fallback"]),
                num(&x["fallback_share"]),
                num(&x["symbol_module"]),
                num(&f["t_file"]["sum"]),
                num(&f["t_sym"]["sum"]),
                num(&f["r"]),
                num(&f["t_file"]["median"]),
                num(&f["t_sym"]["median"]),
                num(&f["t_file"]["p90"]),
                num(&f["t_sym"]["p90"]),
            ));
        }
        o.push_str(&format!(
            "\nFlags that trace to several kinds: T_file {}, T_sym {}.\n",
            num(&e["flagged_through_several_kinds"]["t_file"]),
            num(&e["flagged_through_several_kinds"]["t_sym"]),
        ));
    }

    o.push_str("\n## Amendments (copied from PREREG.md)\n\n");
    let amend = prereg.split_once("## Amendments").map_or("", |(_, rest)| rest.trim());
    o.push_str(amend);
    o.push('\n');
    o
}
