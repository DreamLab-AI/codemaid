---
codemaid: 1
source: crates/codemaid-mermaid/src/class.rs
module: codemaid_mermaid::class
language: rust
source_hash: blake3:9f12255f29c87a651b85c28e21ade81cf8752be893cf6ac1aaa1e989ebcc3481
lines: 260
fragments: 7
---
# `codemaid_mermaid::class` · crates/codemaid-mermaid/src/class.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_mermaid__class__Class["Class"] {
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
  class codemaid_mermaid__class__ClassDiagram["ClassDiagram"] {
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
  class codemaid_mermaid__class__ClassRelation["ClassRelation"] {
    <<struct>>
    +from: Ident
    +to: Ident
    +kind: ClassRelationKind
    +label: Option#lt;String#gt;
  }
  class codemaid_mermaid__class__ClassRelationKind["ClassRelationKind"] {
    <<enum>>
    Inheritance
    Composition
    Aggregation
    Association
    Dependency
    Realization
  }
  class codemaid_mermaid__class__Direction["Direction"] {
    <<enum>>
    TB
    BT
    LR
    RL
    ~as_str(crate) &'static str
  }
  class codemaid_mermaid__class["codemaid_mermaid::class"] {
    <<module>>
    -vis_marker(c: char) &'static str
  }
  class codemaid_mermaid__escape__Ident["Ident"] {
    <<struct in crates/codemaid-mermaid/src/escape.rs>>
  }
  class codemaid_mermaid__writer__CodeWriter["CodeWriter"] {
    <<struct in crates/codemaid-mermaid/src/writer.rs>>
  }
  codemaid_mermaid__class__Class *-- codemaid_mermaid__escape__Ident : id
  codemaid_mermaid__class__Class ..> codemaid_mermaid__writer__CodeWriter
  codemaid_mermaid__class__ClassDiagram o-- codemaid_mermaid__class__Class : classes
  codemaid_mermaid__class__ClassDiagram o-- codemaid_mermaid__class__ClassRelation : relations
  codemaid_mermaid__class__ClassDiagram ..> codemaid_mermaid__class__ClassRelationKind
  codemaid_mermaid__class__ClassDiagram o-- codemaid_mermaid__class__Direction : direction
  codemaid_mermaid__class__ClassDiagram ..> codemaid_mermaid__escape__Ident
  codemaid_mermaid__class__ClassRelation *-- codemaid_mermaid__class__ClassRelationKind : kind
  codemaid_mermaid__class__ClassRelation *-- codemaid_mermaid__escape__Ident : from, to
```

## `codemaid_mermaid::class::Class::new`
`pub fn new(id: Ident, label: &str) -> Self` · L49-L55
> A class with display `label` (generics may be written with `<>`).
```mermaid
sequenceDiagram
  participant codemaid_mermaid__class__Class as Class
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__class__Class->>codemaid_mermaid__escape: escape_text(label)
```

## `codemaid_mermaid::class::Class::annotation`
`pub fn annotation(&mut self, text: &str) -> &mut Self` · L62-L66
> `<<annotation>>`, e.g.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__class__Class as Class
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__class__Class->>codemaid_mermaid__escape: escape_text(text)
```

## `codemaid_mermaid::class::Class::field`
`pub fn field(&mut self, vis: char, name: &str, ty: &str) -> &mut Self` · L68-L79
> A field line: `<vis>name: Type`.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__class__Class as Class
  participant codemaid_mermaid__class as class mod
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__class__Class->>codemaid_mermaid__class: vis_marker(vis)
  alt ty.is_empty()
    codemaid_mermaid__class__Class->>codemaid_mermaid__escape: escape_type(name, true)
  else
    codemaid_mermaid__class__Class->>codemaid_mermaid__escape: escape_type(name, true)
    codemaid_mermaid__class__Class->>codemaid_mermaid__escape: escape_type(ty, true)
  end
```

## `codemaid_mermaid::class::Class::method`
`pub fn method(&mut self, vis: char, name: &str, params: &str, ret: &str) -> &mut Self` · L81-L93
> A method line: `<vis>name(params) Ret`.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__class__Class as Class
  participant codemaid_mermaid__class as class mod
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__class__Class->>codemaid_mermaid__class: vis_marker(vis)
  codemaid_mermaid__class__Class->>codemaid_mermaid__escape: escape_type(params, true)
  codemaid_mermaid__class__Class->>codemaid_mermaid__escape: escape_type(name, true)
  opt !ret.is_empty()
    codemaid_mermaid__class__Class->>codemaid_mermaid__escape: escape_type(ret, true)
  end
```

## `codemaid_mermaid::class::Class::write`
`fn write(&self, w: &mut CodeWriter)` · L106-L126
```mermaid
sequenceDiagram
  participant codemaid_mermaid__class__Class as Class
  participant codemaid_mermaid__escape__Ident as Ident
  participant codemaid_mermaid__writer__CodeWriter as CodeWriter
  codemaid_mermaid__class__Class->>codemaid_mermaid__escape__Ident: as_str()
  opt self.annotation.is_none() && self.members.is_empty()
    codemaid_mermaid__class__Class->>codemaid_mermaid__writer__CodeWriter: line(head)
    Note over codemaid_mermaid__class__Class: return
  end
  codemaid_mermaid__class__Class->>codemaid_mermaid__writer__CodeWriter: line(_)
  codemaid_mermaid__class__Class->>codemaid_mermaid__writer__CodeWriter: indented(|..|)
  opt via indented
    opt let Some(a) = &self.annotation
      codemaid_mermaid__class__Class->>codemaid_mermaid__writer__CodeWriter: line(_)
    end
    loop for m in &self.members
      codemaid_mermaid__class__Class->>codemaid_mermaid__writer__CodeWriter: line(m)
    end
  end
  codemaid_mermaid__class__Class->>codemaid_mermaid__writer__CodeWriter: line(#quot;}#quot;)
```

## `codemaid_mermaid::class::ClassDiagram::render`
`pub fn render(&self) -> String` · L231-L259
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__class__ClassDiagram as ClassDiagram
  participant codemaid_mermaid__writer__CodeWriter as CodeWriter
  codemaid_mermaid__class__ClassDiagram->>codemaid_mermaid__writer__CodeWriter: CodeWriter::new()
  codemaid_mermaid__class__ClassDiagram->>codemaid_mermaid__writer__CodeWriter: line(#quot;classDiagram#quot;)
  codemaid_mermaid__class__ClassDiagram->>codemaid_mermaid__writer__CodeWriter: indented(|..|)
  opt via indented
    opt let Some(d) = self.direction
      codemaid_mermaid__class__ClassDiagram->>codemaid_mermaid__writer__CodeWriter: line(_)
    end
    loop for r in &self.relations
      alt Some(label)
        codemaid_mermaid__class__ClassDiagram->>codemaid_mermaid__writer__CodeWriter: line(_)
      else None
        codemaid_mermaid__class__ClassDiagram->>codemaid_mermaid__writer__CodeWriter: line(_)
      end
    end
  end
  codemaid_mermaid__class__ClassDiagram->>codemaid_mermaid__writer__CodeWriter: finish()
```
