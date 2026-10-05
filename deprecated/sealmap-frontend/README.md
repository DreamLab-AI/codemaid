# sealmap-frontend (deprecated)

This crate was renamed to [`sealmap-extract`](https://crates.io/crates/sealmap-extract)
in sealmap 0.2.0. Version 0.1.1 only re-exports `sealmap-extract` so existing
builds keep working; it will receive no further changes.

Replace the dependency:

```toml
[dependencies]
sealmap-extract = "0.2"
```

and the import paths (`sealmap_frontend::` becomes `sealmap_extract::`).

Licensed under either of MIT or Apache-2.0, at your option.
Source: <https://github.com/DreamLab-AI/sealmap>.
