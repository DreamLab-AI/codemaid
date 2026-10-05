---
sealmap: 1
source: crates/sealmap-mermaid/src/er.rs
module: sealmap_mermaid::er
language: rust
source_hash: blake3:1a3a5d12a512cf2b3af9b898247462778af3d03833cc09b5fc5e343e3b6215ba
lines: 202
fragments: 5
---
# `sealmap_mermaid::er` · crates/sealmap-mermaid/src/er.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__er__Attr["Attr"] {
    <<struct>>
    -ty: String
    -name: String
    -key: Option#lt;&'static str#gt;
    -comment: Option#lt;String#gt;
  }
  class sealmap_mermaid__er__Cardinality["Cardinality"] {
    <<enum>>
    One
    ZeroOrOne
    Many
    OneOrMore
    -left(self) &'static str
    -right(self) &'static str
  }
  class sealmap_mermaid__er__Entity["Entity"] {
    <<struct>>
    -id: Ident
    -label: String
    -attrs: Vec#lt;Attr#gt;
  }
  class sealmap_mermaid__er__ErDiagram["ErDiagram"] {
    <<struct>>
    -entities: Vec#lt;Entity#gt;
    -rels: Vec#lt;Rel#gt;
    +attr(&mut self, entity: &Ident, ty: &str, name: &str, key: Option#lt;&str#gt;) &mut Self
    +entity(&mut self, id: Ident, label: &str) &mut Self
    +entity_count(&self) usize
    +new() Self
    +relation(&mut self, from: &Ident, from_card: Cardinality, to: &Ident, to_card: Cardinality, identifying: bool, label: &str,) &mut Self
    +render(&self) String
  }
  class sealmap_mermaid__er__Rel["Rel"] {
    <<struct>>
    -from: Ident
    -from_card: Cardinality
    -to: Ident
    -to_card: Cardinality
    -identifying: bool
    -label: String
  }
  class sealmap_mermaid__er["sealmap_mermaid::er"] {
    <<module>>
    -squeeze_type(ty: &str) String
  }
  class sealmap_mermaid__escape__Ident["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
  }
  sealmap_mermaid__er__Entity o-- sealmap_mermaid__er__Attr : attrs
  sealmap_mermaid__er__Entity *-- sealmap_mermaid__escape__Ident : id
  sealmap_mermaid__er__ErDiagram ..> sealmap_mermaid__er__Cardinality
  sealmap_mermaid__er__ErDiagram o-- sealmap_mermaid__er__Entity : entities
  sealmap_mermaid__er__ErDiagram o-- sealmap_mermaid__er__Rel : rels
  sealmap_mermaid__er__ErDiagram ..> sealmap_mermaid__escape__Ident
  sealmap_mermaid__er__Rel *-- sealmap_mermaid__er__Cardinality : from_card, to_card
  sealmap_mermaid__er__Rel *-- sealmap_mermaid__escape__Ident : from, to
```

## `sealmap_mermaid::er::ErDiagram::entity`
`pub fn entity(&mut self, id: Ident, label: &str) -> &mut Self` · L95-L101
> Declare an entity.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__er__ErDiagram as ErDiagram
  participant sealmap_mermaid__escape as escape mod
  opt !self.entities.iter().any(| e | e.id == id)
    sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__escape: escape_text(label)
  end
```

## `sealmap_mermaid::er::ErDiagram::attr`
`pub fn attr(&mut self, entity: &Ident, ty: &str, name: &str, key: Option<&str>) -> &mut Self` · L103-L119
> Add an attribute to an existing entity (no-op if `entity` is unknown).
```mermaid
sequenceDiagram
  participant sealmap_mermaid__er__ErDiagram as ErDiagram
  participant sealmap_mermaid__er as er mod
  participant sealmap_mermaid__escape as escape mod
  participant sealmap_mermaid__escape__Ident as Ident
  sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__er: squeeze_type(ty)
  opt via then
    sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__escape: escape_text(ty)
  end
  opt let Some(e) = self.entities.iter_mut().find(| e | &e…
    sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__escape__Ident: Ident::new(name)
  end
```

## `sealmap_mermaid::er::ErDiagram::relation`
`pub fn relation(&mut self, from: &Ident, from_card: Cardinality, to: &Ident, to_card: Cardinality, identifying: bool, label: &str,) -> &mut Self` · L121-L137
> Add a relationship.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__er__ErDiagram as ErDiagram
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__escape: escape_text(label)
```

## `sealmap_mermaid::er::ErDiagram::render`
`pub fn render(&self) -> String` · L144-L182
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__er__ErDiagram as ErDiagram
  participant sealmap_mermaid__writer__CodeWriter as CodeWriter
  participant sealmap_mermaid__er__Cardinality as Cardinality
  sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: CodeWriter::new()
  sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: line(#quot;erDiagram#quot;)
  sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: indented(|..|)
  opt via indented
    loop for e in &self.entities
      opt e.attrs.is_empty()
        sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: line(head)
      end
      sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: line(_)
      sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: indented(|..|)
      opt via indented
        loop for a in &e.attrs
          sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: line(line)
        end
      end
      sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: line(#quot;}#quot;)
    end
    loop for r in &self.rels
      sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__er__Cardinality: ~left()
      sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__er__Cardinality: ~right()
      sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: line(_)
    end
  end
  sealmap_mermaid__er__ErDiagram->>sealmap_mermaid__writer__CodeWriter: finish()
```
