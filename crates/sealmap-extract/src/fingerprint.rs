//! Per-symbol fingerprints: `sig_hash` (the contract) and `body_hash` (the
//! implementation).
//!
//! A language adapter feeds a [`Fingerprinter`] a **token stream**: identifiers,
//! punctuation, literals and delimited groups, in source order. Whitespace and
//! comments (doc comments included) are never tokens, so reformatting a file
//! or editing its comments changes no fingerprint. Positions are never
//! hashed either, so moving an item within a file or to another file changes
//! neither of its fingerprints. Every language adapter uses this one module, which is
//! what makes a Rust and a TypeScript fingerprint the same kind of thing.
//!
//! # What goes into each hash
//!
//! The language adapter decides which tokens form the contract and which form the
//! implementation; the rule it follows is:
//!
//! | | `sig_hash` | `body_hash` |
//! |---|---|---|
//! | callables | name, visibility, attributes, generics, parameters, return type, `where` clause; for methods also the impl or trait header they sit in | the body block |
//! | data types | name, visibility, attributes, generics, fields or variants | generics, fields or variants |
//! | traits / interfaces | header plus every member signature | every member, bodies included |
//! | constants, statics | name, visibility, attributes, type | the value |
//! | aliases | the whole declaration | generics and aliased type |
//! | modules | name, visibility, attributes | the fingerprints of every member, in order, plus any other statement (imports, ...) |
//!
//! The name is in `sig_hash` and never in `body_hash`, so a renamed symbol
//! keeps its `body_hash` and a rename detector can match it to its old id.
//!
//! # Normalisation
//!
//! * Tokens are length-prefixed and tagged by kind, so `ab` is never `a b`
//!   and an identifier is never a literal of the same text.
//! * Punctuation spacing is ignored (`a - -b` and `a --b` are the same
//!   tokens): spacing comes from layout, and layout is formatting.
//! * Literal text has `\r\n` and lone `\r` turned into `\n`, so a checkout
//!   with Windows line endings fingerprints like one with Unix endings.
//! * Invisible groups ([`Delim::None`]) contribute their tokens but no
//!   delimiters.
//!
//! The two hashes use distinct BLAKE3 key-derivation contexts, so a
//! signature and a body with identical tokens still get different values.
//! [`ALGORITHM`] names the whole scheme; any change to the encoding or the
//! table above is a new algorithm id, and the golden tests in this crate and
//! in each language adapter fail until it is bumped.
//!
//! # Example
//!
//! ```
//! use sealmap_extract::fingerprint::{Delim, Fingerprinter, Token};
//!
//! // `fn add(a: u8) -> u8 { a + 1 }`, body only.
//! let body = |extra_space: bool| {
//!     let mut fp = Fingerprinter::body();
//!     fp.open(Delim::Brace);
//!     fp.ident("a");
//!     if extra_space {
//!         // Whitespace is not a token: nothing to feed.
//!     }
//!     fp.punct('+');
//!     fp.literal("1");
//!     fp.close(Delim::Brace);
//!     fp.finish()
//! };
//! assert_eq!(body(false), body(true));
//!
//! let mut other = Fingerprinter::body();
//! for t in [Token::Open(Delim::Brace), Token::Ident("a"), Token::Punct('-'), Token::Literal("1"), Token::Close(Delim::Brace)] {
//!     other.token(t);
//! }
//! assert_ne!(other.finish(), body(false));
//! ```

use sealmap_model::Fingerprint;

/// The fingerprint algorithm id, recorded wherever fingerprints are stored
/// (`algorithm = "sm1"` in a seal lock).
pub const ALGORITHM: &str = "sm1";

/// The delimiter of a token group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Delim {
    /// `( … )`.
    Paren,
    /// `[ … ]`.
    Bracket,
    /// `{ … }`.
    Brace,
    /// An invisible group (macro expansion artefacts); hashed transparently.
    None,
}

impl Delim {
    fn byte(self) -> Option<u8> {
        match self {
            Self::Paren => Some(b'('),
            Self::Bracket => Some(b'['),
            Self::Brace => Some(b'{'),
            Self::None => None,
        }
    }
}

