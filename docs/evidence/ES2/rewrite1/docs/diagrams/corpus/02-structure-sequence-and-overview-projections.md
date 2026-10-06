---
id: COR-02
title: Structure, sequence and overview projections
area: corpus
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - crates/sealmap-corpus/src/sequence.rs
  - crates/sealmap-corpus/src/naming.rs
  - crates/sealmap-corpus/src/structure.rs
  - crates/sealmap-corpus/src/overview.rs
  - crates/sealmap-corpus/src/lib.rs
  - crates/sealmap-corpus/src/document.rs
  - crates/sealmap-corpus/src/corpus_readme.md
  - crates/sealmap-corpus/tests/contract.rs
  - crates/sealmap-model/src/codebase.rs
  - docs/DESIGN.md
verified_commit: 4ed7a51f92f7241a1c420e49e8036a8d9189912a
---
## For developers

Three projections turn the model into Mermaid through the typed writers
(MER-01) and the injective ids (MER-02):

- **sequence** (`crates/sealmap-corpus/src/sequence.rs:17`): one
  `sequenceDiagram` per callable whose flow makes at least `min_calls` calls,
  with lanes for the caller's owner, the internal types it talks to and one
  lane per external crate;
- **structure** (`crates/sealmap-corpus/src/structure.rs:14`): one
  `classDiagram` per file of what it defines, with stubs for what it touches;
- **overview** (`crates/sealmap-corpus/src/overview.rs:14`): the crate graph,
  per-crate module graphs, data models as `erDiagram` and a trait map, each
  capped so Mermaid can render it.

The agent-facing legend for all three ships inside every corpus as
`_README.md` (`crates/sealmap-corpus/src/corpus_readme.md:19`-`30`). The
dogfood fix in `6c8a9b0` (no doubled owner on external path calls) is the most
recent change here.

## For the business

These are the diagrams an agent actually reads when it consults the generated
corpus, so their honesty is what an adopter is relying on. Two conventions
carry most of it. A `~` before a message means the callee was inferred, not
proven. A lane marked `ext` is outside the codebase, so an arrow into it is a
dependency call, not your code.

The caps are a deliberate trade: a diagram over its message or edge budget is
cut, with a note saying how much was left out and where the full list is,
rather than rendered unreadably or not at all. Long functions and hub crates
are therefore always summarised in the diagrams; the complete call lists stay
in the index.

## COR-02.1 Rendering one callable

```mermaid
sequenceDiagram
    autonumber
    participant DR as document render<br/>sealmap-corpus/src/document.rs:59
    participant SR as sequence render<br/>sequence.rs:17
    participant ST as Ctx.steps<br/>sequence.rs:66
    participant LO as lane_of<br/>naming.rs:42
    DR->>SR: symbol with a flow
    SR->>SR: fewer calls than min_calls, no diagram (sequence.rs:19)
    SR->>SR: first lane is the caller's owner (sequence.rs:22)
    SR->>ST: flow steps (sequence.rs:27)
    loop each call while budget lasts
        ST->>LO: target (sequence.rs:76)
        LO-->>ST: lane, alias, prefix, external flag
        ST->>ST: tilde for inferred, prefix, label, await, question mark (sequence.rs:80)
    end
    ST->>ST: branches alt, spawned par, loops, opt, returns as notes (sequence.rs:98)
    SR->>SR: external lanes aliased with ext (sequence.rs:32)
    SR->>SR: overflow note, more calls in _index.json (sequence.rs:40)
    SR-->>DR: text, participants, truncated count
```

**What it shows.** Every message comes from the caller's owner; each target is
mapped to a lane; inferred calls get a `~`, awaited calls `.await`, fallible
calls `?`; control-flow steps become fragments and early returns become notes.

**Why it is this way.** The first lane is the owner (the type for a method,
the module for a free function), the participant a symbol belongs to in a
sequence (`crates/sealmap-model/src/codebase.rs:145`-`147`). A path call's
label already names its owner, so the lane prefix is added only when it is not
already there (`crates/sealmap-corpus/src/sequence.rs:83`-`88`), the fix for
`Hasher::Hasher::new_derive_key` pinned by
`external_path_calls_are_not_double_qualified`
(`crates/sealmap-corpus/tests/contract.rs:219`).

**Debt:** a sequence over `max_messages` (default 80,
`crates/sealmap-corpus/src/lib.rs:162`) drops every later call from the
diagram, wherever it sits in the flow, and says only how many
(`crates/sealmap-corpus/src/sequence.rs:71`-`73`); the dropped calls are
recoverable from `_index.json`, not from the diagram.

## COR-02.2 Which lane a call lands on

```mermaid
sequenceDiagram
    autonumber
    participant T as a call target
    participant LO as lane_of, naming.rs:42
    participant LN as the lane it lands on
    T->>LO: which lane?
    alt defined in the codebase (naming.rs:43)
        LO-->>LN: its owner, type or module (naming.rs:44)
    else sym:? unresolved (naming.rs:47)
        LO-->>LN: one shared lane named ? (naming.rs:49)
    else parent internal (naming.rs:54)
        LO-->>LN: the parent's lane (naming.rs:55)
    else external target, external lanes option (naming.rs:59)
        alt CrateRoot, the default (naming.rs:60-64)
            LO-->>LN: one lane per crate root, owner<br/>shown as a message prefix
        else Owner (naming.rs:66-68)
            LO-->>LN: one lane per external owner
        end
    end
```

