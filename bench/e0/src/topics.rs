//! Topic files and their citations, read with the rules of VisionFlow's
//! `scripts/diagram-index-gen.cjs` at the pinned corpus revision: its minimal
//! front-matter reader, its mermaid-block walk, its citation regexes, the bare
//! `:N` binding (same line, then the message's participant, then the last path
//! in the diagram), the `:a,b,c` expansion, ranges (start line only), the
//! host:port exemption, exact-before-suffix resolution against `sources:`, the
//! repository prefixes and the `verified_commit` map.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use fancy_regex::Regex;

/// A front-matter value: a scalar or a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Str(String),
    List(Vec<String>),
}

/// One topic file.
#[derive(Debug, Clone)]
pub struct Topic {
    /// Path inside the corpus repository, e.g. `docs/diagrams/visionclaw/03-x.md`.
    pub file: String,
    pub id: String,
    pub area: String,
    /// `sources:` entries, `:suffix` stripped as the generator does.
    pub sources: Vec<String>,
    /// Raw `verified_commit`.
    pub verified_commit: String,
    /// Every citation: each mermaid block's, then each section's prose, in document order.
    pub citations: Vec<Citation>,
    /// Bare `:N` references the generator cannot bind (no path before it, or an unbound participant).
    pub unbound_bare: usize,
    /// The whole file text.
    pub text: String,
}

/// How a citation's path resolved against the topic's `sources:`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Resolved {
    /// Exactly one `sources:` entry.
    Source(String),
    /// No `sources:` entry matches.
    NotInSources,
    /// More than one entry matches.
    Ambiguous,
}

/// One `path:line` citation (a range contributes its start line).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    /// Diagram id, or `<H2 id>.prose` for a section's prose.
    pub diagram: String,
    /// `true` for a citation in prose outside the mermaid blocks.
    pub prose: bool,
    /// The block's diagram kind (see [`kind_of`]), or `prose`.
    pub kind: String,
    pub cited: String,
    pub line: u32,
    pub resolved: Resolved,
}

const CITE: &str = r"([A-Za-z0-9_./-]*[A-Za-z0-9_-]\.[A-Za-z0-9]{1,12}):([0-9]+)(?:\s*-\s*([0-9]+))?";

