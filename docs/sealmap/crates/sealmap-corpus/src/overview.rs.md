---
sealmap: 2
source: crates/sealmap-corpus/src/overview.rs
module: "sym:cargo sealmap_corpus . overview/"
language: rust
source_hash: blake3:7f78e627ded9d9d91e4c4f39eacf2d18e848bfcffe2cfad5537b54111b3bacfd
lines: 261
fragments: 9
---
# `sym:cargo sealmap_corpus . overview/` · crates/sealmap-corpus/src/overview.rs
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
    -root(id: &SymbolId) String
    -trait_map(cb: &Codebase, opts: &CorpusOptions) Option#lt;String#gt;
  }
  class sealmap_corpus___tCorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_mermaid__er___tCardinality["Cardinality"] {
    <<enum in crates/sealmap-mermaid/src/er.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  sealmap_corpus__overview ..> sealmap_corpus___tCorpusOptions
  sealmap_corpus__overview ..> sealmap_mermaid__er___tCardinality
  sealmap_corpus__overview ..> sealmap_model__codebase___tCodebase
  sealmap_corpus__overview ..> sealmap_model__sym___tSymbolId
```

## `sym:cargo sealmap_corpus . overview/render().`
`pub(crate) fn render(cb: &Codebase, opts: &CorpusOptions) -> String` · L14-L42
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_model__codebase___tCodebase as Codebase
  sealmap_corpus__overview->>sealmap_model__codebase___tCodebase: stats()
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

## `sym:cargo sealmap_corpus . overview/root().`
`fn root(id: &SymbolId) -> String` · L44-L46
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_corpus__overview->>sealmap_model__sym___tSymbolId: root()
```

## `sym:cargo sealmap_corpus . overview/crates_of().`
`fn crates_of(cb: &Codebase) -> BTreeSet<String>` · L48-L50
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  opt via map
    sealmap_corpus__overview->>sealmap_corpus__overview: root(&f.module)
  end
```

## `sym:cargo sealmap_corpus . overview/module_of().`
`fn module_of(cb: &Codebase, id: &SymbolId) -> Option<SymbolId>` · L52-L56
> Module that physically contains a symbol (its file's module).
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_model__codebase___tCodebase as Codebase
  sealmap_corpus__overview->>sealmap_model__codebase___tCodebase: symbol(id)?
  sealmap_corpus__overview->>sealmap_model__codebase___tCodebase: file(&s.file)
```

## `sym:cargo sealmap_corpus . overview/crate_graph().`
`fn crate_graph(cb: &Codebase, crates: &BTreeSet<String>, opts: &CorpusOptions) -> Option<String>` · L77-L129
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_mermaid__flowchart___tFlowchart as Flowchart
  participant sealmap_model__path___tSourcePath as SourcePath
  participant sealmap_mermaid__escape___tIdent as Ident
  loop for r in cb.relations.iter().filter(| r | STRUCTURAL.co…
    sealmap_corpus__overview->>sealmap_corpus__overview: root(&r.from)
    sealmap_corpus__overview->>sealmap_corpus__overview: root(&r.to)
  end
  opt crates.len()<2 && edges.is_empty()
    Note over sealmap_corpus__overview: return None
  end
  sealmap_corpus__overview->>sealmap_corpus__overview: heaviest(edges, opts.max_edges)
  sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: Flowchart::new(LR)
  loop for k in crates
    loop each via find
      sealmap_corpus__overview->>sealmap_corpus__overview: root(&file.module)
    end
    opt via map
      sealmap_corpus__overview->>sealmap_model__path___tSourcePath: ~components()
    end
  end
  loop for (group, ks) in &groups
    alt multi
      sealmap_corpus__overview->>sealmap_mermaid__escape___tIdent: Ident::new(&_)
      sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: subgraph(new(), group, None, |..|)
      opt via subgraph
        loop for k in ks
          sealmap_corpus__overview->>sealmap_mermaid__escape___tIdent: Ident::new(k)
          sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: ~node(new(), k, Stadium)
        end
      end
    else
      loop for k in ks
        sealmap_corpus__overview->>sealmap_mermaid__escape___tIdent: Ident::new(k)
        sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: node(new(), k, Stadium)
      end
    end
  end
  loop for e in &externals
    sealmap_corpus__overview->>sealmap_mermaid__escape___tIdent: Ident::new(e)
    sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: node(new(), e, Hexagon)
  end
  loop for ((a, b), n) in &edges
    sealmap_corpus__overview->>sealmap_mermaid__escape___tIdent: Ident::new(a)
    sealmap_corpus__overview->>sealmap_mermaid__escape___tIdent: Ident::new(b)
    sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: edge(&new(), &new(), Solid, Some())
  end
  sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: render()
```

## `sym:cargo sealmap_corpus . overview/module_graph().`
`fn module_graph(cb: &Codebase, krate: &str, opts: &CorpusOptions) -> Option<String>` · L131-L164
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_mermaid__flowchart___tFlowchart as Flowchart
  participant sealmap_model__sym___tSymbolId as SymbolId
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
  sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: Flowchart::new(LR)
  loop for m in &modules
    sealmap_corpus__overview->>sealmap_model__sym___tSymbolId: descriptors()
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(m)
    sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: node(ident(), &label, Rect)
  end
  loop for ((a, b), n) in &edges
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(a)
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(b)
    sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: edge(&ident(), &ident(), Solid, Some())
  end
  sealmap_corpus__overview->>sealmap_mermaid__flowchart___tFlowchart: render()
```

## `sym:cargo sealmap_corpus . overview/data_model().`
`fn data_model(cb: &Codebase, krate: &str, opts: &CorpusOptions) -> Option<String>` · L166-L212
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_mermaid__er___tErDiagram as ErDiagram
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
  sealmap_corpus__overview->>sealmap_mermaid__er___tErDiagram: ErDiagram::new()
  loop for id in &keep
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(id)
    sealmap_corpus__overview->>sealmap_mermaid__er___tErDiagram: entity(clone(), &s.name)
    loop for m in s.members.iter().filter(| m | m.kind == Member…
      sealmap_corpus__overview->>sealmap_mermaid__er___tErDiagram: attr(&e, unwrap_or(), &m.name, None)
    end
  end
  loop for (from, field, to, card, owned) in &rels
    opt keep.contains(from) && keep.contains(to)
      sealmap_corpus__overview->>sealmap_corpus__naming: ident(from)
      sealmap_corpus__overview->>sealmap_corpus__naming: ident(to)
      sealmap_corpus__overview->>sealmap_mermaid__er___tErDiagram: relation(&ident(), One, &ident(), _, _, field)
    end
  end
  sealmap_corpus__overview->>sealmap_mermaid__er___tErDiagram: render()
```

## `sym:cargo sealmap_corpus . overview/trait_map().`
`fn trait_map(cb: &Codebase, opts: &CorpusOptions) -> Option<String>` · L230-L261
```mermaid
sequenceDiagram
  participant sealmap_corpus__overview as overview mod
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_mermaid__class___tClassDiagram as ClassDiagram
  participant sealmap_corpus__naming as naming mod
  participant sealmap_mermaid__class___tClass as Class
  sealmap_corpus__overview->>sealmap_model__codebase___tCodebase: symbols_of_kind(Trait)
  sealmap_corpus__overview->>sealmap_model__codebase___tCodebase: relations_of_kind(Implements)
  sealmap_corpus__overview->>sealmap_model__codebase___tCodebase: relations_of_kind(Extends)
  opt impls.is_empty()
    Note over sealmap_corpus__overview: return None
  end
  sealmap_corpus__overview->>sealmap_mermaid__class___tClassDiagram: ClassDiagram::new(LR)
  loop for t in &traits
    opt impls.iter().any(| r | &r.to == * t || &r.from == * …
      sealmap_corpus__overview->>sealmap_corpus__naming: ident(t)
      sealmap_corpus__overview->>sealmap_mermaid__class___tClass: Class::new(ident(), &_.name)
      sealmap_corpus__overview->>sealmap_mermaid__class___tClass: annotation(#quot;trait#quot;)
      sealmap_corpus__overview->>sealmap_mermaid__class___tClassDiagram: class(c)
    end
  end
  loop for r in &impls
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(&r.from)
    sealmap_corpus__overview->>sealmap_mermaid__class___tClassDiagram: has_class(&ident())
    opt !d.has_class(&ident(&r.from))
      sealmap_corpus__overview->>sealmap_corpus__naming: ident(&r.from)
      sealmap_corpus__overview->>sealmap_mermaid__class___tClass: Class::new(ident(), &name())
      sealmap_corpus__overview->>sealmap_mermaid__class___tClassDiagram: class(new())
    end
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(&r.from)
    sealmap_corpus__overview->>sealmap_corpus__naming: ident(&r.to)
    sealmap_corpus__overview->>sealmap_mermaid__class___tClassDiagram: relation(&ident(), &ident(), kind, None)
  end
  sealmap_corpus__overview->>sealmap_mermaid__class___tClassDiagram: render()
```
