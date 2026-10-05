---
sealmap: 1
source: crates/sealmap-corpus/src/sequence.rs
module: sealmap_corpus::sequence
language: rust
source_hash: blake3:542ad5e9f93a10124a0bcc2962eb577ad6d2be0796da817d107f54fc2d2fd3f5
lines: 151
fragments: 4
---
# `sealmap_corpus::sequence` · crates/sealmap-corpus/src/sequence.rs
> Flow → `sequenceDiagram`.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__sequence__Ctx["Ctx#lt;'a#gt;"] {
    <<struct>>
    -cb: &'a Codebase
    -opts: &'a CorpusOptions
    -caller: SymbolId
    -lanes: Vec#lt;#40;SymbolId, String, bool#41;#gt;
    -budget: usize
    -dropped: usize
    -arms(&mut self, kind: BlockKind, arms: &[sealmap_model::Arm], out: &mut SeqBuilder)
    -lane(&mut self, lane: Lane) SymbolId
    -steps(&mut self, steps: &[Step], out: &mut SeqBuilder)
  }
  class sealmap_corpus__sequence__Rendered["Rendered"] {
    <<struct>>
    +text: String
    +participants: Vec#lt;SymbolId#gt;
    +truncated: usize
  }
  class sealmap_corpus__sequence["sealmap_corpus::sequence"] {
    <<module>>
    ~render(crate) Option#lt;Rendered#gt;
  }
  class sealmap_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__symbol__Symbol["Symbol"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_corpus__naming__Lane["Lane"] {
    <<struct in crates/sealmap-corpus/src/naming.rs>>
  }
  class sealmap_mermaid__sequence__BlockKind["BlockKind"] {
    <<enum in crates/sealmap-mermaid/src/sequence.rs>>
  }
  class sealmap_mermaid__sequence__SeqBuilder["SeqBuilder"] {
    <<struct in crates/sealmap-mermaid/src/sequence.rs>>
  }
  class sealmap_model__flow__Arm["Arm"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__flow__Step["Step"] {
    <<enum in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_corpus__sequence ..> sealmap_corpus__CorpusOptions
  sealmap_corpus__sequence ..> sealmap_corpus__sequence__Rendered
  sealmap_corpus__sequence ..> sealmap_model__codebase__Codebase
  sealmap_corpus__sequence ..> sealmap_model__symbol__Symbol
  sealmap_corpus__sequence__Ctx o-- sealmap_corpus__CorpusOptions : opts
  sealmap_corpus__sequence__Ctx ..> sealmap_corpus__naming__Lane
  sealmap_corpus__sequence__Ctx ..> sealmap_mermaid__sequence__BlockKind
  sealmap_corpus__sequence__Ctx ..> sealmap_mermaid__sequence__SeqBuilder
  sealmap_corpus__sequence__Ctx o-- sealmap_model__codebase__Codebase : cb
  sealmap_corpus__sequence__Ctx ..> sealmap_model__flow__Arm
  sealmap_corpus__sequence__Ctx ..> sealmap_model__flow__Step
  sealmap_corpus__sequence__Ctx o-- sealmap_model__symbol__SymbolId : caller, lanes
  sealmap_corpus__sequence__Rendered o-- sealmap_model__symbol__SymbolId : participants
```

## `sealmap_corpus::sequence::render`
`pub(crate) fn render(cb: &Codebase, sym: &Symbol, opts: &CorpusOptions) -> Option<Rendered>` · L17-L47
```mermaid
sequenceDiagram
  participant sealmap_corpus__sequence as sequence mod
  participant sealmap_model__codebase__Codebase as Codebase
  participant sealmap_corpus__naming as naming mod
  participant sealmap_corpus__sequence__Ctx as Ctx
  participant sealmap_mermaid__sequence__SeqBuilder as SeqBuilder
  participant sealmap_mermaid__sequence__SequenceDiagram as SequenceDiagram
  opt flow.call_count()<opts.min_calls
    Note over sealmap_corpus__sequence: return None
  end
  sealmap_corpus__sequence->>sealmap_model__codebase__Codebase: owner_of(&sym.id)
  sealmap_corpus__sequence->>sealmap_corpus__naming: alias_for(cb, &caller)
  sealmap_corpus__sequence->>sealmap_corpus__sequence__Ctx: lane(_)
  sealmap_corpus__sequence->>sealmap_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
  sealmap_corpus__sequence->>sealmap_corpus__sequence__Ctx: steps(&flow.steps, &body)
  sealmap_corpus__sequence->>sealmap_mermaid__sequence__SequenceDiagram: SequenceDiagram::new()
  loop for (id, alias, external) in &ctx.lanes
    alt * external
      sealmap_corpus__sequence->>sealmap_corpus__naming: ident(id)
      sealmap_corpus__sequence->>sealmap_mermaid__sequence__SequenceDiagram: participant(ident(), &_)
    else
      sealmap_corpus__sequence->>sealmap_corpus__naming: ident(id)
      sealmap_corpus__sequence->>sealmap_mermaid__sequence__SequenceDiagram: participant(ident(), alias)
    end
  end
  opt ctx.dropped> 0
    sealmap_corpus__sequence->>sealmap_corpus__naming: ident(&_.0)
  end
  sealmap_corpus__sequence->>sealmap_mermaid__sequence__SequenceDiagram: render()
```

## `sealmap_corpus::sequence::Ctx::steps`
`fn steps(&mut self, steps: &[Step], out: &mut SeqBuilder)` · L66-L118
```mermaid
sequenceDiagram
  participant sealmap_corpus__sequence__Ctx as Ctx
  participant sealmap_corpus__naming as naming mod
  participant sealmap_mermaid__sequence__SeqBuilder as SeqBuilder
  sealmap_corpus__sequence__Ctx->>sealmap_corpus__naming: ident(&self.caller)
  loop for step in steps
    alt Step::Call(c)
      sealmap_corpus__sequence__Ctx->>sealmap_corpus__naming: lane_of(self.cb, &c.target, self.opts)
      sealmap_corpus__sequence__Ctx->>sealmap_corpus__sequence__Ctx: lane(lane)
      sealmap_corpus__sequence__Ctx->>sealmap_corpus__naming: ident(&lane())
      sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: message(&me, &to, Sync, &text)
    else Step::Branch { arms }
      sealmap_corpus__sequence__Ctx->>sealmap_corpus__sequence__Ctx: arms(Alt, arms, out)
    else Step::Parallel { arms }
      sealmap_corpus__sequence__Ctx->>sealmap_corpus__sequence__Ctx: arms(Par, arms, out)
    else Step::Loop { label, body }
      sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
      sealmap_corpus__sequence__Ctx->>sealmap_corpus__sequence__Ctx: steps(body, &inner)
      sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: is_empty()
      opt !inner.is_empty()
        sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: block(Loop, label, |..|)
      end
    else Step::Optional { label, body }
      sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
      sealmap_corpus__sequence__Ctx->>sealmap_corpus__sequence__Ctx: steps(body, &inner)
      sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: is_empty()
      opt !inner.is_empty()
        sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: block(Opt, label, |..|)
      end
    else Step::Return(exit)
      sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: note(&_, &exit.label)
    end
  end
```

## `sealmap_corpus::sequence::Ctx::arms`
`fn arms(&mut self, kind: BlockKind, arms: &[sealmap_model::Arm], out: &mut SeqBuilder)` · L120-L150
```mermaid
sequenceDiagram
  participant sealmap_corpus__sequence__Ctx as Ctx
  participant sealmap_mermaid__sequence__SeqBuilder as SeqBuilder
  loop for arm in arms
    sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
    sealmap_corpus__sequence__Ctx->>sealmap_corpus__sequence__Ctx: steps(&arm.steps, &inner)
    sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: is_empty()
  end
  opt built.is_empty()
    Note over sealmap_corpus__sequence__Ctx: return
  end
  opt built.len() == 1 && kind == BlockKind::Alt
    sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: block(Opt, &label, |..|)
    Note over sealmap_corpus__sequence__Ctx: return
  end
  sealmap_corpus__sequence__Ctx->>sealmap_mermaid__sequence__SeqBuilder: arms(kind, arms)
```
