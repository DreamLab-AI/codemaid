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
  - crates/sealmap-extract/src/raw.rs
  - docs/DESIGN.md
  - README.md
verified_commit: af4b8b44098e3f9a8cd01a550715f02827f1a8cd
---
## For developers

Resolution is the one sequential pass of the Rust adapter. It builds lookup
tables from every raw file, then resolves each path and method call to a
`SymbolId` with a confidence, without a type checker. Paths go through local
items, `use` imports (renames, globs, `pub use` re-exports),
`crate`/`self`/`super`/`Self` and sibling workspace crates; method calls are
resolved from what the walker recorded about the receiver
(`crates/sealmap-rust/src/lib.rs:23`-`27` summarises the same list; the tables are in
`crates/sealmap-rust/src/resolve.rs:392`-`415`).

Three commits shaped it. The hardening work replaced an exponential recursive
glob walk with a precomputed table and a breadth-first search
(`docs/DESIGN.md:165`). The grammar work keyed every table by typed ids and
split types from values, so a module and a function of one name no longer
collide (commit `5973a4e`). The `Path::parent` fix taught it receiver
provenance: a value taken from std or a dependency is never matched to an
internal method by name (`docs/DESIGN.md:167`, commit `5aac8a8`). This topic
draws the path walk, the method decision, the provenance rule, and the false
guesses that remain.

## For the business

Resolution quality is the difference between a diagram that shows how the
code actually talks to itself and one that invents conversations. sealmap
resolves most calls exactly; for the rest it either says it does not know, or
makes a narrow, labelled guess: a method name that exists exactly once in the
codebase and is not a common name like `get` or `insert`.

The `Path::parent` fix shows the stakes: before it, a call on a file path was
drawn as a call into sealmap's own id type. On a 934-file workspace it
withdrew 76 name guesses, 64 of them on std or third-party values (commit
`5aac8a8`). The guesses that are left are tagged `inferred` and drawn with a
`~`; the main remaining source of wrong ones is untyped closure parameters,
recorded below.

## EXT-05.1 The tables, built before anything resolves

```mermaid
classDiagram
    direction LR
    class Resolver {
        crates  resolve.rs:394
        items module to name to Slots  resolve.rs:396
        uses module to use entries  resolve.rs:398
        internal every id that will exist  resolve.rs:400
        fields type to field refs  resolve.rs:402
        methods type to name to id  resolve.rs:404
        by_name method name to ids  resolve.rs:406
        impls type to traits  resolve.rs:408
        trait_methods  resolve.rs:410
        globs module to glob targets  resolve.rs:414
    }
    class Slots {
        ty  resolve.rs:363
        value  resolve.rs:364
        get(ns) own namespace first  resolve.rs:379
    }
    class Ns {
        <<enum>>
        Type Value  resolve.rs:355
    }
    Resolver *-- Slots
    Slots ..> Ns
```

**What it shows.** Every table is an ordered map keyed by typed ids. Each name
in a module has a type slot and a value slot, so `mod config` and
`fn config` in one module are both reachable.

**Why it is this way.** Rust keeps types and modules apart from values
(`crates/sealmap-rust/src/resolve.rs:351`-`353`); looking a path's last segment
up in its own namespace first, with the other as a fallback, mirrors that
without a full name-resolution pass.

**Invariant:** inherent impls are registered before trait impls, so an
inherent method wins a name lookup over a trait method
(`crates/sealmap-rust/src/resolve.rs:488`-`490`).

## EXT-05.2 Walking a path

