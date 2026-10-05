---
sealmap: 1
source: crates/sealmap-corpus/src/overview.rs
module: sealmap_corpus::overview
language: rust
source_hash: blake3:072cd647f0e3f2fd62c3952facc96a717afd8aa28c23dca0bc9f12f94e789cfd
lines: 259
fragments: 9
---
# `sealmap_corpus::overview` · crates/sealmap-corpus/src/overview.rs
> `_overview.md`: cross-cutting diagrams for the whole codebase.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__overview["sealmap_corpus::overview"] {
    <<module>>
    -const STRUCTURAL: [RelationKind#59; 6]
    -cardinality(ty: &str) #40;Cardinality, bool#41;
    -crate_graph(cb: &Codebase, crates: &BTreeSet#lt;String#gt;, opts: &CorpusOptions) Option#lt;String#gt;
    -crates_of(cb: &Codebase) BTreeSet#lt;String#gt;
    -data_model(cb: &Codebase, krate: &str, opts: &CorpusOptions) Option#lt;String#gt;
    -heaviest(edges: BTreeMap#lt;K, usize#gt;, max: usize) #40;Vec#lt;#40;K, usize#41;#gt;, usize#41;
    -module_graph(cb: &Codebase, krate: &str, opts: &CorpusOptions) Option#lt;String#gt;
    -module_of(cb: &Codebase, id: &SymbolId) Option#lt;SymbolId#gt;
    ~render(crate) String
    -root(id: &SymbolId) &str
    -trait_map(cb: &Codebase, opts: &CorpusOptions) Option#lt;String#gt;
  }
  class sealmap_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_mermaid__er__Cardinality["Cardinality"] {
    <<enum in crates/sealmap-mermaid/src/er.rs>>
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_corpus__overview ..> sealmap_corpus__CorpusOptions
  sealmap_corpus__overview ..> sealmap_mermaid__er__Cardinality
  sealmap_corpus__overview ..> sealmap_model__codebase__Codebase
  sealmap_corpus__overview ..> sealmap_model__symbol__SymbolId
```

## `sealmap_corpus::overview::render`
`pub(crate) fn render(cb: &Codebase, opts: &CorpusOptions) -> String` · L14-L42
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_model__codebase__Codebase as Codebase
  sealmap_corpus__overview->>sealmap_model__codebase__Codebase: stats()
  sealmap_corpus__overview->>sealmap_corpus__overview: crates_of(cb)
  sealmap_corpus__overview->>sealmap_corpus__overview: crate_graph(cb, &crates, opts)
  loop for krate in &crates
    sealmap_corpus__overview->>sealmap_corpus__overview: module_graph(cb, krate, opts)
  end
  loop for krate in &crates
    sealmap_corpus__overview->>sealmap_corpus__overview: data_model(cb, krate, opts)
  end
  sealmap_corpus__overview->>sealmap_corpus__overview: trait_map(cb, opts)
```

## `sealmap_corpus::overview::root`
`fn root(id: &SymbolId) -> &str` · L44-L46
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_model__symbol__SymbolId as SymbolId
  sealmap_corpus__overview->>sealmap_model__symbol__SymbolId: as_str()
```

## `sealmap_corpus::overview::crates_of`
`fn crates_of(cb: &Codebase) -> BTreeSet<String>` · L48-L50
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  opt via map
    sealmap_corpus__overview->>sealmap_corpus__overview: root(&f.module)
  end
```

## `sealmap_corpus::overview::module_of`
`fn module_of(cb: &Codebase, id: &SymbolId) -> Option<SymbolId>` · L52-L56
> Module that physically contains a symbol (its file's module).
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_model__codebase__Codebase as Codebase
  sealmap_corpus__overview->>sealmap_model__codebase__Codebase: symbol(id)?
  sealmap_corpus__overview->>sealmap_model__codebase__Codebase: file(&s.file)
```

## `sealmap_corpus::overview::crate_graph`
`fn crate_graph(cb: &Codebase, crates: &BTreeSet<String>, opts: &CorpusOptions) -> Option<String>` · L77-L128
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_mermaid__flowchart__Flowchart as Flowchart
  participant sealmap_model__path__SourcePath as SourcePath
  participant sealmap_mermaid__escape__Ident as Ident
  loop for r in cb.relations.iter().filter(| r | STRUCTURAL.co…
    sealmap_corpus__overview->>sealmap_corpus__overview: root(&r.from)
    sealmap_corpus__overview->>sealmap_corpus__overview: root(&r.to)
  end
  opt crates.len()<2 && edges.is_empty()
    Note over sealmap_corpus__overview: return None
  end
  sealmap_corpus__overview->>sealmap_corpus__overview: heaviest(edges, opts.max_edges)
  sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: Flowchart::new(LR)
  loop for k in crates
    loop each via find
      sealmap_corpus__overview->>sealmap_corpus__overview: root(&file.module)
    end
    opt via map
      sealmap_corpus__overview->>sealmap_model__path__SourcePath: ~components()
    end
  end
  loop for (group, ks) in &groups
    alt multi
      sealmap_corpus__overview->>sealmap_mermaid__escape__Ident: Ident::new(&_)
      sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: subgraph(new(), group, None, |..|)
      opt via subgraph
        loop for k in ks
          sealmap_corpus__overview->>sealmap_mermaid__escape__Ident: Ident::new(k)
          sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: ~node(new(), k, Stadium)
        end
      end
    else
      loop for k in ks
        sealmap_corpus__overview->>sealmap_mermaid__escape__Ident: Ident::new(k)
        sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: node(new(), k, Stadium)
      end
    end
  end
  loop for e in &externals
    sealmap_corpus__overview->>sealmap_mermaid__escape__Ident: Ident::new(e)
    sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: node(new(), e, Hexagon)
  end
  loop for ((a, b), n) in &edges
    sealmap_corpus__overview->>sealmap_mermaid__escape__Ident: Ident::new(a)
    sealmap_corpus__overview->>sealmap_mermaid__escape__Ident: Ident::new(b)
    sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: edge(&new(), &new(), Solid, Some())
  end
  sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: render()
```

## `sealmap_corpus::overview::module_graph`
`fn module_graph(cb: &Codebase, krate: &str, opts: &CorpusOptions) -> Option<String>` · L130-L162
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_mermaid__flowchart__Flowchart as Flowchart
  participant sealmap_corpus__naming as naming mod
  loop each via filter
    sealmap_corpus__overview->>sealmap_corpus__overview: root(m)
  end
  opt modules.len()<2
    Note over sealmap_corpus__overview: return None
  end
  loop each via filter
    sealmap_corpus__overview->>sealmap_corpus__overview: root(&r.from)
  end
  loop for r in cb.relations.iter().filter(| r | STRUCTURAL.co…
    sealmap_corpus__overview->>sealmap_corpus__overview: module_of(cb, &r.from)
    sealmap_corpus__overview->>sealmap_corpus__overview: module_of(cb, &r.to)
  end
  sealmap_corpus__overview->>sealmap_corpus__overview: heaviest(edges, opts.max_edges)
  sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: Flowchart::new(LR)
  loop for m in &modules
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(m)
    sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: node(ident(), label, Rect)
  end
  loop for ((a, b), n) in &edges
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(a)
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(b)
    sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: edge(&ident(), &ident(), Solid, Some())
  end
  sealmap_corpus__overview->>sealmap_mermaid__flowchart__Flowchart: render()
```

## `sealmap_corpus::overview::data_model`
`fn data_model(cb: &Codebase, krate: &str, opts: &CorpusOptions) -> Option<String>` · L164-L210
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_mermaid__er__ErDiagram as ErDiagram
  participant sealmap_corpus__naming as naming mod
  loop each via filter
    sealmap_corpus__overview->>sealmap_corpus__overview: root(&s.id)
  end
  loop for id in &types
    loop for m in s.members.iter().filter(| m | m.kind == Member…
      loop for t in m.refs.iter().filter(| t | types.contains(* t)…
        sealmap_corpus__overview->>sealmap_corpus__overview: cardinality(ty)
      end
    end
  end
  opt rels.is_empty()
    Note over sealmap_corpus__overview: return None
  end
  sealmap_corpus__overview->>sealmap_mermaid__er__ErDiagram: ErDiagram::new()
  loop for id in &keep
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(id)
    sealmap_corpus__overview->>sealmap_mermaid__er__ErDiagram: entity(clone(), &s.name)
    loop for m in s.members.iter().filter(| m | m.kind == Member…
      sealmap_corpus__overview->>sealmap_mermaid__er__ErDiagram: attr(&e, unwrap_or(), &m.name, None)
    end
  end
  loop for (from, field, to, card, owned) in &rels
    opt keep.contains(from) && keep.contains(to)
      sealmap_corpus__overview->>sealmap_corpus__naming: ident(from)
      sealmap_corpus__overview->>sealmap_corpus__naming: ident(to)
      sealmap_corpus__overview->>sealmap_mermaid__er__ErDiagram: relation(&ident(), One, &ident(), _, _, field)
    end
  end
  sealmap_corpus__overview->>sealmap_mermaid__er__ErDiagram: render()
```

## `sealmap_corpus::overview::trait_map`
`fn trait_map(cb: &Codebase, opts: &CorpusOptions) -> Option<String>` · L228-L259
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_model__codebase__Codebase as Codebase
  participant sealmap_mermaid__class__ClassDiagram as ClassDiagram
  participant sealmap_corpus__naming as naming mod
  participant sealmap_mermaid__class__Class as Class
  sealmap_corpus__overview->>sealmap_model__codebase__Codebase: symbols_of_kind(Trait)
  sealmap_corpus__overview->>sealmap_model__codebase__Codebase: relations_of_kind(Implements)
  sealmap_corpus__overview->>sealmap_model__codebase__Codebase: relations_of_kind(Extends)
  opt impls.is_empty()
    Note over sealmap_corpus__overview: return None
  end
  sealmap_corpus__overview->>sealmap_mermaid__class__ClassDiagram: ClassDiagram::new(LR)
  loop for t in &traits
    opt impls.iter().any(| r | &r.to == * t || &r.from == * …
      sealmap_corpus__overview->>sealmap_corpus__naming: ident(t)
      sealmap_corpus__overview->>sealmap_mermaid__class__Class: Class::new(ident(), &_.name)
      sealmap_corpus__overview->>sealmap_mermaid__class__Class: annotation(#quot;trait#quot;)
      sealmap_corpus__overview->>sealmap_mermaid__class__ClassDiagram: class(c)
    end
  end
  loop for r in &impls
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(&r.from)
    sealmap_corpus__overview->>sealmap_mermaid__class__ClassDiagram: has_class(&ident())
    opt !d.has_class(&ident(&r.from))
      sealmap_corpus__overview->>sealmap_corpus__naming: ident(&r.from)
      sealmap_corpus__overview->>sealmap_mermaid__class__Class: Class::new(ident(), name())
      sealmap_corpus__overview->>sealmap_mermaid__class__ClassDiagram: class(new())
    end
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(&r.from)
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(&r.to)
    sealmap_corpus__overview->>sealmap_mermaid__class__ClassDiagram: relation(&ident(), &ident(), kind, None)
  end
  sealmap_corpus__overview->>sealmap_mermaid__class__ClassDiagram: render()
```
