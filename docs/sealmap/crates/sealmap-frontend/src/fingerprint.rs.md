---
sealmap: 2
source: crates/sealmap-frontend/src/fingerprint.rs
module: "sym:cargo sealmap_frontend . fingerprint/"
language: rust
source_hash: blake3:8b34cb26cb6b392535cebc4b0fdfa43656a4d43bde5942d2c6ebaa9cafdadf2c
lines: 328
fragments: 12
---
# `sym:cargo sealmap_frontend . fingerprint/` · crates/sealmap-frontend/src/fingerprint.rs
> Per-symbol fingerprints: `sig_hash` (the contract) and `body_hash` (the implementation).

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_frontend__fingerprint___tDelim["Delim"] {
    <<enum>>
    Paren
    Bracket
    Brace
    None
    -byte(self) Option#lt;u8#gt;
  }
  class sealmap_frontend__fingerprint___tFingerprinter["Fingerprinter"] {
    <<struct>>
    -hasher: blake3::Hasher
    +body() Self
    +close(&mut self, d: Delim)
    +fingerprint(&mut self, f: Fingerprint)
    +finish(&self) Fingerprint
    +ident(&mut self, s: &str)
    +literal(&mut self, s: &str)
    +new(kind: Kind) Self
    +open(&mut self, d: Delim)
    +punct(&mut self, c: char)
    -record(&mut self, tag: u8, payload: &[u8])
    +section(&mut self, label: &str)
    +sig() Self
    +token(&mut self, token: Token#lt;'_#gt;)
  }
  class sealmap_frontend__fingerprint___tKind["Kind"] {
    <<enum>>
    Sig
    Body
    -context(self) &'static str
  }
  class sealmap_frontend__fingerprint___tToken["Token#lt;'a#gt;"] {
    <<enum>>
    Ident#40;&'a str#41;
    Punct#40;char#41;
    Literal#40;&'a str#41;
    Open#40;Delim#41;
    Close#40;Delim#41;
  }
  class sealmap_frontend__fingerprint["sealmap_frontend::fingerprint"] {
    <<module>>
    +const ALGORITHM: &str
    -mod tag
  }
  class sealmap_frontend__fingerprint__tag["sealmap_frontend::fingerprint::tag"] {
    <<module>>
    +const CLOSE: u8
    +const FINGERPRINT: u8
    +const IDENT: u8
    +const LITERAL: u8
    +const OPEN: u8
    +const PUNCT: u8
    +const SECTION: u8
  }
  class sealmap_model__hash___tFingerprint["Fingerprint"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class _blake3__Hasher["blake3::Hasher"] {
    <<external>>
  }
  sealmap_frontend__fingerprint___tFingerprinter ..> sealmap_frontend__fingerprint___tDelim
  sealmap_frontend__fingerprint___tFingerprinter ..> sealmap_frontend__fingerprint___tKind
  sealmap_frontend__fingerprint___tFingerprinter ..> sealmap_frontend__fingerprint___tToken
  sealmap_frontend__fingerprint___tFingerprinter ..> sealmap_model__hash___tFingerprint
  sealmap_frontend__fingerprint___tFingerprinter *-- _blake3__Hasher : hasher
  sealmap_frontend__fingerprint___tToken *-- sealmap_frontend__fingerprint___tDelim : Open, Close
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#new().`
`pub fn new(kind: Kind) -> Self` · L162-L165
> A fingerprinter for `kind`.
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  participant sealmap_frontend__fingerprint___tKind as Kind
  participant _blake3 as blake3 ext
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tKind: context()
  sealmap_frontend__fingerprint___tFingerprinter->>_blake3: Hasher::Hasher::new_derive_key(context())
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#record().`
`fn record(&mut self, tag: u8, payload: &[u8])` · L177-L182
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  participant _blake3 as blake3 ext
  sealmap_frontend__fingerprint___tFingerprinter->>_blake3: Hasher::update(&_)
  sealmap_frontend__fingerprint___tFingerprinter->>_blake3: Hasher::update(&to_le_bytes())
  sealmap_frontend__fingerprint___tFingerprinter->>_blake3: Hasher::update(payload)
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#token().`
`pub fn token(&mut self, token: Token<'_>)` · L184-L208
> Feed one token.
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  participant sealmap_frontend__fingerprint___tDelim as Delim
  alt Token::Ident(s)
    sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: record(IDENT, as_bytes())
  else Token::Punct(c)
    sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: record(PUNCT, as_bytes())
  else Token::Literal(s)
    alt s.contains('\r')
      sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: record(LITERAL, as_bytes())
    else
      sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: record(LITERAL, as_bytes())
    end
  else Token::Open(d)
    sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tDelim: ~byte()
    opt let Some(b) = d.byte()
      sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: record(OPEN, &_)
    end
  else Token::Close(d)
    sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tDelim: ~byte()
    opt let Some(b) = d.byte()
      sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: record(CLOSE, &_)
    end
  end
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#ident().`
`pub fn ident(&mut self, s: &str)` · L210-L213
> Feed an identifier.
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: token(Ident())
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#punct().`
`pub fn punct(&mut self, c: char)` · L215-L218
> Feed a punctuation character.
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: token(Punct())
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#literal().`
`pub fn literal(&mut self, s: &str)` · L220-L223
> Feed a literal.
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: token(Literal())
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#open().`
`pub fn open(&mut self, d: Delim)` · L225-L228
> Open a group.
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: token(Open())
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#close().`
`pub fn close(&mut self, d: Delim)` · L230-L233
> Close a group.
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: token(Close())
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#section().`
`pub fn section(&mut self, label: &str)` · L235-L240
> Mark the start of a named part (`"attrs"`, `"generics"`, ...), so tokens cannot drift from one part into the next without changing the fingerprint.
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: record(SECTION, as_bytes())
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#fingerprint().`
`pub fn fingerprint(&mut self, f: Fingerprint)` · L242-L246
> Fold in a nested symbol's fingerprint (how a module's `body_hash` covers its members without re-reading their tokens).
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  participant sealmap_model__hash___tFingerprint as Fingerprint
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_model__hash___tFingerprint: as_bytes()
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_frontend__fingerprint___tFingerprinter: record(FINGERPRINT, as_bytes())
```

## `sym:cargo sealmap_frontend . fingerprint/Fingerprinter#finish().`
`pub fn finish(&self) -> Fingerprint` · L248-L251
> The fingerprint.
```mermaid
sequenceDiagram
  participant sealmap_frontend__fingerprint___tFingerprinter as Fingerprinter
  participant _blake3 as blake3 ext
  participant sealmap_model__hash___tFingerprint as Fingerprint
  sealmap_frontend__fingerprint___tFingerprinter->>_blake3: Hasher::finalize()
  sealmap_frontend__fingerprint___tFingerprinter->>sealmap_model__hash___tFingerprint: Fingerprint::from_blake3(&finalize())
```
