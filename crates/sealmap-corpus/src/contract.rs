//! The 1:1 contract: verify a corpus directory against freshly generated
//! output, and write a directory into compliance.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use sealmap_model::{ContentHash, SourcePath};
use serde::Serialize;

use crate::document::{front_matter_hash, is_generated};
use crate::{Corpus, is_reserved};

/// How a corpus file deviates from the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Drift {
    /// A source file has no document.
    Missing,
    /// A generated document has no source file.
    Orphaned,
    /// The source changed since the document was generated.
    Stale,
    /// The source is unchanged but the document differs (hand edit, or a new
    /// generator version / options).
    Modified,
}

impl fmt::Display for Drift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Missing => "missing",
            Self::Orphaned => "orphaned",
            Self::Stale => "stale",
            Self::Modified => "modified",
        })
    }
}

/// One deviation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DriftEntry {
    /// Corpus-relative path.
    pub path: SourcePath,
    /// What is wrong.
    pub drift: Drift,
}

/// Result of [`verify`] / [`write()`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    /// Deviations, in path order. For [`write()`], what was fixed.
    pub entries: Vec<DriftEntry>,
    /// Number of expected files checked.
    pub checked: usize,
}

impl Report {
    /// `true` when the corpus satisfies the contract.
    pub fn is_clean(&self) -> bool {
        self.entries.is_empty()
    }

    /// Count of entries with the given drift.
    pub fn count(&self, drift: Drift) -> usize {
        self.entries.iter().filter(|e| e.drift == drift).count()
    }
}

/// Compare `expected` (freshly generated) with `actual` (file path → text as
/// found on disk or elsewhere).
///
/// Only `.md` files that carry the sealmap header, and the reserved `_`
/// files, are considered part of the corpus; anything else in the directory
/// is ignored.
pub fn verify_against(expected: &Corpus, actual: &BTreeMap<SourcePath, String>) -> Report {
    let mut entries = Vec::new();
    for (path, want) in &expected.files {
        match actual.get(path) {
            None => entries.push(DriftEntry { path: path.clone(), drift: Drift::Missing }),
            Some(have) if have == want => {}
            Some(have) => {
                let drift = if is_reserved(path) {
                    Drift::Modified
                } else {
                    let want_hash = front_matter_hash(want);
                    match front_matter_hash(have) {
                        Some(h) if Some(&h) == want_hash.as_ref() => Drift::Modified,
                        _ => Drift::Stale,
                    }
                };
                entries.push(DriftEntry { path: path.clone(), drift });
            }
        }
    }
    for (path, text) in actual {
        if !expected.files.contains_key(path) && path.extension() == Some("md") && is_generated(text) {
            entries.push(DriftEntry { path: path.clone(), drift: Drift::Orphaned });
        }
    }
    entries.sort();
    Report { entries, checked: expected.files.len() }
}

/// Read every file under `dir` (UTF-8 only) keyed by relative path. A
/// missing directory reads as empty.
pub fn read_dir_corpus(dir: &Path) -> io::Result<BTreeMap<SourcePath, String>> {
    let mut out = BTreeMap::new();
    if dir.exists() {
        read_rec(dir, dir, &mut out)?;
    }
    Ok(out)
}

fn read_rec(root: &Path, dir: &Path, out: &mut BTreeMap<SourcePath, String>) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let ty = e.file_type()?;
        if ty.is_dir() {
            read_rec(root, &e.path(), out)?;
        } else if ty.is_file() {
            if let (Ok(text), Ok(rel)) = (fs::read_to_string(e.path()), SourcePath::relative_to(&e.path(), root)) {
                out.insert(rel, text);
            }
        }
    }
    Ok(())
}

/// Verify the corpus in `dir` against `expected`.
pub fn verify(dir: &Path, expected: &Corpus) -> io::Result<Report> {
    Ok(verify_against(expected, &read_dir_corpus(dir)?))
}

/// Bring `dir` into compliance with `expected`: write missing, stale and
/// modified files, delete orphaned generated documents (and nothing else).
/// Unchanged files are not touched, so mtimes stay stable. Returns what was
/// changed.
pub fn write(dir: &Path, expected: &Corpus) -> io::Result<Report> {
    let report = verify(dir, expected)?;
    for e in &report.entries {
        let path = dir.join(e.path.as_str());
        match e.drift {
            Drift::Orphaned => {
                fs::remove_file(&path)?;
                remove_empty_parents(dir, &path);
            }
            _ => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&path, &expected.files[&e.path])?;
            }
        }
    }
    Ok(report)
}

fn remove_empty_parents(root: &Path, file: &Path) {
    let mut cur = file.parent();
    while let Some(dir) = cur {
        if dir == root || fs::remove_dir(dir).is_err() {
            break;
        }
        cur = dir.parent();
    }
}

/// Hash of a whole corpus (all paths and contents), for cheap equality
/// checks across machines.
pub fn corpus_hash(corpus: &Corpus) -> ContentHash {
    let mut buf = String::new();
    for (p, t) in &corpus.files {
        buf.push_str(p.as_str());
        buf.push('\0');
        buf.push_str(t);
        buf.push('\0');
    }
    ContentHash::of_bytes(buf.as_bytes())
}
