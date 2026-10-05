---
sealmap: 2
source: crates/sealmap-mermaid/src/flowchart.rs
module: "sym:cargo sealmap_mermaid . flowchart/"
language: rust
source_hash: blake3:0d79132ed36593485a06875a3dde087f63e65d154cd6e65d1672bc0cc6c06bae
lines: 185
fragments: 4
---
# `sym:cargo sealmap_mermaid . flowchart/` · crates/sealmap-mermaid/src/flowchart.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__flowchart___tEdgeStyle["EdgeStyle"] {
    <<enum>>
    Solid
    Dotted
    Thick
    Line
    -token(self) &'static str
  }
  class sealmap_mermaid__flowchart___tFlowchart["Flowchart"] {
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
  class sealmap_mermaid__flowchart___tNodeShape["NodeShape"] {
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
  class sealmap_mermaid__flowchart___tStmt["Stmt"] {
    <<enum>>
    Node#123; id: Ident, label: String, shape: NodeShape #125;
    Edge#123; from: Ident, to: Ident, style: EdgeStyle, label: Opti…
    Subgraph#123; id: Ident, label: String, direction: Option#lt;Direction…
  }
  class sealmap_mermaid__class___tDirection["Direction"] {
    <<enum in crates/sealmap-mermaid/src/class.rs>>
  }
  class sealmap_mermaid__escape___tIdent["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
  }
  sealmap_mermaid__flowchart___tFlowchart *-- sealmap_mermaid__class___tDirection : direction
  sealmap_mermaid__flowchart___tFlowchart ..> sealmap_mermaid__escape___tIdent
  sealmap_mermaid__flowchart___tFlowchart ..> sealmap_mermaid__flowchart___tEdgeStyle
  sealmap_mermaid__flowchart___tFlowchart ..> sealmap_mermaid__flowchart___tNodeShape
  sealmap_mermaid__flowchart___tFlowchart o-- sealmap_mermaid__flowchart___tStmt : body
  sealmap_mermaid__flowchart___tStmt o-- sealmap_mermaid__class___tDirection : Subgraph
  sealmap_mermaid__flowchart___tStmt o-- sealmap_mermaid__escape___tIdent : Node, Edge, Subgraph
  sealmap_mermaid__flowchart___tStmt *-- sealmap_mermaid__flowchart___tEdgeStyle : Edge
  sealmap_mermaid__flowchart___tStmt *-- sealmap_mermaid__flowchart___tNodeShape : Node
```

## `sym:cargo sealmap_mermaid . flowchart/Flowchart#node().`
`pub fn node(&mut self, id: Ident, label: &str, shape: NodeShape) -> &mut Self` · L108-L112
> Declare a node.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__flowchart___tFlowchart as Flowchart
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__flowchart___tFlowchart->>sealmap_mermaid__escape: escape_text(label)
```

## `sym:cargo sealmap_mermaid . flowchart/Flowchart#subgraph().`
`pub fn subgraph(&mut self, id: Ident, label: &str, direction: Option<Direction>, f: impl FnOnce(&mut Flowchart),) -> &mut Self` · L125-L137
> Add a subgraph; `f` fills it.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__flowchart___tFlowchart as Flowchart
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__flowchart___tFlowchart->>sealmap_mermaid__flowchart___tFlowchart: Flowchart::new(self.direction)
  sealmap_mermaid__flowchart___tFlowchart->>sealmap_mermaid__escape: escape_text(label)
```

## `sym:cargo sealmap_mermaid . flowchart/Flowchart#render().`
`pub fn render(&self) -> String` · L153-L184
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__flowchart___tFlowchart as Flowchart
  participant sealmap_mermaid__writer___tCodeWriter as CodeWriter
  participant sealmap_mermaid__class___tDirection as Direction
  sealmap_mermaid__flowchart___tFlowchart->>sealmap_mermaid__writer___tCodeWriter: CodeWriter::new()
  sealmap_mermaid__flowchart___tFlowchart->>sealmap_mermaid__class___tDirection: as_str()
  sealmap_mermaid__flowchart___tFlowchart->>sealmap_mermaid__writer___tCodeWriter: line(_)
  sealmap_mermaid__flowchart___tFlowchart->>sealmap_mermaid__writer___tCodeWriter: indented(|..|)
  sealmap_mermaid__flowchart___tFlowchart->>sealmap_mermaid__writer___tCodeWriter: finish()
```
