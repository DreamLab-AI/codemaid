//! E0b `results.json` and `RESULTS.md`: pure functions of the run, so a
//! re-run on the same pins writes the same bytes.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use crate::judge::{MIN_ELIGIBLE_B, Sample};
use crate::mapping::Mapped;
use crate::rcount::{RChange, RFlags};
use crate::region::{Kind, path_label};
use crate::report::{HIDDEN_CEILING, R_THRESHOLD, boot, num, opt, round, share};
use crate::stats;
use crate::topics::Topic;
use crate::{RepoRun, counting::Flags};

/// Drivers listed per repository.
const TOP_DRIVERS: usize = 20;

fn sum(v: &[u32]) -> u64 {
    v.iter().map(|&x| u64::from(x)).sum()
}

fn block(file: &[u32], sym: &[u32], region: &[u32]) -> Value {
    json!({
        "commits": file.len(),
        "sum": {"t_file": sum(file), "t_sym": sum(sym), "t_region": sum(region)},
        "median": {"t_file": round(stats::median(file)), "t_sym": round(stats::median(sym)), "t_region": round(stats::median(region))},
        "p90": {"t_file": stats::p90(file), "t_sym": stats::p90(sym), "t_region": stats::p90(region)},
        "r_region": opt(stats::ratio(file, region)),
        "r_sym": opt(stats::ratio(file, sym)),
        "bootstrap_r_region": boot(&stats::bootstrap(file, region)),
        "bootstrap_r_sym": boot(&stats::bootstrap(file, sym)),
    })
}

type Series = (Vec<u32>, Vec<u32>, Vec<u32>);

fn count_flags(sym: &[(usize, Flags)], reg: &[(usize, RFlags)]) -> (u32, u32, u32) {
    let f = reg.iter().filter(|(_, x)| x.file).count() as u32;
    let s = sym.iter().filter(|(_, x)| x.sym).count() as u32;
    let r = reg.iter().filter(|(_, x)| x.region).count() as u32;
    (f, s, r)
}

/// (T_file, T_sym, T_region) per commit; `rust` restricts to `.rs` sources and citations.
fn series(r: &RepoRun, rust: bool) -> Series {
    let mut out: Series = (Vec::new(), Vec::new(), Vec::new());
    for row in &r.rows {
        let (f, s, g) =
            if rust { count_flags(&row.flags_rust, &row.rflags_rust) } else { count_flags(&row.flags, &row.rflags) };
        out.0.push(f);
        out.1.push(s);
        out.2.push(g);
    }
    out
}

fn change_label(c: RChange) -> &'static str {
    match c {
        RChange::Same => "same",
        RChange::Hash => "hash",
        RChange::OneSided => "one_sided",
        RChange::Missing => "path_missing_at_both",
        RChange::AbsentBothFileChanged => "absent_both_file_changed",
    }
}

/// Why each T_region flag was set.
fn reasons(r: &RepoRun) -> Value {
    let (mut n, mut reg, mut whole, mut fb, mut unc) = (0, 0, 0, 0, 0);
    let (mut only_reg, mut only_whole, mut only_fb, mut only_unc) = (0, 0, 0, 0);
    let mut file_not_region = 0;
    let mut events: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &r.rows {
        for (_, x) in &row.rflags {
            for (_, c) in &x.changed {
                *events.entry(change_label(*c)).or_default() += 1;
            }
            if x.file && !x.region {
                file_not_region += 1;
            }
            if !x.region {
                continue;
            }
            n += 1;
            reg += usize::from(x.by_region);
            whole += usize::from(x.by_whole_symbol);
            fb += usize::from(x.by_fallback_citation);
            unc += usize::from(x.by_uncited_source);
            let k = [x.by_region, x.by_whole_symbol, x.by_fallback_citation, x.by_uncited_source];
            if k.iter().filter(|b| **b).count() == 1 {
                only_reg += usize::from(k[0]);
                only_whole += usize::from(k[1]);
                only_fb += usize::from(k[2]);
                only_unc += usize::from(k[3]);
            }
        }
    }
    json!({
        "t_region_flags": n, "t_file_not_t_region": file_not_region,
        "with_reason": {"cited_region_changed": reg, "whole_symbol_region_changed": whole,
                        "fallback_citation_file_changed": fb, "uncited_source_changed": unc},
        "only_reason": {"cited_region": only_reg, "whole_symbol_region": only_whole,
                        "fallback_citation_file": only_fb, "uncited_source": only_unc},
        "region_change_events": events,
    })
}

