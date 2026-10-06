//! E0c (`docs/evidence/E0c/PREREG.md`): file-level against call-flow staleness
//! for **sequence citations only**, over E0's commit windows.
//!
//! E0's own `run_repo` supplies the window, the citation reading and the
//! mapping at each topic's stamp, unchanged; this module keeps the citations
//! whose block is a `sequenceDiagram`, replays the window with fresh models,
//! and counts T_file^seq, T_sym^seq (E0's `flag`) and T_flow ([`crate::flow`]).
//!
//! ```text
//! e0 e0c --corpus <VisionFlow> --visionclaw <project> --agentbox <project/agentbox> \
//!        --out docs/evidence/E0c --scratch <dir>
//! e0 e0c-score --out docs/evidence/E0c   # endpoint 4, once judge verdicts exist
//! ```

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::flow::{self, Driver, FlowFlags};
use crate::git::Repo;
use crate::mapping::{Mapped, RepoCitation, Tracking, tracking};
use crate::model::{Model, Models};
use crate::stats::{self, Bootstrap};
use crate::topics::{Citation, RepoKey, Topic};
use crate::{AGENTBOX_SHA, Args, CORPUS_SHA, RepoRun, VISIONCLAW_SHA, counting, fallback_label};

/// The generator's kind for a sequence diagram block.
pub const SEQ: &str = "sequenceDiagram";
/// The pre-registered thresholds.
pub const R_THRESHOLD: f64 = 2.0;
pub const HIDDEN_CEILING: f64 = 0.10;
/// Fewer eligible endpoint-4 pairs than this: underpowered, not passed.
pub const MIN_ELIGIBLE: usize = 20;
/// Pairs drawn per stratum.
pub const STRATA: [(RepoKey, usize); 2] = [(RepoKey::Visionclaw, 30), (RepoKey::Agentbox, 10)];
/// The pre-registered, narrowed judge question.
pub const QUESTION: &str = "Does this change make any sequence diagram in this topic wrong or misleading (a call, branch condition, argument or return a diagram shows)?";
/// Symbols listed per repository in the driver table.
const TOP_DRIVERS: usize = 15;

/// One topic's sequence-citation scope in one repository.
pub struct SeqTopic {
    pub topic: usize,
    pub stamp: Option<String>,
    /// Sequence citations (repository-relative file, mapping).
    pub cites: Vec<(String, Mapped)>,
    /// Tracking over every sequence citation.
    pub all: Tracking,
    /// Tracking over `.rs` sequence citations; `None` when there are none.
    pub rs: Option<Tracking>,
}

/// Build a tracking set from sequence citations only. `None` when there are none.
pub fn seq_tracking(cites: &[(Citation, String, Mapped)], rs_only: bool) -> Option<Tracking> {
    let rc: Vec<RepoCitation> = cites
        .iter()
        .filter(|(c, f, _)| c.kind == SEQ && (!rs_only || f.ends_with(".rs")))
        .map(|(_, f, m)| RepoCitation { file: f.clone(), mapped: m.clone() })
        .collect();
    if rc.is_empty() {
        return None;
    }
    let files: BTreeSet<String> = rc.iter().map(|r| r.file.clone()).collect();
    Some(tracking(&files, &rc))
}

/// `true` when the change P→C wholly postdates `stamp` (the stamp is P or an ancestor of P).
pub fn post_stamp(stamp: Option<&str>, parent: &str, is_ancestor: &mut impl FnMut(&str, &str) -> bool) -> bool {
    stamp.is_some_and(|s| is_ancestor(s, parent))
}

/// One topic's counts for one commit.
#[derive(Debug, Clone, Default)]
pub struct Cell {
    pub file: bool,
    pub sym: bool,
    pub flow: FlowFlags,
}

/// A commit's outcome.
pub struct Row {
    pub sha: String,
    pub parent: String,
    pub cells: Vec<(usize, Cell)>,
    pub cells_rs: Vec<(usize, Cell)>,
}

/// Symbol-change events of one commit: window position, subject, events by class.
type CommitEvent = (usize, String, BTreeMap<Driver, usize>);

/// Per-symbol driver tally.
#[derive(Default)]
struct DriverAgg {
    flags: usize,
    commits: BTreeSet<String>,
    classes: BTreeMap<Driver, usize>,
    example: Option<Value>,
}

/// Everything computed for one repository.
pub struct Run {
    pub key: RepoKey,
    pub topics: Vec<SeqTopic>,
    pub rows: Vec<Row>,
    /// Sequence citations mapped to a symbol: (module, flow with calls, flow without calls) at the stamp.
    mapping_shape: (usize, usize, usize),
    drivers: BTreeMap<String, DriverAgg>,
    driver_classes: BTreeMap<Driver, usize>,
    /// Symbol-change events per commit: (window position, subject, classes).
    commit_events: BTreeMap<String, CommitEvent>,
    /// Pairs flagged by T_file^seq but not T_flow: (sha, parent, topic, post-stamp).
    hidden_pool: Vec<(String, String, usize, bool)>,
    pub extractions: usize,
}

fn cell(t: &Tracking, changed: &BTreeSet<String>, p: &Model, c: &Model) -> Cell {
    let e0 = counting::flag(t, changed, p, c);
    Cell { file: e0.file, sym: e0.sym, flow: flow::flag(t, changed, p, c) }
}

