---
codemaid: 1
source: crates/codemaid-rust/src/collect.rs
module: codemaid_rust::collect
language: rust
source_hash: blake3:6fe2fe57ab23bc9f7f9687572467c88bb1f1574fae63cdd8a3da172e76d60b2f
lines: 997
fragments: 34
---
# `codemaid_rust::collect` · crates/codemaid-rust/src/collect.rs
> Pass 1: parse one file with `syn` and collect unresolved raw data.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_rust__collect__Collector["Collector#lt;'a#gt;"] {
    <<struct>>
    -opts: &'a RustOptions
    -raw: &'a mut RawFile
    -item(&mut self, module: &Segs, item: &Item)
    -module(&mut self, path: Segs, attrs: &[Attribute], items: &[Item], span: Span, vis: Visibility)
    -skip(&self, attrs: &[Attribute]) bool
  }
  class codemaid_rust__collect__FlowWalker["FlowWalker"] {
    <<struct>>
    -env: BTreeMap#lt;String, Recv#gt;
    -arg(&mut self, a: &Expr, out: &mut Vec#lt;RawStep#gt;, deferred: &mut Vec#lt;Vec#lt;RawStep#gt;#gt;)
    -bind(&mut self, pat: &Pat, init: Option#lt;&Expr#gt;)
    -block(&mut self, b: &Block) Vec#lt;RawStep#gt;
    -cond(&mut self, cond: &Expr, out: &mut Vec#lt;RawStep#gt;)
    -expr(&mut self, e: &Expr, out: &mut Vec#lt;RawStep#gt;)
    -flush_deferred(&self, callee: &str, deferred: Vec#lt;Vec#lt;RawStep#gt;#gt;, out: &mut Vec#lt;RawStep#gt;)
    -mac(&mut self, m: &syn::Macro, out: &mut Vec#lt;RawStep#gt;)
    -new(env: BTreeMap#lt;String, Recv#gt;) Self
    -recv(&self, e: &Expr) Recv
    -stmts(&mut self, stmts: &[Stmt], out: &mut Vec#lt;RawStep#gt;)
    -sub(&mut self, e: &Expr) Vec#lt;RawStep#gt;
    -sub_block(&mut self, b: &Block) Vec#lt;RawStep#gt;
  }
  class codemaid_rust__collect["codemaid_rust::collect"] {
    <<module>>
    -const LABEL_MAX: usize
    -const LOOPING: &[&str]
    -const SIG_MAX: usize
    -arg_sketch(e: &Expr) String
    -call(callee: Callee, name: &str, args: &Punctuated#lt;Expr, syn::Token![,]#gt;, kind: CallKind, span: PmSpan) RawStep
    ~collect_file(crate) RawFile
    -cond_label(cond: &Expr) String
    -constructed_type(e: &Expr) Option#lt;Segs#gt;
    -doc_of(attrs: &[Attribute]) Option#lt;String#gt;
    -fields_of(fields: &Fields) Vec#lt;RawMember#gt;
    -first_path(ty: &Type) Option#lt;Segs#gt;
    -flatten_use(tree: &UseTree, prefix: &mut Segs, out: &mut Vec#lt;RawUse#gt;)
    -fn_tags(sig: &syn::Signature, attrs: &[Attribute]) Vec#lt;String#gt;
    -generics_of(g: &Generics) Vec#lt;String#gt;
    -is_call(e: &Expr) bool
    -is_test_attr(a: &Attribute) bool
    -last_call_mut(out: &mut [RawStep]) Option#lt;&mut RawStep#gt;
    -params_of(sig: &syn::Signature) BTreeMap#lt;String, Recv#gt;
    ~path_segs(crate) Segs
    -push_arms(arms: Vec#lt;#40;String, Vec#lt;RawStep#gt;#41;#gt;, out: &mut Vec#lt;RawStep#gt;)
    -sig_refs(sig: &syn::Signature, out: &mut Vec#lt;Segs#gt;)
    -span_of(s: PmSpan) Span
    -tags_of(attrs: &[Attribute]) Vec#lt;String#gt;
    -type_refs(ty: &Type, out: &mut Vec#lt;Segs#gt;)
    -vis_of(v: &syn::Visibility) Visibility
    -vis_prefix(v: &syn::Visibility) String
  }
  class codemaid_model__flow__CallKind["CallKind"] {
    <<enum in crates/codemaid-model/src/flow.rs>>
  }
  class codemaid_model__path__SourcePath["SourcePath"] {
    <<struct in crates/codemaid-model/src/path.rs>>
  }
  class codemaid_model__symbol__Span["Span"] {
    <<struct in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_model__symbol__Visibility["Visibility"] {
    <<enum in crates/codemaid-model/src/symbol.rs>>
  }
  class codemaid_rust__RustOptions["RustOptions"] {
    <<struct in crates/codemaid-rust/src/lib.rs>>
  }
  class codemaid_rust__layout__FileRole["FileRole"] {
    <<struct in crates/codemaid-rust/src/layout.rs>>
  }
  class codemaid_rust__raw__Callee["Callee"] {
    <<enum in crates/codemaid-rust/src/raw.rs>>
  }
  class codemaid_rust__raw__RawFile["RawFile"] {
    <<struct in crates/codemaid-rust/src/raw.rs>>
  }
  class codemaid_rust__raw__RawMember["RawMember"] {
    <<struct in crates/codemaid-rust/src/raw.rs>>
  }
  class codemaid_rust__raw__RawStep["RawStep"] {
    <<enum in crates/codemaid-rust/src/raw.rs>>
  }
  class codemaid_rust__raw__RawUse["RawUse"] {
    <<struct in crates/codemaid-rust/src/raw.rs>>
  }
  class codemaid_rust__raw__Recv["Recv"] {
    <<enum in crates/codemaid-rust/src/raw.rs>>
  }
  class codemaid_rust__raw__Segs["Segs"] {
    <<type in crates/codemaid-rust/src/raw.rs>>
  }
  class proc_macro2__Span["proc_macro2::Span"] {
    <<external>>
  }
  class syn__Attribute["syn::Attribute"] {
    <<external>>
  }
  class syn__Expr["syn::Expr"] {
    <<external>>
  }
  class syn__Fields["syn::Fields"] {
    <<external>>
  }
  class syn__Generics["syn::Generics"] {
    <<external>>
  }
  class syn__Path["syn::Path"] {
    <<external>>
  }
  class syn__Signature["syn::Signature"] {
    <<external>>
  }
  class syn__Token["syn::Token"] {
    <<external>>
  }
  class syn__Type["syn::Type"] {
    <<external>>
  }
  class syn__UseTree["syn::UseTree"] {
    <<external>>
  }
  class syn__Visibility["syn::Visibility"] {
    <<external>>
  }
  class syn__punctuated__Punctuated["syn::punctuated::Punctuated"] {
    <<external>>
  }
  class syn__Item["syn::Item"] {
    <<external>>
  }
  class syn__Block["syn::Block"] {
    <<external>>
  }
  class syn__Macro["syn::Macro"] {
    <<external>>
  }
  class syn__Pat["syn::Pat"] {
    <<external>>
  }
  class syn__Stmt["syn::Stmt"] {
    <<external>>
  }
  codemaid_rust__collect ..> codemaid_model__flow__CallKind
  codemaid_rust__collect ..> codemaid_model__path__SourcePath
  codemaid_rust__collect ..> codemaid_model__symbol__Span
  codemaid_rust__collect ..> codemaid_model__symbol__Visibility
  codemaid_rust__collect ..> codemaid_rust__RustOptions
  codemaid_rust__collect ..> codemaid_rust__layout__FileRole
  codemaid_rust__collect ..> codemaid_rust__raw__Callee
  codemaid_rust__collect ..> codemaid_rust__raw__RawFile
  codemaid_rust__collect ..> codemaid_rust__raw__RawMember
  codemaid_rust__collect ..> codemaid_rust__raw__RawStep
  codemaid_rust__collect ..> codemaid_rust__raw__RawUse
  codemaid_rust__collect ..> codemaid_rust__raw__Recv
  codemaid_rust__collect ..> codemaid_rust__raw__Segs
  codemaid_rust__collect ..> proc_macro2__Span
  codemaid_rust__collect ..> syn__Attribute
  codemaid_rust__collect ..> syn__Expr
  codemaid_rust__collect ..> syn__Fields
  codemaid_rust__collect ..> syn__Generics
  codemaid_rust__collect ..> syn__Path
  codemaid_rust__collect ..> syn__Signature
  codemaid_rust__collect ..> syn__Token
  codemaid_rust__collect ..> syn__Type
  codemaid_rust__collect ..> syn__UseTree
  codemaid_rust__collect ..> syn__Visibility
  codemaid_rust__collect ..> syn__punctuated__Punctuated
  codemaid_rust__collect__Collector ..> codemaid_model__symbol__Span
  codemaid_rust__collect__Collector ..> codemaid_model__symbol__Visibility
  codemaid_rust__collect__Collector o-- codemaid_rust__RustOptions : opts
  codemaid_rust__collect__Collector o-- codemaid_rust__raw__RawFile : raw
  codemaid_rust__collect__Collector ..> codemaid_rust__raw__Segs
  codemaid_rust__collect__Collector ..> syn__Attribute
  codemaid_rust__collect__Collector ..> syn__Item
  codemaid_rust__collect__FlowWalker ..> codemaid_rust__raw__RawStep
  codemaid_rust__collect__FlowWalker o-- codemaid_rust__raw__Recv : env
  codemaid_rust__collect__FlowWalker ..> syn__Block
  codemaid_rust__collect__FlowWalker ..> syn__Expr
  codemaid_rust__collect__FlowWalker ..> syn__Macro
  codemaid_rust__collect__FlowWalker ..> syn__Pat
  codemaid_rust__collect__FlowWalker ..> syn__Stmt
```

## `codemaid_rust::collect::collect_file`
`pub(crate) fn collect_file(path: &SourcePath, role: &FileRole, text: &str, opts: &RustOptions) -> RawFile` · L24-L55
> Parse `text` and collect everything pass 2 needs.
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant codemaid_model__hash__ContentHash as ContentHash
  participant syn as syn ext
  participant codemaid_model__symbol__Span as Span
  participant codemaid_rust__collect__Collector as Collector
  codemaid_rust__collect->>codemaid_model__hash__ContentHash: ContentHash::of_text(text)
  codemaid_rust__collect->>syn: syn::parse_file(text)
  opt Err(e)
    codemaid_rust__collect->>codemaid_model__symbol__Span: Span::new(1, 1, max(), 1)
    Note over codemaid_rust__collect: return raw
  end
  codemaid_rust__collect->>codemaid_model__symbol__Span: Span::new(1, 1, max(), 1)
  codemaid_rust__collect->>codemaid_rust__collect__Collector: module(clone(), &file.attrs, &file.items, span, Public)
```

## `codemaid_rust::collect::Collector::module`
`fn module(&mut self, path: Segs, attrs: &[Attribute], items: &[Item], span: Span, vis: Visibility)` · L63-L81
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__Collector as Collector
  participant codemaid_rust__collect as collect mod
  loop for item in items
    opt let Item::Use(u) = item
      codemaid_rust__collect__Collector->>codemaid_rust__collect: flatten_use(&u.tree, &new(), &uses)
    end
  end
  codemaid_rust__collect__Collector->>codemaid_rust__collect: doc_of(attrs)
  codemaid_rust__collect__Collector->>codemaid_rust__collect: tags_of(attrs)
  loop for item in items
    codemaid_rust__collect__Collector->>codemaid_rust__collect__Collector: item(&path, item)
  end
```

## `codemaid_rust::collect::Collector::skip`
`fn skip(&self, attrs: &[Attribute]) -> bool` · L83-L85
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__Collector as Collector
  participant syn as syn ext
  codemaid_rust__collect__Collector->>syn: Attribute::iter()
```

## `codemaid_rust::collect::Collector::item`
`fn item(&mut self, module: &Segs, item: &Item)` · L87-L301
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__Collector as Collector
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__tidy as tidy mod
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  opt closure
    codemaid_rust__collect__Collector->>codemaid_rust__collect: vis_of(vis)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: span_of(span)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: doc_of(attrs)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: tags_of(attrs)
  end
  alt Item::Mod(m)
    codemaid_rust__collect__Collector->>codemaid_rust__collect__Collector: skip(&m.attrs)
    opt self.skip(&m.attrs)
      Note over codemaid_rust__collect__Collector: return
    end
    opt let Some((_, items)) = &m.content
      codemaid_rust__collect__Collector->>codemaid_rust__collect: span_of(span())
      codemaid_rust__collect__Collector->>codemaid_rust__collect: vis_of(&m.vis)
      codemaid_rust__collect__Collector->>codemaid_rust__collect__Collector: module(path, &m.attrs, items, span_of(), vis_of())
    end
  else Item::Struct(s)
    codemaid_rust__collect__Collector->>codemaid_rust__collect__Collector: skip(&s.attrs)
    opt self.skip(&s.attrs)
      Note over codemaid_rust__collect__Collector: return
    end
    codemaid_rust__collect__Collector->>codemaid_rust__collect: generics_of(&s.generics)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: fields_of(&s.fields)
  else Item::Union(u)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: generics_of(&u.generics)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: fields_of(&Named())
  else Item::Enum(e)
    codemaid_rust__collect__Collector->>codemaid_rust__collect__Collector: skip(&e.attrs)
    opt self.skip(&e.attrs)
      Note over codemaid_rust__collect__Collector: return
    end
    codemaid_rust__collect__Collector->>codemaid_rust__collect: generics_of(&e.generics)
    loop for v in &e.variants
      loop for f in v.fields.iter()
        codemaid_rust__collect__Collector->>codemaid_rust__collect: type_refs(&f.ty, &refs)
      end
      alt Fields::Unnamed(u)
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(u)
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: squeeze(&tokens())
      else Fields::Named(n)
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(n)
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: squeeze(&tokens())
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: clip(&squeeze(), LABEL_MAX)
      end
      codemaid_rust__collect__Collector->>codemaid_rust__collect: span_of(span())
    end
  else Item::Trait(t)
    codemaid_rust__collect__Collector->>codemaid_rust__collect__Collector: skip(&t.attrs)
    opt self.skip(&t.attrs)
      Note over codemaid_rust__collect__Collector: return
    end
    codemaid_rust__collect__Collector->>codemaid_rust__collect: generics_of(&t.generics)
    loop for b in &t.supertraits
      opt let TypeParamBound::Trait(tb) = b
        codemaid_rust__collect__Collector->>codemaid_rust__collect: path_segs(&tb.path)
      end
    end
    loop for ti in &t.items
      alt TraitItem::Fn(f)
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(&f.sig)
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: clip(&tokens(), SIG_MAX)
        codemaid_rust__collect__Collector->>codemaid_rust__collect: sig_refs(&f.sig, &refs)
        alt Some(block)
          codemaid_rust__collect__Collector->>codemaid_rust__collect: params_of(&f.sig)
          codemaid_rust__collect__Collector->>codemaid_rust__collect: span_of(span())
          codemaid_rust__collect__Collector->>codemaid_rust__collect: doc_of(&f.attrs)
          codemaid_rust__collect__Collector->>codemaid_rust__collect: generics_of(&f.sig.generics)
          codemaid_rust__collect__Collector->>codemaid_rust__collect: fn_tags(&f.sig, &f.attrs)
          codemaid_rust__collect__Collector->>codemaid_rust__collect__FlowWalker: FlowWalker::new(params)
        else None
          codemaid_rust__collect__Collector->>codemaid_rust__collect: span_of(span())
        end
      else TraitItem::Type(ty)
        opt via then
          codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(&ty.bounds)
          codemaid_rust__collect__Collector->>codemaid_rust__tidy: squeeze(&tokens())
        end
        codemaid_rust__collect__Collector->>codemaid_rust__collect: span_of(span())
      else TraitItem::Const(k)
        codemaid_rust__collect__Collector->>codemaid_rust__collect: type_refs(&k.ty, &refs)
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(&k.ty)
        codemaid_rust__collect__Collector->>codemaid_rust__collect: span_of(span())
      end
    end
  else Item::Type(t)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: generics_of(&t.generics)
    codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(&t.ty)
    codemaid_rust__collect__Collector->>codemaid_rust__tidy: clip(&_, SIG_MAX)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: type_refs(&t.ty, &it.sig_refs)
  else Item::Fn(f)
    codemaid_rust__collect__Collector->>codemaid_rust__collect__Collector: skip(&f.attrs)
    opt self.skip(&f.attrs)
      Note over codemaid_rust__collect__Collector: return
    end
    codemaid_rust__collect__Collector->>codemaid_rust__collect: generics_of(&f.sig.generics)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: vis_prefix(&f.vis)
    codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(&f.sig)
    codemaid_rust__collect__Collector->>codemaid_rust__tidy: clip(&_, SIG_MAX)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: fn_tags(&f.sig, &_)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: sig_refs(&f.sig, &it.sig_refs)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: params_of(&f.sig)
    codemaid_rust__collect__Collector->>codemaid_rust__collect__FlowWalker: FlowWalker::new(params_of())
  else Item::Const(k)
    codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(&k.ty)
    codemaid_rust__collect__Collector->>codemaid_rust__tidy: clip(&_, SIG_MAX)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: type_refs(&k.ty, &it.sig_refs)
  else Item::Static(s)
    codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(&s.ty)
    codemaid_rust__collect__Collector->>codemaid_rust__tidy: clip(&_, SIG_MAX)
    codemaid_rust__collect__Collector->>codemaid_rust__collect: type_refs(&s.ty, &it.sig_refs)
  else Item::Impl(i)
    codemaid_rust__collect__Collector->>codemaid_rust__collect__Collector: skip(&i.attrs)
    opt self.skip(&i.attrs)
      Note over codemaid_rust__collect__Collector: return
    end
    codemaid_rust__collect__Collector->>codemaid_rust__collect: first_path(&i.self_ty)
    opt via map
      codemaid_rust__collect__Collector->>codemaid_rust__collect: path_segs(p)
      codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(p)
    end
    loop for ii in &i.items
      opt let ImplItem::Fn(f) = ii
        codemaid_rust__collect__Collector->>codemaid_rust__collect__Collector: skip(&f.attrs)
        codemaid_rust__collect__Collector->>codemaid_rust__collect: sig_refs(&f.sig, &refs)
        opt not is_trait_impl
          codemaid_rust__collect__Collector->>codemaid_rust__collect: vis_of(&f.vis)
        end
        codemaid_rust__collect__Collector->>codemaid_rust__collect: span_of(span())
        codemaid_rust__collect__Collector->>codemaid_rust__collect: vis_prefix(&f.vis)
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: tokens(&f.sig)
        codemaid_rust__collect__Collector->>codemaid_rust__tidy: clip(&_, SIG_MAX)
        codemaid_rust__collect__Collector->>codemaid_rust__collect: doc_of(&f.attrs)
        codemaid_rust__collect__Collector->>codemaid_rust__collect: generics_of(&f.sig.generics)
        codemaid_rust__collect__Collector->>codemaid_rust__collect: fn_tags(&f.sig, &f.attrs)
        codemaid_rust__collect__Collector->>codemaid_rust__collect: params_of(&f.sig)
        codemaid_rust__collect__Collector->>codemaid_rust__collect__FlowWalker: FlowWalker::new(params_of())
      end
    end
  end
```

## `codemaid_rust::collect::span_of`
`fn span_of(s: PmSpan) -> Span` · L306-L309
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant proc_macro2 as proc_macro2 ext
  participant codemaid_model__symbol__Span as Span
  codemaid_rust__collect->>proc_macro2: Span::start()
  codemaid_rust__collect->>proc_macro2: Span::end()
  codemaid_rust__collect->>codemaid_model__symbol__Span: Span::new(_, _, _, _)
```

## `codemaid_rust::collect::vis_of`
`fn vis_of(v: &syn::Visibility) -> Visibility` · L311-L318
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__tidy as tidy mod
  opt syn::Visibility::Restricted(r)
    codemaid_rust__collect->>codemaid_rust__tidy: tokens(&r.path)
  end
```

## `codemaid_rust::collect::vis_prefix`
`fn vis_prefix(v: &syn::Visibility) -> String` · L320-L325
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__tidy as tidy mod
  opt v
    codemaid_rust__collect->>codemaid_rust__tidy: tokens(v)
  end
```

## `codemaid_rust::collect::is_test_attr`
`fn is_test_attr(a: &Attribute) -> bool` · L327-L333
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant syn as syn ext
  participant codemaid_rust__tidy as tidy mod
  codemaid_rust__collect->>syn: Attribute::path()
  opt p.is_ident(#quot;test#quot;) || p.segments.last().is_some_and(…
    Note over codemaid_rust__collect: return true
  end
  codemaid_rust__collect->>codemaid_rust__tidy: tokens(&a.meta)
```

## `codemaid_rust::collect::doc_of`
`fn doc_of(attrs: &[Attribute]) -> Option<String>` · L335-L367
> First sentence of the doc comment, at most 160 chars.
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__tidy as tidy mod
  opt text.is_empty()
    Note over codemaid_rust__collect: return None
  end
  codemaid_rust__collect->>codemaid_rust__tidy: clip(first, 160)
```

## `codemaid_rust::collect::tags_of`
`fn tags_of(attrs: &[Attribute]) -> Vec<String>` · L369-L382
> Attributes worth keeping as tags.
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__tidy as tidy mod
  loop for a in attrs
    opt multi || KEEP.iter().any(| k | p.is_ident(k))
      codemaid_rust__collect->>codemaid_rust__tidy: tokens(&a.meta)
      codemaid_rust__collect->>codemaid_rust__tidy: squeeze(&tokens())
      codemaid_rust__collect->>codemaid_rust__tidy: clip(&t, LABEL_MAX)
    end
  end
```

## `codemaid_rust::collect::fn_tags`
`fn fn_tags(sig: &syn::Signature, attrs: &[Attribute]) -> Vec<String>` · L384-L396
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  codemaid_rust__collect->>codemaid_rust__collect: tags_of(attrs)
```

## `codemaid_rust::collect::generics_of`
`fn generics_of(g: &Generics) -> Vec<String>` · L398-L407
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__tidy as tidy mod
  opt via map
    alt GenericParam::Type(t)
      codemaid_rust__collect->>codemaid_rust__tidy: tokens(t)
      codemaid_rust__collect->>codemaid_rust__tidy: squeeze(&tokens())
    else GenericParam::Lifetime(l)
      codemaid_rust__collect->>codemaid_rust__tidy: tokens(l)
      codemaid_rust__collect->>codemaid_rust__tidy: squeeze(&tokens())
    else GenericParam::Const(c)
      codemaid_rust__collect->>codemaid_rust__tidy: tokens(c)
      codemaid_rust__collect->>codemaid_rust__tidy: squeeze(&tokens())
    end
  end
```

## `codemaid_rust::collect::fields_of`
`fn fields_of(fields: &Fields) -> Vec<RawMember>` · L409-L426
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant syn as syn ext
  participant codemaid_rust__tidy as tidy mod
  codemaid_rust__collect->>syn: Fields::iter()
  opt via map
    codemaid_rust__collect->>codemaid_rust__collect: type_refs(&f.ty, &refs)
    codemaid_rust__collect->>codemaid_rust__tidy: tokens(&f.ty)
    codemaid_rust__collect->>codemaid_rust__collect: vis_of(&f.vis)
    codemaid_rust__collect->>codemaid_rust__collect: span_of(span())
  end
```

## `codemaid_rust::collect::type_refs`
`fn type_refs(ty: &Type, out: &mut Vec<Segs>)` · L432-L445
> Every path mentioned in a type, outermost first.
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant syn as syn ext
  codemaid_rust__collect->>syn: Visit::Visit::visit_type(&V(), ty)
```

## `codemaid_rust::collect::sig_refs`
`fn sig_refs(sig: &syn::Signature, out: &mut Vec<Segs>)` · L447-L456
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  loop for input in &sig.inputs
    opt let FnArg::Typed(t) = input
      codemaid_rust__collect->>codemaid_rust__collect: type_refs(&t.ty, out)
    end
  end
  opt let syn::ReturnType::Type(_, ty) = &sig.output
    codemaid_rust__collect->>codemaid_rust__collect: type_refs(ty, out)
  end
```

## `codemaid_rust::collect::first_path`
`fn first_path(ty: &Type) -> Option<Segs>` · L458-L467
> Outermost path of a type, looking through references and parens.
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  alt Type::Path(p)
    codemaid_rust__collect->>codemaid_rust__collect: path_segs(&p.path)
  else Type::Reference(r)
    codemaid_rust__collect->>codemaid_rust__collect: first_path(&r.elem)
  else Type::Paren(p)
    codemaid_rust__collect->>codemaid_rust__collect: first_path(&p.elem)
  else Type::Group(g)
    codemaid_rust__collect->>codemaid_rust__collect: first_path(&g.elem)
  end
```

## `codemaid_rust::collect::params_of`
`fn params_of(sig: &syn::Signature) -> BTreeMap<String, Recv>` · L469-L486
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  loop for input in &sig.inputs
    opt FnArg::Typed(t)
      opt let Pat::Ident(id) = &* t.pat
        codemaid_rust__collect->>codemaid_rust__collect: type_refs(&t.ty, &refs)
      end
    end
  end
```

## `codemaid_rust::collect::flatten_use`
`fn flatten_use(tree: &UseTree, prefix: &mut Segs, out: &mut Vec<RawUse>)` · L488-L521
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  alt UseTree::Path(p)
    codemaid_rust__collect->>codemaid_rust__collect: flatten_use(&p.tree, prefix, out)
  else UseTree::Group(g)
    loop for t in &g.items
      codemaid_rust__collect->>codemaid_rust__collect: flatten_use(t, prefix, out)
    end
  end
```

## `codemaid_rust::collect::FlowWalker::block`
`fn block(&mut self, b: &Block) -> Vec<RawStep>` · L561-L565
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: stmts(&b.stmts, &out)
```

## `codemaid_rust::collect::FlowWalker::stmts`
`fn stmts(&mut self, stmts: &[Stmt], out: &mut Vec<RawStep>)` · L567-L587
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  loop for s in stmts
    alt Stmt::Local(l)
      opt let Some(init) = &l.init
        codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&init.expr, out)
        opt let Some((_, diverge)) = &init.diverge
          codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub(diverge)
        end
      end
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: bind(&l.pat, map())
    else Stmt::Expr(e, _)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(e, out)
    else Stmt::Macro(m)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: mac(&m.mac, out)
    end
  end
```

## `codemaid_rust::collect::FlowWalker::bind`
`fn bind(&mut self, pat: &Pat, init: Option<&Expr>)` · L589-L617
> Record the type of a `let` binding when it is evident.
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  participant codemaid_rust__collect as collect mod
  alt Pat::Type(pt)
    alt Pat::Ident(id)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: type_refs(&pt.ty, &refs)
    else _
      Note over codemaid_rust__collect__FlowWalker: return
    end
  else _
    Note over codemaid_rust__collect__FlowWalker: return
  end
```

## `codemaid_rust::collect::FlowWalker::sub`
`fn sub(&mut self, e: &Expr) -> Vec<RawStep>` · L619-L623
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(e, &out)
```

## `codemaid_rust::collect::FlowWalker::sub_block`
`fn sub_block(&mut self, b: &Block) -> Vec<RawStep>` · L625-L630
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: block(b)
```

## `codemaid_rust::collect::FlowWalker::expr`
`fn expr(&mut self, e: &Expr, out: &mut Vec<RawStep>)` · L632-L835
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__tidy as tidy mod
  alt Expr::Call(c)
    loop for a in &c.args
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: arg(a, out, &deferred)
    end
    alt Expr::Path(p)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: path_segs(&p.path)
      opt !name.starts_with(| ch: char | ch.is_uppercase())
        codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: call(Path(), &shown, &c.args, Function, span())
        codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: flush_deferred(&name, deferred, out)
        Note over codemaid_rust__collect__FlowWalker: return
      end
    else other
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(other, out)
    end
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: flush_deferred(#quot;#quot;, deferred, out)
  else Expr::MethodCall(m)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&m.receiver, out)
    loop for a in &m.args
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: arg(a, out, &deferred)
    end
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: recv(&m.receiver)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: call(_, &name, &m.args, Method, span())
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: flush_deferred(&name, deferred, out)
  else Expr::Await(a)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&a.base, out)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: is_call(&a.base)
    opt is_call(&a.base)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: last_call_mut(out)
    end
  else Expr::Try(t)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&t.expr, out)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: is_call(&t.expr)
    opt is_call(&t.expr)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: last_call_mut(out)
    end
  else Expr::If(i)
    loop while let Some(ifx) = cur.take()
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: cond_label(&ifx.cond)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: cond(&ifx.cond, &cond_steps)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub_block(&ifx.then_branch)
      alt Some(Expr::Block(b))
        codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub_block(&b.block)
      else Some(other)
        codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub(other)
      end
    end
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: push_arms(arms, out)
  else Expr::Match(m)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&m.expr, out)
    opt via map
      codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: tokens(&a.pat)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: squeeze(&tokens())
      opt let Some((_, g)) = &a.guard
        codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: tokens(g)
        codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: squeeze(&tokens())
      end
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub(&a.body)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: clip(&label, LABEL_MAX)
    end
  else Expr::ForLoop(f)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&f.expr, out)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: tokens(&f.pat)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: squeeze(&tokens())
    codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: tokens(&f.expr)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: squeeze(&tokens())
    codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: clip(&_, LABEL_MAX)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub_block(&f.body)
  else Expr::While(w)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect: cond_label(&w.cond)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: clip(&_, LABEL_MAX)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: cond(&w.cond, &body)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub_block(&w.body)
  else Expr::Loop(l)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub_block(&l.body)
  else Expr::Block(b)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: stmts(&b.block.stmts, out)
  else Expr::Unsafe(b)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: stmts(&b.block.stmts, out)
  else Expr::Async(b)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: stmts(&b.block.stmts, out)
  else Expr::TryBlock(b)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: stmts(&b.block.stmts, out)
  else Expr::Const(b)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: stmts(&b.block.stmts, out)
  else Expr::Closure(c)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub(&c.body)
  else Expr::Return(r)
    opt let Some(e) = &r.expr
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(e, out)
    end
    opt via map_or_else
      codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: tokens(e)
      codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: squeeze(&tokens())
      codemaid_rust__collect__FlowWalker->>codemaid_rust__tidy: clip(&_, LABEL_MAX)
    end
  else Expr::Macro(m)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: mac(&m.mac, out)
  else Expr::Binary(b)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&b.left, out)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&b.right, out)
  else Expr::Assign(a)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&a.right, out)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&a.left, out)
  else Expr::Unary(u)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&u.expr, out)
  else Expr::Paren(p)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&p.expr, out)
  else Expr::Group(g)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&g.expr, out)
  else Expr::Reference(r)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&r.expr, out)
  else Expr::Field(f)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&f.base, out)
  else Expr::Index(i)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&i.expr, out)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&i.index, out)
  else Expr::Cast(c)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&c.expr, out)
  else Expr::Let(l)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&l.expr, out)
  else Expr::Tuple(t)
    loop each via for_each
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(e, out)
    end
  else Expr::Array(a)
    loop each via for_each
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(e, out)
    end
  else Expr::Repeat(r)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&r.expr, out)
  else Expr::Range(r)
    opt let Some(s) = &r.start
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(s, out)
    end
    opt let Some(e) = &r.end
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(e, out)
    end
  else Expr::Struct(s)
    loop for f in &s.fields
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(&f.expr, out)
    end
    opt let Some(rest) = &s.rest
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(rest, out)
    end
  else Expr::Break(b)
    opt let Some(e) = &b.expr
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(e, out)
    end
  else Expr::Yield(y)
    opt let Some(e) = &y.expr
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(e, out)
    end
  end