/// The share of Rust citations whose region is the whole symbol, and the
/// kinds of the regions the others narrowed to.
fn regions_block(r: &RepoRun) -> Value {
    let (mut rs, mut mapped, mut whole, mut whole_module, mut whole_item, mut unlocated, mut module) =
        (0, 0, 0, 0, 0, 0, 0);
    let mut by_kind: BTreeMap<&str, usize> = Kind::ALL.iter().map(|k| (k.label(), 0)).collect();
    let mut depth: BTreeMap<usize, usize> = BTreeMap::new();
    let mut distinct = BTreeSet::new();
    for t in &r.tracked {
        for ((_, file, m), cr) in t.cites.iter().zip(&t.cite_regions) {
            if !file.ends_with(".rs") {
                continue;
            }
            rs += 1;
            let (Mapped::Symbol { module: is_mod, .. }, Some(cr)) = (m, cr) else { continue };
            mapped += 1;
            module += usize::from(*is_mod);
            unlocated += usize::from(!cr.located);
            distinct.insert((file.clone(), cr.id.clone(), cr.path.clone()));
            *depth.entry(cr.path.len()).or_default() += 1;
            match cr.path.last() {
                None => {
                    whole += 1;
                    if *is_mod {
                        whole_module += 1;
                    } else {
                        whole_item += 1;
                    }
                }
                Some((k, _)) => *by_kind.entry(k.label()).or_default() += 1,
            }
        }
    }
    json!({
        "rust_citations": rs,
        "rust_citations_mapped_to_a_symbol": mapped,
        "of_which_module_symbol": module,
        "region_is_whole_symbol": whole,
        "region_is_whole_symbol_item": whole_item,
        "region_is_whole_symbol_module": whole_module,
        "whole_symbol_share_of_mapped": share(whole, mapped),
        "symbol_node_not_found_at_stamp": unlocated,
        "innermost_region_kind": by_kind,
        "path_depth": depth.iter().map(|(d, n)| (d.to_string(), json!(n))).collect::<serde_json::Map<_, _>>(),
        "distinct_regions": distinct.len(),
    })
}

/// What still sets T_region flags: every cited region, whole symbol and
/// whole file that changed in a flagged (commit, topic) pair, ranked by the
/// number of such pairs. `sole` counts pairs it flagged alone.
fn drivers(all: &[Topic], r: &RepoRun) -> Value {
    let module_ids: BTreeSet<&String> = r
        .tracked
        .iter()
        .flat_map(|t| &t.cites)
        .filter_map(|(_, _, m)| match m {
            Mapped::Symbol { id, module: true } => Some(id),
            _ => None,
        })
        .collect();
    #[derive(Default)]
    struct D {
        pairs: usize,
        sole: usize,
        commits: BTreeSet<String>,
        topics: BTreeSet<String>,
    }
    let mut by: BTreeMap<(String, String, String, String), D> = BTreeMap::new();
    let mut by_file: BTreeMap<String, (usize, BTreeSet<String>)> = BTreeMap::new();
    let mut by_class: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut flagged = 0;
    for row in &r.rows {
        for (ti, x) in &row.rflags {
            if !x.region {
                continue;
            }
            flagged += 1;
            let mut keys: BTreeSet<(String, String, String, String)> = BTreeSet::new();
            for ((file, id, path), _) in &x.changed {
                let class = match (path.is_empty(), module_ids.contains(id)) {
                    (false, _) => "region",
                    (true, true) => "whole_symbol_module",
                    (true, false) => "whole_symbol_item",
                };
                keys.insert((class.into(), file.clone(), id.clone(), path_label(path)));
            }
            for f in &x.fallback_hit {
                keys.insert(("fallback_file".into(), f.clone(), String::new(), String::new()));
            }
            for f in &x.uncited_hit {
                keys.insert(("uncited_file".into(), f.clone(), String::new(), String::new()));
            }
            let sole = keys.len() == 1;
            let classes: BTreeSet<&str> = keys.iter().map(|k| k.0.as_str()).collect();
            for c in &classes {
                let class: &'static str = match *c {
                    "region" => "region",
                    "whole_symbol_module" => "whole_symbol_module",
                    "whole_symbol_item" => "whole_symbol_item",
                    "fallback_file" => "fallback_file",
                    _ => "uncited_file",
                };
                let e = by_class.entry(class).or_default();
                e.0 += 1;
                e.1 += usize::from(classes.len() == 1);
            }
            let files: BTreeSet<&String> = keys.iter().map(|k| &k.1).collect();
            for f in files {
                let e = by_file.entry(f.clone()).or_default();
                e.0 += 1;
                e.1.insert(row.sha.clone());
            }
            for k in keys {
                let d = by.entry(k).or_default();
                d.pairs += 1;
                d.sole += usize::from(sole);
                d.commits.insert(row.sha.clone());
                d.topics.insert(all[*ti].id.clone());
            }
        }
    }
    let mut ranked: Vec<_> = by.into_iter().collect();
    ranked.sort_by(|a, b| b.1.pairs.cmp(&a.1.pairs).then_with(|| a.0.cmp(&b.0)));
    let top: Vec<Value> = ranked
        .iter()
        .take(TOP_DRIVERS)
        .map(|((class, file, id, path), d)| {
            json!({"class": class, "file": file, "symbol": id, "region": path, "flagged_pairs": d.pairs,
                   "sole_driver_pairs": d.sole, "commits": d.commits.len(), "topics": d.topics.len()})
        })
        .collect();
    let mut files: Vec<_> = by_file.into_iter().collect();
    files.sort_by(|a, b| b.1.0.cmp(&a.1.0).then_with(|| a.0.cmp(&b.0)));
    json!({
        "t_region_flags": flagged,
        "by_class": by_class.iter().map(|(k, (n, only))| (k.to_string(), json!({"flagged_pairs": n, "only_class_pairs": only})))
            .collect::<serde_json::Map<_, _>>(),
        "top_drivers": top,
        "top_files": files.iter().take(TOP_DRIVERS)
            .map(|(f, (n, c))| json!({"file": f, "flagged_pairs": n, "commits": c.len()})).collect::<Vec<_>>(),
    })
}

