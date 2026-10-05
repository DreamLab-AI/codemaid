---
sealmap: 2
source: crates/sealmap-dense/src/lib.rs
module: "sym:cargo sealmap_dense ."
language: rust
source_hash: blake3:1159782250506d42342b0f1341bb2f98ac19eac74410e2f03feadcc230475466
lines: 516
fragments: 14
---
# `sym:cargo sealmap_dense .` · crates/sealmap-dense/src/lib.rs
> The compact **agent projection** of a [`sealmap_model::Codebase`]: what an LLM agent reads instead of the source, or instead of a Mermaid corpus, when it needs…

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_dense___tDense["Dense#lt;'a#gt;"] {
    <<struct>>
    -cb: &'a Codebase
    -shorts: ShortNames#lt;'a#gt;
    -graph: Graph#lt;'a#gt;
    -header(&self, kind: &str) String
    -in_source_order(&self, ids: impl Iterator#lt;Item = &'a SymbolId#gt;) Vec#lt;&'a SymbolId#gt;
    +index(&self) String
    -index_line(&self, out: &mut String, short: &str, id: &SymbolId)
    +new(cb: &'a Codebase) Self
    -reach(&self, seeds: &BTreeSet#lt;&'a SymbolId#gt;, depth: usize, next: impl Fn#40;&SymbolId#41; -> Option#lt;Vec#lt;&'a SymbolId#gt;#gt;,) BTreeSet#lt;&'a SymbolId#gt;
    +render(&self, options: &DenseOptions) DenseOutput
    +resolve(&self, short: &str) Option#lt;&'a SymbolId#gt;
    +short(&self, id: &SymbolId) Option#lt;&str#gt;
    +slice(&self, seeds: I, options: &SliceOptions) Result#lt;String, SliceError#gt;
    -slice_text(&self, seeds: &BTreeSet#lt;&'a SymbolId#gt;, depth: usize) String
    +text(&self, options: &DenseOptions) String
  }
  class sealmap_dense___tDenseOptions["DenseOptions"] {
    <<struct>>
    +max_depth: usize
    +Default::default() Self
    +with_max_depth(max_depth: usize) Self
  }
  class sealmap_dense___tDenseOutput["DenseOutput"] {
    <<struct>>
    +text: String
    +index: String
    +files(&self) [#40;&'static str, &str#41;#59; 2]
  }
  class sealmap_dense___tReadmeDoctests["ReadmeDoctests"] {
    <<struct>>
  }
  class sealmap_dense___tSliceError["SliceError"] {
    <<enum>>
    UnknownSymbols#40;Vec#lt;SymbolId#gt;#41;
    OverBudget#123; #35;[doc = #quot; Size of the whole slice in bytes.#quot;] bytes: …
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
  }
  class sealmap_dense___tSliceOptions["SliceOptions"] {
    <<struct>>
    +depth: usize
    +max_bytes: Option#lt;usize#gt;
    +Default::default() Self
    +max_bytes(mut self, max_bytes: usize) Self
    +new(depth: usize) Self
  }
  class sealmap_dense {
    <<module>>
    +const FORMAT_VERSION: u32
    +const INDEX_FILE: &str
    -const LEGEND: &str
    +const TEXT_FILE: &str
    -encode_path(out: &mut String, path: &str)
    +render(codebase: &Codebase, options: &DenseOptions) DenseOutput
    +mod short
    +mod skeleton
    +slice(codebase: &Codebase, seeds: I, options: &SliceOptions) Result#lt;String, SliceError#gt;
    +mod tree
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_dense__short___tShortNames["ShortNames#lt;'a#gt;"] {
    <<struct in crates/sealmap-dense/src/short.rs>>
  }
  class sealmap_dense__tree___tGraph["Graph#lt;'a#gt;"] {
    <<struct in crates/sealmap-dense/src/tree.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  sealmap_dense ..> sealmap_dense___tDenseOptions
  sealmap_dense ..> sealmap_dense___tDenseOutput
  sealmap_dense ..> sealmap_dense___tSliceError
  sealmap_dense ..> sealmap_dense___tSliceOptions
  sealmap_dense ..> sealmap_model__codebase___tCodebase
  sealmap_dense___tDense ..> sealmap_dense___tDenseOptions
  sealmap_dense___tDense ..> sealmap_dense___tDenseOutput
  sealmap_dense___tDense ..> sealmap_dense___tSliceError
  sealmap_dense___tDense ..> sealmap_dense___tSliceOptions
  sealmap_dense___tDense *-- sealmap_dense__short___tShortNames : shorts
  sealmap_dense___tDense *-- sealmap_dense__tree___tGraph : graph
  sealmap_dense___tDense o-- sealmap_model__codebase___tCodebase : cb
  sealmap_dense___tDense ..> sealmap_model__sym___tSymbolId
  sealmap_dense___tSliceError o-- sealmap_model__sym___tSymbolId : UnknownSymbols, OverBudget
```

## `sym:cargo sealmap_dense . Dense#new().`
`pub fn new(cb: &'a Codebase) -> Self` · L286-L289
> Index `cb`: assign short names and read the call graph from the flows.
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  participant sealmap_dense__short___tShortNames as ShortNames
  participant sealmap_dense__tree___tGraph as Graph
  sealmap_dense___tDense->>sealmap_dense__short___tShortNames: ShortNames::new(cb)
  sealmap_dense___tDense->>sealmap_dense__tree___tGraph: Graph::new(cb)
```

## `sym:cargo sealmap_dense . Dense#short().`
`pub fn short(&self, id: &SymbolId) -> Option<&str>` · L291-L294
> The short name of a symbol, or `None` if `id` is not in the codebase.
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  participant sealmap_dense__short___tShortNames as ShortNames
  sealmap_dense___tDense->>sealmap_dense__short___tShortNames: get(id)
```

## `sym:cargo sealmap_dense . Dense#resolve().`
`pub fn resolve(&self, short: &str) -> Option<&'a SymbolId>` · L296-L299
> The symbol a short name stands for.
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  participant sealmap_dense__short___tShortNames as ShortNames
  sealmap_dense___tDense->>sealmap_dense__short___tShortNames: resolve(short)
```

## `sym:cargo sealmap_dense . Dense#render().`
`pub fn render(&self, options: &DenseOptions) -> DenseOutput` · L301-L304
> The full projection: [`text`](Self::text) and [`index`](Self::index).
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  sealmap_dense___tDense->>sealmap_dense___tDense: text(options)
  sealmap_dense___tDense->>sealmap_dense___tDense: index()
```

## `sym:cargo sealmap_dense . Dense#index().`
`pub fn index(&self) -> String` · L306-L313
> The `_index.txt` text: every symbol, ordered by short name.
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  participant sealmap_dense__short___tShortNames as ShortNames
  sealmap_dense___tDense->>sealmap_dense__short___tShortNames: iter()
  loop for (short, id) in self.shorts.iter()
    sealmap_dense___tDense->>sealmap_dense___tDense: index_line(&out, short, id)
  end
```

## `sym:cargo sealmap_dense . Dense#text().`
`pub fn text(&self, options: &DenseOptions) -> String` · L315-L336
> Skeleton lines for every symbol, then the call trees.
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  participant sealmap_dense__skeleton as skeleton mod
  participant sealmap_dense__tree___tTreeWriter as TreeWriter
  sealmap_dense___tDense->>sealmap_dense___tDense: header(#quot;#quot;)
  sealmap_dense___tDense->>sealmap_dense__skeleton: skeleton::write_files(&out, self.cb, &self.shorts, valu…
  sealmap_dense___tDense->>sealmap_dense__tree___tTreeWriter: TreeWriter::new(self.cb, &self.shorts, &self.graph, max…
  sealmap_dense___tDense->>sealmap_dense___tDense: in_source_order(copied())
  loop for &id in &callables
    opt !self.graph.callers.contains_key(id)
      sealmap_dense___tDense->>sealmap_dense__tree___tTreeWriter: root(id, #quot;#quot;)
    end
  end
  loop for &id in &callables
    sealmap_dense___tDense->>sealmap_dense__tree___tTreeWriter: is_expanded(id)
    opt !writer.is_expanded(id)
      sealmap_dense___tDense->>sealmap_dense__tree___tTreeWriter: root(id, #quot; ↺#quot;)
    end
  end
```

## `sym:cargo sealmap_dense . Dense#slice().`
`pub fn slice<'s, I>(&self, seeds: I, options: &SliceOptions) -> Result<String, SliceError> where I: IntoIterator<Item = &'s SymbolId>,` · L338-L377
> The dense text for `seeds` plus their callees and callers to `options.depth` call levels: skeleton lines, call trees from each seed, caller trees up to each se…
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  opt !unknown.is_empty()
    Note over sealmap_dense___tDense: return Err(SliceError::UnknownSymbols(unknown.into_iter…
  end
  sealmap_dense___tDense->>sealmap_dense___tDense: slice_text(&known, options.depth)
  opt Some(budget) if text.len()> budget
    opt via map
      sealmap_dense___tDense->>sealmap_dense___tDense: slice_text(&from(), options.depth)
    end
  end
```

## `sym:cargo sealmap_dense . Dense#slice_text().`
`fn slice_text(&self, seeds: &BTreeSet<&'a SymbolId>, depth: usize) -> String` · L379-L414
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_dense__skeleton as skeleton mod
  participant sealmap_dense__tree___tTreeWriter as TreeWriter
  participant sealmap_dense__tree___tCallerWriter as CallerWriter
  participant sealmap_dense__short___tShortNames as ShortNames
  sealmap_dense___tDense->>sealmap_dense___tDense: reach(seeds, depth, |..|)
  sealmap_dense___tDense->>sealmap_dense___tDense: reach(seeds, depth, |..|)
  sealmap_dense___tDense->>sealmap_dense___tDense: header(&_)
  loop each via filter_map
    sealmap_dense___tDense->>sealmap_model__codebase___tCodebase: symbol(id)
  end
  sealmap_dense___tDense->>sealmap_dense__skeleton: skeleton::write_files(&out, self.cb, &self.shorts, filt…
  sealmap_dense___tDense->>sealmap_dense___tDense: in_source_order(copied())
  opt depth> 0
    sealmap_dense___tDense->>sealmap_dense__tree___tTreeWriter: TreeWriter::new(self.cb, &self.shorts, &self.graph, dep…
    loop for &id in &ordered
      sealmap_dense___tDense->>sealmap_dense__tree___tTreeWriter: root(id, #quot;#quot;)
    end
    sealmap_dense___tDense->>sealmap_dense__tree___tCallerWriter: CallerWriter::new(self.cb, &self.shorts, &self.graph, d…
    loop for &id in &ordered
      sealmap_dense___tDense->>sealmap_dense__tree___tCallerWriter: root(id)
    end
  end
  loop each via filter_map
    sealmap_dense___tDense->>sealmap_dense__short___tShortNames: get(id)
  end
  loop for (short, id) in named
    sealmap_dense___tDense->>sealmap_dense___tDense: index_line(&out, short, id)
  end
```

## `sym:cargo sealmap_dense . Dense#in_source_order().`
`fn in_source_order(&self, ids: impl Iterator<Item = &'a SymbolId>) -> Vec<&'a SymbolId>` · L441-L448
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_dense__skeleton as skeleton mod
  loop each via sort_by
    sealmap_dense___tDense->>sealmap_model__codebase___tCodebase: symbol(a)
    sealmap_dense___tDense->>sealmap_model__codebase___tCodebase: symbol(b)
    opt (Some(x), Some(y))
      sealmap_dense___tDense->>sealmap_dense__skeleton: order_key(x)
      sealmap_dense___tDense->>sealmap_dense__skeleton: order_key(y)
    end
  end
```

## `sym:cargo sealmap_dense . Dense#header().`
`fn header(&self, kind: &str) -> String` · L450-L452
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  participant sealmap_dense__skeleton as skeleton mod
  sealmap_dense___tDense->>sealmap_dense__skeleton: one_line(&self.cb.name)
```

## `sym:cargo sealmap_dense . Dense#index_line().`
`fn index_line(&self, out: &mut String, short: &str, id: &SymbolId)` · L454-L465
```mermaid
sequenceDiagram
  participant sealmap_dense___tDense as Dense
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_dense as sealmap_dense mod
  participant sealmap_dense__skeleton as skeleton mod
  sealmap_dense___tDense->>sealmap_model__sym___tSymbolId: as_str()
  sealmap_dense___tDense->>sealmap_model__codebase___tCodebase: symbol(id)
  opt let Some(sym) = self.cb.symbol(id)
    sealmap_dense___tDense->>sealmap_dense: encode_path(out, as_str())
    sealmap_dense___tDense->>sealmap_dense__skeleton: skeleton::span(out, sym)
  end
```

## `sym:cargo sealmap_dense . render().`
`pub fn render(codebase: &Codebase, options: &DenseOptions) -> DenseOutput` · L468-L471
> The full projection of `codebase` in one call; see [`Dense::render`].
```mermaid
sequenceDiagram
  participant sealmap_dense as sealmap_dense mod
  participant sealmap_dense___tDense as Dense
  sealmap_dense->>sealmap_dense___tDense: Dense::new(codebase)
```

## `sym:cargo sealmap_dense . slice().`
`pub fn slice<'s, I>(codebase: &Codebase, seeds: I, options: &SliceOptions) -> Result<String, SliceError> where I: IntoIterator<Item = &'s SymbolId>,` · L473-L484
> One slice of `codebase` in one call; see [`Dense::slice`].
```mermaid
sequenceDiagram
  participant sealmap_dense as sealmap_dense mod
  participant sealmap_dense___tDense as Dense
  sealmap_dense->>sealmap_dense___tDense: Dense::new(codebase)
```
