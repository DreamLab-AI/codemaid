# sealmap-frontend

[![crates.io](https://img.shields.io/crates/v/sealmap-frontend.svg)](https://crates.io/crates/sealmap-frontend)
[![docs.rs](https://img.shields.io/docsrs/sealmap-frontend)](https://docs.rs/sealmap-frontend)

The language-neutral half of a [sealmap](https://github.com/DreamLab-AI/sealmap)
frontend. A frontend parses source and walks function bodies into a raw flow;
everything after that is shared here, so every language gets flows, labels,
ids and confidences by the same rules:

| Module | Owns |
|---|---|
| `raw` | the raw flow IR (`RawStep`, `RawCall`, `Callee`, `Recv`) and walk-time shapers |
| `lower` | flow normalisation: raw steps plus a call resolver → a model `Flow` |
| `confidence` | the confidence lattice, the external-call policy, call-edge aggregation |
| `labels` | label rules: token spacing, clipping limits, call/condition/closure labels |
| `fingerprint` | per-symbol `sig_hash` / `body_hash`: BLAKE3 over a language-neutral, comment- and whitespace-free token stream |
| `ids` | the `sym:` symbol-id builder |
| `isolate` | per-file collection on big stacks with a panic guard (feature `parallel`: rayon) |

It has no parser dependency; a frontend brings its own.

```rust
use sealmap_frontend::raw::{Callee, RawCall, RawStep};
use sealmap_frontend::{ids, lower};
use sealmap_model::{CallKind, Confidence, SymbolKind};

let call = |name: &str| {
    RawStep::Call(RawCall::new(Callee::Path(vec![name.into()]), format!("{name}()"), CallKind::Function, 1))
};
let raw = vec![call("load"), RawStep::Loop("for x in xs".into(), vec![call("save")])];

let app = ids::module_id(&ids::package("cargo", "app"), &[]);
let flow = lower::lower_flow(&raw, &mut |c: &RawCall| {
    let Callee::Path(segs) = &c.callee else { return None };
    Some(c.to_call(ids::item_id(&app, SymbolKind::Function, &segs[0]), Confidence::Exact))
})
.unwrap();
let targets: Vec<_> = flow.calls().map(|c| c.target.to_string()).collect();
assert_eq!(targets, ["sym:cargo app . load().", "sym:cargo app . save()."]);
```

## Licence

Licensed under either of [Apache License, Version 2.0](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-APACHE) or [MIT licence](https://github.com/DreamLab-AI/sealmap/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