fn repo_json(all: &[Topic], r: &RepoRun) -> Value {
    let (f, s, g) = series(r, false);
    let (fr, sr, gr) = series(r, true);
    let id = |ti: usize| all[ti].id.clone();
    let mut commits = Vec::new();
    let mut region_not_sym = Vec::new();
    for row in &r.rows {
        let sym: BTreeMap<usize, bool> = row.flags.iter().map(|(t, x)| (*t, x.sym)).collect();
        let mut why = serde_json::Map::new();
        for (ti, x) in &row.rflags {
            if x.region && !sym[ti] {
                region_not_sym.push(json!({"commit": row.sha, "topic": id(*ti)}));
            }
            if !x.region {
                continue;
            }
            why.insert(
                id(*ti),
                json!({
                    "regions": x.changed.iter().map(|((file, sid, p), c)| json!([file, sid, path_label(p), change_label(*c)])).collect::<Vec<_>>(),
                    "fallback_files": x.fallback_hit, "uncited_files": x.uncited_hit,
                }),
            );
        }
        let (tf, ts, tr) = count_flags(&row.flags, &row.rflags);
        let (_, _, trr) = count_flags(&row.flags_rust, &row.rflags_rust);
        commits.push(json!({
            "sha": row.sha, "parent": row.parent, "date": row.date, "subject": row.subject,
            "t_file": tf, "t_sym": ts, "t_region": tr, "t_region_rust_only": trr, "why": why,
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
        "endpoints": block(&f, &s, &g),
        "rust_only": block(&fr, &sr, &gr),
        "t_region_reasons": reasons(r),
        "regions": regions_block(r),
        "drivers": drivers(all, r),
        "checks": {
            // Narrower tracking can only drop flags: a region flag without a symbol flag would be a bug.
            "t_region_without_t_sym": region_not_sym,
            "t_region_without_t_file": r.rows.iter().flat_map(|x| &x.rflags).filter(|(_, x)| x.region && !x.file).count(),
            "symbol_node_not_found_at_p_or_c": r.node_misses,
            "region_file_unparsable_at_p_or_c": r.unparsed,
        },
        "commits": commits,
    })
}

fn endpoint4(sample: &Sample) -> Value {
    let pop: serde_json::Map<String, Value> = sample.population.iter().map(|(k, n)| (k.clone(), json!(n))).collect();
    let before: serde_json::Map<String, Value> =
        sample.before_stamp_filter.iter().map(|(k, n)| (k.clone(), json!(n))).collect();
    let eligible: usize = sample.population.iter().map(|(_, n)| n).sum();
    let count = |k: &str| sample.pairs.iter().filter(|p| p.repo == k).count();
    let underpowered = eligible < MIN_ELIGIBLE_B;
    json!({
        "status": if underpowered { "UNDERPOWERED" } else { "PENDING" },
        "reason": if underpowered {
            "fewer than 20 eligible pairs (PREREG endpoint 4): reported as underpowered, not passed"
        } else {
            "no subagent tool in the harness context; prompts prepared, no verdict produced"
        },
        "question": crate::judge::QUESTION,
        "seed": stats::SEED,
        "flagged_by_t_file_not_t_region": before,
        "eligible_after_stamp": pop,
        "eligible_total": eligible,
        "minimum_eligible": MIN_ELIGIBLE_B,
        "drawn": {"visionclaw": count("visionclaw"), "agentbox": count("agentbox"), "total": sample.pairs.len()},
        "sample": sample.pairs,
    })
}

/// The whole E0b results document.
pub fn results_json(all: &[Topic], runs: &[RepoRun], sample: &Sample) -> Value {
    let mut repos = serde_json::Map::new();
    let mut pooled: Series = Default::default();
    let mut pooled_rs: Series = Default::default();
    for r in runs {
        repos.insert(r.key.as_str().into(), repo_json(all, r));
        for (dst, src) in [(&mut pooled, series(r, false)), (&mut pooled_rs, series(r, true))] {
            dst.0.extend(src.0);
            dst.1.extend(src.1);
            dst.2.extend(src.2);
        }
    }
    let vc = &runs[0];
    let (vf, _, vg) = series(vc, false);
    let (vfr, _, vgr) = series(vc, true);
    let r1 = stats::ratio(&vf, &vg);
    let r2 = stats::ratio(&vfr, &vgr);
    let e1 = r1.is_some_and(|x| x >= R_THRESHOLD);
    let e2 = r2.is_some_and(|x| x >= R_THRESHOLD);
    let pf = |b: bool| if b { "PASS" } else { "FAIL" };
    let e4 = endpoint4(sample);
    let underpowered = e4["status"] == "UNDERPOWERED";
    let overall = match (e1, e2, underpowered) {
        (false, false, _) => "DOES NOT HOLD (endpoints 1 and 2 both failed)",
        (_, _, true) => "DOES NOT HOLD (endpoint 4 underpowered)",
        (true, _, false) => "UNDETERMINED until endpoint 4 is judged (endpoint 1 passed)",
        (false, true, false) => "UNDETERMINED until endpoint 4 is judged (endpoint 2 passed, endpoint 1 failed)",
    };
    json!({
        "experiment": "E0b",
        "prereg": "docs/evidence/E0b/PREREG.md",
        "pins": {"corpus": crate::CORPUS_SHA, "visionclaw": crate::VISIONCLAW_SHA, "agentbox": crate::AGENTBOX_SHA,
                 "sealmap": "0.2.0 crates in-tree, sealmap_rust::extract_dir with RustOptions::default(); region hashes through sealmap-rust's fingerprint.rs compiled in"},
        "corpus": {
            "topics": all.len(),
            "citations": all.iter().map(|t| t.citations.len()).sum::<usize>(),
        },
        "region_kinds": Kind::ALL.iter().map(|k| k.label()).collect::<Vec<_>>(),
        "repos": repos,
        "pooled": block(&pooled.0, &pooled.1, &pooled.2),
        "pooled_rust_only": block(&pooled_rs.0, &pooled_rs.1, &pooled_rs.2),
        "endpoint4": e4,
        "verdict": {
            "endpoint1": {"r_region": opt(r1), "threshold": R_THRESHOLD, "result": pf(e1)},
            "endpoint2": {"r_region_rust_only": opt(r2), "threshold": R_THRESHOLD, "result": pf(e2)},
            "endpoint4": {"threshold": HIDDEN_CEILING, "result": e4["status"].clone()},
            "e0b": overall,
        },
    })
}

fn g<'a>(v: &'a Value, path: &[&str]) -> &'a Value {
    path.iter().fold(v, |acc, k| &acc[*k])
}