/// One token of the language-neutral stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token<'a> {
    /// An identifier or keyword, exactly as written (raw-identifier prefixes
    /// such as Rust's `r#` are the language adapter's to strip or keep).
    Ident(&'a str),
    /// One punctuation character.
    Punct(char),
    /// A literal, as written in source (`"a\n"`, `1_000u32`, `b'x'`).
    Literal(&'a str),
    /// The start of a delimited group.
    Open(Delim),
    /// The end of a delimited group.
    Close(Delim),
}

/// Which of the two per-symbol hashes a [`Fingerprinter`] computes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The contract: `sig_hash`.
    Sig,
    /// The implementation: `body_hash`.
    Body,
}

impl Kind {
    fn context(self) -> &'static str {
        match self {
            Self::Sig => "sealmap sm1 2026-10 symbol signature fingerprint",
            Self::Body => "sealmap sm1 2026-10 symbol body fingerprint",
        }
    }
}

/// A streaming BLAKE3 fingerprint over a normalised token stream.
///
/// Feed tokens with [`token`](Self::token) (or the shorthands), mark the
/// boundaries between parts of a declaration with
/// [`section`](Self::section), fold in fingerprints of nested symbols with
/// [`fingerprint`](Self::fingerprint), then [`finish`](Self::finish).
#[derive(Debug, Clone)]
pub struct Fingerprinter {
    hasher: blake3::Hasher,
}

/// Record tags. Each record is `tag, u32 little-endian length, payload`
/// (opens and closes carry their delimiter byte as payload), so the byte
/// stream decodes back to exactly one token sequence.
mod tag {
    pub const IDENT: u8 = b'i';
    pub const PUNCT: u8 = b'p';
    pub const LITERAL: u8 = b'l';
    pub const OPEN: u8 = b'o';
    pub const CLOSE: u8 = b'c';
    pub const SECTION: u8 = b's';
    pub const FINGERPRINT: u8 = b'f';
}

impl Fingerprinter {
    /// A fingerprinter for `kind`.
    pub fn new(kind: Kind) -> Self {
        Self { hasher: blake3::Hasher::new_derive_key(kind.context()) }
    }

    /// Shorthand for `new(Kind::Sig)`.
    pub fn sig() -> Self {
        Self::new(Kind::Sig)
    }

    /// Shorthand for `new(Kind::Body)`.
    pub fn body() -> Self {
        Self::new(Kind::Body)
    }

    fn record(&mut self, tag: u8, payload: &[u8]) {
        let len = u32::try_from(payload.len()).unwrap_or(u32::MAX);
        self.hasher.update(&[tag]);
        self.hasher.update(&len.to_le_bytes());
        self.hasher.update(payload);
    }

    /// Feed one token.
    pub fn token(&mut self, token: Token<'_>) {
        match token {
            Token::Ident(s) => self.record(tag::IDENT, s.as_bytes()),
            Token::Punct(c) => self.record(tag::PUNCT, c.encode_utf8(&mut [0; 4]).as_bytes()),
            Token::Literal(s) => {
                if s.contains('\r') {
                    let normalised = s.replace("\r\n", "\n").replace('\r', "\n");
                    self.record(tag::LITERAL, normalised.as_bytes());
                } else {
                    self.record(tag::LITERAL, s.as_bytes());
                }
            }
            Token::Open(d) => {
                if let Some(b) = d.byte() {
                    self.record(tag::OPEN, &[b]);
                }
            }
            Token::Close(d) => {
                if let Some(b) = d.byte() {
                    self.record(tag::CLOSE, &[b]);
                }
            }
        }
    }

    /// Feed an identifier.
    pub fn ident(&mut self, s: &str) {
        self.token(Token::Ident(s));
    }

    /// Feed a punctuation character.
    pub fn punct(&mut self, c: char) {
        self.token(Token::Punct(c));
    }

    /// Feed a literal.
    pub fn literal(&mut self, s: &str) {
        self.token(Token::Literal(s));
    }

    /// Open a group.
    pub fn open(&mut self, d: Delim) {
        self.token(Token::Open(d));
    }