/// Replay one repository's E0 window for its sequence-cited topics.
pub fn run_repo(key: RepoKey, repo: &Repo, e0: &RepoRun, scratch: &Path) -> Result<Run, String> {
    let mut models = Models::new(repo.clone(), key.as_str(), scratch);
    let mut topics_in = Vec::new();
    let mut shape = (0, 0, 0);
    for t in &e0.tracked {
        let Some(all) = seq_tracking(&t.cites, false) else { continue };
        let cites: Vec<(String, Mapped)> =
            t.cites.iter().filter(|(c, _, _)| c.kind == SEQ).map(|(_, f, m)| (f.clone(), m.clone())).collect();
        if let Some(s) = &t.stamp {
            let m = models.at(s)?;
            for (_, mp) in &cites {
                if let Mapped::Symbol { id, module } = mp {
                    if *module {
                        shape.0 += 1;
                    } else if m.flows.get(id).is_some_and(|f| !f.calls.is_empty()) {
                        shape.1 += 1;
                    } else {
                        shape.2 += 1;
                    }
                }
            }
        }
        topics_in.push(SeqTopic {
            topic: t.topic,
            stamp: t.stamp.clone(),
            cites,
            all,
            rs: seq_tracking(&t.cites, true),
        });
    }
    let mut rows = Vec::new();
    let mut drivers: BTreeMap<String, DriverAgg> = BTreeMap::new();
    let mut driver_classes: BTreeMap<Driver, usize> = BTreeMap::new();
    let mut commit_events: BTreeMap<String, CommitEvent> = BTreeMap::new();
    for (pos, r) in e0.rows.iter().enumerate() {
        let changed: BTreeSet<String> = repo.changed_paths(&r.parent, &r.sha)?.into_iter().collect();
        let p = models.at(&r.parent)?;
        let c = models.at(&r.sha)?;
        let mut cells = Vec::new();
        let mut cells_rs = Vec::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for st in &topics_in {
            let x = cell(&st.all, &changed, &p, &c);
            for (id, _) in &x.flow.changed {
                let d = flow::driver(id, &p, &c).ok_or("driver of an unchanged flow")?;
                let agg = drivers.entry(id.clone()).or_default();
                agg.flags += 1;
                agg.commits.insert(r.sha.clone());
                if seen.insert(id.clone()) {
                    *agg.classes.entry(d).or_default() += 1;
                    *driver_classes.entry(d).or_default() += 1;
                    let ev =
                        commit_events.entry(r.sha.clone()).or_insert_with(|| (pos, r.subject.clone(), BTreeMap::new()));
                    *ev.2.entry(d).or_default() += 1;
                }
                if agg.example.is_none() {
                    let none = Vec::new();
                    let a = p.flows.get(id).map_or(&none, |f| &f.calls);
                    let b = c.flows.get(id).map_or(&none, |f| &f.calls);
                    let (added, removed) = flow::call_diff(a, b);
                    agg.example = Some(json!({
                        "commit": r.sha, "driver": d,
                        "calls_at_p": a.len(), "calls_at_c": b.len(),
                        "added": added.iter().take(8).collect::<Vec<_>>(), "added_total": added.len(),
                        "removed": removed.iter().take(8).collect::<Vec<_>>(), "removed_total": removed.len(),
                    }));
                }
            }
            cells.push((st.topic, x));
            if let Some(rs) = &st.rs {
                cells_rs.push((st.topic, cell(rs, &changed, &p, &c)));
            }
        }
        rows.push(Row { sha: r.sha.clone(), parent: r.parent.clone(), cells, cells_rs });
    }
    let mut anc: HashMap<(String, String), bool> = HashMap::new();
    let mut is_anc = |a: &str, b: &str| *anc.entry((a.into(), b.into())).or_insert_with(|| repo.is_ancestor(a, b));
    let mut hidden_pool = Vec::new();
    for r in &rows {
        for (ti, x) in &r.cells {
            if x.file && !x.flow.flow {
                let stamp = topics_in.iter().find(|t| t.topic == *ti).and_then(|t| t.stamp.as_deref());
                let post = post_stamp(stamp, &r.parent, &mut is_anc);
                hidden_pool.push((r.sha.clone(), r.parent.clone(), *ti, post));
            }
        }
    }
    Ok(Run {
        key,
        topics: topics_in,
        rows,
        mapping_shape: shape,
        drivers,
        driver_classes,
        commit_events,
        hidden_pool,
        extractions: models.extractions,
    })
}

// ------------------------------------------------------------------ statistics

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
    json!({"resamples": b.resamples, "seed": b.seed, "ci95": [round(b.lo), round(b.hi)], "infinite_resamples": b.infinite})
}

/// Per-commit series: T_file^seq, T_sym^seq, T_flow, T_flow with E0's absent-at-both rule.
#[derive(Default)]
struct Series {
    file: Vec<u32>,
    sym: Vec<u32>,
    flow: Vec<u32>,
    flow_abs: Vec<u32>,
}

impl Series {
    fn of<'a>(rows: impl Iterator<Item = &'a [(usize, Cell)]>) -> Self {
        let mut s = Series::default();
        for cells in rows {
            let n = |f: &dyn Fn(&Cell) -> bool| cells.iter().filter(|(_, c)| f(c)).count() as u32;
            s.file.push(n(&|c| c.file));
            s.sym.push(n(&|c| c.sym));
            s.flow.push(n(&|c| c.flow.flow));
            s.flow_abs.push(n(&|c| c.flow.flow || c.flow.absent_both));
        }
        s
    }

    fn extend(&mut self, o: Series) {
        self.file.extend(o.file);
        self.sym.extend(o.sym);
        self.flow.extend(o.flow);
        self.flow_abs.extend(o.flow_abs);
    }

    fn block(&self) -> Value {
        let sum = |v: &[u32]| v.iter().map(|&x| u64::from(x)).sum::<u64>();
        json!({
            "commits": self.file.len(),
            "sum": {"t_file_seq": sum(&self.file), "t_sym_seq": sum(&self.sym), "t_flow": sum(&self.flow)},
            "median": {"t_file_seq": round(stats::median(&self.file)), "t_sym_seq": round(stats::median(&self.sym)), "t_flow": round(stats::median(&self.flow))},
            "p90": {"t_file_seq": stats::p90(&self.file), "t_sym_seq": stats::p90(&self.sym), "t_flow": stats::p90(&self.flow)},
            "r_flow": opt(stats::ratio(&self.file, &self.flow)),
            "bootstrap_r_flow": boot(&stats::bootstrap(&self.file, &self.flow)),
            "r_sym_seq": opt(stats::ratio(&self.file, &self.sym)),
            "bootstrap_r_sym_seq": boot(&stats::bootstrap(&self.file, &self.sym)),
            "sensitivity_absent_both": {
                "t_flow": sum(&self.flow_abs),
                "r_flow": opt(stats::ratio(&self.file, &self.flow_abs)),
            },
        })
    }
}