/// A short console summary.
pub fn summary(v: &Value) -> String {
    let mut s = String::new();
    for (k, b) in [("visionclaw", "endpoints"), ("visionclaw", "rust_only"), ("agentbox", "endpoints")] {
        let e = g(v, &["repos", k, b]);
        s.push_str(&format!(
            "{k} {b}: R_region={} CI95=[{}, {}] R_sym={} median file/sym/region={}/{}/{}\n",
            num(&e["r_region"]),
            num(&e["bootstrap_r_region"]["ci95"][0]),
            num(&e["bootstrap_r_region"]["ci95"][1]),
            num(&e["r_sym"]),
            num(&e["median"]["t_file"]),
            num(&e["median"]["t_sym"]),
            num(&e["median"]["t_region"]),
        ));
    }
    s.push_str(&format!("endpoint 4: {}\n", v["endpoint4"]["status"].as_str().unwrap_or("")));
    s.push_str(&format!("verdict: {}\n", v["verdict"]["e0b"].as_str().unwrap_or("")));
    s
}

fn row(o: &mut String, label: &str, e: &Value) {
    o.push_str(&format!(
        "| {label} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {}–{} | {} | {}–{} |\n",
        num(&e["commits"]),
        num(&e["sum"]["t_file"]),
        num(&e["sum"]["t_sym"]),
        num(&e["sum"]["t_region"]),
        num(&e["median"]["t_file"]),
        num(&e["median"]["t_sym"]),
        num(&e["median"]["t_region"]),
        num(&e["p90"]["t_file"]),
        num(&e["p90"]["t_sym"]),
        num(&e["p90"]["t_region"]),
        num(&e["r_region"]),
        num(&e["bootstrap_r_region"]["ci95"][0]),
        num(&e["bootstrap_r_region"]["ci95"][1]),
        num(&e["r_sym"]),
        num(&e["bootstrap_r_sym"]["ci95"][0]),
        num(&e["bootstrap_r_sym"]["ci95"][1]),
    ));
}

