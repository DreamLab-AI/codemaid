# Changelog

All crates in this workspace share one version.

## Unreleased

### Fixed
- `write` (and so `sealmap generate`) followed symbolic links in the output
  directory and could write outside it. It now refuses, before changing
  anything, any path that runs through a link below the output directory.
- A binding made by an `if let` or `while let` condition stayed in scope after
  the statement (and in its `else` branches), so a later call on a shadowed
  outer name was drawn as an Exact call on the condition's type.

## 0.2.0 — 2026-10-06

### Added
- `sym:` symbol ids (SCIP-descriptor style): readable, never file-path based,
  injective; canonical text round-trips exactly.
- Per-symbol `sig_hash` and `body_hash` (BLAKE3-16, algorithm `sm1`), blind to
  formatting, comments and moves; schema v2 carries both on every symbol.
- The seal surface in `sealmap-corpus` and the CLI: `seals.lock`, `topic_hash`,
  `resolve`, `seal-check`, `stale [--since REV]`, `verify` and `seal sign`.
- `sealmap-dense`: skeletons with `L<start>-<end>` spans, indented call trees, a
  short-name index and byte-budgeted slices that refuse rather than truncate.
  Measured at 0.17-0.29× source size (0.32-0.46× with the index).
- `sealmap pack`: a deterministic review pack from pegged topics, by id or by
  `--diff REV`, with dense slices and bounded source windows; over budget it
  refuses and names each topic's size.
- `generate --check`: compare a directory with a fresh generation.

### Changed
- Renamed from `codemaid-*`; licence is now `MIT OR Apache-2.0`.
- `sealmap-frontend` is renamed `sealmap-extract`; `sealmap-frontend` 0.1.1 is
  a deprecated shim re-exporting it.
- Generated output goes to a gitignored `.sealmap/` and is no longer committed.
- Mermaid diagram ids are derived from `sym:` ids and cannot collide.

### Fixed
- Stack overflow on deep generic types; `.gitignore` ignored by the loader;
  exponential glob-import resolution.
- Std and third-party receivers bound to same-named internal methods
  (`Path::parent()` drawn as `SymbolId::parent`).
- Single-capital type names treated as generic parameters; declared
  multi-letter generics not treated as generics.
- `Self::f(..)` calls dropped; calls in match guards not walked; method calls
  on struct literals lost; guessed edges unstable across unrelated crates.

### Minimum Rust version
- 1.85, checked in CI. `Cargo.lock` pins `ignore` 0.4.29: 0.4.30 needs Rust
  1.88 without declaring it. Outside this workspace on 1.85, run
  `cargo update -p ignore --precise 0.4.29`.

## 0.1.0 — 2026-10-05

First release, as `sealmap-*` (formerly `codemaid`).
