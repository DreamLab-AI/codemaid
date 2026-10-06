//! Citation-to-symbol mapping at a topic's stamp (PREREG "Mapping citations to
//! symbols"), producing what each topic tracks in one repository.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::model::Model;

/// Why a citation tracks its whole file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Fallback {
    /// The cited file is not Rust.
    NotRust,
    /// The topic declares no `verified_commit` for this repository, so there is no stamp to resolve at.
    NoStamp,
    /// The stamp names no commit in the repository.
    StampUnknown,
    /// The file does not exist at the stamp.
    AbsentAtStamp,
    /// The file is in the model at the stamp but failed to parse.
    ParseError,
    /// The line is inside no symbol (this includes files the default
    /// extraction leaves out, such as `tests/`, and lines past the end).
    NoSymbol,
}

/// The outcome of mapping one citation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mapped {
    /// The innermost symbol's id; `module` is `true` when that symbol is a module.
    Symbol {
        id: String,
        module: bool,
    },
    Fallback(Fallback),
}

/// What resolving one citation needs to know about its stamp.
pub enum Stamp<'a> {
    /// No sha for this repository in `verified_commit`.
    Missing,
    /// A sha that names no commit.
    Unknown,
    /// The model at the stamp, and whether the cited file exists there.
    Model { model: &'a Model, file_exists: bool },
}

/// Map one citation of `file` at `line`.
pub fn map_citation(file: &str, line: u32, stamp: &Stamp<'_>) -> Mapped {
    if !file.ends_with(".rs") {
        return Mapped::Fallback(Fallback::NotRust);
    }
    match stamp {
        Stamp::Missing => Mapped::Fallback(Fallback::NoStamp),
        Stamp::Unknown => Mapped::Fallback(Fallback::StampUnknown),
        Stamp::Model { file_exists: false, .. } => Mapped::Fallback(Fallback::AbsentAtStamp),
        Stamp::Model { model, .. } if model.parse_errors.contains(file) => Mapped::Fallback(Fallback::ParseError),
        Stamp::Model { model, .. } => match model.innermost(file, line) {
            Some((id, s)) => Mapped::Symbol { id: id.to_string(), module: s.module },
            None => Mapped::Fallback(Fallback::NoSymbol),
        },
    }
}

/// What one topic tracks in one repository.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tracking {
    /// Every `sources:` file in this repository (repository-relative). T_file reads only this.
    pub sources: BTreeSet<String>,
    /// Cited symbols, each with the file it was cited in.
    pub symbols: BTreeSet<(String, String)>,
    /// Files tracked whole because a citation of theirs fell back.
    pub fallback_files: BTreeSet<String>,
    /// Sources tracked whole because no citation names them.
    pub uncited_files: BTreeSet<String>,
}

impl Tracking {
    /// Every file tracked whole.
    pub fn files(&self) -> BTreeSet<&String> {
        self.fallback_files.iter().chain(&self.uncited_files).collect()
    }
}

/// One citation already resolved to a `sources:` file in this repository.
#[derive(Debug, Clone)]
pub struct RepoCitation {
    pub file: String,
    pub mapped: Mapped,
}

/// Build a topic's tracking set from its sources and its mapped citations.
pub fn tracking(sources: &BTreeSet<String>, cites: &[RepoCitation]) -> Tracking {
    let mut t = Tracking { sources: sources.clone(), ..Tracking::default() };
    let mut cited: BTreeMap<&str, usize> = BTreeMap::new();
    for c in cites {
        *cited.entry(c.file.as_str()).or_default() += 1;
        match &c.mapped {
            Mapped::Symbol { id, .. } => {
                t.symbols.insert((id.clone(), c.file.clone()));
            }
            Mapped::Fallback(_) => {
                t.fallback_files.insert(c.file.clone());
            }
        }
    }
    for s in sources {
        if !cited.contains_key(s.as_str()) {
            t.uncited_files.insert(s.clone());
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Model;
    use sealmap_model::SourceSet;
    use sealmap_rust::{RustOptions, extract};

    fn model(files: &[(&str, &str)]) -> Model {
        let mut s = SourceSet::new();
        for (p, t) in files {
            s.insert(p, t).unwrap();
        }
        Model::from_extraction(&extract(&s, &RustOptions::default()))
    }

    const LIB: &str = "pub struct Db;\n\nimpl Db {\n    pub fn get(&self) -> u8 {\n        let x = 1;\n        x\n    }\n}\n\nuse std::fmt;\n";

    #[test]
    fn a_citation_inside_a_fn_maps_to_the_fn() {
        let m = model(&[("Cargo.toml", "[package]\nname = \"shop\""), ("src/lib.rs", LIB)]);
        let stamp = Stamp::Model { model: &m, file_exists: true };
        assert_eq!(
            map_citation("src/lib.rs", 5, &stamp),
            Mapped::Symbol { id: "sym:cargo shop . Db#get().".into(), module: false }
        );
        // `use` at the top level is inside only the file's module.
        assert!(matches!(map_citation("src/lib.rs", 10, &stamp), Mapped::Symbol { module: true, .. }));
        // Past the end of the file: inside no symbol.
        assert_eq!(map_citation("src/lib.rs", 99, &stamp), Mapped::Fallback(Fallback::NoSymbol));
    }

    #[test]
    fn non_rust_unparsable_and_unstamped_files_fall_back() {
        let m = model(&[("Cargo.toml", "[package]\nname = \"shop\""), ("src/lib.rs", "pub fn f( {")]);
        let ok = Stamp::Model { model: &m, file_exists: true };
        assert_eq!(map_citation("flake.nix", 3, &ok), Mapped::Fallback(Fallback::NotRust));
        assert_eq!(map_citation("src/lib.rs", 1, &ok), Mapped::Fallback(Fallback::ParseError));
        assert_eq!(map_citation("src/lib.rs", 1, &Stamp::Missing), Mapped::Fallback(Fallback::NoStamp));
        let gone = Stamp::Model { model: &m, file_exists: false };
        assert_eq!(map_citation("src/gone.rs", 1, &gone), Mapped::Fallback(Fallback::AbsentAtStamp));
        // A test file is not extracted by default: inside no symbol.
        let t =
            model(&[("Cargo.toml", "[package]\nname = \"shop\""), ("src/lib.rs", ""), ("tests/it.rs", "fn t() {}")]);
        let ts = Stamp::Model { model: &t, file_exists: true };
        assert_eq!(map_citation("tests/it.rs", 1, &ts), Mapped::Fallback(Fallback::NoSymbol));
    }

    #[test]
    fn uncited_sources_and_fallbacks_are_file_level() {
        let sources: BTreeSet<String> = ["a.rs", "b.rs", "c.nix"].map(String::from).into();
        let cites = vec![
            RepoCitation { file: "a.rs".into(), mapped: Mapped::Symbol { id: "sym:x".into(), module: false } },
            RepoCitation { file: "c.nix".into(), mapped: Mapped::Fallback(Fallback::NotRust) },
        ];
        let t = tracking(&sources, &cites);
        assert_eq!(t.symbols, [("sym:x".to_string(), "a.rs".to_string())].into());
        assert_eq!(t.fallback_files, ["c.nix"].map(String::from).into());
        assert_eq!(t.uncited_files, ["b.rs"].map(String::from).into());
    }
}
