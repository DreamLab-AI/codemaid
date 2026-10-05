//! **Deprecated:** renamed to [`sealmap_extract`] in sealmap 0.2.0.
//!
//! This crate only re-exports `sealmap-extract` so existing builds keep
//! working. Depend on `sealmap-extract` directly and change
//! `sealmap_frontend::` paths to `sealmap_extract::`.

#[doc(inline)]
pub use sealmap_extract::*;
