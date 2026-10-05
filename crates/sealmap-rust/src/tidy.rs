//! Compact printing of syn nodes, through the shared label rules.

use quote::ToTokens;
use sealmap_frontend::labels::squeeze;

/// Print any syn node compactly (see [`squeeze`]).
pub(crate) fn tokens(node: &impl ToTokens) -> String {
    squeeze(&node.to_token_stream().to_string())
}
