//! The part of a sealmap model E0 reads, extracted from a git revision with the
//! 0.2.0 defaults and memoised by the hash of the extractor's input tree.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sealmap_model::{Confidence, PARSE_ERROR_TAG, RelationKind, SymbolKind};
use sealmap_rust::{Extraction, RustOptions};

use crate::git::{Repo, TreeEntry};

/// One symbol: where it is and its two fingerprints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sym {
    pub file: String,
    pub start: u32,
    pub end: u32,
    /// Columns of the span (1-based), used by E0b to find the symbol's syntax node.
    pub start_col: u32,
    pub end_col: u32,
    pub sig: [u8; 16],
    pub body: [u8; 16],
    pub module: bool,
}

/// Symbols by id, call edges (exact or inferred, internal targets only) and
/// files that failed to parse.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Model {
    pub syms: BTreeMap<String, Sym>,
    pub calls: BTreeMap<String, BTreeSet<String>>,
    pub parse_errors: BTreeSet<String>,
    /// E0c: every symbol's flow (signature hash and ordered calls).
    pub flows: BTreeMap<String, crate::flow::FlowRec>,
}

impl Model {
    /// Project an extraction.
    pub fn from_extraction(x: &Extraction) -> Self {
        let cb = &x.codebase;
        let mut m = Model::default();
        for s in cb.symbols.values() {
            m.syms.insert(
                s.id.to_string(),
                Sym {
                    file: s.file.as_str().to_string(),
                    start: s.span.start_line,
                    end: s.span.end_line,
                    start_col: s.span.start_col,
                    end_col: s.span.end_col,
                    sig: *s.sig_hash.as_bytes(),
                    body: *s.body_hash.as_bytes(),
                    module: s.kind == SymbolKind::Module,
                },
            );
            if s.kind == SymbolKind::Module && s.tags.iter().any(|t| t == PARSE_ERROR_TAG) {
                m.parse_errors.insert(s.file.as_str().to_string());
            }
        }
        for r in cb.relations_of_kind(RelationKind::Calls) {
            if matches!(r.confidence, Confidence::Exact | Confidence::Inferred) && cb.is_internal(&r.to) {
                m.calls.entry(r.from.to_string()).or_default().insert(r.to.to_string());
            }
        }
        m.flows = crate::flow::project(cb);
        m
    }

    /// The innermost symbol in `file` whose span contains `line`: the smallest
    /// span; ties (identical spans) go to the lexicographically smallest id.
    pub fn innermost(&self, file: &str, line: u32) -> Option<(&str, &Sym)> {
        self.syms
            .iter()
            .filter(|(_, s)| s.file == file && s.start <= line && line <= s.end)
            .min_by(|(ia, a), (ib, b)| (a.end - a.start).cmp(&(b.end - b.start)).then_with(|| ia.cmp(ib)))
            .map(|(id, s)| (id.as_str(), s))
    }
}

/// The files `sealmap_rust::load_dir` can see: `.rs`, `Cargo.toml`, and the
/// ignore files that decide which of those it skips. Symlinks and gitlinks are
/// left out (the loader does not follow links).
fn extractor_inputs(tree: &[TreeEntry]) -> Vec<&TreeEntry> {
    tree.iter()
        .filter(|e| e.mode.starts_with("100"))
        .filter(|e| {
            let name = e.path.rsplit('/').next().unwrap_or(&e.path);
            name.ends_with(".rs") || name == "Cargo.toml" || name == ".gitignore" || name == ".ignore"
        })
        .collect()
}

/// Content key of the extractor's input at a revision.
pub fn tree_key(tree: &[TreeEntry]) -> String {
    let mut h = blake3::Hasher::new();
    for e in extractor_inputs(tree) {
        h.update(e.mode.as_bytes());
        h.update(b" ");
        h.update(e.oid.as_bytes());
        h.update(b" ");
        h.update(e.path.as_bytes());
        h.update(b"\n");
    }
    h.finalize().to_hex().to_string()
}

/// Revision-to-model memo for one repository.
pub struct Models {
    repo: Repo,
    name: String,
    scratch: PathBuf,
    by_key: HashMap<String, Arc<Model>>,
    key_of: HashMap<String, String>,
    /// Extractions actually run (distinct input trees).
    pub extractions: usize,
    /// Revisions served.
    pub lookups: usize,
}

impl Models {
    /// `name` is used as the extraction root's directory name, which is what
    /// `extract_dir` names the codebase after.
    pub fn new(repo: Repo, name: &str, scratch: &Path) -> Self {
        Self {
            repo,
            name: name.into(),
            scratch: scratch.to_path_buf(),
            by_key: HashMap::new(),
            key_of: HashMap::new(),
            extractions: 0,
            lookups: 0,
        }
    }

    /// The model at `rev`.
    pub fn at(&mut self, rev: &str) -> Result<Arc<Model>, String> {
        self.lookups += 1;
        let key = match self.key_of.get(rev) {
            Some(k) => k.clone(),
            None => {
                let tree = self.repo.ls_tree(rev)?;
                let key = tree_key(&tree);
                if !self.by_key.contains_key(&key) {
                    let model = self.extract(&tree, &key)?;
                    self.by_key.insert(key.clone(), Arc::new(model));
                }
                self.key_of.insert(rev.into(), key.clone());
                key
            }
        };
        Ok(self.by_key[&key].clone())
    }

    fn extract(&mut self, tree: &[TreeEntry], key: &str) -> Result<Model, String> {
        let inputs = extractor_inputs(tree);
        let oids: Vec<String> = inputs.iter().map(|e| e.oid.clone()).collect();
        let blobs = self.repo.read_blobs(&oids)?;
        let base = self.scratch.join(&key[..16]);
        let root = base.join(&self.name);
        let _ = std::fs::remove_dir_all(&base);
        for (e, bytes) in inputs.iter().zip(&blobs) {
            let p = root.join(&e.path);
            if let Some(dir) = p.parent() {
                std::fs::create_dir_all(dir).map_err(|err| format!("{}: {err}", dir.display()))?;
            }
            std::fs::write(&p, bytes).map_err(|err| format!("{}: {err}", p.display()))?;
        }
        std::fs::create_dir_all(&root).map_err(|err| format!("{}: {err}", root.display()))?;
        let (_, extraction) =
            sealmap_rust::extract_dir(&root, &RustOptions::default()).map_err(|e| format!("extract: {e}"))?;
        let _ = std::fs::remove_dir_all(&base);
        self.extractions += 1;
        Ok(Model::from_extraction(&extraction))
    }
}
