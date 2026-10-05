use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

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
    pub schema: u32,
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
        Self { schema: crate::MODEL_SCHEMA_VERSION, name: name.into(), ..Self::default() }
    }

    /// Add or replace a file.
    pub fn add_file(&mut self, file: SourceFile) {
        self.files.insert(file.path.clone(), file);
    }

    /// Add a symbol. If the id already exists the two are merged: the first
    /// definition wins for scalar fields, list fields are concatenated. This
    /// handles e.g. `#[cfg]`-gated duplicate definitions deterministically.
    pub fn add_symbol(&mut self, symbol: Symbol) {
        match self.symbols.get_mut(&symbol.id) {
            Some(existing) => {
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
