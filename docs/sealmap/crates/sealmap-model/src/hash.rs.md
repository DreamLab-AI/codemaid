---
sealmap: 2
source: crates/sealmap-model/src/hash.rs
module: "sym:cargo sealmap_model . hash/"
language: rust
source_hash: blake3:9f6ed72ca28fa074cd46e0c305382f50ce246cc9aaf298bb6538bc18ff044990
lines: 189
fragments: 6
---
# `sym:cargo sealmap_model . hash/` · crates/sealmap-model/src/hash.rs

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__hash___tContentHash["ContentHash"] {
    <<struct>>
    -0: String
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    +of_bytes(bytes: &[u8]) Self
    +of_text(text: &str) Self
    +parse(s: &str) Option#lt;Self#gt;
    +short(&self, n: usize) &str
  }
  class sealmap_model__hash___tFingerprint["Fingerprint"] {
    <<struct>>
    -0: [u8#59; 16]
    +Serialize::serialize(&self, serializer: S) Result#lt;S::Ok, S::Error#gt;
    +Deserialize#lt;'de#gt;::deserialize(deserializer: D) Result#lt;Self, D::Error#gt;
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_bytes(&self) &[u8#59; 16]
    +from_blake3(hash: &blake3::Hash) Self
    +from_bytes(bytes: [u8#59; 16]) Self
    +is_unset(&self) bool
    +merge(self, other: Self) Self
    +parse(s: &str) Option#lt;Self#gt;
  }
  class sealmap_model__hash["sealmap_model::hash"] {
    <<module>>
    ~normalise_newlines(crate) std::borrow::Cow#lt;'_, str#gt;
  }
  class _blake3__Hash["blake3::Hash"] {
    <<external>>
  }
  class _serde__Deserialize["serde::Deserialize"] {
    <<external>>
  }
  class _serde__Serialize["serde::Serialize"] {
    <<external>>
  }
  sealmap_model__hash___tFingerprint ..> _blake3__Hash
  sealmap_model__hash___tFingerprint ..|> _serde__Deserialize
  sealmap_model__hash___tFingerprint ..|> _serde__Serialize
```

## `sym:cargo sealmap_model . hash/ContentHash#of_text().`
`pub fn of_text(text: &str) -> Self` · L24-L28
> Hash `text` after normalising line endings.
```mermaid
sequenceDiagram
  participant sealmap_model__hash___tContentHash as ContentHash
  participant sealmap_model__hash as hash mod
  sealmap_model__hash___tContentHash->>sealmap_model__hash: normalise_newlines(text)
```

## `sym:cargo sealmap_model . hash/ContentHash#of_bytes().`
`pub fn of_bytes(bytes: &[u8]) -> Self` · L30-L33
> Hash raw bytes with no normalisation.
```mermaid
sequenceDiagram
  participant sealmap_model__hash___tContentHash as ContentHash
  participant _blake3 as blake3 ext
  sealmap_model__hash___tContentHash->>_blake3: blake3::hash(bytes)
```

## `sym:cargo sealmap_model . hash/Fingerprint#from_blake3().`
`pub fn from_blake3(hash: &blake3::Hash) -> Self` · L96-L101
> Truncate a full BLAKE3 digest.
```mermaid
sequenceDiagram
  participant sealmap_model__hash___tFingerprint as Fingerprint
  participant _blake3 as blake3 ext
  sealmap_model__hash___tFingerprint->>_blake3: Hash::as_bytes()
```

## `sym:cargo sealmap_model . hash/Fingerprint#merge().`
`pub fn merge(self, other: Self) -> Self` · L126-L144
> Fold two fingerprints of the same symbol into one, independent of their order.
```mermaid
sequenceDiagram
  participant sealmap_model__hash___tFingerprint as Fingerprint
  participant _blake3 as blake3 ext
  sealmap_model__hash___tFingerprint->>_blake3: Hasher::new_derive_key(#quot;sealmap sm1 …)
  sealmap_model__hash___tFingerprint->>_blake3: Hasher::update(&lo.0)
  sealmap_model__hash___tFingerprint->>_blake3: Hasher::update(&hi.0)
  sealmap_model__hash___tFingerprint->>_blake3: Hasher::finalize()
```

## `` sym:cargo sealmap_model . hash/Fingerprint#[`Deserialize<'de>`]deserialize(). ``
`fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>` · L164-L167
```mermaid
sequenceDiagram
  participant sealmap_model__hash___tFingerprint as Fingerprint
  participant _serde as serde ext
  opt via ok_or_else
    sealmap_model__hash___tFingerprint->>_serde: Error::custom(_)
  end
```
