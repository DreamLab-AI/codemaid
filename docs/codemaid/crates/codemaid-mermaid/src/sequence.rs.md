---
codemaid: 1
source: crates/codemaid-mermaid/src/sequence.rs
module: codemaid_mermaid::sequence
language: rust
source_hash: blake3:6bfdd43ce4c958a37197c69649ce656ead1203374f886362ba76b61a2e3a7273
lines: 289
fragments: 13
---
# `codemaid_mermaid::sequence` · crates/codemaid-mermaid/src/sequence.rs

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_mermaid__sequence__Arrow["Arrow"] {
    <<enum>>
    Sync
    Reply
    Async
    Fail
    -token(self) &'static str
  }
  class codemaid_mermaid__sequence__BlockKind["BlockKind"] {
    <<enum>>
    Loop
    Opt
    Alt
    Par
    Critical
    Break
    -next_arm(self) &'static str
    -open(self) &'static str
  }
  class codemaid_mermaid__sequence__Item["Item"] {
    <<enum>>
    Message#123; from: Ident, to: Ident, arrow: Arrow, text: String #125;
    Note#123; over: Vec#lt;Ident#gt;, text: String #125;
    Block#123; kind: BlockKind, arms: Vec#lt;#40;String, Vec#lt;Item#gt;#41;#gt; #125;
  }
  class codemaid_mermaid__sequence__SeqBuilder["SeqBuilder"] {
    <<struct>>
    -items: Vec#lt;Item#gt;
    +arms(&mut self, kind: BlockKind, arms: Vec#lt;#40;String, Box#lt;dyn FnOnce#40;&mut SeqBuilder#41; + '_#gt;#41;#gt;) &mut Self
    +block(&mut self, kind: BlockKind, label: &str, f: impl FnOnce#40;&mut SeqBuilder#41;) &mut Self
    +extend(&mut self, other: SeqBuilder) &mut Self
    +is_empty(&self) bool
    +loop_block(&mut self, label: &str, f: impl FnOnce#40;&mut SeqBuilder#41;) &mut Self
    +message(&mut self, from: &Ident, to: &Ident, arrow: Arrow, text: &str) &mut Self
    +message_count(&self) usize
    +note(&mut self, over: &[&Ident], text: &str) &mut Self
    +opt_block(&mut self, label: &str, f: impl FnOnce#40;&mut SeqBuilder#41;) &mut Self
    -write(items: &[Item], w: &mut CodeWriter)
  }
  class codemaid_mermaid__sequence__SequenceDiagram["SequenceDiagram"] {
    <<struct>>
    -title: Option#lt;String#gt;
    -autonumber: bool
    -participants: Vec#lt;#40;Ident, String, bool#41;#gt;
    -body: SeqBuilder
    +Deref::deref(&self) &SeqBuilder
    +DerefMut::deref_mut(&mut self) &mut SeqBuilder
    +actor(&mut self, id: Ident, alias: &str) &mut Self
    +autonumber(&mut self, on: bool) &mut Self
    -declare(&mut self, id: Ident, alias: &str, actor: bool) &mut Self
    +new() Self
    +participant(&mut self, id: Ident, alias: &str) &mut Self
    +participants(&self) impl Iterator#lt;Item = &Ident#gt;
    +render(&self) String
    +title(&mut self, title: &str) &mut Self
  }
  class codemaid_mermaid__escape__Ident["Ident"] {
    <<struct in crates/codemaid-mermaid/src/escape.rs>>
  }
  class codemaid_mermaid__writer__CodeWriter["CodeWriter"] {
    <<struct in crates/codemaid-mermaid/src/writer.rs>>
  }
  codemaid_mermaid__sequence__Item o-- codemaid_mermaid__escape__Ident : Message, Note
  codemaid_mermaid__sequence__Item *-- codemaid_mermaid__sequence__Arrow : Message
  codemaid_mermaid__sequence__Item o-- codemaid_mermaid__sequence__BlockKind : Block
  codemaid_mermaid__sequence__SeqBuilder ..> codemaid_mermaid__escape__Ident
  codemaid_mermaid__sequence__SeqBuilder ..> codemaid_mermaid__sequence__Arrow
  codemaid_mermaid__sequence__SeqBuilder ..> codemaid_mermaid__sequence__BlockKind
  codemaid_mermaid__sequence__SeqBuilder o-- codemaid_mermaid__sequence__Item : items
  codemaid_mermaid__sequence__SeqBuilder ..> codemaid_mermaid__writer__CodeWriter
  codemaid_mermaid__sequence__SequenceDiagram o-- codemaid_mermaid__escape__Ident : participants
  codemaid_mermaid__sequence__SequenceDiagram *-- codemaid_mermaid__sequence__SeqBuilder : body
```

## `codemaid_mermaid::sequence::SeqBuilder::message`
`pub fn message(&mut self, from: &Ident, to: &Ident, arrow: Arrow, text: &str) -> &mut Self` · L85-L89
> `from ->> to: text` (or another arrow).
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__escape: escape_text(text)
```

## `codemaid_mermaid::sequence::SeqBuilder::note`
`pub fn note(&mut self, over: &[&Ident], text: &str) -> &mut Self` · L91-L97
> `Note over a,b: text` (one or two participants; `right of` when only one is given would be ambiguous for merging, so `over` is always used).
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__escape: escape_text(text)
```

## `codemaid_mermaid::sequence::SeqBuilder::block`
`pub fn block(&mut self, kind: BlockKind, label: &str, f: impl FnOnce(&mut SeqBuilder)) -> &mut Self` · L99-L105
> A single-arm fragment (`loop`, `opt`, `break`).
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
  codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__escape: escape_text(label)
```

## `codemaid_mermaid::sequence::SeqBuilder::loop_block`
`pub fn loop_block(&mut self, label: &str, f: impl FnOnce(&mut SeqBuilder)) -> &mut Self` · L107-L110
> Shorthand for `block(BlockKind::Loop, ..)`.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__sequence__SeqBuilder: block(Loop, label, f)
```

## `codemaid_mermaid::sequence::SeqBuilder::opt_block`
`pub fn opt_block(&mut self, label: &str, f: impl FnOnce(&mut SeqBuilder)) -> &mut Self` · L112-L115
> Shorthand for `block(BlockKind::Opt, ..)`.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__sequence__SeqBuilder: block(Opt, label, f)
```

## `codemaid_mermaid::sequence::SeqBuilder::arms`
`pub fn arms(&mut self, kind: BlockKind, arms: Vec<(String, Box<dyn FnOnce(&mut SeqBuilder) + '_>)>) -> &mut Self` · L117-L146
> A multi-arm fragment (`alt`/`else`, `par`/`and`, `critical`/`option`).
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  participant codemaid_mermaid__escape as escape mod
  opt arms.is_empty()
    Note over codemaid_mermaid__sequence__SeqBuilder: return self
  end
  opt via map
    codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
    codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__escape: escape_text(&label)
  end
```

## `codemaid_mermaid::sequence::SeqBuilder::write`
`fn write(items: &[Item], w: &mut CodeWriter)` · L175-L195
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  participant codemaid_mermaid__writer__CodeWriter as CodeWriter
  participant codemaid_mermaid__sequence__BlockKind as BlockKind
  loop for item in items
    alt Item::Message { from, to, arrow, text }
      codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__writer__CodeWriter: line(_)
    else Item::Note { over, text }
      codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__writer__CodeWriter: line(_)
    else Item::Block { kind, arms }
      loop for (i, (label, body)) in arms.iter().enumerate()
        opt not i == 0
          codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__sequence__BlockKind: ~next_arm()
        end
        codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__writer__CodeWriter: line(_)
        codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__writer__CodeWriter: indented(|..|)
      end
      codemaid_mermaid__sequence__SeqBuilder->>codemaid_mermaid__writer__CodeWriter: line(#quot;end#quot;)
    end
  end
```

## `codemaid_mermaid::sequence::SequenceDiagram::title`
`pub fn title(&mut self, title: &str) -> &mut Self` · L218-L222
> Set the diagram title.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SequenceDiagram as SequenceDiagram
  participant codemaid_mermaid__escape as escape mod
  codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__escape: escape_text(title)
```

## `codemaid_mermaid::sequence::SequenceDiagram::participant`
`pub fn participant(&mut self, id: Ident, alias: &str) -> &mut Self` · L230-L234
> Declare a participant lane with a display alias.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SequenceDiagram as SequenceDiagram
  codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__sequence__SequenceDiagram: declare(id, alias, false)
```

## `codemaid_mermaid::sequence::SequenceDiagram::actor`
`pub fn actor(&mut self, id: Ident, alias: &str) -> &mut Self` · L236-L239
> Declare an actor lane (stick figure), e.g.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SequenceDiagram as SequenceDiagram
  codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__sequence__SequenceDiagram: declare(id, alias, true)
```

## `codemaid_mermaid::sequence::SequenceDiagram::declare`
`fn declare(&mut self, id: Ident, alias: &str, actor: bool) -> &mut Self` · L241-L246
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SequenceDiagram as SequenceDiagram
  participant codemaid_mermaid__escape as escape mod
  opt !self.participants.iter().any(| (p, _, _) | * p == i…
    codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__escape: escape_text(alias)
  end
```

## `codemaid_mermaid::sequence::SequenceDiagram::render`
`pub fn render(&self) -> String` · L253-L275
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant codemaid_mermaid__sequence__SequenceDiagram as SequenceDiagram
  participant codemaid_mermaid__writer__CodeWriter as CodeWriter
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__writer__CodeWriter: CodeWriter::new()
  codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__writer__CodeWriter: line(#quot;sequenceDiag…)
  codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__writer__CodeWriter: indented(|..|)
  opt via indented
    opt let Some(t) = &self.title
      codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__writer__CodeWriter: line(_)
    end
    opt self.autonumber
      codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__writer__CodeWriter: line(#quot;autonumber#quot;)
    end
    loop for (id, alias, actor) in &self.participants
      alt alias.is_empty() || alias == id.as_str()
        codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__writer__CodeWriter: line(_)
      else
        codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__writer__CodeWriter: line(_)
      end
    end
    codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__sequence__SeqBuilder: SeqBuilder::write(&self.body.items, w)
  end
  codemaid_mermaid__sequence__SequenceDiagram->>codemaid_mermaid__writer__CodeWriter: finish()
```