fn series(r: &Run) -> Series {
    Series::of(r.rows.iter().map(|x| x.cells.as_slice()))
}

fn series_rs(r: &Run) -> Series {
    Series::of(r.rows.iter().map(|x| x.cells_rs.as_slice()))
}

fn reasons(r: &Run) -> Value {
    let (mut fb_only, mut flow_only, mut both, mut flow_not_sym, mut file_not_flow, mut abs) = (0, 0, 0, 0, 0, 0);
    for row in &r.rows {
        for (_, c) in &row.cells {
            match (c.flow.by_fallback, c.flow.by_flow) {
                (true, false) => fb_only += 1,
                (false, true) => flow_only += 1,
                (true, true) => both += 1,
                _ => {}
            }
            if c.flow.flow && !c.sym {
                flow_not_sym += 1;
            }
            if c.file && !c.flow.flow {
                file_not_flow += 1;
            }
            if !c.flow.flow && c.flow.absent_both {
                abs += 1;
            }
        }
    }
    json!({
        "t_flow_by_fallback_only": fb_only,
        "t_flow_by_flow_only": flow_only,
        "t_flow_by_both": both,
        "t_flow_not_t_sym_seq": flow_not_sym,
        "t_file_seq_not_t_flow": file_not_flow,
        "absent_both_unflagged": abs,
    })
}

fn coverage(all: &[Topic], r: &Run) -> Value {
    let mut fb: BTreeMap<&'static str, usize> = BTreeMap::new();
    let (mut n, mut n_rs, mut fb_n, mut fb_rs) = (0, 0, 0, 0);
    for t in &r.topics {
        for (f, m) in &t.cites {
            n += 1;
            let rs = f.ends_with(".rs");
            n_rs += usize::from(rs);
            if let Mapped::Fallback(why) = m {
                fb_n += 1;
                fb_rs += usize::from(rs);
                *fb.entry(fallback_label(*why)).or_default() += 1;
            }
        }
    }
    let (module, with_calls, no_calls) = r.mapping_shape;
    json!({
        "topics_in_scope": r.topics.len(),
        "topics_in_scope_rs": r.topics.iter().filter(|t| t.rs.is_some()).count(),
        "topics": r.topics.iter().map(|t| all[t.topic].id.clone()).collect::<Vec<_>>(),
        "sequence_citations": n,
        "sequence_citations_rs": n_rs,
        "fallback": fb_n,
        "fallback_share": share(fb_n, n),
        "fallback_rs": fb_rs,
        "fallback_share_rs": share(fb_rs, n_rs),
        "fallback_by_reason": fb,
        "symbol_citations_at_stamp": {"module": module, "flow_with_calls": with_calls, "flow_without_calls": no_calls},
    })
}

fn drivers_json(r: &Run) -> Value {
    let mut top: Vec<(&String, &DriverAgg)> = r.drivers.iter().collect();
    top.sort_by(|a, b| b.1.flags.cmp(&a.1.flags).then(b.1.commits.len().cmp(&a.1.commits.len())).then(a.0.cmp(b.0)));
    let mut commits: Vec<(&String, &CommitEvent)> = r.commit_events.iter().collect();
    let total = |m: &BTreeMap<Driver, usize>| m.values().sum::<usize>();
    commits.sort_by(|a, b| total(&b.1.2).cmp(&total(&a.1.2)).then(a.1.0.cmp(&b.1.0)));
    json!({
        "symbol_change_events_by_class": r.driver_classes,
        "top_commits": commits.iter().take(TOP_DRIVERS).map(|(sha, (_, subject, classes))| json!({
            "commit": sha, "subject": subject, "events": total(classes), "classes": classes,
        })).collect::<Vec<_>>(),
        "symbols_changed": r.drivers.len(),
        "top": top.iter().take(TOP_DRIVERS).map(|(id, a)| json!({
            "symbol": id, "flags": a.flags, "commits": a.commits.len(), "classes": a.classes, "example": a.example,
        })).collect::<Vec<_>>(),
    })
}

fn verdict_of(r: Option<f64>) -> &'static str {
    if r.is_some_and(|x| x >= R_THRESHOLD) { "PASS" } else { "FAIL" }
}

// ------------------------------------------------------------------ endpoint 4

/// One drawn pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pair {
    pub id: String,
    pub repo: String,
    pub commit: String,
    pub parent: String,
    pub topic: String,
    pub diff_files: Vec<String>,
    pub diff_lines: usize,
    pub prompt: String,
}

/// The sample and the populations it came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    /// Per stratum: pairs flagged by T_file^seq and not T_flow, and those after the stamp (eligible).
    pub population: Vec<(String, usize, usize)>,
    pub pairs: Vec<Pair>,
}

impl Sample {
    fn eligible(&self) -> usize {
        self.population.iter().map(|p| p.2).sum()
    }

    fn to_json(&self) -> Value {
        let count = |k: &str| self.pairs.iter().filter(|p| p.repo == k).count();
        let pop: serde_json::Map<String, Value> = self
            .population
            .iter()
            .map(|(k, all, post)| (k.clone(), json!({"t_file_seq_not_t_flow": all, "eligible_post_stamp": post})))
            .collect();
        json!({
            "status": "PENDING",
            "reason": "no subagent tool in the harness context; prompts prepared, no verdict produced",
            "question": QUESTION,
            "seed": stats::SEED,
            "population": pop,
            "eligible_total": self.eligible(),
            "underpowered": self.eligible() < MIN_ELIGIBLE,
            "drawn": {"visionclaw": count("visionclaw"), "agentbox": count("agentbox"), "total": self.pairs.len()},
            "sample": self.pairs,
        })
    }
}

