use std::fmt;

/// Words Mermaid treats as keywords in at least one diagram kind. An id equal
/// to one of these (case-insensitively) gets a trailing `_`.
const RESERVED: &[&str] = &[
    "end",
    "graph",
    "flowchart",
    "subgraph",
    "class",
    "classdef",
    "style",
    "linkstyle",
    "click",
    "call",
    "href",
    "direction",
    "participant",
    "actor",
    "loop",
    "alt",
    "else",
    "opt",
    "par",
    "and",
    "rect",
    "note",
    "critical",
    "break",
    "box",
    "autonumber",
    "activate",
    "deactivate",
    "create",
    "destroy",
    "namespace",
    "default",
];

/// A Mermaid-safe identifier.
///
/// Ids contain only ASCII letters, digits and `_`, never start with a digit
/// and never collide with a Mermaid keyword. [`Ident::from_path`] maps a
/// `::`-separated symbol path to `__`-separated form, which is **stable,
/// readable and injective**: the same path always gives the same id in every
/// diagram, and two different paths never share one, so diagrams produced
/// separately can be merged by id.
///
/// ```
/// use codemaid_mermaid::Ident;
///
/// assert_eq!(Ident::from_path("my_crate::net::Client").as_str(), "my_crate__net__Client");
/// assert_eq!(Ident::from_path("std::fs").as_str(), "std__fs");
/// // Paths the plain form cannot represent unambiguously get a hash suffix.
/// assert_ne!(Ident::from_path("a::b__c"), Ident::from_path("a::b::c"));
/// assert_ne!(Ident::from_path("T::<From<u8>>::from"), Ident::from_path("T::<From_u8_>::from"));
/// assert_eq!(Ident::new("end").as_str(), "end_");
/// assert_eq!(Ident::new("9lives").as_str(), "_9lives");
/// assert_eq!(Ident::new("a-b c<T>").as_str(), "a_b_c_T_");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(String);

impl Ident {
    /// Sanitise an arbitrary string into an id. Every character outside
    /// `[A-Za-z0-9_]` becomes `_`.
    pub fn new(raw: &str) -> Self {
        let mut s: String = raw.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
        if s.is_empty() || s.starts_with(|c: char| c.is_ascii_digit()) {
            s.insert(0, '_');
        }
        if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(&s)) {
            s.push('_');
        }
        Self(s)
    }

    /// Map a `::`-separated path to an id, using `__` as the separator.
    ///
    /// The mapping is injective. When every segment is plain (ASCII letters,
    /// digits and single inner `_`, not starting with a digit), the id is
    /// just the segments joined by `__`; it can be split back unambiguously
    /// and never contains `___` (a plain path that is a Mermaid keyword gets
    /// a trailing `___`). Any other path (`__` or `_` at a segment edge,
    /// generics, `<Trait>` impl segments, raw identifiers) gets the sanitised
    /// text plus `___h` and the 64-bit FNV-1a hash of the exact path, so it
    /// cannot equal a plain id or another path's id short of a hash
    /// collision.
    pub fn from_path(path: &str) -> Self {
        let joined = path.replace("::", "__");
        if is_plain_path(path) {
            // A keyword gets `___`, which no plain id contains and no hashed
            // id ends with.
            if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(&joined)) {
                return Self(joined + "___");
            }
            return Self(joined);
        }
        let mut s = Self::new(&joined).0;
        s.push_str(&format!("___h{:016x}", fnv1a64(path.as_bytes())));
        Self(s)
    }

    /// The id text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `true` if every `::` segment is non-empty, ASCII alphanumeric plus `_`,
/// does not start or end with `_`, contains no `__`, and the first segment
/// does not start with a digit. Joining such segments with `__` is
/// reversible.
fn is_plain_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with(|c: char| c.is_ascii_digit())
        && path.split("::").all(|seg| {
            !seg.is_empty()
                && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !seg.starts_with('_')
                && !seg.ends_with('_')
                && !seg.contains("__")
        })
}

