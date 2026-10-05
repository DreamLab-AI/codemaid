use std::fmt;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

/// A normalised, relative, `/`-separated path to a source file, used as the
/// primary key for files everywhere in sealmap.
///
/// Normalisation makes keys identical on every platform:
///
/// ```
/// use sealmap_model::SourcePath;
///
/// let a = SourcePath::new("src/./net/../lib.rs").unwrap();
/// let b = SourcePath::new("src\\lib.rs").unwrap();
/// assert_eq!(a, b);
/// assert_eq!(a.as_str(), "src/lib.rs");
///
/// assert!(SourcePath::new("/etc/passwd").is_err());
/// assert!(SourcePath::new("../outside.rs").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SourcePath(String);

/// Why a string could not become a [`SourcePath`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathError {
    /// The path was absolute (or had a drive / root prefix).
    #[error("path `{0}` must be relative to the codebase root")]
    Absolute(String),
    /// The path climbs above its root with `..`.
    #[error("path `{0}` escapes the codebase root")]
    Escapes(String),
    /// The path normalised to nothing.
    #[error("path is empty")]
    Empty,
}

impl SourcePath {
    /// Normalise `raw` into a [`SourcePath`].
    ///
    /// Backslashes are treated as separators, `.` components are dropped and
    /// `..` components are resolved lexically. Absolute paths and paths that
    /// escape the root are rejected.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, PathError> {
        let raw = raw.as_ref();
        let unified = raw.replace('\\', "/");
        if unified.starts_with('/') || unified.as_bytes().get(1) == Some(&b':') {
            return Err(PathError::Absolute(raw.to_owned()));
        }
        let mut parts: Vec<&str> = Vec::new();
        for part in unified.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    if parts.pop().is_none() {
                        return Err(PathError::Escapes(raw.to_owned()));
                    }
                }
                p => parts.push(p),
            }
        }
        if parts.is_empty() {
            return Err(PathError::Empty);
        }
        Ok(Self(parts.join("/")))
    }

    /// Make `path` relative to `root` and normalise it.
    ///
    /// Non-UTF-8 components are converted lossily.
    pub fn relative_to(path: &Path, root: &Path) -> Result<Self, PathError> {
        let rel = path.strip_prefix(root).unwrap_or(path);
        let joined: Vec<String> = rel
            .components()
            .filter_map(|c| match c {
                Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
                Component::ParentDir => Some("..".to_owned()),
                _ => None,
            })
            .collect();
        Self::new(joined.join("/"))
    }

    /// The normalised path string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The final component, e.g. `lib.rs`.
    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// The extension without the dot, if any.
    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name();
        name.rfind('.').filter(|&i| i > 0).map(|i| &name[i + 1..])
    }

    /// Path components in order.
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }

    /// Append a suffix to the whole path (`src/lib.rs` + `.mmd` →
    /// `src/lib.rs.mmd`). This is how the corpus maps sources 1:1 to diagrams.
    pub fn with_suffix(&self, suffix: &str) -> Self {
        Self(format!("{}{}", self.0, suffix))
    }
}

impl fmt::Display for SourcePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for SourcePath {
    type Error = PathError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SourcePath> for String {
    fn from(value: SourcePath) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_drive_letters_and_empty() {
        assert!(matches!(SourcePath::new("C:\\x.rs"), Err(PathError::Absolute(_))));
        assert_eq!(SourcePath::new("./"), Err(PathError::Empty));
    }

    #[test]
    fn extension_and_suffix() {
        let p = SourcePath::new("a/b.rs").unwrap();
        assert_eq!(p.extension(), Some("rs"));
        assert_eq!(p.with_suffix(".mmd").as_str(), "a/b.rs.mmd");
        assert_eq!(SourcePath::new(".gitignore").unwrap().extension(), None);
    }
}
