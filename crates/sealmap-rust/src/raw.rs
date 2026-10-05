//! Pass-1 output: plain, unresolved data extracted from one file.
//!
//! Everything here is owned `String`s and `Vec`s (no syn nodes), so files can
//! be parsed in parallel and resolved afterwards in a single deterministic
//! pass. The flow part of it (steps, calls, receivers) is the shared IR from
//! `sealmap-extract`; the item, `use` and `impl` tables are Rust's own.

use sealmap_model::{ContentHash, Fingerprint, MemberKind, SourcePath, Span, SymbolKind, Visibility};

pub(crate) use sealmap_extract::raw::{Callee, RawCall, RawStep, Recv, Segs};

use crate::layout::FileRole;

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
    pub sig_hash: Fingerprint,
    pub body_hash: Fingerprint,
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
    /// Generic type and const parameters the member declares itself (a
    /// required trait method's own); its item's are in scope too.
    pub type_params: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RawFn {
    pub name: String,
    pub vis: Visibility,
    pub span: Span,
    pub signature: String,
    pub doc: Option<String>,
    pub generics: Vec<String>,
    /// Names of the generic type and const parameters the function declares
    /// itself; the enclosing impl's or trait's are in scope too.
    pub type_params: Vec<String>,
    pub tags: Vec<String>,
    pub sig_refs: Vec<Segs>,
    pub flow: Vec<RawStep>,
    pub sig_hash: Fingerprint,
    pub body_hash: Fingerprint,
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
    /// Names of the generic type and const parameters the item declares.
    pub type_params: Vec<String>,
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
    pub sig_hash: Fingerprint,
    pub body_hash: Fingerprint,
}

#[derive(Debug, Clone)]
pub(crate) struct RawImpl {
    pub module: Segs,
    pub self_ty: Option<Segs>,
    /// Trait path and its display form (`From<String>`).
    pub trait_: Option<(Segs, String)>,
    /// Names of the impl block's generic type and const parameters.
    pub type_params: Vec<String>,
    pub methods: Vec<RawFn>,
}
