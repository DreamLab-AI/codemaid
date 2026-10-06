//! # sealmap
//!
//! Deterministic, dense **codebase → Mermaid** generation for LLM agents.
//!
//! sealmap reads source code (pure Rust, no toolchain or Node at runtime),
//! builds a language-neutral model of it, and projects that model into a
//! corpus of Mermaid diagrams with exactly one document per source file:
//!
//! * **sequence diagrams** for every function's ordered calls, with
//!   `alt`/`opt`/`loop`/`par` fragments for control flow;
//! * **class diagrams** of what each file defines and how it relates to the
//!   rest of the code;
//! * **ER diagrams** of each crate's data model, and **flowcharts** of crate,
//!   repository and module dependencies;
//! * a JSON **index** linking every call to the fragment that expands it,
//!   so an orchestrating agent can merge and inline diagrams by stable id.
//!
//! Every symbol is named by a `sym:` id (`sym:cargo shop . db/Db#insert().`,
//! no file path, see [`model::sym`]) and carries two fingerprints:
//! `sig_hash` for its contract and `body_hash` for its implementation, both
//! blind to formatting, comments and moves. Diagram ids are derived from the
//! `sym:` ids injectively. The model and index JSON are schema v2.
//!
//! This crate is a facade over the workspace:
//!
//! | Crate | Role |
//! |---|---|
//! | [`model`] (`sealmap-model`) | language-neutral code model and in-memory sources |
//! | [`mermaid`] (`sealmap-mermaid`) | typed Mermaid writers and injective diagram ids |
//! | [`extract`] (`sealmap-extract`) | language-neutral extraction core shared by the language adapters: flow IR and lowering, confidence, labels, `sym:` ids, fingerprints |
//! | [`rust`] (`sealmap-rust`) | Rust language adapter (syn) |
//! | [`corpus`] (`sealmap-corpus`) | projections, index, 1:1 contract; the seal lock and its checks; review packs |
//! | [`dense`] (`sealmap-dense`) | compact agent projection: skeletons, call trees, short-name index, budgeted slices |
//!
//! Depend on the facade for the common path, or on the individual crates to
//! keep dependencies minimal (for example, `sealmap-mermaid` with
//! `default-features = false` has none).
//!
//! ## Quick start
//!
//! ```no_run
//! use std::path::Path;
//!
//! // Generate the corpus for a checkout and write it into `.sealmap/`.
//! let corpus = sealmap::generate_dir(Path::new("."), &sealmap::Options::default())?;
//! let changed = sealmap::corpus::write(Path::new(".sealmap"), &corpus)?;
//! println!("{} files updated", changed.entries.len());
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Several repositories are analysed together (so cross-repository calls
//! resolve) with [`generate_repos`]:
//!
//! ```no_run
//! # use std::path::Path;
//! let corpus = sealmap::generate_repos(
//!     &[("api", Path::new("../api")), ("core", Path::new("../core"))],
//!     &sealmap::Options::default(),
//! )?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! ## Determinism
//!
//! The same sources and options always produce byte-identical output on any
//! machine: ordered collections everywhere, normalised paths and newlines,
//! BLAKE3 hashes, no timestamps, and parallel work collected in input order.
//! That is what makes generated output safe to rebuild on demand instead of
//! committing it, and what lets a seal pin exact symbol versions.
//!
//! ## Seals
//!
//! Hand-written diagram topics cite symbols by `sym:` id, and a lockfile
//! (`seals.lock`) records the `sig_hash` and `body_hash` each cited symbol
//! had when the topic was reviewed. [`corpus::seal`] holds the lock format
//! and the checks: `verify` (the CI gate), `seal_check`, `stale`, `resolve`
//! and `sign`. The `sealmap` binary wraps them, reading topics from
//! `docs/diagrams/` by default.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::io;
use std::path::Path;

pub use sealmap_corpus as corpus;
pub use sealmap_dense as dense;
pub use sealmap_extract as extract;
pub use sealmap_mermaid as mermaid;
pub use sealmap_model as model;
pub use sealmap_rust as rust;

pub use sealmap_corpus::{Corpus, CorpusOptions, generate};
pub use sealmap_model::{Codebase, SourceSet};

/// Options for the end-to-end helpers.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Language adapter options.
    pub rust: rust::RustOptions,
    /// Projection options.
    pub corpus: CorpusOptions,
}

/// Load, extract and project a single directory.
pub fn generate_dir(root: &Path, options: &Options) -> io::Result<Corpus> {
    let (_, extraction) = rust::extract_dir(root, &options.rust)?;
    Ok(generate(&extraction.codebase, &options.corpus))
}

/// Load several repositories as one codebase. Each repository's files are
/// prefixed with its name (`api/src/lib.rs`), crates resolve across
/// repositories, and the overview groups crates by repository.
pub fn generate_repos(repos: &[(&str, &Path)], options: &Options) -> io::Result<Corpus> {
    let sources = load_repos(repos)?;
    let mut ro = options.rust.clone();
    if ro.name == "codebase" {
        ro.name = repos.iter().map(|(n, _)| *n).collect::<Vec<_>>().join("+");
    }
    let extraction = rust::extract(&sources, &ro);
    Ok(generate(&extraction.codebase, &options.corpus))
}

/// Load several repositories into one [`SourceSet`] under `<name>/` prefixes.
///
/// Fails with [`io::ErrorKind::InvalidInput`] when one name is used twice or
/// is a prefix directory of another (`api` and `api/v2`), since one
/// repository's files would then replace the other's.
pub fn load_repos(repos: &[(&str, &Path)]) -> io::Result<SourceSet> {
    for (i, (a, _)) in repos.iter().enumerate() {
        for (b, _) in &repos[i + 1..] {
            let nested = |x: &str, y: &str| y.strip_prefix(x).is_some_and(|rest| rest.starts_with('/'));
            if a == b || nested(a, b) || nested(b, a) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("repository names `{a}` and `{b}` clash: each repository needs its own prefix"),
                ));
            }
        }
    }
    let load = model::LoadOptions { extensions: vec!["rs".into(), "toml".into()], ..Default::default() };
    let mut all = SourceSet::new();
    for (name, path) in repos {
        let set = SourceSet::load_dir(path, &load)?;
        for (p, text) in set.iter() {
            if p.extension() == Some("rs") || p.file_name() == "Cargo.toml" {
                all.insert(format!("{name}/{p}"), text).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            }
        }
    }
    Ok(all)
}

/// Compiles and runs the README examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
