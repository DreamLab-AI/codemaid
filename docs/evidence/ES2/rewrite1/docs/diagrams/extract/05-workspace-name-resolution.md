---
id: EXT-05
title: Workspace-wide name resolution
area: extract
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - crates/sealmap-rust/src/resolve.rs
  - crates/sealmap-rust/src/collect.rs
  - crates/sealmap-rust/src/lib.rs
  - crates/sealmap-rust/tests/extract.rs
  - crates/sealmap-rust/tests/resolver_den01.rs
  - crates/sealmap-rust/src/layout.rs
  - crates/sealmap-extract/src/raw.rs
  - docs/DESIGN.md
  - README.md
verified_commit: 4ed7a51f92f7241a1c420e49e8036a8d9189912a
---
## For developers

Resolution is the one sequential pass of the Rust adapter. It builds lookup
tables from every raw file, then resolves each path and method call to a
`SymbolId` with a confidence, without a type checker. Paths go through local
items, `use` imports (renames, globs, `pub use` re-exports),
`crate`/`self`/`super`/`Self` and sibling workspace crates; method calls are
resolved from what the walker recorded about the receiver
(`crates/sealmap-rust/src/lib.rs:24`-`30` summarises the same list; the tables are in
`crates/sealmap-rust/src/resolve.rs:456`-`482`).

Three commits shaped it. The hardening work replaced an exponential recursive
glob walk with a precomputed table and a breadth-first search
(`docs/DESIGN.md:234`). The grammar work keyed every table by typed ids and
split types from values, so a module and a function of one name no longer
collide (commit `5973a4e`). The `Path::parent` fix taught it receiver
provenance: a value taken from std or a dependency is never matched to an
internal method by name (`docs/DESIGN.md:236`, commit `5aac8a8`). This topic
draws the path walk, the method decision, the provenance rule, and the false
guesses that remain.

## For the business

Resolution quality is the difference between a diagram that shows how the
code actually talks to itself and one that invents conversations. sealmap
resolves most calls exactly; for the rest it either says it does not know, or
makes a narrow, labelled guess: a method name that exists exactly once among
the crates the calling crate can reach, and is not a common name like `get`
or `insert`.

The `Path::parent` fix shows the stakes: before it, a call on a file path was
drawn as a call into sealmap's own id type. On a 934-file workspace it
withdrew 76 name guesses, 64 of them on std or third-party values (commit
`5aac8a8`). The guesses that are left are tagged `inferred` and drawn with a
`~`; the main remaining source of wrong ones is untyped closure parameters,
recorded below.

## EXT-05.1 The tables, built before anything resolves

```mermaid
sequenceDiagram
    participant RV as Resolver<br/>resolve.rs:459
    participant SL as Slots<br/>resolve.rs:372
    participant NS as Ns, an enum<br/>Type Value Prefix, resolve.rs:360
    Note over RV: plan, crate reach for name guesses (resolve.rs:459)<br/>crates (resolve.rs:460)<br/>items, module to name to Slots (resolve.rs:462)<br/>uses, module to use entries (resolve.rs:464)<br/>internal, every id that will exist (resolve.rs:466)<br/>fields, type to field refs (resolve.rs:469)<br/>methods, type to name to id (resolve.rs:471)<br/>by_name, method name to ids (resolve.rs:473)<br/>impls, type to traits (resolve.rs:475)<br/>trait_methods (resolve.rs:477)<br/>globs, module to glob targets (resolve.rs:481)
    RV->>SL: every name in a module carries Slots
    Note over SL: ty (resolve.rs:372) and value (resolve.rs:373)
    SL->>NS: get(ns) reads the own namespace first,<br/>a prefix type only (resolve.rs:388)
```

**What it shows.** Every table is an ordered map keyed by typed ids. Each name
in a module has a type slot and a value slot, so `mod config` and
`fn config` in one module are both reachable. A segment with more after it
is a prefix and reads the type slot alone, since only a module or type has
members (`crates/sealmap-rust/src/resolve.rs:392`).

