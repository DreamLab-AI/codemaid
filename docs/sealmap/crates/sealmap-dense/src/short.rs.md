---
sealmap: 2
source: crates/sealmap-dense/src/short.rs
module: "sym:cargo sealmap_dense . short/"
language: rust
source_hash: blake3:9456a6d150ad378b4834b0a26a55f331e051f7fd2ceb717eb85b945bf4454842
lines: 228
fragments: 3
---
# `sym:cargo sealmap_dense . short/` · crates/sealmap-dense/src/short.rs
> Short names: the compact, per-output handles the dense text uses in place of full `sym:` ids.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_dense__short___tShortNames["ShortNames#lt;'a#gt;"] {
    <<struct>>
    -by_id: BTreeMap#lt;&'a SymbolId, String#gt;
    -by_short: BTreeMap#lt;String, &'a SymbolId#gt;
    ~get(crate) Option#lt;&str#gt;
    ~iter(crate) impl Iterator#lt;Item = #40;&str, &'a SymbolId#41;#gt; + '_
    ~new(crate) Self
    ~resolve(crate) Option#lt;&'a SymbolId#gt;
  }
  class sealmap_dense__short["sealmap_dense::short"] {
    <<module>>
    -const MAX_LEVEL: u8
    ~candidate(crate) String
    -collisions(names: &[String]) Vec#lt;Vec#lt;usize#gt;#gt;
    -render(items: I) String
    -tail(items: &'v [DescriptorView#lt;'a#gt;], blocks: bool) Vec#lt;&'v DescriptorView#lt;'a#gt;#gt;
  }
  class sealmap_model__sym___tDescriptorView["DescriptorView#lt;'a#gt;"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  sealmap_dense__short ..> sealmap_model__sym___tDescriptorView
  sealmap_dense__short ..> sealmap_model__sym___tSymbolId
  sealmap_dense__short___tShortNames ..> sealmap_model__codebase___tCodebase
  sealmap_dense__short___tShortNames o-- sealmap_model__sym___tSymbolId : by_id, by_short
```

## `sym:cargo sealmap_dense . short/ShortNames#new().`
`pub(crate) fn new(cb: &'a Codebase) -> Self` · L35-L76
> Assign a unique short name to every symbol of `cb`.
```mermaid
sequenceDiagram
  participant sealmap_dense__short___tShortNames as ShortNames
  participant sealmap_dense__short as short mod
  opt via map
    sealmap_dense__short___tShortNames->>sealmap_dense__short: candidate(id, 0)
  end
  loop loop
    sealmap_dense__short___tShortNames->>sealmap_dense__short: collisions(&names)
    loop for i in groups.into_iter().flatten()
      loop while level [i]<MAX_LEVEL
        sealmap_dense__short___tShortNames->>sealmap_dense__short: candidate(_, _)
      end
    end
    opt !progressed
      sealmap_dense__short___tShortNames->>sealmap_dense__short: collisions(&names)
    end
  end
```

## `sym:cargo sealmap_dense . short/candidate().`
`pub(crate) fn candidate(id: &SymbolId, level: u8) -> String` · L103-L139
> The short name of `id` at `level` (see the module docs).
```mermaid
sequenceDiagram
  participant sealmap_dense__short as short mod
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__hash___tContentHash as ContentHash
  sealmap_dense__short->>sealmap_model__sym___tSymbolId: view()
  alt IdView::Global(g)
    opt closure
      sealmap_dense__short->>sealmap_dense__short: render(items)
    end
    alt 0
      sealmap_dense__short->>sealmap_dense__short: tail(items, false)
      sealmap_dense__short->>sealmap_dense__short: render(tail())
    else 1
      sealmap_dense__short->>sealmap_dense__short: tail(items, true)
      sealmap_dense__short->>sealmap_dense__short: render(tail())
    end
  else IdView::Path(_) | IdView::Unresolved(_)
    sealmap_dense__short->>sealmap_model__sym___tSymbolId: display_path()
  end
  opt level>= 4
    sealmap_dense__short->>sealmap_model__sym___tSymbolId: as_str()
    sealmap_dense__short->>sealmap_model__hash___tContentHash: ContentHash::of_text(as_str())
    sealmap_dense__short->>sealmap_model__hash___tContentHash: short(_)
  end
```
