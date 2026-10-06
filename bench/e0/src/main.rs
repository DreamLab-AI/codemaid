//! # E0 harness
//!
//! Replays pinned first-parent history of VisionClaw and agentbox and counts,
//! per commit, the diagram topics that file-level staleness (`sources:`) flags
//! against those that symbol-level staleness (sealmap fingerprints) flags, with
//! and without one-hop callees. Everything it does is fixed by
//! `docs/evidence/E0/PREREG.md`; the pins below are copied from it.
//!
//! ```text
//! e0 run --corpus <VisionFlow> --visionclaw <project> --agentbox <project/agentbox> \
//!        --out docs/evidence/E0 --scratch <dir>
//! e0 score --out docs/evidence/E0      # endpoint 4, once judge verdicts exist
//! ```
//!
//! All inputs are read through `git` at the pinned revisions; no working tree,
//! index or ref of an input repository is touched.

mod counting;
mod git;
mod judge;
mod mapping;
mod model;
mod report;
mod stats;
mod topics;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

use counting::{Flags, SymChange, flag};
use git::Repo;
use mapping::{Fallback, Mapped, RepoCitation, Stamp, Tracking, map_citation, tracking};
use model::Models;
use topics::{RepoKey, Resolved, Topic};

/// Corpus revision (VisionFlow).
pub(crate) const CORPUS_SHA: &str = "cace76a09db2c98732a41cd8d798b9ef0a1f74e2";
/// VisionClaw head.
pub(crate) const VISIONCLAW_SHA: &str = "9c8dcdc92d7ac50644f336bbd4e156f161219be0";
/// agentbox head.
pub(crate) const AGENTBOX_SHA: &str = "b487c95fa9caf7096a3284f91138935a7f3c3531";
/// Eligible commits per repository.
pub(crate) const WINDOW: usize = 100;
/// Corpus areas read.
pub(crate) const AREAS: [&str; 2] = ["visionclaw", "agentbox"];

/// One topic as tracked in one repository.
pub(crate) struct TopicInRepo {
    pub topic: usize,
    pub tracking: Tracking,
    /// The same topic tracked from its mermaid citations only (sensitivity, amendment #9).
    pub tracking_mermaid: Tracking,
    /// The topic's `.rs` sources and citations only (exploratory, amendment #10).
    pub tracking_rust: Tracking,
    pub stamp: Option<String>,
    pub cites: Vec<(topics::Citation, String, Mapped)>,
}

/// A commit's per-topic outcome.
pub(crate) struct CommitRow {
    pub sha: String,
    pub parent: String,
    pub subject: String,
    pub date: String,
    pub flags: Vec<(usize, Flags)>,
    pub flags_mermaid: Vec<(usize, Flags)>,
    pub flags_rust: Vec<(usize, Flags)>,
}

/// Everything computed for one repository.
pub(crate) struct RepoRun {
    pub key: RepoKey,
    pub head: String,
    pub tracked: Vec<TopicInRepo>,
    pub other_repo_citations: usize,
    pub rows: Vec<CommitRow>,
    pub ineligible: usize,
    pub merges: usize,
    pub roots: usize,
    pub walked: usize,
    pub extractions: usize,
    pub lookups: usize,
    /// (commit, topic) → the commit precedes (is an ancestor of) the topic's stamp.
    pub precedes_stamp: BTreeMap<(String, usize), bool>,
}

struct Args {
    corpus: PathBuf,
    visionclaw: PathBuf,
    agentbox: PathBuf,
    out: PathBuf,
    scratch: PathBuf,
}

fn arg(list: &[String], name: &str) -> Result<PathBuf, String> {
    list.iter()
        .position(|a| a == name)
        .and_then(|i| list.get(i + 1))
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing {name} <path>"))
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let result = match argv.first().map(String::as_str) {
        Some("run") => (|| {
            let a = Args {
                corpus: arg(&argv, "--corpus")?,
                visionclaw: arg(&argv, "--visionclaw")?,
                agentbox: arg(&argv, "--agentbox")?,
                out: arg(&argv, "--out")?,
                scratch: arg(&argv, "--scratch")?,
            };
            run(&a)
        })(),
        Some("score") => arg(&argv, "--out").and_then(|out| judge::score(&out)),
        _ => Err("usage: e0 run --corpus D --visionclaw D --agentbox D --out D --scratch D | e0 score --out D".into()),
    };
    if let Err(e) = result {
        eprintln!("e0: {e}");
        std::process::exit(1);
    }
}

