You are repairing a diagrams-as-code corpus rewrite. The working directory is a plain copy of the sealmap repository (source code and docs) at revision 44fef36. The corpus is `docs/diagrams/`: 20 topic files under `docs/diagrams/<area>/NN-*.md`. Every other file in this directory is source code you may read but must not change.

Earlier, every non-`sequenceDiagram` block in the 20 topics was rewritten under this brief:

Brief (fixed):

Rewrite every non-`sequenceDiagram` block in every topic as one or more `sequenceDiagram` blocks that carry the same facts and the same `path:line` citations. Express decisions as `alt`/`opt`/`loop`/`break` blocks, and structure as participants and notes. Keep every narrative, register marker and invariant verbatim. Add nothing that is not in the original topic. Existing `sequenceDiagram` blocks stay as they are.

The corpus before that rewrite is in `.original/docs/diagrams/` (read-only): each topic there has the original diagram under the same heading. An independent fidelity gate then compared each rewritten diagram with the original diagram, the original topic and the cited code. It failed the 36 diagrams listed below. For each one it lists facts the rewrite states that the original topic does not state (unsupported), or states differently (changed). An arrow between participants claims a call or message, in that order; an `alt`/`opt`/`break` claims a condition and a `loop` claims repetition; exclusive outcomes must not be drawn as all happening. Where the original states no order or no call (for example class or type relationships), say the relationship in a note or in a participant label instead of drawing it as a message.

Task: repair each listed diagram once, so that it no longer states the listed unsupported or changed facts, while remaining a `sequenceDiagram` that carries the original diagram's facts and `path:line` citations and adds nothing that is not in the original topic.

Operating rules:
- Edit only the mermaid fences of the listed diagrams. Do not touch any other diagram, any prose, any heading, REGISTER.md, README.md, COVERAGE.md, DECISIONS-TIMELINE.md, the config, the tools, or anything outside `docs/diagrams/<area>/`. Do not edit `.original/`.
- If one diagram needs more than one sequenceDiagram, add the extra blocks as new sections with new headings that continue that topic's numbering (`## <ID>.<next n> <title>`), each with one mermaid block. Do not renumber existing headings.
- Carry every `path:line` citation from the original block. Do not invent or re-derive citations.
- Keep any scratch files under `.scratch/` in the working directory. The only exception: Chromium's socket path limit means the render gate needs a short temp directory, so run the gate with `TMPDIR=/tmp/es2-glm` (create it); write nothing else outside the working directory.
- Before finishing, the corpus must pass this gate from the working directory (structure and Mermaid grammar, renders no wider than 4500px):
    TMPDIR=/tmp/es2-glm node docs/diagrams/tools/diagram-index-gen.cjs docs/diagrams --check --render --jobs 8
  Fix any error it reports, then run it again until it exits 0.
- Finish with a short summary: diagrams repaired, blocks added, and anything you could not repair.

The failing diagrams and the gate's findings:

### COR-01.3: `docs/diagrams/corpus/01-generating-the-corpus-and-index.md`, under `## COR-01.3 The index as types`
- unsupported: "IX->>DE: documents (index.rs:97)". Arrow claims a message from Index to DocumentEntry; the original states only composition `Index *-- DocumentEntry` (a field), no call or message.
- unsupported: "DE->>FE: fragments (index.rs:82)". Arrow claims a message, and an order after IX->>DE; the original has only composition `DocumentEntry *-- FragmentEntry`, with no message or ordering.
- unsupported: "FE->>CR: calls (index.rs:53)". Arrow claims a message, ordered after DE->>FE; the original has only composition `FragmentEntry *-- CallRef`.
- unsupported: "FE->>FK: kind (index.rs:11)". Arrow claims a message, ordered last; the original has only the type association `FragmentEntry --> FragmentKind`.

### COR-02.3: `docs/diagrams/corpus/02-structure-sequence-and-overview-projections.md`, under `## COR-02.3 Assembling a file's structure diagram`
- unsupported: "ED->>SUB: field edges first". Original only says a field edge subsumes a uses edge for the same pair (structure.rs:172-176); it claims no ordering or priority of field edges, and the code iterates all edges in BTreeSet order, not field edges first
- changed: "SUB->>STUB: stubs for the rest" (original: "stubs for targets defined elsewhere, kind in file, or external"). Stubs are made only for edge targets not already drawn (structure.rs:178), not for 'the rest' of the edges or symbols

