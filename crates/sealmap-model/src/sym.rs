//! The `sym:` symbol-id grammar.
//!
//! A [`SymbolId`] names a definition by **where it sits in the program**, never
//! by where it sits on disk: no file path, no line number. Moving an item
//! between files keeps its id; renaming it, or moving it to another module,
//! changes it. Whether the definition's *content* changed is a separate
//! question, answered by content fingerprints kept beside the id: identity
//! and freshness are kept apart, as rustc keeps `DefPath` apart from its
//! query fingerprints.
//!
//! The grammar follows SCIP's symbol strings (descriptor suffixes make the
//! kind explicit), so a global id maps to a SCIP symbol by replacing the
//! `sym:` prefix with a scheme name.
//!
//! # Grammar
//!
//! ```text
//! id          ::= 'sym:' ( global | path | unresolved )
//! global      ::= manager ' ' package ' ' version ( ' ' descriptor+ )?
//! path        ::= 'extern ' name ( '::' name )*
//! unresolved  ::= '? ' name
//!
//! manager     ::= [a-z] [a-z0-9-]*          -- not `extern`
//! package     ::= field                     -- not `.`
//! version     ::= '.' | field               -- `.` = the current tree
//! field       ::= any characters except control characters, non-empty,
//!                 no leading or trailing space; a space is written twice
//!
//! descriptor  ::= name '/'                  -- namespace (module)
//!               | name '#'                  -- type (struct, enum, union, trait, alias)
//!               | name '.'                  -- term (const, static)
//!               | name '(' disambiguator? ').'   -- method or function
//!               | name ':'                  -- meta
//!               | name '!'                  -- macro
//!               | '[' name ']'              -- type parameter (also: trait-impl block)
//!               | '(' name ')'              -- parameter
//! disambiguator ::= simple+
//!
//! name        ::= simple+ | '`' ( any character, '`' written '``' )* '`'
//! simple      ::= [A-Za-z0-9_+$-]
//! ```
//!
//! A name made only of `simple` characters is written bare; anything else
//! (including the empty name) is quoted in backticks, with backticks doubled.
//! A global id without descriptors names the package root (a crate's root
//! module).
//!
//! The three forms are:
//!
//! * **global**: a definition with known kinds all the way down. Everything a
//!   language adapter *defines* has a global id. The version is `.` for "the tree
//!   being analysed", so releasing a new version does not churn every id.
//! * **path**: a path as written that could not be resolved to kinds, such
//!   as a call into a dependency (`sym:extern serde_json::to_string`). These
//!   are references, never definitions.
//! * **unresolved**: a method called on a receiver of unknown type
//!   (`sym:? insert`).
//!
//! # Examples
//!
//! ```
//! use sealmap_model::{Descriptor, Package, SymbolId};
//!
//! let pkg = Package::current("cargo", "shop").unwrap();
//! let db = SymbolId::global(pkg, vec![Descriptor::namespace("db"), Descriptor::r#type("Db")]);
//! let put = db.child(Descriptor::type_parameter("Store")).unwrap().child(Descriptor::method("put")).unwrap();
//!
//! assert_eq!(db.to_string(), "sym:cargo shop . db/Db#");
//! assert_eq!(put.to_string(), "sym:cargo shop . db/Db#[Store]put().");
//! // Methods in a trait-impl block belong to the type.
//! assert_eq!(put.parent(), Some(db.clone()));
//!
//! // Parsing is the exact inverse of printing.
//! assert_eq!("sym:cargo shop . db/Db#[Store]put().".parse::<SymbolId>().unwrap(), put);
//!
//! // Names outside the simple set are quoted.
//! let from = db.child(Descriptor::type_parameter("From<String>")).unwrap().child(Descriptor::method("from")).unwrap();
//! assert_eq!(from.to_string(), "sym:cargo shop . db/Db#[`From<String>`]from().");
//!
//! // References that never resolved to kinds.
//! assert_eq!(SymbolId::path(["tokio", "spawn"]).unwrap().to_string(), "sym:extern tokio::spawn");
//! assert_eq!(SymbolId::unresolved("insert").to_string(), "sym:? insert");
//! ```

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The prefix every printed id starts with.
pub const SYM_PREFIX: &str = "sym:";

/// The reserved word that introduces a [path id](SymbolId::path).
const EXTERN: &str = "extern";

/// The canonical identity of a symbol. See the [module docs](self) for the
/// grammar.
///
/// An id holds its canonical text (shared, so cloning is cheap). Because
/// printing is injective and only canonical text parses, two ids are equal
/// exactly when their texts are, and ids order by their text. A parent's text
/// is a prefix of its children's, so a parent always sorts before them.
/// Structural accessors ([`descriptors`](Self::descriptors),
/// [`package`](Self::package), ...) read the text on demand.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(Arc<str>);

