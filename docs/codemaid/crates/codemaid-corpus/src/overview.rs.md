---
codemaid: 1
source: crates/codemaid-corpus/src/overview.rs
module: codemaid_corpus::overview
language: rust
source_hash: blake3:de41115146635850a075900bfba56e061b809eb9e8c3a1e16ac05c8cba87a9bd
lines: 260
fragments: 9
---
# `codemaid_corpus::overview` · crates/codemaid-corpus/src/overview.rs
> `_overview.md`: cross-cutting diagrams for the whole codebase.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_corpus__overview["codemaid_corpus::overview"] {
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
  class codemaid_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/codemaid-corpus/src/lib.rs>>
  }
  class codemaid_mermaid__er__Cardinality["Cardinality"] {
    <<enum in crates/codemaid-mermaid/src/er.rs>>
  }
  class codemaid_model__codebase__Codebase["Codebase"] {
    <<struct in crates/codemaid-model/src/codebase.rs>>
  }
  class codemaid_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  codemaid_corpus__overview ..> codemaid_corpus__CorpusOptions
  codemaid_corpus__overview ..> codemaid_mermaid__er__Cardinality
  codemaid_corpus__overview ..> codemaid_model__codebase__Codebase
  codemaid_corpus__overview ..> codemaid_model__symbol__SymbolId
```

## `codemaid_corpus::overview::render`
`pub(crate) fn render(cb: &Codebase, opts: &CorpusOptions) -> String` · L14-L43
```mermaid
sequenceDiagram
  participant codemaid_corpus__overview as overview mod
  participant codemaid_model__codebase__Codebase as Codebase
  codemaid_corpus__overview->>codemaid_model__codebase__Codebase: stats()
  codemaid_corpus__overview->>codemaid_corpus__overview: crates_of(cb)
  codemaid_corpus__overview->>codemaid_corpus__overview: crate_graph(cb, &crates, opts)
  loop for krate in &crates
    codemaid_corpus__overview->>codemaid_corpus__overview: module_graph(cb, krate, opts)
  end
  loop for krate in &crates
    codemaid_corpus__overview->>codemaid_corpus__overview: data_model(cb, krate, opts)
  end
  codemaid_corpus__overview->>codemaid_corpus__overview: trait_map(cb, opts)
```

## `codemaid_corpus::overview::root`
`fn root(id: &SymbolId) -> &str` · L45-L47
```mermaid
sequenceDiagram
  participant codemaid_corpus__overview as overview mod
  participant codemaid_model__symbol__SymbolId as SymbolId
  codemaid_corpus__overview->>codemaid_model__symbol__SymbolId: as_str()
```

## `codemaid_corpus::overview::crates_of`
`fn crates_of(cb: &Codebase) -> BTreeSet<String>` · L49-L51
```mermaid
sequenceDiagram
  participant codemaid_corpus__overview as overview mod
  opt via map
    codemaid_corpus__overview->>codemaid_corpus__overview: root(&f.module)
  end
```

## `codemaid_corpus::overview::module_of`
`fn module_of(cb: &Codebase, id: &SymbolId) -> Option<SymbolId>` · L53-L57
> Module that physically contains a symbol (its file's module).
```mermaid
sequenceDiagram
  participant codemaid_corpus__overview as overview mod
  participant codemaid_model__codebase__Codebase as Codebase
  codemaid_corpus__overview->>codemaid_model__codebase__Codebase: symbol(id)?
  codemaid_corpus__overview->>codemaid_model__codebase__Codebase: file(&s.file)