**Why it is this way.** Rust keeps types and modules apart from values
(`crates/sealmap-rust/src/resolve.rs:355`-`358`); looking a path's last segment
up in its own namespace first, with the other as a fallback, mirrors that
without a full name-resolution pass. The fallback once applied to prefixes
too, so the CLI's `fn pack` captured `pack::shard(..)` from the imported
module `pack` (DEN-01.6, fixed in `926144c`, pinned by
`a_path_prefix_never_resolves_to_a_function`,
`crates/sealmap-rust/tests/resolver_den01.rs:241`).

**Invariant:** inherent impls are registered before trait impls, so an
inherent method wins a name lookup over a trait method
(`crates/sealmap-rust/src/resolve.rs:557`-`559`).

## EXT-05.2 Walking a path

```mermaid
sequenceDiagram
    autonumber
    participant WK as Resolver.walk over the<br/>segments as written
    participant BS as the first segment, resolve.rs:650
    participant RE as each later segment<br/>resolve.rs:687
    alt depth over 8 (resolve.rs:642)
        WK-->>WK: give up, None, becomes a path id, External (resolve.rs:601)
    else within the depth cap
        WK->>BS: resolve the first segment (resolve.rs:650)
        alt crate, self, super, Self (resolve.rs:651-661)
            BS-->>RE: base found
        else item in this module (resolve.rs:663)
            BS-->>RE: base found
        else use alias
            BS-->>WK: re-walk the target plus the rest (resolve.rs:675)
        else glob imports, breadth first (resolve.rs:676)
            BS-->>RE: base found
        else a workspace crate name (resolve.rs:678)
            BS-->>RE: base found
        else otherwise
            BS-->>WK: a path id (resolve.rs:681)
        end
        RE->>RE: item, method, pub use re-export,<br/>undefined member, or extend the path (resolve.rs:687)
        alt the id is internal (resolve.rs:596)
            RE-->>WK: Exact
        else root is a workspace crate (resolve.rs:599)
            RE-->>WK: Inferred
        else neither
            RE-->>WK: External
        end
    end
```

**What it shows.** The first segment picks a base (a keyword, a local item,
an import, a glob, a crate), later segments walk down through items and
methods, and the result is exact only if the id is one the codebase defines.

**Why it is this way.** Import targets are re-resolved from the importing
module, so renames and chains of `pub use` resolve to the definition, not the
alias (`crates/sealmap-rust/src/resolve.rs:670`-`672`). The test
`resolves_paths_through_imports_renames_globs_and_reexports` pins the common
shapes (`crates/sealmap-rust/tests/extract.rs:37`).

**Debt:** the walk gives up after eight re-entries
(`crates/sealmap-rust/src/resolve.rs:642`), and a path it gives up on becomes
an external path id with no diagnostic, so a long re-export chain silently
leaves the codebase.

## EXT-05.3 Globs without exponential cost

```mermaid
sequenceDiagram
    autonumber
    participant NW as Resolver.new<br/>resolve.rs:485
    participant WK as walk<br/>resolve.rs:633
    participant GB as glob<br/>resolve.rs:719
    NW->>NW: round 0 and round 1 over every glob import (resolve.rs:544)
    NW->>WK: resolve the glob target, globs off in round 0 (resolve.rs:548)
    WK-->>NW: module id, kept if it has items (resolve.rs:549)
    NW->>NW: replace the glob table after each round (resolve.rs:555)
    Note over NW: later, any lookup that misses locally
    WK->>GB: name, namespace (resolve.rs:676)
    GB->>GB: queue the module's glob targets, visit each once (resolve.rs:722)
    GB-->>WK: first module in BFS order defining the name (resolve.rs:727)
```

**What it shows.** Glob targets are resolved up front into a table; a lookup
through globs is then a bounded breadth-first search where each module is
visited once, so cyclic globs are harmless.