/// The structured form, used to build and validate ids.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Repr {
    Global { package: Package, descriptors: Vec<Descriptor> },
    Path(Vec<String>),
    Unresolved(String),
}

/// The package part of a global id: manager, name and version.
///
/// ```
/// use sealmap_model::{Package, Version};
///
/// let p = Package::current("npm", "@acme/web app").unwrap();
/// assert_eq!(p.name(), "@acme/web app");
/// assert_eq!(p.version(), &Version::Current);
/// assert!(Package::current("Cargo", "x").is_err()); // managers are lower-case
/// assert!(Package::current("cargo", " x").is_err()); // no edge spaces
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Package {
    manager: String,
    name: String,
    version: Version,
}

/// The version component of a [`Package`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    /// The tree being analysed, printed `.`.
    Current,
    /// A named release, e.g. `1.2.0`.
    Release(String),
}

/// One step of a global id: a name plus the kind-explicit suffix.
///
/// ```
/// use sealmap_model::{Descriptor, Suffix};
///
/// let d = Descriptor::method("run");
/// assert_eq!(d.name(), "run");
/// assert_eq!(d.suffix(), &Suffix::Method { disambiguator: None });
/// assert!(Descriptor::new("f", Suffix::Method { disambiguator: Some("a b".into()) }).is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Descriptor {
    name: String,
    suffix: Suffix,
}

/// The kind of a [`Descriptor`], after SCIP's descriptor suffixes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suffix {
    /// `name/`: a module or namespace.
    Namespace,
    /// `name#`: a type (struct, enum, union, trait, type alias, interface).
    Type,
    /// `name.`: a term (constant, static, variable).
    Term,
    /// `name(disambiguator).`: a function or method. The optional
    /// disambiguator tells overloads apart.
    Method {
        /// Overload disambiguator, made of simple characters only.
        disambiguator: Option<String>,
    },
    /// `[name]`: a type parameter. Within a type, it also marks a
    /// trait-implementation block: `Db#[Store]put().` is `put` from
    /// `impl Store for Db`.
    TypeParameter,
    /// `(name)`: a parameter.
    Parameter,
    /// `name:`: metadata.
    Meta,
    /// `name!`: a macro.
    Macro,
}

/// Why an id or one of its parts could not be built or parsed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    /// The text does not start with `sym:`.
    #[error("symbol id must start with `sym:`")]
    MissingPrefix,
    /// A manager name outside `[a-z][a-z0-9-]*`, or the reserved `extern`.
    #[error("invalid package manager `{0}`")]
    Manager(String),
    /// An empty package name or version, `.` as a package name, a leading or
    /// trailing space, or a control character.
    #[error("invalid package field `{0}`")]
    Field(String),
    /// A method disambiguator with characters outside the simple set.
    #[error("invalid method disambiguator `{0}`")]
    Disambiguator(String),
    /// A path id needs at least one segment.
    #[error("a path id needs at least one segment")]
    EmptyPath,
    /// The text does not follow the grammar at byte offset `at`.
    #[error("malformed symbol id at byte {at}: {why}")]
    Syntax {
        /// Byte offset into the input.
        at: usize,
        /// What was expected.
        why: &'static str,
    },
}

// ------------------------------------------------------------------ Package

impl Package {
    /// A package with an explicit version.
    pub fn new(manager: impl Into<String>, name: impl Into<String>, version: Version) -> Result<Self, IdError> {
        let manager = manager.into();
        let name = name.into();
        check_manager(&manager)?;
        check_field(&name)?;
        if name == "." {
            return Err(IdError::Field(name));
        }
        if let Version::Release(v) = &version {
            check_field(v)?;
            if v == "." {
                return Err(IdError::Field(v.clone()));
            }
        }
        Ok(Self { manager, name, version })
    }

    /// A package at [`Version::Current`].
    pub fn current(manager: impl Into<String>, name: impl Into<String>) -> Result<Self, IdError> {
        Self::new(manager, name, Version::Current)
    }

    /// The package manager (`cargo`, `npm`, ...).
    pub fn manager(&self) -> &str {
        &self.manager
    }

    /// The package name (for Rust, the crate name as used in paths).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The version.
    pub fn version(&self) -> &Version {
        &self.version
    }
}

fn check_manager(m: &str) -> Result<(), IdError> {
    let mut cs = m.chars();
    let ok = matches!(cs.next(), Some('a'..='z'))
        && cs.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && m != EXTERN;
    if ok { Ok(()) } else { Err(IdError::Manager(m.to_owned())) }
}

