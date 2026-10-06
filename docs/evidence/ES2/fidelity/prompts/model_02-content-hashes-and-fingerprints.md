You are a blind fidelity judge for a diagram rewrite. Work alone and read only the two files named below.

- ORIGINAL topic: /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/fidelity/inputs/model_02-content-hashes-and-fingerprints/original.md
- REWRITE of the same topic: /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/fidelity/inputs/model_02-content-hashes-and-fingerprints/rewrite.md

Both are Markdown topic files from a diagrams-as-code corpus: prose sections plus ```mermaid diagrams that cite source code as `path:line`. The rewrite was supposed to turn every non-sequence diagram (flowchart, classDiagram, stateDiagram) into one or more `sequenceDiagram` blocks that carry the same facts and the same `path:line` citations, keep all prose verbatim, and add nothing that is not in the original topic. Extra diagram sections with new headings are allowed.

Compare the DIAGRAMS. For every original diagram that the rewrite changed, list each fact it states: a component, a call or message, an order, a condition or branch, a value, cap or constant, a type, field or variant, a state or transition, an outcome, or a relationship. Then check whether the rewrite's diagrams still state it, in any form (a participant, message, note, alt/opt/loop/break block). Then list each fact the rewrite's diagrams state that appears nowhere in the ORIGINAL topic, in its diagrams or its prose.

- facts_dropped: facts in an original diagram that no rewrite diagram states. Restated, merged or moved into a note does not count as dropped.
- facts_invented: facts in a rewrite diagram that the original topic does not state anywhere. Sequence scaffolding, such as a "caller" participant or an activation, is not a fact. An explicit claim of a call, order, condition, value, outcome or relationship that the original does not make is invented. So is a change of meaning, such as a reversed order, a different condition or the wrong outcome.
- citations_dropped: each `path:line` (or `path:a-b`, or a bare `file.rs:N`) present in an original diagram but absent from every rewrite diagram.
- citations_added: each citation in a rewrite diagram that appears nowhere in the original topic.
- prose_changed: true if any text outside the mermaid fences differs, apart from added headings and their new diagram sections.

Be exact and conservative. Quote the original or rewrite text for each fact (short). Do not judge whether the facts are true of the code.

Your answer is exactly one JSON object:
{"topic": "model/02-content-hashes-and-fingerprints.md", "diagrams_rewritten": <n>, "facts_checked": <n>, "facts_dropped": [{"diagram": "<id>", "fact": "<short quote>"}], "facts_invented": [{"diagram": "<id>", "fact": "<short quote>", "why": "<one line>"}], "citations_dropped": ["<cite>"], "citations_added": ["<cite>"], "prose_changed": true|false, "notes": "<one line>"}
Reply with exactly that JSON object as your whole final message, and nothing else.

The shell is unavailable in this sandbox; the two files' contents follow.

----- /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/fidelity/inputs/model_02-content-hashes-and-fingerprints/original.md -----
````markdown
---
id: MOD-02
title: Content hashes and per-symbol fingerprints
area: model
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - crates/sealmap-model/src/hash.rs
  - crates/sealmap-model/src/symbol.rs
  - crates/sealmap-corpus/src/seal/check.rs
  - crates/sealmap-model/src/codebase.rs
  - crates/sealmap-model/src/source.rs
  - crates/sealmap-extract/src/fingerprint.rs
  - docs/DESIGN.md
  - README.md
verified_commit: 4ed7a51f92f7241a1c420e49e8036a8d9189912a
---
## For developers

sealmap keeps two kinds of hash and never confuses them. A `ContentHash` is
BLAKE3 over a whole file's text with line endings normalised, printed
`blake3:<64 hex>`; it is what the 0.1 corpus uses to tell a stale document
from a hand-edited one. A `Fingerprint` is 16 bytes of a keyed BLAKE3 digest,
printed `blake3-16:<32 hex>`, and every symbol carries two of them:
`sig_hash` for its contract and `body_hash` for its implementation
(`crates/sealmap-model/src/symbol.rs:209`, `crates/sealmap-model/src/symbol.rs:212`).

The model crate only stores, compares, prints, parses and merges these values
(`crates/sealmap-model/src/hash.rs:61`-`65`). How the token streams that
feed a fingerprint are chosen per kind of symbol is the extraction core's job
and is drawn in EXT-06. After this topic you should know what each value
means, how two `#[cfg]` twins that share an id end up with one fingerprint,
and what the pair lets the seal check decide (built; drawn in COR-04).

