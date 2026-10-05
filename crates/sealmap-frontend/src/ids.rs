//! The symbol-id builder.
//!
//! Every id a frontend mints goes through here, so all frontends share one
//! grammar. Today that is the v0.1 path grammar: `::`-separated segments,
//! with trait implementations as a `<Trait>` segment between the type and the
//! method (`app::Db::<Store>::put`) and unresolved methods as `?::name`.
//!
//! ```
//! use sealmap_frontend::ids;
//!
//! let db = ids::item_id("app::store", "Db");
//! assert_eq!(db.as_str(), "app::store::Db");
//! let seg = ids::trait_impl_segment("From<std::string::String>");
//! assert_eq!(ids::method_id(&db, Some(&seg), "from").as_str(), "app::store::Db::<From<std.string.String>>::from");
//! assert_eq!(ids::module_id(&["app".into(), "store".into()]).as_str(), "app::store");
//! ```

use sealmap_model::SymbolId;

/// Separator between id segments.
pub const SEP: &str = "::";

/// The textual module path for `segments` (`["app", "db"]` → `app::db`), as
/// used for lookup tables keyed by module.
pub fn module_path(segments: &[String]) -> String {
    segments.join(SEP)
}

/// The id of the module at `segments`.
pub fn module_id(segments: &[String]) -> SymbolId {
    path_id(segments)
}

/// The id spelled by a path as written (`["serde_json", "to_string"]` →
/// `serde_json::to_string`); used for targets outside the analysed code.
pub fn path_id(segments: &[String]) -> SymbolId {
    SymbolId::new(module_path(segments))
}

/// The id of the enclosing module of the module at `segments`, if it has one.
pub fn parent_module_id(segments: &[String]) -> Option<SymbolId> {
    (segments.len() > 1).then(|| module_id(&segments[..segments.len() - 1]))
}

/// The id of item `name` declared in `module` (a [`module_path()`]).
pub fn item_id(module: &str, name: &str) -> SymbolId {
    SymbolId::new(format!("{module}{SEP}{name}"))
}

/// The segment that places trait-implementation methods under their type:
/// the trait's display form in angle brackets, with path separators turned
/// into `.` so the segment stays one segment (`From<a::B>` → `<From<a.B>>`).
pub fn trait_impl_segment(trait_display: &str) -> String {
    format!("<{}>", trait_display.replace(SEP, "."))
}

/// The id of method `name` on `owner`, under a [`trait_impl_segment`] when it
/// implements a trait.
pub fn method_id(owner: &SymbolId, trait_segment: Option<&str>, name: &str) -> SymbolId {
    match trait_segment {
        Some(t) => owner.child(t).child(name),
        None => owner.child(name),
    }
}

/// The placeholder target of a method call whose receiver could not be
/// resolved (`?::name`).
pub fn unresolved_method_id(name: &str) -> SymbolId {
    SymbolId::new(format!("?{SEP}{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids() {
        assert!(parent_module_id(&["app".into()]).is_none());
        assert_eq!(parent_module_id(&["app".into(), "a".into(), "b".into()]).unwrap().as_str(), "app::a");
        let ty = SymbolId::new("app::T");
        assert_eq!(method_id(&ty, None, "go").as_str(), "app::T::go");
        assert_eq!(unresolved_method_id("go").as_str(), "?::go");
    }
}
