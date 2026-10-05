---
codemaid: 1
kind: overview
codebase: codemaid
---
# codemaid overview
30 files · 393 symbols · 1093 relations · 144 flows · 707 calls

## crates
```mermaid
flowchart LR
  codemaid(["codemaid"])
  codemaid_corpus(["codemaid_corpus"])
  codemaid_main(["codemaid_main"])
  codemaid_mermaid(["codemaid_mermaid"])
  codemaid_model(["codemaid_model"])
  codemaid_rust(["codemaid_rust"])
  blake3{{"blake3"}}
  clap{{"clap"}}
  proc_macro2{{"proc_macro2"}}
  quote{{"quote"}}
  serde{{"serde"}}
  serde_json{{"serde_json"}}
  syn{{"syn"}}
  codemaid -->|"9"| codemaid_corpus
  codemaid -->|"1"| codemaid_mermaid
  codemaid -->|"8"| codemaid_model
  codemaid -->|"4"| codemaid_rust
  codemaid_corpus -->|"69"| codemaid_mermaid
  codemaid_corpus -->|"136"| codemaid_model
  codemaid_corpus -->|"4"| serde
  codemaid_corpus -->|"2"| serde_json
  codemaid_main -->|"3"| clap
  codemaid_main -->|"4"| codemaid
  codemaid_main -->|"7"| codemaid_corpus
  codemaid_main -->|"3"| codemaid_model
  codemaid_main -->|"4"| codemaid_rust
  codemaid_main -->|"1"| serde_json
  codemaid_model -->|"1"| blake3
  codemaid_model -->|"12"| serde
  codemaid_rust -->|"117"| codemaid_model
  codemaid_rust -->|"6"| proc_macro2
  codemaid_rust -->|"3"| quote
  codemaid_rust -->|"61"| syn
```

## modules: codemaid_corpus
```mermaid
flowchart LR
  codemaid_corpus["codemaid_corpus"]
  codemaid_corpus__contract["contract"]
  codemaid_corpus__document["document"]
  codemaid_corpus__index["index"]
  codemaid_corpus__naming["naming"]
  codemaid_corpus__overview["overview"]
  codemaid_corpus__sequence["sequence"]
  codemaid_corpus__structure["structure"]
  codemaid_corpus -->|"8"| codemaid_corpus__contract
  codemaid_corpus -->|"1"| codemaid_corpus__document
  codemaid_corpus -->|"8"| codemaid_corpus__index
  codemaid_corpus -->|"1"| codemaid_corpus__overview
  codemaid_corpus__contract -->|"7"| codemaid_corpus
  codemaid_corpus__contract -->|"4"| codemaid_corpus__document
  codemaid_corpus__document -->|"4"| codemaid_corpus
  codemaid_corpus__document -->|"5"| codemaid_corpus__index
  codemaid_corpus__document -->|"2"| codemaid_corpus__sequence
  codemaid_corpus__document -->|"2"| codemaid_corpus__structure
  codemaid_corpus__naming -->|"3"| codemaid_corpus
  codemaid_corpus__overview -->|"6"| codemaid_corpus
  codemaid_corpus__overview -->|"4"| codemaid_corpus__naming
  codemaid_corpus__sequence -->|"3"| codemaid_corpus
  codemaid_corpus__sequence -->|"9"| codemaid_corpus__naming
  codemaid_corpus__structure -->|"2"| codemaid_corpus
  codemaid_corpus__structure -->|"2"| codemaid_corpus__naming
```

