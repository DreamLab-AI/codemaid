//! # sealmap-model
//!
//! A language-neutral, **deterministic** model of a codebase: source files,
//! the symbols they define, and the typed relations between those symbols.
//!
//! This crate is the shared vocabulary of the `sealmap` workspace. Language
//! frontends (such as `sealmap-rust`) *produce* a [`Codebase`]; projections
//! (such as `sealmap-corpus`) *consume* one and render Mermaid diagrams from
//! it. Because both sides only meet here, you can add a frontend for another
//! language, or a new diagram projection, without touching the other half.
//!
//! The navigation API is loosely inspired by
//! [ts-morph](https://github.com/dsherret/ts-morph) (MIT, © David Sherret):
//! a single [`Codebase`] value you can ask for files, symbols and relations,
//! with an in-memory [`SourceSet`] standing in for the file system so tests and
//! embedders never need to touch disk.
//!
//! ## Determinism contract
//!
//! Every type in this crate upholds the following, and downstream crates rely
//! on it to produce byte-identical output across runs, machines and operating
//! systems:
//!
//! 1. **Ordered collections only.** All maps and sets are `BTreeMap` /
//!    `BTreeSet`; iteration order is a function of the data, never of a hash
//!    seed or insertion order.
//! 2. **Normalised paths.** [`SourcePath`] is always relative, `/`-separated
//!    and free of `.`/`..` components, so `src\lib.rs` on Windows and
//!    `src/lib.rs` on Linux are the same key.
//! 3. **Stable hashing.** [`ContentHash`] is BLAKE3 over the source text with
//!    line endings normalised to `\n`; it never uses `std`'s randomly seeded
//!    hasher.
//! 4. **No ambient data.** Nothing in the model records timestamps, absolute
//!    paths, user names or environment variables.
//!
//! ## Built for sequence diagrams
//!
//! Callables carry a [`Flow`]: the ordered tree of calls, branches, loops and
//! early exits in their body. That is what makes dense, per-function sequence
//! diagrams possible without running the code, and because every call target
//! is a canonical [`SymbolId`], separately rendered sequences can be merged or
//! inlined into each other by a downstream orchestrator.
//!
//! ## Symbol ids
//!
//! Every symbol is named by a `sym:` id (see [`sym`]): kind-explicit,
//! SCIP-style, with no file path, so moving code between files never changes
//! an id.
//!
//! ## Example
//!
//! Build a tiny model by hand and query it:
//!
//! ```
//! use sealmap_model::*;
//!
//! let pkg = Package::current("cargo", "demo").unwrap();
//! let root = SymbolId::package_root(pkg.clone());
//! let id = |name: &str| root.child(Descriptor::r#type(name)).unwrap();
//!
//! let mut cb = Codebase::new("demo");
//! let file = SourcePath::new("src/lib.rs").unwrap();
//! cb.add_file(SourceFile::new(file.clone(), "rust", root.clone(), "pub struct A;"));
//!
//! cb.add_symbol(Symbol::new(id("A"), "A", SymbolKind::Struct, file.clone()));
//! cb.add_symbol(Symbol::new(id("B"), "B", SymbolKind::Struct, file.clone()));
//! cb.add_relation(Relation::new(id("A"), id("B"), RelationKind::FieldType, Confidence::Exact));
//!
//! assert_eq!(id("A").to_string(), "sym:cargo demo . A#");
//! assert_eq!(cb.symbols_in_file(&file).count(), 2);
//! assert_eq!(cb.relations_from(&id("A")).count(), 1);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod codebase;
pub mod flow;
mod hash;
mod path;
mod source;
pub mod sym;
mod symbol;

pub use codebase::{Codebase, CodebaseStats};
pub use flow::{Arm, Call, CallKind, Exit, Flow, Step};
pub use hash::ContentHash;
pub use path::{PathError, SourcePath};
pub use source::{LoadOptions, SourceFile, SourceSet};
pub use sym::{Descriptor, IdError, Package, Suffix, SymbolId, Version};
pub use symbol::{Confidence, Member, MemberKind, Relation, RelationKind, Span, Symbol, SymbolKind, Visibility};

/// Version of the serialised model schema (the JSON shape of [`Codebase`]).
///
/// Bumped whenever a field is added, removed or changes meaning, so agents and
/// tools reading a persisted model can refuse a schema they do not understand.
pub const MODEL_SCHEMA_VERSION: u32 = 1;

/// Compiles and runs the README examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
