//! Reading an authored topic file: its front matter, its `sealed:` pointer,
//! its `topic_hash` and the `sym:` ids it cites.

use sealmap_model::{Fingerprint, IdError, SymbolId};
use serde::Serialize;

/// The front-matter key of the pointer line (`sealed: seals.lock`).
pub const POINTER_KEY: &str = "sealed";

/// The front matter of a topic, as line ranges into its normalised text.
struct FrontMatter<'a> {
    /// Lines between the opening and closing `---`, each without its `\n`.
    lines: Vec<&'a str>,
    /// Byte offset just past the closing `---` line.
    end: usize,
}

/// `\r\n` and lone `\r` become `\n`.
fn normalise(text: &str) -> std::borrow::Cow<'_, str> {
    if text.contains('\r') { text.replace("\r\n", "\n").replace('\r', "\n").into() } else { text.into() }
}

/// Split normalised text into front matter and body. Front matter opens on
/// the first line with exactly `---` and closes at the next line that is
/// exactly `---`.
fn front_matter(text: &str) -> Option<FrontMatter<'_>> {
    let rest = text.strip_prefix("---\n")?;
    let mut offset = 4;
    let mut lines = Vec::new();
    for line in rest.split_inclusive('\n') {
        offset += line.len();
        let bare = line.strip_suffix('\n').unwrap_or(line);
        if bare == "---" {
            return Some(FrontMatter { lines, end: offset });
        }
        lines.push(bare);
    }
    None
}

/// The value of a top-level front-matter key, if the line is `key: value`.
fn key_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(key)?.strip_prefix(':')?;
    Some(unquote(rest.trim()))
}

fn unquote(v: &str) -> &str {
    for q in ['"', '\''] {
        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
            return &v[1..v.len() - 1];
        }
    }
    v
}

/// The topic's front-matter `id:`, if it has front matter with one.
///
/// ```
/// use sealmap_corpus::seal::topic_id;
///
/// assert_eq!(topic_id("---\nid: COR-04\ntitle: x\n---\nbody").as_deref(), Some("COR-04"));
/// assert_eq!(topic_id("# no front matter"), None);
/// ```
pub fn topic_id(text: &str) -> Option<String> {
    let text = normalise(text);
    let fm = front_matter(&text)?;
    fm.lines.iter().find_map(|l| key_value(l, "id")).filter(|v| !v.is_empty()).map(str::to_owned)
}

/// The values of every `sealed:` line in the topic's front matter, in order.
/// A well-formed sealed topic has exactly one, naming the lock file.
pub fn pointers(text: &str) -> Vec<String> {
    let text = normalise(text);
    front_matter(&text)
        .map(|fm| fm.lines.iter().filter_map(|l| key_value(l, POINTER_KEY)).map(str::to_owned).collect())
        .unwrap_or_default()
}

/// The fingerprint that binds a topic's prose to its seal.
///
/// **Definition (`sm1`).** Take the topic file's text and
///
/// 1. normalise line endings: `\r\n`, then any lone `\r`, become `\n`;
/// 2. if the text opens with front matter (a first line that is exactly
///    `---`, closed by a later line that is exactly `---`), delete every
///    line inside it that starts with `sealed:` at column 0, together with
///    its `\n`; nothing else is touched, and text outside the front matter
///    is never edited;
/// 3. hash the resulting UTF-8 bytes with plain (unkeyed) BLAKE3 and keep
///    the first 16 bytes, printed `blake3-16:<32 lower-case hex>`.
///
/// So adding, removing or changing the pointer line never changes the
/// hash, and neither does a checkout that converts line endings; any other
/// byte does, including whitespace and the stamp in `verified_commit`. A
/// shell equivalent for a topic with one pointer line is
/// `tr -d '\r' | sed '2,/^---$/{/^sealed:/d}' | b3sum | cut -c1-32`
/// (for `\r\n` input; a lone `\r` needs the full rule).
///
/// ```
/// use sealmap_corpus::seal::topic_hash;
///
/// let unsealed = "---\nid: COR-04\n---\nBody.\n";
/// let sealed = "---\nid: COR-04\nsealed: seals.lock\n---\nBody.\n";
/// assert_eq!(topic_hash(unsealed), topic_hash(sealed));
/// assert_eq!(topic_hash(sealed), topic_hash(&sealed.replace('\n', "\r\n")));
/// assert_ne!(topic_hash(sealed), topic_hash(&sealed.replace("Body.", "Body!")));
/// // Golden value: plain BLAKE3 of the pointer-free text, first 16 bytes.
/// assert_eq!(topic_hash(unsealed).to_string(), format!("blake3-16:{}", &blake3::hash(unsealed.as_bytes()).to_hex()[..32]));
/// ```
pub fn topic_hash(text: &str) -> Fingerprint {
    let text = normalise(text);
    let hashed = without_pointer(&text);
    Fingerprint::from_blake3(&blake3::hash(hashed.as_bytes()))
}

