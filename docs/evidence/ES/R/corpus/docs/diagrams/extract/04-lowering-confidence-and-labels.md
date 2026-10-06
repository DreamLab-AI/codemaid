---
id: EXT-04
title: Lowering, confidence and labels
area: extract
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - crates/sealmap-extract/src/lower.rs
  - crates/sealmap-extract/src/confidence.rs
  - crates/sealmap-extract/src/labels.rs
  - crates/sealmap-extract/src/raw.rs
  - docs/DESIGN.md
  - crates/sealmap-model/src/flow.rs
  - crates/sealmap-model/src/symbol.rs
  - crates/sealmap-rust/src/resolve.rs
  - crates/sealmap-rust/src/collect.rs
  - crates/sealmap-rust/src/tidy.rs
  - crates/sealmap-rust/src/lib.rs
  - README.md
verified_commit: 4ed7a51f92f7241a1c420e49e8036a8d9189912a
---
## For developers

Between the walker's raw IR (EXT-03) and the model's `Flow` (MOD-03) sit three
pieces of shared, language-neutral policy, all in `sealmap-extract`:

- **lowering** (`lower`): the shape rules that run once every call has been
  resolved or dropped (`crates/sealmap-extract/src/lower.rs:4`-`12`);
- **confidence** (`confidence`): the three-level evidence lattice, the policy
  for which external calls stay in a flow, and the aggregation of call sites
  into call relations (`crates/sealmap-extract/src/confidence.rs:1`-`7`);
- **labels** (`labels`): how source text becomes the short strings on arrows
  and fragments (`crates/sealmap-extract/src/labels.rs:1`-`7`).

They were extracted from the Rust adapter in `a02b381` with no behaviour
change, so a second adapter would draw flows, confidences and labels by the
same rules (the design calls this the parity lever, `docs/DESIGN.md:131`). The resolver
closure that lowering takes is the Rust adapter's `Resolver::call`, drawn in
EXT-04.4; how it finds targets is EXT-05.

## For the business

This is where sealmap is honest about what it does not know. Every call in
every diagram carries one of three tags: `exact` (resolved through explicit
paths, imports or declared types), `inferred` (matched by a weaker rule) or
`external` (outside the analysed code). Nothing is guessed silently
(`README.md:318`-`320`), and the generated sequence diagrams mark inferred
calls with a `~` so a reviewer can see which arrows to doubt (COR-01).

By default calls into the standard library and calls on values of unknown
type are left out, which keeps diagrams about *your* code and its
dependencies. The cost of that density is that a reviewer never sees those
calls; the policy can be widened to everything for debugging.

## EXT-04.1 Lowering a raw flow

```mermaid
sequenceDiagram
    autonumber
    participant RS as Resolver.flow<br/>resolve.rs:817
    participant LF as lower_flow<br/>lower.rs:36
    participant LS as lower_steps<br/>lower.rs:53
    participant CB as resolver closure<br/>resolve.rs:818
    RS->>LF: raw steps and a call resolver
    LF->>LS: lower the top level (lower.rs:40)
    loop each raw step
        LS->>CB: a raw call (lower.rs:61)
        CB-->>LS: model Call, or None to drop it
        LS->>LS: recurse into arms and bodies, drop empty ones (lower.rs:94)
    end
    LS-->>LF: model steps
    LF->>LF: drop a trailing top-level return (lower.rs:42)
    alt nothing but returns left
        LF-->>RS: None (lower.rs:47)
    else
        LF-->>RS: Flow (lower.rs:49)
    end
```

**What it shows.** Lowering is one recursive pass that asks the adapter's
closure about every call, then prunes: dropped calls disappear, fragments with
empty bodies disappear, a final `return` is just the end of the function, and a
flow with nothing but returns is no flow at all.

**Why it is this way.** Resolution decides which calls survive, and only after
that is it known which branches still say anything, so the shape rules have to
run after resolution, not in the walker (`crates/sealmap-extract/src/lower.rs:1`-`5`).

**Invariant:** a symbol whose body makes no kept call has `flow: None`, so
"has a sequence diagram" and "has a flow" are the same test
(`crates/sealmap-extract/src/lower.rs:46`-`48`).

## EXT-04.2 Collapsing a branch after resolution

```mermaid
sequenceDiagram
    autonumber
    participant BR as raw Branch<br/>with labelled arms (lower.rs:65)
    participant LO as the lowering
    participant OU as the model fragment
    BR->>LO: lower every arm,<br/>drop the empty ones (lower.rs:71)
    alt no arms left (lower.rs:73)
        LO-->>OU: nothing emitted (lower.rs:74)
    else one arm left
        alt the survivor is an unlabelled else<br/>after the first arm (lower.rs:78)
            LO-->>OU: Optional labelled<br/>not first-condition (lower.rs:79)
        else
            LO-->>OU: Optional labelled with its own<br/>condition, if prefix trimmed (lower.rs:81)
        end
    else two or more arms left
        LO-->>OU: Branch with the surviving arms (lower.rs:85)
    end
```

**What it shows.** `if cached { log() } else { fetch() }` with `log` dropped as
std becomes an `opt` labelled `not cached` around `fetch()`; a branch where
every arm lost its calls vanishes.

**Why it is this way.** The raw IR keeps empty arms precisely so this rule can
name the condition that guards the survivor
(`crates/sealmap-extract/src/raw.rs:32`-`34`, EXT-03.1); without the
labels an `opt` would read as unconditional.