## modules: codemaid_mermaid
```mermaid
flowchart LR
  codemaid_mermaid["codemaid_mermaid"]
  codemaid_mermaid__class["class"]
  codemaid_mermaid__er["er"]
  codemaid_mermaid__escape["escape"]
  codemaid_mermaid__flowchart["flowchart"]
  codemaid_mermaid__sequence["sequence"]
  codemaid_mermaid__writer["writer"]
  codemaid_mermaid -->|"5"| codemaid_mermaid__class
  codemaid_mermaid -->|"2"| codemaid_mermaid__er
  codemaid_mermaid -->|"3"| codemaid_mermaid__escape
  codemaid_mermaid -->|"3"| codemaid_mermaid__flowchart
  codemaid_mermaid -->|"4"| codemaid_mermaid__sequence
  codemaid_mermaid -->|"1"| codemaid_mermaid__writer
  codemaid_mermaid__class -->|"14"| codemaid_mermaid__escape
  codemaid_mermaid__class -->|"8"| codemaid_mermaid__writer
  codemaid_mermaid__er -->|"11"| codemaid_mermaid__escape
  codemaid_mermaid__er -->|"5"| codemaid_mermaid__writer
  codemaid_mermaid__flowchart -->|"6"| codemaid_mermaid__class
  codemaid_mermaid__flowchart -->|"8"| codemaid_mermaid__escape
  codemaid_mermaid__flowchart -->|"5"| codemaid_mermaid__writer
  codemaid_mermaid__sequence -->|"16"| codemaid_mermaid__escape
  codemaid_mermaid__sequence -->|"8"| codemaid_mermaid__writer
```

## modules: codemaid_model
```mermaid
flowchart LR
  codemaid_model["codemaid_model"]
  codemaid_model__codebase["codebase"]
  codemaid_model__flow["flow"]
  codemaid_model__hash["hash"]
  codemaid_model__path["path"]
  codemaid_model__source["source"]
  codemaid_model__symbol["symbol"]
  codemaid_model -->|"2"| codemaid_model__codebase
  codemaid_model -->|"6"| codemaid_model__flow
  codemaid_model -->|"1"| codemaid_model__hash
  codemaid_model -->|"2"| codemaid_model__path
  codemaid_model -->|"3"| codemaid_model__source
  codemaid_model -->|"10"| codemaid_model__symbol
  codemaid_model__codebase -->|"1"| codemaid_model__flow
  codemaid_model__codebase -->|"4"| codemaid_model__path
  codemaid_model__codebase -->|"4"| codemaid_model__source
  codemaid_model__codebase -->|"26"| codemaid_model__symbol
  codemaid_model__flow -->|"6"| codemaid_model__symbol
  codemaid_model__source -->|"6"| codemaid_model__hash
  codemaid_model__source -->|"12"| codemaid_model__path
  codemaid_model__source -->|"3"| codemaid_model__symbol
  codemaid_model__symbol -->|"2"| codemaid_model__flow
  codemaid_model__symbol -->|"3"| codemaid_model__path
```

## modules: codemaid_rust
```mermaid
flowchart LR
  codemaid_rust["codemaid_rust"]
  codemaid_rust__collect["collect"]
  codemaid_rust__layout["layout"]
  codemaid_rust__raw["raw"]
  codemaid_rust__resolve["resolve"]
  codemaid_rust__tidy["tidy"]
  codemaid_rust -->|"1"| codemaid_rust__collect
  codemaid_rust -->|"1"| codemaid_rust__layout
  codemaid_rust -->|"1"| codemaid_rust__resolve
  codemaid_rust__collect -->|"3"| codemaid_rust
  codemaid_rust__collect -->|"2"| codemaid_rust__layout
  codemaid_rust__collect -->|"30"| codemaid_rust__raw
  codemaid_rust__collect -->|"25"| codemaid_rust__tidy
  codemaid_rust__raw -->|"2"| codemaid_rust__layout
  codemaid_rust__resolve -->|"10"| codemaid_rust
  codemaid_rust__resolve -->|"16"| codemaid_rust__raw
```

