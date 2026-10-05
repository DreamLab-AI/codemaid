//! The raw flow IR: what a frontend records while walking a function body,
//! before any name is resolved.
//!
//! Everything here is owned `String`s and `Vec`s (no parser nodes), so files
//! can be walked in parallel and resolved afterwards in one deterministic
//! pass. [`lower`](crate::lower) turns it into a model
//! [`Flow`](sealmap_model::Flow).
//!
//! ```
//! use sealmap_frontend::raw::{Callee, RawCall, RawStep, last_call_mut};
//! use sealmap_model::CallKind;
//!
//! let mut out = vec![RawStep::Call(RawCall::new(Callee::Path(vec!["load".into()]), "load()".into(), CallKind::Function, 3))];
//! // `load()?` — the walker marks the call it just pushed as fallible.
//! if let Some(RawStep::Call(c)) = last_call_mut(&mut out) {
//!     c.fallible = true;
//! }
//! assert!(matches!(&out[0], RawStep::Call(c) if c.fallible));
//! ```

use sealmap_model::{Call, CallKind, Confidence, SymbolId};

/// Path segments as written, generics stripped: `std::collections::HashMap`
/// is `["std", "collections", "HashMap"]`.
pub type Segs = Vec<String>;

/// One step of an unresolved flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawStep {
    /// A call.
    Call(RawCall),
    /// Alternatives (`if`/`else`, `match`): `(label, steps)` per arm. Empty
    /// arms are kept so that [`lower`](crate::lower) can describe the arm that
    /// survives resolution.
    Branch(Vec<(String, Vec<RawStep>)>),
    /// A loop with its label and body.
    Loop(String, Vec<RawStep>),
    /// A body that may or may not run, with its label.
    Optional(String, Vec<RawStep>),
    /// Concurrent arms (spawned tasks): `(label, steps)` per arm.
    Parallel(Vec<(String, Vec<RawStep>)>),
    /// An early exit: label and 1-based line.
    Return(String, u32),
}

/// An unresolved call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCall {
    /// What is called, as written.
    pub callee: Callee,
    /// Short message text for diagrams (see
    /// [`labels::call_label`](crate::labels::call_label)).
    pub label: String,
    /// How it is called.
    pub kind: CallKind,
    /// Awaited (`.await`).
    pub awaited: bool,
    /// Error-propagating (`?`).
    pub fallible: bool,
    /// 1-based source line.
    pub line: u32,
}

impl RawCall {
    /// A call that is neither awaited nor fallible (the walker sets those
    /// flags afterwards, through [`last_call_mut`]).
    pub fn new(callee: Callee, label: String, kind: CallKind, line: u32) -> Self {
        Self { callee, label, kind, awaited: false, fallible: false, line }
    }

    /// The model call for this raw call, once its target is resolved.
    pub fn to_call(&self, target: SymbolId, confidence: Confidence) -> Call {
        Call {
            target,
            label: self.label.clone(),
            kind: self.kind,
            confidence,
            awaited: self.awaited,
            fallible: self.fallible,
            line: self.line,
        }
    }
}

/// The callee of a [`RawCall`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Callee {
    /// `a::b::c(..)`.
    Path(Segs),
    /// `recv.name(..)`.
    Method {
        /// What the method is called on.
        recv: Recv,
        /// Method name.
        name: String,
    },
}

/// What a method is called on, as far as the walker can tell without types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recv {
    /// `self`.
    SelfValue,
    /// `self.field`.
    SelfField(String),
    /// A local or parameter whose declared/constructed type mentions these
    /// paths (outermost first, e.g. `Arc<Db>` → `[Arc], [Db]`).
    Typed(Vec<Segs>),
    /// A plain variable or field whose type is not evident. Calls on it may
    /// be matched to a distinctive internal method name (`inferred`).
    Untyped,
    /// Anything else (call chains, indexing, literals). Not resolved.
    Unknown,
}

/// Push the arms of an `if`/`else if`/`else` chain: nothing if every arm is
/// empty, an [`RawStep::Optional`] for a lone arm, otherwise a
/// [`RawStep::Branch`] that keeps its empty arms (lowering drops them after
/// resolution and uses their labels to describe what remains).
pub fn push_arms(arms: Vec<(String, Vec<RawStep>)>, out: &mut Vec<RawStep>) {
    if arms.iter().all(|(_, s)| s.is_empty()) {
        return;
    }
    if arms.len() == 1 {
        let (label, steps) = arms.into_iter().next().unwrap_or_default();
        out.push(RawStep::Optional(label, steps));
    } else {
        out.push(RawStep::Branch(arms));
    }
}

/// The last call pushed at this level (fragments pushed after it, such as a
/// deferred closure, are skipped over).
pub fn last_call_mut(out: &mut [RawStep]) -> Option<&mut RawStep> {
    out.iter_mut().rev().find(|s| matches!(s, RawStep::Call(_)))
}

/// Place the bodies of closures passed to `callee` after the call itself,
/// since they run during (not before) it. A `*spawn*` callee makes each body a
/// parallel task, a callee in `looping` (methods whose closure runs once per
/// element) makes it a loop, and anything else an optional fragment. The
/// labels come from [`labels::deferred_label`](crate::labels::deferred_label).
pub fn place_deferred(callee: &str, deferred: Vec<Vec<RawStep>>, looping: &[&str], out: &mut Vec<RawStep>) {
    use crate::labels::{DeferredShape, deferred_label, deferred_shape};
    for steps in deferred {
        let shape = deferred_shape(callee, looping);
        let label = deferred_label(callee, shape);
        out.push(match shape {
            DeferredShape::Parallel => RawStep::Parallel(vec![(label, steps)]),
            DeferredShape::Loop => RawStep::Loop(label, steps),
            DeferredShape::Optional => RawStep::Optional(label, steps),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(n: &str) -> RawStep {
        RawStep::Call(RawCall::new(Callee::Path(vec![n.into()]), n.into(), CallKind::Function, 1))
    }

    #[test]
    fn push_arms_shapes() {
        let mut out = Vec::new();
        push_arms(vec![("a".into(), vec![]), (String::new(), vec![])], &mut out);
        assert!(out.is_empty());
        push_arms(vec![("a".into(), vec![call("f")])], &mut out);
        assert!(matches!(&out[0], RawStep::Optional(l, s) if l == "a" && s.len() == 1));
        push_arms(vec![("a".into(), vec![call("f")]), (String::new(), vec![])], &mut out);
        assert!(matches!(&out[1], RawStep::Branch(arms) if arms.len() == 2));
    }

    #[test]
    fn deferred_bodies_follow_the_callee_shape() {
        let mut out = Vec::new();
        place_deferred("spawn", vec![vec![call("a")]], &["map"], &mut out);
        place_deferred("map", vec![vec![call("b")]], &["map"], &mut out);
        place_deferred("with", vec![vec![call("c")]], &["map"], &mut out);
        place_deferred("", vec![vec![call("d")]], &["map"], &mut out);
        assert!(matches!(&out[0], RawStep::Parallel(arms) if arms[0].0 == "spawned task"));
        assert!(matches!(&out[1], RawStep::Loop(l, _) if l == "each via map"));
        assert!(matches!(&out[2], RawStep::Optional(l, _) if l == "via with"));
        assert!(matches!(&out[3], RawStep::Optional(l, _) if l == "closure"));
    }
}
