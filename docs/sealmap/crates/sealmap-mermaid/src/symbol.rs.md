---
sealmap: 2
source: crates/sealmap-mermaid/src/symbol.rs
module: "sym:cargo sealmap_mermaid . symbol/"
language: rust
source_hash: blake3:3e50807acb0dbb673cee678314011b7fdeab3ed798cc85214d476ac8ee8b9a44
lines: 366
fragments: 5
---
# `sym:cargo sealmap_mermaid . symbol/` · crates/sealmap-mermaid/src/symbol.rs
> Injective diagram ids for `sym:` symbol ids (feature `model`).

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_mermaid__escape___tIdent["Ident"] {
    <<struct in crates/sealmap-mermaid/src/escape.rs>>
    +from_symbol(id: &SymbolId) Self
  }
  class sealmap_mermaid__symbol["sealmap_mermaid::symbol"] {
    <<module>>
    -escape(s: &str, out: &mut String)
    -head(out: &mut String, escaped_tag: &str, prefix: &str, name: &str)
    -is_plain(s: &str) bool
    -tagged(out: &mut String, tag: char, name: &str)
    -untagged(out: &mut String, tag: char, name: &str)
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/sym.rs>>
  }
  sealmap_mermaid__escape___tIdent ..> sealmap_model__sym___tSymbolId
```

## `sym:cargo sealmap_mermaid . escape/Ident#from_symbol().`
`pub fn from_symbol(id: &SymbolId) -> Self` · L10-L108
> The diagram id of a symbol, derived from the structure of its [`SymbolId`] by an **injective** encoding: two different ids never share a diagram id, by constru…
```mermaid
sequenceDiagram
  participant sealmap_mermaid__escape___tIdent as Ident
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_mermaid__symbol as symbol mod
  participant sealmap_mermaid__escape as escape mod
  sealmap_mermaid__escape___tIdent->>sealmap_model__sym___tSymbolId: as_str()
  sealmap_mermaid__escape___tIdent->>sealmap_model__sym___tSymbolId: view()
  alt IdView::Global(g)
    sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: head(&out, #quot;__P#quot;, #quot;#quot;, &g.package)
    opt g.manager != #quot;cargo#quot;
      sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 'g', g.manager)
    end
    opt let Some(v) = &g.version
      sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 'r', v)
    end
    loop for d in &g.descriptors
      alt DescriptorKind::Namespace
        sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: untagged(&out, 'n', &d.name)
      else DescriptorKind::Type
        sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 't', &d.name)
      else DescriptorKind::Term
        sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 'v', &d.name)
      else DescriptorKind::Method
        sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 'f', &d.name)
        opt let Some(dis) = d.disambiguator
          sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 'd', dis)
        end
      else DescriptorKind::TypeParameter
        sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 'p', &d.name)
      else DescriptorKind::Parameter
        sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 'a', &d.name)
      else DescriptorKind::Meta
        sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 'k', &d.name)
      else DescriptorKind::Macro
        sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: tagged(&out, 'x', &d.name)
      end
    end
  else IdView::Path(segments)
    sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: head(&out, #quot;__S#quot;, #quot;_#quot;, first)
    loop for s in rest
      sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: untagged(&out, 's', s)
    end
  else IdView::Unresolved(name)
    sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: is_plain(&name)
    opt not is_plain(&name)
      sealmap_mermaid__escape___tIdent->>sealmap_mermaid__symbol: escape(&name, &out)
    end
  end
  sealmap_mermaid__escape___tIdent->>sealmap_mermaid__escape: is_reserved(&out)
  sealmap_mermaid__escape___tIdent->>sealmap_mermaid__escape___tIdent: Ident::from_valid(out)
```

## `sym:cargo sealmap_mermaid . symbol/head().`
`fn head(out: &mut String, escaped_tag: &str, prefix: &str, name: &str)` · L130-L140
> The first name: `prefix` + the name when plain and starting with a letter, otherwise `escaped_tag` + the escaped name.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__symbol as symbol mod
  sealmap_mermaid__symbol->>sealmap_mermaid__symbol: is_plain(name)
  opt not is_plain(name) && name.starts_with(| c: char | c.is_…
    sealmap_mermaid__symbol->>sealmap_mermaid__symbol: escape(name, out)
  end
```

## `sym:cargo sealmap_mermaid . symbol/tagged().`
`fn tagged(out: &mut String, tag: char, name: &str)` · L142-L152
> `___` + tag + plain name, or `___` + upper-case tag + escaped name.
```mermaid
sequenceDiagram
  participant sealmap_mermaid__symbol as symbol mod
  sealmap_mermaid__symbol->>sealmap_mermaid__symbol: is_plain(name)
  opt not is_plain(name)
    sealmap_mermaid__symbol->>sealmap_mermaid__symbol: escape(name, out)
  end
```

## `sym:cargo sealmap_mermaid . symbol/untagged().`
`fn untagged(out: &mut String, tag: char, name: &str)` · L154-L164
> `__` + plain name, or the escaped tagged form (`tag` upper-cased).
```mermaid
sequenceDiagram
  participant sealmap_mermaid__symbol as symbol mod
  sealmap_mermaid__symbol->>sealmap_mermaid__symbol: is_plain(name)
  opt not is_plain(name)
    sealmap_mermaid__symbol->>sealmap_mermaid__symbol: escape(name, out)
  end
```
