//! Mapping files to crates and module paths using Cargo conventions.

use std::collections::BTreeMap;

use sealmap_model::{SourcePath, SourceSet};

/// Which Cargo target a file belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TargetKind {
    Lib,
    Bin,
    Test,
    Example,
    Bench,
}

/// A file's place in the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileRole {
    /// Crate name as used in paths (`-` replaced by `_`).
    pub crate_name: String,
    /// Module path including the crate name, e.g. `["mycrate", "net", "tcp"]`.
    pub module: Vec<String>,
    pub target: TargetKind,
}

/// A Cargo package found in the source set.
#[derive(Debug, Clone)]
struct Package {
    dir: String,
    name: String,
    has_lib: bool,
    lib_path: Option<String>,
}

/// Discover packages from every `Cargo.toml` with a `[package]` table.
///
/// When no manifest exists, the whole set is treated as a single library
/// crate called `fallback`, with module paths derived from directories.
pub(crate) fn plan(sources: &SourceSet, fallback: &str) -> BTreeMap<SourcePath, FileRole> {
    let mut packages: Vec<Package> = Vec::new();
    for (path, text) in sources.iter() {
        if path.file_name() != "Cargo.toml" {
            continue;
        }
        let Ok(doc) = text.parse::<toml::Table>() else { continue };
        let Some(pkg) = doc.get("package").and_then(|p| p.as_table()) else { continue };
        let Some(name) = pkg.get("name").and_then(|n| n.as_str()) else { continue };
        let dir = path.as_str().strip_suffix("Cargo.toml").unwrap_or("").trim_end_matches('/');
        let lib_path = doc
            .get("lib")
            .and_then(|l| l.get("path"))
            .and_then(|p| p.as_str())
            .and_then(|p| SourcePath::new(join(dir, p)).ok())
            .map(|p| p.as_str().to_owned());
        let lib_name = doc.get("lib").and_then(|l| l.get("name")).and_then(|n| n.as_str()).unwrap_or(name);
        let default_lib = join(dir, "src/lib.rs");
        let has_lib = lib_path.is_some() || sources.get_str(&default_lib).is_some();
        packages.push(Package { dir: dir.to_owned(), name: lib_name.replace('-', "_"), has_lib, lib_path });
    }
    // Longest directory first, so nested packages win over their parents.
    packages.sort_by(|a, b| b.dir.len().cmp(&a.dir.len()).then(a.dir.cmp(&b.dir)));

    let mut out = BTreeMap::new();
    for path in sources.paths() {
        if path.extension() != Some("rs") {
            continue;
        }
        let p = path.as_str();
        let role = match packages.iter().find(|pkg| pkg.dir.is_empty() || p.starts_with(&format!("{}/", pkg.dir))) {
            Some(pkg) => {
                let rel = if pkg.dir.is_empty() { p } else { &p[pkg.dir.len() + 1..] };
                role_in_package(pkg, rel, p)
            }
            None => Some(FileRole {
                crate_name: sanitize(fallback),
                module: module_from_rel(&sanitize(fallback), p.strip_prefix("src/").unwrap_or(p)),
                target: TargetKind::Lib,
            }),
        };
        if let Some(role) = role {
            out.insert(path.clone(), role);
        }
    }
    out
}

