//! E0d (`docs/evidence/E0d/PREREG.md`): language-agnostic hunk overlap
//! ([`crate::hunk`]) against file-level staleness, over E0's commit windows,
//! with every earlier detector recomputed on the same commits and scored on
//! one set of blind staleness labels.
//!
//! E0's `run_repo` (with E0b's regions) supplies the window, the citation
//! reading, T_file, T_sym, T_hop and T_region; E0c's `run_repo` supplies
//! T_flow; this module adds T_hunk(k), draws the label sample and scores it.
//!
//! ```text
//! e0 e0d --corpus <VisionFlow> --visionclaw <project> --agentbox <project/agentbox> \
//!        --out docs/evidence/E0d --scratch <dir>
//! e0 e0d-score --out docs/evidence/E0d   # endpoint 4, once labels exist
//! ```

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::git::Repo;
use crate::hunk::{self, HFlags, HTopic, Status, Store, Why};
use crate::stats;
use crate::topics::{RepoKey, Topic};
use crate::{AGENTBOX_SHA, Args, CORPUS_SHA, VISIONCLAW_SHA, e0c};

/// The pre-registered ratio threshold.
pub const R_THRESHOLD: f64 = 2.0;
/// The pre-registered recall floor for hunk(5) (and, by amendment #1, the reading of every detector).
pub const RECALL_FLOOR: f64 = 0.90;
/// Pairs drawn per stratum.
pub const STRATA: [(RepoKey, usize); 2] = [(RepoKey::Visionclaw, 45), (RepoKey::Agentbox, 15)];
/// The registered label question, verbatim.
pub const QUESTION: &str = "Did this change make anything this topic states wrong or misleading?";

/// Every detector scored, in report order: (key, label).
pub const DETECTORS: [(&str, &str); 10] = [
    ("file", "T_file (E0)"),
    ("sym", "T_sym (E0)"),
    ("hop", "T_hop (E0)"),
    ("region", "T_region (E0b)"),
    ("flow", "T_flow (E0c)"),
    ("hunk0", "T_hunk(0) (sensitivity)"),
    ("hunk5", "T_hunk(5) (primary)"),
    ("hunk20", "T_hunk(20) (sensitivity)"),
    ("hunk5_literal_lost", "T_hunk(5) literal-lost (sensitivity)"),
    ("hunk5_uncited", "T_hunk(5)+uncited (sensitivity)"),
];

/// One (commit, topic) cell: every detector's flag.
#[derive(Debug, Clone, Default)]
pub struct Cell {
    pub file: bool,
    pub sym: bool,
    pub hop: bool,
    pub region: bool,
    /// T_flow; `false` when the topic has no sequence citation (out of E0c's scope).
    pub flow: bool,
    pub flow_in_scope: bool,
    pub hunk: HFlags,
}

impl Cell {
    /// A detector's flag by its key in [`DETECTORS`].
    pub fn get(&self, key: &str) -> bool {
        match key {
            "file" => self.file,
            "sym" => self.sym,
            "hop" => self.hop,
            "region" => self.region,
            "flow" => self.flow,
            "hunk0" => self.hunk.k[0],
            "hunk5" => self.hunk.k[1],
            "hunk20" => self.hunk.k[2],
            "hunk5_literal_lost" => self.hunk.literal_lost,
            "hunk5_uncited" => self.hunk.uncited,
            _ => false,
        }
    }
}

/// One commit of the window.
pub struct Row {
    pub sha: String,
    pub parent: String,
    pub cells: Vec<(usize, Cell)>,
    /// E0c's per-commit T_file^seq and T_flow (its own scope), for the side-by-side R_flow.
    pub seq: (u32, u32),
}

/// Citation coverage of the overlap rule in one repository.
#[derive(Debug, Default, Serialize)]
pub struct Coverage {
    pub topics: usize,
    pub citations: usize,
    pub ranges: usize,
    pub non_rust_citations: usize,
    pub cited_files: usize,
    pub topics_with_uncited_sources: usize,
    pub uncited_sources: usize,
    /// Citations whose relocation is ambiguous at the stamp itself, by reason.
    pub ambiguous_at_stamp: BTreeMap<Why, usize>,
}

/// Everything computed for one repository.
pub struct Run {
    pub key: RepoKey,
    pub rows: Vec<Row>,
    pub coverage: Coverage,
    /// Endpoint-4 population: (sha, parent, topic) flagged by T_file whose change postdates the stamp.
    pub pool: Vec<(String, String, usize)>,
    /// Pairs flagged by T_file, before the after-stamp filter.
    pub file_flagged: usize,
    pub diff_calls: usize,
    pub extractions: usize,
}

/// Build one topic's overlap tracking from E0's mapped citations.
pub fn htopic(t: &crate::TopicInRepo) -> HTopic {
    let mut cites: BTreeMap<String, Vec<(u32, u32)>> = BTreeMap::new();
    for (c, rel, _) in &t.cites {
        let end = c.end.filter(|e| *e >= c.line).unwrap_or(c.line);
        cites.entry(rel.clone()).or_default().push((c.line, end));
    }
    HTopic { topic: t.topic, stamp: t.stamp.clone(), cites, uncited: t.tracking.uncited_files.clone() }
}

fn coverage(tracked: &[crate::TopicInRepo], topics: &[HTopic], store: &mut Store) -> Result<Coverage, String> {
    let mut cov = Coverage { topics: topics.len(), ..Coverage::default() };
    for t in tracked {
        cov.citations += t.cites.len();
        cov.ranges += t.cites.iter().filter(|(c, _, _)| c.end.is_some_and(|e| e > c.line)).count();
        cov.non_rust_citations += t.cites.iter().filter(|(_, f, _)| !f.ends_with(".rs")).count();
    }
    for h in topics {
        cov.cited_files += h.cites.len();
        cov.uncited_sources += h.uncited.len();
        cov.topics_with_uncited_sources += usize::from(!h.uncited.is_empty());
        for (file, spans) in &h.cites {
            let at = h.stamp.as_deref().unwrap_or("");
            let st = if h.stamp.is_some() {
                store.relocate(h.stamp.as_deref(), at, file, spans)?
            } else {
                vec![Status::Ambiguous(Why::NoStamp); spans.len()]
            };
            for s in st {
                if let Status::Ambiguous(w) = s {
                    *cov.ambiguous_at_stamp.entry(w).or_default() += 1;
                }
            }
        }
    }
    Ok(cov)
}