/// The normalised text with every front-matter pointer line removed.
fn without_pointer(text: &str) -> std::borrow::Cow<'_, str> {
    let Some(fm) = front_matter(text) else { return text.into() };
    if !fm.lines.iter().any(|l| l.starts_with("sealed:")) {
        return text.into();
    }
    let mut out = String::with_capacity(text.len());
    out.push_str("---\n");
    for l in &fm.lines {
        if !l.starts_with("sealed:") {
            out.push_str(l);
            out.push('\n');
        }
    }
    out.push_str(after_close(text, &fm));
    out.into()
}

/// The closing `---` and everything after it (the closing line may be the
/// last line and lack its `\n`).
fn after_close<'a>(text: &'a str, fm: &FrontMatter<'_>) -> &'a str {
    let newline = usize::from(text[..fm.end].ends_with('\n'));
    &text[fm.end - newline - 3..]
}

/// `text` (line endings normalised) with exactly one pointer line,
/// `sealed: <lock_name>`: an existing pointer line is replaced in place, and
/// otherwise one is added as the last front-matter line. Returns `None` if
/// the text has no front matter.
///
/// ```
/// use sealmap_corpus::seal::with_pointer;
///
/// let t = with_pointer("---\nid: A-01\n---\nBody\n", "seals.lock").unwrap();
/// assert_eq!(t, "---\nid: A-01\nsealed: seals.lock\n---\nBody\n");
/// assert_eq!(with_pointer(&t, "seals.lock").unwrap(), t);
/// ```
pub fn with_pointer(text: &str, lock_name: &str) -> Option<String> {
    let text = normalise(text);
    let fm = front_matter(&text)?;
    let pointer = format!("{POINTER_KEY}: {lock_name}");
    let mut out = String::with_capacity(text.len() + pointer.len() + 1);
    out.push_str("---\n");
    let mut placed = false;
    for l in &fm.lines {
        if key_value(l, POINTER_KEY).is_some() {
            if !placed {
                out.push_str(&pointer);
                out.push('\n');
                placed = true;
            }
        } else {
            out.push_str(l);
            out.push('\n');
        }
    }
    if !placed {
        out.push_str(&pointer);
        out.push('\n');
    }
    out.push_str(after_close(&text, &fm));
    Some(out)
}

/// A `sym:` citation found in a topic's text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Citation {
    /// The cited text, as written inside the code span (with a line break
    /// inside the span read as a space, as Markdown renders it).
    pub text: String,
    /// The 1-based line the code span starts on.
    pub line: u32,
    /// The parsed id, or why the text is not a canonical `sym:` id.
    #[serde(skip)]
    pub id: Result<SymbolId, IdError>,
}