```mermaid
flowchart TB
    S["segments as written"]
    DEP{"depth over 8?<br/>resolve.rs:555"}
    NONE["None: becomes a path id, External<br/>resolve.rs:532"]
    F{"first segment<br/>resolve.rs:563"}
    KW["crate, self, super, Self<br/>resolve.rs:564-574"]
    LOC["item in this module<br/>resolve.rs:576"]
    USE["use alias: re-walk the target<br/>plus the rest<br/>resolve.rs:588"]
    GL["glob imports, breadth first<br/>resolve.rs:589"]
    CR["a workspace crate name<br/>resolve.rs:591"]
    PATH["otherwise a path id<br/>resolve.rs:594"]
    REST["each later segment: item, method,<br/>pub use re-export, undefined member,<br/>or extend the path<br/>resolve.rs:600"]
    RES{"id internal?<br/>resolve.rs:527"}
    EXA["Exact"]
    INF["Inferred: root is a workspace crate<br/>resolve.rs:530"]
    EXT["External"]
    S --> DEP
    DEP -->|yes| NONE
    DEP -->|no| F
    F --> KW --> REST
    F --> LOC --> REST
    F --> USE
    F --> GL --> REST
    F --> CR --> REST
    F --> PATH
    REST --> RES
    RES -->|yes| EXA
    RES -->|no| INF
    RES -->|no| EXT
```

**What it shows.** The first segment picks a base (a keyword, a local item,
an import, a glob, a crate), later segments walk down through items and
methods, and the result is exact only if the id is one the codebase defines.

**Why it is this way.** Import targets are re-resolved from the importing
module, so renames and chains of `pub use` resolve to the definition, not the
alias (`crates/sealmap-rust/src/resolve.rs:583`-`585`). The test
`resolves_paths_through_imports_renames_globs_and_reexports` pins the common
shapes (`crates/sealmap-rust/tests/extract.rs:37`).

**Debt:** the walk gives up after eight re-entries
(`crates/sealmap-rust/src/resolve.rs:555`), and a path it gives up on becomes
an external path id with no diagnostic, so a long re-export chain silently
leaves the codebase.

## EXT-05.3 Globs without exponential cost

```mermaid
sequenceDiagram
    autonumber
    participant NW as Resolver.new<br/>resolve.rs:418
    participant WK as walk<br/>resolve.rs:546
    participant GB as glob<br/>resolve.rs:632
    NW->>NW: round 0 and round 1 over every glob import (resolve.rs:475)
    NW->>WK: resolve the glob target, globs off in round 0 (resolve.rs:479)
    WK-->>NW: module id, kept if it has items (resolve.rs:480)
    NW->>NW: replace the glob table after each round (resolve.rs:486)
    Note over NW: later, any lookup that misses locally
    WK->>GB: name, namespace (resolve.rs:589)
    GB->>GB: queue the module's glob targets, visit each once (resolve.rs:635)
    GB-->>WK: first module in BFS order defining the name (resolve.rs:640)
```

**What it shows.** Glob targets are resolved up front into a table; a lookup
through globs is then a bounded breadth-first search where each module is
visited once, so cyclic globs are harmless.

**Why it is this way.** The recursive walk this replaced made 908 M calls on
one file of an older crate; the table brought it to 43 ms (`docs/DESIGN.md:165`).
`glob_cycles_resolve_quickly` keeps it that way
(`crates/sealmap-rust/tests/extract.rs:278`).

**Open:** the table is built in exactly two rounds, so a glob target that is
itself only reachable through a glob of a glob is found
(`crates/sealmap-rust/src/resolve.rs:473`-`474`); nothing records whether a
third level of glob-reached glob targets is meant to resolve.

## EXT-05.4 Deciding a method call

```mermaid
flowchart TB
    R["receiver recorded by the walker<br/>Resolver.method, resolve.rs:763"]
    SV["self: the impl's type<br/>resolve.rs:765"]
    SF["self.field: the field's declared type<br/>resolve.rs:766"]
    TY["typed local or parameter<br/>resolve.rs:773"]
    UT["untyped variable<br/>resolve.rs:774"]
    DV["derived part of another value<br/>resolve.rs:775"]
    DR["returned, computed or unknown:<br/>dropped<br/>resolve.rs:782"]
    RT["receiver_type: look through Box,<br/>Arc, Rc and guards<br/>resolve.rs:828"]
    LK{"method on the type, on the trait,<br/>or on a trait it implements?<br/>resolve.rs:785-795"}
    EX["Exact"]
    UM["Type method id, External<br/>resolve.rs:798"]
    BN["by_name_only<br/>resolve.rs:846"]
    SV --> LK
    SF --> RT
    TY --> RT
    RT --> LK
    LK -->|yes| EX
    LK -->|no| UM
    R --> SV
    R --> SF
    R --> TY
    R --> UT --> BN
    R --> DV
    R --> DR
    DV -->|origin internal| BN
```