The fingerprints arrived in commit `0a304ba` (step 2 of the 0.2 plan,
`docs/DESIGN.md:296`), after rustc's split between a definition's identity
and its query fingerprints (`docs/DESIGN.md:130`).

## For the business

Keeping a reviewed diagram true has a cost, and the expensive part today is
deciding *whether* a change touched what a topic describes. A content hash
per file answers that coarsely: any edit anywhere in a file flags every topic
that cites it. Per-symbol fingerprints answer it per function, and they answer
two questions rather than one: did the function's contract change, or only its
behaviour?

That split is what lets an adopter route maintenance by cost. A behaviour
change is a cheap re-review; a contract change is worth a more careful look.
The values are deliberately blind to formatting and comments, so a
reformatting commit costs nothing at all. The tooling that acts on this split
(seals, `sealmap verify` and `sealmap stale`) now exists (COR-04); this
repository's own topics are not sealed yet.

## MOD-02.1 The two hash types

```mermaid
classDiagram
    direction LR
    class ContentHash {
        +String blake3 prefix and 64 hex  hash.rs:21
        +of_text(text) normalises newlines  hash.rs:25
        +of_bytes(bytes) raw  hash.rs:31
        +parse(s) Option  hash.rs:38
        +short(n) str  hash.rs:49
    }
    class Fingerprint {
        +16 bytes  hash.rs:85
        +PREFIX blake3-16  hash.rs:89
        +from_blake3(hash) truncates  hash.rs:97
        +is_unset() all zeros  hash.rs:109
        +parse(s) Option  hash.rs:114
        +merge(other) order independent  hash.rs:138
    }
    class SourceFile {
        +hash ContentHash  source.rs:24
    }
    class Symbol {
        +sig_hash Fingerprint  symbol.rs:209
        +body_hash Fingerprint  symbol.rs:212
    }
    SourceFile --> ContentHash
    Symbol --> Fingerprint
```

**What it shows.** Files carry a full-length content hash; symbols carry two
truncated fingerprints. Both print with a self-describing algorithm prefix.

**Why it is this way.** The prefix keeps the algorithm visible in every
model and lock if it ever changes (`crates/sealmap-model/src/hash.rs:8`-`9`).
Truncating to 128 bits keeps an accidental collision out of reach while
halving the text a full digest costs in every model and lock
(`crates/sealmap-model/src/hash.rs:67`-`69`).

**Invariant:** the default fingerprint is all zeros and means "not
fingerprinted" (`crates/sealmap-model/src/hash.rs:109`-`111`); a symbol built
by hand starts there (`crates/sealmap-model/src/symbol.rs:244`-`245`) until an
adapter fills it in.

## MOD-02.2 From bytes to printed value

```mermaid
flowchart TB
    subgraph FILE["file text"]
        T["source text"]
        NL["CR LF and lone CR become LF<br/>normalise_newlines, hash.rs:171"]
        B3["blake3 hash, full 32 bytes<br/>hash.rs:32"]
        CH["blake3 colon 64 hex"]
        T --> NL --> B3 --> CH
    end
    subgraph SYM["symbol tokens"]
        TK["token stream from an adapter"]
        KD["keyed hasher, sig or body context<br/>fingerprint.rs:129"]
        TR["first 16 bytes kept<br/>from_blake3, hash.rs:99"]
        FP["blake3-16 colon 32 hex<br/>Display, hash.rs:149"]
        TK --> KD --> TR --> FP
    end
    FILE --> SYM
```

