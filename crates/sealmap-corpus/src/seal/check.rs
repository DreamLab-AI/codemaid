//! Classifying seals against a code model: `seal_check`, `verify`,
//! `resolve` and `stale`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use sealmap_model::{Codebase, Fingerprint, SourcePath, Span, Symbol, SymbolId};
use serde::Serialize;

use super::lock::{Lock, LockError, SymbolSeal, TopicSeal};
use super::topic::{citations, pointers, topic_hash, topic_id};

/// What a check found, one variant per row of the design's verify table.
/// Everything except [`Class::Holds`] fails the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    /// The id resolves and both `sig` and `body` match the seal.
    Holds,
    /// Same signature, different body: behaviour changed.
    Behaviour,
    /// The signature changed: the contract changed.
    Contract,
    /// The id is gone. When another symbol of the same kind has the sealed
    /// body, it is listed as a rename candidate.
    Absent,
    /// A file that may hold the sealed id failed to parse. The check fails
    /// closed: the symbol is never reported as holding or absent.
    Unparsable,
    /// A topic cites a `sym:` id its seal does not cover (or the topic has
    /// no seal), or the citation is not a canonical `sym:` id.
    UnsealedCitation,
    /// The lock seals something the corpus no longer has: a topic file that
    /// does not exist, or a symbol its topic no longer cites.
    Orphan,
    /// The topic's text changed since it was sealed (`topic_hash` differs).
    ProseEdited,
    /// The lock is unreadable or not canonical, or a topic's `sealed:`
    /// pointer and the lock disagree.
    LockFault,
}

impl Class {
    /// `true` only for [`Class::Holds`].
    pub fn passes(self) -> bool {
        self == Self::Holds
    }

    /// The kebab-case name used in human output.
    pub fn name(self) -> &'static str {
        match self {
            Self::Holds => "holds",
            Self::Behaviour => "behaviour",
            Self::Contract => "contract",
            Self::Absent => "absent",
            Self::Unparsable => "unparsable",
            Self::UnsealedCitation => "unsealed-citation",
            Self::Orphan => "orphan",
            Self::ProseEdited => "prose-edited",
            Self::LockFault => "lock-fault",
        }
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One classified fact about a seal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Finding {
    /// The topic id the finding is about, if it concerns one.
    pub topic: Option<String>,
    /// The topic file, relative to the lock's directory, if known.
    pub file: Option<SourcePath>,
    /// The symbol (or cited text) the finding is about, if any.
    pub symbol: Option<String>,
    /// The class.
    pub class: Class,
    /// Human-readable detail; empty for a plain [`Class::Holds`].
    pub detail: String,
    /// Rename candidates for [`Class::Absent`]: symbols of the same kind
    /// whose `body_hash` equals the sealed body. Empty otherwise.
    pub candidates: Vec<SymbolId>,
}

/// The result of [`seal_check`] or [`verify`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SealReport {
    /// Every finding, sorted by topic, file, symbol and class. Includes one
    /// [`Class::Holds`] finding per sealed symbol that holds and per topic
    /// whose prose holds, so the report is also a full account of what was
    /// checked.
    pub findings: Vec<Finding>,
    /// Topics with neither a seal nor a `sym:` citation (legacy topics).
    /// Informational coverage; never a failure.
    pub unsealed_topics: Vec<SourcePath>,
}

impl SealReport {
    /// `true` when every finding holds: the gate passes.
    pub fn passes(&self) -> bool {
        self.findings.iter().all(|f| f.class.passes())
    }

    /// The findings that fail the gate.
    pub fn failures(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| !f.class.passes())
    }

    /// Number of findings of `class`.
    pub fn count(&self, class: Class) -> usize {
        self.findings.iter().filter(|f| f.class == class).count()
    }
}

/// The authored topic files a lock seals: path relative to the lock's
/// directory → text. Files without front matter carrying an `id:` (indexes,
/// READMEs) are not topics and are ignored.
pub type Topics = BTreeMap<SourcePath, String>;

/// The state of one sealed symbol in a code model.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SymbolState {
    class: Class,
    detail: String,
    candidates: Vec<SymbolId>,
}

