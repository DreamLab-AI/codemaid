//! # sealmap-rust
//!
//! A pure-Rust frontend that turns Rust source into a
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
//!    type (`self`, `self.field`, typed params, `let x = Foo::new()`).
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
//! let place = out.codebase.symbol(&SymbolId::new("shop::Orders::place")).unwrap();
//! let calls: Vec<_> = place.flow.as_ref().unwrap().calls().map(|c| c.target.as_str()).collect();
//! assert_eq!(calls, ["shop::db::Db::exists", "shop::db::Db::insert"]);
//! assert!(out.diagnostics.is_empty());
//! ```

#![forbid(unsafe_code)]

mod collect;
mod layout;
mod raw;
mod resolve;
mod tidy;

use std::path::Path;

use sealmap_model::{Codebase, LoadOptions, SourcePath, SourceSet};

/// Which calls to external code are kept in flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExternalCalls {
    /// Keep every external call, including `std` and unresolved method calls
    /// (`?::name`). Verbose; useful for debugging the frontend.
    All,
    /// Keep calls into non-std dependencies (`tokio::spawn`,
    /// `serde_json::to_string`); drop `std`/`core`/`alloc` and unresolved
    /// method calls. The default.
    #[default]
    NonStd,
    /// Keep only calls into the analysed code.
    None,
}

/// Frontend options.
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

/// A non-fatal problem found while extracting (currently: files that failed
/// to parse; they still get a module symbol so every file is represented).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// File the problem is in.
    pub file: SourcePath,
    /// Human-readable message with `line:col` prefix.
    pub message: String,
}

/// Result of [`extract`].
#[derive(Debug, Clone)]
pub struct Extraction {
    /// The model.
    pub codebase: Codebase,
    /// Non-fatal problems, in file order.
    pub diagnostics: Vec<Diagnostic>,
}

/// Extract a [`Codebase`] from the `.rs` (and `Cargo.toml`) files in
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

/// Stack for collector threads. syn's recursive-descent parser and the flow
/// walker use stack in proportion to syntactic nesting (measured: one level
/// of `UInt<UInt<..>>` costs ~40 KiB in a debug build), and generated files
/// such as typenum's `gen/consts.rs` nest 64 deep, which overflows the 2 MiB
/// default of rayon workers. Pages are only committed when touched, so the
/// cost is address space (64 MiB × threads), not memory.
const COLLECT_STACK_BYTES: usize = 64 * 1024 * 1024;

type Job<'a> = (SourcePath, layout::FileRole, &'a str);

/// Pass 1 over every file, on threads with [`COLLECT_STACK_BYTES`] of stack.
/// A panic while collecting one file degrades that file to a diagnostic
/// instead of aborting the run.
fn collect_all(jobs: &[Job<'_>], options: &RustOptions) -> Vec<raw::RawFile> {
    let one = |(p, r, t): &Job<'_>| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| collect::collect_file(p, r, t, options)))
            .unwrap_or_else(|e| {
                let why = e
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_owned())
                    .or_else(|| e.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".into());
                collect::failed_file(p, r, t, format!("1:1: internal error while collecting: {why}"))
            })
    };
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        if let Ok(pool) = rayon::ThreadPoolBuilder::new().stack_size(COLLECT_STACK_BYTES).build() {
            return pool.install(|| jobs.par_iter().map(one).collect());
        }
    }
    // Sequential (feature off, or the pool could not be built): one thread
    // with the same stack, so behaviour does not depend on the feature.
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("sealmap-collect".into())
            .stack_size(COLLECT_STACK_BYTES)
            .spawn_scoped(scope, || jobs.iter().map(one).collect())
            .ok()
            .and_then(|h| h.join().ok())
    })
    .unwrap_or_else(|| jobs.iter().map(one).collect())
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
