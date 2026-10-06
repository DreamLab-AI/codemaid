---
id: EXT-02
title: Minting ids for Rust definitions
area: extract
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - crates/sealmap-extract/src/ids.rs
  - crates/sealmap-rust/src/collect.rs
  - crates/sealmap-rust/src/resolve.rs
  - crates/sealmap-rust/src/lib.rs
  - crates/sealmap-model/src/symbol.rs
  - README.md
verified_commit: 4ed7a51f92f7241a1c420e49e8036a8d9189912a
---
## For developers

The grammar in MOD-01 says what an id may look like; this topic says which id
each Rust definition actually gets. Every id an adapter mints goes through
`sealmap_extract::ids` (`crates/sealmap-extract/src/ids.rs:3`-`4`), so two
adapters can only ever disagree about what they collect, never about how an
id is spelled. The Rust collector decides which items become symbols, and the
resolver decides which type a method belongs to.

The rule that matters most is that a method sits under the type that owns
it, never under its `impl` block (`crates/sealmap-extract/src/ids.rs:20`-`28`):
splitting an `impl` or moving it to another file keeps every method id. The
README promises the same thing to users (`README.md:177`-`179`). This came
with the grammar in commit `5973a4e`; the undefined-member rule at the end of
this topic came with the dogfood fix in `6c8a9b0`.

## For the business

A seal is a promise about named functions, so the names have to stay put when
the code is merely tidied. This topic is where that is decided for Rust.
Reorganising methods across `impl` blocks or files, the most common
refactor in a growing Rust codebase, changes no name and therefore invalidates
no reviewed diagram. A real rename, or moving a function to another module,
does change the name, which is what a reviewer would expect to look at again.

One limit for adopters to know: items that a macro generates are not seen,
so they get no name and cannot be cited or sealed (`README.md:342`).

## EXT-02.1 The id table

```mermaid
flowchart TB
    PKG["package root<br/>sym:cargo app .<br/>ids.rs:8"]
    MOD["module<br/>sym:cargo app . store/<br/>module_id, ids.rs:68"]
    ITEM["type, fn, const, macro in a scope<br/>item_id, ids.rs:75"]
    METH["inherent method under its type<br/>method_id, ids.rs:82"]
    TRAIT["trait-impl method, Trait descriptor<br/>spelled as written, ids.rs:84"]
    FOREIGN["impl on a type outside the code<br/>module/impl#35;[SelfTy][Trait]m().<br/>impl_method_id, ids.rs:105"]
    EXT["path with unknown kinds<br/>path_id, ids.rs:116"]
    UNR["method on an unknown receiver<br/>unresolved_method_id, ids.rs:122"]
    PKG --> MOD --> ITEM --> METH --> TRAIT
    METH --> FOREIGN
    ITEM -.->|scope not global, ids.rs:76| EXT
    METH -.->|unknown receiver| UNR
```

**What it shows.** Each kind of definition and reference has one builder.
A trait-impl method adds a `[Trait]` descriptor before the method name; an
impl on a type the code does not define is anchored in the module holding
the impl, after rust-analyzer's `impl#[SelfType][Trait]` form
(`crates/sealmap-extract/src/ids.rs:25`-`28`).

**Why it is this way.** The trait is spelled as the impl writes it, generic
arguments kept, so `impl From<A>` and `impl From<B>` give two ids
(`crates/sealmap-extract/src/ids.rs:23`-`25`). A scope that is not a global id
degrades to a path id instead of failing (`crates/sealmap-extract/src/ids.rs:76`).

**Invariant:** building an id never fails: a package name the grammar cannot
hold is repaired to `_`, and an invalid manager falls back to `unknown _ .`
(`crates/sealmap-extract/src/ids.rs:57`-`63`).

## EXT-02.2 Which Rust items become symbols

```mermaid
sequenceDiagram
    participant IT as an item in a module<br/>Collector.item, collect.rs:191
    participant MO as inline mod<br/>a module of its own<br/>collect.rs:212
    participant DA as struct, enum, union<br/>collect.rs:226
    participant TR as trait, with provided methods<br/>collect.rs:275
    participant AL as type alias<br/>collect.rs:360
    participant FN as free function with a flow<br/>collect.rs:377
    participant VA as const, static<br/>collect.rs:394
    participant MA as macro_rules<br/>collect.rs:410
    participant IM as impl block<br/>methods kept as RawImpl<br/>collect.rs:431
    participant RE as use, extern crate, foreign blocks<br/>module body only<br/>collect.rs:483
    alt inline mod (collect.rs:212)
        IT->>MO: a module of its own
    else struct, enum, union (collect.rs:226)
        IT->>DA: a data type
    else trait (collect.rs:275)
        IT->>TR: with provided methods
    else type alias (collect.rs:360)
        IT->>AL: an alias
    else free function (collect.rs:377)
        IT->>FN: with a flow
    else const, static (collect.rs:394)
        IT->>VA: a value
    else macro_rules (collect.rs:410)
        alt with a name (collect.rs:410)
            IT->>MA: macro symbol, public if macro_export (collect.rs:411)
        else an unnamed item-level invocation
            IT->>MA: folded into the module body hash only (collect.rs:428)
        end
    else impl block (collect.rs:431)
        IT->>IM: methods kept as RawImpl
    else use, extern crate, foreign blocks (collect.rs:483)
        IT->>RE: module body only
    end
```