/// Replay one repository's E0 window with every detector.
pub fn run_repo(key: RepoKey, repo: &Repo, all: &[Topic], scratch: &Path) -> Result<Run, String> {
    let head = match key {
        RepoKey::Visionclaw => VISIONCLAW_SHA,
        RepoKey::Agentbox => AGENTBOX_SHA,
    };
    let e0 = crate::run_repo(key, repo, head, all, &scratch.join("e0"), true)?;
    let seq = e0c::run_repo(key, repo, &e0, &scratch.join("e0c"))?;
    let topics: Vec<HTopic> = e0.tracked.iter().map(htopic).collect();
    let mut store = Store::new(repo.clone());
    let cov = coverage(&e0.tracked, &topics, &mut store)?;
    let mut rows = Vec::new();
    for (i, r) in e0.rows.iter().enumerate() {
        let changed: BTreeSet<String> = repo.changed_paths(&r.parent, &r.sha)?.into_iter().collect();
        let srow = &seq.rows[i];
        if srow.sha != r.sha {
            return Err(format!("E0c row {i} is {} where E0 has {}", srow.sha, r.sha));
        }
        let mut cells = Vec::new();
        for (j, h) in topics.iter().enumerate() {
            let (ti, f) = &r.flags[j];
            let (rti, rf) = &r.rflags[j];
            if *ti != h.topic || *rti != h.topic {
                return Err(format!("topic order differs at commit {}", r.sha));
            }
            let fl = srow.cells.iter().find(|(t, _)| *t == h.topic);
            cells.push((
                h.topic,
                Cell {
                    file: f.file,
                    sym: f.sym,
                    hop: f.hop,
                    region: rf.region,
                    flow: fl.is_some_and(|(_, c)| c.flow.flow),
                    flow_in_scope: fl.is_some(),
                    hunk: hunk::flag(&mut store, h, &r.parent, &r.sha, &changed)?,
                },
            ));
        }
        let seq_counts = (
            srow.cells.iter().filter(|(_, c)| c.file).count() as u32,
            srow.cells.iter().filter(|(_, c)| c.flow.flow).count() as u32,
        );
        rows.push(Row { sha: r.sha.clone(), parent: r.parent.clone(), cells, seq: seq_counts });
    }
    let mut anc: HashMap<(String, String), bool> = HashMap::new();
    let mut is_anc = |a: &str, b: &str| *anc.entry((a.into(), b.into())).or_insert_with(|| repo.is_ancestor(a, b));
    let (mut pool, mut file_flagged) = (Vec::new(), 0);
    for r in &rows {
        for (ti, c) in &r.cells {
            if !c.file {
                continue;
            }
            file_flagged += 1;
            let stamp = topics.iter().find(|t| t.topic == *ti).and_then(|t| t.stamp.as_deref());
            if e0c::post_stamp(stamp, &r.parent, &mut is_anc) {
                pool.push((r.sha.clone(), r.parent.clone(), *ti));
            }
        }
    }
    Ok(Run {
        key,
        rows,
        coverage: cov,
        pool,
        file_flagged,
        diff_calls: store.diff_calls,
        extractions: e0.extractions + seq.extractions,
    })
}

// ------------------------------------------------------------------ statistics

fn round(x: f64) -> Value {
    if x.is_finite() { json!((x * 10_000.0).round() / 10_000.0) } else { Value::Null }
}

fn opt(x: Option<f64>) -> Value {
    x.map_or(Value::Null, round)
}

/// Per-commit counts for every detector, plus E0c's own-scope pair.
#[derive(Default)]
struct Series {
    by: BTreeMap<&'static str, Vec<u32>>,
    seq_file: Vec<u32>,
    seq_flow: Vec<u32>,
}

impl Series {
    fn of(rows: &[Row]) -> Self {
        let mut s = Series::default();
        for r in rows {
            for (k, _) in DETECTORS {
                s.by.entry(k).or_default().push(r.cells.iter().filter(|(_, c)| c.get(k)).count() as u32);
            }
            s.seq_file.push(r.seq.0);
            s.seq_flow.push(r.seq.1);
        }
        s
    }

    fn extend(&mut self, o: Series) {
        for (k, v) in o.by {
            self.by.entry(k).or_default().extend(v);
        }
        self.seq_file.extend(o.seq_file);
        self.seq_flow.extend(o.seq_flow);
    }

    fn ratio_block(num: &[u32], den: &[u32]) -> Value {
        let b = stats::bootstrap(num, den);
        let sum = |v: &[u32]| v.iter().map(|&x| u64::from(x)).sum::<u64>();
        json!({
            "sum": sum(den), "median": round(stats::median(den)), "p90": stats::p90(den),
            "r": opt(stats::ratio(num, den)), "ci95": [round(b.lo), round(b.hi)], "infinite_resamples": b.infinite,
        })
    }

    fn block(&self) -> Value {
        let file = &self.by["file"];
        let mut d = serde_json::Map::new();
        for (k, _) in DETECTORS {
            d.insert(k.into(), Self::ratio_block(file, &self.by[k]));
        }
        let sum = |v: &[u32]| v.iter().map(|&x| u64::from(x)).sum::<u64>();
        let fb = stats::bootstrap(&self.seq_file, &self.seq_flow);
        json!({
            "commits": file.len(),
            "detectors": d,
            "flow_e0c_scope": {
                "t_file_seq": sum(&self.seq_file), "t_flow": sum(&self.seq_flow),
                "r_flow": opt(stats::ratio(&self.seq_file, &self.seq_flow)), "ci95": [round(fb.lo), round(fb.hi)],
            },
        })
    }
}