/// Read every topic in the two areas at the corpus pin, in path order.
fn load_topics(corpus: &Repo) -> Result<Vec<Topic>, String> {
    let mut out = Vec::new();
    for e in corpus.ls_tree(CORPUS_SHA)? {
        let Some(rest) = e.path.strip_prefix("docs/diagrams/") else { continue };
        let mut parts = rest.split('/');
        let (Some(area), Some(_)) = (parts.next(), parts.next()) else { continue };
        if !AREAS.contains(&area) || !e.path.ends_with(".md") {
            continue;
        }
        let skip = ["hero", "archive", "rendered", "src", "upgraded", "regen-2026-06-14", "triptych-src"];
        if rest.split('/').any(|seg| skip.contains(&seg) || seg.starts_with('.')) {
            continue;
        }
        let bytes = corpus.show(CORPUS_SHA, &e.path).ok_or_else(|| format!("cannot read {}", e.path))?;
        let text = String::from_utf8(bytes).map_err(|_| format!("{}: not UTF-8", e.path))?;
        out.push(topics::parse_topic(&e.path, &text)?);
    }
    Ok(out)
}

fn run(a: &Args) -> Result<(), String> {
    let t0 = Instant::now();
    let corpus = Repo::open(&a.corpus);
    if corpus.resolve_commit(CORPUS_SHA).as_deref() != Some(CORPUS_SHA) {
        return Err(format!("corpus pin {CORPUS_SHA} not found in {}", a.corpus.display()));
    }
    let all = load_topics(&corpus)?;
    let mut timing = vec![("load topics".to_string(), t0.elapsed())];
    let mut runs = Vec::new();
    for key in RepoKey::ALL {
        let (dir, head) = match key {
            RepoKey::Visionclaw => (&a.visionclaw, VISIONCLAW_SHA),
            RepoKey::Agentbox => (&a.agentbox, AGENTBOX_SHA),
        };
        let t = Instant::now();
        let r = run_repo(key, &Repo::open(dir), head, &all, &a.scratch.join(key.as_str()))?;
        timing.push((format!("{} ({} extractions, {} lookups)", key.as_str(), r.extractions, r.lookups), t.elapsed()));
        runs.push(r);
    }
    let t = Instant::now();
    let repos: Vec<(RepoKey, Repo)> =
        vec![(RepoKey::Visionclaw, Repo::open(&a.visionclaw)), (RepoKey::Agentbox, Repo::open(&a.agentbox))];
    let sample = judge::prepare(&a.out, &all, &runs, &repos)?;
    let prereg = std::fs::read_to_string(a.out.join("PREREG.md")).map_err(|e| format!("PREREG.md: {e}"))?;
    let results = report::results_json(&all, &runs, &sample);
    let text = serde_json::to_string_pretty(&results).map_err(|e| e.to_string())? + "\n";
    std::fs::write(a.out.join("results.json"), text).map_err(|e| format!("results.json: {e}"))?;
    std::fs::write(a.out.join("RESULTS.md"), report::results_md(&results, &prereg))
        .map_err(|e| format!("RESULTS.md: {e}"))?;
    timing.push(("sample, prompts, report".into(), t.elapsed()));
    timing.push(("total".into(), t0.elapsed()));
    let mut tm = String::from(
        "# E0 timing\n\nWall-clock times of the last `e0 run`. Not part of the byte-identical output (PREREG amendment 2026-10-06 #2).\n\n| Step | Seconds |\n|---|---|\n",
    );
    for (k, d) in &timing {
        tm.push_str(&format!("| {k} | {:.1} |\n", d.as_secs_f64()));
    }
    std::fs::write(a.out.join("TIMING.md"), tm).map_err(|e| format!("TIMING.md: {e}"))?;
    eprint!("{}", report::summary(&results));
    Ok(())
}