### COR-02.5: `docs/diagrams/corpus/02-structure-sequence-and-overview-projections.md`, under `## COR-02.5 The overview and its caps`
- unsupported: "OV->>CG: the first view ... OV->>MG: the second view ... OV->>DM: the third view ... OV->>TM: the fourth view". The original flowchart fans out from _overview.md to four unordered branches and its prose only lists them; it never states an emission order. The code agrees with this order (overview.rs:25-39), but the original topic does not claim it.

### COR-03.3: `docs/diagrams/corpus/03-generate-check-write-and-the-cli.md`, under `## COR-03.3 A corpus file's drift states`
- changed: "NW->>ST: Clean ... then alt ... then opt a new source / NW->>ST: Missing" (original: "[*] --> Clean: generate then write ... [*] --> Missing: new source"). Missing is an alternative initial state exclusive with Clean; the rewrite draws Clean as unconditional and Missing as an optional step after Clean and a drift, so two exclusive entries are shown as both happening, in sequence
- changed: "participant NW as generate then write, or a new source; NW->>ST: Clean" (original: "[*] --> Missing: new source"). the arrow attributes reaching Clean to 'a new source' as well; in the original a new source leads to Missing, not Clean

### COR-03.4: `docs/diagrams/corpus/03-generate-check-write-and-the-cli.md`, under `## COR-03.4 Reading a directory`
- changed: "else not readable ... RD->>RD: skipped" (original: "readable as UTF-8 and relative to the root? -->|no| skipped"). The else branch is labelled only 'not readable', but the original's 'no' branch (and contract.rs:125, which needs both read_to_string and relative_to to succeed) also skips files that are readable but not relative to the root.

### COR-04.1: `docs/diagrams/corpus/04-seals-the-lock-and-the-checks.md`, under `## COR-04.1 The lock and the report as types`
- unsupported: "LK->>TS: topics, a BTreeMap of id to TopicSeal". Arrow claims a message from Lock to TopicSeal; the original states only a composition field (Lock *-- TopicSeal : topics), no call or message.
- unsupported: "TS->>SS: symbols, a BTreeMap of SymbolId to SymbolSeal". Arrow claims a message from TopicSeal to SymbolSeal; original states only composition (TopicSeal *-- SymbolSeal : symbols).
- unsupported: "SR->>FD: findings, a Vec of Finding". Arrow claims a message from SealReport to Finding; original states only composition (SealReport *-- Finding : findings).
- unsupported: "FD->>CL: one class per finding". Arrow claims a message from Finding to Class; original states only an association (Finding --> Class), and the arrows together imply a call order the original never claims.

### COR-05.3: `docs/diagrams/corpus/05-the-seal-commands.md`, under `## COR-05.3 seal sign, from the command line`
- changed: "alt TOPIC an id, else a path under the cwd or the diagrams dir (main.rs:775-789) ... end wrapping date, library call and writes" (original: "A --> L --> T --> D --> S (T has a single outgoing edge, no failure branch)"). A one-branch alt makes the date, sign and writes conditional on the TOPIC guard; the original draws topic resolution as an unconditional step in the chain
- changed: "LIB-->>CMD: refused, message, exit 1, nothing written (main.rs:673-675)" (original: "S -->|error| R "refused: message, exit 1, nothing written""). Arrow claims the library itself returns the message and exit code to the command; main.rs:673-675 has the CLI print the message and return ExitCode 1 after the library's Err

### COR-05.4: `docs/diagrams/corpus/05-the-seal-commands.md`, under `## COR-05.4 Exit codes`
- unsupported: "the gates<br/>verify, seal-check, generate --check". the original topic calls only verify 'the gate' and says nothing grouping seal-check and generate --check as gates; the participant label adds a classification the original does not make

