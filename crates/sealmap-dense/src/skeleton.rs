//! Skeleton lines: one Rust-like line per symbol, grouped by file in source
//! order.

use std::collections::BTreeMap;

use sealmap_model::{Codebase, Member, MemberKind, RelationKind, SourcePath, Symbol, SymbolId, Visibility};

use crate::short::ShortNames;

/// Source order: file, then position, then id (a total order, so equal
/// spans still sort the same way every time).
pub(crate) fn order_key(sym: &Symbol) -> (&SourcePath, u32, u32, &SymbolId) {
    (&sym.file, sym.span.start_line, sym.span.start_col, &sym.id)
}

/// Write `symbols` as `## <path>` sections, each listing its symbols in
/// source order. Files are ordered by path.
pub(crate) fn write_files<'a>(
    out: &mut String,
    cb: &Codebase,
    shorts: &ShortNames<'_>,
    symbols: impl IntoIterator<Item = &'a Symbol>,
) {
    let mut by_file: BTreeMap<&SourcePath, Vec<&Symbol>> = BTreeMap::new();
    for sym in symbols {
        by_file.entry(&sym.file).or_default().push(sym);
    }
    for (path, mut syms) in by_file {
        syms.sort_by(|a, b| order_key(a).cmp(&order_key(b)));
        out.push_str("## ");
        out.push_str(&one_line(path.as_str()));
        out.push('\n');
        for sym in syms {
            line(out, cb, shorts, sym);
        }
    }
}

/// `<vis> <signature or kind name><members><impls> L<a>-<b> @<short>`.
pub(crate) fn line(out: &mut String, cb: &Codebase, shorts: &ShortNames<'_>, sym: &Symbol) {
    vis(out, &sym.visibility);
    match &sym.signature {
        Some(sig) => out.push_str(&one_line(strip_vis(sig))),
        None => {
            out.push_str(match sym.kind {
                sealmap_model::SymbolKind::Macro => "macro_rules!",
                kind => kind.keyword(),
            });
            out.push(' ');
            out.push_str(&sym.name);
            if !sym.generics.is_empty() {
                out.push('<');
                out.push_str(&one_line(&sym.generics.join(", ")));
                out.push('>');
            }
        }
    }
    if !sym.members.is_empty() {
        out.push_str(" { ");
        for (i, m) in sym.members.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            member(out, m);
        }
        out.push_str(" }");
    }
    let supers: Vec<String> = cb
        .relations_from(&sym.id)
        .filter(|r| matches!(r.kind, RelationKind::Implements | RelationKind::Extends))
        .map(|r| shorts.get(&r.to).map_or_else(|| r.to.name().into_owned(), str::to_owned))
        .collect();
    if !supers.is_empty() {
        out.push_str(" : ");
        out.push_str(&supers.join(" + "));
    }
    out.push(' ');
    span(out, sym);
    out.push_str(" @");
    out.push_str(shorts.get(&sym.id).unwrap_or("_"));
    out.push('\n');
}

/// `L<start>-<end>`, always both ends.
pub(crate) fn span(out: &mut String, sym: &Symbol) {
    out.push('L');
    out.push_str(&sym.span.start_line.to_string());
    out.push('-');
    out.push_str(&sym.span.end_line.to_string());
}

fn member(out: &mut String, m: &Member) {
    let ty = m.ty.as_deref().map(one_line);
    match m.kind {
        MemberKind::Field if m.name.bytes().all(|b| b.is_ascii_digit()) => out.push_str(ty.as_deref().unwrap_or("_")),
        MemberKind::Field => {
            out.push_str(&m.name);
            if let Some(ty) = ty {
                out.push_str(": ");
                out.push_str(&ty);
            }
        }
        MemberKind::Variant => {
            out.push_str(&m.name);
            if let Some(ty) = ty {
                out.push_str(&ty);
            }
        }
        MemberKind::AssocConst => {
            out.push_str("const ");
            out.push_str(&m.name);
            if let Some(ty) = ty {
                out.push_str(": ");
                out.push_str(&ty);
            }
        }
        MemberKind::AssocType => {
            out.push_str("type ");
            out.push_str(&m.name);
            if let Some(ty) = ty {
                out.push_str(": ");
                out.push_str(&ty);
            }
        }
        MemberKind::RequiredMethod => match ty {
            Some(sig) => out.push_str(strip_vis(&sig)),
            None => {
                out.push_str("fn ");
                out.push_str(&m.name);
            }
        },
    }
}

/// Write the Rust spelling of a visibility, with its trailing space.
fn vis(out: &mut String, v: &Visibility) {
    match v {
        Visibility::Public => out.push_str("pub "),
        Visibility::Crate => out.push_str("pub(crate) "),
        Visibility::Restricted(p) if p == "self" => {}
        Visibility::Restricted(p) if p == "super" => out.push_str("pub(super) "),
        Visibility::Restricted(p) => {
            out.push_str("pub(in ");
            out.push_str(&one_line(p));
            out.push_str(") ");
        }
        Visibility::Private => {}
    }
}

/// A signature without its leading visibility (the line writes its own).
fn strip_vis(sig: &str) -> &str {
    let s = sig.trim_start();
    if let Some(rest) = s.strip_prefix("pub") {
        if let Some(scoped) = rest.strip_prefix('(') {
            if let Some(close) = scoped.find(')') {
                return scoped[close + 1..].trim_start();
            }
        } else if rest.starts_with(' ') {
            return rest.trim_start();
        }
    }
    s
}

/// Text folded onto one line: every run of whitespace becomes one space.
pub(crate) fn one_line(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut gap = false;
    for c in text.chars() {
        if c.is_whitespace() {
            gap = true;
        } else {
            if gap && !out.is_empty() {
                out.push(' ');
            }
            gap = false;
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility_is_stripped_once() {
        assert_eq!(strip_vis("pub fn a()"), "fn a()");
        assert_eq!(strip_vis("pub(crate) fn a()"), "fn a()");
        assert_eq!(strip_vis("pub(in crate::x) fn a()"), "fn a()");
        assert_eq!(strip_vis("fn public()"), "fn public()");
        assert_eq!(strip_vis("pubfn"), "pubfn");
    }

    #[test]
    fn whitespace_folds() {
        assert_eq!(one_line("  a\n\t b  c "), "a b c");
    }
}
