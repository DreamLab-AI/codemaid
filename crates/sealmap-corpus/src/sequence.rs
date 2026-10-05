//! Flow → `sequenceDiagram`.

use sealmap_mermaid::{Arrow, BlockKind, SeqBuilder, SequenceDiagram};
use sealmap_model::{Codebase, Confidence, Step, Symbol, SymbolId};

use crate::CorpusOptions;
use crate::naming::{Lane, alias_for, ident, lane_of};

/// Result of rendering one callable.
pub(crate) struct Rendered {
    pub text: String,
    /// Participant ids in lane order.
    pub participants: Vec<SymbolId>,
    pub truncated: usize,
}

pub(crate) fn render(cb: &Codebase, sym: &Symbol, opts: &CorpusOptions) -> Option<Rendered> {
    let flow = sym.flow.as_ref()?;
    if flow.call_count() < opts.min_calls {
        return None;
    }
    let caller = cb.owner_of(&sym.id).unwrap_or_else(|| sym.id.clone());
    let mut ctx = Ctx { cb, opts, caller: caller.clone(), lanes: Vec::new(), budget: opts.max_messages, dropped: 0 };
    ctx.lane(Lane { alias: alias_for(cb, &caller), id: caller, prefix: String::new(), external: false });

    let mut body = SeqBuilder::default();
    ctx.steps(&flow.steps, &mut body);

    let mut seq = SequenceDiagram::new();
    for (id, alias, external) in &ctx.lanes {
        if *external {
            seq.participant(ident(id), &format!("{alias} ext"));
        } else {
            seq.participant(ident(id), alias);
        }
    }
    seq.extend(body);
    if ctx.dropped > 0 {
        let first = ident(&ctx.lanes[0].0);
        seq.note(&[&first], &format!("+{} more calls in _index.json", ctx.dropped));
    }
    Some(Rendered {
        text: seq.render(),
        participants: ctx.lanes.iter().map(|(id, _, _)| id.clone()).collect(),
        truncated: ctx.dropped,
    })
}

struct Ctx<'a> {
    cb: &'a Codebase,
    opts: &'a CorpusOptions,
    caller: SymbolId,
    lanes: Vec<(SymbolId, String, bool)>,
    budget: usize,
    dropped: usize,
}

impl Ctx<'_> {
    fn lane(&mut self, lane: Lane) -> SymbolId {
        if !self.lanes.iter().any(|(id, _, _)| *id == lane.id) {
            self.lanes.push((lane.id.clone(), lane.alias, lane.external));
        }
        lane.id
    }

    fn steps(&mut self, steps: &[Step], out: &mut SeqBuilder) {
        let me = ident(&self.caller);
        for step in steps {
            match step {
                Step::Call(c) => {
                    if self.budget == 0 {
                        self.dropped += 1;
                        continue;
                    }
                    self.budget -= 1;
                    let lane = lane_of(self.cb, &c.target, self.opts);
                    let prefix = lane.prefix.clone();
                    let to = ident(&self.lane(lane));
                    let mut text = String::new();
                    if c.confidence == Confidence::Inferred {
                        text.push('~');
                    }
                    text.push_str(&prefix);
                    text.push_str(&c.label);
                    if c.awaited {
                        text.push_str(".await");
                    }
                    if c.fallible {
                        text.push('?');
                    }
                    out.message(&me, &to, Arrow::Sync, &text);
                }
                Step::Branch { arms } => self.arms(BlockKind::Alt, arms, out),
                Step::Parallel { arms } => self.arms(BlockKind::Par, arms, out),
                Step::Loop { label, body } => {
                    let mut inner = SeqBuilder::default();
                    self.steps(body, &mut inner);
                    if !inner.is_empty() {
                        out.block(BlockKind::Loop, label, |b| {
                            b.extend(inner);
                        });
                    }
                }
                Step::Optional { label, body } => {
                    let mut inner = SeqBuilder::default();
                    self.steps(body, &mut inner);
                    if !inner.is_empty() {
                        out.block(BlockKind::Opt, label, |b| {
                            b.extend(inner);
                        });
                    }
                }
                Step::Return(exit) => {
                    out.note(&[&me], &exit.label);
                }
            }
        }
    }

    fn arms(&mut self, kind: BlockKind, arms: &[sealmap_model::Arm], out: &mut SeqBuilder) {
        let mut built: Vec<(String, SeqBuilder)> = Vec::new();
        for arm in arms {
            let mut inner = SeqBuilder::default();
            self.steps(&arm.steps, &mut inner);
            if !inner.is_empty() {
                built.push((arm.label.clone(), inner));
            }
        }
        if built.is_empty() {
            return;
        }
        if built.len() == 1 && kind == BlockKind::Alt {
            let (label, inner) = built.pop().expect("one arm");
            out.block(BlockKind::Opt, &label, |b| {
                b.extend(inner);
            });
            return;
        }
        type ArmFn<'x> = Box<dyn FnOnce(&mut SeqBuilder) + 'x>;
        let arms: Vec<(String, ArmFn<'_>)> = built
            .into_iter()
            .map(|(label, inner)| {
                let f: ArmFn<'_> = Box::new(move |b: &mut SeqBuilder| {
                    b.extend(inner);
                });
                (label, f)
            })
            .collect();
        out.arms(kind, arms);
    }
}
