//! Label rules: how source text becomes the short strings on diagram arrows
//! and fragments.
//!
//! Labels must be short and read like source. Printed token streams put
//! spaces between almost every token (`Vec < String >`, `& mut self`);
//! [`squeeze`] folds that back into the form a person would write. The rules
//! are purely textual, so the result depends only on the text.
//!
//! ```
//! use sealmap_frontend::labels::{LABEL_MAX, call_label, clip, squeeze};
//!
//! assert_eq!(squeeze("fn foo (& self , a : u8) -> u8"), "fn foo(&self, a: u8) -> u8");
//! assert_eq!(call_label("Db::open", &["path".into(), "_".into()]), "Db::open(path, _)");
//! assert!(clip(&"x".repeat(100), LABEL_MAX).ends_with('…'));
//! ```

/// Max characters for labels on arrows and fragments.
pub const LABEL_MAX: usize = 56;
/// Max characters for a stored signature.
pub const SIGNATURE_MAX: usize = 200;
/// Max characters for a literal shown as a call argument.
pub const ARG_LITERAL_MAX: usize = 14;
/// Max characters for a doc summary (the first sentence of a doc comment).
pub const DOC_SUMMARY_MAX: usize = 160;

const RULES: &[(&str, &str)] = &[
    (" :: ", "::"),
    (":: ", "::"),
    (" ::", "::"),
    (" ,", ","),
    (" ;", ";"),
    (" . ", "."),
    (" .", "."),
    (". ", "."),
    ("( ", "("),
    (" )", ")"),
    ("[ ", "["),
    (" ]", "]"),
    (" < ", "<"),
    ("< ", "<"),
    (" <", "<"),
    (" >", ">"),
    ("& ", "&"),
    (" ?", "?"),
    (" !", "!"),
    ("! (", "!("),
    ("! [", "!["),
    ("! {", "!{"),
    (" :", ":"),
    ("# [", "#["),
    ("' ", "'"),
    ("{ }", "{}"),
    ("! ", "!"),
];

/// Apply the spacing rules until the string stops changing.
pub fn squeeze(raw: &str) -> String {
    let mut s = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    loop {
        let before = s.len();
        for (from, to) in RULES {
            if s.contains(from) {
                s = s.replace(from, to);
            }
        }
        if s.len() == before {
            break;
        }
    }
    s = call_parens(&s);
    // `->` and `=>` lose their surrounding spaces to the `>` rule; restore.
    s.replace("->", " -> ")
        .replace("=>", " => ")
        .replace("&&", " && ")
        .replace("! =", "!=")
        .replace("!=", " != ")
        .replace("  ", " ")
        .replace("( ", "(")
}

/// Remove the space in `name (` and `> (` (calls, signatures, generics), but
/// not after keywords such as `in (` or `if (`.
fn call_parens(s: &str) -> String {
    const KEYWORDS: &[&str] = &["in", "if", "match", "return", "while", "as", "else", "mut", "move", "impl", "dyn"];
    let mut out = String::with_capacity(s.len());
    let bytes: Vec<char> = s.chars().collect();
    for (i, &c) in bytes.iter().enumerate() {
        if c == ' ' && bytes.get(i + 1) == Some(&'(') && i > 0 {
            let prev = bytes[i - 1];
            if prev == '>' || prev.is_alphanumeric() || prev == '_' {
                let word: String = out.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                let word: String = word.chars().rev().collect();
                if !KEYWORDS.contains(&word.as_str()) {
                    continue;
                }
            }
        }
        out.push(c);
    }
    out
}

/// Truncate to at most `max` chars on a char boundary, appending `…`.
pub fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_owned();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// The message for a call: `name(arg, arg)`, clipped to [`LABEL_MAX`].
/// `args` are the frontend's argument sketches (identifiers and short
/// literals kept, anything else `_`).
pub fn call_label(name: &str, args: &[String]) -> String {
    clip(&format!("{name}({})", args.join(", ")), LABEL_MAX)
}

/// The label for a condition (`if` / `while` guard), clipped as if it were
/// prefixed with `if ` so that the guard and the `if` arm agree.
pub fn condition_label(text: &str) -> String {
    clip(&format!("if {text}"), LABEL_MAX).trim_start_matches("if ").to_owned()
}

/// How a closure body passed to a call is placed in the flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferredShape {
    /// The callee spawns a task (`*spawn*`).
    Parallel,
    /// The callee runs the closure once per element.
    Loop,
    /// Anything else: the closure may run.
    Optional,
}

/// The shape of a closure body passed to `callee`; `looping` lists the
/// callee names that run their closure once per element.
pub fn deferred_shape(callee: &str, looping: &[&str]) -> DeferredShape {
    if callee.contains("spawn") {
        DeferredShape::Parallel
    } else if looping.contains(&callee) {
        DeferredShape::Loop
    } else {
        DeferredShape::Optional
    }
}

/// The label of a deferred closure body: `spawned task`, `each via map`,
/// `via with`, or `closure` when the callee is not a plain name.
pub fn deferred_label(callee: &str, shape: DeferredShape) -> String {
    match shape {
        DeferredShape::Parallel => format!("{callee}ed task"),
        DeferredShape::Loop => format!("each via {callee}"),
        DeferredShape::Optional if callee.is_empty() => "closure".to_owned(),
        DeferredShape::Optional => format!("via {callee}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squeezes_common_shapes() {
        assert_eq!(squeeze("Vec < String >"), "Vec<String>");
        assert_eq!(squeeze("& mut self"), "&mut self");
        assert_eq!(squeeze("Option < & 'a str >"), "Option<&'a str>");
        assert_eq!(squeeze("fn foo (& self , a : u8) -> u8"), "fn foo(&self, a: u8) -> u8");
        assert_eq!(squeeze("std :: fs :: read (p) ?"), "std::fs::read(p)?");
        assert_eq!(squeeze("HashMap < K , Vec < V > >"), "HashMap<K, Vec<V>>");
        assert_eq!(squeeze("Some (x) => y"), "Some(x) => y");
        assert_eq!(squeeze("! a . b () && c != d"), "!a.b() && c != d");
    }

    #[test]
    fn clips_on_char_boundary() {
        assert_eq!(clip("abcdef", 4), "abc…");
        assert_eq!(clip("ab", 4), "ab");
        assert_eq!(clip("äöüß", 3), "äö…");
    }

    #[test]
    fn condition_labels_clip_with_the_if_prefix() {
        assert_eq!(condition_label("ready"), "ready");
        let long = "x".repeat(LABEL_MAX);
        assert_eq!(condition_label(&long).chars().count(), LABEL_MAX - 3);
    }
}
