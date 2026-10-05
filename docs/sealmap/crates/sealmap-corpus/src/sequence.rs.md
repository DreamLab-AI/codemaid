---
sealmap: 2
source: crates/sealmap-corpus/src/sequence.rs
module: "sym:cargo sealmap_corpus . sequence/"
language: rust
source_hash: blake3:f68162dfb4d79fe2296e2b5413dc685629c4eafe8a2b1a6216fae29333548389
lines: 156
fragments: 4
---
# `sym:cargo sealmap_corpus . sequence/` · crates/sealmap-corpus/src/sequence.rs
> Flow → `sequenceDiagram`.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__sequence___tCtx["Ctx#lt;'a#gt;"] {
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
  class sealmap_corpus__sequence___tRendered["Rendered"] {
    <<struct>>
    +text: String
    +participants: Vec#lt;SymbolId#gt;
    +truncated: usize
  }
  class sealmap_corpus__sequence["sealmap_corpus::sequence"] {
    <<module>>
    ~render(crate) Option#lt;Rendered#gt;
  }
  class sealmap_corpus___tCorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__symbol___tSymbol["Symbol"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_corpus__naming___tLane["Lane"] {
    <<struct in crates/sealmap-corpus/src/naming.rs>>
  }
  class sealmap_mermaid__sequence___tBlockKind["BlockKind"] {
    <<enum in crates/sealmap-mermaid/src/sequence.rs>>
  }
  class sealmap_mermaid__sequence___tSeqBuilder["SeqBuilder"] {
    <<struct in crates/sealmap-mermaid/src/sequence.rs>>
  }
  class sealmap_model__flow___tArm["Arm"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__flow___tStep["Step"] {
    <<enum in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  sealmap_corpus__sequence ..> sealmap_corpus___tCorpusOptions
  sealmap_corpus__sequence ..> sealmap_corpus__sequence___tRendered
  sealmap_corpus__sequence ..> sealmap_model__codebase___tCodebase
  sealmap_corpus__sequence ..> sealmap_model__symbol___tSymbol
  sealmap_corpus__sequence___tCtx o-- sealmap_corpus___tCorpusOptions : opts
  sealmap_corpus__sequence___tCtx ..> sealmap_corpus__naming___tLane
  sealmap_corpus__sequence___tCtx ..> sealmap_mermaid__sequence___tBlockKind
  sealmap_corpus__sequence___tCtx ..> sealmap_mermaid__sequence___tSeqBuilder
  sealmap_corpus__sequence___tCtx o-- sealmap_model__codebase___tCodebase : cb
  sealmap_corpus__sequence___tCtx ..> sealmap_model__flow___tArm
  sealmap_corpus__sequence___tCtx ..> sealmap_model__flow___tStep
  sealmap_corpus__sequence___tCtx o-- sealmap_model__sym___tSymbolId : caller, lanes
  sealmap_corpus__sequence___tRendered o-- sealmap_model__sym___tSymbolId : participants
```

## `sym:cargo sealmap_corpus . sequence/render().`
`pub(crate) fn render(cb: &Codebase, sym: &Symbol, opts: &CorpusOptions) -> Option<Rendered>` · L17-L47
```mermaid
sequenceDiagram
  participant sealmap_corpus__sequence as sequence mod
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_corpus__naming as naming mod
  participant sealmap_corpus__sequence___tCtx as Ctx
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  participant sealmap_mermaid__sequence___tSequenceDiagram as SequenceDiagram
  opt flow.call_count()<opts.min_calls
    Note over sealmap_corpus__sequence: return None
  end
  sealmap_corpus__sequence->>sealmap_model__codebase___tCodebase: owner_of(&sym.id)
  sealmap_corpus__sequence->>sealmap_corpus__naming: alias_for(cb, &caller)
  sealmap_corpus__sequence->>sealmap_corpus__sequence___tCtx: lane(_)
  sealmap_corpus__sequence->>sealmap_mermaid__sequence___tSeqBuilder: ~SeqBuilder::default()
  sealmap_corpus__sequence->>sealmap_corpus__sequence___tCtx: steps(&flow.steps, &body)
  sealmap_corpus__sequence->>sealmap_mermaid__sequence___tSequenceDiagram: SequenceDiagram::new()
  loop for (id, alias, external) in &ctx.lanes
    alt * external
      sealmap_corpus__sequence->>sealmap_corpus__naming: ident(id)
      sealmap_corpus__sequence->>sealmap_mermaid__sequence___tSequenceDiagram: participant(ident(), &_)
    else
      sealmap_corpus__sequence->>sealmap_corpus__naming: ident(id)
      sealmap_corpus__sequence->>sealmap_mermaid__sequence___tSequenceDiagram: participant(ident(), alias)
    end
  end
  opt ctx.dropped> 0
    sealmap_corpus__sequence->>sealmap_corpus__naming: ident(&_.0)
  end
  sealmap_corpus__sequence->>sealmap_mermaid__sequence___tSequenceDiagram: render()
```

## `sym:cargo sealmap_corpus . sequence/Ctx#steps().`
`fn steps(&mut self, steps: &[Step], out: &mut SeqBuilder)` · L66-L123
```mermaid
sequenceDiagram
  participant sealmap_corpus__sequence___tCtx as Ctx
  participant sealmap_corpus__naming as naming mod
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  sealmap_corpus__sequence___tCtx->>sealmap_corpus__naming: ident(&self.caller)
  loop for step in steps
    alt Step::Call(c)
      sealmap_corpus__sequence___tCtx->>sealmap_corpus__naming: lane_of(self.cb, &c.target, self.opts)
      sealmap_corpus__sequence___tCtx->>sealmap_corpus__sequence___tCtx: lane(lane)
      sealmap_corpus__sequence___tCtx->>sealmap_corpus__naming: ident(&lane())
      sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: message(&me, &to, Sync, &text)
    else Step::Branch { arms }
      sealmap_corpus__sequence___tCtx->>sealmap_corpus__sequence___tCtx: arms(Alt, arms, out)
    else Step::Parallel { arms }
      sealmap_corpus__sequence___tCtx->>sealmap_corpus__sequence___tCtx: arms(Par, arms, out)
    else Step::Loop { label, body }
      sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: ~SeqBuilder::default()
      sealmap_corpus__sequence___tCtx->>sealmap_corpus__sequence___tCtx: steps(body, &inner)
      sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: is_empty()
      opt !inner.is_empty()
        sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: block(Loop, label, |..|)
      end
    else Step::Optional { label, body }
      sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: ~SeqBuilder::default()
      sealmap_corpus__sequence___tCtx->>sealmap_corpus__sequence___tCtx: steps(body, &inner)
      sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: is_empty()
      opt !inner.is_empty()
        sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: block(Opt, label, |..|)
      end
    else Step::Return(exit)
      sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: note(&_, &exit.label)
    end
  end
```

## `sym:cargo sealmap_corpus . sequence/Ctx#arms().`
`fn arms(&mut self, kind: BlockKind, arms: &[sealmap_model::Arm], out: &mut SeqBuilder)` · L125-L155
```mermaid
sequenceDiagram
  participant sealmap_corpus__sequence___tCtx as Ctx
  participant sealmap_mermaid__sequence___tSeqBuilder as SeqBuilder
  loop for arm in arms
    sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: ~SeqBuilder::default()
    sealmap_corpus__sequence___tCtx->>sealmap_corpus__sequence___tCtx: steps(&arm.steps, &inner)
    sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: is_empty()
  end
  opt built.is_empty()
    Note over sealmap_corpus__sequence___tCtx: return
  end
  opt built.len() == 1 && kind == BlockKind::Alt
    sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: block(Opt, &label, |..|)
    Note over sealmap_corpus__sequence___tCtx: return
  end
  sealmap_corpus__sequence___tCtx->>sealmap_mermaid__sequence___tSeqBuilder: arms(kind, arms)
```
