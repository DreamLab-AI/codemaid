---
codemaid: 1
source: crates/codemaid-corpus/src/sequence.rs
module: codemaid_corpus::sequence
language: rust
source_hash: blake3:79127007b97586cf17f5ac898737036ed4cc7b3cf8ee745d4890d4254bff3eab
lines: 151
fragments: 4
---
# `codemaid_corpus::sequence` · crates/codemaid-corpus/src/sequence.rs
> Flow → `sequenceDiagram`.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_corpus__sequence__Ctx["Ctx#lt;'a#gt;"] {
    <<struct>>
    -cb: &'a Codebase
    -opts: &'a CorpusOptions
    -caller: SymbolId
    -lanes: Vec#lt;#40;SymbolId, String, bool#41;#gt;
    -budget: usize
    -dropped: usize
    -arms(&mut self, kind: BlockKind, arms: &[codemaid_model::Arm], out: &mut SeqBuilder)
    -lane(&mut self, lane: Lane) SymbolId
    -steps(&mut self, steps: &[Step], out: &mut SeqBuilder)
  }
  class codemaid_corpus__sequence__Rendered["Rendered"] {
    <<struct>>
    +text: String
    +participants: Vec#lt;SymbolId#gt;
    +truncated: usize
  }
  class codemaid_corpus__sequence["codemaid_corpus::sequence"] {
    <<module>>
    ~render(crate) Option#lt;Rendered#gt;
  }
  class codemaid_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/codemaid-corpus/src/lib.rs>>
  }
  class codemaid_model__codebase__Codebase["Codebase"] {
    <<struct in crates/codemaid-model/src/codebase.rs>>
  }
  class codemaid_model__symbol__Symbol["Symbol"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_corpus__naming__Lane["Lane"] {
    <<struct in crates/codemaid-corpus/src/naming.rs>>
  }
  class codemaid_mermaid__sequence__BlockKind["BlockKind"] {
    <<enum in crates/codemaid-mermaid/src/sequence.rs>>
  }
  class codemaid_mermaid__sequence__SeqBuilder["SeqBuilder"] {
    <<struct in crates/codemaid-mermaid/src/sequence.rs>>
  }
  class codemaid_model__flow__Arm["Arm"] {
    <<struct in crates/codemaid-model/src/flow.rs>>
  }
  class codemaid_model__flow__Step["Step"] {
    <<enum in crates/codemaid-model/src/flow.rs>>
  }
  class codemaid_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  codemaid_corpus__sequence ..> codemaid_corpus__CorpusOptions
  codemaid_corpus__sequence ..> codemaid_corpus__sequence__Rendered
  codemaid_corpus__sequence ..> codemaid_model__codebase__Codebase
  codemaid_corpus__sequence ..> codemaid_model__symbol__Symbol
  codemaid_corpus__sequence__Ctx o-- codemaid_corpus__CorpusOptions : opts
  codemaid_corpus__sequence__Ctx ..> codemaid_corpus__naming__Lane
  codemaid_corpus__sequence__Ctx ..> codemaid_mermaid__sequence__BlockKind
  codemaid_corpus__sequence__Ctx ..> codemaid_mermaid__sequence__SeqBuilder
  codemaid_corpus__sequence__Ctx o-- codemaid_model__codebase__Codebase : cb
  codemaid_corpus__sequence__Ctx ..> codemaid_model__flow__Arm
  codemaid_corpus__sequence__Ctx ..> codemaid_model__flow__Step
  codemaid_corpus__sequence__Ctx o-- codemaid_model__symbol__SymbolId : caller, lanes
  codemaid_corpus__sequence__Rendered o-- codemaid_model__symbol__SymbolId : participants
```

## `codemaid_corpus::sequence::render`
`pub(crate) fn render(cb: &Codebase, sym: &Symbol, opts: &CorpusOptions) -> Option<Rendered>` · L17-L47
```mermaid
sequenceDiagram
  participant codemaid_corpus__sequence as sequence mod
  participant codemaid_model__codebase__Codebase as Codebase
  participant codemaid_corpus__naming as naming mod
  participant codemaid_corpus__sequence__Ctx as Ctx
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  participant codemaid_mermaid__sequence__SequenceDiagram as SequenceDiagram
  opt flow.call_count()<opts.min_calls
    Note over codemaid_corpus__sequence: return None
  end
  codemaid_corpus__sequence->>codemaid_model__codebase__Codebase: owner_of(&sym.id)
  codemaid_corpus__sequence->>codemaid_corpus__naming: alias_for(cb, &caller)
  codemaid_corpus__sequence->>codemaid_corpus__sequence__Ctx: lane(_)
  codemaid_corpus__sequence->>codemaid_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
  codemaid_corpus__sequence->>codemaid_corpus__sequence__Ctx: steps(&flow.steps, &body)
  codemaid_corpus__sequence->>codemaid_mermaid__sequence__SequenceDiagram: SequenceDiagram::new()
  loop for (id, alias, external) in &ctx.lanes
    alt * external
      codemaid_corpus__sequence->>codemaid_corpus__naming: ident(id)
      codemaid_corpus__sequence->>codemaid_mermaid__sequence__SequenceDiagram: participant(ident(), &_)
    else
      codemaid_corpus__sequence->>codemaid_corpus__naming: ident(id)
      codemaid_corpus__sequence->>codemaid_mermaid__sequence__SequenceDiagram: participant(ident(), alias)
    end
  end
  opt ctx.dropped> 0
    codemaid_corpus__sequence->>codemaid_corpus__naming: ident(&_.0)
  end
  codemaid_corpus__sequence->>codemaid_mermaid__sequence__SequenceDiagram: render()
```

## `codemaid_corpus::sequence::Ctx::steps`
`fn steps(&mut self, steps: &[Step], out: &mut SeqBuilder)` · L66-L118
```mermaid
sequenceDiagram
  participant codemaid_corpus__sequence__Ctx as Ctx
  participant codemaid_corpus__naming as naming mod
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  codemaid_corpus__sequence__Ctx->>codemaid_corpus__naming: ident(&self.caller)
  loop for step in steps
    alt Step::Call(c)
      codemaid_corpus__sequence__Ctx->>codemaid_corpus__naming: lane_of(self.cb, &c.target, self.opts)
      codemaid_corpus__sequence__Ctx->>codemaid_corpus__sequence__Ctx: lane(lane)
      codemaid_corpus__sequence__Ctx->>codemaid_corpus__naming: ident(&lane())
      codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: message(&me, &to, Sync, &text)
    else Step::Branch { arms }
      codemaid_corpus__sequence__Ctx->>codemaid_corpus__sequence__Ctx: arms(Alt, arms, out)
    else Step::Parallel { arms }
      codemaid_corpus__sequence__Ctx->>codemaid_corpus__sequence__Ctx: arms(Par, arms, out)
    else Step::Loop { label, body }
      codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
      codemaid_corpus__sequence__Ctx->>codemaid_corpus__sequence__Ctx: steps(body, &inner)
      codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: is_empty()
      opt !inner.is_empty()
        codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: block(Loop, label, |..|)
      end
    else Step::Optional { label, body }
      codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
      codemaid_corpus__sequence__Ctx->>codemaid_corpus__sequence__Ctx: steps(body, &inner)
      codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: is_empty()
      opt !inner.is_empty()
        codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: block(Opt, label, |..|)
      end
    else Step::Return(exit)
      codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: note(&_, &exit.label)
    end
  end
```

## `codemaid_corpus::sequence::Ctx::arms`
`fn arms(&mut self, kind: BlockKind, arms: &[codemaid_model::Arm], out: &mut SeqBuilder)` · L120-L150
```mermaid
sequenceDiagram
  participant codemaid_corpus__sequence__Ctx as Ctx
  participant codemaid_mermaid__sequence__SeqBuilder as SeqBuilder
  loop for arm in arms
    codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: ~SeqBuilder::default()
    codemaid_corpus__sequence__Ctx->>codemaid_corpus__sequence__Ctx: steps(&arm.steps, &inner)
    codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: is_empty()
  end
  opt built.is_empty()
    Note over codemaid_corpus__sequence__Ctx: return
  end
  opt built.len() == 1 && kind == BlockKind::Alt
    codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: block(Opt, &label, |..|)
    Note over codemaid_corpus__sequence__Ctx: return
  end
  codemaid_corpus__sequence__Ctx->>codemaid_mermaid__sequence__SeqBuilder: arms(kind, arms)
```