fn reasons(r: &Run) -> Value {
    let mut n: BTreeMap<&str, usize> = BTreeMap::new();
    let (mut hunk_not_file, mut monotone_breaks, mut only_overlap, mut only_whole) = (0, 0, 0, 0);
    for row in &r.rows {
        for (_, c) in &row.cells {
            let h = &c.hunk;
            for (k, v) in [
                ("overlap_k5", h.by_overlap5),
                ("deleted", h.by_deleted),
                ("lost", h.by_lost),
                ("ambiguous", h.by_ambiguous),
                ("binary", h.by_binary),
            ] {
                if h.k[1] && v {
                    *n.entry(k).or_default() += 1;
                }
            }
            let whole = h.by_deleted || h.by_lost || h.by_ambiguous || h.by_binary;
            if h.k[1] {
                only_overlap += usize::from(h.by_overlap5 && !whole);
                only_whole += usize::from(!h.by_overlap5 && whole);
            }
            hunk_not_file += usize::from(h.k[2] && !c.file);
            monotone_breaks += usize::from((h.k[0] && !h.k[1]) || (h.k[1] && !h.k[2]));
        }
    }
    json!({
        "t_hunk5_flags_by_reason": n,
        "t_hunk5_only_overlap": only_overlap,
        "t_hunk5_only_whole_file_rule": only_whole,
        "check_t_hunk20_without_t_file": hunk_not_file,
        "check_k_monotonicity_breaks": monotone_breaks,
    })
}

fn verdict_of(r: Option<f64>) -> &'static str {
    if r.is_some_and(|x| x >= R_THRESHOLD) { "PASS" } else { "FAIL" }
}

// ------------------------------------------------------------------ endpoint 4 sample

/// One drawn pair, as the labellers' side sees it (no detector output).
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
    /// Per stratum: (repo, pairs flagged by T_file, of which after the stamp).
    pub population: Vec<(String, usize, usize)>,
    pub pairs: Vec<Pair>,
}

/// The detectors' flags for each drawn pair; kept out of `labels/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detectors {
    pub note: String,
    pub detectors: Vec<String>,
    /// id → detector key → flagged.
    pub flags: BTreeMap<String, BTreeMap<String, bool>>,
    /// id → whether the topic is in E0c's scope (has a sequence citation).
    pub flow_in_scope: BTreeMap<String, bool>,
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

/// The label prompt: the topic, the diff and the registered question, and nothing a detector computed.
pub fn prompt(id: &str, repo: &str, topic: &Topic, diff: &str) -> String {
    let (tf, df) = (fence(&topic.text), fence(diff));
    format!(
        "# Label {id}\n\n\
You are an independent labeller for a pre-registered experiment. Below are (1) a TOPIC: a Markdown file of Mermaid \
diagrams, with short prose notes, that document code, with `path:line` citations, where paths under `../project/` are \
in the VisionClaw repository and paths under `../project/agentbox/` are in the agentbox repository; and (2) a DIFF: \
one commit's changes in the {repo} repository, restricted to the files the topic lists under `sources:`. The topic \
was verified against a revision of that repository at or before this commit's parent, so the diff is a change made \
after the topic was written.\n\n\
Answer exactly one question:\n\n> **{QUESTION}**\n\n\
Judge only from the two texts below; do not look anything else up. Reply with a single JSON object and nothing else:\n\n\
```json\n{{\"verdict\": \"yes\" | \"no\", \"statements\": [\"<diagram or section id>: <the statement, quoted>\", ...], \
\"line_anchors_only\": true | false, \"reason\": \"<one or two sentences>\"}}\n```\n\n\
`statements` lists every statement in the topic (a message, node, edge, guard, label or prose claim) that the change \
makes wrong or misleading; it is empty when the verdict is no. `line_anchors_only` is true when the only such \
statements are `path:line` anchors whose cited code merely moved.\n\n\
## TOPIC ({file})\n\n{tf}markdown\n{text}\n{tf}\n\n## DIFF\n\n{df}diff\n{diff}\n{df}\n",
        file = topic.file,
        text = topic.text.trim_end(),
        diff = diff.trim_end(),
    )
}

const PROTOCOL: &str = "# E0d endpoint 4 labelling protocol\n\n\
Generated by `e0 e0d`. Do not edit. Implements endpoint 4 of `docs/evidence/E0d/PREREG.md` (amendment #1).\n\n\
The sample is (commit, topic) pairs whose commit changed one of the topic's `sources:` files and whose parent is the \
topic's stamp or a descendant of it, drawn uniformly per stratum (45 VisionClaw, 15 agentbox; seed 20261006). It \
was drawn without reference to any detector.\n\n\
## Blindness\n\n\
A labeller sees **only** the text of one `<id>.prompt.md`. It must not be given, and must not be able to read, \
`../detectors.json`, `../results.json`, `../RESULTS.md`, `../TIMING.md`, any other label file, or any E0/E0b/E0c/E0d \
`judge/` or `labels/` directory. Use a fresh subagent per prompt with no repository access (or, failing that, the \
instruction to read nothing), and give it the prompt text verbatim as its whole task. The context that wrote the \
harness or drew the sample never labels.\n\n\
## Steps\n\n\
1. Each `<id>.prompt.md` goes, unchanged, to a fresh labeller. Its reply (one JSON object) is saved as \
`<id>.l1.json`.\n\
2. Every pair whose first verdict is `yes` goes, unchanged, to a second fresh labeller that does not see the first \
reply. Its reply is saved as `<id>.l2.json`. A first `no` gets no second label.\n\
3. A pair is **stale** when both labellers answer `yes`.\n\
4. `e0 e0d-score --out docs/evidence/E0d` joins the labels with `detectors.json` and writes `endpoint4.json` and \
`ENDPOINT4.md`: recall and precision, each with a Wilson 95% interval, for every detector. **Success:** recall of \
T_hunk(5) ≥ 0.90. A detector whose flag ratio clears 2.0 with recall below 0.90 reads as a fail.\n\n\
`line_anchors_only` is recorded. A sensitivity row (not the endpoint) drops pairs where both labellers set it.\n";