## data: codemaid_corpus
```mermaid
erDiagram
  codemaid_corpus__Corpus["Corpus"] {
    BTreeMap[SourcePath_String] files "BTreeMap<SourcePath, String>"
    Index index
  }
  codemaid_corpus__CorpusOptions["CorpusOptions"] {
    usize min_calls
    usize max_messages
    usize max_edges
    usize max_entities
    bool include_private
    ExternalLanes external_lanes
    bool emit_model
    bool pretty_json
  }
  codemaid_corpus__ExternalLanes["ExternalLanes"]
  codemaid_corpus__contract__Drift["Drift"]
  codemaid_corpus__contract__DriftEntry["DriftEntry"] {
    SourcePath path
    Drift drift
  }
  codemaid_corpus__contract__Report["Report"] {
    Vec[DriftEntry] entries
    usize checked
  }
  codemaid_corpus__index__CallRef["CallRef"] {
    SymbolId target
    Confidence confidence
    u32 line
    Option[String] expands
  }
  codemaid_corpus__index__DocumentEntry["DocumentEntry"] {
    SourcePath source
    SourcePath document
    SymbolId module
    ContentHash source_hash
    ContentHash document_hash
    Vec[FragmentEntry] fragments
  }
  codemaid_corpus__index__FragmentEntry["FragmentEntry"] {
    String id
    FragmentKind kind
    SourcePath document
    SymbolId symbol
    Span span
    Vec[SymbolId] participants
    Vec[CallRef] calls
    usize truncated
    ContentHash hash
  }
  codemaid_corpus__index__FragmentKind["FragmentKind"]
  codemaid_corpus__index__Index["Index"] {
    u32 schema
    String generator
    String codebase
    CodebaseStats stats
    Vec[DocumentEntry] documents
  }
  codemaid_corpus__sequence__Ctx["Ctx"] {
    __aCodebase cb "&'a Codebase"
    __aCorpusOptions opts "&'a CorpusOptions"
    SymbolId caller
    Vec[(SymbolId_String_bool)] lanes "Vec<(SymbolId, String, bool)>"
    usize budget
    usize dropped
  }
  codemaid_corpus__Corpus ||--|| codemaid_corpus__index__Index : "index"
  codemaid_corpus__CorpusOptions ||--|| codemaid_corpus__ExternalLanes : "external_lanes"
  codemaid_corpus__contract__DriftEntry ||--|| codemaid_corpus__contract__Drift : "drift"
  codemaid_corpus__contract__Report ||--o{ codemaid_corpus__contract__DriftEntry : "entries"
  codemaid_corpus__index__DocumentEntry ||--o{ codemaid_corpus__index__FragmentEntry : "fragments"
  codemaid_corpus__index__FragmentEntry ||--|| codemaid_corpus__index__FragmentKind : "kind"
  codemaid_corpus__index__FragmentEntry ||--o{ codemaid_corpus__index__CallRef : "calls"
  codemaid_corpus__index__Index ||--o{ codemaid_corpus__index__DocumentEntry : "documents"
  codemaid_corpus__sequence__Ctx ||..|| codemaid_corpus__CorpusOptions : "opts"
```

## data: codemaid_main
```mermaid
erDiagram
  codemaid_main__Cli["Cli"] {
    Cmd cmd
  }
  codemaid_main__Cmd["Cmd"]
  codemaid_main__Common["Common"] {
    PathBuf path
    Vec[String] repos
    PathBuf out
    Option[String] name
    bool tests
    String external
    bool owner_lanes
    usize min_calls
    bool public_only
    bool no_model
    bool pretty
  }
  codemaid_main__Cli ||--|| codemaid_main__Cmd : "cmd"
  codemaid_main__Cmd ||--|| codemaid_main__Common : "Generate"
  codemaid_main__Cmd ||--|| codemaid_main__Common : "Verify"
  codemaid_main__Cmd ||--|| codemaid_main__Common : "Model"
```