static CITE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(CITE).unwrap());
static BARE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(^|[^A-Za-z0-9_./:-]):([0-9]+)(?:\s*-\s*([0-9]+))?(?![0-9.])").unwrap());
static COMMA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r":([0-9]+)((?:,\s*[0-9]+)+)").unwrap());
static COMMA_ITEM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r",\s*([0-9]+)").unwrap());
static PART_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]*(?:participant|actor)\s+([A-Za-z0-9_]+)(?:\s+as\s+(.+))?$").unwrap());
static MSG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[ \t]*([A-Za-z0-9_]+)\s*(?:-->>|->>|-->|->|--x|-x|--\)|-\))\s*[+-]?\s*([A-Za-z0-9_]+)\s*:").unwrap()
});
static NOTE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[ \t]*Note\s+(?:over|left of|right of)\s+([A-Za-z0-9_]+)(?:\s*,\s*([A-Za-z0-9_]+))?\s*:").unwrap()
});
static HOST_PORT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+(\.[0-9]+)+$").unwrap());
static KEY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*):\s*(.*)$").unwrap());
static ITEM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*-\s+").unwrap());
static FENCE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^```([A-Za-z0-9_]*)").unwrap());
static H2_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^##\s+(\S+)\s*(.*)$").unwrap());

fn parse_scalar(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\''))) {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

fn parse_inline(s: &str) -> Value {
    let s = s.trim();
    if s.starts_with('[') && s.ends_with(']') {
        let inner = s[1..s.len() - 1].trim();
        if inner.is_empty() {
            return Value::List(Vec::new());
        }
        return Value::List(inner.split(',').map(parse_scalar).filter(|x| !x.is_empty()).collect());
    }
    Value::Str(parse_scalar(s))
}

/// The generator's front-matter reader. Returns the fields and the body.
pub fn parse_frontmatter(text: &str) -> Option<(BTreeMap<String, Value>, &str)> {
    if !text.starts_with("---\n") {
        return None;
    }
    let end = 4 + text[4..].find("\n---")?;
    let mut fm = BTreeMap::new();
    let mut key: Option<String> = None;
    for raw in text[4..end].split('\n') {
        if raw.trim().is_empty() || raw.trim().starts_with('#') {
            continue;
        }
        if let Ok(Some(m)) = KEY_RE.captures(raw) {
            let k = m[1].to_string();
            let v = if m[2].trim().is_empty() { Value::List(Vec::new()) } else { parse_inline(&m[2]) };
            fm.insert(k.clone(), v);
            key = Some(k);
        } else if ITEM_RE.is_match(raw).unwrap_or(false) {
            if let Some(k) = &key {
                let item = parse_scalar(&ITEM_RE.replace(raw, ""));
                let slot = fm.entry(k.clone()).or_insert_with(|| Value::List(Vec::new()));
                if let Value::Str(_) = slot {
                    *slot = Value::List(Vec::new());
                }
                if let Value::List(l) = slot {
                    l.push(item);
                }
            }
        }
    }
    Some((fm, &text[end + 4..]))
}

/// Mermaid blocks under an H2, as `(diagram id, source)`, and the prose of
/// each section (every line outside a fence that is not a heading), as
/// `(<H2 id>.prose, text)`; prose before the first H2 is `_.prose`.
pub type Blocks = (Vec<(String, String)>, Vec<(String, String)>);

/// Split a topic body into mermaid blocks and section prose.
pub fn mermaid_blocks(body: &str) -> Blocks {
    let mut out = Vec::new();
    let mut h2: Option<String> = None;
    let mut in_fence = false;
    let mut lang = String::new();
    let mut buf: Vec<&str> = Vec::new();
    let mut prose: Vec<(String, Vec<&str>)> = Vec::new();
    for ln in body.split('\n') {
        if !in_fence {
            if let Ok(Some(f)) = FENCE_RE.captures(ln) {
                in_fence = true;
                lang = f[1].to_string();
                buf.clear();
                continue;
            }
            if let Ok(Some(h)) = H2_RE.captures(ln) {
                h2 = Some(h[1].to_string());
                continue;
            }
            if ln.starts_with('#') {
                continue;
            }
            let section = format!("{}.prose", h2.as_deref().unwrap_or("_"));
            match prose.last_mut() {
                Some((id, lines)) if *id == section => lines.push(ln),
                _ => prose.push((section, vec![ln])),
            }
        } else {
            if ln.starts_with("```") {
                in_fence = false;
                if lang == "mermaid" {
                    if let Some(id) = &h2 {
                        out.push((id.clone(), buf.join("\n")));
                    }
                }
                continue;
            }
            buf.push(ln);
        }
    }
    (out, prose.into_iter().map(|(id, l)| (id, l.join("\n"))).collect())
}

fn resolve(sources: &[String], cited: &str) -> Resolved {
    let dotted = format!("./{cited}");
    let exact: Vec<&String> = sources.iter().filter(|s| *s == cited || **s == dotted).collect();
    let suffix = format!("/{cited}");
    let hits: Vec<&String> =
        if exact.is_empty() { sources.iter().filter(|s| s.ends_with(&suffix)).collect() } else { exact };
    match hits.as_slice() {
        [] => Resolved::NotInSources,
        [one] => Resolved::Source((*one).clone()),
        _ => Resolved::Ambiguous,
    }
}

