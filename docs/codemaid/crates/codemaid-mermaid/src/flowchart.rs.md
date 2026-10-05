---
codemaid: 1
source: crates/codemaid-mermaid/src/flowchart.rs
module: codemaid_mermaid::flowchart
language: rust
source_hash: blake3:29b2616b4dbc2cc4f3fed5367f4f7a08b47b3260d621291f4a7b5432177b5df4
lines: 185
fragments: 4
---
# `codemaid_mermaid::flowchart` · crates/codemaid-mermaid/src/flowchart.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_mermaid__flowchart__EdgeStyle["EdgeStyle"] {
    <<enum>>
    Solid
    Dotted
    Thick
    Line
    -token(self) &'static str
  }
  class codemaid_mermaid__flowchart__Flowchart["Flowchart"] {
    <<struct>>
    -direction: Direction
    -body: Vec#lt;Stmt#gt;
    +edge(&mut self, from: &Ident, to: &Ident, style: EdgeStyle, label: Option#lt;&str#gt;) &mut Self
    +new(direction: Direction) Self
    +node(&mut self, id: Ident, label: &str, shape: NodeShape) &mut Self
    +node_count(&self) usize
    +render(&self) String
    +subgraph(&mut self, id: Ident, label: &str, direction: Option#lt;Direction#gt;, f: impl FnOnce#40;&mut Flowchart#41;,) &mut Self
  }
  class codemaid_mermaid__flowchart__NodeShape["NodeShape"] {
    <<enum>>
    Rect
    Round
    Stadium
    Subroutine
    Cylinder
    Hexagon
    Circle
    Rhombus
    -wrap(self, label: &str) String
  }
  class codemaid_mermaid__flowchart__Stmt["Stmt"] {
    <<enum>>
    Node#123; id: Ident, label: String, shape: NodeShape #125;
    Edge#123; from: Ident, to: Ident, style: EdgeStyle, label: Opti…
    Subgraph#123; id: Ident, label: String, direction: Option#lt;Direction…
  }
  class codemaid_mermaid__class__Direction["Direction"] {
    <<enum in crates/codemaid-mermaid/src/class.rs>>
  }
  class codemaid_mermaid__escape__Ident["Ident"] {
    <<struct in crates/codemaid-mermaid/src/escape.rs>>
  }
  codemaid_mermaid__flowchart__Flowchart *-- codemaid_mermaid__class__Direction : direction
  codemaid_mermaid__flowchart__Flowchart ..> codemaid_mermaid__escape__Ident
  codemaid_mermaid__flowchart__Flowchart ..> codemaid_mermaid__flowchart__EdgeStyle
  codemaid_mermaid__flowchart__Flowchart ..> codemaid_mermaid__flowchart__NodeShape
  codemaid_mermaid__flowchart__Flowchart o-- codemaid_mermaid__flowchart__Stmt : body
  codemaid_mermaid__flowchart__Stmt o-- codemaid_mermaid__class__Direction : Subgraph
  codemaid_mermaid__flowchart__Stmt o-- codemaid_mermaid__escape__Ident : Node, Edge, Subgraph
  codemaid_mermaid__flowchart__Stmt *-- codemaid_mermaid__flowchart__EdgeStyle : Edge
  codemaid_mermaid__flowchart__Stmt *-- codemaid_mermaid__flowchart__NodeShape : Node
```

## `codemaid_mermaid::flowchart::Flowchart::node`
`pub fn node(&mut self, id: Ident, label: &str, shape: NodeShape) -> &mut Self` · L108-L112
> Declare a node.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__flowchart__Flowchart as Flowchart
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__flowchart__Flowchart->>codemaid_mermaid__escape: escape_text(label)
```

## `codemaid_mermaid::flowchart::Flowchart::subgraph`
`pub fn subgraph(&mut self, id: Ident, label: &str, direction: Option<Direction>, f: impl FnOnce(&mut Flowchart),) -> &mut Self` · L125-L137
> Add a subgraph; `f` fills it.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__flowchart__Flowchart as Flowchart
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__flowchart__Flowchart->>codemaid_mermaid__flowchart__Flowchart: Flowchart::new(self.direction)
  codemaid_mermaid__flowchart__Flowchart->>codemaid_mermaid__escape: escape_text(label)
```

## `codemaid_mermaid::flowchart::Flowchart::render`
`pub fn render(&self) -> String` · L153-L184
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__flowchart__Flowchart as Flowchart
  participant codemaid_mermaid__writer__CodeWriter as CodeWriter
  participant codemaid_mermaid__class__Direction as Direction
  codemaid_mermaid__flowchart__Flowchart->>codemaid_mermaid__writer__CodeWriter: CodeWriter::new()
  codemaid_mermaid__flowchart__Flowchart->>codemaid_mermaid__class__Direction: as_str()
  codemaid_mermaid__flowchart__Flowchart->>codemaid_mermaid__writer__CodeWriter: line(_)
  codemaid_mermaid__flowchart__Flowchart->>codemaid_mermaid__writer__CodeWriter: indented(|..|)
  codemaid_mermaid__flowchart__Flowchart->>codemaid_mermaid__writer__CodeWriter: finish()
```