/// Classify one sealed symbol against `codebase`.
///
/// Order matters, and is what makes the check fail closed: a symbol that is
/// itself the module of an unparsable file, or that is absent while one of
/// its ancestor modules is unparsable, is [`Class::Unparsable`] before it
/// can be called absent.
fn classify(
    codebase: &Codebase,
    unparsable: &BTreeMap<SymbolId, SourcePath>,
    id: &SymbolId,
    seal: &SymbolSeal,
) -> SymbolState {
    let state = |class, detail: String| SymbolState { class, detail, candidates: Vec::new() };
    match codebase.symbol(id) {
        Some(s) if s.is_unparsable() => state(Class::Unparsable, format!("{} does not parse", s.file)),
        Some(s) if s.sig_hash != seal.sig => {
            state(Class::Contract, format!("sig {} -> {} ({})", seal.sig, s.sig_hash, location(s)))
        }
        Some(s) if s.body_hash != seal.body => {
            state(Class::Behaviour, format!("body {} -> {} ({})", seal.body, s.body_hash, location(s)))
        }
        Some(s) => state(Class::Holds, location(s)),
        None => {
            if let Some(file) = unparsable_ancestor(unparsable, id) {
                return state(Class::Unparsable, format!("{file} may hold it and does not parse"));
            }
            let candidates = rename_candidates(codebase, id, seal.body);
            let detail = if candidates.is_empty() {
                "id not found; no symbol of the same kind has the sealed body".to_owned()
            } else {
                format!("id not found; rename suspected ({} candidate(s) with the sealed body)", candidates.len())
            };
            SymbolState { class: Class::Absent, detail, candidates }
        }
    }
}

fn location(s: &Symbol) -> String {
    format!("{}:{}-{}", s.file, s.span.start_line, s.span.end_line)
}

/// Module ids of files that failed to parse, with the file.
fn unparsable_modules(codebase: &Codebase) -> BTreeMap<SymbolId, SourcePath> {
    codebase.symbols.values().filter(|s| s.is_unparsable()).map(|s| (s.id.clone(), s.file.clone())).collect()
}

fn unparsable_ancestor<'a>(unparsable: &'a BTreeMap<SymbolId, SourcePath>, id: &SymbolId) -> Option<&'a SourcePath> {
    let mut cur = id.parent();
    while let Some(p) = cur {
        if let Some(file) = unparsable.get(&p) {
            return Some(file);
        }
        cur = p.parent();
    }
    None
}

/// Symbols of the same descriptor kind as `id` whose body hash is `body`.
/// An unset fingerprint never matches.
fn rename_candidates(codebase: &Codebase, id: &SymbolId, body: Fingerprint) -> Vec<SymbolId> {
    if body.is_unset() {
        return Vec::new();
    }
    let suffix = id.last().map(|d| d.suffix().clone());
    codebase
        .symbols
        .values()
        .filter(|s| s.body_hash == body && &s.id != id && s.id.last().map(|d| d.suffix().clone()) == suffix)
        .map(|s| s.id.clone())
        .collect()
}

/// Classify the lock entries for `only` (topic ids; `None` = every entry)
/// against `codebase` and the topic texts.
///
/// Per entry it reports:
///
/// * the topic file is missing from `topics` → [`Class::Orphan`] (nothing
///   else is checked for that entry);
/// * the file's front-matter `id:` differs from the entry's id, or its
///   `sealed:` pointer is missing, repeated or names another lock than
///   `lock_name` → [`Class::LockFault`];
/// * `topic_hash` differs → [`Class::ProseEdited`], else a holding finding;
/// * each sealed symbol → [`Class::Holds`], [`Class::Behaviour`],
///   [`Class::Contract`], [`Class::Absent`] or [`Class::Unparsable`];
///   a sealed symbol the topic no longer cites is also an [`Class::Orphan`];
/// * each cited `sym:` id the entry does not seal, or that does not parse →
///   [`Class::UnsealedCitation`].
///
/// An id in `only` with no entry in the lock is a [`Class::LockFault`].
pub fn seal_check(
    lock: &Lock,
    lock_name: &str,
    topics: &Topics,
    codebase: &Codebase,
    only: Option<&BTreeSet<String>>,
) -> SealReport {
    let unparsable = unparsable_modules(codebase);
    let mut findings = Vec::new();
    if let Some(only) = only {
        for id in only.iter().filter(|id| !lock.topics.contains_key(*id)) {
            findings.push(fault(Some(id.clone()), None, format!("no lock entry for topic `{id}`")));
        }
    }
    for entry in lock.topics.values().filter(|t| only.is_none_or(|o| o.contains(&t.id))) {
        check_entry(entry, lock_name, topics, codebase, &unparsable, &mut findings);
    }
    findings.sort();
    SealReport { findings, unsealed_topics: Vec::new() }
}