fn check_field(f: &str) -> Result<(), IdError> {
    let ok = !f.is_empty() && !f.starts_with(' ') && !f.ends_with(' ') && !f.chars().any(char::is_control);
    if ok { Ok(()) } else { Err(IdError::Field(f.to_owned())) }
}

// --------------------------------------------------------------- Descriptor

impl Descriptor {
    /// A descriptor, checking the method disambiguator if there is one.
    pub fn new(name: impl Into<String>, suffix: Suffix) -> Result<Self, IdError> {
        if let Suffix::Method { disambiguator: Some(d) } = &suffix {
            if d.is_empty() || !d.chars().all(is_simple) {
                return Err(IdError::Disambiguator(d.clone()));
            }
        }
        Ok(Self { name: name.into(), suffix })
    }

    /// `name/`.
    pub fn namespace(name: impl Into<String>) -> Self {
        Self { name: name.into(), suffix: Suffix::Namespace }
    }

    /// `name#`.
    pub fn r#type(name: impl Into<String>) -> Self {
        Self { name: name.into(), suffix: Suffix::Type }
    }

    /// `name.`.
    pub fn term(name: impl Into<String>) -> Self {
        Self { name: name.into(), suffix: Suffix::Term }
    }

    /// `name().`.
    pub fn method(name: impl Into<String>) -> Self {
        Self { name: name.into(), suffix: Suffix::Method { disambiguator: None } }
    }

    /// `[name]`.
    pub fn type_parameter(name: impl Into<String>) -> Self {
        Self { name: name.into(), suffix: Suffix::TypeParameter }
    }

    /// `(name)`.
    pub fn parameter(name: impl Into<String>) -> Self {
        Self { name: name.into(), suffix: Suffix::Parameter }
    }

    /// `name:`.
    pub fn meta(name: impl Into<String>) -> Self {
        Self { name: name.into(), suffix: Suffix::Meta }
    }

    /// `name!`.
    pub fn r#macro(name: impl Into<String>) -> Self {
        Self { name: name.into(), suffix: Suffix::Macro }
    }

    /// The name, unescaped.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The kind suffix.
    pub fn suffix(&self) -> &Suffix {
        &self.suffix
    }

    fn write(&self, out: &mut String) {
        match &self.suffix {
            Suffix::TypeParameter => {
                out.push('[');
                write_name(&self.name, out);
                out.push(']');
            }
            Suffix::Parameter => {
                out.push('(');
                write_name(&self.name, out);
                out.push(')');
            }
            other => {
                write_name(&self.name, out);
                match other {
                    Suffix::Namespace => out.push('/'),
                    Suffix::Type => out.push('#'),
                    Suffix::Term => out.push('.'),
                    Suffix::Meta => out.push(':'),
                    Suffix::Macro => out.push('!'),
                    Suffix::Method { disambiguator } => {
                        out.push('(');
                        out.push_str(disambiguator.as_deref().unwrap_or(""));
                        out.push_str(").");
                    }
                    Suffix::TypeParameter | Suffix::Parameter => unreachable!("handled above"),
                }
            }
        }
    }
}

/// SCIP's simple-identifier characters.
fn is_simple(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-' | '$')
}

fn write_name(name: &str, out: &mut String) {
    if !name.is_empty() && name.chars().all(is_simple) {
        out.push_str(name);
        return;
    }
    out.push('`');
    for c in name.chars() {
        if c == '`' {
            out.push('`');
        }
        out.push(c);
    }
    out.push('`');
}

fn write_field(field: &str, out: &mut String) {
    for c in field.chars() {
        if c == ' ' {
            out.push(' ');
        }
        out.push(c);
    }
}

// ----------------------------------------------------------------- SymbolId

impl SymbolId {
    fn from_repr(repr: &Repr) -> Self {
        Self(print(repr).into())
    }

    /// A global id: `package` followed by `descriptors` (empty for the
    /// package root).
    pub fn global(package: Package, descriptors: Vec<Descriptor>) -> Self {
        Self::from_repr(&Repr::Global { package, descriptors })
    }

    /// The package root (a crate's root module).
    pub fn package_root(package: Package) -> Self {
        Self::global(package, Vec::new())
    }