/// Every citation in one diagram's source, with the generator's binding rules.
/// Returns the citations and the number of bare refs it could not bind.
pub fn diagram_citations(diagram: &str, src: &str, sources: &[String], prose: bool) -> (Vec<Citation>, usize) {
    let mut cites = Vec::new();
    let mut unbound = 0;
    let mut check = |cited: &str, a: &str| {
        if HOST_PORT_RE.is_match(cited).unwrap_or(false) {
            return;
        }
        let line = a.parse::<u32>().unwrap_or(0);
        cites.push(Citation {
            diagram: diagram.into(),
            prose,
            kind: String::new(),
            cited: cited.into(),
            line,
            resolved: resolve(sources, cited),
        });
    };
    let text = src.replace("\\n", "\n");
    let mut part_file: BTreeMap<String, Option<String>> = BTreeMap::new();
    for pm in PART_RE.captures_iter(&text).flatten() {
        let label = pm.get(2).map_or("", |m| m.as_str());
        let file = CITE_RE.captures(label).ok().flatten().map(|c| c[1].to_string());
        part_file.insert(pm[1].to_string(), file);
    }
    enum Ctx {
        File(String),
        Unbound,
        None,
    }
    let line_context = |line: &str| -> Ctx {
        let ids: Vec<String> = if let Ok(Some(m)) = MSG_RE.captures(line) {
            vec![m[1].to_string(), m[2].to_string()]
        } else if let Ok(Some(n)) = NOTE_RE.captures(line) {
            std::iter::once(n[1].to_string()).chain(n.get(2).map(|x| x.as_str().to_string())).collect()
        } else {
            Vec::new()
        };
        if ids.is_empty() {
            return Ctx::None;
        }
        for id in &ids {
            if let Some(Some(f)) = part_file.get(id) {
                return Ctx::File(f.clone());
            }
        }
        if ids.iter().any(|id| part_file.contains_key(id)) { Ctx::Unbound } else { Ctx::None }
    };
    let mut last_path: Option<String> = None;
    for raw in text.split('\n') {
        let line = COMMA_RE
            .replace_all(raw, |c: &fancy_regex::Captures| {
                format!(":{}{}", &c[1], COMMA_ITEM_RE.replace_all(&c[2], " :$1"))
            })
            .into_owned();
        let mut paths: Vec<(String, usize)> = Vec::new();
        let mut stripped = String::with_capacity(line.len());
        let mut at = 0;
        for c in CITE_RE.captures_iter(&line).flatten() {
            let m = c.get(0).unwrap();
            stripped.push_str(&line[at..m.start()]);
            stripped.push_str(&" ".repeat(m.as_str().len()));
            at = m.end();
            let cited = c[1].to_string();
            paths.push((cited.clone(), m.start()));
            last_path = Some(cited.clone());
            check(&cited, &c[2]);
        }
        stripped.push_str(&line[at..]);
        for bm in BARE_RE.captures_iter(&stripped).flatten() {
            let off = bm.get(0).unwrap().start() + bm[1].len();
            let before: Vec<&(String, usize)> = paths.iter().filter(|(_, o)| *o < off).collect();
            let lc = if before.is_empty() { line_context(&line) } else { Ctx::None };
            let ctx = match (before.last(), lc) {
                (_, Ctx::Unbound) => {
                    unbound += 1;
                    continue;
                }
                (Some((p, _)), _) => Some(p.clone()),
                (None, Ctx::File(f)) => Some(f),
                (None, Ctx::None) => last_path.clone(),
            };
            match ctx {
                Some(p) => check(&p, &bm[2]),
                None => unbound += 1,
            }
        }
    }
    (cites, unbound)
}

/// A mermaid block's kind: the generator's rule (first token of the block's
/// first line), with `graph` folded into `flowchart` and `stateDiagram` into
/// `stateDiagram-v2`.
pub fn kind_of(src: &str) -> String {
    let first = src.split('\n').next().unwrap_or("").split_whitespace().next().unwrap_or("");
    match first {
        "graph" | "flowchart" => "flowchart".into(),
        "stateDiagram" | "stateDiagram-v2" => "stateDiagram-v2".into(),
        "" => "unknown".into(),
        k => k.into(),
    }
}