/// The pairs of one stratum eligible for endpoint 4, in window then topic order.
pub fn eligible(pool: &[(String, String, usize, bool)]) -> Vec<(String, String, usize)> {
    pool.iter().filter(|p| p.3).map(|(s, p, t, _)| (s.clone(), p.clone(), *t)).collect()
}

fn fence(body: &str) -> String {
    let (mut longest, mut run) = (0, 0);
    for ch in body.chars() {
        if ch == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    "`".repeat((longest + 1).max(3))
}

fn prompt(id: &str, repo: &str, topic: &Topic, diff: &str) -> String {
    let tf = fence(&topic.text);
    let df = fence(diff);
    format!(
        "# E0c judgement {id}\n\n\
You are an independent judge for a pre-registered experiment. Below are (1) a TOPIC: a Markdown file of Mermaid \
diagrams that document code, with `path:line` citations, where paths under `../project/` are in the VisionClaw \
repository and paths under `../project/agentbox/` are in the agentbox repository; and (2) a DIFF: one commit's \
changes in the {repo} repository, restricted to the files the topic's `sequenceDiagram` blocks cite.\n\n\
Consider only the topic's `sequenceDiagram` blocks. Answer exactly one question:\n\n> **{QUESTION}**\n\n\
Judge only from the two texts below; do not look anything else up. Reply with a single JSON object and nothing else:\n\n\
```json\n{{\"verdict\": \"yes\" | \"no\", \"statements\": [\"<diagram id>: <the message, guard, argument or return, quoted>\", ...], \
\"line_anchors_only\": true | false, \"reason\": \"<one or two sentences>\"}}\n```\n\n\
`statements` lists every element of a sequence diagram the change makes wrong or misleading (empty when the verdict \
is no). `line_anchors_only` is true when the only such elements are `path:line` anchors whose cited code merely moved.\n\n\
## TOPIC ({file})\n\n{tf}markdown\n{text}\n{tf}\n\n## DIFF\n\n{df}diff\n{diff}\n{df}\n",
        file = topic.file,
        text = topic.text.trim_end(),
        diff = diff.trim_end(),
    )
}

const PROTOCOL: &str = "# E0c endpoint 4 judging protocol\n\n\
Generated by `e0 e0c`. Do not edit. Follows E0's `judge/PROTOCOL.md` with the question narrowed by \
`docs/evidence/E0c/PREREG.md`.\n\n\
Pairs are (commit, topic) flagged by T_file^seq but not T_flow, drawn only from commits whose parent is the topic's \
stamp or a descendant of it (amendment #1).\n\n\
1. Each `<id>.prompt.md` is given, unchanged, to a fresh Claude subagent with no other context. Its reply (one JSON \
object) is saved as `<id>.j1.json`.\n\
2. Every pair whose first verdict is `yes` is given, unchanged, to a second fresh subagent, independent of the first \
(it does not see the first verdict). Its reply is saved as `<id>.j2.json`.\n\
3. A pair is a **hidden change** when both judges answer `yes`. A first `yes` the second judge does not confirm counts \
as not hidden.\n\
4. `e0 e0c-score --out docs/evidence/E0c` computes the hidden-change rate per stratum and pooled, with the Wilson 95% \
upper bound, and writes `endpoint4.json` and `ENDPOINT4.md`. **Success:** pooled rate ≤ 10%. Fewer than 20 eligible \
pairs: underpowered, not passed.\n\n\
The judge must not be the context that drew the sample or wrote the harness. `line_anchors_only` is recorded and not \
scored.\n";

fn prepare(out: &Path, all: &[Topic], runs: &[Run], repos: &[(RepoKey, Repo)]) -> Result<Sample, String> {
    let dir = out.join("judge");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for e in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        if e.file_name().to_string_lossy().ends_with(".prompt.md") {
            let _ = std::fs::remove_file(e.path());
        }
    }
    let mut rng = stats::rng();
    let mut population = Vec::new();
    let mut pairs = Vec::new();
    for (key, k) in STRATA {
        let run = runs.iter().find(|r| r.key == key).ok_or("missing run")?;
        let repo = &repos.iter().find(|(r, _)| *r == key).ok_or("missing repo")?.1;
        let pool = eligible(&run.hidden_pool);
        population.push((key.as_str().to_string(), run.hidden_pool.len(), pool.len()));
        let prefix = match key {
            RepoKey::Visionclaw => "vc",
            RepoKey::Agentbox => "ab",
        };
        for (i, (sha, parent, ti)) in stats::draw(&pool, k, &mut rng).into_iter().enumerate() {
            let id = format!("{prefix}-{:02}", i + 1);
            let topic = &all[ti];
            let st = run.topics.iter().find(|t| t.topic == ti).ok_or("drawn topic out of scope")?;
            let files: Vec<String> = st.all.sources.iter().cloned().collect();
            let diff = repo.diff(&parent, &sha, &files)?;
            let file = format!("{id}.prompt.md");
            std::fs::write(dir.join(&file), prompt(&id, key.as_str(), topic, &diff)).map_err(|e| e.to_string())?;
            pairs.push(Pair {
                id,
                repo: key.as_str().into(),
                commit: sha,
                parent,
                topic: topic.id.clone(),
                diff_files: files,
                diff_lines: diff.lines().count(),
                prompt: format!("judge/{file}"),
            });
        }
    }
    let sample = Sample { population, pairs };
    let text = serde_json::to_string_pretty(&sample).map_err(|e| e.to_string())? + "\n";
    std::fs::write(dir.join("sample.json"), text).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("PROTOCOL.md"), PROTOCOL).map_err(|e| e.to_string())?;
    Ok(sample)
}