fn check_entry(
    entry: &TopicSeal,
    lock_name: &str,
    topics: &Topics,
    codebase: &Codebase,
    unparsable: &BTreeMap<SymbolId, SourcePath>,
    out: &mut Vec<Finding>,
) {
    let finding = |symbol: Option<String>, class, detail: String| Finding {
        topic: Some(entry.id.clone()),
        file: Some(entry.file.clone()),
        symbol,
        class,
        detail,
        candidates: Vec::new(),
    };
    let Some(text) = topics.get(&entry.file) else {
        out.push(finding(None, Class::Orphan, format!("sealed topic file {} does not exist", entry.file)));
        return;
    };
    match topic_id(text) {
        Some(id) if id == entry.id => {}
        Some(id) => out.push(finding(None, Class::LockFault, format!("the file's front matter says `id: {id}`"))),
        None => out.push(finding(None, Class::LockFault, "the file has no front-matter `id:`".into())),
    }
    if let Some(problem) = pointer_problem(&pointers(text), lock_name) {
        out.push(finding(None, Class::LockFault, problem));
    }
    let hash = topic_hash(text);
    if hash == entry.topic_hash {
        out.push(finding(None, Class::Holds, "prose".into()));
    } else {
        out.push(finding(None, Class::ProseEdited, format!("topic_hash {} -> {hash}", entry.topic_hash)));
    }

    let cited = citations(text);
    let cited_ids: BTreeSet<&SymbolId> = cited.iter().filter_map(|c| c.id.as_ref().ok()).collect();
    for (id, seal) in &entry.symbols {
        let s = classify(codebase, unparsable, id, seal);
        out.push(Finding { candidates: s.candidates, ..finding(Some(id.to_string()), s.class, s.detail) });
        if !cited_ids.contains(id) {
            out.push(finding(Some(id.to_string()), Class::Orphan, "sealed but no longer cited by the topic".into()));
        }
    }
    unsealed_citations(&entry.id, &entry.file, &cited, |id| entry.symbols.contains_key(id), out);
}

fn unsealed_citations(
    topic: &str,
    file: &SourcePath,
    cited: &[super::topic::Citation],
    sealed: impl Fn(&SymbolId) -> bool,
    out: &mut Vec<Finding>,
) {
    let mut seen = BTreeSet::new();
    for c in cited {
        let detail = match &c.id {
            Err(e) => format!("line {}: not a canonical sym: id ({e})", c.line),
            Ok(id) if sealed(id) => continue,
            Ok(_) => format!("line {}: cited but not sealed", c.line),
        };
        if seen.insert(c.text.clone()) {
            out.push(Finding {
                topic: Some(topic.to_owned()),
                file: Some(file.clone()),
                symbol: Some(c.text.clone()),
                class: Class::UnsealedCitation,
                detail,
                candidates: Vec::new(),
            });
        }
    }
}

fn pointer_problem(pointers: &[String], lock_name: &str) -> Option<String> {
    match pointers {
        [] => Some(format!("sealed in the lock but the topic has no `sealed: {lock_name}` line")),
        [p] if p == lock_name => None,
        [p] => Some(format!("the topic's pointer names `{p}`, not `{lock_name}`")),
        _ => Some(format!("the topic has {} `sealed:` lines", pointers.len())),
    }
}

fn fault(topic: Option<String>, file: Option<SourcePath>, detail: String) -> Finding {
    Finding { topic, file, symbol: None, class: Class::LockFault, detail, candidates: Vec::new() }
}

/// The corpus gate: [`seal_check`] over every entry, plus what only a
/// corpus-wide view can see.
///
/// `lock_text` is the lock file's content (`None` when there is no lock),
/// `lock_name` its file name (the value every pointer must carry). On top
/// of `seal_check` it reports:
///
/// * a lock that does not parse → one [`Class::LockFault`] and nothing else;
/// * a lock that is not canonical ([`Lock::parse_canonical`]) →
///   [`Class::LockFault`];
/// * a topic with a `sealed:` pointer but no entry → [`Class::LockFault`];
/// * a `sym:` citation in a topic without an entry →
///   [`Class::UnsealedCitation`].
///
/// Topics with neither a seal nor a citation are listed in
/// [`SealReport::unsealed_topics`] and do not fail the gate, so a corpus can
/// migrate one topic at a time.
///
/// ```
/// use std::collections::BTreeMap;
/// use sealmap_corpus::seal::{Class, verify};
/// use sealmap_model::{Codebase, SourcePath};
///
/// let mut topics = BTreeMap::new();
/// topics.insert(SourcePath::new("a/01-x.md").unwrap(), "---\nid: A-01\n---\nCites `sym:cargo shop . Db#`.\n".to_owned());
/// let report = verify(None, "seals.lock", &topics, &Codebase::new("shop"));
/// assert!(!report.passes());
/// assert_eq!(report.count(Class::UnsealedCitation), 1);
/// ```
pub fn verify(lock_text: Option<&str>, lock_name: &str, topics: &Topics, codebase: &Codebase) -> SealReport {
    let lock = match lock_text.map(Lock::parse) {
        None => Lock::new(),
        Some(Ok(lock)) => lock,
        Some(Err(e)) => {
            return SealReport {
                findings: vec![fault(None, None, format!("{lock_name}: {e}"))],
                unsealed_topics: Vec::new(),
            };
        }
    };
    let mut report = seal_check(&lock, lock_name, topics, codebase, None);
    if let Some(text) = lock_text {
        if lock.to_toml() != text {
            report.findings.push(fault(None, None, format!("{lock_name}: {}", LockError::NonCanonical)));
        }
    }
    for (path, text) in topics {
        let Some(id) = topic_id(text) else { continue };
        if lock.by_file(path).is_some() {
            continue;
        }
        let cited = citations(text);
        let has_pointer = !pointers(text).is_empty();
        if has_pointer {
            report.findings.push(fault(
                Some(id.clone()),
                Some(path.clone()),
                format!("the topic has a `sealed:` pointer but {lock_name} has no entry for {path}"),
            ));
        }
        unsealed_citations(&id, path, &cited, |_| false, &mut report.findings);
        if cited.is_empty() && !has_pointer {
            report.unsealed_topics.push(path.clone());
        }
    }
    report.findings.sort();
    report
}

