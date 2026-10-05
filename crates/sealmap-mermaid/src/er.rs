use crate::escape::{Ident, escape_text};
use crate::writer::CodeWriter;

/// One side of an ER relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cardinality {
    /// Exactly one (`||`).
    One,
    /// Zero or one (`|o` / `o|`), e.g. `Option<T>`.
    ZeroOrOne,
    /// Zero or more (`}o` / `o{`), e.g. `Vec<T>`, `HashMap<K, T>`.
    Many,
    /// One or more (`}|` / `|{`).
    OneOrMore,
}

impl Cardinality {
    fn left(self) -> &'static str {
        match self {
            Self::One => "||",
            Self::ZeroOrOne => "|o",
            Self::Many => "}o",
            Self::OneOrMore => "}|",
        }
    }
    fn right(self) -> &'static str {
        match self {
            Self::One => "||",
            Self::ZeroOrOne => "o|",
            Self::Many => "o{",
            Self::OneOrMore => "|{",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Attr {
    ty: String,
    name: String,
    key: Option<&'static str>,
    comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entity {
    id: Ident,
    label: String,
    attrs: Vec<Attr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rel {
    from: Ident,
    from_card: Cardinality,
    to: Ident,
    to_card: Cardinality,
    identifying: bool,
    label: String,
}

/// A Mermaid `erDiagram`, used for data-model views (structs as entities,
/// fields as attributes, field types as relationships with cardinality).
///
/// Attribute types must be single tokens in Mermaid's grammar, so they are
/// squeezed into one (`Vec<String>` → `Vec[String]`); when that loses
/// information the original is kept as the attribute comment.
///
/// ```
/// use sealmap_mermaid::{Cardinality, ErDiagram, Ident};
///
/// let (user, order) = (Ident::new("User"), Ident::new("Order"));
/// let mut er = ErDiagram::new();
/// er.entity(user.clone(), "User")
///     .attr(&user, "u64", "id", Some("PK"))
///     .attr(&user, "Vec<Order>", "orders", None);
/// er.entity(order.clone(), "Order");
/// er.relation(&user, Cardinality::One, &order, Cardinality::Many, true, "orders");
/// assert_eq!(
///     er.render(),
///     "erDiagram\n  User {\n    u64 id PK\n    Vec[Order] orders\n  }\n  Order\n  User ||--o{ Order : \"orders\"\n"
/// );
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ErDiagram {
    entities: Vec<Entity>,
    rels: Vec<Rel>,
}

impl ErDiagram {
    /// An empty diagram.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare an entity. Duplicate ids are ignored.
    pub fn entity(&mut self, id: Ident, label: &str) -> &mut Self {
        if !self.entities.iter().any(|e| e.id == id) {
            self.entities.push(Entity { id, label: escape_text(label), attrs: Vec::new() });
        }
        self
    }

    /// Add an attribute to an existing entity (no-op if `entity` is unknown).
    /// `key` may be `PK`, `FK` or `UK`.
    pub fn attr(&mut self, entity: &Ident, ty: &str, name: &str, key: Option<&str>) -> &mut Self {
        let key = match key {
            Some("PK") => Some("PK"),
            Some("FK") => Some("FK"),
            Some("UK") => Some("UK"),
            _ => None,
        };
        let squeezed = squeeze_type(ty);
        let lossless = ty.replace(' ', "").replace('<', "[").replace('>', "]");
        let comment = (squeezed != lossless).then(|| escape_text(ty));
        if let Some(e) = self.entities.iter_mut().find(|e| &e.id == entity) {
            e.attrs.push(Attr { ty: squeezed, name: Ident::new(name).as_str().to_owned(), key, comment });
        }
        self
    }

    /// Add a relationship. Duplicates are ignored.
    pub fn relation(
        &mut self,
        from: &Ident,
        from_card: Cardinality,
        to: &Ident,
        to_card: Cardinality,
        identifying: bool,
        label: &str,
    ) -> &mut Self {
        let rel =
            Rel { from: from.clone(), from_card, to: to.clone(), to_card, identifying, label: escape_text(label) };
        if !self.rels.contains(&rel) {
            self.rels.push(rel);
        }
        self
    }

    /// Number of entities.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Render to Mermaid text.
    pub fn render(&self) -> String {
        let mut w = CodeWriter::new();
        w.line("erDiagram");
        w.indented(|w| {
            for e in &self.entities {
                let head = if e.label == e.id.as_str() || e.label.is_empty() {
                    e.id.to_string()
                } else {
                    format!("{}[\"{}\"]", e.id, e.label)
                };
                if e.attrs.is_empty() {
                    w.line(head);
                    continue;
                }
                w.line(format!("{head} {{"));
                w.indented(|w| {
                    for a in &e.attrs {
                        let mut line = format!("{} {}", a.ty, a.name);
                        if let Some(k) = a.key {
                            line.push(' ');
                            line.push_str(k);
                        }
                        if let Some(c) = &a.comment {
                            line.push_str(&format!(" \"{c}\""));
                        }
                        w.line(line);
                    }
                });
                w.line("}");
            }
            for r in &self.rels {
                let link = if r.identifying { "--" } else { ".." };
                let label = if r.label.is_empty() { "_".to_owned() } else { r.label.clone() };
                w.line(format!("{} {}{link}{} {} : \"{label}\"", r.from, r.from_card.left(), r.to_card.right(), r.to));
            }
        });
        w.finish()
    }
}

/// Squeeze a type into Mermaid's ER attribute-type token
/// (`[A-Za-z_][A-Za-z0-9_\-\[\]()]*`).
fn squeeze_type(ty: &str) -> String {
    let mut out = String::with_capacity(ty.len());
    for c in ty.chars() {
        match c {
            '<' => out.push('['),
            '>' => out.push(']'),
            c if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '[' | ']' | '(' | ')') => out.push(c),
            ' ' => {}
            _ => out.push('_'),
        }
    }
    if out.is_empty() || !out.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
        out.insert(0, '_');
    }
    out
}