**Why it is this way.** The recursive walk this replaced made 908 M calls on
one file of an older crate; the table brought it to 43 ms (`docs/DESIGN.md:234`).
`glob_cycles_resolve_quickly` keeps it that way
(`crates/sealmap-rust/tests/extract.rs:278`).

**Open:** the table is built in exactly two rounds, so a glob target that is
itself only reachable through a glob of a glob is found
(`crates/sealmap-rust/src/resolve.rs:542`-`543`); nothing records whether a
third level of glob-reached glob targets is meant to resolve.

## EXT-05.4 Deciding a method call

```mermaid
sequenceDiagram
    autonumber
    participant R as a receiver recorded by the walker<br/>Resolver.method, resolve.rs:858
    participant RT as receiver_type<br/>look through Box, Arc, Rc and guards<br/>resolve.rs:921
    participant LK as the method lookup<br/>resolve.rs:877-887
    participant BN as by_name_only, resolve.rs:954
    alt self (resolve.rs:860)
        R->>LK: the impl's type
    else self.field
        R->>RT: the field's declared type (resolve.rs:861)
        RT->>LK: the type under the wrapper
    else typed local or parameter (resolve.rs:865)
        R->>RT: its declared type
        RT->>LK: the type under the wrapper
    else untyped variable (resolve.rs:866)
        R->>BN: no type evidence
    else derived part of another value (resolve.rs:867)
        alt its origin is internal
            R->>BN: name guess
        else its origin is std or a dependency
            R->>R: never bound to an internal method (resolve.rs:868-872)
        end
    else returned, computed or unknown
        R->>R: dropped (resolve.rs:874)
    end
    opt a receiver with a known type
        alt method on the type, on the trait, or on a trait it implements (resolve.rs:877-887)
            LK-->>R: Exact
        else no such method
            LK-->>R: Type method id, External (resolve.rs:890)
        end
    end
```

**What it shows.** A receiver with a known type is resolved against that
type's methods, its trait's methods and the methods of every trait it
implements; a receiver with no type evidence falls back to a name match;
values the walker could only describe as computed or returned are not
resolved at all.

**Debt:** a method called on a returned or computed receiver, as in
`a.b().c()` or `(x + y).m()`, is never bound to an internal method: the
match yields no type (`crates/sealmap-rust/src/resolve.rs:874`) and the call
falls through to an external, unresolved id
(`crates/sealmap-rust/src/resolve.rs:876`), so builder chains and fluent APIs
lose their internal edges. Fixing it needs return-type inference, which the
adapter does not do.

**Why it is this way.** Smart pointers and lock guards are looked through, but
any other wrapper (`Vec`, `Option`, `Mutex`) is itself the receiver
(`crates/sealmap-rust/src/resolve.rs:917`-`919`), so `vec.len()` is a std call,
not a call on the element type.

**Fixed at `fec7aff` (found in the 2026-10-05 review triage).** A generic
parameter used to be recognised by its spelling (one capital letter, optional
digits). So every call through a real `struct A` or `struct V2` was dropped,
and a parameter named `Store` was bound to a concrete `struct Store`. A
receiver is now a parameter only when the item, impl or trait declares it
(`crates/sealmap-rust/src/resolve.rs:932`; the scope is
`crates/sealmap-rust/src/resolve.rs:414`, filled from
`crates/sealmap-rust/src/collect.rs:788`). The fix is pinned by
`single_capital_type_names_are_concrete_types_not_generics`
(`crates/sealmap-rust/tests/extract.rs:421`) and
`declared_multi_letter_generics_shadow_concrete_types` (`:459`).

**Invariant:** a method call on a value derived from std or a dependency is
never bound to an internal method of the same name; it becomes `sym:? name`
(`crates/sealmap-rust/src/resolve.rs:868`-`872`), pinned by
`std_receiver_method_is_not_bound_to_same_named_internal_method`
(`crates/sealmap-rust/tests/extract.rs:310`).

## EXT-05.5 The name guess and its provenance gate