**What it shows.** A file hash normalises newlines and hashes everything; a
symbol fingerprint hashes a token stream under one of two key-derivation
contexts and keeps the first 16 bytes.

**Why it is this way.** Distinct contexts mean a signature and a body with
identical tokens still get different values
(`crates/sealmap-extract/src/fingerprint.rs:40`-`41`). The context strings
embed the algorithm id and a date, so any change to the scheme is a new
algorithm id (`crates/sealmap-extract/src/fingerprint.rs:77`).

**Invariant:** golden values pin the scheme; an encoding change fails
`golden_values` until `ALGORITHM` is bumped
(`crates/sealmap-extract/src/fingerprint.rs:269`).

## MOD-02.3 Merging cfg twins

```mermaid
sequenceDiagram
    autonumber
    participant AD as adapter
    participant CB as add_symbol<br/>codebase.rs:71
    participant FP as Fingerprint.merge<br/>hash.rs:138
    AD->>CB: first definition of an id
    CB->>CB: insert as is (codebase.rs:84)
    AD->>CB: second definition, same id, other cfg
    CB->>FP: fold sig_hash (codebase.rs:74)
    FP->>FP: sort the pair, keyed hash of both (hash.rs:139)
    FP-->>CB: merged sig_hash
    CB->>FP: fold body_hash (codebase.rs:75)
    FP-->>CB: merged body_hash
    CB->>CB: extend members, add new tags (codebase.rs:76)
    Note over CB: DEBT: scalar fields such as file and span<br/>keep the first twin only, codebase.rs:67
```

**What it shows.** Two definitions with one id, typically
`#[cfg(unix)]` and `#[cfg(windows)]` twins, become one symbol whose
fingerprints are an order-independent fold of both.

**Why it is this way.** A seal on a cfg-gated function has to notice an edit
to either twin; folding both values with a dedicated key context means an
edit to either changes the result, whichever twin was seen first
(`crates/sealmap-model/src/hash.rs:126`-`130`).

**Debt:** the merged symbol keeps only the first twin's file, span, signature
and doc (`crates/sealmap-model/src/codebase.rs:66`-`70`), so the second
definition has no location anywhere in the model.

## MOD-02.4 A fingerprint's life

```mermaid
stateDiagram-v2
    [*] --> Unset: Symbol.new, symbol.rs 244
    Unset --> Set: adapter assigns sig and body
    Set --> Merged: a cfg twin with the same id arrives
    Merged --> Merged: another twin
    Set --> Printed: serialised as blake3-16 text
    Merged --> Printed
    Printed --> Set: parsed back, either case accepted
    Printed --> Refused: wrong prefix or length
```

**What it shows.** A fingerprint starts unset, is set by an adapter, may be
merged with twins, and travels as text in the JSON model; text with the wrong
prefix or length is refused on read.

**Why it is this way.** Serialisation goes through `Display` and `parse`
rather than serde's byte encoding (`crates/sealmap-model/src/hash.rs:157`-`168`),
so the JSON is readable and identical to what a lock would hold.

**Invariant:** a malformed fingerprint string fails deserialisation instead of
reading as zeros (`crates/sealmap-model/src/hash.rs:166`).

## MOD-02.5 What the pair decides

```mermaid
flowchart TB
    S["a sealed symbol: id, sig, body<br/>check.rs:138"]
    R{"is the id in the model?"}
    PU{"is it the module of a file<br/>tagged parse_error?<br/>check.rs:146"}
    SG{"sig_hash equal?<br/>check.rs:147"}
    BD{"body_hash equal?<br/>check.rs:150"}
    AU{"is an ancestor module<br/>tagged parse_error?<br/>check.rs:155"}
    UNP["unparsable, fail closed"]
    HOLDS["holds<br/>check.rs:153"]
    BEH["behaviour, body only"]
    CON["contract"]
    ABS["absent: same-kind ids with the sealed<br/>body are rename candidates<br/>check.rs:158"]
    S --> R
    R -->|yes| PU
    PU -->|yes| UNP
    PU -->|no| SG
    SG -->|no| CON
    SG -->|yes| BD
    BD -->|yes| HOLDS
    BD -->|no| BEH
    R -->|no| AU
    AU -->|yes| UNP
    AU -->|no| ABS
```

