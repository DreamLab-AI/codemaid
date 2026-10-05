//! Ordered call/control flow of a function body: the raw material for
//! sequence diagrams.
//!
//! A [`Flow`] is a tree of [`Step`]s in **source order**. Calls are leaves;
//! branches, loops and optional blocks are interior nodes that map one-to-one
//! onto Mermaid `alt` / `else`, `loop` and `opt` fragments. Frontends drop
//! anything that contains no calls, so a flow is already the minimal skeleton
//! of "who talks to whom, in what order, under which condition".
//!
//! Every [`Call::target`] is a canonical [`SymbolId`]. When the target is
//! itself a callable in the codebase, its own flow is a separate fragment
//! with the same id, which is what lets an orchestrator *inline* one sequence
//! into another, or stitch fragments across files, without guessing.
//!
//! ```
//! use sealmap_model::*;
//!
//! let flow = Flow::new(vec![
//!     Step::Call(Call::new(SymbolId::new("app::Db::open"), "open", Confidence::Exact)),
//!     Step::Loop {
//!         label: "for row in rows".into(),
//!         body: vec![Step::Call(Call::new(
//!             SymbolId::new("app::Db::insert"),
//!             "insert",
//!             Confidence::Inferred,
//!         ))],
//!     },
//! ]);
//! assert_eq!(flow.call_count(), 2);
//! assert_eq!(flow.calls().map(|c| c.target.name()).collect::<Vec<_>>(), ["open", "insert"]);
//! ```

use serde::{Deserialize, Serialize};

use crate::symbol::{Confidence, SymbolId};

/// The ordered call/control skeleton of one callable body.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Flow {
    /// Top-level steps in source order.
    pub steps: Vec<Step>,
}

impl Flow {
    /// Wrap a list of steps.
    pub fn new(steps: Vec<Step>) -> Self {
        Self { steps }
    }

    /// `true` when the body makes no calls at all.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Every call in depth-first source order.
    pub fn calls(&self) -> impl Iterator<Item = &Call> {
        let mut out = Vec::new();
        collect_calls(&self.steps, &mut out);
        out.into_iter()
    }

    /// Number of calls in the whole tree.
    pub fn call_count(&self) -> usize {
        self.calls().count()
    }

    /// Maximum nesting depth of control fragments (0 for a flat sequence).
    pub fn depth(&self) -> usize {
        fn depth(steps: &[Step]) -> usize {
            steps
                .iter()
                .map(|s| match s {
                    Step::Call(_) | Step::Return(_) => 0,
                    Step::Branch { arms, .. } => 1 + arms.iter().map(|a| depth(&a.steps)).max().unwrap_or(0),
                    Step::Loop { body, .. } | Step::Optional { body, .. } => 1 + depth(body),
                    Step::Parallel { arms } => 1 + arms.iter().map(|a| depth(&a.steps)).max().unwrap_or(0),
                })
                .max()
                .unwrap_or(0)
        }
        depth(&self.steps)
    }
}

fn collect_calls<'a>(steps: &'a [Step], out: &mut Vec<&'a Call>) {
    for step in steps {
        match step {
            Step::Call(c) => out.push(c),
            Step::Branch { arms, .. } | Step::Parallel { arms } => {
                for arm in arms {
                    collect_calls(&arm.steps, out);
                }
            }
            Step::Loop { body, .. } | Step::Optional { body, .. } => collect_calls(body, out),
            Step::Return(_) => {}
        }
    }
}

/// One node of a [`Flow`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "step", rename_all = "snake_case")]
pub enum Step {
    /// A call. Rendered as a message arrow. Calls nested in receivers and
    /// arguments appear as separate, earlier steps (evaluation order).
    Call(Call),
    /// Mutually exclusive arms (`if`/`else if`/`else`, `match`).
    /// Rendered as `alt` / `else`.
    Branch {
        /// Arms in source order. Arms without calls are kept (with empty
        /// steps) only when they matter for reading the branch, e.g. an
        /// `else` that returns early.
        arms: Vec<Arm>,
    },
    /// A loop (`for`, `while`, `loop`, iterator adaptors with closures).
    /// Rendered as `loop`.
    Loop {
        /// Loop header, e.g. `for item in items`.
        label: String,
        /// Body.
        body: Vec<Step>,
    },
    /// A block that may not run (`if` without `else`, `if let`, closures
    /// passed to `map`/`and_then`). Rendered as `opt`.
    Optional {
        /// Condition, e.g. `if let Some(x) = cache.get(k)`.
        label: String,
        /// Body.
        body: Vec<Step>,
    },
    /// Concurrent arms (`join!`, `select!`, spawned tasks).
    /// Rendered as `par` / `and`.
    Parallel {
        /// Arms.
        arms: Vec<Arm>,
    },
    /// An explicit early exit (`return`, `?` propagation, `break` out of a
    /// labelled loop). Rendered as a note or a reply arrow.
    Return(Exit),
}

/// One arm of a [`Step::Branch`] or [`Step::Parallel`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arm {
    /// Condition or pattern, e.g. `Ok(conn)`, `x > 3`, `else`.
    pub label: String,
    /// Steps in the arm.
    pub steps: Vec<Step>,
}

/// How a call is spelled at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallKind {
    /// Path call: `foo()`, `Type::new()`, `module::f()`.
    Function,
    /// Method call: `x.foo()`.
    Method,
    /// Macro invocation that the frontend chose to keep (`println!` is
    /// dropped, `tokio::spawn`-like or user macros are kept).
    Macro,
}

/// A single call site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
    /// Canonical id of the callee.
    pub target: SymbolId,
    /// Message text shown on the arrow (callee name plus a compact argument
    /// sketch, e.g. `insert(key, value)`).
    pub label: String,
    /// How the call is spelled.
    pub kind: CallKind,
    /// How sure the frontend is about `target`.
    pub confidence: Confidence,
    /// `.await`ed at the call site.
    pub awaited: bool,
    /// Followed by `?` (errors propagate to the caller).
    pub fallible: bool,
    /// 1-based line of the call site.
    pub line: u32,
}

impl Call {
    /// A synchronous, infallible function call at line 0. Frontends set the remaining fields.
    pub fn new(target: SymbolId, label: impl Into<String>, confidence: Confidence) -> Self {
        Self {
            target,
            label: label.into(),
            kind: CallKind::Function,
            confidence,
            awaited: false,
            fallible: false,
            line: 0,
        }
    }
}

/// An early exit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exit {
    /// What is returned / propagated, e.g. `Err(e)`, `?`, `None`.
    pub label: String,
    /// 1-based line.
    pub line: u32,
}
