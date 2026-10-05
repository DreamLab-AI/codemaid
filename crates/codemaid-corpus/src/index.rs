//! `_index.json`: the merge map for orchestrating agents.

use std::collections::BTreeMap;

use codemaid_model::{Codebase, CodebaseStats, Confidence, ContentHash, SourcePath, Span, SymbolId};
use serde::{Deserialize, Serialize};

/// Kind of diagram fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentKind {
    /// Per-file `classDiagram`.
    Structure,
    /// Per-callable `sequenceDiagram`.
    Sequence,
}

/// One call made by a sequence fragment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallRef {
    /// Callee id.
    pub target: SymbolId,
    /// Resolution confidence.
    pub confidence: Confidence,
    /// 1-based line of the call site.
    pub line: u32,
    /// Id of the fragment that renders the callee's own sequence, if any.
    /// Following these links is how an orchestrator inlines or stitches
    /// sequences across files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expands: Option<String>,
}

/// One diagram inside a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentEntry {
    /// Fragment id: the symbol id for sequences, `structure:<file>` for
    /// structure diagrams. Unique across the corpus.
    pub id: String,
    /// Diagram kind.
    pub kind: FragmentKind,
    /// Document holding it.
    pub document: SourcePath,
    /// Symbol the fragment describes (callable or module).
    pub symbol: SymbolId,
    /// Source span of that symbol.
    pub span: Span,
    /// Mermaid ids of participants / classes, in diagram order.
    pub participants: Vec<SymbolId>,
    /// Calls, in source order (sequence fragments only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<CallRef>,
    /// Messages elided by `max_messages` (0 when complete).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub truncated: usize,
    /// Hash of the fragment's Mermaid text, for caching merged results.
    pub hash: ContentHash,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// One document (one per source file).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentEntry {
    /// Source file.
    pub source: SourcePath,
    /// Document path.
    pub document: SourcePath,
    /// Module the source defines.
    pub module: SymbolId,
    /// Hash of the source the document was generated from.
    pub source_hash: ContentHash,
    /// Hash of the document text.
    pub document_hash: ContentHash,
    /// Fragments in document order.
    pub fragments: Vec<FragmentEntry>,
}

/// The corpus index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    /// [`crate::CORPUS_SCHEMA_VERSION`].
    pub schema: u32,
    /// Generator name and version, e.g. `codemaid-corpus 0.1.0`.
    pub generator: String,
    /// Codebase name.
    pub codebase: String,
    /// Model statistics.
    pub stats: CodebaseStats,
    /// Documents in source-path order.
    pub documents: Vec<DocumentEntry>,
}

impl Index {
    pub(crate) fn new(cb: &Codebase) -> Self {
        Self {
            schema: crate::CORPUS_SCHEMA_VERSION,
            generator: format!("codemaid-corpus {}", env!("CARGO_PKG_VERSION")),
            codebase: cb.name.clone(),
            stats: cb.stats(),
            documents: Vec::new(),
        }
    }

    /// Fill in [`CallRef::expands`] once all fragments are known.
    pub(crate) fn link_expansions(&mut self) {
        let seqs: BTreeMap<String, ()> =
            self.fragments().filter(|f| f.kind == FragmentKind::Sequence).map(|f| (f.id.clone(), ())).collect();
        for doc in &mut self.documents {
            for frag in &mut doc.fragments {
                for call in &mut frag.calls {
                    if seqs.contains_key(call.target.as_str()) {
                        call.expands = Some(call.target.as_str().to_owned());
                    }
                }
            }
        }
    }

    /// All fragments in document order.
    pub fn fragments(&self) -> impl Iterator<Item = &FragmentEntry> {
        self.documents.iter().flat_map(|d| d.fragments.iter())
    }

    /// Look up a fragment by id.
    pub fn fragment(&self, id: &str) -> Option<&FragmentEntry> {
        self.fragments().find(|f| f.id == id)
    }
}