### COR-06.1: `docs/diagrams/corpus/06-review-packs.md`, under `## COR-06.1 From a request to a pack`
- changed: "LIB-->>OUT: exit 1, total, header and per-topic sizes (main.rs:736-743)" (original: "REF["exit 1: total, header and per-topic sizes main.rs:736-743"]; prose: "Selection and IO live in the CLI""). Arrow makes the library the sender of the exit-1 refusal report; the cited main.rs:731-744 shows the CLI prints the sizes and returns ExitCode 1 (shown twice, shard and pack branches)
- changed: "LIB-->>OUT: the packs / LIB-->>OUT: the pack" (original: "OUT["stdout, -o FILE, or pack-01.txt ... main.rs:746-757"]; prose: "Selection and IO live in the CLI""). Arrow claims the library writes the pack(s) to output; original prose and cited main.rs:746-757 place the writing in the CLI, the library only returns the packs

### COR-06.3: `docs/diagrams/corpus/06-review-packs.md`, under `## COR-06.3 The budget: refuse, or shard by topic`
- changed: "pack alt block (P-->>T OverBudget / the pack) followed unconditionally by `loop each topic` on shard" (original: "T --> P and T --> S as two separate paths (COR-06.1: MODE{"--shard?"} -->|no| PK, -->|yes| SH)"). pack and shard are exclusive modes; the sequence draws pack's budget check and then the shard loop as both happening in order
- changed: "else no file could hold it: S-->>T: TopicOverBudget names the topic (src/pack.rs:318), inside `loop each topic` with no break" (original: "A -->|no| TO["TopicOverBudget names the topic"] (terminal node)"). original and pack.rs:318 (`return Err(...)`) end shard on TopicOverBudget; the rewrite's alt inside the loop lets iteration continue to the next topic

### DEL-01.1: `docs/diagrams/delivery/01-ci-msrv-and-determinism.md`, under `## DEL-01.1 The four CI jobs`
- changed: "R->>M: next job" (original: "RUST --> MSRV (unlabelled subgraph edge)"). Sequence arrow claims the rust job hands off to / is followed by the msrv job; ci.yml:9-47 has no `needs:`, the jobs are independent and run in parallel, so the cited code contradicts the message/ordering claim.
- changed: "M->>X: next job" (original: "MSRV --> MER (unlabelled subgraph edge)"). Same: ci.yml:39-65 has no `needs:`; msrv does not trigger or precede the mermaid job.
- changed: "X->>D: next job" (original: "MER --> DIA (unlabelled subgraph edge)"). Same: ci.yml:49-84 has no `needs:`; mermaid does not trigger or precede the diagrams job.

### DEL-01.4: `docs/diagrams/delivery/01-ci-msrv-and-determinism.md`, under `## DEL-01.4 Crates, dependencies and publication`
- changed: "MO->>MM: depends" (original: "MO --> MM"). labelled message reads 'MO depends on MM', reversing the relationship; cited manifests show sealmap-mermaid depends on sealmap-model (optional), not the reverse; autonumber also imposes an order the original does not claim
- changed: "MO->>XT: depends" (original: "MO --> XT"). labelled message reads 'MO depends on XT', reversing the relationship; cited manifests show sealmap-extract depends on sealmap-model; autonumber also imposes an order the original does not claim
- changed: "XT->>RU: depends" (original: "XT --> RU"). labelled message reads 'XT depends on RU', reversing the relationship; cited manifests show sealmap-rust depends on sealmap-extract; autonumber also imposes an order the original does not claim
- changed: "MO->>CO: depends" (original: "MO --> CO"). labelled message reads 'MO depends on CO', reversing the relationship; cited manifests show sealmap-corpus depends on sealmap-model; autonumber also imposes an order the original does not claim
- changed: "MM->>CO: depends" (original: "MM --> CO"). labelled message reads 'MM depends on CO', reversing the relationship; cited manifests show sealmap-corpus depends on sealmap-mermaid; autonumber also imposes an order the original does not claim
- changed: "RU->>FA: depends" (original: "RU --> FA"). labelled message reads 'RU depends on FA', reversing the relationship; cited manifests show the facade depends on sealmap-rust; autonumber also imposes an order the original does not claim
- changed: "CO->>FA: depends" (original: "CO --> FA"). labelled message reads 'CO depends on FA', reversing the relationship; cited manifests show the facade depends on sealmap-corpus; autonumber also imposes an order the original does not claim
- changed: "MO->>DE: depends" (original: "MO --> DE"). labelled message reads 'MO depends on DE', reversing the relationship; cited manifests show sealmap-dense depends on sealmap-model; autonumber also imposes an order the original does not claim
- changed: "DE->>FA: depends" (original: "DE --> FA"). labelled message reads 'DE depends on FA', reversing the relationship; cited manifests show the facade depends on sealmap-dense; autonumber also imposes an order the original does not claim

