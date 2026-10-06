---
id: MOD-03
title: The codebase model, its sources and schema v2
area: model
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - crates/sealmap-model/src/lib.rs
  - crates/sealmap-model/src/codebase.rs
  - crates/sealmap-model/src/symbol.rs
  - crates/sealmap-model/src/flow.rs
  - crates/sealmap-model/src/path.rs
  - crates/sealmap-model/src/source.rs
  - crates/sealmap-model/Cargo.toml
  - docs/DESIGN.md
  - README.md
verified_commit: 4ed7a51f92f7241a1c420e49e8036a8d9189912a
---
## For developers

`sealmap-model` is the vocabulary both halves of the workspace meet in:
language adapters produce a `Codebase`, projections consume one, and neither
side knows the other (`crates/sealmap-model/src/lib.rs:6`-`10`). This topic
draws that value: files, symbols, relations and the ordered call `Flow` of
every callable; the `SourceSet` adapters read from instead of the disk; the
path normalisation that makes keys identical on every platform; and the
schema-v2 JSON reader that refuses anything else.

The crate's determinism contract has four clauses: ordered collections only,
normalised paths, stable hashing, no ambient data
(`crates/sealmap-model/src/lib.rs:18`-`34`). Every diagram here is one of
those clauses made concrete. Schema v2 came with commit `01f82b4`; v1 has no
reader because nothing outside the workspace consumed it
(`crates/sealmap-model/src/lib.rs:106`-`111`).

Not covered: the id grammar (MOD-01), the fingerprints (MOD-02), and how an
adapter fills the model (EXT-01 onwards).

## For the business

Everything sealmap claims about a codebase is a claim about this one value.
An adopter can persist it (`_model.json`), diff it, or feed it to a tool of
their own, and the same checkout produces the same bytes on any machine.
That is what makes the output safe to cache and cheap to compare in CI: no
timestamps, no machine paths, no user names.

Two properties lower risk directly. The loader honours the repository's own
ignore files, so vendored and generated trees do not inflate the model (the
design measured 47,229 files walked instead of 934 before this,
`docs/DESIGN.md:233`). And a model written by another version is refused
outright rather than half-read, so a stale artefact cannot be mistaken for a
current one.

## MOD-03.1 The model as types

```mermaid
sequenceDiagram
    participant CB as Codebase<br/>schema_version u32<br/>codebase.rs:18
    participant SF as SourceFile<br/>path language module hash lines<br/>source.rs:16
    participant SY as Symbol<br/>symbol.rs:192
    participant FL as Flow<br/>steps, flow.rs:40
    participant RE as Relation<br/>symbol.rs:325
    participant CO as Confidence, an enum<br/>symbol.rs:299
    Note over CB: files, a BTreeMap of path to SourceFile (codebase.rs:22)<br/>symbols, a BTreeMap of SymbolId to Symbol (codebase.rs:24)<br/>relations, a BTreeSet (codebase.rs:26)
    Note over CB,SF: Codebase *-- SourceFile
    Note over CB,SY: Codebase *-- Symbol
    Note over CB,RE: Codebase *-- Relation
    Note over SY,FL: Symbol o-- Flow, may carry a flow (symbol.rs:228)
    Note over RE,CO: Relation --> Confidence, carries a confidence (symbol.rs:325)
    Note over SY: id name kind visibility file span (symbol.rs:192)<br/>sig_hash body_hash (symbol.rs:209)<br/>parent signature doc generics tags (symbol.rs:214)<br/>members (symbol.rs:226)
```

**What it shows.** Three ordered maps or sets and nothing else. A symbol may
carry a flow; a relation always carries a confidence.

**Why it is this way.** Ordered collections make serialisation deterministic
and every query iterate in key order (`crates/sealmap-model/src/codebase.rs:12`-`14`).
Confidence is on every relation because there is no type checker behind the
model, and nothing is meant to be guessed silently (`README.md:318`-`320`).

**Invariant:** a relation from a symbol to itself is never stored
(`crates/sealmap-model/src/codebase.rs:91`).

**Debt:** `relations_from` is a range scan because `Relation` orders by
`from` first (`crates/sealmap-model/src/codebase.rs:130`-`133`), but
`relations_to`, `children` and `symbols_in_file` scan everything
(`crates/sealmap-model/src/codebase.rs:136`,
`crates/sealmap-model/src/codebase.rs:122`,
`crates/sealmap-model/src/codebase.rs:112`); projections that call them per
document have had to build their own indexes (COR-01).

## MOD-03.2 Loading a directory

```mermaid
sequenceDiagram
    autonumber
    participant RT as root directory
    participant WK as ignore walker, WalkBuilder<br/>source.rs:154
    participant SS as SourceSet<br/>source.rs:183
    RT->>WK: walk the root directory
    Note over WK: hidden skipped, parents off, no global<br/>or git exclude, no symlinks (source.rs:155-162)
    Note over WK: skip target, node_modules, vendor,<br/>dist, build, out below the root (source.rs:163)
    loop over the entries that survive
        alt extension wanted and under 2 MiB (source.rs:175-177)
            alt valid UTF-8 (source.rs:181)
                WK->>SS: path made relative and normalised (source.rs:182)
                SS-->>WK: text kept, newlines normalised (source.rs:183)
            else not valid UTF-8
                WK->>WK: skipped silently
            end
        else extension not wanted or too big
            WK->>WK: skipped silently
        end
    end
```

**What it shows.** `SourceSet::load_dir` walks with the `ignore` crate,
honouring `.gitignore` and `.ignore` files under the root only, then keeps
files by extension and size and stores their text with `\n` line endings.