**What it shows.** A receiver with a known type is resolved against that
type's methods, its trait's methods and the methods of every trait it
implements; a receiver with no type evidence falls back to a name match;
values the walker could only describe as computed or returned are not
resolved at all.

**Why it is this way.** Smart pointers and lock guards are looked through, but
any other wrapper (`Vec`, `Option`, `Mutex`) is itself the receiver
(`crates/sealmap-rust/src/resolve.rs:825`-`827`), so `vec.len()` is a std call,
not a call on the element type.

**Invariant:** a method call on a value derived from std or a dependency is
never bound to an internal method of the same name; it becomes `sym:? name`
(`crates/sealmap-rust/src/resolve.rs:776`-`780`), pinned by
`std_receiver_method_is_not_bound_to_same_named_internal_method`
(`crates/sealmap-rust/tests/extract.rs:310`).

## EXT-05.5 The name guess and its provenance gate

```mermaid
sequenceDiagram
    autonumber
    participant ME as method<br/>resolve.rs:763
    participant IO as internal_origin<br/>resolve.rs:806
    participant BN as by_name_only<br/>resolve.rs:846
    ME->>IO: is this derived value's origin internal code (resolve.rs:776)
    IO->>IO: self or untyped variable, yes (resolve.rs:811)
    IO->>IO: typed, field or returned, only if an internal type or fn (resolve.rs:818)
    IO->>IO: unknown, no (resolve.rs:821)
    IO-->>ME: true
    ME->>BN: method name
    BN->>BN: not in the common-name list (resolve.rs:847)
    BN->>BN: exactly one internal method of that name (resolve.rs:849)
    BN-->>ME: that method, Inferred (resolve.rs:851)
    Note over BN: otherwise sym:? name, External, resolve.rs:856
```

**What it shows.** A guess is made only for a distinctive name (not one of
about 125 common method names such as `get`, `insert`, `run`) that exists
exactly once among internal methods, and only for receivers whose origin is
internal or simply unknown.

**Why it is this way.** A plain variable of unevident type is treated as
possibly internal, so the guess still finds genuine internal calls through
untyped locals; anything traced to std or a dependency is excluded
(`crates/sealmap-rust/src/resolve.rs:801`-`805`).

**Debt:** an untyped variable counts as an internal origin
(`crates/sealmap-rust/src/resolve.rs:811`), so a part taken from a plain local
whose value is in fact a std or dependency type is still matched to a
distinctive internal method by name.

## EXT-05.6 Closure parameters are untyped

```mermaid
flowchart TB
    C["xs.iter().for_each with a closure<br/>whose parameter is conn"]
    AR["closure argument walked with the<br/>environment cloned, parameters<br/>never bound<br/>collect.rs:1307-1310"]
    RV["conn is not in the environment:<br/>Untyped<br/>collect.rs:1343"]
    BN["name guess on Untyped<br/>resolve.rs:774"]
    G["vacuum matched to the only internal<br/>vacuum, Inferred, though conn is a u8<br/>tests/extract.rs:351"]
    SH["a parameter that shadows an outer<br/>typed name keeps the outer type"]
    C --> AR --> RV --> BN --> G
    AR --> SH
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
(`crates/sealmap-rust/src/collect.rs:1307`-`1313`,
`crates/sealmap-rust/src/resolve.rs:774`), so `|conn| ... d.vacuum()` over a
slice of `u8` is drawn as an inferred call to `Db::vacuum`
(`crates/sealmap-rust/tests/extract.rs:351`).

**Debt:** a closure parameter that shadows an outer typed local is resolved
with the outer local's type, because the cloned environment still holds the
outer binding (`crates/sealmap-rust/src/collect.rs:1308`-`1309`).