### DEL-02.6: `docs/diagrams/delivery/02-design-versus-code.md`, under `## DEL-02.6 Dogfooding the gate on this repository`
- changed: "alt topics with no seal and no citation ... else MER-02 cites a fixture id in prose" (original: "V --> C; V --> M (prose: verify treats fifteen topics as legacy coverage and fails on one)"). the original shows both outcomes from the same verify run (15 coverage plus 1 failure, so the run exits 1); the alt makes them exclusive outcomes of the run, implying a run could return 'coverage only, never a failure' instead of exit 1

### DEN-01.4: `docs/diagrams/dense/01-the-dense-agent-projection.md`, under `## DEN-01.4 Short names and collisions`
- changed: "L->>L: level 0 ... L->>L: level 1 ... level 2 ... level 3 ... levels 4 and 5 (five unconditional numbered steps before the loop)" (original: "each symbol climbs only when it must; G{any name shared?} -->|yes| B moves to its next distinct level"). Climbing the ladder is drawn as an unconditional sequence every symbol runs through levels 0-5 before any collision check; the original (and short.rs:38-55) starts every name at level 0 and climbs only for colliding group members.
- changed: "loop any name shared (short.rs:41) ... every member of each group moves to its next distinct level" (original: "ladder --> G; G -->|yes| B; B --> G"). Order changed: the rewrite puts the collision loop after all five levels have already been climbed, so the 'next distinct level' step has no lower level to start from; in the original and in code the collision check drives each climb from level 0.

### DEN-01.6: `docs/diagrams/dense/01-the-dense-agent-projection.md`, under `## DEN-01.6 What reading sealmap's own projection revealed`
- unsupported: "F-->>R: call paths starting Self or self resolve ... (and the four other F-->>R replies)". Each fix is drawn as a return message from 'the fix' back to the read of dense.txt; the original only draws read -> defect -> fix (R --> S --> SF), with no edge from a fix back to the read.
- unsupported: "autonumbered order: Self::f defect, its fix, then guard defect, its fix, ... then pack defect, its fix". The original fans the five defects out of R in parallel with no order; the sequence claims a strict interleaved order (each defect raised only after the previous fix returned), whereas the prose says the first four came from one read and three were fixed together in d227dd8.

### EXT-01.2: `docs/diagrams/extract/01-the-adapter-pipeline.md`, under `## EXT-01.2 Where a file sits in the workspace`
- changed: "alt under tests, examples or benches (layout.rs:181) ... placed after the whole lib/src-bin/main.rs alt, so it follows 'own crate' (src/bin) and 'binary crate, pkg_main when a lib exists' too" (original: "SB -->|yes| OWN; MAIN -->|yes| BIN; only SRC --> TEB"). Original makes the src/bin and src/main.rs outcomes terminal and routes only the library-crate node into the tests/examples/benches check; the rewrite applies that check after every package branch, drawing exclusive outcomes (e.g. binary crate then also no target) as both happening; code returns early at layout.rs:170 and the src/bin branch never reaches the tests/examples/benches prefixes

### EXT-02.1: `docs/diagrams/extract/02-minting-ids-for-definitions.md`, under `## EXT-02.1 The id table`
- changed: "ME->>TR: a trait-impl method / ME->>FO: an impl method whose type is outside the code (successive messages, no alt)" (original: "METH --> TRAIT; METH --> FOREIGN (separate builders, one per kind of definition)"). The sequence draws both outcomes as happening one after the other in a single run. They are exclusive: impl_method_id returns method_id (the type-owned [Trait] form) only when the owner is global, and the module/impl anchor otherwise (ids.rs:102-110). No alt marks the choice, though the same rewrite uses one for this branch in EXT-02.3.

### EXT-02.2: `docs/diagrams/extract/02-minting-ids-for-definitions.md`, under `## EXT-02.2 Which Rust items become symbols`
- changed: "IT->>MO ... IT->>DA ... IT->>TR ... IT->>AL ... IT->>FN ... IT->>VA ... IT->>IM ... IT->>RE (unconditional, sequential messages)" (original: "IT --> MOD / IT --> DATA / ... / IT --> IMPL / IT --> REST (fan-out from one item to exclusive kinds)"). One item takes exactly one match arm (collect.rs:211-483); the rewrite draws eight mutually exclusive outcomes as all happening, in a fixed order, with only the macro case under alt

