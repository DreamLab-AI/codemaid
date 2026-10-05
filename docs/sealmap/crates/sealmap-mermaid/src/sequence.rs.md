---
sealmap: 2
source: crates/sealmap-mermaid/src/sequence.rs
module: "sym:cargo sealmap_mermaid . sequence/"
language: rust
source_hash: blake3:65885c8bae5ba6873df84c86d0e56c718c7c781847668ab4452d719462c17507
lines: 289
fragments: 13
---
# `sym:cargo sealmap_mermaid . sequence/` · crates/sealmap-mermaid/src/sequence.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__sequence___tArrow["Arrow"] {
    <<enum>>
    Sync
    Reply
    Async
    Fail
    -token(self) &'static str
  }
  class sealmap_mermaid__sequence___tBlockKind["BlockKind"] {
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
  class sealmap_mermaid__sequence___tItem["Item"] {
    <<enum>>
    Message#123; from: Ident, to: Ident, arrow: Arrow, text: String #125;
    Note#123; over: Vec#lt;Ident#gt;, text: String #125;
    Block#123; kind: BlockKind, arms: Vec#lt;#40;String, Vec#lt;Item#gt;#41;#gt; #125;
  }
  class sealmap_mermaid__sequence___tSeqBuilder["SeqBuilder"] {
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
  class sealmap_mermaid__sequence___tSequenceDiagram["SequenceDiagram"] {
    <<struct>>
    -title: Option#lt;String#gt;
    -autonumber: bool
    -participants: Vec#lt;#40;Ident, String, bool#41;#gt;
    -body: SeqBuilder
    +DerefMut::deref_mut(&mut self) &mut SeqBuilder
    +Deref::deref(&self) &SeqBuilder
    +actor(&mut self, id: Ident, alias: &str) &mut Self
    +autonumber(&mut self, on: bool) &mut Self
    -declare(&mut self, id: Ident, alias: &str, actor: bool) &mut Self
    +new() Self
    +participant(&mut self, id: Ident, alias: &str) &mut Self
    +participants(&self) impl Iterator#lt;Item = &Ident#gt;
    +render(&self) String
    +title(&mut self, title: &str) &mut Self
  }
  class sealmap_mermaid__escape___tIdent["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
  }
  class sealmap_mermaid__writer___tCodeWriter["CodeWriter"] {
    <<struct in crates/sealmap-mermaid/src/writer.rs>>
  }
  sealmap_mermaid__sequence___tItem o-- sealmap_mermaid__escape___tIdent : Message, Note
  sealmap_mermaid__sequence___tItem *-- sealmap_mermaid__sequence___tArrow : Message
  sealmap_mermaid__sequence___tItem o-- sealmap_mermaid__sequence___tBlockKind : Block
  sealmap_mermaid__sequence___tSeqBuilder ..> sealmap_mermaid__escape___tIdent
  sealmap_mermaid__sequence___tSeqBuilder ..> sealmap_mermaid__sequence___tArrow
  sealmap_mermaid__sequence___tSeqBuilder ..> sealmap_mermaid__sequence___tBlockKind
  sealmap_mermaid__sequence___tSeqBuilder o-- sealmap_mermaid__sequence___tItem : items
  sealmap_mermaid__sequence___tSeqBuilder ..> sealmap_mermaid__writer___tCodeWriter
  sealmap_mermaid__sequence___tSequenceDiagram o-- sealmap_mermaid__escape___tIdent : participants
  sealmap_mermaid__sequence___tSequenceDiagram *-- sealmap_mermaid__sequence___tSeqBuilder : body
```

## `sym:cargo sealmap_mermaid . sequence/SeqBuilder#message().`
`pub fn message(&mut self, from: &Ident, to: &Ident, arrow: Arrow, text: &str) -> &mut Self` · L85-L89
> `from ->> to: text` (or another arrow).
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__escape: escape_text(text)
```

## `sym:cargo sealmap_mermaid . sequence/SeqBuilder#note().`
`pub fn note(&mut self, over: &[&Ident], text: &str) -> &mut Self` · L91-L97
> `Note over a,b: text` (one or two participants; `right of` when only one is given would be ambiguous for merging, so `over` is always used).
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__escape: escape_text(text)
```

## `sym:cargo sealmap_mermaid . sequence/SeqBuilder#block().`
`pub fn block(&mut self, kind: BlockKind, label: &str, f: impl FnOnce(&mut SeqBuilder)) -> &mut Self` · L99-L105
> A single-arm fragment (`loop`, `opt`, `break`).
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  participant _sealmap_mermaid as sealmap_mermaid ext
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__sequence___tSeqBuilder->>_sealmap_mermaid: ~SeqBuilder::SeqBuilder::default()
  sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__escape: escape_text(label)
```

## `sym:cargo sealmap_mermaid . sequence/SeqBuilder#loop_block().`
`pub fn loop_block(&mut self, label: &str, f: impl FnOnce(&mut SeqBuilder)) -> &mut Self` · L107-L110
> Shorthand for `block(BlockKind::Loop, ..)`.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__sequence___tSeqBuilder: block(Loop, label, f)
```

## `sym:cargo sealmap_mermaid . sequence/SeqBuilder#opt_block().`
`pub fn opt_block(&mut self, label: &str, f: impl FnOnce(&mut SeqBuilder)) -> &mut Self` · L112-L115
> Shorthand for `block(BlockKind::Opt, ..)`.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__sequence___tSeqBuilder: block(Opt, label, f)
```

## `sym:cargo sealmap_mermaid . sequence/SeqBuilder#arms().`
`pub fn arms(&mut self, kind: BlockKind, arms: Vec<(String, Box<dyn FnOnce(&mut SeqBuilder) + '_>)>) -> &mut Self` · L117-L146
> A multi-arm fragment (`alt`/`else`, `par`/`and`, `critical`/`option`).
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  participant _sealmap_mermaid as sealmap_mermaid ext
  participant sealmap_mermaid__escape as escape mod
  opt arms.is_empty()
    Note over sealmap_mermaid__sequence___tSeqBuilder: return self
  end
  opt via map
    sealmap_mermaid__sequence___tSeqBuilder->>_sealmap_mermaid: ~SeqBuilder::SeqBuilder::default()
    sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__escape: escape_text(&label)
  end
```

## `sym:cargo sealmap_mermaid . sequence/SeqBuilder#write().`
`fn write(items: &[Item], w: &mut CodeWriter)` · L175-L195
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  participant sealmap_mermaid__writer___tCodeWriter as CodeWriter
  participant sealmap_mermaid__sequence___tBlockKind as BlockKind
  loop for item in items
    alt Item::Message { from, to, arrow, text }
      sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__writer___tCodeWriter: line(_)
    else Item::Note { over, text }
      sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__writer___tCodeWriter: line(_)
    else Item::Block { kind, arms }
      loop for (i, (label, body)) in arms.iter().enumerate()
        opt not i == 0
          sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__sequence___tBlockKind: ~next_arm()
        end
        sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__writer___tCodeWriter: line(_)
        sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__writer___tCodeWriter: indented(|..|)
      end
      sealmap_mermaid__sequence___tSeqBuilder->>sealmap_mermaid__writer___tCodeWriter: line(#quot;end#quot;)
    end
  end
```

## `sym:cargo sealmap_mermaid . sequence/SequenceDiagram#title().`
`pub fn title(&mut self, title: &str) -> &mut Self` · L218-L222
> Set the diagram title.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSequenceDiagram as SequenceDiagram
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__escape: escape_text(title)
```

## `sym:cargo sealmap_mermaid . sequence/SequenceDiagram#participant().`
`pub fn participant(&mut self, id: Ident, alias: &str) -> &mut Self` · L230-L234
> Declare a participant lane with a display alias.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSequenceDiagram as SequenceDiagram
  sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__sequence___tSequenceDiagram: declare(id, alias, false)
```

## `sym:cargo sealmap_mermaid . sequence/SequenceDiagram#actor().`
`pub fn actor(&mut self, id: Ident, alias: &str) -> &mut Self` · L236-L239
> Declare an actor lane (stick figure), e.g.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSequenceDiagram as SequenceDiagram
  sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__sequence___tSequenceDiagram: declare(id, alias, true)
```

## `sym:cargo sealmap_mermaid . sequence/SequenceDiagram#declare().`
`fn declare(&mut self, id: Ident, alias: &str, actor: bool) -> &mut Self` · L241-L246
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSequenceDiagram as SequenceDiagram
  participant sealmap_mermaid__escape as escape mod
  opt !self.participants.iter().any(| (p, _, _) | * p == i…
    sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__escape: escape_text(alias)
  end
```

## `sym:cargo sealmap_mermaid . sequence/SequenceDiagram#render().`
`pub fn render(&self) -> String` · L253-L275
> Render to Mermaid text.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__sequence___tSequenceDiagram as SequenceDiagram
  participant sealmap_mermaid__writer___tCodeWriter as CodeWriter
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__writer___tCodeWriter: CodeWriter::new()
  sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__writer___tCodeWriter: line(#quot;sequenceDiag…)
  sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__writer___tCodeWriter: indented(|..|)
  opt via indented
    opt let Some(t) = &self.title
      sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__writer___tCodeWriter: line(_)
    end
    opt self.autonumber
      sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__writer___tCodeWriter: line(#quot;autonumber#quot;)
    end
    loop for (id, alias, actor) in &self.participants
      alt alias.is_empty() || alias == id.as_str()
        sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__writer___tCodeWriter: line(_)
      else
        sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__writer___tCodeWriter: line(_)
      end
    end
    sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__sequence___tSeqBuilder: SeqBuilder::write(&self.body.items, w)
  end
  sealmap_mermaid__sequence___tSequenceDiagram->>sealmap_mermaid__writer___tCodeWriter: finish()
```