## data: codemaid_mermaid
```mermaid
erDiagram
  codemaid_mermaid__class__Class["Class"] {
    Ident id
    String label
    Option[String] annotation
    Vec[String] members
  }
  codemaid_mermaid__class__ClassDiagram["ClassDiagram"] {
    Option[Direction] direction_
    Vec[Class] classes
    Vec[ClassRelation] relations
  }
  codemaid_mermaid__class__ClassRelation["ClassRelation"] {
    Ident from
    Ident to
    ClassRelationKind kind
    Option[String] label
  }
  codemaid_mermaid__class__ClassRelationKind["ClassRelationKind"]
  codemaid_mermaid__class__Direction["Direction"]
  codemaid_mermaid__er__Attr["Attr"] {
    String ty
    String name
    Option[__staticstr] key "Option<&'static str>"
    Option[String] comment
  }
  codemaid_mermaid__er__Cardinality["Cardinality"]
  codemaid_mermaid__er__Entity["Entity"] {
    Ident id
    String label
    Vec[Attr] attrs
  }
  codemaid_mermaid__er__ErDiagram["ErDiagram"] {
    Vec[Entity] entities
    Vec[Rel] rels
  }
  codemaid_mermaid__er__Rel["Rel"] {
    Ident from
    Cardinality from_card
    Ident to
    Cardinality to_card
    bool identifying
    String label
  }
  codemaid_mermaid__escape__Ident["Ident"] {
    String _0
  }
  codemaid_mermaid__flowchart__EdgeStyle["EdgeStyle"]
  codemaid_mermaid__flowchart__Flowchart["Flowchart"] {
    Direction direction_
    Vec[Stmt] body
  }
  codemaid_mermaid__flowchart__NodeShape["NodeShape"]
  codemaid_mermaid__flowchart__Stmt["Stmt"]
  codemaid_mermaid__sequence__Arrow["Arrow"]
  codemaid_mermaid__sequence__BlockKind["BlockKind"]
  codemaid_mermaid__sequence__Item["Item"]
  codemaid_mermaid__sequence__SeqBuilder["SeqBuilder"] {
    Vec[Item] items
  }
  codemaid_mermaid__sequence__SequenceDiagram["SequenceDiagram"] {
    Option[String] title
    bool autonumber_
    Vec[(Ident_String_bool)] participants "Vec<(Ident, String, bool)>"
    SeqBuilder body
  }
  codemaid_mermaid__class__Class ||--|| codemaid_mermaid__escape__Ident : "id"
  codemaid_mermaid__class__ClassDiagram ||--o| codemaid_mermaid__class__Direction : "direction"
  codemaid_mermaid__class__ClassDiagram ||--o{ codemaid_mermaid__class__Class : "classes"
  codemaid_mermaid__class__ClassDiagram ||--o{ codemaid_mermaid__class__ClassRelation : "relations"
  codemaid_mermaid__class__ClassRelation ||--|| codemaid_mermaid__escape__Ident : "from"
  codemaid_mermaid__class__ClassRelation ||--|| codemaid_mermaid__escape__Ident : "to"
  codemaid_mermaid__class__ClassRelation ||--|| codemaid_mermaid__class__ClassRelationKind : "kind"
  codemaid_mermaid__er__Entity ||--|| codemaid_mermaid__escape__Ident : "id"
  codemaid_mermaid__er__Entity ||--o{ codemaid_mermaid__er__Attr : "attrs"
  codemaid_mermaid__er__ErDiagram ||--o{ codemaid_mermaid__er__Entity : "entities"
  codemaid_mermaid__er__ErDiagram ||--o{ codemaid_mermaid__er__Rel : "rels"
  codemaid_mermaid__er__Rel ||--|| codemaid_mermaid__escape__Ident : "from"
  codemaid_mermaid__er__Rel ||--|| codemaid_mermaid__er__Cardinality : "from_card"
  codemaid_mermaid__er__Rel ||--|| codemaid_mermaid__escape__Ident : "to"
  codemaid_mermaid__er__Rel ||--|| codemaid_mermaid__er__Cardinality : "to_card"
  codemaid_mermaid__flowchart__Flowchart ||--|| codemaid_mermaid__class__Direction : "direction"
  codemaid_mermaid__flowchart__Flowchart ||--o{ codemaid_mermaid__flowchart__Stmt : "body"
  codemaid_mermaid__flowchart__Stmt ||--|| codemaid_mermaid__escape__Ident : "Node"
  codemaid_mermaid__flowchart__Stmt ||--|| codemaid_mermaid__flowchart__NodeShape : "Node"
  codemaid_mermaid__flowchart__Stmt ||--|| codemaid_mermaid__escape__Ident : "Edge"
  codemaid_mermaid__flowchart__Stmt ||--|| codemaid_mermaid__flowchart__EdgeStyle : "Edge"
  codemaid_mermaid__flowchart__Stmt ||--|| codemaid_mermaid__escape__Ident : "Subgraph"
  codemaid_mermaid__flowchart__Stmt ||--|| codemaid_mermaid__class__Direction : "Subgraph"
  codemaid_mermaid__sequence__Item ||--|| codemaid_mermaid__escape__Ident : "Message"
  codemaid_mermaid__sequence__Item ||--|| codemaid_mermaid__sequence__Arrow : "Message"
  codemaid_mermaid__sequence__Item ||--|| codemaid_mermaid__escape__Ident : "Note"
  codemaid_mermaid__sequence__Item ||--|| codemaid_mermaid__sequence__BlockKind : "Block"
  codemaid_mermaid__sequence__SeqBuilder ||--o{ codemaid_mermaid__sequence__Item : "items"
  codemaid_mermaid__sequence__SequenceDiagram ||--o{ codemaid_mermaid__escape__Ident : "participants"
  codemaid_mermaid__sequence__SequenceDiagram ||--|| codemaid_mermaid__sequence__SeqBuilder : "body"
```

