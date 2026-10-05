//! # sealmap-rust
//!
//! A pure-Rust language adapter that turns Rust source into a
//! [`sealmap_model::Codebase`]: modules, types, traits, functions and
//! methods, the relations between them, and an ordered call [`Flow`] for every
//! function body (the input for sequence diagrams).
//!
//! It parses with [`syn`] and never runs `rustc`, `cargo` or anything else, so
//! it works on any checkout, including code that does not compile, and needs
//! no toolchain at runtime.
//!
//! [`Flow`]: sealmap_model::Flow
//!
//! ## How it works
//!
//! 1. **Layout.** `Cargo.toml` files in the [`SourceSet`] define packages;
//!    each `.rs` file is mapped to a crate and module path by Cargo's
//!    conventions (`src/lib.rs`, `src/a/mod.rs`, `src/bin/x.rs`, ...). Without
//!    any manifest, the whole set is one crate named after
//!    [`RustOptions::name`].
//! 2. **Collect (per file, parallel).** Each file is parsed and reduced to
//!    plain data: items, `use` tables, impl blocks and unresolved call flows.
//! 3. **Resolve (whole workspace, sequential).** Paths are resolved through
//!    local items, `use` imports (including renames, globs and `pub use`
//!    re-exports), `crate`/`self`/`super`/`Self`, and sibling workspace crates.
//!    Method calls are resolved from the receiver's declared or constructed
//!    type (`self`, `self.field`, typed params, `let x = Foo::new()`, a
//!    struct literal), and `Self::f(..)` from the enclosing impl's self type.
//!    Calls in `match` guards are part of their arm.
//!
//! ## Ids
//!
//! Every definition gets a `sym:` id in the `cargo` package named after its
//! crate (see `sealmap_extract::ids` for the full table):
//! `sym:cargo shop . db/Db#insert().`. Binary, test, example and bench
//! targets are packages of their own (`shop_main`, `test_it`, ...), so a
//! binary and the library never share an id.
//!
//! ## Fingerprints
//!
//! Every symbol carries `sig_hash` (its contract) and `body_hash` (its
//! implementation), computed with `sealmap_extract::fingerprint` from syn's
//! token streams. Comments, doc comments, whitespace, lint attributes and the
//! rewrites rustfmt makes (trailing commas, braces around a closure or match
//! arm body, `use` order) never change them; neither does moving an item
//! within or between files. Reflowing tokio, VisionClaw and sealmap itself
//! with an aggressive rustfmt configuration changes no id and no
//! fingerprint.
//!
//! ## What "resolved" means here
//!
//! There is no type checker. Every call and relation carries a
//! [`Confidence`](sealmap_model::Confidence):
//!
//! * `exact`: resolved through explicit paths, imports or declared types to a
//!   symbol defined in the analysed code;
//! * `inferred`: matched by a weaker rule (a unique, distinctive method name
//!   on an unknown receiver, or a method reached through deref);
//! * `external`: outside the analysed code.
//!
//! Calls into `std`/`core`/`alloc`, prelude constructors (`Some`, `Ok`,
//! `Vec::new` via `std`) and method calls on receivers of unknown type are
//! dropped by default ([`ExternalCalls::NonStd`]), which keeps sequence
//! diagrams dense: they show *your* code talking to itself and to its
//! dependencies.
//!
//! ## Determinism
//!
//! Output depends only on the source set and options. Files are processed in
//! path order, items in source order, and parallel parsing (feature
//! `parallel`, on by default) collects results in input order, so the model
//! is identical with or without it.
//!
//! ## Example
//!
//! ```
//! use sealmap_model::{SourceSet, Step, SymbolId};
//! use sealmap_rust::{RustOptions, extract};
//!
//! let mut src = SourceSet::new();
//! src.insert("Cargo.toml", "[package]\nname = \"shop\"").unwrap();
//! src.insert("src/lib.rs", r#"
//!     pub mod db;
//!     use db::Db;
//!
//!     /// Order service.
//!     pub struct Orders { db: Db }
//!
//!     impl Orders {
//!         pub fn place(&self, id: u64) -> Result<(), String> {
//!             if self.db.exists(id) {
//!                 return Err("dup".into());
//!             }
//!             self.db.insert(id)?;
//!             Ok(())
//!         }
//!     }
//! "#).unwrap();
//! src.insert("src/db.rs", r#"
//!     pub struct Db;
//!     impl Db {
//!         pub fn exists(&self, id: u64) -> bool { false }
//!         pub fn insert(&self, id: u64) -> Result<(), String> { Ok(()) }
//!     }
//! "#).unwrap();
//!
//! let out = extract(&src, &RustOptions::default());
//! let place = out.codebase.symbol(&SymbolId::parse("sym:cargo shop . Orders#place().").unwrap()).unwrap();
//! let calls: Vec<_> = place.flow.as_ref().unwrap().calls().map(|c| c.target.to_string()).collect();
//! assert_eq!(calls, ["sym:cargo shop . db/Db#exists().", "sym:cargo shop . db/Db#insert()."]);
//! assert!(out.diagnostics.is_empty());
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod collect;
mod fingerprint;
mod layout;
mod raw;
mod resolve;
mod tidy;