**What it shows.** The decision the seal check takes from the two
fingerprints: a body-only change is behaviour, a signature change is
contract, and a vanished id whose body reappears in a symbol of the same kind
is a suspected rename. Unparsable code is decided first, so it is never
reported as holding or absent (`crates/sealmap-corpus/src/seal/check.rs:146`,
`crates/sealmap-corpus/src/seal/check.rs:155`); the design's table is
`docs/DESIGN.md:99`-`107`.

**Why it is this way.** The name sits in `sig_hash` and never in `body_hash`,
which is what makes a rename detectable by body
(`crates/sealmap-extract/src/fingerprint.rs:26`-`27`); the README states the
same table for users (`README.md:189`-`196`).

The crate-surface table records that an optional `flow_hash` was considered
and not built (`docs/DESIGN.md:130`); the model has only the two
(`crates/sealmap-model/src/symbol.rs:209`-`212`), as commit `0a304ba` left it.
````

----- /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/es2/fidelity/inputs/model_02-content-hashes-and-fingerprints/rewrite.md -----
````markdown
---
id: MOD-02
title: Content hashes and per-symbol fingerprints
area: model
governing: [docs/DESIGN.md, README.md]
adrs: []
sources:
  - crates/sealmap-model/src/hash.rs
  - crates/sealmap-model/src/symbol.rs
  - crates/sealmap-corpus/src/seal/check.rs
  - crates/sealmap-model/src/codebase.rs
  - crates/sealmap-model/src/source.rs
  - crates/sealmap-extract/src/fingerprint.rs
  - docs/DESIGN.md
  - README.md
verified_commit: 4ed7a51f92f7241a1c420e49e8036a8d9189912a
---
## For developers

sealmap keeps two kinds of hash and never confuses them. A `ContentHash` is
BLAKE3 over a whole file's text with line endings normalised, printed
`blake3:<64 hex>`; it is what the 0.1 corpus uses to tell a stale document
from a hand-edited one. A `Fingerprint` is 16 bytes of a keyed BLAKE3 digest,
printed `blake3-16:<32 hex>`, and every symbol carries two of them:
`sig_hash` for its contract and `body_hash` for its implementation
(`crates/sealmap-model/src/symbol.rs:209`, `crates/sealmap-model/src/symbol.rs:212`).

The model crate only stores, compares, prints, parses and merges these values
(`crates/sealmap-model/src/hash.rs:61`-`65`). How the token streams that
feed a fingerprint are chosen per kind of symbol is the extraction core's job
and is drawn in EXT-06. After this topic you should know what each value
means, how two `#[cfg]` twins that share an id end up with one fingerprint,
and what the pair lets the seal check decide (built; drawn in COR-04).

The fingerprints arrived in commit `0a304ba` (step 2 of the 0.2 plan,
`docs/DESIGN.md:296`), after rustc's split between a definition's identity
and its query fingerprints (`docs/DESIGN.md:130`).

## For the business

Keeping a reviewed diagram true has a cost, and the expensive part today is
deciding *whether* a change touched what a topic describes. A content hash
per file answers that coarsely: any edit anywhere in a file flags every topic
that cites it. Per-symbol fingerprints answer it per function, and they answer
two questions rather than one: did the function's contract change, or only its
behaviour?

That split is what lets an adopter route maintenance by cost. A behaviour
change is a cheap re-review; a contract change is worth a more careful look.
The values are deliberately blind to formatting and comments, so a
reformatting commit costs nothing at all. The tooling that acts on this split
(seals, `sealmap verify` and `sealmap stale`) now exists (COR-04); this
repository's own topics are not sealed yet.

## MOD-02.1 The two hash types

