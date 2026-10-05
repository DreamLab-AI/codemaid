---
sealmap: 1
source: crates/sealmap-mermaid/src/flowchart.rs
module: sealmap_mermaid::flowchart
language: rust
source_hash: blake3:4393e0731142da67b3b6cb0c8c279774c066e56f69595011e7b6cdf840a4e764
lines: 185
fragments: 4
---
# `sealmap_mermaid::flowchart` · crates/sealmap-mermaid/src/flowchart.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__flowchart__EdgeStyle["EdgeStyle"] {
    <<enum>>
    Solid
    Dotted
    Thick
    Line
    -token(self) &'static str
  }
  class sealmap_mermaid__flowchart__Flowchart["Flowchart"] {
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
  class sealmap_mermaid__flowchart__NodeShape["NodeShape"] {
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
  class sealmap_mermaid__flowchart__Stmt["Stmt"] {
    <<enum>>
    Node#123; id: Ident, label: String, shape: NodeShape #125;
    Edge#123; from: Ident, to: Ident, style: EdgeStyle, label: Opti…
    Subgraph#123; id: Ident, label: String, direction: Option#lt;Direction…
  }
  class sealmap_mermaid__class__Direction["Direction"] {
    <<enum in crates/sealmap-mermaid/src/class.rs>>
  }
  class sealmap_mermaid__escape__Ident["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
  }
  sealmap_mermaid__flowchart__Flowchart *-- sealmap_mermaid__class__Direction : direction
  sealmap_mermaid__flowchart__Flowchart ..> sealmap_mermaid__escape__Ident
  sealmap_mermaid__flowchart__Flowchart ..> sealmap_mermaid__flowchart__EdgeStyle
  sealmap_mermaid__flowchart__Flowchart ..> sealmap_mermaid__flowchart__NodeShape
  sealmap_mermaid__flowchart__Flowchart o-- sealmap_mermaid__flowchart__Stmt : body
  sealmap_mermaid__flowchart__Stmt o-- sealmap_mermaid__class__Direction : Subgraph
  sealmap_mermaid__flowchart__Stmt o-- sealmap_mermaid__escape__Ident : Node, Edge, Subgraph
  sealmap_mermaid__flowchart__Stmt *-- sealmap_mermaid__flowchart__EdgeStyle : Edge
  sealmap_mermaid__flowchart__Stmt *-- sealmap_mermaid__flowchart__NodeShape : Node
```

## `sealmap_mermaid::flowchart::Flowchart::node`
`pub fn node(&mut self, id: Ident, label: &str, shape: NodeShape) -> &mut Self` · L108-L112
> Declare a node.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__flowchart__Flowchart as Flowchart
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__flowchart__Flowchart->>sealmap_mermaid__escape: escape_text(label)
```

## `sealmap_mermaid::flowchart::Flowchart::subgraph`
`pub fn subgraph(&mut self, id: Ident, label: &str, direction: Option<Direction>, f: impl FnOnce(&mut Flowchart),) -> &mut Self` · L125-L137
> Add a subgraph; `f` fills it.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__flowchart__Flowchart as Flowchart
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__flowchart__Flowchart->>sealmap_mermaid__flowchart__Flowchart: Flowchart::new(self.direction)
  sealmap_mermaid__flowchart__Flowchart->>sealmap_mermaid__escape: escape_text(label)
```

## `sealmap_mermaid::flowchart::Flowchart::render`
`pub fn render(&self) -> String` · L153-L184
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__flowchart__Flowchart as Flowchart
  participant sealmap_mermaid__writer__CodeWriter as CodeWriter
  participant sealmap_mermaid__class__Direction as Direction
  sealmap_mermaid__flowchart__Flowchart->>sealmap_mermaid__writer__CodeWriter: CodeWriter::new()
  sealmap_mermaid__flowchart__Flowchart->>sealmap_mermaid__class__Direction: as_str()
  sealmap_mermaid__flowchart__Flowchart->>sealmap_mermaid__writer__CodeWriter: line(_)
  sealmap_mermaid__flowchart__Flowchart->>sealmap_mermaid__writer__CodeWriter: indented(|..|)
  sealmap_mermaid__flowchart__Flowchart->>sealmap_mermaid__writer__CodeWriter: finish()
```