    /// Close a group.
    pub fn close(&mut self, d: Delim) {
        self.token(Token::Close(d));
    }

    /// Mark the start of a named part (`"attrs"`, `"generics"`, ...), so
    /// tokens cannot drift from one part into the next without changing the
    /// fingerprint.
    pub fn section(&mut self, label: &str) {
        self.record(tag::SECTION, label.as_bytes());
    }

    /// Fold in a nested symbol's fingerprint (how a module's `body_hash`
    /// covers its members without re-reading their tokens).
    pub fn fingerprint(&mut self, f: Fingerprint) {
        self.record(tag::FINGERPRINT, f.as_bytes());
    }

    /// The fingerprint.
    pub fn finish(&self) -> Fingerprint {
        Fingerprint::from_blake3(&self.hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn of(kind: Kind, tokens: &[Token<'_>]) -> Fingerprint {
        let mut fp = Fingerprinter::new(kind);
        for t in tokens {
            fp.token(*t);
        }
        fp.finish()
    }

    /// Pinned values: any change to the encoding, the contexts or the
    /// truncation fails here. Bump [`ALGORITHM`] when that is intended.
    #[test]
    fn golden_values() {
        assert_eq!(ALGORITHM, "sm1");
        assert_eq!(of(Kind::Sig, &[]).to_string(), "blake3-16:b0dfa318e9fc9bb5b577c6f8be0aa206");
        assert_eq!(of(Kind::Body, &[]).to_string(), "blake3-16:6d9fd803f137d8db3ae28e1ad8e4c67c");
        let add = [
            Token::Open(Delim::Brace),
            Token::Ident("a"),
            Token::Punct('+'),
            Token::Literal("1"),
            Token::Close(Delim::Brace),
        ];
        assert_eq!(of(Kind::Body, &add).to_string(), "blake3-16:a6253eb7723698ef6076dae4fbc0dbbc");
        let mut fp = Fingerprinter::sig();
        fp.section("name");
        fp.ident("add");
        fp.fingerprint(of(Kind::Body, &add));
        assert_eq!(fp.finish().to_string(), "blake3-16:e4cc87578f4afde856d1051c846f075f");
    }

    #[test]
    fn token_boundaries_and_kinds_are_hashed() {
        use Token::*;
        let pairs: [(&[Token<'_>], &[Token<'_>]); 6] = [
            (&[Ident("ab")], &[Ident("a"), Ident("b")]),
            (&[Ident("a")], &[Literal("a")]),
            (&[Punct('-'), Punct('>')], &[Punct('>'), Punct('-')]),
            (&[Open(Delim::Paren), Close(Delim::Paren)], &[Open(Delim::Bracket), Close(Delim::Bracket)]),
            (
                &[Open(Delim::Paren), Ident("a"), Close(Delim::Paren), Ident("b")],
                &[Open(Delim::Paren), Ident("a"), Ident("b"), Close(Delim::Paren)],
            ),
            (&[Literal("\"a b\"")], &[Literal("\"a  b\"")]),
        ];
        for (a, b) in pairs {
            assert_ne!(of(Kind::Body, a), of(Kind::Body, b), "{a:?} vs {b:?}");
        }
        assert_ne!(of(Kind::Sig, &[Ident("a")]), of(Kind::Body, &[Ident("a")]));
    }

    #[test]
    fn normalisation() {
        use Token::*;
        assert_eq!(of(Kind::Body, &[Literal("\"a\r\nb\""),]), of(Kind::Body, &[Literal("\"a\nb\"")]));
        assert_eq!(of(Kind::Body, &[Literal("\"a\rb\""),]), of(Kind::Body, &[Literal("\"a\nb\"")]));
        assert_eq!(of(Kind::Body, &[Open(Delim::None), Ident("x"), Close(Delim::None)]), of(Kind::Body, &[Ident("x")]));
    }

    #[test]
    fn sections_separate_parts() {
        let mut a = Fingerprinter::sig();
        a.section("generics");
        a.ident("T");
        a.section("params");
        let mut b = Fingerprinter::sig();
        b.section("generics");
        b.section("params");
        b.ident("T");
        assert_ne!(a.finish(), b.finish());
    }
}