fn role_in_package(pkg: &Package, rel: &str, full: &str) -> Option<FileRole> {
    if let Some(lib) = &pkg.lib_path {
        if lib == full {
            return Some(FileRole {
                crate_name: pkg.name.clone(),
                module: vec![pkg.name.clone()],
                target: TargetKind::Lib,
            });
        }
    }
    let (target, sub) = if let Some(rest) = rel.strip_prefix("src/bin/") {
        (TargetKind::Bin, rest)
    } else if let Some(rest) = rel.strip_prefix("src/") {
        if rest == "main.rs" {
            let name = if pkg.has_lib { format!("{}_main", pkg.name) } else { pkg.name.clone() };
            return Some(FileRole { crate_name: name.clone(), module: vec![name], target: TargetKind::Bin });
        }
        // Other files under src/ belong to the library when there is one,
        // otherwise to the main binary, which then carries the package name.
        let name = pkg.name.clone();
        return Some(FileRole {
            crate_name: name.clone(),
            module: module_from_rel(&name, rest),
            target: TargetKind::Lib,
        });
    } else if let Some(rest) = rel.strip_prefix("tests/") {
        (TargetKind::Test, rest)
    } else if let Some(rest) = rel.strip_prefix("examples/") {
        (TargetKind::Example, rest)
    } else {
        // build.rs and stray files are not part of a target.
        (TargetKind::Bench, rel.strip_prefix("benches/")?)
    };
    // Each top-level file (or directory with main.rs) is its own crate.
    let (first, tail) = match sub.split_once('/') {
        Some((dir, tail)) => (dir.to_owned(), Some(tail)),
        None => (sub.trim_end_matches(".rs").to_owned(), None),
    };
    let prefix = match target {
        TargetKind::Test => "test_",
        TargetKind::Example => "example_",
        TargetKind::Bench => "bench_",
        _ => "",
    };
    let crate_name = sanitize(&format!("{prefix}{first}"));
    let module = match tail {
        None | Some("main.rs") => vec![crate_name.clone()],
        Some(t) => module_from_rel(&crate_name, t),
    };
    Some(FileRole { crate_name, module, target })
}

/// `a/b.rs` → `[krate, a, b]`; `a/mod.rs` → `[krate, a]`; `lib.rs` → `[krate]`.
fn module_from_rel(krate: &str, rel: &str) -> Vec<String> {
    let mut module = vec![krate.to_owned()];
    let parts: Vec<&str> = rel.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        let last = i + 1 == parts.len();
        if last {
            let stem = part.trim_end_matches(".rs");
            if !(stem == "mod" || (i == 0 && (stem == "lib" || stem == "main"))) {
                module.push(sanitize(stem));
            }
        } else {
            module.push(sanitize(part));
        }
    }
    module
}

fn join(dir: &str, p: &str) -> String {
    if dir.is_empty() { p.to_owned() } else { format!("{dir}/{p}") }
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(files: &[(&str, &str)]) -> SourceSet {
        let mut s = SourceSet::new();
        for (p, t) in files {
            s.insert(p, t).unwrap();
        }
        s
    }

    #[test]
    fn workspace_layout() {
        let s = set(&[
            ("Cargo.toml", "[workspace]\nmembers=['crates/*']"),
            ("crates/my-core/Cargo.toml", "[package]\nname='my-core'"),
            ("crates/my-core/src/lib.rs", ""),
            ("crates/my-core/src/net/mod.rs", ""),
            ("crates/my-core/src/net/tcp.rs", ""),
            ("crates/my-core/src/main.rs", ""),
            ("crates/my-core/src/bin/tool.rs", ""),
            ("crates/my-core/tests/it.rs", ""),
            ("crates/my-core/build.rs", ""),
        ]);
        let plan = plan(&s, "x");
        let m = |p: &str| plan.get(&SourcePath::new(p).unwrap()).map(|r| r.module.join("::"));
        assert_eq!(m("crates/my-core/src/lib.rs").as_deref(), Some("my_core"));
        assert_eq!(m("crates/my-core/src/net/mod.rs").as_deref(), Some("my_core::net"));
        assert_eq!(m("crates/my-core/src/net/tcp.rs").as_deref(), Some("my_core::net::tcp"));
        assert_eq!(m("crates/my-core/src/main.rs").as_deref(), Some("my_core_main"));
        assert_eq!(m("crates/my-core/src/bin/tool.rs").as_deref(), Some("tool"));
        assert_eq!(m("crates/my-core/tests/it.rs").as_deref(), Some("test_it"));
        assert_eq!(m("crates/my-core/build.rs"), None);
    }

    #[test]
    fn no_manifest_fallback() {
        let s = set(&[("src/lib.rs", ""), ("src/a/b.rs", "")]);
        let plan = plan(&s, "demo");
        let m = |p: &str| plan[&SourcePath::new(p).unwrap()].module.join("::");
        assert_eq!(m("src/lib.rs"), "demo");
        assert_eq!(m("src/a/b.rs"), "demo::a::b");
    }
}