```mermaid
sequenceDiagram
    participant SF as SourceFile<br/>hash ContentHash<br/>source.rs:24
    participant CH as ContentHash<br/>blake3 prefix and 64 hex<br/>hash.rs:21
    participant SY as Symbol<br/>symbol.rs:209
    participant FP as Fingerprint<br/>16 bytes, prefix blake3-16<br/>hash.rs:85
    SF->>CH: carries one whole-file content hash (source.rs:24)
    Note over CH: of_text normalises newlines (hash.rs:25)<br/>of_bytes is raw (hash.rs:31)<br/>parse (hash.rs:38)<br/>short(n) (hash.rs:49)
    SY->>FP: carries sig_hash (symbol.rs:209) and body_hash (symbol.rs:212)
    Note over FP: from_blake3 truncates (hash.rs:97)<br/>is_unset is all zeros (hash.rs:109)<br/>parse (hash.rs:114)<br/>merge is order independent (hash.rs:138)
```

**What it shows.** Files carry a full-length content hash; symbols carry two
truncated fingerprints. Both print with a self-describing algorithm prefix.

**Why it is this way.** The prefix keeps the algorithm visible in every
model and lock if it ever changes (`crates/sealmap-model/src/hash.rs:8`-`9`).
Truncating to 128 bits keeps an accidental collision out of reach while
halving the text a full digest costs in every model and lock
(`crates/sealmap-model/src/hash.rs:67`-`69`).

**Invariant:** the default fingerprint is all zeros and means "not
fingerprinted" (`crates/sealmap-model/src/hash.rs:109`-`111`); a symbol built
by hand starts there (`crates/sealmap-model/src/symbol.rs:244`-`245`) until an
adapter fills it in.

## MOD-02.2 From bytes to printed value

```mermaid
sequenceDiagram
    autonumber
    participant TX as file text<br/>source text
    participant NL as normalise_newlines<br/>hash.rs:171
    participant B3 as the blake3 hash<br/>hash.rs:32
    participant TK as symbol tokens<br/>token stream from an adapter
    participant KD as keyed hasher, sig or body context<br/>fingerprint.rs:129
    participant TR as first 16 bytes kept<br/>from_blake3, hash.rs:99
    participant FP as the printed fingerprint<br/>blake3-16 colon 32 hex<br/>Display, hash.rs:149
    Note over TX,FP: one pipeline per hash kind, file text first, then symbol tokens
    TX->>NL: CR LF and lone CR become LF (hash.rs:171)
    NL->>B3: hash the normalised text, full 32 bytes (hash.rs:32)
    B3-->>TX: printed as blake3, 64 hex
    TK->>KD: hash the token stream (fingerprint.rs:129)
    KD->>TR: from_blake3 keeps the first 16 bytes (hash.rs:99)
    TR->>FP: printed as blake3-16, 32 hex (Display, hash.rs:149)
```

**What it shows.** A file hash normalises newlines and hashes everything; a
symbol fingerprint hashes a token stream under one of two key-derivation
contexts and keeps the first 16 bytes.

**Why it is this way.** Distinct contexts mean a signature and a body with
identical tokens still get different values
(`crates/sealmap-extract/src/fingerprint.rs:40`-`41`). The context strings
embed the algorithm id and a date, so any change to the scheme is a new
algorithm id (`crates/sealmap-extract/src/fingerprint.rs:77`).

**Invariant:** golden values pin the scheme; an encoding change fails
`golden_values` until `ALGORITHM` is bumped
(`crates/sealmap-extract/src/fingerprint.rs:269`).

## MOD-02.3 Merging cfg twins

```mermaid
sequenceDiagram
    autonumber
    participant AD as adapter
    participant CB as add_symbol<br/>codebase.rs:71
    participant FP as Fingerprint.merge<br/>hash.rs:138
    AD->>CB: first definition of an id
    CB->>CB: insert as is (codebase.rs:84)
    AD->>CB: second definition, same id, other cfg
    CB->>FP: fold sig_hash (codebase.rs:74)
    FP->>FP: sort the pair, keyed hash of both (hash.rs:139)
    FP-->>CB: merged sig_hash
    CB->>FP: fold body_hash (codebase.rs:75)
    FP-->>CB: merged body_hash
    CB->>CB: extend members, add new tags (codebase.rs:76)
    Note over CB: DEBT: scalar fields such as file and span<br/>keep the first twin only, codebase.rs:67
```