fn prepare(out: &Path, all: &[Topic], runs: &[Run], repos: &[(RepoKey, Repo)]) -> Result<(Sample, Detectors), String> {
    let dir = out.join("labels");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for e in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        if e.file_name().to_string_lossy().ends_with(".prompt.md") {
            let _ = std::fs::remove_file(e.path());
        }
    }
    let mut rng = stats::rng();
    let (mut population, mut pairs) = (Vec::new(), Vec::new());
    let mut det = Detectors {
        note: "Detector flags for the drawn pairs. Never shown to a labeller (labels/PROTOCOL.md).".into(),
        detectors: DETECTORS.iter().map(|(k, _)| k.to_string()).collect(),
        flags: BTreeMap::new(),
        flow_in_scope: BTreeMap::new(),
    };
    for (key, k) in STRATA {
        let run = runs.iter().find(|r| r.key == key).ok_or("missing run")?;
        let repo = &repos.iter().find(|(r, _)| *r == key).ok_or("missing repo")?.1;
        population.push((key.as_str().to_string(), run.file_flagged, run.pool.len()));
        let prefix = match key {
            RepoKey::Visionclaw => "vc",
            RepoKey::Agentbox => "ab",
        };
        for (i, (sha, parent, ti)) in stats::draw(&run.pool, k, &mut rng).into_iter().enumerate() {
            let id = format!("{prefix}-{:02}", i + 1);
            let topic = &all[ti];
            let files: Vec<String> = topic
                .sources
                .iter()
                .filter_map(|s| match crate::topics::repo_of(s) {
                    (Some(r), rel) if r == key.as_str() => Some(rel),
                    _ => None,
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let diff = repo.diff(&parent, &sha, &files)?;
            let file = format!("{id}.prompt.md");
            std::fs::write(dir.join(&file), prompt(&id, key.as_str(), topic, &diff)).map_err(|e| e.to_string())?;
            let row = run.rows.iter().find(|r| r.sha == sha).ok_or("drawn commit not in window")?;
            let cell = &row.cells.iter().find(|(t, _)| *t == ti).ok_or("drawn topic not tracked")?.1;
            det.flags.insert(id.clone(), DETECTORS.iter().map(|(k, _)| (k.to_string(), cell.get(k))).collect());
            det.flow_in_scope.insert(id.clone(), cell.flow_in_scope);
            pairs.push(Pair {
                id,
                repo: key.as_str().into(),
                commit: sha,
                parent,
                topic: topic.id.clone(),
                diff_files: files,
                diff_lines: diff.lines().count(),
                prompt: format!("labels/{file}"),
            });
        }
    }
    let sample = Sample { population, pairs };
    let text = serde_json::to_string_pretty(&sample).map_err(|e| e.to_string())? + "\n";
    std::fs::write(dir.join("sample.json"), text).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("PROTOCOL.md"), PROTOCOL).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(&det).map_err(|e| e.to_string())? + "\n";
    std::fs::write(out.join("detectors.json"), text).map_err(|e| e.to_string())?;
    Ok((sample, det))
}

// ------------------------------------------------------------------ report