/// Every `sym:` citation in a topic, in text order.
///
/// A citation is a Markdown **inline code span** whose content starts the
/// way a global id does: `sym:`, a manager (`[a-z][a-z0-9-]*`, not
/// `extern`), then a space, as in `` `sym:cargo shop . db/Db#insert().` ``.
/// Ids contain spaces, so nothing but the code span delimits one. A span
/// that starts like that but does not parse is still returned (with the
/// parse error), so a mistyped citation fails a check instead of vanishing.
/// Spans holding the other forms (`` `sym:? insert` ``,
/// `` `sym:extern tokio::spawn` ``) or a bare `` `sym:` `` are mentions of
/// the grammar, not citations: those ids name no definition and can never
/// be sealed.
///
/// Spans follow CommonMark: a run of
/// N backticks opens a span that the next run of exactly N closes, a line
/// break inside a span reads as a space, and one leading and one trailing
/// space are stripped when both are present. An id whose quoted name holds
/// a backtick (written doubled, see [`sealmap_model::sym`]) therefore needs
/// a delimiter longer than any backtick run inside it, padded with spaces:
/// ```` ``` sym:cargo x . `a``b`# ``` ````. Spans are matched within a
/// paragraph (a blank line ends one), so a citation wrapped across lines is
/// still found.
///
/// Not scanned: the front matter, fenced code blocks (```` ``` ```` or
/// `~~~`), and so Mermaid diagrams. An id written there is an example, not a
/// citation.
///
/// ```
/// use sealmap_corpus::seal::citations;
///
/// let text = "---\nid: A-01\n---\nSee `sym:cargo shop . db/Db#insert().` and `sym:cargo\nshop . Orders#`.\n\n```text\n`sym:? ignored`\n```\n";
/// let c = citations(text);
/// assert_eq!(c.len(), 2);
/// assert_eq!(c[1].text, "sym:cargo shop . Orders#");
/// assert_eq!(c[1].line, 4);
/// assert!(c.iter().all(|c| c.id.is_ok()));
/// ```
pub fn citations(text: &str) -> Vec<Citation> {
    let text = normalise(text);
    let (body, first_line) = match front_matter(&text) {
        Some(fm) => (&text[fm.end..], fm.lines.len() as u32 + 3),
        None => (&text[..], 1),
    };
    let mut out = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut para: Vec<(u32, &str)> = Vec::new();
    for (i, line) in body.lines().enumerate() {
        let n = first_line + i as u32;
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        match fence {
            Some((c, len)) => {
                let run = trimmed.chars().take_while(|&x| x == c).count();
                if indent <= 3 && run >= len && trimmed[run * c.len_utf8()..].trim().is_empty() {
                    fence = None;
                }
                continue;
            }
            None => {
                if let Some(open) = fence_open(trimmed).filter(|_| indent <= 3) {
                    scan_paragraph(&para, &mut out);
                    para.clear();
                    fence = Some(open);
                    continue;
                }
            }
        }
        if line.trim().is_empty() {
            scan_paragraph(&para, &mut out);
            para.clear();
        } else {
            para.push((n, line));
        }
    }
    scan_paragraph(&para, &mut out);
    out
}

/// A fence opener: three or more backticks (with no backtick in the info
/// string) or three or more tildes.
fn fence_open(trimmed: &str) -> Option<(char, usize)> {
    for c in ['`', '~'] {
        let run = trimmed.chars().take_while(|&x| x == c).count();
        if run >= 3 && !(c == '`' && trimmed[run..].contains('`')) {
            return Some((c, run));
        }
    }
    None
}

/// Find code spans across the lines of one paragraph.
fn scan_paragraph(lines: &[(u32, &str)], out: &mut Vec<Citation>) {
    if lines.is_empty() {
        return;
    }
    // Join with '\n' and remember where each line starts.
    let mut joined = String::new();
    let mut starts = Vec::with_capacity(lines.len());
    for (i, (n, l)) in lines.iter().enumerate() {
        if i > 0 {
            joined.push('\n');
        }
        starts.push((joined.len(), *n));
        joined.push_str(l);
    }
    let line_at = |pos: usize| starts.iter().rev().find(|(s, _)| *s <= pos).map_or(lines[0].0, |(_, n)| *n);
    let b = joined.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' {
            i += 2;
            continue;
        }
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let open = run_len(b, i);
        let content_start = i + open;
        // Find the next run of exactly `open` backticks.
        let mut j = content_start;
        let mut close = None;
        while j < b.len() {
            if b[j] == b'`' {
                let r = run_len(b, j);
                if r == open {
                    close = Some(j);
                    break;
                }
                j += r;
            } else {
                j += 1;
            }
        }
        match close {
            Some(end) => {
                let raw = joined[content_start..end].replace('\n', " ");
                let content = strip_one_space(&raw);
                if looks_global(content) {
                    out.push(Citation { text: content.to_owned(), line: line_at(i), id: SymbolId::parse(content) });
                }
                i = end + open;
            }
            // An unmatched run is literal text.
            None => i = content_start,
        }
    }
}