    /// A path as written, for references whose kinds are unknown.
    pub fn path<I, S>(segments: I) -> Result<Self, IdError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let segs: Vec<String> = segments.into_iter().map(Into::into).collect();
        if segs.is_empty() {
            return Err(IdError::EmptyPath);
        }
        Ok(Self::from_repr(&Repr::Path(segs)))
    }

    /// A method called on a receiver of unknown type.
    pub fn unresolved(name: impl Into<String>) -> Self {
        Self::from_repr(&Repr::Unresolved(name.into()))
    }

    /// Parse the canonical form. The exact inverse of [`Display`](fmt::Display):
    /// only canonical text is accepted (a simple name in backticks, for
    /// instance, is rejected), so `parse(s)` succeeding implies
    /// `parse(s)?.to_string() == s`.
    pub fn parse(text: &str) -> Result<Self, IdError> {
        let repr = Parser { s: text, pos: 0 }.id()?;
        // The parser only accepts canonical text; printing back is the
        // guard that keeps that true, since ids compare by text.
        if print(&repr) != text {
            return Err(IdError::Syntax { at: 0, why: "not in canonical form" });
        }
        Ok(Self(text.into()))
    }

    /// The smallest possible id, used as a range bound.
    pub(crate) fn min_value() -> Self {
        Self("".into())
    }

    /// The canonical text, `sym:` prefix included.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn repr(&self) -> Repr {
        Parser { s: &self.0, pos: 0 }.id().expect("a SymbolId always holds canonical text")
    }

    fn shape(&self) -> Shape<'_> {
        Shape::of(&self.0)
    }

    /// `true` for global (kind-explicit) ids.
    pub fn is_global(&self) -> bool {
        matches!(self.shape(), Shape::Global { .. })
    }

    /// `true` for [`SymbolId::unresolved`] ids.
    pub fn is_unresolved(&self) -> bool {
        matches!(self.shape(), Shape::Unresolved(_))
    }

    /// The package of a global id.
    pub fn package(&self) -> Option<Package> {
        match self.repr() {
            Repr::Global { package, .. } => Some(package),
            _ => None,
        }
    }

    /// The descriptors of a global id (empty for the other forms).
    pub fn descriptors(&self) -> Vec<Descriptor> {
        match self.repr() {
            Repr::Global { descriptors, .. } => descriptors,
            _ => Vec::new(),
        }
    }

    /// The segments of a path id.
    pub fn segments(&self) -> Option<Vec<String>> {
        match self.repr() {
            Repr::Path(s) => Some(s),
            _ => None,
        }
    }

    /// The last descriptor of a global id.
    pub fn last(&self) -> Option<Descriptor> {
        self.descriptors().pop()
    }

    /// The short name: the last descriptor's name, the package name of a
    /// package root, the last path segment, or the unresolved method name.
    ///
    /// ```
    /// # use sealmap_model::SymbolId;
    /// assert_eq!(SymbolId::parse("sym:cargo a . b/C#go().").unwrap().name(), "go");
    /// assert_eq!(SymbolId::parse("sym:cargo a .").unwrap().name(), "a");
    /// ```
    pub fn name(&self) -> Cow<'_, str> {
        match self.shape() {
            Shape::Global { name, descriptors, .. } => {
                scan_descriptors(descriptors).last().map_or_else(|| unfield(name), |d| unquote(d.name))
            }
            Shape::Path(body) => scan_path(body).last().map_or(Cow::Borrowed(""), |s| unquote(s)),
            Shape::Unresolved(raw) => unquote(raw),
        }
    }

    /// The first name: the package name, or the first path segment.
    /// `None` for unresolved ids.
    pub fn root(&self) -> Option<Cow<'_, str>> {
        match self.shape() {
            Shape::Global { name, .. } => Some(unfield(name)),
            Shape::Path(body) => scan_path(body).first().map(|s| unquote(s)),
            Shape::Unresolved(_) => None,
        }
    }

    /// The enclosing definition. For a global id, the last descriptor is
    /// dropped, then any trailing type-parameter descriptors, so a method in
    /// a trait-impl block has its type as parent. For a path id, the last
    /// segment is dropped. A package root, a one-segment path and an
    /// unresolved id have no parent.
    ///
    /// ```
    /// # use sealmap_model::SymbolId;
    /// let m = SymbolId::parse("sym:cargo a . T#[Tr]f().").unwrap();
    /// assert_eq!(m.parent().unwrap().to_string(), "sym:cargo a . T#");
    /// assert_eq!(SymbolId::parse("sym:cargo a . T#").unwrap().parent().unwrap().to_string(), "sym:cargo a .");
    /// assert!(SymbolId::parse("sym:cargo a .").unwrap().parent().is_none());
    /// ```
    pub fn parent(&self) -> Option<SymbolId> {
        let text: &str = &self.0;
        match self.shape() {
            Shape::Global { descriptors, .. } => {
                let mut spans = scan_descriptors(descriptors);
                spans.pop()?;
                while spans.last().is_some_and(|d| d.kind == b'[') {
                    spans.pop();
                }
                let base = text.len() - descriptors.len();
                let end = match spans.last() {
                    Some(d) => base + d.end,
                    // No descriptors left: drop the separating space too.
                    None => base - 1,
                };
                Some(Self(text[..end].into()))
            }
            Shape::Path(body) => {
                let segs = scan_path(body);
                if segs.len() < 2 {
                    return None;
                }
                let last = segs[segs.len() - 1];
                let end = text.len() - last.len() - "::".len();
                Some(Self(text[..end].into()))
            }
            Shape::Unresolved(_) => None,
        }
    }

    /// Append a descriptor to a global id. `None` for the other forms,
    /// which have no kinds to extend.
    pub fn child(&self, descriptor: Descriptor) -> Option<SymbolId> {
        let Shape::Global { descriptors, .. } = self.shape() else { return None };
        let mut out = String::with_capacity(self.0.len() + descriptor.name.len() + 4);
        out.push_str(&self.0);
        if descriptors.is_empty() {
            out.push(' ');
        }
        descriptor.write(&mut out);
        Some(Self(out.into()))
    }

    /// Every name in order: package and descriptor names for a global id,
    /// the segments of a path id, the name of an unresolved id.
    pub fn names(&self) -> Vec<Cow<'_, str>> {
        match self.shape() {
            Shape::Global { name, descriptors, .. } => std::iter::once(unfield(name))
                .chain(scan_descriptors(descriptors).into_iter().map(|d| unquote(d.name)))
                .collect(),
            Shape::Path(body) => scan_path(body).into_iter().map(unquote).collect(),
            Shape::Unresolved(raw) => vec![unquote(raw)],
        }
    }

    /// Continue an id with a segment of unknown kind: the result is a path
    /// id made of [`names`](Self::names) plus `segment`. This is how a
    /// reference that leaves known territory (`krate::known::unknown`) is
    /// recorded.
    ///
    /// ```
    /// # use sealmap_model::SymbolId;
    /// let m = SymbolId::parse("sym:cargo a . net/").unwrap();
    /// assert_eq!(m.extend_path("dial").to_string(), "sym:extern a::net::dial");
    /// ```
    pub fn extend_path(&self, segment: &str) -> SymbolId {
        let mut s: Vec<String> = self.names().into_iter().map(Cow::into_owned).collect();
        s.push(segment.to_owned());
        Self::from_repr(&Repr::Path(s))
    }

    /// A borrowed view of the id's parts. Unlike [`package`](Self::package)
    /// and [`descriptors`](Self::descriptors) it allocates nothing but the
    /// descriptor list (and a name only when it needs unescaping), so it
    /// suits hot paths such as deriving a diagram id per call site.
    ///
    /// ```
    /// use sealmap_model::{DescriptorKind, IdView, SymbolId};
    ///
    /// let id = SymbolId::parse("sym:cargo shop . db/Db#[Store]put().").unwrap();
    /// let IdView::Global(g) = id.view() else { unreachable!() };
    /// assert_eq!((g.manager, &*g.package, g.version.is_none()), ("cargo", "shop", true));
    /// let kinds: Vec<_> = g.descriptors.iter().map(|d| (d.kind, &*d.name)).collect();
    /// assert_eq!(kinds, [
    ///     (DescriptorKind::Namespace, "db"),
    ///     (DescriptorKind::Type, "Db"),
    ///     (DescriptorKind::TypeParameter, "Store"),
    ///     (DescriptorKind::Method, "put"),
    /// ]);
    /// ```
    pub fn view(&self) -> IdView<'_> {
        match self.shape() {
            Shape::Global { manager, name, version, descriptors } => IdView::Global(GlobalView {
                manager,
                package: unfield(name),
                version: (version != ".").then(|| unfield(version)),
                descriptors: scan_descriptors(descriptors)
                    .into_iter()
                    .map(|d| DescriptorView {
                        name: unquote(d.name),
                        kind: match d.kind {
                            b'/' => DescriptorKind::Namespace,
                            b'#' => DescriptorKind::Type,
                            b'(' => DescriptorKind::Method,
                            b'[' => DescriptorKind::TypeParameter,
                            b')' => DescriptorKind::Parameter,
                            b':' => DescriptorKind::Meta,
                            b'!' => DescriptorKind::Macro,
                            _ => DescriptorKind::Term,
                        },
                        disambiguator: (!d.disambiguator.is_empty()).then_some(d.disambiguator),
                    })
                    .collect(),
            }),
            Shape::Path(body) => IdView::Path(scan_path(body).into_iter().map(unquote).collect()),
            Shape::Unresolved(raw) => IdView::Unresolved(unquote(raw)),
        }
    }

    /// The names joined by `::`, for human-facing labels. Not injective:
    /// never use it as a key.
    pub fn display_path(&self) -> String {
        self.names().join("::")
    }
}