use std::path::Path;

use sealmap_extract::isolate::{COLLECT_STACK_BYTES, map_isolated};
use sealmap_model::{LoadOptions, SourcePath, SourceSet};

pub use sealmap_extract::{Diagnostic, ExternalCalls, Extraction};

/// Language adapter options.
#[derive(Debug, Clone)]
pub struct RustOptions {
    /// Codebase name, also used as the crate name when no `Cargo.toml` is
    /// present.
    pub name: String,
    /// Include `#[cfg(test)]` modules, `#[test]` functions and files under
    /// `tests/`, `examples/` and `benches/`. Off by default.
    pub include_tests: bool,
    /// External call policy.
    pub external_calls: ExternalCalls,
}

impl Default for RustOptions {
    fn default() -> Self {
        Self { name: "codebase".into(), include_tests: false, external_calls: ExternalCalls::NonStd }
    }
}

/// Extract a [`Codebase`](sealmap_model::Codebase) from the `.rs` (and `Cargo.toml`) files in
/// `sources`. Never fails: unparsable files are reported in
/// [`Extraction::diagnostics`].
pub fn extract(sources: &SourceSet, options: &RustOptions) -> Extraction {
    let plan = layout::plan(sources, &options.name);
    let jobs: Vec<_> = plan
        .into_iter()
        .filter(|(_, role)| options.include_tests || role.target <= layout::TargetKind::Bin)
        .filter_map(|(path, role)| sources.get(&path).map(|text| (path.clone(), role, text)))
        .collect();

    let files = collect_all(&jobs, options);

    let (codebase, diagnostics) = resolve::build(&options.name, files, options);
    Extraction { codebase, diagnostics }
}

type Job<'a> = (SourcePath, layout::FileRole, &'a str);

/// Pass 1 over every file, isolated per file (big stacks, panic guard; see
/// [`sealmap_extract::isolate`]). A panic while collecting one file degrades
/// that file to a diagnostic instead of aborting the run.
fn collect_all(jobs: &[Job<'_>], options: &RustOptions) -> Vec<raw::RawFile> {
    map_isolated(
        jobs,
        COLLECT_STACK_BYTES,
        |(p, r, t)| collect::collect_file(p, r, t, options),
        |(p, r, t), why| collect::failed_file(p, r, t, format!("1:1: internal error while collecting: {why}")),
    )
}

/// Load the `.rs` and `Cargo.toml` files under `root` (honouring
/// `.gitignore`, skipping `target/`, hidden directories and the like) without
/// extracting them.
pub fn load_dir(root: &Path) -> std::io::Result<SourceSet> {
    let load = LoadOptions { extensions: vec!["rs".into(), "toml".into()], ..LoadOptions::default() };
    let mut sources = SourceSet::load_dir(root, &load)?;
    sources.retain(|p| p.extension() == Some("rs") || p.file_name() == "Cargo.toml");
    Ok(sources)
}

/// Load `root` from disk (`.rs` and `Cargo.toml` files, skipping `target/`,
/// hidden directories and the like) and [`extract`] it.
///
/// [`RustOptions::name`] defaults to the directory name when left as
/// `"codebase"`.
pub fn extract_dir(root: &Path, options: &RustOptions) -> std::io::Result<(SourceSet, Extraction)> {
    let sources = load_dir(root)?;
    let mut options = options.clone();
    if options.name == "codebase" {
        if let Some(n) = root.canonicalize().ok().and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
        {
            options.name = n;
        }
    }
    let extraction = extract(&sources, &options);
    Ok((sources, extraction))
}

/// Compiles and runs the README examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
