use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::flow::{Arm, Call, Flow, Step};
use crate::path::SourcePath;
use crate::source::SourceFile;
use crate::sym::SymbolId;
use crate::symbol::{Relation, RelationKind, Symbol, SymbolKind};

/// The whole analysed codebase: files, symbols and relations.
///
/// All collections are ordered, so serialising a `Codebase` (for example with
/// `serde_json`) is deterministic. Query methods return iterators in key
/// order.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Codebase {
    /// Schema version, see [`MODEL_SCHEMA_VERSION`](crate::MODEL_SCHEMA_VERSION).
    pub schema_version: u32,
    /// Human name of the codebase (workspace or package name).
    pub name: String,
    /// Files keyed by path.
    pub files: BTreeMap<SourcePath, SourceFile>,
    /// Symbols keyed by id.
    pub symbols: BTreeMap<SymbolId, Symbol>,
    /// All relations, ordered.
    pub relations: BTreeSet<Relation>,
}

impl Codebase {
    /// An empty codebase called `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self { schema_version: crate::MODEL_SCHEMA_VERSION, name: name.into(), ..Self::default() }
    }

    /// Read a model serialised as JSON, refusing any schema version other
    /// than [`MODEL_SCHEMA_VERSION`](crate::MODEL_SCHEMA_VERSION).
    ///
    /// ```
    /// use sealmap_model::{Codebase, ModelJsonError};
    ///
    /// let json = serde_json::to_string(&Codebase::new("demo")).unwrap();
    /// assert_eq!(Codebase::from_json(&json).unwrap().name, "demo");
    ///
    /// let v1 = r#"{"schema":1,"name":"old","files":{},"symbols":{},"relations":[]}"#;
    /// assert!(matches!(Codebase::from_json(v1), Err(ModelJsonError::Version { found: Some(1), .. })));
    /// ```
    pub fn from_json(text: &str) -> Result<Self, ModelJsonError> {
        #[derive(Deserialize)]
        struct Probe {
            schema_version: Option<u32>,
            schema: Option<u32>,
        }
        let probe: Probe = serde_json::from_str(text).map_err(ModelJsonError::Json)?;
        let found = probe.schema_version.or(probe.schema);
        if found != Some(crate::MODEL_SCHEMA_VERSION) {
            return Err(ModelJsonError::Version { found, expected: crate::MODEL_SCHEMA_VERSION });
        }
        serde_json::from_str(text).map_err(ModelJsonError::Json)
    }

    /// Add or replace a file.
    pub fn add_file(&mut self, file: SourceFile) {
        self.files.insert(file.path.clone(), file);
    }

    /// Add a symbol. If the id already exists the two are merged: the first
    /// definition wins for scalar fields, list fields are concatenated, and
    /// the fingerprints are folded with [`Fingerprint::merge`](crate::Fingerprint::merge)
    /// so an edit to either definition shows. This handles e.g.
    /// `#[cfg]`-gated duplicate definitions deterministically.
    ///
    /// Flows are kept, not dropped: twins whose flows differ become the arms
    /// of one [`Step::Branch`], one arm per definition, each labelled
    /// `cfg twin at <file>:<line>` (an arm is empty when its definition makes
    /// no calls). Flows that differ only in line numbers stay one flat flow.
    ///
    /// ```
    /// use sealmap_model::*;
    ///
    /// let id = |s: &str| SymbolId::parse(s).unwrap();
    /// let file = SourcePath::new("src/lib.rs").unwrap();
    /// let twin = |line, callee: &str| {
    ///     let mut s = Symbol::new(id("sym:cargo app . plat()."), "plat", SymbolKind::Function, file.clone());
    ///     s.span.start_line = line;
    ///     s.flow = Some(Flow::new(vec![Step::Call(Call::new(id(callee), "f", Confidence::Exact))]));
    ///     s
    /// };
    /// let mut cb = Codebase::new("app");
    /// cb.add_symbol(twin(2, "sym:cargo app . unix_impl()."));
    /// cb.add_symbol(twin(5, "sym:cargo app . windows_impl()."));
    /// let flow = cb.symbol(&id("sym:cargo app . plat().")).unwrap().flow.as_ref().unwrap();
    /// let names: Vec<_> = flow.calls().map(|c| c.target.name()).collect();
    /// assert_eq!(names, ["unix_impl", "windows_impl"]);
    /// let Step::Branch { arms } = &flow.steps[0] else { unreachable!() };
    /// assert_eq!(arms[1].label, "cfg twin at src/lib.rs:5");
    /// ```
    pub fn add_symbol(&mut self, symbol: Symbol) {
        match self.symbols.get_mut(&symbol.id) {
            Some(existing) => {
                existing.sig_hash = existing.sig_hash.merge(symbol.sig_hash);
                existing.body_hash = existing.body_hash.merge(symbol.body_hash);
                merge_twin_flow(existing, symbol.flow, &symbol.file, symbol.span.start_line);
                existing.members.extend(symbol.members);
                for tag in symbol.tags {
                    if !existing.tags.contains(&tag) {
                        existing.tags.push(tag);
                    }
                }
            }
            None => {
                self.symbols.insert(symbol.id.clone(), symbol);
            }
        }
    }

    /// Add a relation (duplicates are ignored). Self-loops are dropped.
    pub fn add_relation(&mut self, relation: Relation) {
        if relation.from != relation.to {
            self.relations.insert(relation);
        }
    }

    /// Look up a file.
    pub fn file(&self, path: &SourcePath) -> Option<&SourceFile> {
        self.files.get(path)
    }

    /// Look up a symbol.
    pub fn symbol(&self, id: &SymbolId) -> Option<&Symbol> {
        self.symbols.get(id)
    }

    /// `true` if `id` is defined inside the codebase.
    pub fn is_internal(&self, id: &SymbolId) -> bool {
        self.symbols.contains_key(id)
    }

    /// Symbols defined in `path`, in id order.
    pub fn symbols_in_file<'a>(&'a self, path: &'a SourcePath) -> impl Iterator<Item = &'a Symbol> {
        self.symbols.values().filter(move |s| &s.file == path)
    }

    /// Symbols of `kind`, in id order.
    pub fn symbols_of_kind(&self, kind: SymbolKind) -> impl Iterator<Item = &Symbol> {
        self.symbols.values().filter(move |s| s.kind == kind)
    }

    /// Direct children of `id` (methods of a type, items of a module).
    pub fn children<'a>(&'a self, id: &'a SymbolId) -> impl Iterator<Item = &'a Symbol> {
        self.symbols.values().filter(move |s| s.parent.as_ref() == Some(id))
    }

    /// Relations whose source is `id`.
    ///
    /// `Relation` orders by `from` first, so this is a range scan
    /// (O(log n + k)), not a pass over every relation.
    pub fn relations_from<'a>(&'a self, id: &'a SymbolId) -> impl Iterator<Item = &'a Relation> {
        let lower = Relation::new(id.clone(), SymbolId::min_value(), RelationKind::Contains, crate::Confidence::Exact);
        self.relations.range(lower..).take_while(move |r| &r.from == id)
    }

    /// Relations whose target is `id`.
    pub fn relations_to<'a>(&'a self, id: &'a SymbolId) -> impl Iterator<Item = &'a Relation> {
        self.relations.iter().filter(move |r| &r.to == id)
    }

    /// Relations of one kind.
    pub fn relations_of_kind(&self, kind: RelationKind) -> impl Iterator<Item = &Relation> {
        self.relations.iter().filter(move |r| r.kind == kind)
    }

    /// The innermost *type* that owns `id` (the type for a method), or the
    /// module for free items. This is the "participant" a symbol belongs to
    /// in a sequence diagram.
    ///
    /// For external ids the parent path is returned as-is.
    pub fn owner_of(&self, id: &SymbolId) -> Option<SymbolId> {
        match self.symbols.get(id) {
            Some(sym) if sym.kind == SymbolKind::Method || sym.kind == SymbolKind::Function => sym.parent.clone(),
            Some(sym) => Some(sym.id.clone()),
            None => id.parent(),
        }
    }

    /// Summary counts.
    pub fn stats(&self) -> CodebaseStats {
        let mut stats = CodebaseStats {
            files: self.files.len(),
            symbols: self.symbols.len(),
            relations: self.relations.len(),
            ..CodebaseStats::default()
        };
        for sym in self.symbols.values() {
            if let Some(flow) = &sym.flow {
                stats.flows += 1;
                stats.calls += flow.call_count();
            }
        }
        stats
    }
}

