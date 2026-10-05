---
sealmap: 1
source: crates/sealmap-corpus/src/structure.rs
module: sealmap_corpus::structure
language: rust
source_hash: blake3:71a3e09c57caab314472784dfc57eb7a9b52435b483a740eef2b49ce29e6e423
lines: 296
fragments: 4
---
# `sealmap_corpus::structure` · crates/sealmap-corpus/src/structure.rs
> Per-file `classDiagram`: what the file defines and how it relates to the rest of the codebase.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_corpus__structure["sealmap_corpus::structure"] {
    <<module>>
    -is_shared(cb: &Codebase, from: &SymbolId, fields: &str) bool
    ~render(crate) Option#lt;#40;String, usize#41;#gt;
    -short_trait(t: &str) &str
    ~split_signature(crate) #40;String, String#41;
    -type_label(cb: &Codebase, id: &SymbolId) String
    -variant_text(name: &str, ty: Option#lt;&str#gt;) String
  }
  class sealmap_corpus__CorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_corpus__Lookup["Lookup#lt;'a#gt;"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__source__SourceFile["SourceFile"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  sealmap_corpus__structure ..> sealmap_corpus__CorpusOptions
  sealmap_corpus__structure ..> sealmap_corpus__Lookup
  sealmap_corpus__structure ..> sealmap_model__codebase__Codebase
  sealmap_corpus__structure ..> sealmap_model__source__SourceFile
  sealmap_corpus__structure ..> sealmap_model__symbol__SymbolId
```

## `sealmap_corpus::structure::render`
`pub(crate) fn render(cb: &Codebase, lookup: &Lookup<'_>, file: &SourceFile, opts: &CorpusOptions,) -> Option<(String, usize)>` · L12-L203
> Render the structure diagram for `file`.
```mermaid
sequenceDiagram
  participant sealmap_corpus__structure as structure mod
  participant sealmap_corpus__Lookup as Lookup
  participant sealmap_mermaid__class__ClassDiagram as ClassDiagram
  participant sealmap_model__symbol__SymbolKind as SymbolKind
  participant sealmap_corpus__naming as naming mod
  participant sealmap_mermaid__class__Class as Class
  participant sealmap_model__codebase__Codebase as Codebase
  participant sealmap_model__symbol__Visibility as Visibility
  sealmap_corpus__structure->>sealmap_corpus__Lookup: in_file(&file.path)
  sealmap_corpus__structure->>sealmap_mermaid__class__ClassDiagram: ClassDiagram::new(LR)
  loop for s in &here
    sealmap_corpus__structure->>sealmap_model__symbol__SymbolKind: ~is_type()
  end
  loop for owner in &owners
    sealmap_corpus__structure->>sealmap_corpus__naming: ident(owner)
    sealmap_corpus__structure->>sealmap_corpus__structure: type_label(cb, owner)
    sealmap_corpus__structure->>sealmap_mermaid__class__Class: Class::new(ident(), &type_label())
    sealmap_corpus__structure->>sealmap_model__codebase__Codebase: symbol(owner)
    alt Some(t) if t.file == file.path
      sealmap_corpus__structure->>sealmap_model__symbol__SymbolKind: ~keyword()
      sealmap_corpus__structure->>sealmap_mermaid__class__Class: annotation(keyword())
      loop for m in &t.members
        alt MemberKind::Field
          sealmap_corpus__structure->>sealmap_model__symbol__Visibility: ~uml_marker()
          sealmap_corpus__structure->>sealmap_mermaid__class__Class: field(uml_marker(), &m.name, unwrap_or())
        else MemberKind::Variant
          sealmap_corpus__structure->>sealmap_corpus__structure: variant_text(&m.name, as_deref())
          sealmap_corpus__structure->>sealmap_mermaid__class__Class: field('', &variant_text(), #quot;#quot;)
        else MemberKind::AssocType
          sealmap_corpus__structure->>sealmap_mermaid__class__Class: field('+', &_, unwrap_or())
        else MemberKind::AssocConst
          sealmap_corpus__structure->>sealmap_mermaid__class__Class: field('+', &_, unwrap_or())
        else MemberKind::RequiredMethod
          sealmap_corpus__structure->>sealmap_corpus__structure: split_signature(unwrap_or())
          sealmap_corpus__structure->>sealmap_mermaid__class__Class: method('+', &m.name, &params, &ret)
        end
      end
    else Some(t)
      sealmap_corpus__structure->>sealmap_model__symbol__SymbolKind: ~keyword()
      sealmap_corpus__structure->>sealmap_mermaid__class__Class: annotation(&_)
    else None
      sealmap_corpus__structure->>sealmap_mermaid__class__Class: annotation(#quot;external#quot;)
    end
    loop for m in here.iter().filter(| s | s.kind == SymbolKind:…
      sealmap_corpus__structure->>sealmap_corpus__structure: split_signature(unwrap_or())
      opt Some(tr)
        sealmap_corpus__structure->>sealmap_corpus__structure: short_trait(tr)
      end
      sealmap_corpus__structure->>sealmap_model__symbol__Visibility: ~uml_marker()
      sealmap_corpus__structure->>sealmap_mermaid__class__Class: method(uml_marker(), &name, &params, &ret)
    end
    sealmap_corpus__structure->>sealmap_mermaid__class__ClassDiagram: class(class)
  end
  loop for module in &modules
    sealmap_corpus__structure->>sealmap_corpus__naming: ident(&module.id)
    sealmap_corpus__structure->>sealmap_mermaid__class__Class: Class::new(ident(), &to_string())
    sealmap_corpus__structure->>sealmap_mermaid__class__Class: annotation(#quot;module#quot;)
    sealmap_corpus__structure->>sealmap_corpus__Lookup: children(&module.id)
    loop for s in lookup.children(&module.id).filter(| s | visib…
      alt SymbolKind::Module
        sealmap_corpus__structure->>sealmap_model__symbol__Visibility: ~uml_marker()
        sealmap_corpus__structure->>sealmap_mermaid__class__Class: field(uml_marker(), &_, #quot;#quot;)
      else SymbolKind::Function
        sealmap_corpus__structure->>sealmap_corpus__structure: split_signature(unwrap_or())
        sealmap_corpus__structure->>sealmap_model__symbol__Visibility: ~uml_marker()
        sealmap_corpus__structure->>sealmap_mermaid__class__Class: method(uml_marker(), &s.name, &params, &ret)
      else SymbolKind::Const | SymbolKind::Static
        sealmap_corpus__structure->>sealmap_model__symbol__Visibility: ~uml_marker()
        sealmap_corpus__structure->>sealmap_model__symbol__SymbolKind: ~keyword()
        sealmap_corpus__structure->>sealmap_mermaid__class__Class: field(uml_marker(), &_, ty)
      else SymbolKind::Macro
        sealmap_corpus__structure->>sealmap_model__symbol__Visibility: ~uml_marker()
        sealmap_corpus__structure->>sealmap_mermaid__class__Class: field(uml_marker(), &_, #quot;#quot;)
      end
    end
    opt any || owners.is_empty()
      sealmap_corpus__structure->>sealmap_mermaid__class__ClassDiagram: class(class)
    end
  end
  sealmap_corpus__structure->>sealmap_mermaid__class__ClassDiagram: class_count()
  opt d.class_count() == 0
    Note over sealmap_corpus__structure: return None
  end
  loop for s in &here
    sealmap_corpus__structure->>sealmap_model__codebase__Codebase: relations_from(&s.id)
  end
  loop for owner in &owners
    sealmap_corpus__structure->>sealmap_model__codebase__Codebase: relations_from(owner)
  end
  loop for (from, to, kind, label) in &edges
    opt !drawn.contains(to)
      sealmap_corpus__structure->>sealmap_corpus__naming: ident(to)
      sealmap_corpus__structure->>sealmap_corpus__structure: type_label(cb, to)
      sealmap_corpus__structure->>sealmap_mermaid__class__Class: Class::new(ident(), &type_label())
      sealmap_corpus__structure->>sealmap_model__codebase__Codebase: symbol(to)
      alt Some(t)
        sealmap_corpus__structure->>sealmap_model__symbol__SymbolKind: ~keyword()
        sealmap_corpus__structure->>sealmap_mermaid__class__Class: annotation(&_)
      else None
        sealmap_corpus__structure->>sealmap_mermaid__class__Class: annotation(#quot;external#quot;)
      end
      sealmap_corpus__structure->>sealmap_mermaid__class__ClassDiagram: class(stub)
    end
    sealmap_corpus__structure->>sealmap_corpus__naming: ident(from)
    sealmap_corpus__structure->>sealmap_corpus__naming: ident(to)
    sealmap_corpus__structure->>sealmap_mermaid__class__ClassDiagram: relation(&ident(), &ident(), k, label)
  end
  sealmap_corpus__structure->>sealmap_mermaid__class__ClassDiagram: class_count()
  sealmap_corpus__structure->>sealmap_mermaid__class__ClassDiagram: render()
```

## `sealmap_corpus::structure::type_label`
`fn type_label(cb: &Codebase, id: &SymbolId) -> String` · L205-L215
> Display label for a type: name with generics.
```mermaid
sequenceDiagram
  participant sealmap_corpus__structure as structure mod
  participant sealmap_model__codebase__Codebase as Codebase
  participant sealmap_model__symbol__SymbolId as SymbolId
  sealmap_corpus__structure->>sealmap_model__codebase__Codebase: symbol(id)
  opt None
    sealmap_corpus__structure->>sealmap_model__symbol__SymbolId: as_str()
  end
```

## `sealmap_corpus::structure::is_shared`
`fn is_shared(cb: &Codebase, from: &SymbolId, fields: &str) -> bool` · L230-L253
> Is the field holding the target behind `Option`, `Arc`, `Rc`, a reference or a collection (aggregation rather than composition)?
```mermaid
sequenceDiagram
  participant sealmap_corpus__structure as structure mod
  participant sealmap_model__codebase__Codebase as Codebase
  sealmap_corpus__structure->>sealmap_model__codebase__Codebase: symbol(from)
  opt let-else
    Note over sealmap_corpus__structure: return false
  end
```