## data: codemaid_model
```mermaid
erDiagram
  codemaid_model__codebase__Codebase["Codebase"] {
    u32 schema
    String name
    BTreeMap[SourcePath_SourceFile] files "BTreeMap<SourcePath, SourceFile>"
    BTreeMap[SymbolId_Symbol] symbols "BTreeMap<SymbolId, Symbol>"
    BTreeSet[Relation] relations
  }
  codemaid_model__flow__Arm["Arm"] {
    String label
    Vec[Step] steps
  }
  codemaid_model__flow__Call["Call"] {
    SymbolId target
    String label
    CallKind kind
    Confidence confidence
    bool awaited
    bool fallible
    u32 line
  }
  codemaid_model__flow__CallKind["CallKind"]
  codemaid_model__flow__Exit["Exit"] {
    String label
    u32 line
  }
  codemaid_model__flow__Flow["Flow"] {
    Vec[Step] steps
  }
  codemaid_model__flow__Step["Step"]
  codemaid_model__hash__ContentHash["ContentHash"] {
    String _0
  }
  codemaid_model__path__SourcePath["SourcePath"] {
    String _0
  }
  codemaid_model__source__SourceFile["SourceFile"] {
    SourcePath path
    String language
    SymbolId module
    ContentHash hash
    u32 lines
  }
  codemaid_model__source__SourceSet["SourceSet"] {
    BTreeMap[SourcePath_String] files "BTreeMap<SourcePath, String>"
  }
  codemaid_model__symbol__Confidence["Confidence"]
  codemaid_model__symbol__Member["Member"] {
    String name
    MemberKind kind
    Option[String] ty
    Visibility visibility
    Span span
    Vec[SymbolId] refs
  }
  codemaid_model__symbol__MemberKind["MemberKind"]
  codemaid_model__symbol__Relation["Relation"] {
    SymbolId from
    SymbolId to
    RelationKind kind
    Confidence confidence
  }
  codemaid_model__symbol__RelationKind["RelationKind"]
  codemaid_model__symbol__Span["Span"] {
    u32 start_line
    u32 start_col
    u32 end_line
    u32 end_col
  }
  codemaid_model__symbol__Symbol["Symbol"] {
    SymbolId id
    String name
    SymbolKind kind
    Visibility visibility
    SourcePath file
    Span span
    Option[SymbolId] parent
    Option[String] signature
    Option[String] doc
    Vec[String] generics
    Vec[String] tags
    Vec[Member] members
    Option[Flow] flow
  }
  codemaid_model__symbol__SymbolId["SymbolId"] {
    String _0
  }
  codemaid_model__symbol__SymbolKind["SymbolKind"]
  codemaid_model__symbol__Visibility["Visibility"]
  codemaid_model__codebase__Codebase ||--o{ codemaid_model__path__SourcePath : "files"
  codemaid_model__codebase__Codebase ||--o{ codemaid_model__source__SourceFile : "files"
  codemaid_model__codebase__Codebase ||--o{ codemaid_model__symbol__SymbolId : "symbols"
  codemaid_model__codebase__Codebase ||--o{ codemaid_model__symbol__Symbol : "symbols"
  codemaid_model__codebase__Codebase ||--o{ codemaid_model__symbol__Relation : "relations"
  codemaid_model__flow__Arm ||--o{ codemaid_model__flow__Step : "steps"
  codemaid_model__flow__Call ||--|| codemaid_model__symbol__SymbolId : "target"
  codemaid_model__flow__Call ||--|| codemaid_model__flow__CallKind : "kind"
  codemaid_model__flow__Call ||--|| codemaid_model__symbol__Confidence : "confidence"
  codemaid_model__flow__Flow ||--o{ codemaid_model__flow__Step : "steps"
  codemaid_model__flow__Step ||--|| codemaid_model__flow__Call : "Call"
  codemaid_model__flow__Step ||--|| codemaid_model__flow__Arm : "Branch"
  codemaid_model__flow__Step ||--|| codemaid_model__flow__Arm : "Parallel"
  codemaid_model__flow__Step ||--|| codemaid_model__flow__Exit : "Return"
  codemaid_model__source__SourceFile ||--|| codemaid_model__path__SourcePath : "path"
  codemaid_model__source__SourceFile ||--|| codemaid_model__symbol__SymbolId : "module"
  codemaid_model__source__SourceFile ||--|| codemaid_model__hash__ContentHash : "hash"
  codemaid_model__source__SourceSet ||--o{ codemaid_model__path__SourcePath : "files"
  codemaid_model__symbol__Member ||--|| codemaid_model__symbol__MemberKind : "kind"
  codemaid_model__symbol__Member ||--|| codemaid_model__symbol__Visibility : "visibility"
  codemaid_model__symbol__Member ||--|| codemaid_model__symbol__Span : "span"
  codemaid_model__symbol__Member ||--o{ codemaid_model__symbol__SymbolId : "refs"
  codemaid_model__symbol__Relation ||--|| codemaid_model__symbol__SymbolId : "from"
  codemaid_model__symbol__Relation ||--|| codemaid_model__symbol__SymbolId : "to"
  codemaid_model__symbol__Relation ||--|| codemaid_model__symbol__RelationKind : "kind"
  codemaid_model__symbol__Relation ||--|| codemaid_model__symbol__Confidence : "confidence"
  codemaid_model__symbol__Symbol ||--|| codemaid_model__symbol__SymbolId : "id"
  codemaid_model__symbol__Symbol ||--|| codemaid_model__symbol__SymbolKind : "kind"
  codemaid_model__symbol__Symbol ||--|| codemaid_model__symbol__Visibility : "visibility"
  codemaid_model__symbol__Symbol ||--|| codemaid_model__path__SourcePath : "file"
  codemaid_model__symbol__Symbol ||--|| codemaid_model__symbol__Span : "span"
  codemaid_model__symbol__Symbol ||--o| codemaid_model__symbol__SymbolId : "parent"
  codemaid_model__symbol__Symbol ||--o{ codemaid_model__symbol__Member : "members"
  codemaid_model__symbol__Symbol ||--o| codemaid_model__flow__Flow : "flow"
```

