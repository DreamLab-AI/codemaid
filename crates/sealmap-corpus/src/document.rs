//! One Markdown document per source file.

use std::fmt::Write as _;

use sealmap_model::{Codebase, ContentHash, SourceFile, SourcePath, Span, Symbol, SymbolKind};

use crate::index::{CallRef, DocumentEntry, FragmentEntry, FragmentKind};
use crate::{CorpusOptions, Lookup, document_path, sequence, structure};

/// Header line every generated document starts with; `write` only deletes
/// files that carry it.
pub(crate) const MARKER: &str = "sealmap: ";

pub(crate) fn render(
    cb: &Codebase,
    lookup: &Lookup<'_>,
    file: &SourceFile,
    opts: &CorpusOptions,
) -> (SourcePath, String, DocumentEntry) {
    let doc_path = document_path(&file.path);
    let mut fragments = Vec::new();
    let mut body = String::new();

    let module = cb.symbol(&file.module);
    let _ = writeln!(body, "# {} · {}", code(&file.module.to_string()), file.path);
    if let Some(doc) = module.and_then(|m| m.doc.as_deref()) {
        let _ = writeln!(body, "> {}", inline(doc));
    }
    if module.is_some_and(sealmap_model::Symbol::is_unparsable) {
        let _ = writeln!(body, "> ⚠ file did not parse; only its module is listed");
    }

    if let Some((text, _)) = structure::render(cb, lookup, file, opts) {
        let participants = lookup
            .in_file(&file.path)
            .filter(|s| s.kind.is_type() || s.kind == SymbolKind::Module)
            .map(|s| s.id.clone())
            .collect();
        let _ = writeln!(body, "\n## structure\n```mermaid\n{text}```");
        fragments.push(FragmentEntry {
            id: format!("structure:{}", file.path),
            kind: FragmentKind::Structure,
            document: doc_path.clone(),
            symbol: file.module.clone(),
            span: module.map(|m| m.span).unwrap_or_default(),
            sig_hash: module.map(|m| m.sig_hash).unwrap_or_default(),
            body_hash: module.map(|m| m.body_hash).unwrap_or_default(),
            participants,
            calls: Vec::new(),
            truncated: 0,
            hash: ContentHash::of_text(&text),
        });
    }

    let mut callables: Vec<&Symbol> =
        lookup.in_file(&file.path).filter(|s| s.kind.is_callable() && s.flow.is_some()).collect();
    // Source order reads better than id order inside one file.
    callables.sort_by_key(|s| (s.span.start_line, s.span.start_col, s.id.clone()));
    for sym in callables {
        let Some(r) = sequence::render(cb, sym, opts) else { continue };
        let _ = writeln!(body, "\n## {}", code(&sym.id.to_string()));
        let mut meta = String::new();
        if let Some(sig) = &sym.signature {
            let _ = write!(meta, "`{}` · ", sig.replace('`', "'"));
        }
        let _ = write!(meta, "{}", span_text(sym.span));
        let _ = writeln!(body, "{meta}");
        if let Some(doc) = &sym.doc {
            let _ = writeln!(body, "> {}", inline(doc));
        }
        let _ = writeln!(body, "```mermaid\n{}```", r.text);
        let calls = sym
            .flow
            .as_ref()
            .map(|f| {
                f.calls()
                    .map(|c| CallRef {
                        target: c.target.clone(),
                        confidence: c.confidence,
                        line: c.line,
                        expands: None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        fragments.push(FragmentEntry {
            id: sym.id.to_string(),
            kind: FragmentKind::Sequence,
            document: doc_path.clone(),
            symbol: sym.id.clone(),
            span: sym.span,
            sig_hash: sym.sig_hash,
            body_hash: sym.body_hash,
            participants: r.participants,
            calls,
            truncated: r.truncated,
            hash: ContentHash::of_text(&r.text),
        });
    }

    let mut text = String::new();
    let _ = writeln!(text, "---");
    let _ = writeln!(text, "{MARKER}{}", crate::CORPUS_SCHEMA_VERSION);
    let _ = writeln!(text, "source: {}", file.path);
    let _ = writeln!(text, "module: {}", yaml_str(&file.module.to_string()));
    let _ = writeln!(text, "language: {}", file.language);
    let _ = writeln!(text, "source_hash: {}", file.hash);
    let _ = writeln!(text, "lines: {}", file.lines);
    let _ = writeln!(text, "fragments: {}", fragments.len());
    let _ = writeln!(text, "---");
    text.push_str(&body);

    let entry = DocumentEntry {
        source: file.path.clone(),
        document: doc_path.clone(),
        module: file.module.clone(),
        source_hash: file.hash.clone(),
        document_hash: ContentHash::of_text(&text),
        fragments,
    };
    (doc_path, text, entry)
}

/// A double-quoted YAML scalar (JSON string syntax is valid YAML), so ids
/// holding `: ` or ` #` cannot be misread as structure or comments.
fn yaml_str(text: &str) -> String {
    serde_json::to_string(text).expect("strings always serialise")
}

/// Inline Markdown code for `text`, with a longer fence when the text itself
/// holds backticks (quoted `sym:` names do).
fn code(text: &str) -> String {
    if text.contains('`') { format!("`` {text} ``") } else { format!("`{text}`") }
}

/// Make free text safe on one Markdown line (no fences, no newlines).
fn inline(text: &str) -> String {
    text.replace("```", "'''").replace(['\n', '\r'], " ")
}

fn span_text(s: Span) -> String {
    s.compact()
}

/// Read the `source_hash` from a document's front matter.
pub(crate) fn front_matter_hash(text: &str) -> Option<ContentHash> {
    let fm = text.strip_prefix("---\n")?;
    let end = fm.find("\n---")?;
    fm[..end].lines().find_map(|l| l.strip_prefix("source_hash: ")).and_then(ContentHash::parse)
}

/// `true` if `text` is a sealmap-generated document.
pub(crate) fn is_generated(text: &str) -> bool {
    text.strip_prefix("---\n").is_some_and(|t| t.starts_with(MARKER))
}
