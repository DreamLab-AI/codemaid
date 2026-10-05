//! Per-file `classDiagram`: what the file defines and how it relates to the
//! rest of the codebase.

use std::collections::BTreeSet;

use sealmap_mermaid::{Class, ClassDiagram, ClassRelationKind, Direction};
use sealmap_model::{Codebase, MemberKind, RelationKind, SourceFile, Symbol, SymbolId, SymbolKind, Visibility};

use crate::naming::ident;
use crate::{CorpusOptions, Lookup};

/// Render the structure diagram for `file`. Returns `None` when the file
/// defines nothing worth drawing.
pub(crate) fn render(
    cb: &Codebase,
    lookup: &Lookup<'_>,
    file: &SourceFile,
    opts: &CorpusOptions,
) -> Option<(String, usize)> {
    let visible = |s: &Symbol| opts.include_private || s.visibility != Visibility::Private;
    let here: Vec<&Symbol> = lookup.in_file(&file.path).filter(|s| visible(s)).collect();

    let mut d = ClassDiagram::new(Direction::LR);
    let mut drawn: BTreeSet<SymbolId> = BTreeSet::new();

    // Types defined in this file, plus foreign types that this file adds
    // methods to (impl blocks), each with the methods defined here.
    let mut owners: Vec<SymbolId> = Vec::new();
    for s in &here {
        if s.kind.is_type() {
            owners.push(s.id.clone());
        }
    }
    for s in &here {
        if s.kind == SymbolKind::Method {
            if let Some(p) = &s.parent {
                if !owners.contains(p) {
                    owners.push(p.clone());
                }
            }
        }
    }
    for owner in &owners {
        let mut class = Class::new(ident(owner), &type_label(cb, owner));
        match cb.symbol(owner) {
            Some(t) if t.file == file.path => {
                class.annotation(t.kind.keyword());
                for m in &t.members {
                    match m.kind {
                        MemberKind::Field => {
                            class.field(m.visibility.uml_marker(), &m.name, m.ty.as_deref().unwrap_or(""));
                        }
                        MemberKind::Variant => {
                            class.field(' ', &variant_text(&m.name, m.ty.as_deref()), "");
                        }
                        MemberKind::AssocType => {
                            class.field('+', &format!("type {}", m.name), m.ty.as_deref().unwrap_or(""));
                        }
                        MemberKind::AssocConst => {
                            class.field('+', &format!("const {}", m.name), m.ty.as_deref().unwrap_or(""));
                        }
                        MemberKind::RequiredMethod => {
                            let (params, ret) = split_signature(m.ty.as_deref().unwrap_or(""));
                            class.method('+', &m.name, &params, &ret);
                        }
                    }
                }
            }
            Some(t) => {
                class.annotation(&format!("{} in {}", t.kind.keyword(), t.file));
            }
            None => {
                class.annotation("external");
            }
        }
        for m in here.iter().filter(|s| s.kind == SymbolKind::Method && s.parent.as_ref() == Some(owner)) {
            let (params, ret) = split_signature(m.signature.as_deref().unwrap_or(""));
            let name = match m.tags.first().and_then(|t| t.strip_prefix("impl ")) {
                Some(tr) => format!("{}::{}", short_trait(tr), m.name),
                None => m.name.clone(),
            };
            class.method(m.visibility.uml_marker(), &name, &params, &ret);
        }
        d.class(class);
        drawn.insert(owner.clone());
    }

    // The module box: free functions, constants, statics, macros, submodules.
    let modules: Vec<&Symbol> = here.iter().copied().filter(|s| s.kind == SymbolKind::Module).collect();
    for module in &modules {
        let mut class = Class::new(ident(&module.id), &module.id.display_path());
        class.annotation("module");
        let mut any = false;
        for s in lookup.children(&module.id).filter(|s| visible(s)) {
            match s.kind {
                SymbolKind::Module => {
                    class.field(s.visibility.uml_marker(), &format!("mod {}", s.name), "");
                }
                SymbolKind::Function => {
                    if s.file != file.path {
                        continue;
                    }
                    let (params, ret) = split_signature(s.signature.as_deref().unwrap_or(""));
                    class.method(s.visibility.uml_marker(), &s.name, &params, &ret);
                }
                SymbolKind::Const | SymbolKind::Static => {
                    if s.file != file.path {
                        continue;
                    }
                    let ty = s.signature.as_deref().and_then(|sig| sig.split_once(": ").map(|(_, t)| t)).unwrap_or("");
                    class.field(s.visibility.uml_marker(), &format!("{} {}", s.kind.keyword(), s.name), ty);
                }
                SymbolKind::Macro => {
                    if s.file != file.path {
                        continue;
                    }
                    class.field(s.visibility.uml_marker(), &format!("{}!", s.name), "");
                }
                _ => continue,
            }
            any = true;
        }
        // A module that only holds types is already fully described by them.
        if any || owners.is_empty() {
            d.class(class);
            drawn.insert(module.id.clone());
        }
    }
    if d.class_count() == 0 {
        return None;
    }

    // Relations from what we drew (methods aggregate to their owner type).
    let mut edges: BTreeSet<(SymbolId, SymbolId, RelationKind, String)> = BTreeSet::new();
    for s in &here {
        let from = match s.kind {
            SymbolKind::Method => s.parent.clone(),
            k if k.is_type() => Some(s.id.clone()),
            SymbolKind::Function => s.parent.clone(),
            _ => None,
        };
        let Some(from) = from else { continue };
        if !drawn.contains(&from) {
            continue;
        }
        for r in cb.relations_from(&s.id) {
            let label = match r.kind {
                RelationKind::FieldType => s
                    .members
                    .iter()
                    .filter(|m| m.refs.contains(&r.to))
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => String::new(),
            };
            match r.kind {
                RelationKind::Implements | RelationKind::Extends | RelationKind::FieldType | RelationKind::Uses => {
                    edges.insert((from.clone(), r.to.clone(), r.kind, label));
                }
                _ => {}
            }
        }
    }
    // Implementations: `impl Trait for T` relations are stored from T.
    for owner in &owners {
        for r in cb.relations_from(owner).filter(|r| r.kind == RelationKind::Implements) {
            edges.insert((owner.clone(), r.to.clone(), r.kind, String::new()));
        }
    }
    // A field edge subsumes a uses edge between the same pair.
    let owned: BTreeSet<(SymbolId, SymbolId)> =
        edges.iter().filter(|e| e.2 == RelationKind::FieldType).map(|e| (e.0.clone(), e.1.clone())).collect();
    for (from, to, kind, label) in &edges {
        if from == to || (*kind == RelationKind::Uses && owned.contains(&(from.clone(), to.clone()))) {
            continue;
        }
        if !drawn.contains(to) {
            let mut stub = Class::new(ident(to), &type_label(cb, to));
            match cb.symbol(to) {
                Some(t) => {
                    stub.annotation(&format!("{} in {}", t.kind.keyword(), t.file));
                }
                None => {
                    stub.annotation("external");
                }
            }
            d.class(stub);
            drawn.insert(to.clone());
        }
        let k = match kind {
            RelationKind::Implements => ClassRelationKind::Realization,
            RelationKind::Extends => ClassRelationKind::Inheritance,
            RelationKind::FieldType if is_shared(cb, from, label) => ClassRelationKind::Aggregation,
            RelationKind::FieldType => ClassRelationKind::Composition,
            _ => ClassRelationKind::Dependency,
        };
        let label = (!label.is_empty()).then_some(label.as_str());
        d.relation(&ident(from), &ident(to), k, label);
    }
    let count = d.class_count();
    Some((d.render(), count))
}

