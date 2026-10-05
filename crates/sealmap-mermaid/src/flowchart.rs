use crate::class::Direction;
use crate::escape::{Ident, escape_text};
use crate::writer::CodeWriter;

/// Node shapes. Sealmap uses shape to encode kind, so an agent (or a human)
/// can tell a module from a crate from a datastore at a glance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum NodeShape {
    /// `["…"]` rectangle.
    #[default]
    Rect,
    /// `("…")` rounded rectangle.
    Round,
    /// `(["…"])` stadium.
    Stadium,
    /// `[["…"]]` subroutine.
    Subroutine,
    /// `[("…")]` cylinder (datastore).
    Cylinder,
    /// `{{"…"}}` hexagon.
    Hexagon,
    /// `(("…"))` circle.
    Circle,
    /// `{"…"}` rhombus (decision).
    Rhombus,
}

impl NodeShape {
    fn wrap(self, label: &str) -> String {
        let (l, r) = match self {
            Self::Rect => ("[", "]"),
            Self::Round => ("(", ")"),
            Self::Stadium => ("([", "])"),
            Self::Subroutine => ("[[", "]]"),
            Self::Cylinder => ("[(", ")]"),
            Self::Hexagon => ("{{", "}}"),
            Self::Circle => ("((", "))"),
            Self::Rhombus => ("{", "}"),
        };
        format!("{l}\"{label}\"{r}")
    }
}

/// Edge line styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum EdgeStyle {
    /// `-->`.
    #[default]
    Solid,
    /// `-.->`.
    Dotted,
    /// `==>`.
    Thick,
    /// `---` (no arrowhead).
    Line,
}

impl EdgeStyle {
    fn token(self) -> &'static str {
        match self {
            Self::Solid => "-->",
            Self::Dotted => "-.->",
            Self::Thick => "==>",
            Self::Line => "---",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Stmt {
    Node { id: Ident, label: String, shape: NodeShape },
    Edge { from: Ident, to: Ident, style: EdgeStyle, label: Option<String> },
    Subgraph { id: Ident, label: String, direction: Option<Direction>, body: Vec<Stmt> },
}

/// A Mermaid `flowchart`, with nested subgraphs.
///
/// Used for module/crate/repository dependency maps, call graphs and data
/// flow. Statements render in insertion order.
///
/// ```
/// use sealmap_mermaid::{EdgeStyle, Flowchart, Direction, Ident, NodeShape};
///
/// let (a, b) = (Ident::from_path("app::net"), Ident::from_path("app::db"));
/// let mut f = Flowchart::new(Direction::LR);
/// f.subgraph(Ident::new("app"), "app", None, |g| {
///     g.node(a.clone(), "net", NodeShape::Rect);
///     g.node(b.clone(), "db", NodeShape::Cylinder);
/// });
/// f.edge(&a, &b, EdgeStyle::Solid, Some("3 calls"));
/// assert_eq!(
///     f.render(),
///     "flowchart LR\n  subgraph app[\"app\"]\n    app__net[\"net\"]\n    app__db[(\"db\")]\n  end\n  app__net -->|\"3 calls\"| app__db\n"
/// );
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Flowchart {
    direction: Direction,
    body: Vec<Stmt>,
}

impl Flowchart {
    /// An empty flowchart.
    pub fn new(direction: Direction) -> Self {
        Self { direction, body: Vec::new() }
    }

    /// Declare a node.
    pub fn node(&mut self, id: Ident, label: &str, shape: NodeShape) -> &mut Self {
        self.body.push(Stmt::Node { id, label: escape_text(label), shape });
        self
    }

    /// Add an edge.
    pub fn edge(&mut self, from: &Ident, to: &Ident, style: EdgeStyle, label: Option<&str>) -> &mut Self {
        self.body.push(Stmt::Edge {
            from: from.clone(),
            to: to.clone(),
            style,
            label: label.map(escape_text).filter(|l| !l.is_empty()),
        });
        self
    }

    /// Add a subgraph; `f` fills it.
    pub fn subgraph(
        &mut self,
        id: Ident,
        label: &str,
        direction: Option<Direction>,
        f: impl FnOnce(&mut Flowchart),
    ) -> &mut Self {
        let mut inner = Flowchart::new(self.direction);
        f(&mut inner);
        self.body.push(Stmt::Subgraph { id, label: escape_text(label), direction, body: inner.body });
        self
    }

    /// Number of nodes, recursively.
    pub fn node_count(&self) -> usize {
        fn count(b: &[Stmt]) -> usize {
            b.iter()
                .map(|s| match s {
                    Stmt::Node { .. } => 1,
                    Stmt::Edge { .. } => 0,
                    Stmt::Subgraph { body, .. } => count(body),
                })
                .sum()
        }
        count(&self.body)
    }

    /// Render to Mermaid text.
    pub fn render(&self) -> String {
        fn write(body: &[Stmt], w: &mut CodeWriter) {
            for s in body {
                match s {
                    Stmt::Node { id, label, shape } => {
                        w.line(format!("{id}{}", shape.wrap(label)));
                    }
                    Stmt::Edge { from, to, style, label } => {
                        match label {
                            Some(l) => w.line(format!("{from} {}|\"{l}\"| {to}", style.token())),
                            None => w.line(format!("{from} {} {to}", style.token())),
                        };
                    }
                    Stmt::Subgraph { id, label, direction, body } => {
                        w.line(format!("subgraph {id}[\"{label}\"]"));
                        w.indented(|w| {
                            if let Some(d) = direction {
                                w.line(format!("direction {}", d.as_str()));
                            }
                            write(body, w);
                        });
                        w.line("end");
                    }
                }
            }
        }
        let mut w = CodeWriter::new();
        w.line(format!("flowchart {}", self.direction.as_str()));
        w.indented(|w| write(&self.body, w));
        w.finish()
    }
}
