---
sealmap: 2
source: crates/sealmap-mermaid/src/er.rs
module: "sym:cargo sealmap_mermaid . er/"
language: rust
source_hash: blake3:1a3a5d12a512cf2b3af9b898247462778af3d03833cc09b5fc5e343e3b6215ba
lines: 202
fragments: 5
---
# `sym:cargo sealmap_mermaid . er/` · crates/sealmap-mermaid/src/er.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__er___tAttr["Attr"] {
    <<struct>>
    -ty: String
    -name: String
    -key: Option#lt;&'static str#gt;
    -comment: Option#lt;String#gt;
  }
  class sealmap_mermaid__er___tCardinality["Cardinality"] {
    <<enum>>
    One
    ZeroOrOne
    Many
    OneOrMore
    -left(self) &'static str
    -right(self) &'static str
  }
  class sealmap_mermaid__er___tEntity["Entity"] {
    <<struct>>
    -id: Ident
    -label: String
    -attrs: Vec#lt;Attr#gt;
  }
  class sealmap_mermaid__er___tErDiagram["ErDiagram"] {
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
  class sealmap_mermaid__er___tRel["Rel"] {
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
  class sealmap_mermaid__escape___tIdent["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
  }
  sealmap_mermaid__er___tEntity o-- sealmap_mermaid__er___tAttr : attrs
  sealmap_mermaid__er___tEntity *-- sealmap_mermaid__escape___tIdent : id
  sealmap_mermaid__er___tErDiagram ..> sealmap_mermaid__er___tCardinality
  sealmap_mermaid__er___tErDiagram o-- sealmap_mermaid__er___tEntity : entities
  sealmap_mermaid__er___tErDiagram o-- sealmap_mermaid__er___tRel : rels
  sealmap_mermaid__er___tErDiagram ..> sealmap_mermaid__escape___tIdent
  sealmap_mermaid__er___tRel *-- sealmap_mermaid__er___tCardinality : from_card, to_card
  sealmap_mermaid__er___tRel *-- sealmap_mermaid__escape___tIdent : from, to
```

## `sym:cargo sealmap_mermaid . er/ErDiagram#entity().`
`pub fn entity(&mut self, id: Ident, label: &str) -> &mut Self` · L95-L101
> Declare an entity.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__er___tErDiagram as ErDiagram
  participant sealmap_mermaid__escape as escape mod
  opt !self.entities.iter().any(| e | e.id == id)
    sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__escape: escape_text(label)
  end
```

## `sym:cargo sealmap_mermaid . er/ErDiagram#attr().`
`pub fn attr(&mut self, entity: &Ident, ty: &str, name: &str, key: Option<&str>) -> &mut Self` · L103-L119
> Add an attribute to an existing entity (no-op if `entity` is unknown).
```mermaid
sequenceDiagram
  participant sealmap_mermaid__er___tErDiagram as ErDiagram
  participant sealmap_mermaid__er as er mod
  participant sealmap_mermaid__escape as escape mod
  participant sealmap_mermaid__escape___tIdent as Ident
  sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__er: squeeze_type(ty)
  opt via then
    sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__escape: escape_text(ty)
  end
  opt let Some(e) = self.entities.iter_mut().find(| e | &e…
    sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__escape___tIdent: Ident::new(name)
  end
```

## `sym:cargo sealmap_mermaid . er/ErDiagram#relation().`
`pub fn relation(&mut self, from: &Ident, from_card: Cardinality, to: &Ident, to_card: Cardinality, identifying: bool, label: &str,) -> &mut Self` · L121-L137
> Add a relationship.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__er___tErDiagram as ErDiagram
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__escape: escape_text(label)
```

## `sym:cargo sealmap_mermaid . er/ErDiagram#render().`
`pub fn render(&self) -> String` · L144-L182
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__er___tErDiagram as ErDiagram
  participant sealmap_mermaid__writer___tCodeWriter as CodeWriter
  participant sealmap_mermaid__er___tCardinality as Cardinality
  sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: CodeWriter::new()
  sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: line(#quot;erDiagram#quot;)
  sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: indented(|..|)
  opt via indented
    loop for e in &self.entities
      opt e.attrs.is_empty()
        sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: line(head)
      end
      sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: line(_)
      sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: indented(|..|)
      opt via indented
        loop for a in &e.attrs
          sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: line(line)
        end
      end
      sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: line(#quot;}#quot;)
    end
    loop for r in &self.rels
      sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__er___tCardinality: ~left()
      sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__er___tCardinality: ~right()
      sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: line(_)
    end
  end
  sealmap_mermaid__er___tErDiagram->>sealmap_mermaid__writer___tCodeWriter: finish()
```