**What it shows.** Internal targets land on their owner's lane, members the
code does not define land on their internal type's lane, and external targets
land on one lane per crate (the densest option) or one per external owner.

**Why it is this way.** One lane per external crate keeps diagrams narrow; the
owner is kept in the message text (`tokio::fs::read` on a `tokio` lane), so no
information is lost (`crates/sealmap-corpus/src/lib.rs:120`-`128`).

**Debt:** every call on a receiver of unknown type shares one `?` lane
(`crates/sealmap-corpus/src/naming.rs:47`-`49`), so unrelated unknown
receivers in one function are drawn as a single participant; under the default
external policy such calls are dropped before they reach a diagram.

## COR-02.3 Assembling a file's structure diagram

```mermaid
sequenceDiagram
    participant H as symbols in the file<br/>private ones unless include_private is off<br/>structure.rs:20
    participant OW as owners<br/>types defined here, then foreign<br/>types this file adds methods to<br/>structure.rs:29-42
    participant CL as one class per owner<br/>kind annotation, fields, variants,<br/>assoc items, methods with a Trait prefix<br/>structure.rs:43-83
    participant MB as module box of free fns, consts,<br/>statics, macros, submodules, only if<br/>non-empty or nothing else is drawn<br/>structure.rs:89-127
    participant ED as edges from what was drawn<br/>implements, extends, field, uses<br/>structure.rs:134-170
    participant SUB as a field edge subsumes a uses edge<br/>between the same pair<br/>structure.rs:172-176
    participant STUB as stubs for targets defined elsewhere<br/>kind in file, or external<br/>structure.rs:178-190
    H->>OW: grouped by owner
    OW->>CL: one class per owner
    CL->>MB: then the module box
    MB->>ED: then the edges
    ED->>SUB: field edges first
    SUB->>STUB: stubs for the rest
```

**What it shows.** A file's diagram draws the types it defines and the foreign
types it extends with methods, a module box for everything else, and edges to
stubs for what it refers to elsewhere.

**Why it is this way.** A module that only holds types is already described by
them, so its box is omitted (`crates/sealmap-corpus/src/structure.rs:123`-`127`);
methods aggregate their relations to the owner type, so the diagram stays one
box per type (`crates/sealmap-corpus/src/structure.rs:133`-`141`).

**Debt:** whether a field is drawn as aggregation or composition is a
substring test on its type text (`crates/sealmap-corpus/src/structure.rs:235`-`250`),
so a user type whose name ends in `Vec` or `Option`, written with generics,
is drawn as a shared holder.

## COR-02.4 Relations as class arrows

```mermaid
sequenceDiagram
    participant RK as the relation kind
    participant CA as the class arrow, structure.rs:192
    alt Implements
        RK->>CA: realisation, dotted triangle (structure.rs:192)
    else Extends
        RK->>CA: inheritance (structure.rs:193)
    else FieldType behind Option, Arc, Rc, a reference or a collection
        RK->>CA: aggregation, labelled with the fields (structure.rs:194)
    else FieldType, owned
        RK->>CA: composition, labelled with the fields (structure.rs:195)
    else Uses, in a signature
        RK->>CA: dependency, dotted arrow (structure.rs:196)
    end
```

**What it shows.** The five structural relation kinds map one to one onto
Mermaid class arrows, with field edges labelled by the field names that carry
them (`crates/sealmap-corpus/src/structure.rs:147`-`154`).

**Why it is this way.** The same legend is written into every corpus for the
agents that read it (`crates/sealmap-corpus/src/corpus_readme.md:22`-`23`).

## COR-02.5 The overview and its caps

```mermaid
sequenceDiagram
    participant OV as _overview.md<br/>overview.rs:14
    participant CG as crate graph<br/>structural relations between crate roots,<br/>grouped by top directory when several repos<br/>overview.rs:77
    participant MG as module graph per crate<br/>overview.rs:131
    participant DM as data model per crate as erDiagram<br/>highest-degree entities first<br/>overview.rs:166
    participant TM as trait map<br/>implements and extends into internal traits<br/>overview.rs:230
    participant HV as heaviest max_edges edges kept<br/>omitted count as a comment<br/>overview.rs:68, overview.rs:125
    participant ME as max_entities kept<br/>omitted count as a comment<br/>overview.rs:190, overview.rs:209
    participant TT as first max_edges relations in key order<br/>no omitted count<br/>overview.rs:236
    OV->>CG: the first view (overview.rs:77)
    CG->>HV: capped
    OV->>MG: the second view (overview.rs:131)
    MG->>HV: capped
    OV->>DM: the third view (overview.rs:166)
    DM->>ME: capped
    OV->>TM: the fourth view (overview.rs:230)
    TM->>TT: capped
```

**What it shows.** Four kinds of cross-cutting view, each with a cap: edges
by weight for the graphs, entities by degree for the data models.

**Why it is this way.** Mermaid's default edge limit is 500, so the default
cap is 300 (`crates/sealmap-corpus/src/lib.rs:141`-`143`); unresolved targets
have no crate root and are left out of the crate graph
(`crates/sealmap-corpus/src/overview.rs:81`-`82`).

**Debt:** the trait map truncates to the first `max_edges` relations in
relation order rather than the heaviest, and writes no omitted count
(`crates/sealmap-corpus/src/overview.rs:236`), unlike the graphs and data
models, so a cut trait map is indistinguishable from a complete one.
