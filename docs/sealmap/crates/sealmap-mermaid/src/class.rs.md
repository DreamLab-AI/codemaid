---
sealmap: 2
source: crates/sealmap-mermaid/src/class.rs
module: "sym:cargo sealmap_mermaid . class/"
language: rust
source_hash: blake3:9ffc2b8e89e2d5c584a35c79cdf301ad2fa364fe104244d93932ac7fe241ce3e
lines: 260
fragments: 7
---
# `sym:cargo sealmap_mermaid . class/` · crates/sealmap-mermaid/src/class.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__class___tClass["Class"] {
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
  class sealmap_mermaid__class___tClassDiagram["ClassDiagram"] {
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
  class sealmap_mermaid__class___tClassRelation["ClassRelation"] {
    <<struct>>
    +from: Ident
    +to: Ident
    +kind: ClassRelationKind
    +label: Option#lt;String#gt;
  }
  class sealmap_mermaid__class___tClassRelationKind["ClassRelationKind"] {
    <<enum>>
    Inheritance
    Composition
    Aggregation
    Association
    Dependency
    Realization
  }
  class sealmap_mermaid__class___tDirection["Direction"] {
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
  class sealmap_mermaid__escape___tIdent["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
  }
  class sealmap_mermaid__writer___tCodeWriter["CodeWriter"] {
    <<struct in crates/sealmap-mermaid/src/writer.rs>>
  }
  sealmap_mermaid__class___tClass *-- sealmap_mermaid__escape___tIdent : id
  sealmap_mermaid__class___tClass ..> sealmap_mermaid__writer___tCodeWriter
  sealmap_mermaid__class___tClassDiagram o-- sealmap_mermaid__class___tClass : classes
  sealmap_mermaid__class___tClassDiagram o-- sealmap_mermaid__class___tClassRelation : relations
  sealmap_mermaid__class___tClassDiagram ..> sealmap_mermaid__class___tClassRelationKind
  sealmap_mermaid__class___tClassDiagram o-- sealmap_mermaid__class___tDirection : direction
  sealmap_mermaid__class___tClassDiagram ..> sealmap_mermaid__escape___tIdent
  sealmap_mermaid__class___tClassRelation *-- sealmap_mermaid__class___tClassRelationKind : kind
  sealmap_mermaid__class___tClassRelation *-- sealmap_mermaid__escape___tIdent : from, to
```

## `sym:cargo sealmap_mermaid . class/Class#new().`
`pub fn new(id: Ident, label: &str) -> Self` · L49-L55
> A class with display `label` (generics may be written with `<>`).
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class___tClass as Class
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__class___tClass->>sealmap_mermaid__escape: escape_text(label)
```

## `sym:cargo sealmap_mermaid . class/Class#annotation().`
`pub fn annotation(&mut self, text: &str) -> &mut Self` · L62-L66
> `<<annotation>>`, e.g.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class___tClass as Class
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__class___tClass->>sealmap_mermaid__escape: escape_text(text)
```

## `sym:cargo sealmap_mermaid . class/Class#field().`
`pub fn field(&mut self, vis: char, name: &str, ty: &str) -> &mut Self` · L68-L79
> A field line: `<vis>name: Type`.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class___tClass as Class
  participant sealmap_mermaid__class as class mod
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__class___tClass->>sealmap_mermaid__class: vis_marker(vis)
  alt ty.is_empty()
    sealmap_mermaid__class___tClass->>sealmap_mermaid__escape: escape_type(name, true)
  else
    sealmap_mermaid__class___tClass->>sealmap_mermaid__escape: escape_type(name, true)
    sealmap_mermaid__class___tClass->>sealmap_mermaid__escape: escape_type(ty, true)
  end
```

## `sym:cargo sealmap_mermaid . class/Class#method().`
`pub fn method(&mut self, vis: char, name: &str, params: &str, ret: &str) -> &mut Self` · L81-L93
> A method line: `<vis>name(params) Ret`.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class___tClass as Class
  participant sealmap_mermaid__class as class mod
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__class___tClass->>sealmap_mermaid__class: vis_marker(vis)
  sealmap_mermaid__class___tClass->>sealmap_mermaid__escape: escape_type(params, true)
  sealmap_mermaid__class___tClass->>sealmap_mermaid__escape: escape_type(name, true)
  opt !ret.is_empty()
    sealmap_mermaid__class___tClass->>sealmap_mermaid__escape: escape_type(ret, true)
  end
```

## `sym:cargo sealmap_mermaid . class/Class#write().`
`fn write(&self, w: &mut CodeWriter)` · L106-L126
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class___tClass as Class
  participant sealmap_mermaid__escape___tIdent as Ident
  participant sealmap_mermaid__writer___tCodeWriter as CodeWriter
  sealmap_mermaid__class___tClass->>sealmap_mermaid__escape___tIdent: as_str()
  opt self.annotation.is_none() && self.members.is_empty()
    sealmap_mermaid__class___tClass->>sealmap_mermaid__writer___tCodeWriter: line(head)
    Note over sealmap_mermaid__class___tClass: return
  end
  sealmap_mermaid__class___tClass->>sealmap_mermaid__writer___tCodeWriter: line(_)
  sealmap_mermaid__class___tClass->>sealmap_mermaid__writer___tCodeWriter: indented(|..|)
  opt via indented
    opt let Some(a) = &self.annotation
      sealmap_mermaid__class___tClass->>sealmap_mermaid__writer___tCodeWriter: line(_)
    end
    loop for m in &self.members
      sealmap_mermaid__class___tClass->>sealmap_mermaid__writer___tCodeWriter: line(m)
    end
  end
  sealmap_mermaid__class___tClass->>sealmap_mermaid__writer___tCodeWriter: line(#quot;}#quot;)
```

## `sym:cargo sealmap_mermaid . class/ClassDiagram#render().`
`pub fn render(&self) -> String` · L231-L259
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__class___tClassDiagram as ClassDiagram
  participant sealmap_mermaid__writer___tCodeWriter as CodeWriter
  sealmap_mermaid__class___tClassDiagram->>sealmap_mermaid__writer___tCodeWriter: CodeWriter::new()
  sealmap_mermaid__class___tClassDiagram->>sealmap_mermaid__writer___tCodeWriter: line(#quot;classDiagram#quot;)
  sealmap_mermaid__class___tClassDiagram->>sealmap_mermaid__writer___tCodeWriter: indented(|..|)
  opt via indented
    opt let Some(d) = self.direction
      sealmap_mermaid__class___tClassDiagram->>sealmap_mermaid__writer___tCodeWriter: line(_)
    end
    loop for r in &self.relations
      alt Some(label)
        sealmap_mermaid__class___tClassDiagram->>sealmap_mermaid__writer___tCodeWriter: line(_)
      else None
        sealmap_mermaid__class___tClassDiagram->>sealmap_mermaid__writer___tCodeWriter: line(_)
      end
    end
  end
  sealmap_mermaid__class___tClassDiagram->>sealmap_mermaid__writer___tCodeWriter: finish()
```