### EXT-02.4: `docs/diagrams/extract/02-minting-ids-for-definitions.md`, under `## EXT-02.4 Formatting never reaches an id`
- changed: "else comma before a closing paren: TK-->>ID: kept, a one-tuple is a different type (collect.rs:492)" (original: "TC -.-> KEEP (KEEP not connected to ID "the same Trait descriptor text either way"); invariant: "(T,) and (T) stay distinct""). Rewrite routes the kept-paren-comma outcome into the 'same Trait descriptor text either way' participant, claiming (T,) and (T) yield the same text; the original keeps it a side note and the code (collect.rs:495, only ,> ,] patterns replaced) keeps them distinct.

### EXT-03.7: `docs/diagrams/extract/03-the-flow-walker-and-raw-ir.md`, under `## EXT-03.7 What the walker remembers about local names`
- unsupported: "EN->>EN: a typed parameter or self seeds Param / EN->>EN: a let binding seeds Bound / opt a pattern binding ... (drawn in that order)". The original gives Param, Bound and Destructured as three parallel entries from [*]; it claims no order between parameter seeding, let handling and pattern handling, nor that pattern handling follows the let outcome.
- changed: "EN->>EN: a typed parameter or self seeds Param, then EN->>EN: a let binding seeds Bound (both unconditional)" (original: "[*] --> Param: typed parameter or self; [*] --> Bound: let binding"). Alternative entry points for a name are drawn as two unconditional steps that both happen; the let-binding path (and its alt) is shown as always taken.
- changed: "TY->>RS: block ends; CO->>RS: block ends; DE->>RS: block ends" (original: "Typed --> Restored; Computed --> Restored; Derived --> Restored"). Exclusive per-binding states (Typed vs Computed vs Derived) are drawn as three sequential messages that all occur, so mutually exclusive outcomes appear to happen together.

### EXT-04.3: `docs/diagrams/extract/04-lowering-confidence-and-labels.md`, under `## EXT-04.3 The confidence lattice and the external policy`
- changed: "else policy None (confidence.rs:52) / EK-->>FL: internal calls only" (original: "NONE["None: internal calls only"] with no edge to KEEP["kept in the flow"]"). The arrow from keeps to the flow claims the external call is delivered to the flow under None. The original does not link None to the flow, and confidence.rs:52 returns false, so the call is dropped.

### EXT-04.5: `docs/diagrams/extract/04-lowering-confidence-and-labels.md`, under `## EXT-04.5 From tokens to a label`
- unsupported: "autonumbered order 2 TS->>SQ, 3 T->>AS, 4 AS->>CL, 5 SQ->>CO". The original draws two independent chains (T->TS->SQ->CO->CP and T->AS->CL->CP) with no relative order; the rewrite serialises them, claiming argument sketching and call_label happen between squeeze and condition_label. A par block would have kept them independent.

### EXT-05.2: `docs/diagrams/extract/05-workspace-name-resolution.md`, under `## EXT-05.2 Walking a path`
- changed: "else use alias ... BS-->>WK: re-walk the target plus the rest (resolve.rs:675) / else otherwise BS-->>WK: a path id (resolve.rs:681) ... end / RE->>RE: item, method, pub use re-export, undefined member, or extend the path (resolve.rs:687)" (original: "F --> USE ; F --> PATH (no edge from USE or PATH to REST; only KW, LOC, GL, CR --> REST)"). The later-segment walk (and the Exact/Inferred/External step that follows it) sits after the alt block, so it runs unconditionally, including after the use-alias and path-id branches; the original ends those two branches without the later-segment walk, and the code returns early there (resolve.rs:675, 681).

### EXT-05.6: `docs/diagrams/extract/05-workspace-name-resolution.md`, under `## EXT-05.6 Closure parameters are untyped`
- unsupported: "else conn shadows an outer typed name / AR-->>RS: keeps the outer type". The arrow sends the shadowed parameter's outer type to the participant 'the resolver, resolve.rs:866'. The original's SH node hangs off AR only and makes no claim that it reaches resolve.rs:866, which is the Recv::Untyped name-guess arm; a typed receiver goes through the Recv::Typed arm instead.