```

## `codemaid_rust::collect::FlowWalker::cond`
`fn cond(&mut self, cond: &Expr, out: &mut Vec<RawStep>)` · L837-L843
> Condition of `if` / `while`: `let` scrutinee or boolean expression.
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(cond, out)
  opt let Expr::Let(l) = cond
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: bind(&l.pat, Some())
  end
```

## `codemaid_rust::collect::FlowWalker::arg`
`fn arg(&mut self, a: &Expr, out: &mut Vec<RawStep>, deferred: &mut Vec<Vec<RawStep>>)` · L845-L865
> Walk a call argument.
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  alt Expr::Closure(c)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub(&c.body)
  else Expr::Async(b)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: sub_block(&b.block)
  else other
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(other, out)
  end
```

## `codemaid_rust::collect::FlowWalker::mac`
`fn mac(&mut self, m: &syn::Macro, out: &mut Vec<RawStep>)` · L880-L892
> Calls inside macro arguments (`vec![f()]`, `assert!(g())`, `format!("{}", h())`).
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  participant syn as syn ext
  codemaid_rust__collect__FlowWalker->>syn: Macro::parse_body_with(parser)
  alt let Ok(args) = m.parse_body_with(parser)
    loop for a in &args
      codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: expr(a, out)
    end
  else if let Ok(stmts) = m.parse_body_with(Block::parse_withi…
    codemaid_rust__collect__FlowWalker->>syn: Macro::parse_body_with(parse_within)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: stmts(&stmts, out)
  end
```