**Drift (flow.rs vs lower.rs):** the model's doc says arms without calls are
kept "with empty steps" when they matter for reading the branch
(`crates/sealmap-model/src/flow.rs:109`-`111`); lowering removes every arm
whose lowered steps are empty (`crates/sealmap-extract/src/lower.rs:71`), and
an early-return `else` survives only because its `Return` step is not empty.

## EXT-04.3 The confidence lattice and the external policy

```mermaid
sequenceDiagram
    autonumber
    participant CO as Confidence,<br/>strongest first, symbol.rs:299
    participant EK as ExternalCalls.keeps<br/>confidence.rs:46
    participant FL as the flow
    CO->>EK: a call's evidence
    Note over CO: Exact, marker equals, symbol.rs:302<br/>Inferred, marker tilde, symbol.rs:304<br/>External, marker question, symbol.rs:306
    alt Exact or Inferred (confidence.rs:48)
        EK->>FL: kept in the flow
    else External
        alt policy All
            EK->>FL: every external call,<br/>std and sym:? included (confidence.rs:30)
        else policy NonStd, the default
            opt it looks like a dependency
                EK->>FL: kept (confidence.rs:51)
            end
        else policy None
            Note over EK: internal calls only (confidence.rs:52)
        end
    end
```

**What it shows.** Confidence is totally ordered from strongest to weakest
evidence; exact and inferred calls always stay; an external call stays
according to the policy, and under the default only when the adapter says its
target is a real dependency.

**Why it is this way.** The order is the enum's declaration order, so "the
stronger of two" is simply the minimum (`crates/sealmap-extract/src/confidence.rs:59`-`61`).
The dependency test is a closure evaluated only for external calls, so the
policy stays language-neutral while the adapter decides what "dependency"
means (`crates/sealmap-extract/src/confidence.rs:41`-`45`).

**Invariant:** one call relation exists per distinct caller and target, with
the strongest confidence of the call sites behind it
(`crates/sealmap-extract/src/confidence.rs:66`-`78`).

## EXT-04.4 The Rust adapter's keep-or-drop decision

```mermaid
sequenceDiagram
    autonumber
    participant LS as lower_steps<br/>lower.rs:53
    participant CA as Resolver.call<br/>resolve.rs:821
    participant ME as Resolver.method<br/>resolve.rs:858
    participant EC as ExternalCalls.keeps<br/>confidence.rs:46
    LS->>CA: raw call
    alt path call
        CA->>CA: Self outside an impl, drop, else Self and self go on (resolve.rs:830-831)
        CA->>CA: prelude name not shadowed, drop (resolve.rs:833)
        CA->>CA: resolve the path in the value namespace (resolve.rs:836)
        CA->>CA: bare unknown local name, drop as a closure (resolve.rs:839)
    else method call
        CA->>ME: receiver and name (resolve.rs:844)
        ME-->>CA: target and confidence, or None to drop
    end
    CA->>EC: confidence, plus a dependency test (resolve.rs:846)
    EC-->>CA: keep or not
    Note over CA: a dependency is a lower-case external root<br/>that is not a workspace crate, resolve.rs:851
    CA-->>LS: Call with target, label, kind, flags, line (resolve.rs:853)
```

**What it shows.** Path calls to prelude names (`Some`, `Vec::new`, `drop`)
never reach the policy; everything else is resolved, then the policy decides
whether an external target counts as a dependency.

**Why it is this way.** The Rust adapter's documentation states the default
plainly: std, prelude constructors and calls on receivers of unknown type are
dropped so diagrams stay dense (`crates/sealmap-rust/src/lib.rs:64`-`68`).

**Debt:** "looks like a dependency" is a lower-case first letter on the
target's root (`crates/sealmap-rust/src/resolve.rs:851`), so an unresolved
lower-case path that is a local module alias or a misspelling is kept and drawn
as a dependency lane.

## EXT-04.5 From tokens to a label

```mermaid
sequenceDiagram
    autonumber
    participant T as a syn node
    participant SQ as squeeze, labels.rs:57
    participant LA as the label writers
    T->>SQ: token stream printed with a space<br/>between every token (tidy.rs:8)
    SQ->>SQ: fixed-point spacing rules, then call<br/>parens and arrows restored (labels.rs:57)
    T->>LA: argument sketch, names and short<br/>literals kept, anything else underscore<br/>(collect.rs:1459)
    LA->>LA: call_label, name of sketches clipped<br/>to 56 chars (labels.rs:116)
    LA->>LA: condition_label, clipped as if<br/>prefixed with if (labels.rs:122)
    LA->>LA: clip on a char boundary<br/>with an ellipsis (labels.rs:104)
```

**What it shows.** Labels are built from printed tokens folded back into the
spacing a person would write, with arguments reduced to a sketch and every
label clipped to a fixed budget.

**Why it is this way.** Printed token streams put a space between almost
every token (`Vec < String >`); the rules are purely textual, so the result
depends only on the text (`crates/sealmap-extract/src/labels.rs:4`-`7`). The
limits are shared constants: 56 characters on arrows and fragments, 200 for a
signature, 14 for a literal argument, 160 for a doc summary
(`crates/sealmap-extract/src/labels.rs:17`-`24`).

**Invariant:** a guard and the `if` arm it labels clip identically, because
the condition is clipped with its `if ` prefix and the prefix removed after
(`crates/sealmap-extract/src/labels.rs:122`-`124`).