/// Parse one topic file.
pub fn parse_topic(file: &str, text: &str) -> Result<Topic, String> {
    let (fm, body) = parse_frontmatter(text).ok_or_else(|| format!("{file}: no front matter"))?;
    let scalar = |k: &str| match fm.get(k) {
        Some(Value::Str(s)) => Ok(s.clone()),
        _ => Err(format!("{file}: front matter `{k}` missing or not a scalar")),
    };
    let sources: Vec<String> = match fm.get("sources") {
        Some(Value::List(l)) => l.clone(),
        Some(Value::Str(s)) => vec![s.clone()],
        None => Vec::new(),
    }
    .into_iter()
    .map(|s| s.split(':').next().unwrap_or_default().to_string())
    .collect();
    let (blocks, prose) = mermaid_blocks(body);
    let mut citations = Vec::new();
    let mut unbound_bare = 0;
    for (id, src, is_prose) in blocks.iter().map(|(i, s)| (i, s, false)).chain(prose.iter().map(|(i, s)| (i, s, true)))
    {
        let (mut c, u) = diagram_citations(id, src, &sources, is_prose);
        let kind = if is_prose { "prose".to_string() } else { kind_of(src) };
        for x in &mut c {
            x.kind = kind.clone();
        }
        citations.extend(c);
        unbound_bare += u;
    }
    Ok(Topic {
        file: file.into(),
        id: scalar("id")?,
        area: scalar("area")?,
        sources,
        verified_commit: scalar("verified_commit")?,
        citations,
        unbound_bare,
        text: text.into(),
    })
}

/// The repositories this experiment reads, by the generator's prefix table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoKey {
    Visionclaw,
    Agentbox,
}

impl RepoKey {
    pub const ALL: [RepoKey; 2] = [RepoKey::Visionclaw, RepoKey::Agentbox];

    pub fn as_str(self) -> &'static str {
        match self {
            RepoKey::Visionclaw => "visionclaw",
            RepoKey::Agentbox => "agentbox",
        }
    }
}

/// The generator's `REPO_PREFIXES`, longest first where they nest.
const REPO_PREFIXES: &[(&str, &str)] = &[
    ("agentbox", "../project/agentbox/"),
    ("unmute", "../project/voice-stack/unmute/"),
    ("visionclaw", "../project/"),
    ("solid-pod-rs", "../solid-pod-rs/"),
    ("nostr-rust-forum", "../nostr-rust-forum/"),
    ("dreamlab-ai-website", "../dreamlab-ai-website/"),
    ("vowl-wasm", "../vowl-wasm/"),
    ("knowledgegraph", "../knowledgeGraph/"),
    ("visiongraph", "../visionGraph/"),
    ("ruview", "../RuView/"),
    ("wasmvowl", "../WasmVOWL/"),
    ("loom", "../loom/"),
    ("sidestr-rs", "../sidestr-rs/"),
    ("dream-machine", "../dream-machine/"),
    ("prose-sanitiser", "../prose-sanitiser/"),
    ("diagram-ir", "../diagram-ir/"),
];

/// The repository that owns a `sources:` path, and the path inside it.
/// `None` for the corpus repository itself and anything unmapped.
pub fn repo_of(source: &str) -> (Option<&'static str>, String) {
    let clean = source.strip_prefix("./").unwrap_or(source);
    for (key, prefix) in REPO_PREFIXES {
        if let Some(rel) = clean.strip_prefix(prefix) {
            return (Some(key), rel.to_string());
        }
    }
    if !clean.starts_with("../") {
        return (Some("visionflow"), clean.to_string());
    }
    (None, clean.to_string())
}