impl fmt::Debug for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SymbolId({:?})", &*self.0)
    }
}

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for SymbolId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for SymbolId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SymbolId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// The canonical text of `repr`.
fn print(repr: &Repr) -> String {
    let mut out = String::from(SYM_PREFIX);
    match repr {
        Repr::Global { package, descriptors } => {
            out.push_str(&package.manager);
            out.push(' ');
            write_field(&package.name, &mut out);
            out.push(' ');
            match &package.version {
                Version::Current => out.push('.'),
                Version::Release(v) => write_field(v, &mut out),
            }
            if !descriptors.is_empty() {
                out.push(' ');
                for d in descriptors {
                    d.write(&mut out);
                }
            }
        }
        Repr::Path(segs) => {
            out.push_str(EXTERN);
            out.push(' ');
            for (i, s) in segs.iter().enumerate() {
                if i > 0 {
                    out.push_str("::");
                }
                write_name(s, &mut out);
            }
        }
        Repr::Unresolved(n) => {
            out.push_str("? ");
            write_name(n, &mut out);
        }
    }
    out
}

/// A borrowed view of a [`SymbolId`], from [`SymbolId::view`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdView<'a> {
    /// A global id.
    Global(GlobalView<'a>),
    /// A path id's segments.
    Path(Vec<Cow<'a, str>>),
    /// An unresolved method's name.
    Unresolved(Cow<'a, str>),
}

/// The parts of a global id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalView<'a> {
    /// The package manager.
    pub manager: &'a str,
    /// The package name.
    pub package: Cow<'a, str>,
    /// The release version, `None` for [`Version::Current`].
    pub version: Option<Cow<'a, str>>,
    /// The descriptors in order.
    pub descriptors: Vec<DescriptorView<'a>>,
}

/// One descriptor of a [`GlobalView`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptorView<'a> {
    /// The name, unescaped.
    pub name: Cow<'a, str>,
    /// The kind.
    pub kind: DescriptorKind,
    /// A method's disambiguator.
    pub disambiguator: Option<&'a str>,
}