## data: codemaid_rust
```mermaid
erDiagram
  codemaid_rust__Diagnostic["Diagnostic"] {
    SourcePath file
    String message
  }
  codemaid_rust__ExternalCalls["ExternalCalls"]
  codemaid_rust__Extraction["Extraction"] {
    Codebase codebase
    Vec[Diagnostic] diagnostics
  }
  codemaid_rust__RustOptions["RustOptions"] {
    String name
    bool include_tests
    ExternalCalls external_calls
  }
  codemaid_rust__collect__Collector["Collector"] {
    __aRustOptions opts "&'a RustOptions"
    __amutRawFile raw "&'a mut RawFile"
  }
  codemaid_rust__collect__FlowWalker["FlowWalker"] {
    BTreeMap[String_Recv] env "BTreeMap<String, Recv>"
  }
  codemaid_rust__layout__FileRole["FileRole"] {
    String crate_name
    Vec[String] module
    TargetKind target
  }
  codemaid_rust__layout__TargetKind["TargetKind"]
  codemaid_rust__raw__Callee["Callee"]
  codemaid_rust__raw__RawCall["RawCall"] {
    Callee callee
    String label
    CallKind kind
    bool awaited
    bool fallible
    u32 line
  }
  codemaid_rust__raw__RawFile["RawFile"] {
    SourcePath path
    FileRole role
    u32 text_lines
    ContentHash hash
    Vec[RawModule] modules
    Vec[RawItem] items
    Vec[RawImpl] impls
    Option[String] error
  }
  codemaid_rust__raw__RawFn["RawFn"] {
    String name
    Visibility vis
    Span span
    String signature
    Option[String] doc
    Vec[String] generics
    Vec[String] tags
    Vec[Segs] sig_refs
    Vec[RawStep] flow
  }
  codemaid_rust__raw__RawImpl["RawImpl"] {
    Segs module
    Option[Segs] self_ty
    Option[(Segs_String)] trait_ "Option<(Segs, String)>"
    Vec[RawFn] methods
  }
  codemaid_rust__raw__RawItem["RawItem"] {
    Segs module
    String name
    SymbolKind kind
    Visibility vis
    Span span
    Option[String] signature
    Option[String] doc
    Vec[String] generics
    Vec[String] tags
    Vec[RawMember] members
    Vec[Segs] sig_refs
    Vec[Segs] supertraits
    Vec[RawFn] methods
    Vec[RawStep] flow
  }
  codemaid_rust__raw__RawMember["RawMember"] {
    String name
    MemberKind kind
    Option[String] ty
    Visibility vis
    Span span
    Vec[Segs] refs
  }
  codemaid_rust__raw__RawModule["RawModule"] {
    Segs path
    Span span
    Option[String] doc
    Visibility vis
    Vec[RawUse] uses
    Vec[String] tags
  }
  codemaid_rust__raw__RawStep["RawStep"]
  codemaid_rust__raw__RawUse["RawUse"] {
    String alias
    Segs target
  }
  codemaid_rust__raw__Recv["Recv"]
  codemaid_rust__resolve__Resolver["Resolver"] {
    BTreeSet[String] crates
    BTreeMap[String_BTreeMap[String_SymbolId]] items "BTreeMap<String, BTreeMap<String, SymbolId>>"
    BTreeMap[String_Vec[RawUse]] uses "BTreeMap<String, Vec<RawUse>>"
    BTreeSet[SymbolId] internal
    BTreeMap[SymbolId_(String_BTreeMap[String_Vec[Segs]])] fields "BTreeMap<SymbolId, (String, BTreeMap<String, Vec<Segs>>)>"
    BTreeMap[SymbolId_BTreeMap[String_SymbolId]] methods "BTreeMap<SymbolId, BTreeMap<String, SymbolId>>"
    BTreeMap[String_BTreeSet[SymbolId]] by_name "BTreeMap<String, BTreeSet<SymbolId>>"
    BTreeMap[SymbolId_BTreeSet[SymbolId]] impls "BTreeMap<SymbolId, BTreeSet<SymbolId>>"
    BTreeMap[SymbolId_BTreeSet[String]] trait_methods "BTreeMap<SymbolId, BTreeSet<String>>"
  }
  codemaid_rust__Extraction ||--o{ codemaid_rust__Diagnostic : "diagnostics"
  codemaid_rust__RustOptions ||--|| codemaid_rust__ExternalCalls : "external_calls"
  codemaid_rust__collect__Collector ||..|| codemaid_rust__RustOptions : "opts"
  codemaid_rust__collect__Collector ||..|| codemaid_rust__raw__RawFile : "raw"
  codemaid_rust__collect__FlowWalker ||--o{ codemaid_rust__raw__Recv : "env"
  codemaid_rust__layout__FileRole ||--|| codemaid_rust__layout__TargetKind : "target"
  codemaid_rust__raw__Callee ||--|| codemaid_rust__raw__Recv : "Method"
  codemaid_rust__raw__RawCall ||--|| codemaid_rust__raw__Callee : "callee"
  codemaid_rust__raw__RawFile ||--|| codemaid_rust__layout__FileRole : "role"
  codemaid_rust__raw__RawFile ||--o{ codemaid_rust__raw__RawModule : "modules"
  codemaid_rust__raw__RawFile ||--o{ codemaid_rust__raw__RawItem : "items"
  codemaid_rust__raw__RawFile ||--o{ codemaid_rust__raw__RawImpl : "impls"
  codemaid_rust__raw__RawFn ||--o{ codemaid_rust__raw__RawStep : "flow"
  codemaid_rust__raw__RawImpl ||--o{ codemaid_rust__raw__RawFn : "methods"
  codemaid_rust__raw__RawItem ||--o{ codemaid_rust__raw__RawMember : "members"
  codemaid_rust__raw__RawItem ||--o{ codemaid_rust__raw__RawFn : "methods"
  codemaid_rust__raw__RawItem ||--o{ codemaid_rust__raw__RawStep : "flow"
  codemaid_rust__raw__RawModule ||--o{ codemaid_rust__raw__RawUse : "uses"
  codemaid_rust__raw__RawStep ||--|| codemaid_rust__raw__RawCall : "Call"
  codemaid_rust__resolve__Resolver ||--o{ codemaid_rust__raw__RawUse : "uses"
```