## `codemaid_rust::collect::FlowWalker::recv`
`fn recv(&self, e: &Expr) -> Recv` · L894-L910
```mermaid
sequenceDiagram
  participant codemaid_rust__collect__FlowWalker as FlowWalker
  alt Expr::Paren(p)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: recv(&p.expr)
  else Expr::Reference(r)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: recv(&r.expr)
  else Expr::Unary(u)
    codemaid_rust__collect__FlowWalker->>codemaid_rust__collect__FlowWalker: recv(&u.expr)
  end
```

## `codemaid_rust::collect::call`
`fn call(callee: Callee, name: &str, args: &Punctuated<Expr, syn::Token![,]>, kind: CallKind, span: PmSpan) -> RawStep` · L913-L917
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant syn as syn ext
  participant codemaid_rust__tidy as tidy mod
  participant proc_macro2 as proc_macro2 ext
  codemaid_rust__collect->>syn: Punctuated::iter()
  codemaid_rust__collect->>codemaid_rust__tidy: clip(&_, LABEL_MAX)
  codemaid_rust__collect->>proc_macro2: Span::start()
```

## `codemaid_rust::collect::arg_sketch`
`fn arg_sketch(e: &Expr) -> String` · L919-L939
> A compact stand-in for an argument: identifiers and short literals are kept, everything else becomes `_`.
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__tidy as tidy mod
  alt Expr::Lit(l)
    codemaid_rust__collect->>codemaid_rust__tidy: tokens(l)
    codemaid_rust__collect->>codemaid_rust__tidy: clip(&tokens(), 14)
  else Expr::Reference(r)
    codemaid_rust__collect->>codemaid_rust__collect: arg_sketch(&r.expr)
  else Expr::Field(f)
    alt syn::Member::Named(n)
      codemaid_rust__collect->>codemaid_rust__collect: arg_sketch(&f.base)
    else syn::Member::Unnamed(i)
      codemaid_rust__collect->>codemaid_rust__collect: arg_sketch(&f.base)
    end
  end
```

