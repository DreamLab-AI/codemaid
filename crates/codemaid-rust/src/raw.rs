//! Pass-1 output: plain, unresolved data extracted from one file.
//!
//! Everything here is owned `String`s and `Vec`s (no syn nodes), so files can
//! be parsed in parallel and resolved afterwards in a single deterministic
//! pass.

use codemaid_model::{CallKind, ContentHash, MemberKind, SourcePath, Span, SymbolKind, Visibility};

use crate::layout::FileRole;

/// Path segments as written, generics stripped: `std::collections::HashMap`.
pub(crate) type Segs = Vec<String>;

#[derive(Debug, Clone)]
pub(crate) struct RawFile {
    pub path: SourcePath,
    pub role: FileRole,
    pub text_lines: u32,
    pub hash: ContentHash,
    pub modules: Vec<RawModule>,
    pub items: Vec<RawItem>,
    pub impls: Vec<RawImpl>,
    /// Parse error, if the file could not be parsed. The file still gets a
    /// module symbol so the 1:1 contract holds.
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RawModule {
    pub path: Segs,
    pub span: Span,
    pub doc: Option<String>,
    pub vis: Visibility,
    pub uses: Vec<RawUse>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RawUse {
    /// Local name (`*` for glob imports).
    pub alias: String,
    pub target: Segs,
}

#[derive(Debug, Clone)]
pub(crate) struct RawMember {
    pub name: String,
    pub kind: MemberKind,
    pub ty: Option<String>,
    pub vis: Visibility,
    pub span: Span,
    pub refs: Vec<Segs>,
}

#[derive(Debug, Clone)]
pub(crate) struct RawFn {
    pub name: String,
    pub vis: Visibility,
    pub span: Span,
    pub signature: String,
    pub doc: Option<String>,
    pub generics: Vec<String>,
    pub tags: Vec<String>,
    pub sig_refs: Vec<Segs>,
    pub flow: Vec<RawStep>,
}

#[derive(Debug, Clone)]
pub(crate) struct RawItem {
    pub module: Segs,
    pub name: String,
    pub kind: SymbolKind,
    pub vis: Visibility,
    pub span: Span,
    pub signature: Option<String>,
    pub doc: Option<String>,
    pub generics: Vec<String>,
    pub tags: Vec<String>,
    pub members: Vec<RawMember>,
    /// Types mentioned in signatures / aliases.
    pub sig_refs: Vec<Segs>,
    /// Supertraits (traits only).
    pub supertraits: Vec<Segs>,
    /// Provided trait methods (traits only).
    pub methods: Vec<RawFn>,
    /// Body flow (free functions only).
    pub flow: Vec<RawStep>,
}

#[derive(Debug, Clone)]
pub(crate) struct RawImpl {
    pub module: Segs,
    pub self_ty: Option<Segs>,
    /// Trait path and its display form (`From<String>`).
    pub trait_: Option<(Segs, String)>,
    pub methods: Vec<RawFn>,
}

#[derive(Debug, Clone)]
pub(crate) enum RawStep {
    Call(RawCall),
    Branch(Vec<(String, Vec<RawStep>)>),
    Loop(String, Vec<RawStep>),
    Optional(String, Vec<RawStep>),
    Parallel(Vec<(String, Vec<RawStep>)>),
    Return(String, u32),
}

#[derive(Debug, Clone)]
pub(crate) struct RawCall {
    pub callee: Callee,
    pub label: String,
    pub kind: CallKind,
    pub awaited: bool,
    pub fallible: bool,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub(crate) enum Callee {
    /// `a::b::c(..)`.
    Path(Segs),
    /// `recv.name(..)`.
    Method { recv: Recv, name: String },
}

#[derive(Debug, Clone)]
pub(crate) enum Recv {
    /// `self`.
    SelfValue,
    /// `self.field`.
    SelfField(String),
    /// A local or parameter whose declared/constructed type mentions these
    /// paths (outermost first, e.g. `Arc<Db>` → `[Arc], [Db]`).
    Typed(Vec<Segs>),
    /// A plain variable or field whose type is not evident. Calls on it may
    /// be matched to a distinctive internal method name (`inferred`).
    Untyped,
    /// Anything else (call chains, indexing, literals). Not resolved.
    Unknown,
}