/// Why [`Codebase::from_json`] refused its input.
#[derive(Debug, thiserror::Error)]
pub enum ModelJsonError {
    /// Not JSON, or not the shape of a [`Codebase`].
    #[error("invalid model JSON: {0}")]
    Json(serde_json::Error),
    /// A schema version this build does not read.
    #[error("model schema version {found:?} is not supported (expected {expected})")]
    Version {
        /// The version found (`schema_version`, or v1's `schema`), if any.
        found: Option<u32>,
        /// The version this build reads.
        expected: u32,
    },
}

/// Summary counts for a [`Codebase`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CodebaseStats {
    /// Number of files.
    pub files: usize,
    /// Number of symbols.
    pub symbols: usize,
    /// Number of relations.
    pub relations: usize,
    /// Number of callables with a non-empty flow.
    pub flows: usize,
    /// Total call sites across all flows.
    pub calls: usize,
}

/// Label prefix of the arms [`Codebase::add_symbol`] makes for `#[cfg]` twins.
const TWIN_ARM: &str = "cfg twin at ";

fn twin_arm(file: &SourcePath, line: u32, flow: Option<Flow>) -> Arm {
    Arm { label: format!("{TWIN_ARM}{file}:{line}"), steps: flow.map(|f| f.steps).unwrap_or_default() }
}

