//! Flow normalisation: lower a raw flow to a model [`Flow`] once calls can be
//! resolved.
//!
//! The frontend supplies the resolver as a closure; this module owns the
//! shape rules, which are the same for every language:
//!
//! - a call the resolver drops (returns `None`) disappears;
//! - a fragment whose body ends up empty disappears;
//! - a branch left with one non-empty arm becomes an optional fragment,
//!   labelled with its condition (or `not <first condition>` for an `else`);
//! - a trailing top-level `return` is the end of the function and is dropped;
//! - a flow with nothing but returns says nothing about calls and is `None`.
//!
//! ```
//! use sealmap_frontend::lower::lower_flow;
//! use sealmap_frontend::raw::{Callee, RawCall, RawStep};
//! use sealmap_model::{CallKind, Confidence, Step, SymbolId};
//!
//! let call = |n: &str| RawStep::Call(RawCall::new(Callee::Path(vec![n.into()]), n.into(), CallKind::Function, 1));
//! // if cached { log() } else { fetch() }  — `log` is std, so it is dropped.
//! let raw = vec![RawStep::Branch(vec![("cached".into(), vec![call("log")]), (String::new(), vec![call("fetch")])])];
//! let flow = lower_flow(&raw, &mut |c: &RawCall| match &c.callee {
//!     Callee::Path(s) if s[0] == "fetch" => Some(c.to_call(SymbolId::unresolved("fetch"), Confidence::Exact)),
//!     _ => None,
//! })
//! .unwrap();
//! assert!(matches!(&flow.steps[0], Step::Optional { label, .. } if label == "not cached"));
//! ```

use sealmap_model::{Arm, Call, Exit, Flow, Step};

use crate::raw::{RawCall, RawStep};

/// Lower `raw` to a [`Flow`], resolving each call with `call` (which returns
/// `None` to drop it). Returns `None` when nothing but returns is left.
pub fn lower_flow<F>(raw: &[RawStep], call: &mut F) -> Option<Flow>
where
    F: FnMut(&RawCall) -> Option<Call>,
{
    let mut steps = lower_steps(raw, call);
    // A trailing top-level `return` is just the end of the function.
    if matches!(steps.last(), Some(Step::Return(_))) {
        steps.pop();
    }
    // A flow that only returns says nothing about calls.
    if steps.iter().all(|s| matches!(s, Step::Return(_))) {
        return None;
    }
    Some(Flow::new(steps))
}

/// Lower a step list without the top-level rules of [`lower_flow`].
pub fn lower_steps<F>(raw: &[RawStep], call: &mut F) -> Vec<Step>
where
    F: FnMut(&RawCall) -> Option<Call>,
{
    let mut out = Vec::new();
    for s in raw {
        match s {
            RawStep::Call(c) => {
                if let Some(c) = call(c) {
                    out.push(Step::Call(c));
                }
            }
            RawStep::Branch(raw_arms) => {
                let first_label = raw_arms.first().map(|(l, _)| l.clone()).unwrap_or_default();
                let all: Vec<(usize, Arm)> = raw_arms
                    .iter()
                    .enumerate()
                    .map(|(i, (label, steps))| (i, Arm { label: label.clone(), steps: lower_steps(steps, call) }))
                    .filter(|(_, a)| !a.steps.is_empty())
                    .collect();
                match all.len() {
                    0 => {}
                    1 => {
                        // One arm left: say which condition guards it.
                        let (i, arm) = all.into_iter().next().expect("one arm");
                        let label = if i > 0 && arm.label.is_empty() {
                            format!("not {first_label}")
                        } else {
                            arm.label.trim_start_matches("if ").to_owned()
                        };
                        out.push(Step::Optional { label, body: arm.steps });
                    }
                    _ => out.push(Step::Branch { arms: all.into_iter().map(|(_, a)| a).collect() }),
                }
            }
            RawStep::Parallel(arms) => {
                let arms = lower_arms(arms, call);
                if arms.iter().any(|a| !a.steps.is_empty()) {
                    out.push(Step::Parallel { arms });
                }
            }
            RawStep::Loop(label, body) => {
                let body = lower_steps(body, call);
                if !body.is_empty() {
                    out.push(Step::Loop { label: label.clone(), body });
                }
            }
            RawStep::Optional(label, body) => {
                let body = lower_steps(body, call);
                if !body.is_empty() {
                    out.push(Step::Optional { label: label.clone(), body });
                }
            }
            RawStep::Return(label, line) => out.push(Step::Return(Exit { label: label.clone(), line: *line })),
        }
    }
    out
}

fn lower_arms<F>(arms: &[(String, Vec<RawStep>)], call: &mut F) -> Vec<Arm>
where
    F: FnMut(&RawCall) -> Option<Call>,
{
    arms.iter()
        .map(|(label, steps)| Arm { label: label.clone(), steps: lower_steps(steps, call) })
        .filter(|a| !a.steps.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use sealmap_model::{CallKind, Confidence, SymbolId};

    use super::*;
    use crate::raw::Callee;

    fn call(n: &str) -> RawStep {
        RawStep::Call(RawCall::new(Callee::Path(vec![n.into()]), n.into(), CallKind::Function, 1))
    }

    fn keep_all(c: &RawCall) -> Option<Call> {
        let Callee::Path(s) = &c.callee else { return None };
        (s[0] != "drop").then(|| c.to_call(SymbolId::unresolved(&s[0]), Confidence::Exact))
    }

    #[test]
    fn trailing_return_and_return_only_flows() {
        let raw = vec![call("a"), RawStep::Return("return x".into(), 2)];
        assert_eq!(lower_flow(&raw, &mut keep_all).unwrap().steps.len(), 1);
        assert!(lower_flow(&[RawStep::Return("return".into(), 1)], &mut keep_all).is_none());
        assert!(lower_flow(&[call("drop")], &mut keep_all).is_none());
    }

    #[test]
    fn branch_arms_collapse_after_resolution() {
        let raw = vec![RawStep::Branch(vec![
            ("x".into(), vec![call("drop")]),
            ("if y".into(), vec![call("b")]),
            (String::new(), vec![call("drop")]),
        ])];
        let flow = lower_flow(&raw, &mut keep_all).unwrap();
        assert!(matches!(&flow.steps[0], Step::Optional { label, .. } if label == "y"));

        let raw = vec![RawStep::Branch(vec![("x".into(), vec![call("a")]), (String::new(), vec![call("b")])])];
        let flow = lower_flow(&raw, &mut keep_all).unwrap();
        assert!(matches!(&flow.steps[0], Step::Branch { arms } if arms.len() == 2));
    }

    #[test]
    fn empty_fragments_disappear() {
        let raw = vec![
            RawStep::Loop("l".into(), vec![call("drop")]),
            RawStep::Optional("o".into(), vec![]),
            RawStep::Parallel(vec![("p".into(), vec![call("drop")])]),
            call("a"),
        ];
        assert_eq!(lower_flow(&raw, &mut keep_all).unwrap().steps.len(), 1);
    }
}
