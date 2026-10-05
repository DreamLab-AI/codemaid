//! Compact, deterministic printing of token streams.
//!
//! `TokenStream::to_string` puts spaces between almost every token
//! (`Vec < String >`, `& mut self`). Labels in diagrams must be short and
//! read like source, so this module squeezes that output back into the form a
//! person would write. The rules are purely textual, so the result depends
//! only on the tokens.

use quote::ToTokens;

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

/// Print any syn node compactly.
pub(crate) fn tokens(node: &impl ToTokens) -> String {
    squeeze(&node.to_token_stream().to_string())
}

/// Apply the spacing rules until the string stops changing.
pub(crate) fn squeeze(raw: &str) -> String {
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
pub(crate) fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_owned();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
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
    }
}
