//! The symbol-id builder.
//!
//! Every id a frontend mints goes through here, so all frontends share one
//! `sym:` grammar (see [`sealmap_model::sym`]):
//!
//! | Definition | Id |
//! |---|---|
//! | package root | `sym:cargo app .` |
//! | module | `sym:cargo app . store/` |
//! | type, trait, alias | `sym:cargo app . store/Db#` |
//! | function | `sym:cargo app . store/open().` |
//! | const, static | `sym:cargo app . store/LIMIT.` |
//! | macro | `sym:cargo app . store/table!` |
//! | inherent method | `sym:cargo app . store/Db#get().` |
//! | trait-impl method | ``sym:cargo app . store/Db#[`From<String>`]from().`` |
//! | trait-impl method on a type outside the code | ``sym:cargo app . store/impl#[Vec][Codec]encode().`` |
//! | reference with unknown kinds | `sym:extern serde_json::to_string` |
//! | method on a receiver of unknown type | `sym:? insert` |
//!
//! Methods sit under the type that owns them, never under the impl block,
//! so splitting an `impl` in two or moving it to another file keeps every
//! method id. Several inherent `impl` blocks of one type share the type as
//! owner. A trait implementation adds a `[Trait]` descriptor, spelled as the
//! impl writes the trait (generic arguments kept, so `impl From<A>` and
//! `impl From<B>` stay apart). When the implementing type is not defined in
//! the analysed code (`impl Codec for Vec<u8>`), there is no type to sit
//! under, so the method is anchored in the module holding the impl, after
//! rust-analyzer's `impl#[SelfType][Trait]` form.
//!
//! ```
//! use sealmap_frontend::ids;
//! use sealmap_model::SymbolKind;
//!
//! let pkg = ids::package("cargo", "app");
//! let store = ids::module_id(&pkg, &["store".into()]);
//! let db = ids::item_id(&store, SymbolKind::Struct, "Db");
//! assert_eq!(db.to_string(), "sym:cargo app . store/Db#");
//! assert_eq!(ids::method_id(&db, None, "get").to_string(), "sym:cargo app . store/Db#get().");
//! assert_eq!(
//!     ids::method_id(&db, Some("From<String>"), "from").to_string(),
//!     "sym:cargo app . store/Db#[`From<String>`]from()."
//! );
//! let vec = ids::path_id(&["Vec".into()]).unwrap();
//! assert_eq!(
//!     ids::impl_method_id(&store, &vec, "Vec", Some("Codec"), "encode").to_string(),
//!     "sym:cargo app . store/impl#[Vec][Codec]encode()."
//! );
//! ```

use sealmap_model::{Descriptor, Package, SymbolId, SymbolKind};

/// The package for `name` under `manager` (`cargo`, `npm`) at the current
/// tree. A name the grammar cannot hold (empty, or with edge spaces or
/// control characters, none of which a manifest allows) is replaced by `_`
/// with those characters removed, so building an id never fails.
pub fn package(manager: &str, name: &str) -> Package {
    Package::current(manager, name).unwrap_or_else(|_| {
        let cleaned: String = name.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_owned();
        let cleaned = if cleaned.is_empty() || cleaned == "." { "_".to_owned() } else { cleaned };
        Package::current(manager, cleaned)
            .or_else(|_| Package::current("unknown", "_"))
            .expect("`unknown _ .` is a valid package")
    })
}

/// The id of the module at `modules` (the path below the package root;
/// empty for the root itself).
pub fn module_id(package: &Package, modules: &[String]) -> SymbolId {
    SymbolId::global(package.clone(), modules.iter().map(|m| SymbolKind::Module.descriptor(m.as_str())).collect())
}

/// The id of item `name` of `kind` declared in `scope` (a module, or a
/// type for associated items). A `scope` that is not a global id yields a
/// path id.
pub fn item_id(scope: &SymbolId, kind: SymbolKind, name: &str) -> SymbolId {
    scope.child(kind.descriptor(name)).unwrap_or_else(|| scope.extend_path(name))
}

/// The id of method `name` owned by `owner` (a type or trait), under a
/// `[Trait]` descriptor when it implements `trait_` (spelled as written).
/// An `owner` that is not a global id yields a path id.
pub fn method_id(owner: &SymbolId, trait_: Option<&str>, name: &str) -> SymbolId {
    let scoped = match trait_ {
        Some(t) => owner.child(Descriptor::type_parameter(t)),
        None => Some(owner.clone()),
    };
    scoped.and_then(|s| s.child(Descriptor::method(name))).unwrap_or_else(|| owner.extend_path(name))
}

/// The id of method `name` in an impl block found in `module`. When `owner`
/// (the resolved implementing type) is a global id this is
/// [`method_id`]; otherwise the type is not part of the analysed code and
/// the method is anchored at `module/impl#[self_ty][trait_]name().`, with
/// `self_ty` the implementing type as written.
pub fn impl_method_id(
    module: &SymbolId,
    owner: &SymbolId,
    self_ty: &str,
    trait_: Option<&str>,
    name: &str,
) -> SymbolId {
    if owner.is_global() {
        return method_id(owner, trait_, name);
    }
    let mut d = vec![Descriptor::r#type("impl"), Descriptor::type_parameter(self_ty)];
    if let Some(t) = trait_ {
        d.push(Descriptor::type_parameter(t));
    }
    d.push(Descriptor::method(name));
    d.into_iter().try_fold(module.clone(), |id, d| id.child(d)).unwrap_or_else(|| module.extend_path(name))
}

/// The id of a path as written whose kinds are unknown
/// (`["serde_json", "to_string"]` → `sym:extern serde_json::to_string`).
/// `None` for an empty path.
pub fn path_id(segments: &[String]) -> Option<SymbolId> {
    SymbolId::path(segments.iter().cloned()).ok()
}

/// The placeholder target of a method call whose receiver could not be
/// resolved (`sym:? name`).
pub fn unresolved_method_id(name: &str) -> SymbolId {
    SymbolId::unresolved(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids() {
        let pkg = package("cargo", "app");
        let root = module_id(&pkg, &[]);
        assert_eq!(root.to_string(), "sym:cargo app .");
        let a = module_id(&pkg, &["a".into(), "b".into()]);
        assert_eq!(a.parent().unwrap().to_string(), "sym:cargo app . a/");
        let f = item_id(&a, SymbolKind::Function, "go");
        assert_eq!(f.to_string(), "sym:cargo app . a/b/go().");
        assert_eq!(item_id(&a, SymbolKind::Module, "go").to_string(), "sym:cargo app . a/b/go/");
        assert_eq!(unresolved_method_id("go").to_string(), "sym:? go");
        assert!(path_id(&[]).is_none());
        let ext = path_id(&["dep".into(), "T".into()]).unwrap();
        assert_eq!(method_id(&ext, Some("X"), "m").to_string(), "sym:extern dep::T::m");
        assert_eq!(item_id(&ext, SymbolKind::Function, "f").to_string(), "sym:extern dep::T::f");
    }

    #[test]
    fn package_names_are_repaired_not_rejected() {
        assert_eq!(package("cargo", "").name(), "_");
        assert_eq!(package("cargo", " a\n").name(), "a");
        assert_eq!(package("Cargo", "a").manager(), "unknown");
    }
}