/// Display label for a type: name with generics.
fn type_label(cb: &Codebase, id: &SymbolId) -> String {
    match cb.symbol(id) {
        Some(s) if !s.generics.is_empty() => {
            let g: Vec<&str> = s.generics.iter().map(|g| g.split(':').next().unwrap_or(g).trim()).collect();
            format!("{}<{}>", s.name, g.join(", "))
        }
        Some(s) => s.name.clone(),
        None => id.display_path(),
    }
}

fn variant_text(name: &str, ty: Option<&str>) -> String {
    match ty {
        Some(t) => format!("{name}{t}"),
        None => name.to_owned(),
    }
}

fn short_trait(t: &str) -> &str {
    let base = t.split('<').next().unwrap_or(t);
    let last = base.rsplit("::").next().unwrap_or(base);
    &t[t.find(last).unwrap_or(0)..]
}

/// Is the field holding the target behind `Option`, `Arc`, `Rc`, a reference
/// or a collection (aggregation rather than composition)?
fn is_shared(cb: &Codebase, from: &SymbolId, fields: &str) -> bool {
    let Some(s) = cb.symbol(from) else { return false };
    fields.split(", ").any(|f| {
        s.members.iter().find(|m| m.name == f).and_then(|m| m.ty.as_deref()).is_some_and(|ty| {
            ty.starts_with('&')
                || [
                    "Option<",
                    "Arc<",
                    "Rc<",
                    "Weak<",
                    "Vec<",
                    "HashMap<",
                    "BTreeMap<",
                    "HashSet<",
                    "BTreeSet<",
                    "VecDeque<",
                ]
                .iter()
                .any(|w| ty.contains(w))
        })
    })
}

/// Split `pub async fn name<T>(a: A, b: B) -> R where ..` into
/// (`a: A, b: B`, `R`).
pub(crate) fn split_signature(sig: &str) -> (String, String) {
    let Some(open) = sig.find('(') else { return (String::new(), String::new()) };
    let mut depth = 0usize;
    let mut close = None;
    for (i, c) in sig[open..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(close) = close else { return (sig[open + 1..].to_owned(), String::new()) };
    let params = sig[open + 1..close].to_owned();
    let rest = &sig[close + 1..];
    let ret = rest
        .split_once("->")
        .map(|(_, r)| r.split(" where ").next().unwrap_or(r).trim().to_owned())
        .unwrap_or_default();
    (params, ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_signatures() {
        assert_eq!(
            split_signature("pub async fn get<T>(&self, k: (u8, u8)) -> Option<T> where T: Clone"),
            ("&self, k: (u8, u8)".into(), "Option<T>".into())
        );
        assert_eq!(split_signature("fn f()"), (String::new(), String::new()));
    }
}