/// Fold a twin's flow into `existing`. A flow that is already the twin
/// branch of earlier merges gains one arm; identical flows stay as they are.
fn merge_twin_flow(existing: &mut Symbol, flow: Option<Flow>, file: &SourcePath, line: u32) {
    let same = match (&existing.flow, &flow) {
        (Some(a), Some(b)) => same_steps(&a.steps, &b.steps),
        (a, b) => a.is_none() && b.is_none(),
    };
    if same {
        return;
    }
    let mut arms = match existing.flow.take().map(twin_arms) {
        Some(Ok(arms)) => arms,
        Some(Err(first)) => vec![twin_arm(&existing.file, existing.span.start_line, Some(first))],
        None => vec![twin_arm(&existing.file, existing.span.start_line, None)],
    };
    arms.push(twin_arm(file, line, flow));
    existing.flow = Some(Flow::new(vec![Step::Branch { arms }]));
}

/// The arms of a flow that is already a twin branch, or the flow back.
fn twin_arms(mut flow: Flow) -> Result<Vec<Arm>, Flow> {
    let twin =
        matches!(flow.steps.as_slice(), [Step::Branch { arms }] if arms.iter().all(|a| a.label.starts_with(TWIN_ARM)));
    if twin {
        if let Some(Step::Branch { arms }) = flow.steps.pop() {
            return Ok(arms);
        }
    }
    Err(flow)
}

/// Two step lists that differ at most in their line numbers.
fn same_steps(a: &[Step], b: &[Step]) -> bool {
    let arms = |x: &[Arm], y: &[Arm]| {
        x.len() == y.len() && x.iter().zip(y).all(|(p, q)| p.label == q.label && same_steps(&p.steps, &q.steps))
    };
    a.len() == b.len()
        && a.iter().zip(b).all(|pair| match pair {
            (Step::Call(x), Step::Call(y)) => Call { line: y.line, ..x.clone() } == *y,
            (Step::Return(x), Step::Return(y)) => x.label == y.label,
            (Step::Branch { arms: x }, Step::Branch { arms: y })
            | (Step::Parallel { arms: x }, Step::Parallel { arms: y }) => arms(x, y),
            (Step::Loop { label: l, body: x }, Step::Loop { label: m, body: y })
            | (Step::Optional { label: l, body: x }, Step::Optional { label: m, body: y }) => {
                l == m && same_steps(x, y)
            }
            _ => false,
        })
}
