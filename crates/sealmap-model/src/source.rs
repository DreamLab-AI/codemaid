use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::hash::{ContentHash, normalise_newlines};
use crate::path::SourcePath;
use crate::sym::SymbolId;

/// Metadata about one analysed source file. The file's text is not stored in
/// the model; only its hash, so the model stays small and the corpus can
/// detect drift.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    /// Normalised path relative to the codebase root.
    pub path: SourcePath,
    /// Frontend language tag, e.g. `rust`.
    pub language: String,
    /// Id of the module this file defines.
    pub module: SymbolId,
    /// Hash of the file text (newline-normalised).
    pub hash: ContentHash,
    /// Number of lines.
    pub lines: u32,
}

impl SourceFile {
    /// Describe `text` as the file at `path`.
    pub fn new(path: SourcePath, language: impl Into<String>, module: SymbolId, text: &str) -> Self {
        Self {
            path,
            language: language.into(),
            module,
            hash: ContentHash::of_text(text),
            lines: text.lines().count() as u32,
        }
    }
}

/// Options for [`SourceSet::load_dir`].
#[derive(Debug, Clone)]
pub struct LoadOptions {
    /// File extensions to include, without the dot (`["rs"]`).
    pub extensions: Vec<String>,
    /// Directory names skipped wherever they appear.
    pub skip_dirs: Vec<String>,
    /// Files larger than this are skipped (they are almost always generated).
    pub max_file_bytes: u64,
    /// Skip hidden files and directories (names starting with `.`).
    pub skip_hidden: bool,
    /// Honour `.gitignore` and `.ignore` files found under the root (not
    /// above it, and never the user's global excludes, so the result still
    /// depends only on the bytes under the root). On by default: ignored
    /// trees are typically vendored registries, build output or caches.
    pub respect_ignore_files: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            extensions: vec!["rs".into()],
            skip_dirs: ["target", "node_modules", "vendor", "dist", "build", "out"].map(String::from).to_vec(),
            max_file_bytes: 2 * 1024 * 1024,
            skip_hidden: true,
            respect_ignore_files: true,
        }
    }
}

/// An in-memory set of source files keyed by [`SourcePath`].
///
/// Frontends read from a `SourceSet`, never from the file system directly.
/// That keeps them pure (same set in, same model out), makes them trivially
/// testable, and lets embedders feed sources from anywhere: a git object
/// store, an editor buffer, a network service.
///
/// Text is stored with line endings normalised to `\n`.
///
/// ```
/// use sealmap_model::SourceSet;
///
/// let mut set = SourceSet::new();
/// set.insert("src/lib.rs", "pub mod a;\r\n").unwrap();
/// set.insert("src/a.rs", "pub fn f() {}").unwrap();
///
/// // Iteration is always in path order, regardless of insertion order.
/// let paths: Vec<_> = set.iter().map(|(p, _)| p.as_str()).collect();
/// assert_eq!(paths, ["src/a.rs", "src/lib.rs"]);
/// assert_eq!(set.get_str("src/lib.rs"), Some("pub mod a;\n"));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceSet {
    files: BTreeMap<SourcePath, String>,
}

impl SourceSet {
    /// An empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a file. The path is normalised; text newlines are
    /// normalised to `\n`.
    pub fn insert(&mut self, path: impl AsRef<str>, text: impl AsRef<str>) -> Result<(), crate::PathError> {
        let path = SourcePath::new(path)?;
        self.files.insert(path, normalise_newlines(text.as_ref()).into_owned());
        Ok(())
    }

    /// Text of the file at `path`.
    pub fn get(&self, path: &SourcePath) -> Option<&str> {
        self.files.get(path).map(String::as_str)
    }

    /// Text of the file at a raw path string (normalised first).
    pub fn get_str(&self, path: &str) -> Option<&str> {
        SourcePath::new(path).ok().and_then(|p| self.get(&p))
    }

    /// Files in path order.
    pub fn iter(&self) -> impl Iterator<Item = (&SourcePath, &str)> {
        self.files.iter().map(|(p, t)| (p, t.as_str()))
    }

    /// Paths in order.
    pub fn paths(&self) -> impl Iterator<Item = &SourcePath> {
        self.files.keys()
    }

    /// Number of files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// `true` if empty.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Keep only files for which `keep` returns `true`.
    pub fn retain(&mut self, mut keep: impl FnMut(&SourcePath) -> bool) {
        self.files.retain(|p, _| keep(p));
    }

    /// Recursively load every matching file under `root`.
    ///
    /// Symlinks are not followed and files that are not valid UTF-8 are
    /// skipped, so the result depends only on the bytes under `root`.
    pub fn load_dir(root: &Path, options: &LoadOptions) -> io::Result<Self> {
        let mut set = Self::new();
        let skip_dirs = options.skip_dirs.clone();
        let walker = ignore::WalkBuilder::new(root)
            .hidden(options.skip_hidden)
            .parents(false)
            .ignore(options.respect_ignore_files)
            .git_ignore(options.respect_ignore_files)
            .git_global(false)
            .git_exclude(false)
            .require_git(false)
            .follow_links(false)
            .filter_entry(move |e| {
                !(e.file_type().is_some_and(|t| t.is_dir())
                    && e.depth() > 0
                    && skip_dirs.iter().any(|d| e.file_name() == d.as_str()))
            })
            .build();
        for entry in walker {
            let entry = entry.map_err(|e| io::Error::other(e.to_string()))?;
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let path = entry.path();
            let ext_ok =
                path.extension().and_then(|e| e.to_str()).is_some_and(|e| options.extensions.iter().any(|x| x == e));
            if !ext_ok || entry.metadata().map_err(|e| io::Error::other(e.to_string()))?.len() > options.max_file_bytes
            {
                continue;
            }
            let Ok(text) = fs::read_to_string(path) else { continue };
            let Ok(rel) = SourcePath::relative_to(path, root) else { continue };
            set.files.insert(rel, normalise_newlines(&text).into_owned());
        }
        Ok(set)
    }
}