/// The generator's `shaFor`: a string stamp applies to the area's own repository
/// only; a `{repo: sha}` map applies by key, case-insensitively.
pub fn sha_for(verified_commit: &str, area: &str, repo: &str) -> Option<String> {
    let v = verified_commit.trim();
    if let Some(rest) = v.strip_prefix('{') {
        // The generator drops the last character whatever it is (`slice(1, -1)`).
        let inner = rest.char_indices().last().map_or("", |(i, _)| &rest[..i]);
        for pair in inner.split(',') {
            let mut it = pair.split(':').map(str::trim);
            if let (Some(k), Some(sha)) = (it.next(), it.next()) {
                if !k.is_empty() && !sha.is_empty() && k.to_lowercase() == repo {
                    return Some(sha.to_string());
                }
            }
        }
        return None;
    }
    let sha_ok = (7..=40).contains(&v.len()) && v.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase());
    (area == repo && sha_ok).then(|| v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srcs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn participant_binding_ranges_and_comma_lists() {
        let sources = srcs(&["../project/src/main.rs", "../project/src/a/timeout.rs"]);
        let src = "sequenceDiagram\n    participant AC as server<br/>src/main.rs:938-1218\n    participant H as handler\n    AC->>H: go (:1020)\n    Note over AC: get_timeout (timeout.rs:37-41) and :50\n    H->>AC: x :7,9";
        let (c, unbound) = diagram_citations("VC-03.1", src, &sources, false);
        let got: Vec<(String, u32)> = c.iter().map(|c| (c.cited.clone(), c.line)).collect();
        assert_eq!(
            got,
            [
                ("src/main.rs".into(), 938),
                ("src/main.rs".into(), 1020),
                ("timeout.rs".into(), 37),
                ("timeout.rs".into(), 50),
                ("src/main.rs".into(), 7),
                ("src/main.rs".into(), 9),
            ]
        );
        assert_eq!(unbound, 0);
        assert_eq!(c[2].resolved, Resolved::Source("../project/src/a/timeout.rs".into()));
    }

    #[test]
    fn unbound_participant_and_host_port() {
        let sources = srcs(&["../project/src/main.rs"]);
        let src = "sequenceDiagram\n    participant A as plain\n    participant B as plain\n    A->>B: call :12\n    Note over A: 127.0.0.1:8080";
        let (c, unbound) = diagram_citations("X-01.1", src, &sources, false);
        assert!(c.is_empty(), "{c:?}");
        assert_eq!(unbound, 1);
    }

    #[test]
    fn exact_wins_over_suffix_and_ambiguity_is_kept() {
        let sources = srcs(&["README.md", "docs/README.md", "a/x.rs", "b/x.rs"]);
        assert_eq!(resolve(&sources, "README.md"), Resolved::Source("README.md".into()));
        assert_eq!(resolve(&sources, "x.rs"), Resolved::Ambiguous);
        assert_eq!(resolve(&sources, "y.rs"), Resolved::NotInSources);
    }

    #[test]
    fn repos_and_stamps() {
        assert_eq!(repo_of("../project/agentbox/flake.nix"), (Some("agentbox"), "flake.nix".into()));
        assert_eq!(repo_of("../project/src/main.rs"), (Some("visionclaw"), "src/main.rs".into()));
        assert_eq!(repo_of("../project/voice-stack/unmute/x.py").0, Some("unmute"));
        assert_eq!(sha_for("abc1234", "visionclaw", "visionclaw"), Some("abc1234".into()));
        assert_eq!(sha_for("abc1234", "visionclaw", "agentbox"), None);
        assert_eq!(
            sha_for("{agentbox: 5ab197a, visionClaw: 7d3ea2e}", "agentbox", "visionclaw"),
            Some("7d3ea2e".into())
        );
    }

    #[test]
    fn frontmatter_lists_and_maps() {
        let text = "---\nid: VC-01\narea: visionclaw\nsources:\n  - ../project/src/main.rs\nadrs: [A, B]\nverified_commit: {visionclaw: abcdef0}\n---\n## VC-01.1 t\n```mermaid\nflowchart LR\n  A[src/main.rs:3]\n```\nsee src/main.rs:9\n";
        let t = parse_topic("x.md", text).unwrap();
        assert_eq!(t.sources, ["../project/src/main.rs"]);
        assert_eq!(t.verified_commit, "{visionclaw: abcdef0}");
        assert_eq!(t.citations.len(), 2);
        assert!(!t.citations[0].prose && t.citations[1].prose);
        assert_eq!((t.citations[0].kind.as_str(), t.citations[1].kind.as_str()), ("flowchart", "prose"));
        assert_eq!(kind_of("graph TD\n  A"), "flowchart");
        assert_eq!(kind_of("stateDiagram\n"), "stateDiagram-v2");
        assert_eq!((t.citations[1].diagram.as_str(), t.citations[1].line), ("VC-01.1.prose", 9));
    }
}
