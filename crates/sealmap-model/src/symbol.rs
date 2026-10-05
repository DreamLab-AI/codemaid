use serde::{Deserialize, Serialize};

use crate::flow::Flow;
use crate::hash::Fingerprint;
use crate::path::SourcePath;
use crate::sym::{Descriptor, SymbolId};

/// What kind of thing a [`Symbol`] is.
///
/// The set is a superset of what common languages need; frontends map their
/// own constructs onto the closest variant (a TypeScript `interface` would be
/// a [`SymbolKind::Trait`], a Python `class` a [`SymbolKind::Struct`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    /// A module / namespace. Every source file has one.
    Module,
    /// A product type with named or positional fields.
    Struct,
    /// A sum type.
    Enum,
    /// An untagged union.
    Union,
    /// An interface / trait / protocol.
    Trait,
    /// A type alias.
    TypeAlias,
    /// A free function.
    Function,
    /// A function attached to a type (inherent or trait method, including
    /// associated functions without `self`).
    Method,
    /// A constant.
    Const,
    /// A static / global variable.
    Static,
    /// A macro definition.
    Macro,
}

impl SymbolKind {
    /// `true` for kinds that are rendered as classes in a structure diagram.
    pub fn is_type(self) -> bool {
        matches!(self, Self::Struct | Self::Enum | Self::Union | Self::Trait | Self::TypeAlias)
    }

    /// `true` for kinds that have a body and therefore may carry a [`Flow`].
    pub fn is_callable(self) -> bool {
        matches!(self, Self::Function | Self::Method)
    }

    /// The `sym:` descriptor for a symbol of this kind called `name`: a
    /// namespace for modules, a type for structs, enums, unions, traits and
    /// aliases, a method for functions and methods, a term for constants and
    /// statics, a macro for macros.
    ///
    /// ```
    /// # use sealmap_model::SymbolKind;
    /// assert_eq!(SymbolKind::Module.descriptor("net").suffix(), &sealmap_model::Suffix::Namespace);
    /// ```
    pub fn descriptor(self, name: impl Into<String>) -> Descriptor {
        match self {
            Self::Module => Descriptor::namespace(name),
            Self::Struct | Self::Enum | Self::Union | Self::Trait | Self::TypeAlias => Descriptor::r#type(name),
            Self::Function | Self::Method => Descriptor::method(name),
            Self::Const | Self::Static => Descriptor::term(name),
            Self::Macro => Descriptor::r#macro(name),
        }
    }

    /// Lower-case keyword used in diagrams and metadata (`struct`, `fn`, ...).
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Module => "mod",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Union => "union",
            Self::Trait => "trait",
            Self::TypeAlias => "type",
            Self::Function => "fn",
            Self::Method => "method",
            Self::Const => "const",
            Self::Static => "static",
            Self::Macro => "macro",
        }
    }
}

/// Visibility of a symbol or member.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Visible to everyone (`pub`).
    Public,
    /// Visible within the package / crate (`pub(crate)`).
    Crate,
    /// Visible within a named scope (`pub(in path)`, `pub(super)`).
    Restricted(String),
    /// Private to the defining scope.
    Private,
}

impl Visibility {
    /// UML-style marker used by Mermaid class diagrams: `+`, `~`, `#` or `-`.
    pub fn uml_marker(&self) -> char {
        match self {
            Self::Public => '+',
            Self::Crate => '~',
            Self::Restricted(_) => '#',
            Self::Private => '-',
        }
    }
}

/// A source range. Lines and columns are **1-based**; `end` is inclusive of
/// the last line containing the item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Span {
    /// First line (1-based).
    pub start_line: u32,
    /// Column on the first line (1-based).
    pub start_col: u32,
    /// Last line (1-based, inclusive).
    pub end_line: u32,
    /// Column on the last line (1-based).
    pub end_col: u32,
}

impl Span {
    /// Construct a span from line/column pairs.
    pub fn new(start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Self {
        Self { start_line, start_col, end_line, end_col }
    }

    /// `L<start>-L<end>` (or `L<start>` for single-line spans), the compact
    /// form used in diagram metadata.
    ///
    /// ```
    /// # use sealmap_model::Span;
    /// assert_eq!(Span::new(3, 1, 9, 2).compact(), "L3-L9");
    /// assert_eq!(Span::new(4, 1, 4, 9).compact(), "L4");
    /// ```
    pub fn compact(&self) -> String {
        if self.start_line == self.end_line {
            format!("L{}", self.start_line)
        } else {
            format!("L{}-L{}", self.start_line, self.end_line)
        }
    }
}

/// What a [`Member`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberKind {
    /// A named or positional field.
    Field,
    /// An enum variant.
    Variant,
    /// An associated constant.
    AssocConst,
    /// An associated type.
    AssocType,
    /// A required (bodiless) trait method signature.
    RequiredMethod,
}