## `codemaid_rust::collect::cond_label`
`fn cond_label(cond: &Expr) -> String` · L941-L947
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  participant codemaid_rust__tidy as tidy mod
  alt Expr::Let(l)
    codemaid_rust__collect->>codemaid_rust__tidy: tokens(&l.pat)
    codemaid_rust__collect->>codemaid_rust__tidy: squeeze(&tokens())
    codemaid_rust__collect->>codemaid_rust__tidy: tokens(&l.expr)
    codemaid_rust__collect->>codemaid_rust__tidy: squeeze(&tokens())
  else other
    codemaid_rust__collect->>codemaid_rust__tidy: tokens(other)
    codemaid_rust__collect->>codemaid_rust__tidy: squeeze(&tokens())
  end
  codemaid_rust__collect->>codemaid_rust__tidy: clip(&_, LABEL_MAX)
```

## `codemaid_rust::collect::is_call`
`fn is_call(e: &Expr) -> bool` · L963-L971
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  alt Expr::Paren(p)
    codemaid_rust__collect->>codemaid_rust__collect: is_call(&p.expr)
  else Expr::Await(a)
    codemaid_rust__collect->>codemaid_rust__collect: is_call(&a.base)
  else Expr::Try(t)
    codemaid_rust__collect->>codemaid_rust__collect: is_call(&t.expr)
  end
```

## `codemaid_rust::collect::constructed_type`
`fn constructed_type(e: &Expr) -> Option<Segs>` · L979-L997
> `Foo::new(..)`, `Foo { ..
```mermaid
sequenceDiagram
  participant codemaid_rust__collect as collect mod
  alt Expr::Call(c)
    opt Expr::Path(p) if p.path.segments.len()>= 2
      codemaid_rust__collect->>codemaid_rust__collect: path_segs(&p.path)
    end
  else Expr::Struct(s)
    codemaid_rust__collect->>codemaid_rust__collect: path_segs(&s.path)
  else Expr::Try(t)
    codemaid_rust__collect->>codemaid_rust__collect: constructed_type(&t.expr)
  else Expr::Await(a)
    codemaid_rust__collect->>codemaid_rust__collect: constructed_type(&a.base)
  else Expr::Paren(p)
    codemaid_rust__collect->>codemaid_rust__collect: constructed_type(&p.expr)
  else Expr::Reference(r)
    codemaid_rust__collect->>codemaid_rust__collect: constructed_type(&r.expr)
  end
```
