//! # codemaid
//!
//! Deterministic, dense **codebase → Mermaid** generation for LLM agents.
//!
//! codemaid reads source code (pure Rust, no toolchain or Node at runtime),
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
//! This crate is a facade over the workspace:
//!
//! | Crate | Role |
//! |---|---|
//! | [`model`] (`codemaid-model`) | language-neutral code model and in-memory sources |
//! | [`mermaid`] (`codemaid-mermaid`) | zero-dependency typed Mermaid writers |
//! | [`rust`] (`codemaid-rust`) | Rust frontend (syn) |
//! | [`corpus`] (`codemaid-corpus`) | projections, index, 1:1 contract |
//!
//! Depend on the facade for the common path, or on the individual crates to
//! keep dependencies minimal (for example, `codemaid-mermaid` alone has none).
//!
//! ## Quick start
//!
//! ```no_run
//! use std::path::Path;
//!
//! // Generate the corpus for a checkout and write it into `docs/codemaid`.
//! let corpus = codemaid::generate_dir(Path::new("."), &codemaid::Options::default())?;
//! let changed = codemaid::corpus::write(Path::new("docs/codemaid"), &corpus)?;
//! println!("{} files updated", changed.entries.len());
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Several repositories are analysed together (so cross-repository calls
//! resolve) with [`generate_repos`]:
//!
//! ```no_run
//! # use std::path::Path;
//! let corpus = codemaid::generate_repos(
//!     &[("api", Path::new("../api")), ("core", Path::new("../core"))],
//!     &codemaid::Options::default(),
//! )?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! ## Determinism
//!
//! The same sources and options always produce byte-identical output on any
//! machine: ordered collections everywhere, normalised paths and newlines,
//! BLAKE3 hashes, no timestamps, and parallel work collected in input order.
//! That is what makes the corpus diffable, cacheable and enforceable in CI
//! (`codemaid verify`).

#![forbid(unsafe_code)]

use std::io;
use std::path::Path;

pub use codemaid_corpus as corpus;
pub use codemaid_mermaid as mermaid;
pub use codemaid_model as model;
pub use codemaid_rust as rust;

pub use codemaid_corpus::{Corpus, CorpusOptions, generate};
pub use codemaid_model::{Codebase, SourceSet};

/// Options for the end-to-end helpers.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Frontend options.
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
pub fn load_repos(repos: &[(&str, &Path)]) -> io::Result<SourceSet> {
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
