//! Mapping files to crates and module paths using Cargo conventions.

use std::collections::{BTreeMap, BTreeSet};

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
    /// The library's crate name (`[lib] name`, else the package name), `-`
    /// replaced by `_`.
    name: String,
    /// `[package] name`, `-` replaced by `_`: what dependency tables name.
    package: String,
    has_lib: bool,
    lib_path: Option<String>,
    /// Packages this one depends on, `-` replaced by `_`, from every
    /// dependency table a target's code can use (`[dependencies]`,
    /// `[dev-dependencies]` and their `[target.*]` forms; a renamed entry
    /// counts under its `package`).
    deps: BTreeSet<String>,
}

/// Where files go and which crates each crate's code can name.
#[derive(Debug, Clone, Default)]
pub(crate) struct Plan {
    /// Each `.rs` file's crate and module.
    pub roles: BTreeMap<SourcePath, FileRole>,
    /// Workspace crate → the workspace crates its code can reach: itself,
    /// its package's library (for a binary, test, example or bench crate)
    /// and the workspace packages its manifest depends on. A crate missing
    /// here (the manifest-less fallback crate) reaches every crate.
    pub reach: BTreeMap<String, BTreeSet<String>>,
}

impl Plan {
    /// Can code in crate `from` name items of crate `to`?
    pub fn reaches(&self, from: &str, to: &str) -> bool {
        self.reach.get(from).is_none_or(|r| r.contains(to))
    }
}

/// The dependency names of one manifest, as packages (`-` → `_`).
fn manifest_deps(doc: &toml::Table) -> BTreeSet<String> {
    const TABLES: &[&str] = &["dependencies", "dev-dependencies", "dev_dependencies"];
    let mut out = BTreeSet::new();
    let mut add = |deps: Option<&toml::Value>| {
        for (key, spec) in deps.and_then(toml::Value::as_table).into_iter().flatten() {
            let package = spec.get("package").and_then(toml::Value::as_str).unwrap_or(key);
            out.insert(package.replace('-', "_"));
        }
    };
    for t in TABLES {
        add(doc.get(*t));
    }
    for cfg in doc.get("target").and_then(toml::Value::as_table).into_iter().flat_map(|t| t.values()) {
        for t in TABLES {
            add(cfg.get(*t));
        }
    }
    out
}

/// Discover packages from every `Cargo.toml` with a `[package]` table.
///
/// When no manifest exists, the whole set is treated as a single library
/// crate called `fallback`, with module paths derived from directories.
pub(crate) fn plan(sources: &SourceSet, fallback: &str) -> Plan {
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
        packages.push(Package {
            dir: dir.to_owned(),
            name: lib_name.replace('-', "_"),
            package: name.replace('-', "_"),
            has_lib,
            lib_path,
            deps: manifest_deps(&doc),
        });
    }
    // Longest directory first, so nested packages win over their parents.
    packages.sort_by(|a, b| b.dir.len().cmp(&a.dir.len()).then(a.dir.cmp(&b.dir)));

    // Package name → library crate name, for the workspace packages that
    // have a library: the only crates a dependency can name.
    let libs: BTreeMap<&str, &str> =
        packages.iter().filter(|p| p.has_lib).map(|p| (p.package.as_str(), p.name.as_str())).collect();
    let mut out = Plan::default();
    for path in sources.paths() {
        if path.extension() != Some("rs") {
            continue;
        }
        let p = path.as_str();
        let role = match packages.iter().find(|pkg| pkg.dir.is_empty() || p.starts_with(&format!("{}/", pkg.dir))) {
            Some(pkg) => {
                let rel = if pkg.dir.is_empty() { p } else { &p[pkg.dir.len() + 1..] };
                let role = role_in_package(pkg, rel, p);
                if let Some(role) = &role {
                    out.reach.entry(role.crate_name.clone()).or_insert_with(|| {
                        let own = pkg.has_lib.then(|| pkg.name.clone());
                        let deps = pkg.deps.iter().filter_map(|d| libs.get(d.as_str()).map(|l| (*l).to_owned()));
                        [role.crate_name.clone()].into_iter().chain(own).chain(deps).collect()
                    });
                }
                role
            }
            None => Some(FileRole {
                crate_name: sanitize(fallback),
                module: module_from_rel(&sanitize(fallback), p.strip_prefix("src/").unwrap_or(p)),
                target: TargetKind::Lib,
            }),
        };
        if let Some(role) = role {
            out.roles.insert(path.clone(), role);
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
        let plan = plan(&s, "x").roles;
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
    fn reach_follows_dependency_tables() {
        let s = set(&[
            ("Cargo.toml", "[workspace]\nmembers=['ledger', 'audit', 'fixtures', 'report']"),
            (
                "ledger/Cargo.toml",
                "[package]\nname='ledger-core'\n[lib]\nname='ledger'\n[dependencies]\ntrail={path='../audit',package='audit-trail'}\nserde='1'\n[dev-dependencies]\nfixtures={path='../fixtures'}",
            ),
            ("ledger/src/lib.rs", ""),
            ("ledger/src/main.rs", ""),
            ("audit/Cargo.toml", "[package]\nname='audit-trail'"),
            ("audit/src/lib.rs", ""),
            ("fixtures/Cargo.toml", "[package]\nname='fixtures'"),
            ("fixtures/src/lib.rs", ""),
            (
                "report/Cargo.toml",
                "[package]\nname='report'\n[target.'cfg(unix)'.dependencies]\nledger-core={workspace=true}",
            ),
            ("report/src/main.rs", ""),
        ]);
        let p = plan(&s, "x");
        let reach = |c: &str| p.reach[c].iter().map(String::as_str).collect::<Vec<_>>();
        // A renamed dependency counts under its package; a non-workspace
        // dependency (`serde`) is not a crate of the codebase.
        assert_eq!(reach("ledger"), ["audit_trail", "fixtures", "ledger"]);
        assert_eq!(reach("ledger_main"), ["audit_trail", "fixtures", "ledger", "ledger_main"]);
        // A binary-only package's crate carries the package name.
        assert_eq!(reach("report"), ["ledger", "report"]);
        assert!(p.reaches("audit_trail", "audit_trail") && !p.reaches("audit_trail", "ledger"));
        // The manifest-less fallback crate reaches everything.
        assert!(p.reaches("x", "ledger"));
    }

    #[test]
    fn no_manifest_fallback() {
        let s = set(&[("src/lib.rs", ""), ("src/a/b.rs", "")]);
        let plan = plan(&s, "demo").roles;
        let m = |p: &str| plan[&SourcePath::new(p).unwrap()].module.join("::");
        assert_eq!(m("src/lib.rs"), "demo");
        assert_eq!(m("src/a/b.rs"), "demo::a::b");
    }
}