**Why it is this way.** Reading only what is under the root, and never the
user's global excludes, keeps the result a function of the checkout's bytes
(`crates/sealmap-model/src/source.rs:53`-`57`). The ignore support was the
hardening fix for the 48-second VisionClaw walk (`docs/DESIGN.md:233`). It
also makes `ignore` a dependency of the model crate
(`crates/sealmap-model/Cargo.toml:21`), which DEL-01 follows to the MSRV.

**Debt:** a file that is not valid UTF-8 is skipped with no diagnostic
(`crates/sealmap-model/src/source.rs:181`), so it disappears from the model
and from every count, while one unreadable directory entry aborts the whole
load (`crates/sealmap-model/src/source.rs:170`).

## MOD-03.3 Normalising a path

```mermaid
sequenceDiagram
    autonumber
    participant RW as raw path text
    participant SP as SourcePath
    participant PE as PathError
    RW->>SP: raw path text
    SP->>SP: backslashes become slashes (path.rs:48)
    alt leading slash or drive letter (path.rs:49)
        SP-->>PE: PathError.Absolute
    else a relative path
        SP->>SP: split on slash (path.rs:53)
        SP->>SP: empty and dot parts dropped (path.rs:55)
        alt dot-dot with nothing to pop (path.rs:57)
            SP-->>PE: PathError.Escapes
        else every dot-dot pops
            alt nothing left (path.rs:64)
                SP-->>PE: PathError.Empty
            else parts remain
                SP-->>RW: parts joined with slash (path.rs:67)
            end
        end
    end
```

**What it shows.** Every path becomes relative, slash-separated and free of
`.` and `..` before it is used as a key, or it is refused.

**Why it is this way.** `src\lib.rs` on Windows and `src/lib.rs` on Linux
must be the same key for the corpus to be byte-identical across machines
(`crates/sealmap-model/src/lib.rs:27`-`29`). The same type names the 1:1
corpus documents, by appending a suffix (`crates/sealmap-model/src/path.rs:109`).

**Invariant:** no `SourcePath` can escape the codebase root, because a `..`
with nothing to pop is an error rather than a clamp
(`crates/sealmap-model/src/path.rs:57`-`58`).

## MOD-03.4 Reading a persisted model

```mermaid
sequenceDiagram
    autonumber
    participant CL as tool or agent
    participant FJ as from_json<br/>codebase.rs:47
    participant SJ as serde_json
    CL->>FJ: text of a _model.json
    FJ->>SJ: probe only schema_version and schema (codebase.rs:49)
    SJ-->>FJ: both optional numbers
    FJ->>FJ: take schema_version, else v1 schema (:54)
    alt not version 2
        FJ-->>CL: ModelJsonError.Version with found and expected (codebase.rs:56)
    else version 2
        FJ->>SJ: full deserialise (codebase.rs:58)
        SJ-->>FJ: Codebase, ids parsed as canonical sym text
        FJ-->>CL: Codebase
    end
```

**What it shows.** The reader looks at the version first, through a two-field
probe, and only then deserialises; a v1 file is reported as "found 1" rather
than failing on a missing field.

**Why it is this way.** Schema v2 changed the meaning of every id, so a v1
model read as v2 would be wrong in every row. The README states the refusal
for users (`README.md:203`-`206`). Every `SymbolId` inside is parsed through
the canonical-only parser, so a hand-edited id fails the whole read.

**Invariant:** `Codebase::new` always stamps the current schema version
(`crates/sealmap-model/src/codebase.rs:32`), so nothing written by this build
can fail its own reader on version.

## MOD-03.5 A flow is the skeleton of a sequence diagram

```mermaid
sequenceDiagram
    participant FL as Flow<br/>steps in source order<br/>flow.rs:38
    participant CL as Call<br/>target, label, kind, confidence,<br/>awaited, fallible, line<br/>flow.rs:165
    participant BR as Branch<br/>arms<br/>flow.rs:108
    participant LO as Loop<br/>label, body<br/>flow.rs:116
    participant OP as Optional<br/>label, body<br/>flow.rs:124
    participant PA as Parallel<br/>arms<br/>flow.rs:132
    participant RT as Return<br/>exit label and line<br/>flow.rs:138
    FL->>CL: a call is a leaf
    CL-->>FL: maps to a message arrow
    FL->>BR: an interior node
    BR-->>FL: maps to alt and else
    FL->>LO: an interior node
    LO-->>FL: maps to loop
    FL->>OP: an interior node
    OP-->>FL: maps to opt
    FL->>PA: an interior node
    PA-->>FL: maps to par and and
    FL->>RT: exit label and line (flow.rs:138)
    RT-->>FL: maps to a note
```

**What it shows.** Calls are leaves; branches, loops, optional blocks and
parallel arms are interior nodes that map one to one onto Mermaid fragments.

**Why it is this way.** A flow holds only what contains calls, so it is
already the minimal "who talks to whom, in what order, under which
condition" (`crates/sealmap-model/src/flow.rs:4`-`8`). Every call target is a
canonical id, which is what lets an orchestrator inline one function's
sequence into another's without guessing (`crates/sealmap-model/src/flow.rs:10`-`13`).

**Debt:** `Flow::calls` collects every call into a fresh vector before
iterating, and `call_count` builds that vector just to count it
(`crates/sealmap-model/src/flow.rs:55`-`64`); `Codebase::stats` does this
once per symbol (`crates/sealmap-model/src/codebase.rs:169`).