/// The kind of a descriptor, as in [`Suffix`] without the payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DescriptorKind {
    /// `name/`.
    Namespace,
    /// `name#`.
    Type,
    /// `name.`.
    Term,
    /// `name(…).`.
    Method,
    /// `[name]`.
    TypeParameter,
    /// `(name)`.
    Parameter,
    /// `name:`.
    Meta,
    /// `name!`.
    Macro,
}

// ------------------------------------------------------------------ scanner
//
// Borrowed views of text already known to be canonical. They never allocate
// unless a name needs unescaping, and never fail: on canonical text every
// branch below is the one the printer took.

enum Shape<'a> {
    /// Raw manager, package-name and version fields; `descriptors` is the
    /// raw text after the version's separating space (empty for a package
    /// root).
    Global { manager: &'a str, name: &'a str, version: &'a str, descriptors: &'a str },
    /// The raw text after `extern `.
    Path(&'a str),
    /// The raw name after `? `.
    Unresolved(&'a str),
}

impl<'a> Shape<'a> {
    fn of(text: &'a str) -> Self {
        let body = text.strip_prefix(SYM_PREFIX).unwrap_or(text);
        if let Some(raw) = body.strip_prefix("? ") {
            return Shape::Unresolved(raw);
        }
        if let Some(raw) = body.strip_prefix("extern ") {
            return Shape::Path(raw);
        }
        // Field separators are single spaces; literal spaces inside fields
        // come in pairs.
        let bytes = body.as_bytes();
        let mut seps = Vec::with_capacity(3);
        let mut i = 0;
        while i < bytes.len() && seps.len() < 3 {
            if bytes[i] == b' ' {
                let run = bytes[i..].iter().take_while(|&&b| b == b' ').count();
                if run % 2 == 1 {
                    seps.push(i + run - 1);
                }
                i += run;
            } else {
                i += 1;
            }
        }
        let (a, b) = match (seps.first(), seps.get(1)) {
            (Some(&a), Some(&b)) => (a, b),
            _ => (0, 0),
        };
        let manager = &body[..a];
        let name = body.get(a + 1..b).unwrap_or("");
        let version = seps.get(2).map_or(body.get(b + 1..).unwrap_or(""), |&c| &body[b + 1..c]);
        let descriptors = seps.get(2).map_or("", |&c| &body[c + 1..]);
        Shape::Global { manager, name, version, descriptors }
    }
}

/// One descriptor in raw text: byte range, raw name and kind byte (the
/// suffix character, or `[` / `(` for type parameters and parameters).
struct DescSpan<'a> {
    end: usize,
    name: &'a str,
    /// The suffix byte (`/ # . ( : !`), `[` for a type parameter, `)` for a
    /// parameter.
    kind: u8,
    /// A method's raw disambiguator (empty when there is none).
    disambiguator: &'a str,
}

