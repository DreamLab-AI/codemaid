---
sealmap: 1
source: crates/sealmap-rust/src/resolve.rs
module: sealmap_rust::resolve
language: rust
source_hash: blake3:0809b201f1cedd5692dc1e3dd94e51244775e6af96264c9bdadcde51e4625fe4
lines: 845
fragments: 18
---
# `sealmap_rust::resolve` · crates/sealmap-rust/src/resolve.rs
> Pass 2: resolve raw paths against the whole workspace and build the model.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_rust__resolve__FlowCtx["FlowCtx#lt;'a#gt;"] {
    <<struct>>
    -module: &'a str
    -self_ty: Option#lt;&'a SymbolId#gt;
  }
  class sealmap_rust__resolve__Resolver["Resolver"] {
    <<struct>>
    -crates: BTreeSet#lt;String#gt;
    -items: BTreeMap#lt;String, BTreeMap#lt;String, SymbolId#gt;#gt;
    -uses: BTreeMap#lt;String, Vec#lt;RawUse#gt;#gt;
    -internal: BTreeSet#lt;SymbolId#gt;
    -fields: BTreeMap#lt;SymbolId, #40;String, BTreeMap#lt;String, Vec#lt;Segs#gt;#gt;#41;#gt;
    -methods: BTreeMap#lt;SymbolId, BTreeMap#lt;String, SymbolId#gt;#gt;
    -by_name: BTreeMap#lt;String, BTreeSet#lt;SymbolId#gt;#gt;
    -impls: BTreeMap#lt;SymbolId, BTreeSet#lt;SymbolId#gt;#gt;
    -trait_methods: BTreeMap#lt;SymbolId, BTreeSet#lt;String#gt;#gt;
    -globs: BTreeMap#lt;String, Vec#lt;String#gt;#gt;
    -add_uses(&self, cb: &mut Codebase, from: &SymbolId, module: &str, raw: &[Segs], self_ty: Option#lt;&SymbolId#gt;)
    -arms(&self, ctx: &FlowCtx#lt;'_#gt;, arms: &[#40;String, Vec#lt;RawStep#gt;#41;], opts: &RustOptions) Vec#lt;Arm#gt;
    -by_name_only(&self, name: &str) #40;SymbolId, Confidence#41;
    -call(&self, ctx: &FlowCtx#lt;'_#gt;, c: &RawCall, opts: &RustOptions) Option#lt;Call#gt;
    -flow(&self, ctx: &FlowCtx#lt;'_#gt;, raw: &[RawStep], opts: &RustOptions) Option#lt;Flow#gt;
    -glob(&self, module: &str, name: &str) Option#lt;SymbolId#gt;
    -local(&self, module: &str, name: &str) bool
    -member(&self, module: &str, m: &RawMember, owner: Option#lt;&SymbolId#gt;) Member
    -method(&self, ctx: &FlowCtx#lt;'_#gt;, recv: &Recv, name: &str) Option#lt;#40;SymbolId, Confidence#41;#gt;
    -method_symbol(&self, id: &SymbolId, parent: &SymbolId, m: &RawFn, f: &RawFile, ctx: &FlowCtx#lt;'_#gt;, opts: &RustOptions, cb: &mut Codebase,) Symbol
    -new(files: &[RawFile]) Self
    -receiver_type(&self, module: &str, refs: &[Segs], self_ty: Option#lt;&SymbolId#gt;) Option#lt;SymbolId#gt;
    -refs(&self, module: &str, raw: &[Segs], self_ty: Option#lt;&SymbolId#gt;) Vec#lt;SymbolId#gt;
    -resolve(&self, module: &str, segs: &[String], self_ty: Option#lt;&SymbolId#gt;) #40;SymbolId, Confidence#41;
    -resolve_internal(&self, module: &str, segs: &[String], self_ty: Option#lt;&SymbolId#gt;) Option#lt;SymbolId#gt;
    -steps(&self, ctx: &FlowCtx#lt;'_#gt;, raw: &[RawStep], opts: &RustOptions) Vec#lt;Step#gt;
    -walk(&self, module: &str, segs: &[String], self_ty: Option#lt;&SymbolId#gt;, depth: u8, use_globs: bool,) Option#lt;SymbolId#gt;
  }
  class sealmap_rust__resolve["sealmap_rust::resolve"] {
    <<module>>
    -const COMMON_METHODS: &[&str]
    -const PRELUDE: &[&str]
    -const STD_ROOTS: &[&str]
    ~build(crate) #40;Codebase, Vec#lt;Diagnostic#gt;#41;
    -cb_has(r: &Resolver, id: &SymbolId) bool
    -is_generic_param(s: &str) bool
    -keep_ref(id: &SymbolId) bool
  }
  class sealmap_model__codebase__Codebase["Codebase"] {
    <<struct in crates/sealmap-model/src/codebase.rs>>
  }
  class sealmap_model__symbol__SymbolId["SymbolId"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_rust__Diagnostic["Diagnostic"] {
    <<struct in crates/sealmap-rust/src/lib.rs>>
  }
  class sealmap_rust__RustOptions["RustOptions"] {
    <<struct in crates/sealmap-rust/src/lib.rs>>
  }
  class sealmap_rust__raw__RawFile["RawFile"] {
    <<struct in crates/sealmap-rust/src/raw.rs>>
  }
  class sealmap_model__flow__Arm["Arm"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__flow__Call["Call"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__flow__Flow["Flow"] {
    <<struct in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__flow__Step["Step"] {
    <<enum in crates/sealmap-model/src/flow.rs>>
  }
  class sealmap_model__symbol__Confidence["Confidence"] {
    <<enum in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol__Member["Member"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_model__symbol__Symbol["Symbol"] {
    <<struct in crates/sealmap-model/src/symbol.rs>>
  }
  class sealmap_rust__raw__RawCall["RawCall"] {
    <<struct in crates/sealmap-rust/src/raw.rs>>
  }
  class sealmap_rust__raw__RawFn["RawFn"] {
    <<struct in crates/sealmap-rust/src/raw.rs>>
  }
  class sealmap_rust__raw__RawMember["RawMember"] {
    <<struct in crates/sealmap-rust/src/raw.rs>>
  }
  class sealmap_rust__raw__RawStep["RawStep"] {
    <<enum in crates/sealmap-rust/src/raw.rs>>
  }
  class sealmap_rust__raw__RawUse["RawUse"] {
    <<struct in crates/sealmap-rust/src/raw.rs>>
  }
  class sealmap_rust__raw__Recv["Recv"] {
    <<enum in crates/sealmap-rust/src/raw.rs>>
  }
  class sealmap_rust__raw__Segs["Segs"] {
    <<type in crates/sealmap-rust/src/raw.rs>>
  }
  sealmap_rust__resolve ..> sealmap_model__codebase__Codebase
  sealmap_rust__resolve ..> sealmap_model__symbol__SymbolId
  sealmap_rust__resolve ..> sealmap_rust__Diagnostic
  sealmap_rust__resolve ..> sealmap_rust__RustOptions
  sealmap_rust__resolve ..> sealmap_rust__raw__RawFile
  sealmap_rust__resolve ..> sealmap_rust__resolve__Resolver
  sealmap_rust__resolve__FlowCtx o-- sealmap_model__symbol__SymbolId : self_ty
  sealmap_rust__resolve__Resolver ..> sealmap_model__codebase__Codebase
  sealmap_rust__resolve__Resolver ..> sealmap_model__flow__Arm
  sealmap_rust__resolve__Resolver ..> sealmap_model__flow__Call
  sealmap_rust__resolve__Resolver ..> sealmap_model__flow__Flow
  sealmap_rust__resolve__Resolver ..> sealmap_model__flow__Step
  sealmap_rust__resolve__Resolver ..> sealmap_model__symbol__Confidence
  sealmap_rust__resolve__Resolver ..> sealmap_model__symbol__Member
  sealmap_rust__resolve__Resolver ..> sealmap_model__symbol__Symbol
  sealmap_rust__resolve__Resolver o-- sealmap_model__symbol__SymbolId : items, internal, fields, methods, by_name, impls, trait_methods
  sealmap_rust__resolve__Resolver ..> sealmap_rust__RustOptions
  sealmap_rust__resolve__Resolver ..> sealmap_rust__raw__RawCall
  sealmap_rust__resolve__Resolver ..> sealmap_rust__raw__RawFile
  sealmap_rust__resolve__Resolver ..> sealmap_rust__raw__RawFn
  sealmap_rust__resolve__Resolver ..> sealmap_rust__raw__RawMember
  sealmap_rust__resolve__Resolver ..> sealmap_rust__raw__RawStep
  sealmap_rust__resolve__Resolver o-- sealmap_rust__raw__RawUse : uses
  sealmap_rust__resolve__Resolver ..> sealmap_rust__raw__Recv
  sealmap_rust__resolve__Resolver o-- sealmap_rust__raw__Segs : fields
  sealmap_rust__resolve__Resolver ..> sealmap_rust__resolve__FlowCtx
```

## `sealmap_rust::resolve::build`
`pub(crate) fn build(name: &str, files: Vec<RawFile>, opts: &RustOptions) -> (Codebase, Vec<Diagnostic>)` · L211-L330
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve as resolve mod
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_model__codebase__Codebase as Codebase
  participant sealmap_model__symbol__SymbolId as SymbolId
  participant sealmap_model__symbol__Symbol as Symbol
  participant sealmap_model__symbol__Relation as Relation
  participant sealmap_model__flow__Flow as Flow
  sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: Resolver::new(&files)
  sealmap_rust__resolve->>sealmap_model__codebase__Codebase: Codebase::new(name)
  loop for f in &files
    sealmap_rust__resolve->>sealmap_model__symbol__SymbolId: SymbolId::new(join())
    sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_file(_)
    loop for m in &f.modules
      sealmap_rust__resolve->>sealmap_model__symbol__SymbolId: SymbolId::new(join())
      sealmap_rust__resolve->>sealmap_model__symbol__Symbol: Symbol::new(clone(), unwrap_or_default(), Module, clone…
      opt via then
        sealmap_rust__resolve->>sealmap_model__symbol__SymbolId: SymbolId::new(join())
      end
      sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_symbol(s)
      loop for u in &m.uses
        opt u.alias == #quot;*#quot;
          sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: resolve_internal(&module, &u.target, None)
          opt let Some(t) = r.resolve_internal(&module, &u.target,…
            sealmap_rust__resolve->>sealmap_model__symbol__Relation: Relation::new(clone(), t, Imports, Exact)
            sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_relation(new())
          end
        end
        sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: resolve(&module, &u.target, None)
        sealmap_rust__resolve->>sealmap_rust__resolve: keep_ref(&t)
        opt keep_ref(&t)
          sealmap_rust__resolve->>sealmap_model__symbol__Relation: Relation::new(clone(), t, Imports, c)
          sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_relation(new())
        end
      end
    end
    loop for it in &f.items
      sealmap_rust__resolve->>sealmap_model__symbol__SymbolId: SymbolId::new(_)
      sealmap_rust__resolve->>sealmap_model__symbol__Symbol: Symbol::new(clone(), &it.name, it.kind, clone())
      opt via map
        sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: member(&module, m, Some())
      end
      loop for m in &s.members
        loop for t in &m.refs
          sealmap_rust__resolve->>sealmap_rust__resolve: cb_has(&r, t)
          sealmap_rust__resolve->>sealmap_model__symbol__Relation: Relation::new(clone(), clone(), FieldType, c)
          sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_relation(new())
        end
      end
      loop for st in &it.supertraits
        sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: resolve(&module, st, None)
        sealmap_rust__resolve->>sealmap_rust__resolve: keep_ref(&t)
        opt keep_ref(&t)
          sealmap_rust__resolve->>sealmap_model__symbol__Relation: Relation::new(clone(), t, Extends, c)
          sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_relation(new())
        end
      end
      sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: add_uses(&cb, &id, &module, &it.sig_refs, Some())
      opt !it.flow.is_empty()
        sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: flow(&ctx, &it.flow, opts)
      end
      loop for m in &it.methods
        sealmap_rust__resolve->>sealmap_model__symbol__SymbolId: child(&m.name)
        sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: method_symbol(&mid, &id, m, f, &ctx, opts, &cb)
        sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_symbol(ms)
      end
      sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_symbol(s)
    end
    loop for imp in &f.impls
      sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: resolve(&module, self_segs, None)
      opt via map
        sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: resolve(&module, segs, Some())
        sealmap_rust__resolve->>sealmap_rust__resolve: keep_ref(&t)
        opt keep_ref(&t) || r.internal.contains(&t)
          sealmap_rust__resolve->>sealmap_model__symbol__Relation: Relation::new(clone(), t, Implements, c)
          sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_relation(new())
        end
      end
      loop for m in &imp.methods
        alt Some(t)
          sealmap_rust__resolve->>sealmap_model__symbol__SymbolId: ~child(t)
        else None
          sealmap_rust__resolve->>sealmap_model__symbol__SymbolId: ~child(&m.name)
        end
        sealmap_rust__resolve->>sealmap_rust__resolve__Resolver: method_symbol(&mid, &ty, m, f, &ctx, opts, &cb)
        sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_symbol(ms)
      end
    end
  end
  loop for s in cb.symbols.values()
    opt let Some(flow) = &s.flow
      sealmap_rust__resolve->>sealmap_model__flow__Flow: ~calls()
    end
  end
  loop for ((from, to), c) in calls
    sealmap_rust__resolve->>sealmap_model__symbol__Relation: Relation::new(from, to, Calls, c)
    sealmap_rust__resolve->>sealmap_model__codebase__Codebase: add_relation(new())
  end
```

## `sealmap_rust::resolve::keep_ref`
`fn keep_ref(id: &SymbolId) -> bool` · L336-L340
> Keep a resolved reference as a relation target? Drops std and prelude.
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve as resolve mod
  participant sealmap_model__symbol__SymbolId as SymbolId
  sealmap_rust__resolve->>sealmap_model__symbol__SymbolId: as_str()
```

## `sealmap_rust::resolve::Resolver::new`
`fn new(files: &[RawFile]) -> Self` · L373-L471
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_model__symbol__SymbolId as SymbolId
  loop for f in files
    loop for m in &f.modules
      sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(&id)
      opt m.path.len()> 1
        sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(&id)
      end
    end
    loop for it in &f.items
      sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(_)
      opt it.kind == SymbolKind::Trait
        loop for m in &it.methods
          sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: child(&m.name)
        end
      end
    end
  end
  loop for round in 0..2
    loop for (module, uses) in &r.uses
      loop for u in uses.iter().filter(| u | u.alias == #quot;*#quot;)
        sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: walk(module, &u.target, None, 0, _)
      end
    end
  end
  loop for pass_trait in [false, true]
    loop for f in files
      loop for imp in &f.impls
        sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: resolve(&module, segs, None)
        opt via map
          sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: resolve(&module, segs, Some())
        end
        loop for m in &imp.methods
          alt Some(t)
            sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: ~child(t)
          else None
            sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: ~child(&m.name)
          end
        end
      end
    end
  end
```

## `sealmap_rust::resolve::Resolver::resolve`
`fn resolve(&self, module: &str, segs: &[String], self_ty: Option<&SymbolId>) -> (SymbolId, Confidence)` · L473-L485
> Resolve `segs` as written in `module`.
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_model__symbol__SymbolId as SymbolId
  sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: walk(module, segs, self_ty, 0, true)
  opt None
    sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(join())
  end
```

## `sealmap_rust::resolve::Resolver::resolve_internal`
`fn resolve_internal(&self, module: &str, segs: &[String], self_ty: Option<&SymbolId>) -> Option<SymbolId>` · L487-L489
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: walk(module, segs, self_ty, 0, true)
```

## `sealmap_rust::resolve::Resolver::walk`
`fn walk(&self, module: &str, segs: &[String], self_ty: Option<&SymbolId>, depth: u8, use_globs: bool,) -> Option<SymbolId>` · L491-L557
> `use_globs` is false only while the glob table itself is being built.
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_model__symbol__SymbolId as SymbolId
  opt depth> 8 || segs.is_empty()
    Note over sealmap_rust__resolve__Resolver: return None
  end
  alt #quot;crate#quot;
    sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(unwrap_or())
  else #quot;self#quot;
    sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(module)
  else #quot;super#quot;
    sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(module)
    opt via unwrap_or_else
      sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(module)
    end
  else name
    alt if let Some(u) = self.uses.get(module).and_then(| us | …
      sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: walk(module, &full, self_ty, _, use_globs)
      opt via or_else
        sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(join())
      end
      Note over sealmap_rust__resolve__Resolver: return self.walk(module, &full, self_ty, depth + 1, use…
    else if let Some(id) = use_globs.then(| | self.glob(module, …
      opt via then
        sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: glob(module, name)
      end
    else if self.crates.contains(name)
      sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(name)
    else
      sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(join())
      Note over sealmap_rust__resolve__Resolver: return Some(SymbolId::new(segs.join(#quot;::#quot;)))
    end
  end
  loop for (i, seg) in rest.iter().enumerate()
    opt self.uses.get(&key).is_some_and(| us | us.iter().any…
      sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: walk(&key, &_, self_ty, _, use_globs)
      Note over sealmap_rust__resolve__Resolver: return self.walk(&key, &rest [i..], self_ty, depth + 1,…
    end
  end
```

## `sealmap_rust::resolve::Resolver::refs`
`fn refs(&self, module: &str, raw: &[Segs], self_ty: Option<&SymbolId>) -> Vec<SymbolId>` · L580-L600
> Resolve a list of raw type refs to kept relation targets.
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_rust__resolve as resolve mod
  loop for segs in raw
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve: is_generic_param(&_)
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve: is_generic_param(&_)
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: local(module, &_)
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: resolve(module, segs, self_ty)
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve: keep_ref(&id)
  end
```

## `sealmap_rust::resolve::Resolver::member`
`fn member(&self, module: &str, m: &RawMember, owner: Option<&SymbolId>) -> Member` · L608-L617
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: refs(module, &m.refs, owner)
```

## `sealmap_rust::resolve::Resolver::add_uses`
`fn add_uses(&self, cb: &mut Codebase, from: &SymbolId, module: &str, raw: &[Segs], self_ty: Option<&SymbolId>)` · L619-L624
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_model__symbol__Relation as Relation
  participant sealmap_model__codebase__Codebase as Codebase
  sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: refs(module, raw, self_ty)
  loop for t in self.refs(module, raw, self_ty)
    sealmap_rust__resolve__Resolver->>sealmap_model__symbol__Relation: Relation::new(clone(), t, Uses, c)
    sealmap_rust__resolve__Resolver->>sealmap_model__codebase__Codebase: add_relation(new())
  end
```

## `sealmap_rust::resolve::Resolver::method_symbol`
`fn method_symbol(&self, id: &SymbolId, parent: &SymbolId, m: &RawFn, f: &RawFile, ctx: &FlowCtx<'_>, opts: &RustOptions, cb: &mut Codebase,) -> Symbol` · L626-L650
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_model__symbol__Symbol as Symbol
  sealmap_rust__resolve__Resolver->>sealmap_model__symbol__Symbol: Symbol::new(clone(), &m.name, Method, clone())
  sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: add_uses(cb, id, ctx.module, &m.sig_refs, ctx.self_ty)
  opt !m.flow.is_empty()
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: flow(ctx, &m.flow, opts)
  end
```

## `sealmap_rust::resolve::Resolver::flow`
`fn flow(&self, ctx: &FlowCtx<'_>, raw: &[RawStep], opts: &RustOptions) -> Option<Flow>` · L652-L663
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_model__flow__Flow as Flow
  sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: steps(ctx, raw, opts)
  opt steps.iter().all(| s | matches!(s, Step::Return(_)))
    Note over sealmap_rust__resolve__Resolver: return None
  end
  sealmap_rust__resolve__Resolver->>sealmap_model__flow__Flow: Flow::new(steps)
```

## `sealmap_rust::resolve::Resolver::steps`
`fn steps(&self, ctx: &FlowCtx<'_>, raw: &[RawStep], opts: &RustOptions) -> Vec<Step>` · L665-L721
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  loop for s in raw
    alt RawStep::Call(c)
      sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: call(ctx, c, opts)
    else RawStep::Branch(raw_arms)
      opt via map
        sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: steps(ctx, steps, opts)
      end
    else RawStep::Parallel(arms)
      sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: arms(ctx, arms, opts)
    else RawStep::Loop(label, body)
      sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: steps(ctx, body, opts)
    else RawStep::Optional(label, body)
      sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: steps(ctx, body, opts)
    end
  end
```

## `sealmap_rust::resolve::Resolver::arms`
`fn arms(&self, ctx: &FlowCtx<'_>, arms: &[(String, Vec<RawStep>)], opts: &RustOptions) -> Vec<Arm>` · L723-L728
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  opt via map
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: steps(ctx, steps, opts)
  end
```

## `sealmap_rust::resolve::Resolver::call`
`fn call(&self, ctx: &FlowCtx<'_>, c: &RawCall, opts: &RustOptions) -> Option<Call>` · L730-L771
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_rust__resolve as resolve mod
  alt Callee::Path(segs)
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: local(ctx.module, &_)
    opt PRELUDE.contains(&segs [0].as_str()) && !self.local(…
      Note over sealmap_rust__resolve__Resolver: return None
    end
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: resolve(ctx.module, segs, ctx.self_ty)
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: local(ctx.module, &_)
    opt segs.len() == 1 && c == Confidence::External && !sel…
      Note over sealmap_rust__resolve__Resolver: return None
    end
  else Callee::Method { recv, name }
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: method(ctx, recv, name)?
  end
  opt Confidence::External
    opt ExternalCalls::NonStd
      sealmap_rust__resolve__Resolver->>sealmap_rust__resolve: keep_ref(&target)
    end
  end
```

## `sealmap_rust::resolve::Resolver::method`
`fn method(&self, ctx: &FlowCtx<'_>, recv: &Recv, name: &str) -> Option<(SymbolId, Confidence)>` · L773-L804
> Resolve a method call.
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_model__symbol__SymbolId as SymbolId
  alt Recv::SelfField(field)
    alt Some((module, refs))
      sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: receiver_type(module, refs, None)
    else None
      sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: by_name_only(name)
      Note over sealmap_rust__resolve__Resolver: return Some(self.by_name_only(name))
    end
  else Recv::Typed(refs)
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: receiver_type(ctx.module, refs, ctx.self_ty)
  else Recv::Untyped
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: by_name_only(name)
    Note over sealmap_rust__resolve__Resolver: return Some(self.by_name_only(name))
  end
  opt let-else
    sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(_)
    Note over sealmap_rust__resolve__Resolver: return Some((SymbolId::new(format!(#quot;?::{name}#quot;)), Confi…
  end
  opt let Some(id) = self.methods.get(&ty).and_then(| m | …
    Note over sealmap_rust__resolve__Resolver: return Some((id.clone(), Confidence::Exact))
  end
  opt self.trait_methods.get(&ty).is_some_and(| n | n.cont…
    Note over sealmap_rust__resolve__Resolver: return Some((ty.child(name), Confidence::Exact))
  end
  loop for tr in self.impls.get(&ty).into_iter().flatten()
    opt self.trait_methods.get(tr).is_some_and(| n | n.conta…
      sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: ~child(name)
      Note over sealmap_rust__resolve__Resolver: return Some((tr.child(name), Confidence::Exact))
    end
  end
```

## `sealmap_rust::resolve::Resolver::receiver_type`
`fn receiver_type(&self, module: &str, refs: &[Segs], self_ty: Option<&SymbolId>) -> Option<SymbolId>` · L806-L824
> The type a method is called on, from the declared type's paths (outermost first).
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_rust__resolve as resolve mod
  loop for segs in refs
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve: is_generic_param(last)
    opt segs.len() == 1 && is_generic_param(last)
      Note over sealmap_rust__resolve__Resolver: return None
    end
    sealmap_rust__resolve__Resolver->>sealmap_rust__resolve__Resolver: resolve(module, segs, self_ty)
    Note over sealmap_rust__resolve__Resolver: return Some(id)
  end
```

## `sealmap_rust::resolve::Resolver::by_name_only`
`fn by_name_only(&self, name: &str) -> (SymbolId, Confidence)` · L826-L837
> Unknown receiver: accept a unique, distinctive internal method name.
```mermaid
sequenceDiagram
  participant sealmap_rust__resolve__Resolver as Resolver
  participant sealmap_model__symbol__SymbolId as SymbolId
  opt !COMMON_METHODS.contains(&name)
    opt let Some(ids) = self.by_name.get(name)
      opt ids.len() == 1
        opt via unwrap_or_else
          sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(name)
        end
        Note over sealmap_rust__resolve__Resolver: return (id, Confidence::Inferred)
      end
    end
  end
  sealmap_rust__resolve__Resolver->>sealmap_model__symbol__SymbolId: SymbolId::new(_)
```