/// Where a symbol is now, or why it cannot be found: the answer of
/// [`resolve`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Resolution {
    /// The id is in the model.
    Found {
        /// The file holding the definition.
        file: SourcePath,
        /// Its span (1-based lines, inclusive).
        span: Span,
        /// Current `sig_hash`.
        sig: Fingerprint,
        /// Current `body_hash`.
        body: Fingerprint,
    },
    /// The id is not in the model, and a file that may hold it does not
    /// parse, so absence is not proven.
    Unparsable {
        /// The unparsable file.
        file: SourcePath,
    },
    /// The id is not in the model.
    Absent {
        /// Symbols of the same kind whose body equals `sealed_body`, when one
        /// was given.
        candidates: Vec<SymbolId>,
    },
}

/// Resolve `id` in `codebase`: its current span and hashes, or why it is
/// missing. When the id is absent and `sealed_body` is known (from a lock),
/// symbols of the same kind with that body are returned as rename
/// candidates.
///
/// ```
/// use sealmap_corpus::seal::{Resolution, resolve};
/// use sealmap_model::{Codebase, SymbolId};
///
/// let id = SymbolId::parse("sym:cargo shop . Db#").unwrap();
/// assert_eq!(resolve(&Codebase::new("shop"), &id, None), Resolution::Absent { candidates: vec![] });
/// ```
pub fn resolve(codebase: &Codebase, id: &SymbolId, sealed_body: Option<Fingerprint>) -> Resolution {
    match codebase.symbol(id) {
        Some(s) if s.is_unparsable() => Resolution::Unparsable { file: s.file.clone() },
        Some(s) => Resolution::Found { file: s.file.clone(), span: s.span, sig: s.sig_hash, body: s.body_hash },
        None => match unparsable_ancestor(&unparsable_modules(codebase), id) {
            Some(file) => Resolution::Unparsable { file: file.clone() },
            None => Resolution::Absent {
                candidates: sealed_body.map(|b| rename_candidates(codebase, id, b)).unwrap_or_default(),
            },
        },
    }
}

/// A sealed symbol that changed, as reported by [`stale`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct StaleSymbol {
    /// The topic sealing it.
    pub topic: String,
    /// The topic file.
    pub file: SourcePath,
    /// The sealed id.
    pub symbol: SymbolId,
    /// [`Class::Behaviour`], [`Class::Contract`], [`Class::Absent`] or
    /// [`Class::Unparsable`].
    pub class: Class,
    /// Human-readable detail.
    pub detail: String,
    /// Rename candidates for [`Class::Absent`].
    pub candidates: Vec<SymbolId>,
}

/// Sealed symbols whose code changed, per topic, in topic and symbol order.
///
/// Each sealed symbol is classified in `after` against a baseline: its
/// hashes in `before` when given and the symbol is there (and parses),
/// otherwise the hashes in the lock. So with `before = None` this is the
/// symbol half of [`seal_check`] (what changed since sealing), and with the
/// model of an older tree it is what changed since that tree, whatever the
/// lock says. Only changes are returned; prose, pointers and citations are
/// not looked at.
pub fn stale(lock: &Lock, before: Option<&Codebase>, after: &Codebase) -> Vec<StaleSymbol> {
    let unparsable = unparsable_modules(after);
    let mut out = Vec::new();
    for entry in lock.topics.values() {
        for (id, sealed) in &entry.symbols {
            let baseline = before
                .and_then(|b| b.symbol(id))
                .filter(|s| !s.is_unparsable())
                .map_or(*sealed, |s| SymbolSeal { sig: s.sig_hash, body: s.body_hash });
            let s = classify(after, &unparsable, id, &baseline);
            if !s.class.passes() {
                out.push(StaleSymbol {
                    topic: entry.id.clone(),
                    file: entry.file.clone(),
                    symbol: id.clone(),
                    class: s.class,
                    detail: s.detail,
                    candidates: s.candidates,
                });
            }
        }
    }
    out.sort();
    out
}