**What it shows.** Modules, data types, traits, aliases, functions, constants,
statics and named macros become symbols; methods arrive through impl blocks
and are given ids in the resolve pass; everything else only feeds the
module's body fingerprint.

**Why it is this way.** An item-level macro invocation such as
`thread_local!` has no name to cite, so the only honest thing to record is
that the module changed when it changes (`crates/sealmap-rust/src/collect.rs:427`-`428`).

**Debt:** the test filter (`self.skip`) is applied to modules, structs,
enums, traits, functions and impls, but not to unions, type aliases,
constants, statics or macros (`crates/sealmap-rust/src/collect.rs:237`,
`crates/sealmap-rust/src/collect.rs:360`,
`crates/sealmap-rust/src/collect.rs:394`); a top-level `#[cfg(test)]` const
is modelled even when tests are excluded.

## EXT-02.3 A method's owner is resolved, then the id is built

```mermaid
sequenceDiagram
    autonumber
    participant BU as build<br/>resolve.rs:213
    participant RS as Resolver.resolve<br/>resolve.rs:588
    participant IM as impl_method<br/>resolve.rs:336
    participant ID as impl_method_id<br/>ids.rs:95
    participant MS as method_symbol<br/>resolve.rs:789
    BU->>RS: self type of the impl as written (resolve.rs:300)
    RS-->>BU: owner id, global if internal
    BU->>IM: module, owner, impl, method name (resolve.rs:309)
    IM->>ID: self type text, trait display text (resolve.rs:339)
    alt owner is a global id
        ID-->>IM: method under the owner type (ids.rs:103)
    else owner outside the code
        ID-->>IM: module/impl anchor (ids.rs:110)
    end
    BU->>MS: id, parent is the owner (resolve.rs:310)
    MS-->>BU: Symbol with parent set to the owner (resolve.rs:800)
    BU->>BU: trait impls tag the method impl Trait (resolve.rs:312)
```

**What it shows.** The impl's self type is resolved through imports and
modules first; only then is the method id built, so several inherent impls
of one type, in any files, produce methods under one owner.

**Why it is this way.** Inherent impls are registered before trait impls so
that an inherent method wins a name lookup over a trait method of the same
name (`crates/sealmap-rust/src/resolve.rs:557`-`559`).

**Invariant:** a trait-impl method is recorded as public whatever its
written visibility, because trait methods are as visible as the trait
(`crates/sealmap-rust/src/collect.rs:458`).

## EXT-02.4 Formatting never reaches an id

```mermaid
flowchart TB
    W["impl Handler of Msg with a trailing comma<br/>as rustfmt writes a broken list"]
    TT["trait_text<br/>collect.rs:493"]
    TK["tokens printed and squeezed<br/>collect.rs:494"]
    TC["trailing comma before a closing<br/>angle or square bracket dropped<br/>collect.rs:495"]
    KEEP["comma before a closing paren kept:<br/>a one-tuple is a different type<br/>collect.rs:492"]
    ID["the same Trait descriptor text<br/>either way"]
    W --> TT --> TK --> TC --> ID
    TC -.-> KEEP
```

**What it shows.** The trait part of a method id is printed from tokens, with
the trailing commas rustfmt adds to a broken generic list removed.

**Why it is this way.** Ids must not change when rustfmt reflows a line; the
README's claim that reflowing tokio, VisionClaw and sealmap at
`max_width = 50` changes no id (`README.md:198`-`199`) depends on this rule
(`crates/sealmap-rust/src/collect.rs:488`-`492`).

**Invariant:** `(T,)` and `(T)` stay distinct in a trait's text, because only
commas before `>` and `]` are dropped (`crates/sealmap-rust/src/collect.rs:495`).

## EXT-02.5 A member the code does not define stays with its type

```mermaid
sequenceDiagram
    autonumber
    participant P as path Fp::default as written
    participant WK as Resolver.walk<br/>walk the path segment by segment<br/>resolve.rs:633
    P->>WK: Fp::default
    loop over each segment
        alt segment defined on the current type or module (resolve.rs:688)
            WK->>WK: continue from the definition
        else not defined here
            alt last segment, current is an internal type (resolve.rs:695)
                alt value position
                    WK-->>P: Type method id (resolve.rs:705)
                else type ns
                    WK-->>P: Type member id (resolve.rs:706)
                end
            else otherwise
                WK-->>P: extend as a path id (resolve.rs:709)
            end
        end
    end
```

**What it shows.** `Fingerprint::default()` from a derive has no definition
in the code, but it still belongs to `Fingerprint`, so it becomes
`Fingerprint#default().` rather than a kind-free `sym:extern` path.

**Why it is this way.** Before `6c8a9b0` such calls were cut loose from their
type and drawn on an external lane in the self-corpus; the fix keeps them on
the type's lane (`crates/sealmap-rust/src/resolve.rs:699`-`703`).

**Debt:** such an id names a symbol the model does not contain, so it is not
internal and resolves with `Inferred` confidence inside the workspace or
`External` outside it (`crates/sealmap-rust/src/resolve.rs:597`-`599`); a seal
could cite an id that `Codebase::symbol` never returns.