/// A field, variant or associated item listed *inside* a type in structure
/// diagrams. Methods with bodies are full [`Symbol`]s instead, because they
/// carry their own [`Flow`] and relations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Member {
    /// Member name (`0`, `1`, ... for positional fields).
    pub name: String,
    /// What the member is.
    pub kind: MemberKind,
    /// Type or signature as written, normalised to single spaces
    /// (e.g. `Vec<String>`, `(u32, u32)`, `fn(&self) -> u8`).
    pub ty: Option<String>,
    /// Member visibility.
    pub visibility: Visibility,
    /// Where it is defined.
    pub span: Span,
    /// Codebase-relevant types this member's type mentions, resolved to
    /// canonical ids (primitives and std prelude types are omitted).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<SymbolId>,
}

/// A named definition in the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Canonical id.
    pub id: SymbolId,
    /// Short name.
    pub name: String,
    /// Kind of definition.
    pub kind: SymbolKind,
    /// Visibility.
    pub visibility: Visibility,
    /// File the definition lives in.
    pub file: SourcePath,
    /// Where in the file.
    pub span: Span,
    /// Fingerprint of the symbol's contract (name, visibility, attributes,
    /// generics, parameters, return type; see `sealmap-frontend`'s
    /// `fingerprint` module for the per-kind rules). Whitespace, comments and
    /// position never affect it.
    pub sig_hash: Fingerprint,
    /// Fingerprint of the symbol's implementation (its body). Excludes the
    /// name, so a renamed symbol keeps it.
    pub body_hash: Fingerprint,
    /// Enclosing symbol (the type for a method, the module for an item).
    pub parent: Option<SymbolId>,
    /// One-line signature, e.g. `pub fn connect(&self, addr: &str) -> Result<Conn>`.
    pub signature: Option<String>,
    /// First sentence of the doc comment, if any. Kept short on purpose: it is
    /// context for an agent, not a copy of the docs.
    pub doc: Option<String>,
    /// Generic parameters as written (`T: Clone`, `'a`, `const N: usize`).
    pub generics: Vec<String>,
    /// Notable attributes / modifiers: derives (`derive(Debug)`), `async`,
    /// `unsafe`, `const`, `test`, `cfg(...)`, trait name for trait impls, etc.
    pub tags: Vec<String>,
    /// Fields, variants and associated items.
    pub members: Vec<Member>,
    /// Ordered control/call flow of the body, for callables.
    pub flow: Option<Flow>,
}

impl Symbol {
    /// A symbol with only the required fields set; everything else empty /
    /// private, and both fingerprints unset ([`Fingerprint::is_unset`]).
    /// Frontends fill in the rest.
    pub fn new(id: SymbolId, name: impl Into<String>, kind: SymbolKind, file: SourcePath) -> Self {
        Self {
            parent: id.parent(),
            id,
            name: name.into(),
            kind,
            visibility: Visibility::Private,
            file,
            span: Span::default(),
            sig_hash: Fingerprint::default(),
            body_hash: Fingerprint::default(),
            signature: None,
            doc: None,
            generics: Vec::new(),
            tags: Vec::new(),
            members: Vec::new(),
            flow: None,
        }
    }
}

/// The kind of a directed edge between two symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    /// `from` lexically contains `to` (module → item, type → method).
    Contains,
    /// `from` implements trait `to`.
    Implements,
    /// Trait `from` has supertrait `to`.
    Extends,
    /// `from` has a field whose type mentions `to` (composition / aggregation).
    FieldType,
    /// `from`'s signature mentions `to` (parameters, return, bounds).
    Uses,
    /// Module `from` imports `to`.
    Imports,
    /// Callable `from` calls `to` (aggregated from its [`Flow`]).
    Calls,
}

impl RelationKind {
    /// Lower-case label used in diagrams and metadata.
    pub fn label(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::Implements => "implements",
            Self::Extends => "extends",
            Self::FieldType => "field",
            Self::Uses => "uses",
            Self::Imports => "imports",
            Self::Calls => "calls",
        }
    }
}

/// How sure the frontend is that a relation's target is correct.
///
/// Static analysis without type inference cannot resolve everything (method
/// calls on values of unknown type are the classic case). Rather than drop
/// such edges or present them as fact, sealmap labels them, so an agent can
/// weigh them accordingly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Resolved through explicit paths, imports or declared types to a symbol
    /// defined in the codebase.
    Exact,
    /// Matched by name or local inference; probably right, not proven.
    Inferred,
    /// Points outside the codebase (std, dependencies) or was not resolved.
    External,
}

impl Confidence {
    /// Single-character marker used in compact metadata (`=`, `~`, `?`).
    pub fn marker(self) -> char {
        match self {
            Self::Exact => '=',
            Self::Inferred => '~',
            Self::External => '?',
        }
    }
}

/// A directed, typed edge between two symbols.
///
/// Ordering is derived field by field, which is what makes relation sets in a
/// [`Codebase`](crate::Codebase) iterate deterministically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Relation {
    /// Source symbol.
    pub from: SymbolId,
    /// Target symbol (may be external).
    pub to: SymbolId,
    /// Edge kind.
    pub kind: RelationKind,
    /// Resolution confidence.
    pub confidence: Confidence,
}

impl Relation {
    /// Construct a relation.
    pub fn new(from: SymbolId, to: SymbolId, kind: RelationKind, confidence: Confidence) -> Self {
        Self { from, to, kind, confidence }
    }
}
