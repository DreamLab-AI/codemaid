use crate::escape::{Ident, escape_text, escape_type};
use crate::writer::CodeWriter;

/// Layout direction for class diagrams and flowcharts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Direction {
    /// Top to bottom.
    #[default]
    TB,
    /// Bottom to top.
    BT,
    /// Left to right.
    LR,
    /// Right to left.
    RL,
}

impl Direction {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::TB => "TB",
            Self::BT => "BT",
            Self::LR => "LR",
            Self::RL => "RL",
        }
    }
}

/// One class box.
///
/// ```
/// use sealmap_mermaid::{Class, Ident};
///
/// let mut c = Class::new(Ident::from_path("app::Cache"), "Cache<K, V>");
/// c.annotation("struct")
///     .field('-', "map", "HashMap<K, V>")
///     .method('+', "get", "&self, k: &K", "Option<&V>");
/// assert_eq!(c.member_count(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    id: Ident,
    label: String,
    annotation: Option<String>,
    members: Vec<String>,
}

impl Class {
    /// A class with display `label` (generics may be written with `<>`).
    pub fn new(id: Ident, label: &str) -> Self {
        // Generic markers (`~`) are not interpreted in quoted labels, so angle
        // brackets become entity codes, which Mermaid renders as `<`/`>`.
        let label = escape_text(label).replace('<', "#lt;").replace('>', "#gt;");
        Self { id, label, annotation: None, members: Vec::new() }
    }

    /// The class id.
    pub fn id(&self) -> &Ident {
        &self.id
    }

    /// `<<annotation>>`, e.g. `trait`, `enum`, `external`.
    pub fn annotation(&mut self, text: &str) -> &mut Self {
        self.annotation = Some(escape_text(text));
        self
    }

    /// A field line: `<vis>name: Type`. `vis` is one of `+ - # ~` (anything
    /// else is dropped).
    pub fn field(&mut self, vis: char, name: &str, ty: &str) -> &mut Self {
        let vis = vis_marker(vis);
        let line = if ty.is_empty() {
            format!("{vis}{}", escape_type(name, true))
        } else {
            format!("{vis}{}: {}", escape_type(name, true), escape_type(ty, true))
        };
        self.members.push(line);
        self
    }

    /// A method line: `<vis>name(params) Ret`. Empty `ret` omits the return.
    pub fn method(&mut self, vis: char, name: &str, params: &str, ret: &str) -> &mut Self {
        let vis = vis_marker(vis);
        // Parentheses inside params would end the method early.
        let params = escape_type(params, true);
        let mut line = format!("{vis}{}({params})", escape_type(name, true));
        if !ret.is_empty() {
            line.push(' ');
            line.push_str(&escape_type(ret, true));
        }
        self.members.push(line);
        self
    }

    /// A raw, pre-escaped member line (no escaping applied). Use sparingly.
    pub fn raw_member(&mut self, line: &str) -> &mut Self {
        self.members.push(line.replace(['{', '}', '\n'], " "));
        self
    }

    /// Number of member lines.
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    fn write(&self, w: &mut CodeWriter) {
        let head = if self.label.is_empty() || self.label == self.id.as_str() {
            format!("class {}", self.id)
        } else {
            format!("class {}[\"{}\"]", self.id, self.label)
        };
        if self.annotation.is_none() && self.members.is_empty() {
            w.line(head);
            return;
        }
        w.line(format!("{head} {{"));
        w.indented(|w| {
            if let Some(a) = &self.annotation {
                w.line(format!("<<{a}>>"));
            }
            for m in &self.members {
                w.line(m);
            }
        });
        w.line("}");
    }
}

fn vis_marker(c: char) -> &'static str {
    match c {
        '+' => "+",
        '-' => "-",
        '#' => "#",
        '~' => "~",
        _ => "",
    }
}