fn results_json(all: &[Topic], runs: &[Run], sample: &Sample, det: &Detectors) -> Value {
    let mut repos = serde_json::Map::new();
    let mut pooled = Series::default();
    for r in runs {
        let s = Series::of(&r.rows);
        repos.insert(
            r.key.as_str().into(),
            json!({
                "coverage": r.coverage,
                "counts": s.block(),
                "reasons": reasons(r),
                "endpoint4_population": {"t_file_pairs": r.file_flagged, "after_stamp": r.pool.len()},
            }),
        );
        pooled.extend(s);
    }
    let vc = Series::of(&runs[0].rows);
    let (f, h) = (&vc.by["file"], &vc.by["hunk5"]);
    let (pf, ph) = (&pooled.by["file"], &pooled.by["hunk5"]);
    let (r1, r2) = (stats::ratio(f, h), stats::ratio(pf, ph));
    let (b1, b2) = (stats::bootstrap(f, h), stats::bootstrap(pf, ph));
    let (e1, e2) = (verdict_of(r1), verdict_of(r2));
    let overall = if e1 == "FAIL" && e2 == "FAIL" {
        "DOES NOT HOLD (endpoints 1 and 2 failed)"
    } else {
        "UNDETERMINED until endpoint 4 is labelled"
    };
    let drawn = |k: &str| sample.pairs.iter().filter(|p| p.repo == k).count();
    let in_sample: BTreeMap<&str, usize> = DETECTORS
        .iter()
        .map(|(k, _)| (*k, det.flags.values().filter(|m| m.get(*k).copied().unwrap_or(false)).count()))
        .collect();
    json!({
        "experiment": "E0d",
        "prereg": "docs/evidence/E0d/PREREG.md",
        "pins": {"corpus": CORPUS_SHA, "visionclaw": VISIONCLAW_SHA, "agentbox": AGENTBOX_SHA,
                 "sealmap": "0.2.0 crates in-tree, sealmap_rust::extract_dir with RustOptions::default()",
                 "git": Repo::git_version()},
        "overlap_rule": "git diff -U0 --diff-algorithm=myers --indent-heuristic --no-textconv --no-ext-diff over blob ids; stamp→P relocation, P→C overlap within k (amendment #1)",
        "k_primary": 5,
        "corpus": {"topics": all.len()},
        "repos": repos,
        "pooled": {"counts": pooled.block()},
        "endpoint4": {
            "status": "PENDING",
            "reason": "no subagent tool in the harness context; blind prompts prepared, no label produced",
            "question": QUESTION,
            "seed": stats::SEED,
            "population": sample.population.iter().map(|(k, a, b)| json!({"repo": k, "t_file_pairs": a, "after_stamp": b})).collect::<Vec<_>>(),
            "drawn": {"visionclaw": drawn("visionclaw"), "agentbox": drawn("agentbox"), "total": sample.pairs.len()},
            "flagged_in_sample": in_sample,
        },
        "verdict": {
            "endpoint1": {"r_hunk": opt(r1), "ci95": [round(b1.lo), round(b1.hi)], "threshold": R_THRESHOLD, "result": e1},
            "endpoint2": {"r_hunk_pooled": opt(r2), "ci95": [round(b2.lo), round(b2.hi)], "threshold": R_THRESHOLD, "result": e2},
            "endpoint4": {"recall_floor": RECALL_FLOOR, "result": "PENDING"},
            "e0d": overall,
        },
    })
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
        "# E0d results\n\nGenerated by `e0 e0d` from the pinned inputs; byte-identical across runs (timing is in \
`TIMING.md`). Pre-registration: [`PREREG.md`](PREREG.md). Machine-readable: [`results.json`](results.json).\n\n",
    );
    let vd = &v["verdict"];
    o.push_str("## Verdict\n\n| Endpoint | Value | 95% CI | Threshold | Result |\n|---|---|---|---|---|\n");
    o.push_str(&format!(
        "| 1. R_hunk = ΣT_file / ΣT_hunk(5), VisionClaw | {} | {} | ≥ 2.0 | **{}** |\n",
        num(&vd["endpoint1"]["r_hunk"]),
        ci(&vd["endpoint1"]),
        num(&vd["endpoint1"]["result"])
    ));
    o.push_str(&format!(
        "| 2. R_hunk, VisionClaw + agentbox pooled | {} | {} | ≥ 2.0 | **{}** |\n",
        num(&vd["endpoint2"]["r_hunk_pooled"]),
        ci(&vd["endpoint2"]),
        num(&vd["endpoint2"]["result"])
    ));
    o.push_str(&format!(
        "| 4. recall of T_hunk(5) on blind labels | — | — | ≥ 0.90 | **{}** |\n\n**E0d: {}.** A ratio is read \
alongside its detector's recall (endpoint 4); a high ratio with low recall is a fail.\n\n",
        num(&vd["endpoint4"]["result"]),
        num(&vd["e0d"])
    ));
    o.push_str("## Per-commit counts, every detector on the same commits\n\nΣ over each repository's 100 E0 window commits; R = ΣT_file / ΣT_x; bootstrap of commits, 10,000 resamples, seed 20261006. T_region is E0b's endpoint-1 form; T_flow is counted as not flagging a topic with no sequence citation. Rows other than T_hunk(5) carry no threshold.\n\n");
    let blocks: [(&str, &Value); 3] = [
        ("VisionClaw", &v["repos"]["visionclaw"]["counts"]),
        ("agentbox", &v["repos"]["agentbox"]["counts"]),
        ("pooled", &v["pooled"]["counts"]),
    ];
    for (name, b) in blocks {
        o.push_str(&format!(
            "### {name} ({} commits)\n\n| Detector | ΣT | median | p90 | R | 95% CI |\n|---|---|---|---|---|---|\n",
            num(&b["commits"])
        ));
        for (k, label) in DETECTORS {
            let d = &b["detectors"][k];
            o.push_str(&format!(
                "| {label} | {} | {} | {} | {} | {} |\n",
                num(&d["sum"]),
                num(&d["median"]),
                num(&d["p90"]),
                num(&d["r"]),
                ci(d)
            ));
        }
        let fl = &b["flow_e0c_scope"];
        o.push_str(&format!(
            "| R_flow in E0c's own scope (ΣT_file^seq {} / ΣT_flow {}) | | | | {} | {} |\n\n",
            num(&fl["t_file_seq"]),
            num(&fl["t_flow"]),
            num(&fl["r_flow"]),
            ci(fl)
        ));
    }
    o.push_str("## What sets T_hunk(5)\n\n| | VisionClaw | agentbox |\n|---|---|---|\n");
    for k in ["overlap_k5", "deleted", "lost", "ambiguous", "binary"] {
        o.push_str(&format!(
            "| flags with reason `{k}` | {} | {} |\n",
            num(&v["repos"]["visionclaw"]["reasons"]["t_hunk5_flags_by_reason"][k]),
            num(&v["repos"]["agentbox"]["reasons"]["t_hunk5_flags_by_reason"][k])
        ));
    }
    for (label, k) in [
        ("only overlap", "t_hunk5_only_overlap"),
        ("only the whole-file rule (deleted, lost, ambiguous, binary)", "t_hunk5_only_whole_file_rule"),
        ("check: T_hunk(20) without T_file (must be 0)", "check_t_hunk20_without_t_file"),
        ("check: k-monotonicity breaks (must be 0)", "check_k_monotonicity_breaks"),
    ] {
        o.push_str(&format!(
            "| {label} | {} | {} |\n",
            num(&v["repos"]["visionclaw"]["reasons"][k]),
            num(&v["repos"]["agentbox"]["reasons"][k])
        ));
    }
    o.push_str("\n## Coverage\n\n| | VisionClaw | agentbox |\n|---|---|---|\n");
    for k in [
        "topics",
        "citations",
        "ranges",
        "non_rust_citations",
        "cited_files",
        "topics_with_uncited_sources",
        "uncited_sources",
    ] {
        o.push_str(&format!(
            "| {k} | {} | {} |\n",
            num(&v["repos"]["visionclaw"]["coverage"][k]),
            num(&v["repos"]["agentbox"]["coverage"][k])
        ));
    }
    for k in ["no_stamp", "absent_at_stamp", "line_out_of_range", "binary"] {
        o.push_str(&format!(
            "| ambiguous at the stamp: {k} | {} | {} |\n",
            num(&v["repos"]["visionclaw"]["coverage"]["ambiguous_at_stamp"][k]),
            num(&v["repos"]["agentbox"]["coverage"]["ambiguous_at_stamp"][k])
        ));
    }
    let e4 = &v["endpoint4"];
    o.push_str(&format!(
        "\n## Endpoint 4\n\n**{}.** Question, verbatim: *{}*\n\n| Stratum | pairs flagged by T_file | after the stamp (population) | drawn |\n|---|---|---|---|\n",
        num(&e4["status"]),
        QUESTION
    ));
    for p in e4["population"].as_array().into_iter().flatten() {
        let k = p["repo"].as_str().unwrap_or("");
        o.push_str(&format!(
            "| {k} | {} | {} | {} |\n",
            num(&p["t_file_pairs"]),
            num(&p["after_stamp"]),
            num(&e4["drawn"][k])
        ));
    }
    o.push_str(&format!(
        "\n{} pairs drawn. Blind prompts, `sample.json` and `PROTOCOL.md` are in `labels/`; each detector's flag for each drawn pair is in `detectors.json`, which no labeller may see. Score with `e0 e0d-score --out docs/evidence/E0d` once labels exist.\n",
        num(&e4["drawn"]["total"])
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

/// `e0 e0d`: compute E0d and write `results.json`, `RESULTS.md`, `detectors.json`, `TIMING.md` and `labels/`.
pub fn run(a: &Args) -> Result<(), String> {
    let t0 = Instant::now();
    let corpus = Repo::open(&a.corpus);
    if corpus.resolve_commit(CORPUS_SHA).as_deref() != Some(CORPUS_SHA) {
        return Err(format!("corpus pin {CORPUS_SHA} not found in {}", a.corpus.display()));
    }
    let all = crate::load_topics(&corpus)?;
    let mut timing = vec![("load topics".to_string(), t0.elapsed())];
    let (mut runs, mut repos) = (Vec::new(), Vec::new());
    for key in RepoKey::ALL {
        let dir = match key {
            RepoKey::Visionclaw => &a.visionclaw,
            RepoKey::Agentbox => &a.agentbox,
        };
        let repo = Repo::open(dir);
        let t = Instant::now();
        let r = run_repo(key, &repo, &all, &a.scratch.join(key.as_str()))?;
        timing.push((
            format!("{} ({} extractions, {} blob diffs)", key.as_str(), r.extractions, r.diff_calls),
            t.elapsed(),
        ));
        runs.push(r);
        repos.push((key, repo));
    }
    let t = Instant::now();
    std::fs::create_dir_all(&a.out).map_err(|e| format!("{}: {e}", a.out.display()))?;
    let (sample, det) = prepare(&a.out, &all, &runs, &repos)?;
    let prereg = std::fs::read_to_string(a.out.join("PREREG.md")).map_err(|e| format!("PREREG.md: {e}"))?;
    let results = results_json(&all, &runs, &sample, &det);
    let text = serde_json::to_string_pretty(&results).map_err(|e| e.to_string())? + "\n";
    std::fs::write(a.out.join("results.json"), text).map_err(|e| format!("results.json: {e}"))?;
    std::fs::write(a.out.join("RESULTS.md"), results_md(&results, &prereg)).map_err(|e| format!("RESULTS.md: {e}"))?;
    timing.push(("sample, prompts, report".into(), t.elapsed()));
    timing.push(("total".into(), t0.elapsed()));
    let mut tm = String::from(
        "# E0d timing\n\nWall-clock times of the last `e0 e0d`. Not part of the byte-identical output (E0d amendment #1, after E0 amendment #2).\n\n| Step | Seconds |\n|---|---|\n",
    );
    for (k, d) in &timing {
        tm.push_str(&format!("| {k} | {:.1} |\n", d.as_secs_f64()));
    }
    std::fs::write(a.out.join("TIMING.md"), tm).map_err(|e| format!("TIMING.md: {e}"))?;
    let vd = &results["verdict"];
    eprintln!(
        "E0d: R_hunk {} [{}], pooled {} [{}], endpoint 4 PENDING ({} pairs drawn) → {}",
        num(&vd["endpoint1"]["r_hunk"]),
        ci(&vd["endpoint1"]),
        num(&vd["endpoint2"]["r_hunk_pooled"]),
        ci(&vd["endpoint2"]),
        sample.pairs.len(),
        num(&vd["e0d"])
    );
    Ok(())
}

// ------------------------------------------------------------------ scoring

/// One label reply.
#[derive(Debug, Deserialize)]
pub struct Label {
    pub verdict: String,
    #[serde(default)]
    pub line_anchors_only: bool,
}

/// A label file as (yes, line_anchors_only), or `None` when absent.
fn label(path: &Path) -> Result<Option<(bool, bool)>, String> {
    let Ok(text) = std::fs::read_to_string(path) else { return Ok(None) };
    let l: Label = serde_json::from_str(text.trim()).map_err(|e| format!("{}: {e}", path.display()))?;
    match l.verdict.to_ascii_lowercase().as_str() {
        "yes" => Ok(Some((true, l.line_anchors_only))),
        "no" => Ok(Some((false, l.line_anchors_only))),
        other => Err(format!("{}: verdict `{other}` is neither yes nor no", path.display())),
    }
}

/// One labelled pair.
#[derive(Debug, Clone)]
pub struct Labelled {
    pub id: String,
    pub repo: String,
    pub stale: bool,
    /// Stale, unless both labellers rest their yes on moved anchors only.
    pub stale_strict: bool,
    pub flags: BTreeMap<String, bool>,
}

fn interval(k: usize, n: usize) -> Value {
    match stats::wilson(k, n) {
        Some((lo, hi)) => json!({"value": round(k as f64 / n as f64), "wilson95": [round(lo), round(hi)]}),
        None => json!({"value": Value::Null, "wilson95": Value::Null}),
    }
}

/// Recall and precision of one detector over `rows`, against `stale_of`.
pub fn score_detector(rows: &[&Labelled], key: &str, stale_of: impl Fn(&Labelled) -> bool) -> Value {
    let stale = rows.iter().filter(|r| stale_of(r)).count();
    let flagged = rows.iter().filter(|r| r.flags.get(key).copied().unwrap_or(false)).count();
    let tp = rows.iter().filter(|r| stale_of(r) && r.flags.get(key).copied().unwrap_or(false)).count();
    json!({"n": rows.len(), "stale": stale, "flagged": flagged, "stale_flagged": tp,
           "recall": interval(tp, stale), "precision": interval(tp, flagged)})
}

/// The joint reading of a ratio and a recall (amendment #1).
pub fn reading(ratio_passes: bool, recall: Option<f64>) -> &'static str {
    match (ratio_passes, recall) {
        (_, None) => "UNDEFINED (no stale pair)",
        (true, Some(r)) if r >= RECALL_FLOOR => "PASS",
        (true, Some(_)) => "FAIL (ratio without recall)",
        (false, Some(r)) if r >= RECALL_FLOOR => "FAIL (ratio)",
        (false, Some(_)) => "FAIL (ratio and recall)",
    }
}

/// `e0 e0d-score`: join the labels with the detectors' flags and write `endpoint4.json` and `ENDPOINT4.md`.
pub fn score(out: &Path) -> Result<(), String> {
    let dir = out.join("labels");
    let read = |p: &Path| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let sample: Sample = serde_json::from_str(&read(&dir.join("sample.json"))?).map_err(|e| format!("sample: {e}"))?;
    let det: Detectors =
        serde_json::from_str(&read(&out.join("detectors.json"))?).map_err(|e| format!("detectors: {e}"))?;
    let results: Value =
        serde_json::from_str(&read(&out.join("results.json"))?).map_err(|e| format!("results: {e}"))?;
    let (mut missing, mut rows, mut first_yes) = (Vec::new(), Vec::new(), 0);
    for p in &sample.pairs {
        let l1 = label(&dir.join(format!("{}.l1.json", p.id)))?;
        first_yes += usize::from(l1.is_some_and(|l| l.0));
        let l2 = label(&dir.join(format!("{}.l2.json", p.id)))?;
        let (stale, anchors) = match (l1, l2) {
            (None, _) => {
                missing.push(format!("{}.l1.json", p.id));
                continue;
            }
            (Some((true, _)), None) => {
                missing.push(format!("{}.l2.json", p.id));
                continue;
            }
            (Some((true, a1)), Some((y2, a2))) => (y2, y2 && a1 && a2),
            (Some((false, _)), _) => (false, false),
        };
        let flags = det.flags.get(&p.id).cloned().ok_or_else(|| format!("detectors.json has no {}", p.id))?;
        rows.push(Labelled { id: p.id.clone(), repo: p.repo.clone(), stale, stale_strict: stale && !anchors, flags });
    }
    if !missing.is_empty() {
        return Err(format!("missing labels: {}", missing.join(", ")));
    }
    let strata: [(&str, Option<&str>); 3] =
        [("pooled", None), ("visionclaw", Some("visionclaw")), ("agentbox", Some("agentbox"))];
    let mut table = serde_json::Map::new();
    let mut strict = serde_json::Map::new();
    for (name, k) in strata {
        let sel: Vec<&Labelled> = rows.iter().filter(|r| k.is_none_or(|k| r.repo == k)).collect();
        let mut by = serde_json::Map::new();
        let mut by_s = serde_json::Map::new();
        for (d, _) in DETECTORS {
            by.insert(d.into(), score_detector(&sel, d, |r| r.stale));
            by_s.insert(d.into(), score_detector(&sel, d, |r| r.stale_strict));
        }
        table.insert(name.into(), Value::Object(by));
        strict.insert(name.into(), Value::Object(by_s));
    }
    // Joint readings: each detector's VisionClaw ratio (hunk(5): endpoint 1 or 2) with its pooled recall.
    let vc = &results["repos"]["visionclaw"]["counts"];
    let ratio_of = |d: &str| -> Option<f64> {
        if d == "flow" { vc["flow_e0c_scope"]["r_flow"].as_f64() } else { vc["detectors"][d]["r"].as_f64() }
    };
    let e12 = ["endpoint1", "endpoint2"].iter().any(|e| results["verdict"][*e]["result"] == "PASS");
    let mut readings = serde_json::Map::new();
    for (d, _) in DETECTORS {
        let ratio = ratio_of(d);
        let passes = if d == "hunk5" { e12 } else { ratio.is_some_and(|r| r >= R_THRESHOLD) };
        let recall = table["pooled"][d]["recall"]["value"].as_f64();
        readings.insert(d.into(), json!({"ratio_visionclaw": opt(ratio), "ratio_passes": passes, "recall": opt(recall), "reading": reading(passes, recall)}));
    }
    let recall = table["pooled"]["hunk5"]["recall"]["value"].as_f64();
    let e4 = match recall {
        None => "NOT PASSED (no stale pair; recall undefined)",
        Some(r) if r >= RECALL_FLOOR => "PASS",
        Some(_) => "FAIL",
    };
    let e0d = if e4 == "PASS" && e12 {
        "HOLDS"
    } else if e4 != "PASS" && !e12 {
        "DOES NOT HOLD (endpoint 4 and endpoints 1–2 failed)"
    } else if e4 != "PASS" {
        "DOES NOT HOLD (endpoint 4 failed)"
    } else {
        "DOES NOT HOLD (endpoints 1 and 2 failed)"
    };
    let doc = json!({
        "question": QUESTION,
        "labelled": rows.len(),
        "stale": rows.iter().filter(|r| r.stale).count(),
        "first_yes": first_yes,
        "scores": table,
        "sensitivity_anchor_only_yes_dropped": strict,
        "readings": readings,
        "endpoint4": {"recall_floor": RECALL_FLOOR, "hunk5_recall": opt(recall), "result": e4},
        "endpoint1": results["verdict"]["endpoint1"],
        "endpoint2": results["verdict"]["endpoint2"],
        "e0d": e0d,
        "pairs": rows.iter().map(|r| json!({"id": r.id, "stale": r.stale, "stale_strict": r.stale_strict, "flags": r.flags})).collect::<Vec<_>>(),
    });
    std::fs::write(out.join("endpoint4.json"), serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())? + "\n")
        .map_err(|e| e.to_string())?;
    std::fs::write(out.join("ENDPOINT4.md"), endpoint4_md(&doc)).map_err(|e| e.to_string())?;
    eprintln!("E0d endpoint 4: {e4} (hunk(5) recall {}) → E0d {e0d}", num(&opt(recall)));
    Ok(())
}