fn run_repo(key: RepoKey, repo: &Repo, head: &str, all: &[Topic], scratch: &Path) -> Result<RepoRun, String> {
    if repo.resolve_commit(head).as_deref() != Some(head) {
        return Err(format!("{} pin {head} not found", key.as_str()));
    }
    let mut models = Models::new(repo.clone(), key.as_str(), scratch);
    let mut tracked = Vec::new();
    let mut other_repo_citations = 0;
    let mut stamp_cache: HashMap<String, Option<String>> = HashMap::new();
    for (ti, t) in all.iter().enumerate() {
        let sources: BTreeSet<String> = t
            .sources
            .iter()
            .filter_map(|s| match topics::repo_of(s) {
                (Some(k), rel) if k == key.as_str() => Some(rel),
                _ => None,
            })
            .collect();
        if sources.is_empty() {
            continue;
        }
        let stamp_raw = topics::sha_for(&t.verified_commit, &t.area, key.as_str());
        let stamp = match &stamp_raw {
            None => None,
            Some(s) => stamp_cache.entry(s.clone()).or_insert_with(|| repo.resolve_commit(s)).clone(),
        };
        let stamp_model = match &stamp {
            Some(s) => Some(models.at(s)?),
            None => None,
        };
        let mut cites = Vec::new();
        let mut rcs = Vec::new();
        for c in &t.citations {
            let src = match &c.resolved {
                Resolved::Source(s) => s,
                // Counted from the topics in the report; excluded from both methods.
                Resolved::NotInSources | Resolved::Ambiguous => continue,
            };
            let rel = match topics::repo_of(src) {
                (Some(k), rel) if k == key.as_str() => rel,
                _ => {
                    if t.area == key.as_str() {
                        other_repo_citations += 1;
                    }
                    continue;
                }
            };
            let st = match (&stamp_raw, &stamp, &stamp_model) {
                (None, _, _) => Stamp::Missing,
                (Some(_), None, _) => Stamp::Unknown,
                (Some(_), Some(s), Some(m)) => Stamp::Model { model: m, file_exists: repo.exists(s, &rel) },
                (Some(_), Some(_), None) => Stamp::Unknown,
            };
            let mapped = map_citation(&rel, c.line, &st);
            rcs.push((c.prose, RepoCitation { file: rel.clone(), mapped: mapped.clone() }));
            cites.push((c.clone(), rel, mapped));
        }
        let all_rc: Vec<RepoCitation> = rcs.iter().map(|(_, r)| r.clone()).collect();
        let mermaid_rc: Vec<RepoCitation> = rcs.iter().filter(|(p, _)| !p).map(|(_, r)| r.clone()).collect();
        let rust_rc: Vec<RepoCitation> = all_rc.iter().filter(|r| r.file.ends_with(".rs")).cloned().collect();
        let rust_sources: BTreeSet<String> = sources.iter().filter(|s| s.ends_with(".rs")).cloned().collect();
        tracked.push(TopicInRepo {
            topic: ti,
            tracking: tracking(&sources, &all_rc),
            tracking_mermaid: tracking(&sources, &mermaid_rc),
            tracking_rust: tracking(&rust_sources, &rust_rc),
            stamp,
            cites,
        });
    }
    let cited: BTreeSet<&String> = tracked.iter().flat_map(|t| t.tracking.sources.iter()).collect();

    let mut rows = Vec::new();
    let (mut ineligible, mut merges, mut roots, mut walked) = (0, 0, 0, 0);
    for h in repo.first_parent_history(head)? {
        if rows.len() == WINDOW {
            break;
        }
        walked += 1;
        if h.parents.len() > 1 {
            merges += 1;
            continue;
        }
        let Some(parent) = h.parents.first() else {
            roots += 1;
            continue;
        };
        let changed: BTreeSet<String> = repo.changed_paths(parent, &h.sha)?.into_iter().collect();
        if !changed.iter().any(|p| cited.contains(p)) {
            ineligible += 1;
            continue;
        }
        let p = models.at(parent)?;
        let c = models.at(&h.sha)?;
        let flags = tracked.iter().map(|t| (t.topic, flag(&t.tracking, &changed, &p, &c))).collect();
        let flags_mermaid = tracked.iter().map(|t| (t.topic, flag(&t.tracking_mermaid, &changed, &p, &c))).collect();
        let flags_rust = tracked.iter().map(|t| (t.topic, flag(&t.tracking_rust, &changed, &p, &c))).collect();
        rows.push(CommitRow {
            sha: h.sha.clone(),
            parent: parent.clone(),
            subject: repo.subject(&h.sha)?,
            date: repo.commit_date(&h.sha)?,
            flags,
            flags_mermaid,
            flags_rust,
        });
    }
    if rows.len() < WINDOW {
        return Err(format!("{}: only {} eligible commits in history", key.as_str(), rows.len()));
    }
    let mut precedes_stamp = BTreeMap::new();
    let mut anc: HashMap<(String, String), bool> = HashMap::new();
    for r in &rows {
        for (ti, f) in &r.flags {
            if !f.file {
                continue;
            }
            let stamp = tracked.iter().find(|t| t.topic == *ti).and_then(|t| t.stamp.clone());
            let v = match stamp {
                Some(s) => *anc.entry((r.sha.clone(), s.clone())).or_insert_with(|| repo.is_ancestor(&r.sha, &s)),
                None => false,
            };
            precedes_stamp.insert((r.sha.clone(), *ti), v);
        }
    }
    Ok(RepoRun {
        key,
        head: head.into(),
        tracked,
        other_repo_citations,
        rows,
        ineligible,
        merges,
        roots,
        walked,
        extractions: models.extractions,
        lookups: models.lookups,
        precedes_stamp,
    })
}

/// `true` if the citation tracks its whole file.
pub(crate) fn is_fallback(m: &Mapped) -> bool {
    matches!(m, Mapped::Fallback(_))
}

/// Label for a fallback reason.
pub(crate) fn fallback_label(f: Fallback) -> &'static str {
    match f {
        Fallback::NotRust => "not_rust",
        Fallback::NoStamp => "no_stamp",
        Fallback::StampUnknown => "stamp_unknown",
        Fallback::AbsentAtStamp => "absent_at_stamp",
        Fallback::ParseError => "parse_error",
        Fallback::NoSymbol => "no_symbol",
    }
}

/// Label for a symbol change.
pub(crate) fn change_label(c: SymChange) -> &'static str {
    match c {
        SymChange::Same => "same",
        SymChange::Hash => "hash",
        SymChange::OneSided => "one_sided",
        SymChange::AbsentBothFileChanged => "absent_both_file_changed",
    }
}