/// The length of the raw name at the start of `s` (quoted or simple).
fn raw_name_len(s: &str) -> usize {
    let b = s.as_bytes();
    if b.first() == Some(&b'`') {
        let mut i = 1;
        while i < b.len() {
            if b[i] == b'`' {
                if b.get(i + 1) == Some(&b'`') {
                    i += 2;
                    continue;
                }
                return i + 1;
            }
            i += 1;
        }
        return b.len();
    }
    s.char_indices().find(|&(_, c)| !is_simple(c)).map_or(s.len(), |(i, _)| i)
}

fn scan_descriptors(s: &str) -> Vec<DescSpan<'_>> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < s.len() {
        let rest = &s[pos..];
        let (kind, name, len, disambiguator) = match rest.as_bytes()[0] {
            open @ (b'[' | b'(') => {
                let n = raw_name_len(&rest[1..]);
                (if open == b'(' { b')' } else { b'[' }, &rest[1..1 + n], n + 2, "")
            }
            _ => {
                let n = raw_name_len(rest);
                let kind = rest.as_bytes().get(n).copied().unwrap_or(b'.');
                if kind == b'(' {
                    // `name(disambiguator).`
                    let close = rest[n..].find(").").unwrap_or(rest.len() - n);
                    (kind, &rest[..n], n + close + 2, rest.get(n + 1..n + close).unwrap_or(""))
                } else {
                    (kind, &rest[..n], n + 1, "")
                }
            }
        };
        pos += len.max(1);
        out.push(DescSpan { end: pos.min(s.len()), name, kind, disambiguator });
    }
    out
}

fn scan_path(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut pos = 0;
    loop {
        let n = raw_name_len(&s[pos..]);
        out.push(&s[pos..pos + n]);
        pos += n;
        if s[pos..].starts_with("::") {
            pos += 2;
        } else {
            return out;
        }
    }
}

/// A raw name without its quotes and doubled backticks.
fn unquote(raw: &str) -> Cow<'_, str> {
    match raw.strip_prefix('`').and_then(|r| r.strip_suffix('`')) {
        Some(inner) if inner.contains("``") => Cow::Owned(inner.replace("``", "`")),
        Some(inner) => Cow::Borrowed(inner),
        None => Cow::Borrowed(raw),
    }
}

/// A raw package field without its doubled spaces.
fn unfield(raw: &str) -> Cow<'_, str> {
    if raw.contains("  ") { Cow::Owned(raw.replace("  ", " ")) } else { Cow::Borrowed(raw) }
}

// ------------------------------------------------------------------- parser