/// Relationship kinds in a class diagram, named by UML meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClassRelationKind {
    /// `<|--` subtype / supertrait.
    Inheritance,
    /// `*--` owns (field by value).
    Composition,
    /// `o--` holds a shared or optional reference.
    Aggregation,
    /// `-->` association.
    Association,
    /// `..>` uses in a signature.
    Dependency,
    /// `..|>` implements an interface.
    Realization,
}

/// One edge. Render order is insertion order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassRelation {
    /// Source.
    pub from: Ident,
    /// Target.
    pub to: Ident,
    /// Kind.
    pub kind: ClassRelationKind,
    /// Optional label.
    pub label: Option<String>,
}

/// A Mermaid `classDiagram`.
///
/// ```
/// use sealmap_mermaid::{Class, ClassDiagram, ClassRelationKind, Direction, Ident};
///
/// let (store, backend) = (Ident::new("Store"), Ident::new("Backend"));
/// let mut d = ClassDiagram::new(Direction::LR);
/// d.class({ let mut c = Class::new(store.clone(), "Store"); c.field('-', "b", "Box<dyn Backend>"); c });
/// d.class({ let mut c = Class::new(backend.clone(), "Backend"); c.annotation("trait"); c });
/// d.relation(&store, &backend, ClassRelationKind::Composition, Some("b"));
/// assert_eq!(d.render(), "classDiagram\n  direction LR\n  class Store {\n    -b: Box#lt;dyn Backend#gt;\n  }\n  class Backend {\n    <<trait>>\n  }\n  Store *-- Backend : b\n");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClassDiagram {
    direction: Option<Direction>,
    classes: Vec<Class>,
    relations: Vec<ClassRelation>,
}

impl ClassDiagram {
    /// An empty diagram with the given direction.
    pub fn new(direction: Direction) -> Self {
        Self { direction: Some(direction), ..Self::default() }
    }

    /// Add a class. A class whose id already exists is ignored.
    pub fn class(&mut self, class: Class) -> &mut Self {
        if !self.classes.iter().any(|c| c.id == class.id) {
            self.classes.push(class);
        }
        self
    }

    /// `true` if a class with `id` was added.
    pub fn has_class(&self, id: &Ident) -> bool {
        self.classes.iter().any(|c| &c.id == id)
    }

    /// Add an edge. Exact duplicates are ignored.
    pub fn relation(&mut self, from: &Ident, to: &Ident, kind: ClassRelationKind, label: Option<&str>) -> &mut Self {
        let rel = ClassRelation {
            from: from.clone(),
            to: to.clone(),
            kind,
            label: label.map(escape_text).filter(|l| !l.is_empty()),
        };
        if !self.relations.contains(&rel) {
            self.relations.push(rel);
        }
        self
    }

    /// Number of classes.
    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    /// Number of relations.
    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    /// Render to Mermaid text.
    pub fn render(&self) -> String {
        let mut w = CodeWriter::new();
        w.line("classDiagram");
        w.indented(|w| {
            if let Some(d) = self.direction {
                w.line(format!("direction {}", d.as_str()));
            }
            for c in &self.classes {
                c.write(w);
            }
            for r in &self.relations {
                // Arrows are written so the head sits on the conceptual target.
                let (l, op, rr) = match r.kind {
                    ClassRelationKind::Inheritance => (&r.to, "<|--", &r.from),
                    ClassRelationKind::Composition => (&r.from, "*--", &r.to),
                    ClassRelationKind::Aggregation => (&r.from, "o--", &r.to),
                    ClassRelationKind::Association => (&r.from, "-->", &r.to),
                    ClassRelationKind::Dependency => (&r.from, "..>", &r.to),
                    ClassRelationKind::Realization => (&r.from, "..|>", &r.to),
                };
                match &r.label {
                    Some(label) => w.line(format!("{l} {op} {rr} : {label}")),
                    None => w.line(format!("{l} {op} {rr}")),
                };
            }
        });
        w.finish()
    }
}
