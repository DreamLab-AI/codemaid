---
codemaid: 1
source: crates/codemaid-mermaid/src/er.rs
module: codemaid_mermaid::er
language: rust
source_hash: blake3:025fe739fbeffe290cde8f7da856e103e7450fab5a34327a0ab939f2dd242005
lines: 202
fragments: 5
---
# `codemaid_mermaid::er` · crates/codemaid-mermaid/src/er.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_mermaid__er__Attr["Attr"] {
    <<struct>>
    -ty: String
    -name: String
    -key: Option#lt;&'static str#gt;
    -comment: Option#lt;String#gt;
  }
  class codemaid_mermaid__er__Cardinality["Cardinality"] {
    <<enum>>
    One
    ZeroOrOne
    Many
    OneOrMore
    -left(self) &'static str
    -right(self) &'static str
  }
  class codemaid_mermaid__er__Entity["Entity"] {
    <<struct>>
    -id: Ident
    -label: String
    -attrs: Vec#lt;Attr#gt;
  }
  class codemaid_mermaid__er__ErDiagram["ErDiagram"] {
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
  class codemaid_mermaid__er__Rel["Rel"] {
    <<struct>>
    -from: Ident
    -from_card: Cardinality
    -to: Ident
    -to_card: Cardinality
    -identifying: bool
    -label: String
  }
  class codemaid_mermaid__er["codemaid_mermaid::er"] {
    <<module>>
    -squeeze_type(ty: &str) String
  }
  class codemaid_mermaid__escape__Ident["Ident"] {
    <<struct in crates/codemaid-mermaid/src/escape.rs>>
  }
  codemaid_mermaid__er__Entity o-- codemaid_mermaid__er__Attr : attrs
  codemaid_mermaid__er__Entity *-- codemaid_mermaid__escape__Ident : id
  codemaid_mermaid__er__ErDiagram ..> codemaid_mermaid__er__Cardinality
  codemaid_mermaid__er__ErDiagram o-- codemaid_mermaid__er__Entity : entities
  codemaid_mermaid__er__ErDiagram o-- codemaid_mermaid__er__Rel : rels
  codemaid_mermaid__er__ErDiagram ..> codemaid_mermaid__escape__Ident
  codemaid_mermaid__er__Rel *-- codemaid_mermaid__er__Cardinality : from_card, to_card
  codemaid_mermaid__er__Rel *-- codemaid_mermaid__escape__Ident : from, to
```

## `codemaid_mermaid::er::ErDiagram::entity`
`pub fn entity(&mut self, id: Ident, label: &str) -> &mut Self` · L95-L101
> Declare an entity.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__er__ErDiagram as ErDiagram
  participant codemaid_mermaid__escape as escape mod
  opt !self.entities.iter().any(| e | e.id == id)
    codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__escape: escape_text(label)
  end
```

## `codemaid_mermaid::er::ErDiagram::attr`
`pub fn attr(&mut self, entity: &Ident, ty: &str, name: &str, key: Option<&str>) -> &mut Self` · L103-L119
> Add an attribute to an existing entity (no-op if `entity` is unknown).
```mermaid
sequenceDiagram
  participant codemaid_mermaid__er__ErDiagram as ErDiagram
  participant codemaid_mermaid__er as er mod
  participant codemaid_mermaid__escape as escape mod
  participant codemaid_mermaid__escape__Ident as Ident
  codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__er: squeeze_type(ty)
  opt via then
    codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__escape: escape_text(ty)
  end
  opt let Some(e) = self.entities.iter_mut().find(| e | &e…
    codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__escape__Ident: Ident::new(name)
  end
```

## `codemaid_mermaid::er::ErDiagram::relation`
`pub fn relation(&mut self, from: &Ident, from_card: Cardinality, to: &Ident, to_card: Cardinality, identifying: bool, label: &str,) -> &mut Self` · L121-L137
> Add a relationship.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__er__ErDiagram as ErDiagram
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__escape: escape_text(label)
```

## `codemaid_mermaid::er::ErDiagram::render`
`pub fn render(&self) -> String` · L144-L182
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__er__ErDiagram as ErDiagram
  participant codemaid_mermaid__writer__CodeWriter as CodeWriter
  participant codemaid_mermaid__er__Cardinality as Cardinality
  codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: CodeWriter::new()
  codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: line(#quot;erDiagram#quot;)
  codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: indented(|..|)
  opt via indented
    loop for e in &self.entities
      opt e.attrs.is_empty()
        codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: line(head)
      end
      codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: line(_)
      codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: indented(|..|)
      opt via indented
        loop for a in &e.attrs
          codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: line(line)
        end
      end
      codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: line(#quot;}#quot;)
    end
    loop for r in &self.rels
      codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__er__Cardinality: ~left()
      codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__er__Cardinality: ~right()
      codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: line(_)
    end
  end
  codemaid_mermaid__er__ErDiagram->>codemaid_mermaid__writer__CodeWriter: finish()
```
