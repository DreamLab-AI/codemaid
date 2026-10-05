---
sealmap: 2
source: crates/sealmap-rust/src/fingerprint.rs
module: "sym:cargo sealmap_rust . fingerprint/"
language: rust
source_hash: blake3:95bb4aeb313f47f054ab564086e2064b4ceaed5d302ccc5c18ec8679a898f9f6
lines: 337
fragments: 16
---
# `sym:cargo sealmap_rust . fingerprint/` · crates/sealmap-rust/src/fingerprint.rs
> Feeding syn nodes to the shared fingerprinter.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_rust__fingerprint___tCanon["Canon"] {
    <<struct>>
    +VisitMut::visit_arm_mut(&mut self, a: &mut syn::Arm)
    +VisitMut::visit_block_mut(&mut self, b: &mut Block)
    +VisitMut::visit_expr_closure_mut(&mut self, c: &mut syn::ExprClosure)
    +VisitMut::visit_macro_mut(&mut self, m: &mut syn::Macro)
    +VisitMut::visit_pat_or_mut(&mut self, p: &mut syn::PatOr)
    +VisitMut::visit_use_group_mut(&mut self, g: &mut syn::UseGroup)
  }
  class sealmap_rust__fingerprint["sealmap_rust::fingerprint"] {
    <<module>>
    -const IGNORED_ATTRS: &[&str]
    -const IGNORED_TOOLS: &[&str]
    -const KEYWORDS: &[&str]
    ~attrs(crate)
    ~canonical(crate) T
    ~feed(crate)
    ~feed_canonical(crate)
    -feed_stream(fp: &mut Fingerprinter, ts: TokenStream)
    -feed_trees(fp: &mut Fingerprinter, ts: TokenStream, delim: Delimiter, arguments: bool)
    -ignored_attr_len(trees: &[TokenTree]) Option#lt;usize#gt;
    -ignored_path(p: &syn::Path) bool
    -is_argument_list(prev2: Option#lt;&TokenTree#gt;, prev: Option#lt;&TokenTree#gt;) bool
    -is_punct(t: &TokenTree, c: char) bool
    -is_trailing(next: Option#lt;&TokenTree#gt;, delim: Delimiter, commas: usize, in_where: bool, arguments: bool) bool
    -unblock(e: &Expr) Option#lt;Expr#gt;
    ~unparsable(crate) #40;Fingerprint, Fingerprint#41;
  }
  class sealmap_extract__fingerprint___tFingerprinter["Fingerprinter"] {
    <<struct in crates/sealmap-extract/src/fingerprint.rs>>
  }
  class sealmap_model__hash___tFingerprint["Fingerprint"] {
    <<struct in crates/sealmap-model/src/hash.rs>>
  }
  class _proc_macro2__Delimiter["proc_macro2::Delimiter"] {
    <<external>>
  }
  class _proc_macro2__TokenStream["proc_macro2::TokenStream"] {
    <<external>>
  }
  class _proc_macro2__TokenTree["proc_macro2::TokenTree"] {
    <<external>>
  }
  class _quote__ToTokens["quote::ToTokens"] {
    <<external>>
  }
  class _syn__Attribute["syn::Attribute"] {
    <<external>>
  }
  class _syn__Expr["syn::Expr"] {
    <<external>>
  }
  class _syn__Path["syn::Path"] {
    <<external>>
  }
  class _syn__Arm["syn::Arm"] {
    <<external>>
  }
  class _syn__Block["syn::Block"] {
    <<external>>
  }
  class _syn__ExprClosure["syn::ExprClosure"] {
    <<external>>
  }
  class _syn__Macro["syn::Macro"] {
    <<external>>
  }
  class _syn__PatOr["syn::PatOr"] {
    <<external>>
  }
  class _syn__UseGroup["syn::UseGroup"] {
    <<external>>
  }
  class _syn__visit_mut__VisitMut["syn::visit_mut::VisitMut"] {
    <<external>>
  }
  sealmap_rust__fingerprint ..> sealmap_extract__fingerprint___tFingerprinter
  sealmap_rust__fingerprint ..> sealmap_model__hash___tFingerprint
  sealmap_rust__fingerprint ..> sealmap_rust__fingerprint___tCanon
  sealmap_rust__fingerprint ..> _proc_macro2__Delimiter
  sealmap_rust__fingerprint ..> _proc_macro2__TokenStream
  sealmap_rust__fingerprint ..> _proc_macro2__TokenTree
  sealmap_rust__fingerprint ..> _quote__ToTokens
  sealmap_rust__fingerprint ..> _syn__Attribute
  sealmap_rust__fingerprint ..> _syn__Expr
  sealmap_rust__fingerprint ..> _syn__Path
  sealmap_rust__fingerprint___tCanon ..> _syn__Arm
  sealmap_rust__fingerprint___tCanon ..> _syn__Block
  sealmap_rust__fingerprint___tCanon ..> _syn__ExprClosure
  sealmap_rust__fingerprint___tCanon ..> _syn__Macro
  sealmap_rust__fingerprint___tCanon ..> _syn__PatOr
  sealmap_rust__fingerprint___tCanon ..> _syn__UseGroup
  sealmap_rust__fingerprint___tCanon ..|> _syn__visit_mut__VisitMut
```

## `sym:cargo sealmap_rust . fingerprint/feed().`
`pub(crate) fn feed(fp: &mut Fingerprinter, node: &impl ToTokens)` · L44-L47
> Feed a syn node's tokens.
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint as fingerprint mod
  participant _quote as quote ext
  sealmap_rust__fingerprint->>_quote: ToTokens::to_token_stream()
  sealmap_rust__fingerprint->>sealmap_rust__fingerprint: feed_stream(fp, to_token_stream())
```

## `sym:cargo sealmap_rust . fingerprint/feed_canonical().`
`pub(crate) fn feed_canonical<T: Clone + ToTokens>(fp: &mut Fingerprinter, node: &T, visit: fn(&mut Canon, &mut T))` · L49-L52
> Feed a node after undoing formatter rewrites on a copy of it.
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint as fingerprint mod
  sealmap_rust__fingerprint->>sealmap_rust__fingerprint: canonical(node, visit)
  sealmap_rust__fingerprint->>sealmap_rust__fingerprint: feed(fp, &canonical())
```

## `sym:cargo sealmap_rust . fingerprint/Canon#[VisitMut]visit_expr_closure_mut().`
`fn visit_expr_closure_mut(&mut self, c: &mut syn::ExprClosure)` · L80-L88
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint___tCanon as Canon
  participant _syn as syn ext
  participant sealmap_rust__fingerprint as fingerprint mod
  sealmap_rust__fingerprint___tCanon->>_syn: visit_mut::visit_expr_closure_mut(self, c)
  opt matches!(c.output, syn::ReturnType::Default)
    sealmap_rust__fingerprint___tCanon->>sealmap_rust__fingerprint: unblock(&c.body)
  end
```

## `sym:cargo sealmap_rust . fingerprint/Canon#[VisitMut]visit_arm_mut().`
`fn visit_arm_mut(&mut self, a: &mut syn::Arm)` · L90-L96
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint___tCanon as Canon
  participant _syn as syn ext
  participant sealmap_rust__fingerprint as fingerprint mod
  sealmap_rust__fingerprint___tCanon->>_syn: visit_mut::visit_arm_mut(self, a)
  sealmap_rust__fingerprint___tCanon->>sealmap_rust__fingerprint: unblock(&a.body)
```

## `sym:cargo sealmap_rust . fingerprint/Canon#[VisitMut]visit_pat_or_mut().`
`fn visit_pat_or_mut(&mut self, p: &mut syn::PatOr)` · L98-L101
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint___tCanon as Canon
  participant _syn as syn ext
  sealmap_rust__fingerprint___tCanon->>_syn: visit_mut::visit_pat_or_mut(self, p)
```

## `sym:cargo sealmap_rust . fingerprint/Canon#[VisitMut]visit_block_mut().`
`fn visit_block_mut(&mut self, b: &mut Block)` · L103-L109
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint___tCanon as Canon
  participant _syn as syn ext
  sealmap_rust__fingerprint___tCanon->>_syn: visit_mut::visit_block_mut(self, b)
```

## `sym:cargo sealmap_rust . fingerprint/Canon#[VisitMut]visit_macro_mut().`
`fn visit_macro_mut(&mut self, m: &mut syn::Macro)` · L111-L124
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint___tCanon as Canon
  participant _syn as syn ext
  participant _proc_macro2 as proc_macro2 ext
  sealmap_rust__fingerprint___tCanon->>_syn: Macro::parse_body_with(parser)
  opt let Ok(args) = m.parse_body_with(parser)
    sealmap_rust__fingerprint___tCanon->>_proc_macro2: TokenStream::new()
    sealmap_rust__fingerprint___tCanon->>_syn: Macro::into_iter()
  end
```

## `sym:cargo sealmap_rust . fingerprint/Canon#[VisitMut]visit_use_group_mut().`
`fn visit_use_group_mut(&mut self, g: &mut syn::UseGroup)` · L126-L131
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint___tCanon as Canon
  participant _syn as syn ext
  sealmap_rust__fingerprint___tCanon->>_syn: visit_mut::visit_use_group_mut(self, g)
```

## `sym:cargo sealmap_rust . fingerprint/attrs().`
`pub(crate) fn attrs(fp: &mut Fingerprinter, attrs: &[Attribute])` · L134-L142
> Feed every attribute that matters.
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint as fingerprint mod
  participant sealmap_extract__fingerprint___tFingerprinter as Fingerprinter
  sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: section(#quot;attrs#quot;)
  loop for a in attrs
    sealmap_rust__fingerprint->>sealmap_rust__fingerprint: ignored_path(path())
    opt !ignored_path(a.path())
      sealmap_rust__fingerprint->>sealmap_rust__fingerprint: feed(fp, a)
    end
  end
```

## `sym:cargo sealmap_rust . fingerprint/unparsable().`
`pub(crate) fn unparsable(name: &str, text: &str) -> (Fingerprint, Fingerprint)` · L144-L154
> The fingerprints of a module whose file did not parse: its name in the contract, its raw text as one literal in the body, so any edit shows.
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint as fingerprint mod
  participant sealmap_extract__fingerprint___tFingerprinter as Fingerprinter
  sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: Fingerprinter::sig()
  sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: section(#quot;unparsable#quot;)
  sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: ident(name)
  sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: Fingerprinter::body()
  sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: section(#quot;unparsable#quot;)
  sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: literal(text)
  sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: finish()
  sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: finish()
```

## `sym:cargo sealmap_rust . fingerprint/feed_stream().`
`fn feed_stream(fp: &mut Fingerprinter, ts: TokenStream)` · L171-L173
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint as fingerprint mod
  sealmap_rust__fingerprint->>sealmap_rust__fingerprint: feed_trees(fp, ts, None, false)
```

## `sym:cargo sealmap_rust . fingerprint/is_argument_list().`
`fn is_argument_list(prev2: Option<&TokenTree>, prev: Option<&TokenTree>) -> bool` · L175-L185
> Does a parenthesised group after `prev` (and `prev2` before it) hold an argument or parameter list rather than a tuple?
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint as fingerprint mod
  participant _proc_macro2 as proc_macro2 ext
  alt Some(TokenTree::Group(g))
    sealmap_rust__fingerprint->>_proc_macro2: TokenTree::delimiter()
  else Some(t) if is_punct(t, '>')
    opt via is_some_and
      sealmap_rust__fingerprint->>sealmap_rust__fingerprint: is_punct(p, '-')
      sealmap_rust__fingerprint->>sealmap_rust__fingerprint: is_punct(p, '=')
    end
  end
```

## `sym:cargo sealmap_rust . fingerprint/feed_trees().`
`fn feed_trees(fp: &mut Fingerprinter, ts: TokenStream, delim: Delimiter, arguments: bool)` · L187-L230
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint as fingerprint mod
  participant _proc_macro2 as proc_macro2 ext
  participant sealmap_extract__fingerprint___tFingerprinter as Fingerprinter
  sealmap_rust__fingerprint->>_proc_macro2: TokenStream::into_iter()
  loop each via filter
    sealmap_rust__fingerprint->>sealmap_rust__fingerprint: is_punct(t, ',')
  end
  loop while i<trees.len()
    sealmap_rust__fingerprint->>sealmap_rust__fingerprint: ignored_attr_len(&_)
    alt TokenTree::Ident(id)
      sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: ident(&text)
    else TokenTree::Punct(p)
      sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: punct(as_char())
    else TokenTree::Literal(l)
      sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: literal(&to_string())
    else TokenTree::Group(g)
      sealmap_rust__fingerprint->>sealmap_rust__fingerprint: is_argument_list(and_then(), and_then())
      sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: open(d)
      sealmap_rust__fingerprint->>sealmap_rust__fingerprint: feed_trees(fp, stream(), delimiter(), args)
      sealmap_rust__fingerprint->>sealmap_extract__fingerprint___tFingerprinter: close(d)
    end
  end
```

## `sym:cargo sealmap_rust . fingerprint/is_trailing().`
`fn is_trailing(next: Option<&TokenTree>, delim: Delimiter, commas: usize, in_where: bool, arguments: bool) -> bool` · L236-L246
> Is a comma followed by `next` (in a group delimited by `delim` holding `commas` top-level commas) a trailing comma that layout alone decides?
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint as fingerprint mod
  participant _proc_macro2 as proc_macro2 ext
  alt Some(TokenTree::Group(g)) if in_where
    sealmap_rust__fingerprint->>_proc_macro2: TokenTree::delimiter()
  else Some(t) if in_where
    sealmap_rust__fingerprint->>sealmap_rust__fingerprint: is_punct(t, '#59;')
  end
```

## `sym:cargo sealmap_rust . fingerprint/ignored_attr_len().`
`fn ignored_attr_len(trees: &[TokenTree]) -> Option<usize>` · L248-L270
> If `trees` starts with an ignored attribute (`#[doc = ..]`, `#![allow(..)]`, `#[clippy::x]`), the number of trees it spans.
```mermaid
sequenceDiagram
  participant sealmap_rust__fingerprint as fingerprint mod
  participant _proc_macro2 as proc_macro2 ext
  sealmap_rust__fingerprint->>_proc_macro2: TokenTree::first()?
  opt let-else
    Note over sealmap_rust__fingerprint: return None
  end
  opt hash.as_char() != '#35;'
    Note over sealmap_rust__fingerprint: return None
  end
  sealmap_rust__fingerprint->>_proc_macro2: TokenTree::get(at)?
  opt let-else
    Note over sealmap_rust__fingerprint: return None
  end
  opt g.delimiter() != Delimiter::Bracket
    Note over sealmap_rust__fingerprint: return None
  end
  opt let-else
    Note over sealmap_rust__fingerprint: return None
  end
```
