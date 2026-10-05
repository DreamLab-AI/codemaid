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
/// and never collide with a Mermaid keyword. [`Ident::new`] sanitises any
/// text (lossy, for ids you mint yourself). With the default `model`
/// feature, `Ident::from_symbol` derives an id from a
/// `sealmap_model::SymbolId` by an **injective** encoding, so two symbols
/// never share a diagram id and diagrams drawn separately can be merged by
/// id.
///
/// ```
/// use sealmap_mermaid::Ident;
///
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

    /// The id text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
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
/// use sealmap_mermaid::escape_text;
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
/// use sealmap_mermaid::escape_type;
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

impl Ident {
    /// Wrap text already known to be a valid, unique id (used by the
    /// injective encoders in this crate).
    #[cfg_attr(not(feature = "model"), allow(dead_code))]
    pub(crate) fn from_valid(text: String) -> Self {
        Self(text)
    }
}

/// `true` if `s` is a Mermaid keyword (case-insensitively).
#[cfg_attr(not(feature = "model"), allow(dead_code))]
pub(crate) fn is_reserved(s: &str) -> bool {
    RESERVED.iter().any(|r| r.eq_ignore_ascii_case(s))
}