### EXT-06.1: `docs/diagrams/extract/06-signature-and-body-fingerprints.md`, under `## EXT-06.1 The shared fingerprinter`
- unsupported: "FP->>TK: token(token) ... FP->>FP: section(label) ... FP->>FP: fingerprint(nested)". The original classDiagram lists methods with no call order; the rewrite asserts token, then section, then fingerprint. The cited code (fingerprint.rs:235-238) says section marks the START of a part, so it comes before that part's tokens, not after them.
- changed: "FP->>KD: the kind picks the key context (drawn after token(token))" (original: "Kind: Sig Body, distinct key contexts; +new(kind)"). The key context is fixed in new(kind) by new_derive_key(kind.context()) (fingerprint.rs:163-164), before any token is fed. Drawing it after token(token) reverses that order.

### EXT-06.3: `docs/diagrams/extract/06-signature-and-body-fingerprints.md`, under `## EXT-06.3 What goes into each value`
- unsupported: "CA->>CA: sig ... then CA->>CA: body ... (likewise DA, TR, VA, MO: sig message always before body message)". Sequenced self-messages claim the sig value is produced before the body value for every kind; the original draws sig and body as unordered siblings in each subgraph, and for traits the code interleaves them (trait_hashes, collect.rs:616-650 feeds sig and body per member in one loop).

### MER-01.1: `docs/diagrams/mermaid/01-typed-writers-and-escaping.md`, under `## MER-01.1 The writers and what they share`
- unsupported: "SD->>CW: render() ... CD->>CW: render() ... ER->>CW ... FC->>CW ... then SB->>ID / CD->>ID / ER->>ID / FC->>ID". Sequence arrows claim one ordered run (four independent builders render one after another, ids becoming Idents after rendering); the original classDiagram states only static relations and no order, and in the code Idents are supplied at build time, before render (e.g. sequence.rs:87 takes &Ident).
- changed: "SB->>ID: every id is an Ident (escape.rs:63)" (original: "class Ident { +new(raw) lossy  escape.rs:63 }; SeqBuilder ..> Ident"). Drawn as a message from SeqBuilder to Ident citing Ident::new, implying SeqBuilder calls Ident::new; the original has only a type dependency, and sequence.rs never calls Ident::new (builders take &Ident from the caller).

### MER-01.2: `docs/diagrams/mermaid/01-typed-writers-and-escaping.md`, under `## MER-01.2 Escaping a label`
- changed: "E->>TY: type text" (original: "E --> TY ("escape_type adds angle brackets, braces, and parens for fields")"). The sequence arrow claims escape_text passes type text to (calls) escape_type. The cited code has the reverse: escape_type calls escape_text (escape.rs:145) and then makes its own pass. The original flowchart edge only layers escape_type on top of escape_text.

### MOD-01.1: `docs/diagrams/model/01-the-sym-id-grammar.md`, under `## MOD-01.1 The id's shape as types`
- changed: "RE->>DE: Path holds segments (sym.rs:114), Unresolved a bare name (sym.rs:115)" (original: "Repr --> Descriptor; Path segments sym.rs:114; Unresolved name sym.rs:115"). Attaches Path and Unresolved to Descriptor; code has Path(Vec<String>) and Unresolved(String), only Global holds Vec<Descriptor> (sym.rs:113-115)
- changed: "accessors on the text itself (sym.rs:436, sym.rs:544, sym.rs:576, sym.rs:634)" (original: "+parse(text) Result sym.rs:436 ... +view() IdView sym.rs:634"). parse (sym.rs:436) is a constructor that runs the Parser to build a Repr (sym.rs:437), not an accessor working on the stored text

### MOD-01.4: `docs/diagrams/model/01-the-sym-id-grammar.md`, under `## MOD-01.4 Escaping names and package fields`
- unsupported: "NM->>WR: a package name or version to print". The package field is sent by the 'a descriptor or path name' participant and comes after the name step. The original has 'a package name or version' as a separate, independent input (F --> FC) and makes no claim that names and fields come from one source or in any order.
- changed: "else a char falls outside the simple set" (original: "S{"non-empty and every char in the simple set?"} -->|no| Q"). The original 'no' branch also covers the empty name. write_name (sym.rs:373) backtick-quotes an empty name too, so the rewrite's else label drops that case and narrows the condition.

