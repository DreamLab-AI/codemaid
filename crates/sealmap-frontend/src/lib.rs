//! # sealmap-frontend
//!
//! The language-neutral half of a sealmap frontend. A frontend (the syn-based
//! `sealmap-rust`, or a future TypeScript one) parses source and walks
//! function bodies; everything after that is shared and lives here, so two
//! frontends produce flows, labels, ids and confidences by the same rules:
//!
//! | Module | Owns |
//! |---|---|
//! | [`raw`] | the raw flow IR a parser emits before resolution ([`RawStep`], [`RawCall`], [`Callee`], [`Recv`]) and the helpers that shape it while walking |
//! | [`lower`] | flow normalisation: raw steps plus a call resolver → a model [`Flow`](sealmap_model::Flow) |
//! | [`confidence`] | the confidence lattice, the external-call policy ([`ExternalCalls`]) and call-edge aggregation |
//! | [`labels`] | label rules: compact token spacing, clipping limits, call, condition and deferred-closure labels |
//! | [`ids`] | the symbol-id builder (module, item, method and trait-impl ids) |
//! | [`isolate`] | per-file collection on big stacks with a panic guard |
//!
//! It has no parser dependency; a frontend brings its own.
//!
//! ## Example
//!
//! A frontend has walked `fn run() { if ready() { go() } }` and resolves
//! every path call to an internal function:
//!
//! ```
//! use sealmap_frontend::raw::{Callee, RawCall, RawStep};
//! use sealmap_frontend::{ids, lower};
//! use sealmap_model::{CallKind, Confidence, Step};
//!
//! let call = |name: &str, line| {
//!     RawStep::Call(RawCall::new(Callee::Path(vec![name.into()]), format!("{name}()"), CallKind::Function, line))
//! };
//! let raw = vec![call("ready", 1), RawStep::Branch(vec![("ready()".into(), vec![call("go", 1)])])];
//!
//! let flow = lower::lower_flow(&raw, &mut |c: &RawCall| {
//!     let Callee::Path(segs) = &c.callee else { return None };
//!     Some(c.to_call(ids::item_id("app", &segs[0]), Confidence::Exact))
//! })
//! .unwrap();
//!
//! let targets: Vec<_> = flow.calls().map(|c| c.target.as_str()).collect();
//! assert_eq!(targets, ["app::ready", "app::go"]);
//! // A branch with one surviving arm is an optional fragment.
//! assert!(matches!(&flow.steps[1], Step::Optional { label, .. } if label == "ready()"));
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod confidence;
pub mod ids;
pub mod isolate;
pub mod labels;
pub mod lower;
pub mod raw;

use sealmap_model::{Codebase, SourcePath};

pub use confidence::{ExternalCalls, aggregate_calls};
pub use raw::{Callee, RawCall, RawStep, Recv, Segs};

/// A non-fatal problem found while extracting (currently: files that failed
/// to parse; they still get a module symbol so every file is represented).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// File the problem is in.
    pub file: SourcePath,
    /// Human-readable message with `line:col` prefix.
    pub message: String,
}

/// Result of a frontend's extraction.
#[derive(Debug, Clone)]
pub struct Extraction {
    /// The model.
    pub codebase: Codebase,
    /// Non-fatal problems, in file order.
    pub diagnostics: Vec<Diagnostic>,
}