```

## `codemaid_corpus::overview::crate_graph`
`fn crate_graph(cb: &Codebase, crates: &BTreeSet<String>, opts: &CorpusOptions) -> Option<String>` · L78-L129
```mermaid
sequenceDiagram
  participant codemaid_corpus__overview as overview mod
  participant codemaid_mermaid__flowchart__Flowchart as Flowchart
  participant codemaid_model__path__SourcePath as SourcePath
  participant codemaid_mermaid__escape__Ident as Ident
  loop for r in cb.relations.iter().filter(| r | STRUCTURAL.co…
    codemaid_corpus__overview->>codemaid_corpus__overview: root(&r.from)
    codemaid_corpus__overview->>codemaid_corpus__overview: root(&r.to)
  end
  opt crates.len()<2 && edges.is_empty()
    Note over codemaid_corpus__overview: return None
  end
  codemaid_corpus__overview->>codemaid_corpus__overview: heaviest(edges, opts.max_edges)
  codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: Flowchart::new(LR)
  loop for k in crates
    loop each via find
      codemaid_corpus__overview->>codemaid_corpus__overview: root(&file.module)
    end
    opt via map
      codemaid_corpus__overview->>codemaid_model__path__SourcePath: ~components()
    end
  end
  loop for (group, ks) in &groups
    alt multi
      codemaid_corpus__overview->>codemaid_mermaid__escape__Ident: Ident::new(&_)
      codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: subgraph(new(), group, None, |..|)
      opt via subgraph
        loop for k in ks
          codemaid_corpus__overview->>codemaid_mermaid__escape__Ident: Ident::new(k)
          codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: ~node(new(), k, Stadium)
        end
      end
    else
      loop for k in ks
        codemaid_corpus__overview->>codemaid_mermaid__escape__Ident: Ident::new(k)
        codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: node(new(), k, Stadium)
      end
    end
  end
  loop for e in &externals
    codemaid_corpus__overview->>codemaid_mermaid__escape__Ident: Ident::new(e)
    codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: node(new(), e, Hexagon)
  end
  loop for ((a, b), n) in &edges
    codemaid_corpus__overview->>codemaid_mermaid__escape__Ident: Ident::new(a)
    codemaid_corpus__overview->>codemaid_mermaid__escape__Ident: Ident::new(b)
    codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: edge(&new(), &new(), Solid, Some())
  end
  codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: render()
```

## `codemaid_corpus::overview::module_graph`
`fn module_graph(cb: &Codebase, krate: &str, opts: &CorpusOptions) -> Option<String>` · L131-L163
```mermaid
sequenceDiagram
  participant codemaid_corpus__overview as overview mod
  participant codemaid_mermaid__flowchart__Flowchart as Flowchart
  participant codemaid_corpus__naming as naming mod
  loop each via filter
    codemaid_corpus__overview->>codemaid_corpus__overview: root(m)
  end
  opt modules.len()<2
    Note over codemaid_corpus__overview: return None
  end
  loop each via filter
    codemaid_corpus__overview->>codemaid_corpus__overview: root(&r.from)
  end
  loop for r in cb.relations.iter().filter(| r | STRUCTURAL.co…
    codemaid_corpus__overview->>codemaid_corpus__overview: module_of(cb, &r.from)
    codemaid_corpus__overview->>codemaid_corpus__overview: module_of(cb, &r.to)
  end
  codemaid_corpus__overview->>codemaid_corpus__overview: heaviest(edges, opts.max_edges)
  codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: Flowchart::new(LR)
  loop for m in &modules
    codemaid_corpus__overview->>codemaid_corpus__naming: ident(m)
    codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: node(ident(), label, Rect)
  end
  loop for ((a, b), n) in &edges
    codemaid_corpus__overview->>codemaid_corpus__naming: ident(a)
    codemaid_corpus__overview->>codemaid_corpus__naming: ident(b)
    codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: edge(&ident(), &ident(), Solid, Some())
  end
  codemaid_corpus__overview->>codemaid_mermaid__flowchart__Flowchart: render()
```

## `codemaid_corpus::overview::data_model`
`fn data_model(cb: &Codebase, krate: &str, opts: &CorpusOptions) -> Option<String>` · L165-L211
```mermaid
sequenceDiagram
  participant codemaid_corpus__overview as overview mod
  participant codemaid_mermaid__er__ErDiagram as ErDiagram
  participant codemaid_corpus__naming as naming mod
  loop each via filter
    codemaid_corpus__overview->>codemaid_corpus__overview: root(&s.id)
  end
  loop for id in &types
    loop for m in s.members.iter().filter(| m | m.kind == Member…
      loop for t in m.refs.iter().filter(| t | types.contains(* t)…
        codemaid_corpus__overview->>codemaid_corpus__overview: cardinality(ty)
      end
    end
  end
  opt rels.is_empty()
    Note over codemaid_corpus__overview: return None
  end
  codemaid_corpus__overview->>codemaid_mermaid__er__ErDiagram: ErDiagram::new()
  loop for id in &keep
    codemaid_corpus__overview->>codemaid_corpus__naming: ident(id)
    codemaid_corpus__overview->>codemaid_mermaid__er__ErDiagram: entity(clone(), &s.name)
    loop for m in s.members.iter().filter(| m | m.kind == Member…
      codemaid_corpus__overview->>codemaid_mermaid__er__ErDiagram: attr(&e, unwrap_or(), &m.name, None)
    end
  end
  loop for (from, field, to, card, owned) in &rels
    opt keep.contains(from) && keep.contains(to)
      codemaid_corpus__overview->>codemaid_corpus__naming: ident(from)
      codemaid_corpus__overview->>codemaid_corpus__naming: ident(to)
      codemaid_corpus__overview->>codemaid_mermaid__er__ErDiagram: relation(&ident(), One, &ident(), _, _, field)
    end
  end
  codemaid_corpus__overview->>codemaid_mermaid__er__ErDiagram: render()
```

## `codemaid_corpus::overview::trait_map`
`fn trait_map(cb: &Codebase, opts: &CorpusOptions) -> Option<String>` · L229-L260
```mermaid
sequenceDiagram
  participant codemaid_corpus__overview as overview mod
  participant codemaid_model__codebase__Codebase as Codebase
  participant codemaid_mermaid__class__ClassDiagram as ClassDiagram
  participant codemaid_corpus__naming as naming mod
  participant codemaid_mermaid__class__Class as Class
  codemaid_corpus__overview->>codemaid_model__codebase__Codebase: symbols_of_kind(Trait)
  codemaid_corpus__overview->>codemaid_model__codebase__Codebase: relations_of_kind(Implements)
  codemaid_corpus__overview->>codemaid_model__codebase__Codebase: relations_of_kind(Extends)
  opt impls.is_empty()
    Note over codemaid_corpus__overview: return None
  end
  codemaid_corpus__overview->>codemaid_mermaid__class__ClassDiagram: ClassDiagram::new(LR)
  loop for t in &traits
    opt impls.iter().any(| r | &r.to == * t || &r.from == * …
      codemaid_corpus__overview->>codemaid_corpus__naming: ident(t)
      codemaid_corpus__overview->>codemaid_mermaid__class__Class: Class::new(ident(), &_.name)
      codemaid_corpus__overview->>codemaid_mermaid__class__Class: annotation(#quot;trait#quot;)
      codemaid_corpus__overview->>codemaid_mermaid__class__ClassDiagram: class(c)
    end
  end
  loop for r in &impls
    codemaid_corpus__overview->>codemaid_corpus__naming: ident(&r.from)
    codemaid_corpus__overview->>codemaid_mermaid__class__ClassDiagram: has_class(&ident())
    opt !d.has_class(&ident(&r.from))
      codemaid_corpus__overview->>codemaid_corpus__naming: ident(&r.from)
      codemaid_corpus__overview->>codemaid_mermaid__class__Class: Class::new(ident(), name())
      codemaid_corpus__overview->>codemaid_mermaid__class__ClassDiagram: class(new())
    end
    codemaid_corpus__overview->>codemaid_corpus__naming: ident(&r.from)
    codemaid_corpus__overview->>codemaid_corpus__naming: ident(&r.to)
    codemaid_corpus__overview->>codemaid_mermaid__class__ClassDiagram: relation(&ident(), &ident(), kind, None)
  end
  codemaid_corpus__overview->>codemaid_mermaid__class__ClassDiagram: render()
```
