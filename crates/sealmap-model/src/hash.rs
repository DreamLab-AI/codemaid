use std::fmt;

use serde::{Deserialize, Serialize};

/// A stable, portable content hash: BLAKE3 over text with `\r\n` and lone
/// `\r` normalised to `\n`.
///
/// Displayed and serialised as `blake3:<64 hex chars>`, so the algorithm is
/// self-describing if it ever changes.
///
/// ```
/// use sealmap_model::ContentHash;
///
/// let unix = ContentHash::of_text("fn main() {}\n");
/// let windows = ContentHash::of_text("fn main() {}\r\n");
/// assert_eq!(unix, windows);
/// assert!(unix.to_string().starts_with("blake3:"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentHash(String);

impl ContentHash {
    /// Hash `text` after normalising line endings.
    pub fn of_text(text: &str) -> Self {
        let normalised = normalise_newlines(text);
        Self::of_bytes(normalised.as_bytes())
    }

    /// Hash raw bytes with no normalisation.
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Self(format!("blake3:{}", blake3::hash(bytes).to_hex()))
    }

    /// Parse a previously displayed hash (`blake3:<hex>`).
    ///
    /// Returns `None` if the prefix or the hex payload is malformed.
    pub fn parse(s: &str) -> Option<Self> {
        let hex = s.strip_prefix("blake3:")?;
        (hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit())).then(|| Self(s.to_ascii_lowercase()))
    }

    /// The full `blake3:<hex>` string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The first `n` hex characters of the digest, handy for short stable ids.
    pub fn short(&self, n: usize) -> &str {
        let hex = &self.0["blake3:".len()..];
        &hex[..n.min(hex.len())]
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Convert `\r\n` and lone `\r` to `\n`.
pub(crate) fn normalise_newlines(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.contains('\r') {
        return std::borrow::Cow::Borrowed(text);
    }
    std::borrow::Cow::Owned(text.replace("\r\n", "\n").replace('\r', "\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_round_trips() {
        let h = ContentHash::of_text("x");
        assert_eq!(ContentHash::parse(h.as_str()), Some(h.clone()));
        assert_eq!(ContentHash::parse("md5:abc"), None);
        assert_eq!(h.short(8).len(), 8);
    }
}