/// `RESULTS.md`, with the PREREG's amendments copied in verbatim.
pub fn results_md(v: &Value, prereg: &str) -> String {
    let mut o = String::new();
    o.push_str("<!-- GENERATED by `cargo run -p sealmap-bench-e0 -- run-b` — do not edit by hand -->\n");
    o.push_str("# E0b results: region-level anchoring inside large symbols\n\n");
    o.push_str("Pre-registration: [PREREG.md](PREREG.md). Machine-readable: [results.json](results.json). ");
    o.push_str("Wall-clock: [TIMING.md](TIMING.md) (not part of the reproducible output). ");
    o.push_str("Inputs, window, citation reading and T_file are E0's ([../E0/RESULTS.md](../E0/RESULTS.md)).\n\n");
    o.push_str("| Input | Revision |\n|---|---|\n");
    for k in ["corpus", "visionclaw", "agentbox", "sealmap"] {
        o.push_str(&format!("| {k} | `{}` |\n", v["pins"][k].as_str().unwrap_or("")));
    }
    let ver = &v["verdict"];
    let vc = g(v, &["repos", "visionclaw"]);
    o.push_str("\n## Verdict\n\n");
    o.push_str(&format!(
        "- **Endpoint 1 (primary):** R_region = ΣT_file / ΣT_region over the {} eligible VisionClaw commits = **{}** (bootstrap 95% CI {}–{}); threshold ≥ {}: **{}**. (R_sym on the same commits: {}.)\n",
        num(&vc["endpoints"]["commits"]),
        num(&ver["endpoint1"]["r_region"]),
        num(&vc["endpoints"]["bootstrap_r_region"]["ci95"][0]),
        num(&vc["endpoints"]["bootstrap_r_region"]["ci95"][1]),
        num(&ver["endpoint1"]["threshold"]),
        ver["endpoint1"]["result"].as_str().unwrap_or(""),
        num(&vc["endpoints"]["r_sym"]),
    ));
    o.push_str(&format!(
        "- **Endpoint 2 (co-primary):** R_region with each topic restricted to its `.rs` sources and citations = **{}** (bootstrap 95% CI {}–{}); threshold ≥ {}: **{}**. (R_sym restricted the same way: {}.)\n",
        num(&ver["endpoint2"]["r_region_rust_only"]),
        num(&vc["rust_only"]["bootstrap_r_region"]["ci95"][0]),
        num(&vc["rust_only"]["bootstrap_r_region"]["ci95"][1]),
        num(&ver["endpoint2"]["threshold"]),
        ver["endpoint2"]["result"].as_str().unwrap_or(""),
        num(&vc["rust_only"]["r_sym"]),
    ));
    let e4 = &v["endpoint4"];
    o.push_str(&format!(
        "- **Endpoint 4 (hidden-change precision):** **{}**. {} eligible pairs (flagged by T_file but not T_region, on commits after the topic's stamp): {} VisionClaw, {} agentbox (of {} and {} before the after-stamp filter). {} drawn ({} VisionClaw, {} agentbox); prompts are in [`judge/`](judge/). No verdict has been produced in the context that drew the sample.\n",
        e4["status"].as_str().unwrap_or(""),
        num(&e4["eligible_total"]),
        num(&e4["eligible_after_stamp"]["visionclaw"]),
        num(&e4["eligible_after_stamp"]["agentbox"]),
        num(&e4["flagged_by_t_file_not_t_region"]["visionclaw"]),
        num(&e4["flagged_by_t_file_not_t_region"]["agentbox"]),
        num(&e4["drawn"]["total"]),
        num(&e4["drawn"]["visionclaw"]),
        num(&e4["drawn"]["agentbox"]),
    ));
    o.push_str(&format!("- **E0b:** {}.\n", ver["e0b"].as_str().unwrap_or("")));

    o.push_str("\n## Endpoints 1–3: topics flagged per commit\n\n");
    o.push_str("| | commits | ΣT_file | ΣT_sym | ΣT_region | median T_file | median T_sym | median T_region | p90 T_file | p90 T_sym | p90 T_region | R_region | 95% CI | R_sym | 95% CI |\n");
    o.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    row(&mut o, "VisionClaw (endpoint 1)", &vc["endpoints"]);
    row(&mut o, "agentbox", &g(v, &["repos", "agentbox"])["endpoints"]);
    row(&mut o, "pooled", &v["pooled"]);
    row(&mut o, "VisionClaw, `.rs` only (endpoint 2)", &vc["rust_only"]);
    row(&mut o, "agentbox, `.rs` only", &g(v, &["repos", "agentbox"])["rust_only"]);
    row(&mut o, "pooled, `.rs` only", &v["pooled_rust_only"]);
    o.push_str("\nMedians and p90s as E0 (amendment #7). Bootstrap: 10,000 commit resamples, ChaCha8 seed 20261006, percentile. T_file and T_sym are E0's counts on the same commits. Only VisionClaw's two ratios carry a threshold.\n");

    o.push_str("\n### Why T_region flags a topic\n\nA flag can have more than one reason; *only* counts flags with exactly one.\n\n");
    o.push_str("| | T_region flags | cited region changed | whole-symbol region changed | fallback-citation file changed | uncited source changed | only region | only whole symbol | only fallback | only uncited | T_file but not T_region |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
    for k in ["visionclaw", "agentbox"] {
        let x = g(v, &["repos", k, "t_region_reasons"]);
        o.push_str(&format!(
            "| {k} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            num(&x["t_region_flags"]),
            num(&x["with_reason"]["cited_region_changed"]),
            num(&x["with_reason"]["whole_symbol_region_changed"]),
            num(&x["with_reason"]["fallback_citation_file_changed"]),
            num(&x["with_reason"]["uncited_source_changed"]),
            num(&x["only_reason"]["cited_region"]),
            num(&x["only_reason"]["whole_symbol_region"]),
            num(&x["only_reason"]["fallback_citation_file"]),
            num(&x["only_reason"]["uncited_source"]),
            num(&x["t_file_not_t_region"]),
        ));
    }
    o.push_str("\nRegion change events (a changed cited region, per commit and topic):\n\n");
    for k in ["visionclaw", "agentbox"] {
        let ev = g(v, &["repos", k, "t_region_reasons", "region_change_events"]);
        let parts: Vec<String> = ev.as_object().into_iter().flatten().map(|(n, c)| format!("{n} {}", num(c))).collect();
        o.push_str(&format!("- {k}: {}\n", if parts.is_empty() { "none".into() } else { parts.join(", ") }));
    }

    o.push_str("\n## Endpoint 3: how far citations narrow\n\n");
    o.push_str("| | `.rs` citations | mapped to a symbol | of which module symbol | region = whole symbol | (item symbol) | (module symbol) | whole-symbol share of mapped | symbol node not found at stamp | distinct regions |\n|---|---|---|---|---|---|---|---|---|---|\n");
    for k in ["visionclaw", "agentbox"] {
        let x = g(v, &["repos", k, "regions"]);
        o.push_str(&format!(
            "| {k} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            num(&x["rust_citations"]),
            num(&x["rust_citations_mapped_to_a_symbol"]),
            num(&x["of_which_module_symbol"]),
            num(&x["region_is_whole_symbol"]),
            num(&x["region_is_whole_symbol_item"]),
            num(&x["region_is_whole_symbol_module"]),
            num(&x["whole_symbol_share_of_mapped"]),
            num(&x["symbol_node_not_found_at_stamp"]),
            num(&x["distinct_regions"]),
        ));
    }
    o.push_str("\nInnermost region kind of the citations that narrowed:\n\n| | ");
    let kinds: Vec<&str> = v["region_kinds"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    o.push_str(&kinds.join(" | "));
    o.push_str(" |\n|---|");
    o.push_str(&"---|".repeat(kinds.len()));
    o.push('\n');
    for k in ["visionclaw", "agentbox"] {
        let x = g(v, &["repos", k, "regions", "innermost_region_kind"]);
        o.push_str(&format!("| {k} | "));
        o.push_str(&kinds.iter().map(|kk| num(&x[*kk])).collect::<Vec<_>>().join(" | "));
        o.push_str(" |\n");
    }

    o.push_str("\n## What still flags\n\nEvery changed cited region, whole symbol or whole file in a T_region-flagged (commit, topic) pair, ranked by the number of such pairs; *sole* counts pairs it flagged alone. Classes: `region` (narrowed region), `whole_symbol_item` / `whole_symbol_module` (no narrower node), `fallback_file` (a non-Rust or otherwise fallback citation's file), `uncited_file` (a source no citation names).\n");
    for k in ["visionclaw", "agentbox"] {
        let d = g(v, &["repos", k, "drivers"]);
        o.push_str(&format!("\n### {k}\n\nBy class (pairs involving the class; pairs flagged by that class only):"));
        for (c, x) in d["by_class"].as_object().into_iter().flatten() {
            o.push_str(&format!(" {c} {} ({});", num(&x["flagged_pairs"]), num(&x["only_class_pairs"])));
        }
        o.push_str("\n\n| class | file | symbol | region | pairs | sole | commits | topics |\n|---|---|---|---|---|---|---|---|\n");
        for x in d["top_drivers"].as_array().into_iter().flatten() {
            o.push_str(&format!(
                "| {} | `{}` | {} | {} | {} | {} | {} | {} |\n",
                x["class"].as_str().unwrap_or(""),
                x["file"].as_str().unwrap_or(""),
                x["symbol"].as_str().filter(|s| !s.is_empty()).map_or(String::new(), |s| format!("`{s}`")),
                x["region"].as_str().unwrap_or(""),
                num(&x["flagged_pairs"]),
                num(&x["sole_driver_pairs"]),
                num(&x["commits"]),
                num(&x["topics"]),
            ));
        }
        o.push_str("\nFiles (any driver in the file):\n\n| file | pairs | commits |\n|---|---|---|\n");
        for x in d["top_files"].as_array().into_iter().flatten() {
            o.push_str(&format!(
                "| `{}` | {} | {} |\n",
                x["file"].as_str().unwrap_or(""),
                num(&x["flagged_pairs"]),
                num(&x["commits"])
            ));
        }
    }

    o.push_str("\n## Checks\n\n");
    for k in ["visionclaw", "agentbox"] {
        let c = g(v, &["repos", k, "checks"]);
        o.push_str(&format!(
            "- {k}: T_region without T_sym: {}; T_region without T_file: {}; symbol node not found at P or C: {}; region file unparsable at P or C: {}.\n",
            c["t_region_without_t_sym"].as_array().map_or(0, Vec::len),
            num(&c["t_region_without_t_file"]),
            num(&c["symbol_node_not_found_at_p_or_c"]),
            num(&c["region_file_unparsable_at_p_or_c"]),
        ));
    }

    o.push_str("\n## Amendments (copied from PREREG.md)\n\n");
    let amend = prereg.split_once("## Amendments").map_or("", |(_, rest)| rest.trim());
    o.push_str(amend);
    o.push('\n');
    o
}