```mermaid
sequenceDiagram
    autonumber
    participant ME as method<br/>resolve.rs:858
    participant IO as internal_origin<br/>resolve.rs:898
    participant BN as by_name_only<br/>resolve.rs:954
    ME->>IO: is this derived value's origin internal code (resolve.rs:868)
    IO->>IO: self or untyped variable, yes (resolve.rs:903)
    IO->>IO: typed, field or returned, only if an internal type or fn (resolve.rs:908)
    IO->>IO: unknown, no (resolve.rs:913)
    IO-->>ME: true
    ME->>BN: method name
    BN->>BN: not in the common-name list (resolve.rs:955)
    BN->>BN: methods of that name in crates the caller reaches (resolve.rs:958-963)
    BN-->>ME: exactly one, Inferred (resolve.rs:964-965)
    Note over BN: otherwise sym:? name, External, resolve.rs:968
```

**What it shows.** A guess is made only for a distinctive name (not one of
about 125 common method names such as `get`, `insert`, `run`) that exists
exactly once among the methods of crates the caller can reach, and only for
receivers whose origin is internal or simply unknown.

**Why it is this way.** A plain variable of unevident type is treated as
possibly internal, so the guess still finds genuine internal calls through
untyped locals; anything traced to std or a dependency is excluded
(`crates/sealmap-rust/src/resolve.rs:893`-`897`).

**Invariant:** a guess depends only on the calling crate and the workspace
crates its manifest lets it name: itself, its package's library and its
workspace dependencies (`crates/sealmap-rust/src/layout.rs:135`). Code added
to a crate it cannot reach never changes it, and two reachable candidates
give no edge rather than a pick
(`crates/sealmap-rust/tests/resolver_den01.rs:158`,
`crates/sealmap-rust/tests/resolver_den01.rs:186`). Until `613b9ba` the name
had to be unique in the whole workspace, so adding two `root` methods in
sealmap-dense removed a correct guess in sealmap-rust (DEN-01.6).

**Debt:** an untyped variable counts as an internal origin
(`crates/sealmap-rust/src/resolve.rs:903`), so a part taken from a plain local
whose value is in fact a std or dependency type is still matched to a
distinctive internal method by name.

## EXT-05.6 Closure parameters are untyped

```mermaid
sequenceDiagram
    autonumber
    participant C as xs.iter().for_each with a closure<br/>whose parameter is conn
    participant AR as the closure argument walked<br/>with the environment cloned<br/>collect.rs:1348-1351
    participant RS as the resolver, resolve.rs:866
    participant TE as the test, tests/extract.rs:351
    C->>AR: walk the closure
    Note over AR: parameters never bound
    alt conn is not in the environment (collect.rs:1384)
        AR-->>RS: Untyped
        RS->>TE: name guess on Untyped (resolve.rs:866)<br/>vacuum matched to the only internal vacuum,<br/>Inferred, though conn is a u8
    else conn shadows an outer typed name
        AR-->>RS: keeps the outer type
    end
```

**What it shows.** A closure's parameters are never entered into the walker's
environment, so a method call on one is either a name guess (no outer binding)
or resolved against the type of an unrelated outer variable with the same name.

**Why it is this way.** Closure parameter types are usually inferred by the
compiler from the call they are passed to, which the walker cannot see; the
test suite records the current behaviour deliberately: "A closure parameter
is untyped, as before" (`crates/sealmap-rust/tests/extract.rs:392`-`393`).

**Debt:** false guesses remain from untyped closure parameters: a call on a
closure parameter is matched to a distinctive internal method by name
(`crates/sealmap-rust/src/collect.rs:1348`-`1354`,
`crates/sealmap-rust/src/resolve.rs:866`), so `|conn| ... d.vacuum()` over a
slice of `u8` is drawn as an inferred call to `Db::vacuum`
(`crates/sealmap-rust/tests/extract.rs:351`).

**Debt:** a closure parameter that shadows an outer typed local is resolved
with the outer local's type, because the cloned environment still holds the
outer binding (`crates/sealmap-rust/src/collect.rs:1349`-`1350`).