#[derive(Deserialize)]
struct Verdict {
    verdict: String,
}

fn verdict(path: &Path) -> Result<Option<bool>, String> {
    let Ok(text) = std::fs::read_to_string(path) else { return Ok(None) };
    let v: Verdict = serde_json::from_str(text.trim()).map_err(|e| format!("{}: {e}", path.display()))?;
    match v.verdict.to_ascii_lowercase().as_str() {
        "yes" => Ok(Some(true)),
        "no" => Ok(Some(false)),
        other => Err(format!("{}: verdict `{other}` is neither yes nor no", path.display())),
    }
}

/// Score endpoint 4 from the judges' verdicts.
pub fn score(out: &Path) -> Result<(), String> {
    let dir = out.join("judge");
    let sample: Sample = serde_json::from_str(
        &std::fs::read_to_string(dir.join("sample.json")).map_err(|e| format!("judge/sample.json: {e}"))?,
    )
    .map_err(|e| format!("judge/sample.json: {e}"))?;
    let mut missing = Vec::new();
    let mut rows = Vec::new();
    for p in &sample.pairs {
        let j1 = verdict(&dir.join(format!("{}.j1.json", p.id)))?;
        let j2 = verdict(&dir.join(format!("{}.j2.json", p.id)))?;
        match (j1, j2) {
            (None, _) => missing.push(format!("{}.j1.json", p.id)),
            (Some(true), None) => missing.push(format!("{}.j2.json", p.id)),
            (Some(a), b) => rows.push((p, a, b, a && b == Some(true))),
        }
    }
    if !missing.is_empty() {
        return Err(format!("missing verdicts: {}", missing.join(", ")));
    }
    let block = |repo: Option<&str>| {
        let sel: Vec<_> = rows.iter().filter(|r| repo.is_none_or(|k| r.0.repo == k)).collect();
        let (n, k) = (sel.len(), sel.iter().filter(|r| r.3).count());
        json!({"n": n, "first_judge_yes": sel.iter().filter(|r| r.1).count(), "hidden": k,
               "rate": if n == 0 { Value::Null } else { json!(k as f64 / n as f64) },
               "wilson95_upper": stats::wilson_upper(k, n)})
    };
    let pooled = block(None);
    let rate = pooled["rate"].as_f64().unwrap_or(f64::NAN);
    let result = if sample.eligible() < MIN_ELIGIBLE {
        "UNDERPOWERED (not passed)"
    } else if rate <= HIDDEN_CEILING {
        "PASS"
    } else {
        "FAIL"
    };
    let doc = json!({
        "question": QUESTION,
        "eligible_total": sample.eligible(),
        "visionclaw": block(Some("visionclaw")),
        "agentbox": block(Some("agentbox")),
        "pooled": pooled,
        "threshold": HIDDEN_CEILING,
        "result": result,
        "pairs": rows.iter().map(|(p, a, b, h)| json!({"id": p.id, "j1": a, "j2": b, "hidden": h})).collect::<Vec<_>>(),
    });
    std::fs::write(out.join("endpoint4.json"), serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())? + "\n")
        .map_err(|e| e.to_string())?;
    let table = ["visionclaw", "agentbox", "pooled"]
        .iter()
        .map(|k| {
            let b = &doc[*k];
            format!(
                "| {k} | {} | {} | {} | {:.3} | {:.3} |",
                b["n"],
                b["first_judge_yes"],
                b["hidden"],
                b["rate"].as_f64().unwrap_or(f64::NAN),
                b["wilson95_upper"].as_f64().unwrap_or(f64::NAN)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let md = format!(
        "# E0c endpoint 4: hidden-change precision\n\nGenerated by `e0 e0c-score` from the verdicts in `judge/`.\n\n\
| | n | first-judge yes | hidden (both yes) | rate | Wilson 95% upper |\n|---|---|---|---|---|---|\n{table}\n\n\
**Result:** pooled rate {rate:.3} against a ceiling of 10%, {} eligible pairs: **{result}**.\n",
        sample.eligible()
    );
    std::fs::write(out.join("ENDPOINT4.md"), md).map_err(|e| e.to_string())?;
    eprintln!("E0c endpoint 4: {result} (pooled rate {rate:.3})");
    Ok(())
}

// ------------------------------------------------------------------ report

fn results_json(all: &[Topic], runs: &[Run], sample: &Sample) -> Value {
    let mut repos = serde_json::Map::new();
    let (mut pooled, mut pooled_rs) = (Series::default(), Series::default());
    for r in runs {
        let (s, s_rs) = (series(r), series_rs(r));
        repos.insert(
            r.key.as_str().into(),
            json!({
                "coverage": coverage(all, r),
                "all": s.block(),
                "rs_only": s_rs.block(),
                "reasons": reasons(r),
                "drivers": drivers_json(r),
            }),
        );
        pooled.extend(s);
        pooled_rs.extend(s_rs);
    }
    let vc = &runs[0];
    let (s1, s2) = (series(vc), series_rs(vc));
    let (r1, r2) = (stats::ratio(&s1.file, &s1.flow), stats::ratio(&s2.file, &s2.flow));
    let (b1, b2) = (stats::bootstrap(&s1.file, &s1.flow), stats::bootstrap(&s2.file, &s2.flow));
    let (e1, e2) = (verdict_of(r1), verdict_of(r2));
    let under = sample.eligible() < MIN_ELIGIBLE;
    let e4 = if under { "UNDERPOWERED (not passed)" } else { "PENDING" };
    let overall = if e1 == "FAIL" && e2 == "FAIL" {
        "DOES NOT HOLD (endpoints 1 and 2 failed)"
    } else if under {
        "DOES NOT HOLD (endpoint 4 underpowered)"
    } else {
        "UNDETERMINED until endpoint 4 is judged"
    };
    json!({
        "experiment": "E0c",
        "prereg": "docs/evidence/E0c/PREREG.md",
        "pins": {"corpus": CORPUS_SHA, "visionclaw": VISIONCLAW_SHA, "agentbox": AGENTBOX_SHA,
                 "sealmap": "0.2.0 crates in-tree, sealmap_rust::extract_dir with RustOptions::default()"},
        "flow_hash": "BLAKE3-16 over `sig <sig_hash>\\n` + `<exact|inferred|external> <target id>\\n` per call of Flow::calls()",
        "corpus": {
            "topics": all.len(),
            "sequence_citations": all.iter().flat_map(|t| &t.citations).filter(|c| c.kind == SEQ).count(),
            "topics_with_a_sequence_citation": all.iter().filter(|t| t.citations.iter().any(|c| c.kind == SEQ)).count(),
        },
        "repos": repos,
        "pooled": {"all": pooled.block(), "rs_only": pooled_rs.block()},
        "endpoint4": sample.to_json(),
        "verdict": {
            "endpoint1": {"r_flow": opt(r1), "ci95": [round(b1.lo), round(b1.hi)], "threshold": R_THRESHOLD, "result": e1},
            "endpoint2": {"r_flow_rs": opt(r2), "ci95": [round(b2.lo), round(b2.hi)], "threshold": R_THRESHOLD, "result": e2},
            "endpoint4": {"threshold": HIDDEN_CEILING, "eligible": sample.eligible(), "result": e4},
            "e0c": overall,
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
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn ci(b: &Value) -> String {
    format!("{}–{}", num(&b["ci95"][0]), num(&b["ci95"][1]))
}

fn results_md(v: &Value, prereg: &str) -> String {
    let mut o = String::from(
        "# E0c results\n\nGenerated by `e0 e0c` from the pinned inputs; byte-identical across runs. Pre-registration: \
[`PREREG.md`](PREREG.md). Scope: citations inside `sequenceDiagram` blocks only.\n\n",
    );
    let vd = &v["verdict"];
    o.push_str("## Verdict\n\n| Endpoint | Value | 95% CI | Threshold | Result |\n|---|---|---|---|---|\n");
    o.push_str(&format!(
        "| 1. R_flow, VisionClaw | {} | {} | ≥ 2.0 | **{}** |\n",
        num(&vd["endpoint1"]["r_flow"]),
        ci(&vd["endpoint1"]),
        num(&vd["endpoint1"]["result"])
    ));
    o.push_str(&format!(
        "| 2. R_flow, VisionClaw `.rs` only | {} | {} | ≥ 2.0 | **{}** |\n",
        num(&vd["endpoint2"]["r_flow_rs"]),
        ci(&vd["endpoint2"]),
        num(&vd["endpoint2"]["result"])
    ));
    o.push_str(&format!(
        "| 4. hidden-change rate | — | — | ≤ 10% | **{}** ({} eligible pairs) |\n\n**E0c: {}.**\n\n",
        num(&vd["endpoint4"]["result"]),
        num(&vd["endpoint4"]["eligible"]),
        num(&vd["e0c"])
    ));
    o.push_str("## Per-commit counts\n\nSums over each repository's 100 E0 window commits. R_flow = ΣT_file^seq / ΣT_flow; R_sym^seq = ΣT_file^seq / ΣT_sym^seq. CIs: bootstrap of commits, 10,000 resamples, seed 20261006.\n\n");
    o.push_str("| Row | ΣT_file^seq | ΣT_sym^seq | ΣT_flow | R_flow | CI | R_sym^seq | CI | median file/sym/flow | p90 file/sym/flow |\n|---|---|---|---|---|---|---|---|---|---|\n");
    let rows: [(&str, &[&str]); 6] = [
        ("VisionClaw", &["repos", "visionclaw", "all"]),
        ("VisionClaw `.rs`", &["repos", "visionclaw", "rs_only"]),
        ("agentbox", &["repos", "agentbox", "all"]),
        ("agentbox `.rs`", &["repos", "agentbox", "rs_only"]),
        ("pooled", &["pooled", "all"]),
        ("pooled `.rs`", &["pooled", "rs_only"]),
    ];
    for (name, path) in rows {
        let b = g(v, path);
        o.push_str(&format!(
            "| {name} | {} | {} | {} | {} | {} | {} | {} | {}/{}/{} | {}/{}/{} |\n",
            num(&b["sum"]["t_file_seq"]),
            num(&b["sum"]["t_sym_seq"]),
            num(&b["sum"]["t_flow"]),
            num(&b["r_flow"]),
            ci(&b["bootstrap_r_flow"]),
            num(&b["r_sym_seq"]),
            ci(&b["bootstrap_r_sym_seq"]),
            num(&b["median"]["t_file_seq"]),
            num(&b["median"]["t_sym_seq"]),
            num(&b["median"]["t_flow"]),
            num(&b["p90"]["t_file_seq"]),
            num(&b["p90"]["t_sym_seq"]),
            num(&b["p90"]["t_flow"]),
        ));
    }
    o.push_str("\nSensitivity (amendment #1, not an endpoint): T_flow with E0's absent-at-both rule added.\n\n| Row | ΣT_flow | R_flow |\n|---|---|---|\n");
    for (name, path) in rows {
        let b = &g(v, path)["sensitivity_absent_both"];
        o.push_str(&format!("| {name} | {} | {} |\n", num(&b["t_flow"]), num(&b["r_flow"])));
    }
    o.push_str("\n## Scope and coverage\n\n| | VisionClaw | agentbox |\n|---|---|---|\n");
    let cov = |k: &str, f: &str| num(&v["repos"][k]["coverage"][f]);
    for (label, f) in [
        ("topics in scope", "topics_in_scope"),
        ("topics in scope (`.rs`)", "topics_in_scope_rs"),
        ("sequence citations", "sequence_citations"),
        ("sequence citations (`.rs`)", "sequence_citations_rs"),
        ("fall back", "fallback"),
        ("fallback share", "fallback_share"),
        ("fallback share (`.rs`)", "fallback_share_rs"),
    ] {
        o.push_str(&format!("| {label} | {} | {} |\n", cov("visionclaw", f), cov("agentbox", f)));
    }
    for k in ["visionclaw", "agentbox"] {
        let c = &v["repos"][k]["coverage"];
        let reasons: Vec<String> = c["fallback_by_reason"]
            .as_object()
            .map(|m| m.iter().map(|(r, n)| format!("{r} {}", num(n))).collect())
            .unwrap_or_default();
        let s = &c["symbol_citations_at_stamp"];
        o.push_str(&format!(
            "\n{k}: fallback reasons: {}. Symbol citations at the stamp: {} with calls, {} without calls (flow = signature only), {} module.\n",
            if reasons.is_empty() { "none".into() } else { reasons.join(", ") },
            num(&s["flow_with_calls"]),
            num(&s["flow_without_calls"]),
            num(&s["module"])
        ));
    }
    o.push_str("\n## Why T_flow flagged\n\n| | VisionClaw | agentbox |\n|---|---|---|\n");
    for f in [
        "t_flow_by_flow_only",
        "t_flow_by_fallback_only",
        "t_flow_by_both",
        "t_flow_not_t_sym_seq",
        "t_file_seq_not_t_flow",
        "absent_both_unflagged",
    ] {
        o.push_str(&format!(
            "| {f} | {} | {} |\n",
            num(&v["repos"]["visionclaw"]["reasons"][f]),
            num(&v["repos"]["agentbox"]["reasons"][f])
        ));
    }
    o.push_str("\n## Drivers (exploratory)\n\nSymbol-change events (one per commit and symbol), by class: signature, calls (added, removed or retargeted), reordered, resolution_only, drift (flow changed with `sig_hash` and `body_hash` unchanged), one_sided.\n");
    for k in ["visionclaw", "agentbox"] {
        let d = &v["repos"][k]["drivers"];
        o.push_str(&format!("\n### {k}\n\nEvents by class: `{}`.\n\n", d["symbol_change_events_by_class"]));
        o.push_str(
            "| Symbol | T_flow flags | commits | classes | example (+added / −removed) |\n|---|---|---|---|---|\n",
        );
        for t in d["top"].as_array().into_iter().flatten() {
            let ex = &t["example"];
            o.push_str(&format!(
                "| `{}` | {} | {} | `{}` | {} {}: +{} −{} |\n",
                num(&t["symbol"]),
                num(&t["flags"]),
                num(&t["commits"]),
                t["classes"],
                num(&ex["driver"]),
                num(&ex["commit"]).get(..9).unwrap_or(""),
                num(&ex["added_total"]),
                num(&ex["removed_total"]),
            ));
        }
        o.push_str("\nCommits with the most symbol-change events:\n\n| Commit | events | classes | subject |\n|---|---|---|---|\n");
        for c in d["top_commits"].as_array().into_iter().flatten() {
            o.push_str(&format!(
                "| {} | {} | `{}` | {} |\n",
                num(&c["commit"]).get(..9).unwrap_or(""),
                num(&c["events"]),
                c["classes"],
                num(&c["subject"]).replace('|', "\\|"),
            ));
        }
    }
    let e4 = &v["endpoint4"];
    o.push_str(&format!(
        "\n## Endpoint 4\n\n**{}.** Question: *{}*\n\n| Stratum | T_file^seq ∧ ¬T_flow | eligible (after stamp) | drawn |\n|---|---|---|---|\n",
        num(&e4["status"]),
        QUESTION
    ));
    for k in ["visionclaw", "agentbox"] {
        o.push_str(&format!(
            "| {k} | {} | {} | {} |\n",
            num(&e4["population"][k]["t_file_seq_not_t_flow"]),
            num(&e4["population"][k]["eligible_post_stamp"]),
            num(&e4["drawn"][k])
        ));
    }
    o.push_str(&format!(
        "\nEligible in total: {}{}. Prompts, `sample.json` and `PROTOCOL.md` are in `judge/`.\n",
        num(&e4["eligible_total"]),
        if e4["underpowered"].as_bool() == Some(true) { " (< 20: underpowered, not passed)" } else { "" }
    ));
    let amendments: Vec<&str> = prereg.lines().filter(|l| l.starts_with("### ")).map(|l| &l[4..]).collect();
    o.push_str("\n## Amendments in force\n\n");
    if amendments.is_empty() {
        o.push_str("None.\n");
    }
    for a in amendments {
        o.push_str(&format!("- {a}\n"));
    }
    o
}

// ------------------------------------------------------------------ run

/// `e0 e0c`: compute E0c and write `results.json`, `RESULTS.md`, `TIMING.md` and `judge/`.
pub fn run(a: &Args) -> Result<(), String> {
    let t0 = Instant::now();
    let corpus = Repo::open(&a.corpus);
    if corpus.resolve_commit(CORPUS_SHA).as_deref() != Some(CORPUS_SHA) {
        return Err(format!("corpus pin {CORPUS_SHA} not found in {}", a.corpus.display()));
    }
    let all = crate::load_topics(&corpus)?;
    let mut timing = vec![("load topics".to_string(), t0.elapsed())];
    let mut runs = Vec::new();
    let mut repos = Vec::new();
    for key in RepoKey::ALL {
        let (dir, head) = match key {
            RepoKey::Visionclaw => (&a.visionclaw, VISIONCLAW_SHA),
            RepoKey::Agentbox => (&a.agentbox, AGENTBOX_SHA),
        };
        let repo = Repo::open(dir);
        let t = Instant::now();
        let e0 = crate::run_repo(key, &repo, head, &all, &a.scratch.join("e0").join(key.as_str()), false)?;
        timing.push((format!("{} E0 window and mapping ({} extractions)", key.as_str(), e0.extractions), t.elapsed()));
        let t = Instant::now();
        let r = run_repo(key, &repo, &e0, &a.scratch.join("e0c").join(key.as_str()))?;
        timing.push((format!("{} E0c replay ({} extractions)", key.as_str(), r.extractions), t.elapsed()));
        runs.push(r);
        repos.push((key, repo));
    }
    let t = Instant::now();
    std::fs::create_dir_all(&a.out).map_err(|e| format!("{}: {e}", a.out.display()))?;
    let sample = prepare(&a.out, &all, &runs, &repos)?;
    let prereg = std::fs::read_to_string(a.out.join("PREREG.md")).map_err(|e| format!("PREREG.md: {e}"))?;
    let results = results_json(&all, &runs, &sample);
    let text = serde_json::to_string_pretty(&results).map_err(|e| e.to_string())? + "\n";
    std::fs::write(a.out.join("results.json"), text).map_err(|e| format!("results.json: {e}"))?;
    std::fs::write(a.out.join("RESULTS.md"), results_md(&results, &prereg)).map_err(|e| format!("RESULTS.md: {e}"))?;
    timing.push(("sample, prompts, report".into(), t.elapsed()));
    timing.push(("total".into(), t0.elapsed()));
    let mut tm = String::from(
        "# E0c timing\n\nWall-clock times of the last `e0 e0c`. Not part of the byte-identical output (E0c amendment #1, after E0 amendment #2).\n\n| Step | Seconds |\n|---|---|\n",
    );
    for (k, d) in &timing {
        tm.push_str(&format!("| {k} | {:.1} |\n", d.as_secs_f64()));
    }
    std::fs::write(a.out.join("TIMING.md"), tm).map_err(|e| format!("TIMING.md: {e}"))?;
    let vd = &results["verdict"];
    eprintln!(
        "E0c: R_flow {} [{}], R_flow(.rs) {} [{}], endpoint 4 {} ({} eligible) → {}",
        num(&vd["endpoint1"]["r_flow"]),
        ci(&vd["endpoint1"]),
        num(&vd["endpoint2"]["r_flow_rs"]),
        ci(&vd["endpoint2"]),
        num(&vd["endpoint4"]["result"]),
        num(&vd["endpoint4"]["eligible"]),
        num(&vd["e0c"])
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::Fallback;
    use crate::topics::{self, Resolved};

    fn cite(kind: &str, file: &str, m: Mapped) -> (Citation, String, Mapped) {
        let c = Citation {
            diagram: "X-01.1".into(),
            prose: kind == "prose",
            kind: kind.into(),
            cited: file.into(),
            line: 1,
            resolved: Resolved::Source(file.into()),
        };
        (c, file.into(), m)
    }

    fn sym(id: &str) -> Mapped {
        Mapped::Symbol { id: id.into(), module: false }
    }

    #[test]
    fn non_sequence_citations_are_ignored() {
        let cites = vec![
            cite(SEQ, "a.rs", sym("sym:a")),
            cite("flowchart", "b.rs", sym("sym:b")),
            cite("prose", "c.rs", sym("sym:c")),
            cite(SEQ, "d.nix", Mapped::Fallback(Fallback::NotRust)),
        ];
        let t = seq_tracking(&cites, false).unwrap();
        assert_eq!(t.sources, ["a.rs", "d.nix"].map(String::from).into());
        assert_eq!(t.symbols, [("sym:a".to_string(), "a.rs".to_string())].into());
        assert_eq!(t.fallback_files, ["d.nix"].map(String::from).into());
        assert!(t.uncited_files.is_empty());
        // A flowchart-only change does not reach T_file^seq.
        let m = Model::default();
        let changed: BTreeSet<String> = ["b.rs".to_string()].into();
        assert!(!cell(&t, &changed, &m, &m).file);
        // `.rs` only drops the nix fallback.
        let rs = seq_tracking(&cites, true).unwrap();
        assert_eq!(rs.sources, ["a.rs"].map(String::from).into());
        // A topic with no sequence citation is out of scope.
        assert!(seq_tracking(&cites[1..3], false).is_none());
    }

    #[test]
    fn the_post_stamp_filter_keeps_only_changes_after_the_stamp() {
        // History: s0 → stamp → p1 → c1 ; c0 is the stamp commit itself (parent s0).
        let ancestors: BTreeSet<(&str, &str)> =
            [("stamp", "stamp"), ("stamp", "p1"), ("s0", "s0"), ("s0", "stamp"), ("s0", "p1")].into();
        let mut is_anc = |a: &str, b: &str| ancestors.contains(&(a, b));
        assert!(post_stamp(Some("stamp"), "p1", &mut is_anc), "parent after the stamp");
        assert!(post_stamp(Some("stamp"), "stamp", &mut is_anc), "parent is the stamp");
        assert!(!post_stamp(Some("stamp"), "s0", &mut is_anc), "the stamp commit itself is before");
        assert!(!post_stamp(None, "p1", &mut is_anc), "no stamp: never eligible");
        let pool = vec![
            ("c1".to_string(), "p1".to_string(), 3, true),
            ("c0".to_string(), "s0".to_string(), 3, false),
            ("c1".to_string(), "p1".to_string(), 5, true),
        ];
        assert_eq!(eligible(&pool), [("c1".to_string(), "p1".to_string(), 3), ("c1".to_string(), "p1".to_string(), 5)]);
    }

    #[test]
    fn prompts_carry_the_narrowed_question() {
        let t = topics::parse_topic(
            "docs/diagrams/agentbox/01-x.md",
            "---\nid: X-01\narea: agentbox\nsources:\n  - ../project/agentbox/a.rs\nverified_commit: abc1234\n---\n## X-01.1 T\n```mermaid\nsequenceDiagram\n  A->>B: go a.rs:3\n```\n",
        )
        .unwrap();
        let p = prompt("ab-01", "agentbox", &t, "diff --git a/a.rs b/a.rs\n");
        assert!(p.contains(QUESTION) && p.contains("sequenceDiagram") && p.contains("diff --git"));
    }
}