### MOD-01.6: `docs/diagrams/model/01-the-sym-id-grammar.md`, under `## MOD-01.6 Where the grammar is documented and claimed`
- changed: "GR->>IB: the one id builder per adapter targets the same grammar" (original: "id builder, one per adapter ... the shared id builder"). Message text reads as a separate builder for each adapter; the original prose says 'shared id builder' and ids.rs:3-4 says 'all language adapters share one sym: grammar' through one builder

### MOD-02.2: `docs/diagrams/model/02-content-hashes-and-fingerprints.md`, under `## MOD-02.2 From bytes to printed value`
- changed: "KD-->>TK: first 16 bytes kept (from_blake3, hash.rs:99)" (original: "KD --> TR["first 16 bytes kept"] --> FP"). the truncated digest is drawn as a reply back to the token stream; the original (and from_blake3 at hash.rs:97-101) flows it forward from the keyed hasher to the Fingerprint that Display prints
- changed: "TK->>FP: printed as blake3-16, 32 hex (Display, hash.rs:149)" (original: "TR["first 16 bytes kept"] --> FP["blake3-16 colon 32 hex, Display"]"). the printing step is drawn as a message from the symbol token stream; in the original (and Display for Fingerprint at hash.rs:148-155) it is the truncated 16-byte fingerprint that is printed, not the tokens

### MOD-02.5: `docs/diagrams/model/02-content-hashes-and-fingerprints.md`, under `## MOD-02.5 What the pair decides`
- unsupported: "MD-->>SC: unparsable, fail closed (in-model branch)". Arrow claims the model replies with the verdict; original only has the seal check deciding 'unparsable, fail closed', and classify() in check.rs computes it, codebase.symbol() only returns the symbol
- unsupported: "MD-->>SC: holds (check.rs:153)". Arrow claims the model returns 'holds'; original shows it as the check's outcome node, and check.rs:153 builds it in classify()
- unsupported: "MD-->>SC: behaviour, body only". Arrow claims a model-to-check message carrying the verdict; no such message in the original or code
- unsupported: "MD-->>SC: contract". Arrow claims a model-to-check message carrying the verdict; no such message in the original or code
- unsupported: "MD-->>SC: unparsable, fail closed (absent branch, check.rs:155)". Arrow claims the model returns the verdict; original's ancestor check is the seal check's decision (unparsable_ancestor in classify)
- unsupported: "MD-->>SC: absent, same-kind ids with the sealed body are rename candidates". Arrow claims the model returns absent with rename candidates; original shows it as the check's outcome, computed by classify/rename_candidates

### MOD-03.1: `docs/diagrams/model/03-the-codebase-model-and-schema-v2.md`, under `## MOD-03.1 The model as types`
- unsupported: "CB->>SF: files ... / CB->>SY: symbols ... / SY->>FL: may carry a flow / CB->>RE: relations ... / RE->>CO: carries a confidence". Five sequence arrows claim messages between participants in a fixed order (SourceFile, Symbol, Flow, Relation, Confidence); the original states only static composition/aggregation/association (Codebase *-- SourceFile, Symbol o-- Flow, Relation --> Confidence) with no calls or ordering

### MOD-03.3: `docs/diagrams/model/03-the-codebase-model-and-schema-v2.md`, under `## MOD-03.3 Normalising a path`
- changed: "participant PE as PathError<br/>path.rs:49" (original: "ABS{"leading slash or drive letter?<br/>path.rs:49"}"). path.rs:49 is attached to the absolute-path check in the original and in the code; the rewrite re-attaches it to the PathError component, which the code defines at path.rs:28, so the citation now locates PathError wrongly

### MOD-03.5: `docs/diagrams/model/03-the-codebase-model-and-schema-v2.md`, under `## MOD-03.5 A flow is the skeleton of a sequence diagram`
- changed: "FL->>RT: an interior node" (original: "Calls are leaves; branches, loops, optional blocks and parallel arms are interior nodes"). Return is not listed as interior in the original; code has Step::Return(Exit) with no body and depth 0 (flow.rs:70, flow.rs:138), i.e. a leaf
