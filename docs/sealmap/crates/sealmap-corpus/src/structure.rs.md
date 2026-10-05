---
sealmap: 2
source: crates/sealmap-corpus/src/structure.rs
module: "sym:cargo sealmap_corpus . structure/"
language: rust
source_hash: blake3:75f20b20f9ff269fa60a054dfc5b8b9d35d5ac41de16902042e860df5f399a46
lines: 296
fragments: 4
---
# `sym:cargo sealmap_corpus . structure/` · crates/sealmap-corpus/src/structure.rs
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
  class sealmap_corpus___tCorpusOptions["CorpusOptions"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_corpus___tLookup["Lookup#lt;'a#gt;"] {
    <<struct in crates/sealmap-corpus/src/lib.rs>>
  }
  class sealmap_model__codebase___tCodebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__source___tSourceFile["SourceFile"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  sealmap_corpus__structure ..> sealmap_corpus___tCorpusOptions
  sealmap_corpus__structure ..> sealmap_corpus___tLookup
  sealmap_corpus__structure ..> sealmap_model__codebase___tCodebase
  sealmap_corpus__structure ..> sealmap_model__source___tSourceFile
  sealmap_corpus__structure ..> sealmap_model__sym___tSymbolId
```

## `sym:cargo sealmap_corpus . structure/render().`
`pub(crate) fn render(cb: &Codebase, lookup: &Lookup<'_>, file: &SourceFile, opts: &CorpusOptions,) -> Option<(String, usize)>` · L12-L203
> Render the structure diagram for `file`.
```mermaid
sequenceDiagram
  participant sealmap_corpus__structure as structure mod
  participant sealmap_corpus___tLookup as Lookup
  participant sealmap_mermaid__class___tClassDiagram as ClassDiagram
  participant sealmap_model__symbol___tSymbolKind as SymbolKind
  participant sealmap_corpus__naming as naming mod
  participant sealmap_mermaid__class___tClass as Class
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_model__symbol___tVisibility as Visibility
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_corpus__structure->>sealmap_corpus___tLookup: in_file(&file.path)
  sealmap_corpus__structure->>sealmap_mermaid__class___tClassDiagram: ClassDiagram::new(LR)
  loop for s in &here
    sealmap_corpus__structure->>sealmap_model__symbol___tSymbolKind: ~is_type()
  end
  loop for owner in &owners
    sealmap_corpus__structure->>sealmap_corpus__naming: ident(owner)
    sealmap_corpus__structure->>sealmap_corpus__structure: type_label(cb, owner)
    sealmap_corpus__structure->>sealmap_mermaid__class___tClass: Class::new(ident(), &type_label())
    sealmap_corpus__structure->>sealmap_model__codebase___tCodebase: symbol(owner)
    alt Some(t) if t.file == file.path
      sealmap_corpus__structure->>sealmap_model__symbol___tSymbolKind: ~keyword()
      sealmap_corpus__structure->>sealmap_mermaid__class___tClass: annotation(keyword())
      loop for m in &t.members
        alt MemberKind::Field
          sealmap_corpus__structure->>sealmap_model__symbol___tVisibility: ~uml_marker()
          sealmap_corpus__structure->>sealmap_mermaid__class___tClass: field(uml_marker(), &m.name, unwrap_or())
        else MemberKind::Variant
          sealmap_corpus__structure->>sealmap_corpus__structure: variant_text(&m.name, as_deref())
          sealmap_corpus__structure->>sealmap_mermaid__class___tClass: field('', &variant_text(), #quot;#quot;)
        else MemberKind::AssocType
          sealmap_corpus__structure->>sealmap_mermaid__class___tClass: field('+', &_, unwrap_or())
        else MemberKind::AssocConst
          sealmap_corpus__structure->>sealmap_mermaid__class___tClass: field('+', &_, unwrap_or())
        else MemberKind::RequiredMethod
          sealmap_corpus__structure->>sealmap_corpus__structure: split_signature(unwrap_or())
          sealmap_corpus__structure->>sealmap_mermaid__class___tClass: method('+', &m.name, &params, &ret)
        end
      end
    else Some(t)
      sealmap_corpus__structure->>sealmap_model__symbol___tSymbolKind: ~keyword()
      sealmap_corpus__structure->>sealmap_mermaid__class___tClass: annotation(&_)
    else None
      sealmap_corpus__structure->>sealmap_mermaid__class___tClass: annotation(#quot;external#quot;)
    end
    loop for m in here.iter().filter(| s | s.kind == SymbolKind:…
      sealmap_corpus__structure->>sealmap_corpus__structure: split_signature(unwrap_or())
      opt Some(tr)
        sealmap_corpus__structure->>sealmap_corpus__structure: short_trait(tr)
      end
      sealmap_corpus__structure->>sealmap_model__symbol___tVisibility: ~uml_marker()
      sealmap_corpus__structure->>sealmap_mermaid__class___tClass: method(uml_marker(), &name, &params, &ret)
    end
    sealmap_corpus__structure->>sealmap_mermaid__class___tClassDiagram: class(class)
  end
  loop for module in &modules
    sealmap_corpus__structure->>sealmap_corpus__naming: ident(&module.id)
    sealmap_corpus__structure->>sealmap_model__sym___tSymbolId: ~display_path()
    sealmap_corpus__structure->>sealmap_mermaid__class___tClass: Class::new(ident(), &display_path())
    sealmap_corpus__structure->>sealmap_mermaid__class___tClass: annotation(#quot;module#quot;)
    sealmap_corpus__structure->>sealmap_corpus___tLookup: children(&module.id)
    loop for s in lookup.children(&module.id).filter(| s | visib…
      alt SymbolKind::Module
        sealmap_corpus__structure->>sealmap_model__symbol___tVisibility: ~uml_marker()
        sealmap_corpus__structure->>sealmap_mermaid__class___tClass: field(uml_marker(), &_, #quot;#quot;)
      else SymbolKind::Function
        sealmap_corpus__structure->>sealmap_corpus__structure: split_signature(unwrap_or())
        sealmap_corpus__structure->>sealmap_model__symbol___tVisibility: ~uml_marker()
        sealmap_corpus__structure->>sealmap_mermaid__class___tClass: method(uml_marker(), &s.name, &params, &ret)
      else SymbolKind::Const | SymbolKind::Static
        sealmap_corpus__structure->>sealmap_model__symbol___tVisibility: ~uml_marker()
        sealmap_corpus__structure->>sealmap_model__symbol___tSymbolKind: ~keyword()
        sealmap_corpus__structure->>sealmap_mermaid__class___tClass: field(uml_marker(), &_, ty)
      else SymbolKind::Macro
        sealmap_corpus__structure->>sealmap_model__symbol___tVisibility: ~uml_marker()
        sealmap_corpus__structure->>sealmap_mermaid__class___tClass: field(uml_marker(), &_, #quot;#quot;)
      end
    end
    opt any || owners.is_empty()
      sealmap_corpus__structure->>sealmap_mermaid__class___tClassDiagram: class(class)
    end
  end
  sealmap_corpus__structure->>sealmap_mermaid__class___tClassDiagram: class_count()
  opt d.class_count() == 0
    Note over sealmap_corpus__structure: return None
  end
  loop for s in &here
    sealmap_corpus__structure->>sealmap_model__codebase___tCodebase: relations_from(&s.id)
  end
  loop for owner in &owners
    sealmap_corpus__structure->>sealmap_model__codebase___tCodebase: relations_from(owner)
  end
  loop for (from, to, kind, label) in &edges
    opt !drawn.contains(to)
      sealmap_corpus__structure->>sealmap_corpus__naming: ident(to)
      sealmap_corpus__structure->>sealmap_corpus__structure: type_label(cb, to)
      sealmap_corpus__structure->>sealmap_mermaid__class___tClass: Class::new(ident(), &type_label())
      sealmap_corpus__structure->>sealmap_model__codebase___tCodebase: symbol(to)
      alt Some(t)
        sealmap_corpus__structure->>sealmap_model__symbol___tSymbolKind: ~keyword()
        sealmap_corpus__structure->>sealmap_mermaid__class___tClass: annotation(&_)
      else None
        sealmap_corpus__structure->>sealmap_mermaid__class___tClass: annotation(#quot;external#quot;)
      end
      sealmap_corpus__structure->>sealmap_mermaid__class___tClassDiagram: class(stub)
    end
    sealmap_corpus__structure->>sealmap_corpus__naming: ident(from)
    sealmap_corpus__structure->>sealmap_corpus__naming: ident(to)
    sealmap_corpus__structure->>sealmap_mermaid__class___tClassDiagram: relation(&ident(), &ident(), k, label)
  end
  sealmap_corpus__structure->>sealmap_mermaid__class___tClassDiagram: class_count()
  sealmap_corpus__structure->>sealmap_mermaid__class___tClassDiagram: render()
```

## `sym:cargo sealmap_corpus . structure/type_label().`
`fn type_label(cb: &Codebase, id: &SymbolId) -> String` · L205-L215
> Display label for a type: name with generics.
```mermaid
sequenceDiagram
  participant sealmap_corpus__structure as structure mod
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_corpus__structure->>sealmap_model__codebase___tCodebase: symbol(id)
  opt None
    sealmap_corpus__structure->>sealmap_model__sym___tSymbolId: display_path()
  end
```

## `sym:cargo sealmap_corpus . structure/is_shared().`
`fn is_shared(cb: &Codebase, from: &SymbolId, fields: &str) -> bool` · L230-L253
> Is the field holding the target behind `Option`, `Arc`, `Rc`, a reference or a collection (aggregation rather than composition)?
```mermaid
sequenceDiagram
  participant sealmap_corpus__structure as structure mod
  participant sealmap_model__codebase___tCodebase as Codebase
  sealmap_corpus__structure->>sealmap_model__codebase___tCodebase: symbol(from)
  opt let-else
    Note over sealmap_corpus__structure: return false
  end
```
