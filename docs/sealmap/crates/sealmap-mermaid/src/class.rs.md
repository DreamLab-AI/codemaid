---
sealmap: 1
source: crates/sealmap-mermaid/src/class.rs
module: sealmap_mermaid::class
language: rust
source_hash: blake3:e4397011d22e899be6e4eb5e37bb9c1bba78d66eb8d7a6e706f2dfdec9d5a38d
lines: 260
fragments: 7
---
# `sealmap_mermaid::class` · crates/sealmap-mermaid/src/class.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__class__Class["Class"] {
    <<struct>>
    -id: Ident
    -label: String
    -annotation: Option#lt;String#gt;
    -members: Vec#lt;String#gt;
    +annotation(&mut self, text: &str) &mut Self
    +field(&mut self, vis: char, name: &str, ty: &str) &mut Self
    +id(&self) &Ident
    +member_count(&self) usize
    +method(&mut self, vis: char, name: &str, params: &str, ret: &str) &mut Self
    +new(id: Ident, label: &str) Self
    +raw_member(&mut self, line: &str) &mut Self
    -write(&self, w: &mut CodeWriter)
  }
  class sealmap_mermaid__class__ClassDiagram["ClassDiagram"] {
    <<struct>>
    -direction: Option#lt;Direction#gt;
    -classes: Vec#lt;Class#gt;
    -relations: Vec#lt;ClassRelation#gt;
    +class(&mut self, class: Class) &mut Self
    +class_count(&self) usize
    +has_class(&self, id: &Ident) bool
    +new(direction: Direction) Self
    +relation(&mut self, from: &Ident, to: &Ident, kind: ClassRelationKind, label: Option#lt;&str#gt;) &mut Self
    +relation_count(&self) usize
    +render(&self) String
  }
  class sealmap_mermaid__class__ClassRelation["ClassRelation"] {
    <<struct>>
    +from: Ident
    +to: Ident
    +kind: ClassRelationKind
    +label: Option#lt;String#gt;
  }
  class sealmap_mermaid__class__ClassRelationKind["ClassRelationKind"] {
    <<enum>>
    Inheritance
    Composition
    Aggregation
    Association
    Dependency
    Realization
  }
  class sealmap_mermaid__class__Direction["Direction"] {
    <<enum>>
    TB
    BT
    LR
    RL
    ~as_str(crate) &'static str
  }
  class sealmap_mermaid__class["sealmap_mermaid::class"] {
    <<module>>
    -vis_marker(c: char) &'static str
  }
  class sealmap_mermaid__escape__Ident["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
  }
  class sealmap_mermaid__writer__CodeWriter["CodeWriter"] {
    <<struct in crates/sealmap-mermaid/src/writer.rs>>
  }
  sealmap_mermaid__class__Class *-- sealmap_mermaid__escape__Ident : id
  sealmap_mermaid__class__Class ..> sealmap_mermaid__writer__CodeWriter
  sealmap_mermaid__class__ClassDiagram o-- sealmap_mermaid__class__Class : classes
  sealmap_mermaid__class__ClassDiagram o-- sealmap_mermaid__class__ClassRelation : relations
  sealmap_mermaid__class__ClassDiagram ..> sealmap_mermaid__class__ClassRelationKind
  sealmap_mermaid__class__ClassDiagram o-- sealmap_mermaid__class__Direction : direction
  sealmap_mermaid__class__ClassDiagram ..> sealmap_mermaid__escape__Ident
  sealmap_mermaid__class__ClassRelation *-- sealmap_mermaid__class__ClassRelationKind : kind
  sealmap_mermaid__class__ClassRelation *-- sealmap_mermaid__escape__Ident : from, to
```

## `sealmap_mermaid::class::Class::new`
`pub fn new(id: Ident, label: &str) -> Self` · L49-L55
> A class with display `label` (generics may be written with `<>`).
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class__Class as Class
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__class__Class->>sealmap_mermaid__escape: escape_text(label)
```

## `sealmap_mermaid::class::Class::annotation`
`pub fn annotation(&mut self, text: &str) -> &mut Self` · L62-L66
> `<<annotation>>`, e.g.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class__Class as Class
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__class__Class->>sealmap_mermaid__escape: escape_text(text)
```

## `sealmap_mermaid::class::Class::field`
`pub fn field(&mut self, vis: char, name: &str, ty: &str) -> &mut Self` · L68-L79
> A field line: `<vis>name: Type`.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class__Class as Class
  participant sealmap_mermaid__class as class mod
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__class__Class->>sealmap_mermaid__class: vis_marker(vis)
  alt ty.is_empty()
    sealmap_mermaid__class__Class->>sealmap_mermaid__escape: escape_type(name, true)
  else
    sealmap_mermaid__class__Class->>sealmap_mermaid__escape: escape_type(name, true)
    sealmap_mermaid__class__Class->>sealmap_mermaid__escape: escape_type(ty, true)
  end
```

## `sealmap_mermaid::class::Class::method`
`pub fn method(&mut self, vis: char, name: &str, params: &str, ret: &str) -> &mut Self` · L81-L93
> A method line: `<vis>name(params) Ret`.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class__Class as Class
  participant sealmap_mermaid__class as class mod
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__class__Class->>sealmap_mermaid__class: vis_marker(vis)
  sealmap_mermaid__class__Class->>sealmap_mermaid__escape: escape_type(params, true)
  sealmap_mermaid__class__Class->>sealmap_mermaid__escape: escape_type(name, true)
  opt !ret.is_empty()
    sealmap_mermaid__class__Class->>sealmap_mermaid__escape: escape_type(ret, true)
  end
```

## `sealmap_mermaid::class::Class::write`
`fn write(&self, w: &mut CodeWriter)` · L106-L126
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class__Class as Class
  participant sealmap_mermaid__escape__Ident as Ident
  participant sealmap_mermaid__writer__CodeWriter as CodeWriter
  sealmap_mermaid__class__Class->>sealmap_mermaid__escape__Ident: as_str()
  opt self.annotation.is_none() && self.members.is_empty()
    sealmap_mermaid__class__Class->>sealmap_mermaid__writer__CodeWriter: line(head)
    Note over sealmap_mermaid__class__Class: return
  end
  sealmap_mermaid__class__Class->>sealmap_mermaid__writer__CodeWriter: line(_)
  sealmap_mermaid__class__Class->>sealmap_mermaid__writer__CodeWriter: indented(|..|)
  opt via indented
    opt let Some(a) = &self.annotation
      sealmap_mermaid__class__Class->>sealmap_mermaid__writer__CodeWriter: line(_)
    end
    loop for m in &self.members
      sealmap_mermaid__class__Class->>sealmap_mermaid__writer__CodeWriter: line(m)
    end
  end
  sealmap_mermaid__class__Class->>sealmap_mermaid__writer__CodeWriter: line(#quot;}#quot;)
```

## `sealmap_mermaid::class::ClassDiagram::render`
`pub fn render(&self) -> String` · L231-L259
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class__ClassDiagram as ClassDiagram
  participant sealmap_mermaid__writer__CodeWriter as CodeWriter
  sealmap_mermaid__class__ClassDiagram->>sealmap_mermaid__writer__CodeWriter: CodeWriter::new()
  sealmap_mermaid__class__ClassDiagram->>sealmap_mermaid__writer__CodeWriter: line(#quot;classDiagram#quot;)
  sealmap_mermaid__class__ClassDiagram->>sealmap_mermaid__writer__CodeWriter: indented(|..|)
  opt via indented
    opt let Some(d) = self.direction
      sealmap_mermaid__class__ClassDiagram->>sealmap_mermaid__writer__CodeWriter: line(_)
    end
    loop for r in &self.relations
      alt Some(label)
        sealmap_mermaid__class__ClassDiagram->>sealmap_mermaid__writer__CodeWriter: line(_)
      else None
        sealmap_mermaid__class__ClassDiagram->>sealmap_mermaid__writer__CodeWriter: line(_)
      end
    end
  end
  sealmap_mermaid__class__ClassDiagram->>sealmap_mermaid__writer__CodeWriter: finish()
```