/// 64-bit FNV-1a. A disambiguator for diagram ids, not a security
/// primitive: ids are public and only need to be stable and well spread.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Escape free text for use as a label (messages, notes, block labels, node
/// text, aliases).
///
/// * `#` → `#35;` and `;` → `#59;` (entity codes Mermaid decodes on render).
/// * `"` → `#quot;`, so labels can sit inside `["…"]` quotes.
/// * Newlines and tabs collapse to a single space; runs of whitespace are
///   squeezed; leading/trailing whitespace is trimmed.
/// * `` ` `` → `#96;`, so a label can never close a Markdown code fence.
/// * `%` → `#37;`, so a label can never start a `%%` comment (Mermaid's
///   lexer also chokes on `%` next to entity codes).
///
/// ```
/// use codemaid_mermaid::escape_text;
/// assert_eq!(escape_text("a; b # c"), "a#59; b #35; c");
/// assert_eq!(escape_text("say \"hi\"\n  now"), "say #quot;hi#quot; now");
/// ```
pub fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_space = true;
    for c in text.chars() {
        match c {
            '#' => out.push_str("#35;"),
            ';' => out.push_str("#59;"),
            '%' => out.push_str("#37;"),
            '`' => out.push_str("#96;"),
            '"' => out.push_str("#quot;"),
            c if c.is_whitespace() => {
                if !last_space {
                    out.push(' ');
                }
                last_space = true;
                continue;
            }
            c => out.push(c),
        }
        last_space = false;
    }
    while out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Escape a type or member text for a class-diagram body.
///
/// On top of [`escape_text`], characters with meaning inside a class body
/// become Mermaid entity codes, which render as the original character:
/// `<`/`>` → `#lt;`/`#gt;` (Mermaid's `~` generic marker cannot nest
/// reliably), `{`/`}` → `#123;`/`#125;` (a `}` would close the body) and,
/// when `is_field` is set, `(`/`)` → `#40;`/`#41;` (a `(` would turn a field
/// into a method). `->` is kept as is.
///
/// ```
/// use codemaid_mermaid::escape_type;
/// assert_eq!(escape_type("HashMap<String, Vec<u8>>", true), "HashMap#lt;String, Vec#lt;u8#gt;#gt;");
/// assert_eq!(escape_type("fn(u8) -> u8", true), "fn#40;u8#41; -> u8");
/// assert_eq!(escape_type("(a: (u8, u8))", false), "(a: (u8, u8))");
/// ```
pub fn escape_type(text: &str, is_field: bool) -> String {
    let escaped = escape_text(&text.replace("->", "\u{1}"));
    let mut out = String::with_capacity(escaped.len() + 8);
    for c in escaped.chars() {
        match c {
            '<' => out.push_str("#lt;"),
            '>' => out.push_str("#gt;"),
            '{' => out.push_str("#123;"),
            '}' => out.push_str("#125;"),
            '(' if is_field => out.push_str("#40;"),
            ')' if is_field => out.push_str("#41;"),
            '\u{1}' => out.push_str("->"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn fnv1a64_matches_reference_vectors() {
        // From the FNV reference test suite (draft-eastlake-fnv, isthe.com/chongo/tech/comp/fnv).
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn plain_paths_keep_their_readable_form() {
        assert_eq!(Ident::from_path("a::b_c::D").as_str(), "a__b_c__D");
        assert_eq!(Ident::from_path("x").as_str(), "x");
        assert_eq!(Ident::from_path("Actor").as_str(), "Actor___");
    }

    #[test]
    fn from_path_is_injective_on_lossy_shapes() {
        let paths = [
            "a::b__c",
            "a::b::c",
            "a__b::c",
            "a::_b",
            "a_::b",
            "a::b_",
            "end",
            "end_",
            "end___",
            "Actor",
            "9x",
            "_9x",
            "T::<From<u8>>::from",
            "T::<From_u8_>::from",
            "T::<From<u16>>::from",
            "a-b",
            "a_b",
            "a b",
            "r#type",
            "r_type",
        ];
        let mut seen: BTreeMap<Ident, &str> = BTreeMap::new();
        for p in paths {
            let id = Ident::from_path(p);
            assert!(id.as_str().chars().all(|c| c.is_ascii_alphanumeric() || c == '_'), "{id}");
            assert!(!id.as_str().starts_with(|c: char| c.is_ascii_digit()), "{id}");
            if let Some(prev) = seen.insert(id.clone(), p) {
                panic!("{prev:?} and {p:?} both map to {id}");
            }
        }
    }
}
