//! # codemaid-mermaid
//!
//! Typed, deterministic writers for the Mermaid diagram kinds that explain
//! code best: [`SequenceDiagram`] (behaviour), [`ClassDiagram`] (structure),
//! [`ErDiagram`] (data) and [`Flowchart`] (dependencies and topology, from
//! module level up to multi-repository maps).
//!
//! The crate has **no dependencies** and knows nothing about code models; it
//! is a safe Mermaid emitter you can use on its own. `codemaid-corpus` uses it
//! to project a `codemaid_model::Codebase` into diagrams.
//!
//! ## Why typed builders instead of `format!`
//!
//! Mermaid's grammar has sharp edges that string concatenation trips over:
//! `;` ends a statement inside sequence messages, `end` is a keyword in node
//! ids, `{`/`}` close class bodies, `(` turns a class member into a method,
//! `<`/`>` collide with generics, and `#` starts an entity code. Escaping
//! uses Mermaid entity codes (`#lt;`, `#59;`, ...), which render as the
//! original characters. Every value
//! that reaches the output here goes through [`Ident`] (for ids) or
//! [`escape_text`] (for labels), so arbitrary source text cannot break a
//! diagram.
//!
//! ## Determinism and density
//!
//! * Rendering is a pure function of the builder: same calls, same bytes.
//!   Builders preserve insertion order for statements (sequence order *is*
//!   meaning) and you control ordering of declarations. Nothing is sorted
//!   behind your back and nothing depends on hash order.
//! * Output uses two-space indentation, `\n` line endings, a trailing newline
//!   and no trailing whitespace.
//! * Output is minimal: no styling, no redundant arrows, no init directives
//!   unless you add them. Density is the default; decoration is opt-in.
//!
//! ## Example
//!
//! ```
//! use codemaid_mermaid::{Arrow, Ident, SequenceDiagram};
//!
//! let api = Ident::from_path("app::Api");
//! let db = Ident::from_path("app::Db");
//!
//! let mut seq = SequenceDiagram::new();
//! seq.participant(api.clone(), "Api");
//! seq.participant(db.clone(), "Db");
//! seq.message(&api, &db, Arrow::Sync, "query(sql); fetch");
//! seq.loop_block("for row in rows", |b| {
//!     b.message(&api, &api, Arrow::Sync, "decode(row)?");
//! });
//!
//! assert_eq!(
//!     seq.render(),
//!     "sequenceDiagram\n\
//!      \x20 participant app__Api as Api\n\
//!      \x20 participant app__Db as Db\n\
//!      \x20 app__Api->>app__Db: query(sql)#59; fetch\n\
//!      \x20 loop for row in rows\n\
//!      \x20   app__Api->>app__Api: decode(row)?\n\
//!      \x20 end\n"
//! );
//! ```

#![forbid(unsafe_code)]

mod class;
mod er;
mod escape;
mod flowchart;
mod sequence;
mod writer;

pub use class::{Class, ClassDiagram, ClassRelation, ClassRelationKind, Direction};
pub use er::{Cardinality, ErDiagram};
pub use escape::{Ident, escape_text, escape_type};
pub use flowchart::{EdgeStyle, Flowchart, NodeShape};
pub use sequence::{Arrow, BlockKind, SeqBuilder, SequenceDiagram};
pub use writer::CodeWriter;