**What it shows.** Two definitions with one id, typically
`#[cfg(unix)]` and `#[cfg(windows)]` twins, become one symbol whose
fingerprints are an order-independent fold of both.

**Why it is this way.** A seal on a cfg-gated function has to notice an edit
to either twin; folding both values with a dedicated key context means an
edit to either changes the result, whichever twin was seen first
(`crates/sealmap-model/src/hash.rs:126`-`130`).

**Debt:** the merged symbol keeps only the first twin's file, span, signature
and doc (`crates/sealmap-model/src/codebase.rs:66`-`70`), so the second
definition has no location anywhere in the model.

## MOD-02.4 A fingerprint's life

```mermaid
sequenceDiagram
    autonumber
    participant NW as a symbol is created<br/>Symbol.new, symbol.rs 244
    participant FP as its Fingerprint value
    participant AD as an adapter
    participant MG as Fingerprint.merge<br/>hash.rs:138
    participant JS as serialisation and read-back
    NW->>FP: born unset, all zeros (symbol.rs 244)
    AD->>FP: assigns sig and body
    opt a cfg twin with the same id arrives
        FP->>MG: merged with the twin
        Note over FP: another twin merges the same way again
    end
    FP->>JS: serialised as blake3-16 text
    alt parsed back, either case accepted
        JS-->>FP: set again
    else wrong prefix or length
        JS-->>FP: refused
    end
```

**What it shows.** A fingerprint starts unset, is set by an adapter, may be
merged with twins, and travels as text in the JSON model; text with the wrong
prefix or length is refused on read.

**Why it is this way.** Serialisation goes through `Display` and `parse`
rather than serde's byte encoding (`crates/sealmap-model/src/hash.rs:157`-`168`),
so the JSON is readable and identical to what a lock would hold.

**Invariant:** a malformed fingerprint string fails deserialisation instead of
reading as zeros (`crates/sealmap-model/src/hash.rs:166`).

## MOD-02.5 What the pair decides

```mermaid
sequenceDiagram
    autonumber
    participant SC as the seal check over a sealed symbol<br/>id, sig, body (check.rs:138)
    alt the id is in the model
        alt is it the module of a file tagged parse_error (check.rs:146)
            SC->>SC: unparsable, fail closed
        else not the module of a parse_error file
            alt sig_hash equal (check.rs:147)
                alt body_hash equal (check.rs:150)
                    SC->>SC: holds (check.rs:153)
                else body_hash differs
                    SC->>SC: behaviour, body only
                end
            else sig_hash differs
                SC->>SC: contract
            end
        end
    else the id is not in the model
        alt is an ancestor module tagged parse_error (check.rs:155)
            SC->>SC: unparsable, fail closed
        else no ancestor module is
            SC->>SC: absent, same-kind ids with the sealed<br/>body are rename candidates (check.rs:158)
        end
    end
```

**What it shows.** The decision the seal check takes from the two
fingerprints: a body-only change is behaviour, a signature change is
contract, and a vanished id whose body reappears in a symbol of the same kind
is a suspected rename. Unparsable code is decided first, so it is never
reported as holding or absent (`crates/sealmap-corpus/src/seal/check.rs:146`,
`crates/sealmap-corpus/src/seal/check.rs:155`); the design's table is
`docs/DESIGN.md:99`-`107`.

**Why it is this way.** The name sits in `sig_hash` and never in `body_hash`,
which is what makes a rename detectable by body
(`crates/sealmap-extract/src/fingerprint.rs:26`-`27`); the README states the
same table for users (`README.md:189`-`196`).

The crate-surface table records that an optional `flow_hash` was considered
and not built (`docs/DESIGN.md:130`); the model has only the two
(`crates/sealmap-model/src/symbol.rs:209`-`212`), as commit `0a304ba` left it.
````
