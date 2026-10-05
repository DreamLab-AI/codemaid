//! The confidence lattice, the external-call policy and call-edge
//! aggregation.
//!
//! [`Confidence`] is totally ordered from strongest to weakest evidence:
//! `Exact < Inferred < External`. Where one function calls the same target
//! several times, the call edge keeps the **strongest** evidence any of those
//! calls had ([`strongest`]).
//!
//! ```
//! use sealmap_extract::confidence::{ExternalCalls, strongest};
//! use sealmap_model::Confidence;
//!
//! assert_eq!(strongest(Confidence::Inferred, Confidence::Exact), Confidence::Exact);
//! // External calls are kept under the default policy only if they look
//! // like a dependency (the language adapter decides what that means).
//! assert!(ExternalCalls::NonStd.keeps(Confidence::External, || true));
//! assert!(!ExternalCalls::NonStd.keeps(Confidence::External, || false));
//! assert!(ExternalCalls::None.keeps(Confidence::Inferred, || false));
//! ```

use std::collections::BTreeMap;

use sealmap_model::{Codebase, Confidence, Relation, RelationKind, SymbolId};

/// Which calls to external code are kept in flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExternalCalls {
    /// Keep every external call, including `std` and unresolved method calls
    /// (`?::name`). Verbose; useful for debugging the language adapter.
    All,
    /// Keep calls into non-std dependencies (`tokio::spawn`,
    /// `serde_json::to_string`); drop `std`/`core`/`alloc` and unresolved
    /// method calls. The default.
    #[default]
    NonStd,
    /// Keep only calls into the analysed code.
    None,
}

impl ExternalCalls {
    /// Whether a call resolved with `confidence` stays in the flow. Exact and
    /// inferred calls always stay; an external call stays under
    /// [`ExternalCalls::NonStd`] only if `is_dependency` says its target is a
    /// real dependency (not the standard library, not an unresolved name).
    /// `is_dependency` is only evaluated when needed.
    pub fn keeps(self, confidence: Confidence, is_dependency: impl FnOnce() -> bool) -> bool {
        match confidence {
            Confidence::Exact | Confidence::Inferred => true,
            Confidence::External => match self {
                ExternalCalls::All => true,
                ExternalCalls::NonStd => is_dependency(),
                ExternalCalls::None => false,
            },
        }
    }
}

/// The stronger of two pieces of evidence for the same edge.
pub fn strongest(a: Confidence, b: Confidence) -> Confidence {
    a.min(b)
}

/// Add one [`RelationKind::Calls`] relation per distinct (caller, target)
/// pair found in the flows of `cb`, with the strongest confidence among the
/// calls behind it.
pub fn aggregate_calls(cb: &mut Codebase) {
    let mut calls: BTreeMap<(SymbolId, SymbolId), Confidence> = BTreeMap::new();
    for s in cb.symbols.values() {
        if let Some(flow) = &s.flow {
            for c in flow.calls() {
                let e = calls.entry((s.id.clone(), c.target.clone())).or_insert(c.confidence);
                *e = strongest(*e, c.confidence);
            }
        }
    }
    for ((from, to), c) in calls {
        cb.add_relation(Relation::new(from, to, RelationKind::Calls, c));
    }
}

#[cfg(test)]
mod tests {
    use sealmap_model::{Call, CallKind, Flow, SourcePath, Step, Symbol, SymbolKind};

    use super::*;

    fn sym(s: &str) -> SymbolId {
        SymbolId::parse(s).unwrap()
    }

    fn call(t: &str, c: Confidence) -> Step {
        Step::Call(Call {
            target: SymbolId::parse(t).unwrap(),
            label: t.into(),
            kind: CallKind::Function,
            confidence: c,
            awaited: false,
            fallible: false,
            line: 1,
        })
    }

    #[test]
    fn aggregation_keeps_the_strongest_evidence() {
        let mut cb = Codebase::new("app");
        let mut s =
            Symbol::new(sym("sym:cargo app . f()."), "f", SymbolKind::Function, SourcePath::new("src/lib.rs").unwrap());
        s.flow = Some(Flow::new(vec![
            call("sym:cargo app . g().", Confidence::Inferred),
            call("sym:cargo app . g().", Confidence::Exact),
            call("sym:extern dep::h", Confidence::External),
        ]));
        cb.add_symbol(s);
        aggregate_calls(&mut cb);
        let edges: Vec<_> = cb.relations.iter().map(|r| (r.to.to_string(), r.confidence)).collect();
        assert_eq!(
            edges,
            [
                ("sym:cargo app . g().".to_owned(), Confidence::Exact),
                ("sym:extern dep::h".to_owned(), Confidence::External)
            ]
        );
    }

    #[test]
    fn policy_table() {
        for p in [ExternalCalls::All, ExternalCalls::NonStd, ExternalCalls::None] {
            assert!(p.keeps(Confidence::Exact, || unreachable!()));
            assert!(p.keeps(Confidence::Inferred, || unreachable!()));
        }
        assert!(ExternalCalls::All.keeps(Confidence::External, || false));
        assert!(!ExternalCalls::None.keeps(Confidence::External, || true));
    }
}