fn iv(v: &Value) -> String {
    match v["wilson95"].as_array() {
        Some(w) => format!("{} ({}–{})", num(&v["value"]), num(&w[0]), num(&w[1])),
        None => "n/a".into(),
    }
}

fn endpoint4_md(doc: &Value) -> String {
    let mut o = format!(
        "# E0d endpoint 4: staleness recall and precision on blind labels\n\nGenerated by `e0 e0d-score` from \
`labels/` and `detectors.json`. Question, verbatim: *{}* A pair is stale when both labellers say yes. {} pairs \
labelled, {} stale.\n\n",
        QUESTION,
        num(&doc["labelled"]),
        num(&doc["stale"])
    );
    o.push_str(&format!(
        "## Verdict\n\n- Endpoint 1: R_hunk {} ({}), **{}**.\n- Endpoint 2: R_hunk pooled {} ({}), **{}**.\n\
- Endpoint 4: recall of T_hunk(5) = {} against ≥ 0.90: **{}**.\n\n**E0d: {}.**\n\n",
        num(&doc["endpoint1"]["r_hunk"]),
        ci(&doc["endpoint1"]),
        num(&doc["endpoint1"]["result"]),
        num(&doc["endpoint2"]["r_hunk_pooled"]),
        ci(&doc["endpoint2"]),
        num(&doc["endpoint2"]["result"]),
        num(&doc["endpoint4"]["hunk5_recall"]),
        num(&doc["endpoint4"]["result"]),
        num(&doc["e0d"])
    ));
    o.push_str("## Every detector: ratio read with recall\n\nRatio: ΣT_file / ΣT_x on VisionClaw's 100 commits (T_flow: E0c's own scope; T_hunk(5): passes if endpoint 1 or 2 passes). Recall and precision: pooled over the labelled sample, Wilson 95% interval. A ratio ≥ 2.0 with recall < 0.90 reads as a fail.\n\n| Detector | ratio | flagged | recall | precision | reading |\n|---|---|---|---|---|---|\n");
    for (d, label) in DETECTORS {
        let s = &doc["scores"]["pooled"][d];
        let r = &doc["readings"][d];
        o.push_str(&format!(
            "| {label} | {} | {}/{} | {} | {} | **{}** |\n",
            num(&r["ratio_visionclaw"]),
            num(&s["flagged"]),
            num(&s["n"]),
            iv(&s["recall"]),
            iv(&s["precision"]),
            num(&r["reading"])
        ));
    }
    for (title, key) in [
        ("By stratum", "scores"),
        (
            "Sensitivity: yes resting only on moved anchors (both labellers) not counted as stale",
            "sensitivity_anchor_only_yes_dropped",
        ),
    ] {
        o.push_str(&format!("\n## {title}\n\n| Detector | stratum | n | stale | flagged | recall | precision |\n|---|---|---|---|---|---|---|\n"));
        for st in ["visionclaw", "agentbox", "pooled"] {
            for (d, label) in DETECTORS {
                let s = &doc[key][st][d];
                o.push_str(&format!(
                    "| {label} | {st} | {} | {} | {} | {} | {} |\n",
                    num(&s["n"]),
                    num(&s["stale"]),
                    num(&s["flagged"]),
                    iv(&s["recall"]),
                    iv(&s["precision"])
                ));
            }
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topics;

    fn lab(stale: bool, flags: &[(&str, bool)]) -> Labelled {
        Labelled {
            id: "x".into(),
            repo: "visionclaw".into(),
            stale,
            stale_strict: stale,
            flags: flags.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    #[test]
    fn recall_and_precision_count_the_right_cells() {
        let rows = [
            lab(true, &[("hunk5", true), ("file", true)]),
            lab(true, &[("hunk5", false), ("file", true)]),
            lab(false, &[("hunk5", true), ("file", true)]),
            lab(false, &[("hunk5", false), ("file", true)]),
        ];
        let refs: Vec<&Labelled> = rows.iter().collect();
        let s = score_detector(&refs, "hunk5", |r| r.stale);
        assert_eq!(
            (s["stale"].as_u64(), s["flagged"].as_u64(), s["stale_flagged"].as_u64()),
            (Some(2), Some(2), Some(1))
        );
        assert_eq!(s["recall"]["value"].as_f64(), Some(0.5));
        assert_eq!(s["precision"]["value"].as_f64(), Some(0.5));
        let f = score_detector(&refs, "file", |r| r.stale);
        assert_eq!(f["recall"]["value"].as_f64(), Some(1.0));
        assert_eq!(f["precision"]["value"].as_f64(), Some(0.5));
        // No stale pair: recall undefined.
        let none = score_detector(&refs[2..], "hunk5", |r| r.stale);
        assert!(none["recall"]["value"].is_null());
    }

    #[test]
    fn a_high_ratio_with_low_recall_reads_as_a_fail() {
        assert_eq!(reading(true, Some(0.95)), "PASS");
        assert_eq!(reading(true, Some(0.5)), "FAIL (ratio without recall)");
        assert_eq!(reading(false, Some(0.95)), "FAIL (ratio)");
        assert_eq!(reading(false, Some(0.2)), "FAIL (ratio and recall)");
        assert_eq!(reading(true, None), "UNDEFINED (no stale pair)");
    }

    #[test]
    fn ranges_are_read_whole_and_uncited_sources_kept_apart() {
        let text = "---\nid: X-01\narea: visionclaw\nsources:\n  - ../project/a.rs\n  - ../project/b.md\n  - ../project/c.ts\nverified_commit: abc1234\n---\n## X-01.1 T\n```mermaid\nsequenceDiagram\n  A->>B: go a.rs:3-9\n  B->>A: back b.md:4\n  A->>A: bad a.rs:12-10\n```\n";
        let t = topics::parse_topic("docs/diagrams/visionclaw/01-x.md", text).unwrap();
        let cites = t
            .citations
            .iter()
            .map(|c| {
                let rel = topics::repo_of(match &c.resolved {
                    topics::Resolved::Source(s) => s,
                    _ => unreachable!(),
                })
                .1;
                (c.clone(), rel, crate::mapping::Mapped::Fallback(crate::mapping::Fallback::NotRust))
            })
            .collect();
        let sources: BTreeSet<String> = ["a.rs", "b.md", "c.ts"].map(String::from).into();
        let tr = crate::mapping::tracking(&sources, &[]);
        let tir = crate::TopicInRepo {
            topic: 7,
            tracking: crate::mapping::Tracking { uncited_files: ["c.ts".to_string()].into(), ..tr.clone() },
            tracking_mermaid: tr.clone(),
            tracking_rust: tr,
            kinds: Default::default(),
            stamp: Some("abc1234".into()),
            cites,
            regions: Default::default(),
            regions_rust: Default::default(),
            cite_regions: Vec::new(),
        };
        let h = htopic(&tir);
        assert_eq!(h.cites["a.rs"], [(3, 9), (12, 12)]);
        assert_eq!(h.cites["b.md"], [(4, 4)]);
        assert_eq!(h.uncited, ["c.ts".to_string()].into());
    }

    #[test]
    fn prompts_carry_the_question_and_no_detector_vocabulary() {
        let t = topics::parse_topic(
            "docs/diagrams/agentbox/01-x.md",
            "---\nid: X-01\narea: agentbox\nsources:\n  - ../project/agentbox/a.rs\nverified_commit: abc1234\n---\n## X-01.1 T\n```mermaid\nsequenceDiagram\n  A->>B: go a.rs:3\n```\n",
        )
        .unwrap();
        let p = prompt("ab-01", "agentbox", &t, "diff --git a/a.rs b/a.rs\n");
        assert!(p.contains(QUESTION) && p.contains("diff --git"));
        for word in ["T_", "hunk", "flag", "detector", "region", "symbol", "flow", "E0"] {
            assert!(!p.contains(word), "prompt mentions `{word}`");
        }
    }
}