struct Parser<'a> {
    s: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn err(&self, why: &'static str) -> IdError {
        IdError::Syntax { at: self.pos, why }
    }

    fn rest(&self) -> &str {
        &self.s[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn eat(&mut self, lit: &str) -> bool {
        if self.rest().starts_with(lit) {
            self.pos += lit.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, lit: &str, why: &'static str) -> Result<(), IdError> {
        if self.eat(lit) { Ok(()) } else { Err(self.err(why)) }
    }

    fn done(&self) -> Result<(), IdError> {
        if self.pos == self.s.len() { Ok(()) } else { Err(self.err("unexpected trailing text")) }
    }

    fn id(mut self) -> Result<Repr, IdError> {
        if !self.eat(SYM_PREFIX) {
            return Err(IdError::MissingPrefix);
        }
        if self.eat("? ") {
            let name = self.name()?;
            self.done()?;
            return Ok(Repr::Unresolved(name));
        }
        if self.eat("extern ") {
            let mut segs = vec![self.name()?];
            while self.eat("::") {
                segs.push(self.name()?);
            }
            self.done()?;
            return Ok(Repr::Path(segs));
        }
        let manager = self.field()?;
        self.expect(" ", "expected a space after the manager")?;
        let name = self.field()?;
        self.expect(" ", "expected a space after the package name")?;
        let version = self.field()?;
        let version = if version == "." { Version::Current } else { Version::Release(version) };
        let package = Package::new(manager, name, version)?;
        let mut descriptors = Vec::new();
        if self.pos < self.s.len() {
            self.expect(" ", "expected a space before the descriptors")?;
            if self.pos == self.s.len() {
                return Err(self.err("expected a descriptor"));
            }
            while self.pos < self.s.len() {
                descriptors.push(self.descriptor()?);
            }
        }
        Ok(Repr::Global { package, descriptors })
    }

    /// A package field, up to (not including) the next single space or the
    /// end. A doubled space is a literal space.
    fn field(&mut self) -> Result<String, IdError> {
        let mut out = String::new();
        loop {
            let rest = self.rest();
            if rest.starts_with("  ") {
                out.push(' ');
                self.pos += 2;
                continue;
            }
            match rest.chars().next() {
                None | Some(' ') => break,
                Some(c) => {
                    out.push(c);
                    self.pos += c.len_utf8();
                }
            }
        }
        if out.is_empty() {
            return Err(self.err("empty package field"));
        }
        Ok(out)
    }

    fn name(&mut self) -> Result<String, IdError> {
        if self.eat("`") {
            let mut out = String::new();
            loop {
                match self.peek() {
                    None => return Err(self.err("unterminated quoted name")),
                    Some('`') => {
                        if self.rest().starts_with("``") {
                            out.push('`');
                            self.pos += 2;
                        } else {
                            if !out.is_empty() && out.chars().all(is_simple) {
                                return Err(self.err("a simple name must not be quoted"));
                            }
                            self.pos += 1;
                            return Ok(out);
                        }
                    }
                    Some(c) => {
                        out.push(c);
                        self.pos += c.len_utf8();
                    }
                }
            }
        }
        let start = self.pos;
        while self.peek().is_some_and(is_simple) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(self.err("expected a name"));
        }
        Ok(self.s[start..self.pos].to_owned())
    }

    fn descriptor(&mut self) -> Result<Descriptor, IdError> {
        if self.eat("[") {
            let name = self.name()?;
            self.expect("]", "expected `]`")?;
            return Ok(Descriptor::type_parameter(name));
        }
        if self.eat("(") {
            let name = self.name()?;
            self.expect(")", "expected `)`")?;
            return Ok(Descriptor::parameter(name));
        }
        let name = self.name()?;
        let suffix = match self.peek() {
            Some('/') => Suffix::Namespace,
            Some('#') => Suffix::Type,
            Some('.') => Suffix::Term,
            Some(':') => Suffix::Meta,
            Some('!') => Suffix::Macro,
            Some('(') => {
                self.pos += 1;
                let start = self.pos;
                while self.peek().is_some_and(is_simple) {
                    self.pos += 1;
                }
                let d = &self.s[start..self.pos];
                let disambiguator = (!d.is_empty()).then(|| d.to_owned());
                self.expect(").", "expected `).`")?;
                return Descriptor::new(name, Suffix::Method { disambiguator });
            }
            _ => return Err(self.err("expected a descriptor suffix")),
        };
        self.pos += 1;
        Ok(Descriptor { name, suffix })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt(s: &str) {
        let id = SymbolId::parse(s).unwrap_or_else(|e| panic!("{s}: {e}"));
        assert_eq!(id.to_string(), s);
    }

    #[test]
    fn round_trips_every_form() {
        rt("sym:cargo a .");
        rt("sym:cargo a . b/");
        rt("sym:cargo a . b/C#d().");
        rt("sym:cargo a 1.2.0 b/C#d(+1).");
        rt("sym:cargo a . T#[`From<a::B>`]from().");
        rt("sym:npm @x/y . index/`POST /api`().");
        rt("sym:npm a  b . c/");
        rt("sym:cargo a . f().(x)[T]m:k!v.");
        rt("sym:cargo a . ``#```a``b```.");
        rt("sym:extern tokio::spawn");
        rt("sym:extern `?`::`a b`");
        rt("sym:? insert");
        rt("sym:? ``");
    }

    #[test]
    fn rejects_malformed() {
        for bad in [
            "",
            "cargo a .",
            "sym:",
            "sym:cargo",
            "sym:cargo a",
            "sym:cargo a . ",
            "sym:Cargo a .",
            "sym:extern",
            "sym:extern a::",
            "sym:cargo a . b",
            "sym:cargo a . `b",
            "sym:cargo a . b(x y).",
            "sym:cargo a . b().c",
            "sym:cargo . .",
            "sym:cargo a   .",
            "sym:? a b",
            "sym:? `a`",
        ] {
            assert!(SymbolId::parse(bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn json_is_the_canonical_string() {
        let id = SymbolId::parse("sym:cargo a . b/C#").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"sym:cargo a . b/C#\"");
        assert_eq!(serde_json::from_str::<SymbolId>(&json).unwrap(), id);
        assert!(serde_json::from_str::<SymbolId>("\"a::b\"").is_err());
    }
}