/// `sym:` + manager + space, the manager not being `extern`.
fn looks_global(content: &str) -> bool {
    let Some(rest) = content.strip_prefix("sym:") else { return false };
    let Some((manager, _)) = rest.split_once(' ') else { return false };
    let mut chars = manager.chars();
    manager != "extern"
        && chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn run_len(b: &[u8], at: usize) -> usize {
    b[at..].iter().take_while(|&&c| c == b'`').count()
}

fn strip_one_space(s: &str) -> &str {
    if s.len() >= 2 && s.starts_with(' ') && s.ends_with(' ') && !s.bytes().all(|c| c == b' ') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_removal_is_exact() {
        let t = "---\nid: X-01\nsealed: seals.lock\nsealedish: no\n---\nsealed: body text stays\n";
        assert_eq!(without_pointer(t), "---\nid: X-01\nsealedish: no\n---\nsealed: body text stays\n");
        // No trailing newline after the closing fence.
        let t = "---\nid: X-01\nsealed: seals.lock\n---";
        assert_eq!(without_pointer(t), "---\nid: X-01\n---");
    }

    #[test]
    fn with_pointer_replaces_and_dedups() {
        let t = "---\nid: X-01\nsealed: other.lock\nsealed: x\n---\nbody";
        assert_eq!(with_pointer(t, "seals.lock").unwrap(), "---\nid: X-01\nsealed: seals.lock\n---\nbody");
        assert_eq!(with_pointer("---\nid: X\n---", "s").unwrap(), "---\nid: X\nsealed: s\n---");
        assert!(with_pointer("no front matter", "s").is_none());
    }

    #[test]
    fn code_spans_follow_commonmark() {
        let t = "a ``` sym:cargo x . `q``r`# ``` b `sym:cargo x . X#` c `not` d \\`sym:cargo x . E#`\n";
        let c: Vec<_> = citations(t).into_iter().map(|c| c.text).collect();
        assert_eq!(c, ["sym:cargo x . `q``r`#", "sym:cargo x . X#"]);
        // Unmatched backticks are literal and do not swallow the paragraph.
        let c = citations("``` x\n");
        assert!(c.is_empty());
        // The first backtick pairs with the second, leaving the last unmatched.
        assert!(citations("a ` b `sym:cargo x . Y#`").is_empty());
    }

    #[test]
    fn fences_and_paragraphs() {
        let t = "x\n~~~~\n`sym:k a . A#`\n~~~\n`sym:k a . B#`\n~~~~\n`sym:k a . C#`\n\n    ```\n`sym:k a . D#`\n";
        let c: Vec<_> = citations(t).into_iter().map(|c| (c.text, c.line)).collect();
        // B is inside the four-tilde fence (a three-tilde run does not close it).
        assert_eq!(c, [("sym:k a . C#".to_owned(), 7), ("sym:k a . D#".to_owned(), 10)]);
        // A span does not cross a blank line.
        assert!(citations("`sym:k a\n\n. A#`").is_empty());
    }

    #[test]
    fn only_global_shaped_spans_are_citations() {
        let t = "`sym:` `sym:extern tokio::spawn` `sym:? insert` `sym:Cargo x . A#` `sym:cargo x . A#()` `sym:npm-2 @a/b . f().`";
        let c: Vec<_> = citations(t).into_iter().map(|c| (c.text, c.id.is_ok())).collect();
        // The malformed global-shaped span is kept, with its parse error.
        assert_eq!(c, [("sym:cargo x . A#()".to_owned(), false), ("sym:npm-2 @a/b . f().".to_owned(), true)]);
    }
}
