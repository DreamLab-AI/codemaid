---
sealmap: 2
source: crates/sealmap-dense/src/skeleton.rs
module: "sym:cargo sealmap_dense . skeleton/"
language: rust
source_hash: blake3:215be3f2b78006b0606150cd0bf0ad03bc3def8aab78e947747c666742fce0d6
lines: 201
fragments: 5
---
# `sym:cargo sealmap_dense . skeleton/` · crates/sealmap-dense/src/skeleton.rs
> Skeleton lines: one Rust-like line per symbol, grouped by file in source order.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_dense__skeleton["sealmap_dense::skeleton"] {
    <<module>>
    ~line(crate)
    -member(out: &mut String, m: &Member)
    ~one_line(crate) String
    ~order_key(crate) #40;&SourcePath, u32, u32, &SymbolId#41;
    ~span(crate)
    -strip_vis(sig: &str) &str
    -vis(out: &mut String, v: &Visibility)
    ~write_files(crate)
  }
  class sealmap_dense__short___tShortNames["ShortNames#lt;'a#gt;"] {
    <<struct in crates/sealmap-dense/src/short.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__path___tSourcePath["SourcePath"] {
    <<struct in crates/sealmap-model/src/path.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__symbol___tMember["Member"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol___tSymbol["Symbol"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol___tVisibility["Visibility"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_dense__skeleton ..> sealmap_dense__short___tShortNames
  sealmap_dense__skeleton ..> sealmap_model__codebase___tCodebase
  sealmap_dense__skeleton ..> sealmap_model__path___tSourcePath
  sealmap_dense__skeleton ..> sealmap_model__sym___tSymbolId
  sealmap_dense__skeleton ..> sealmap_model__symbol___tMember
  sealmap_dense__skeleton ..> sealmap_model__symbol___tSymbol
  sealmap_dense__skeleton ..> sealmap_model__symbol___tVisibility
```

## `sym:cargo sealmap_dense . skeleton/write_files().`
`pub(crate) fn write_files<'a>(out: &mut String, cb: &Codebase, shorts: &ShortNames<'_>, symbols: impl IntoIterator<Item = &'a Symbol>,)` · L16-L37
> Write `symbols` as `## <path>` sections, each listing its symbols in source order.
```mermaid
sequenceDiagram
  participant sealmap_dense__skeleton as skeleton mod
  loop for (path, mut syms) in by_file
    loop each via sort_by
      sealmap_dense__skeleton->>sealmap_dense__skeleton: order_key(a)
      sealmap_dense__skeleton->>sealmap_dense__skeleton: order_key(b)
    end
    sealmap_dense__skeleton->>sealmap_dense__skeleton: one_line(as_str())
    loop for sym in syms
      sealmap_dense__skeleton->>sealmap_dense__skeleton: line(out, cb, shorts, sym)
    end
  end
```

## `sym:cargo sealmap_dense . skeleton/line().`
`pub(crate) fn line(out: &mut String, cb: &Codebase, shorts: &ShortNames<'_>, sym: &Symbol)` · L39-L82
> `<vis> <signature or kind name><members><impls> L<a>-<b> @<short>`.
```mermaid
sequenceDiagram
  participant sealmap_dense__skeleton as skeleton mod
  participant sealmap_model__symbol___tSymbolKind as SymbolKind
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_dense__short___tShortNames as ShortNames
  sealmap_dense__skeleton->>sealmap_dense__skeleton: vis(out, &sym.visibility)
  alt Some(sig)
    sealmap_dense__skeleton->>sealmap_dense__skeleton: strip_vis(sig)
    sealmap_dense__skeleton->>sealmap_dense__skeleton: one_line(strip_vis())
  else None
    opt kind
      sealmap_dense__skeleton->>sealmap_model__symbol___tSymbolKind: ~keyword()
    end
    opt !sym.generics.is_empty()
      sealmap_dense__skeleton->>sealmap_dense__skeleton: one_line(&join())
    end
  end
  opt !sym.members.is_empty()
    loop for (i, m) in sym.members.iter().enumerate()
      sealmap_dense__skeleton->>sealmap_dense__skeleton: member(out, m)
    end
  end
  sealmap_dense__skeleton->>sealmap_model__codebase___tCodebase: relations_from(&sym.id)
  opt via map
    sealmap_dense__skeleton->>sealmap_dense__short___tShortNames: get(&r.to)
  end
  sealmap_dense__skeleton->>sealmap_dense__skeleton: span(out, sym)
  sealmap_dense__skeleton->>sealmap_dense__short___tShortNames: get(&sym.id)
```

## `sym:cargo sealmap_dense . skeleton/member().`
`fn member(out: &mut String, m: &Member)` · L92-L133
```mermaid
sequenceDiagram
  participant sealmap_dense__skeleton as skeleton mod
  opt MemberKind::RequiredMethod
    opt Some(sig)
      sealmap_dense__skeleton->>sealmap_dense__skeleton: strip_vis(&sig)
    end
  end
```

## `sym:cargo sealmap_dense . skeleton/vis().`
`fn vis(out: &mut String, v: &Visibility)` · L135-L149
> Write the Rust spelling of a visibility, with its trailing space.
```mermaid
sequenceDiagram
  participant sealmap_dense__skeleton as skeleton mod
  opt Visibility::Restricted(p)
    sealmap_dense__skeleton->>sealmap_dense__skeleton: one_line(p)
  end
```
