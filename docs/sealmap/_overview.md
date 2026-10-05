---
sealmap: 1
kind: overview
codebase: sealmap
---
# sealmap overview
37 files · 437 symbols · 1217 relations · 161 flows · 745 calls

## crates
```mermaid
flowchart LR
  sealmap(["sealmap"])
  sealmap_corpus(["sealmap_corpus"])
  sealmap_frontend(["sealmap_frontend"])
  sealmap_main(["sealmap_main"])
  sealmap_mermaid(["sealmap_mermaid"])
  sealmap_model(["sealmap_model"])
  sealmap_rust(["sealmap_rust"])
  Callee{{"Callee"}}
  RawCall{{"RawCall"}}
  RawStep{{"RawStep"}}
  Recv{{"Recv"}}
  Segs{{"Segs"}}
  blake3{{"blake3"}}
  clap{{"clap"}}
  ignore{{"ignore"}}
  proc_macro2{{"proc_macro2"}}
  quote{{"quote"}}
  rayon{{"rayon"}}
  serde{{"serde"}}
  serde_json{{"serde_json"}}
  syn{{"syn"}}
  sealmap -->|"9"| sealmap_corpus
  sealmap -->|"1"| sealmap_frontend
  sealmap -->|"1"| sealmap_mermaid
  sealmap -->|"8"| sealmap_model
  sealmap -->|"4"| sealmap_rust
  sealmap_corpus -->|"69"| sealmap_mermaid
  sealmap_corpus -->|"144"| sealmap_model
  sealmap_corpus -->|"4"| serde
  sealmap_corpus -->|"2"| serde_json
  sealmap_frontend -->|"1"| rayon
  sealmap_frontend -->|"44"| sealmap_model
  sealmap_main -->|"3"| clap
  sealmap_main -->|"4"| sealmap
  sealmap_main -->|"7"| sealmap_corpus
  sealmap_main -->|"1"| sealmap_frontend
  sealmap_main -->|"3"| sealmap_model
  sealmap_main -->|"3"| sealmap_rust
  sealmap_main -->|"1"| serde_json
  sealmap_model -->|"1"| blake3
  sealmap_model -->|"1"| ignore
  sealmap_model -->|"12"| serde
  sealmap_rust -->|"1"| Callee
  sealmap_rust -->|"1"| RawCall
  sealmap_rust -->|"11"| RawStep
  sealmap_rust -->|"5"| Recv
  sealmap_rust -->|"12"| Segs
  sealmap_rust -->|"6"| proc_macro2
  sealmap_rust -->|"3"| quote
  sealmap_rust -->|"73"| sealmap_frontend
  sealmap_rust -->|"104"| sealmap_model
  sealmap_rust -->|"62"| syn
```

## modules: sealmap_corpus
```mermaid
flowchart LR
  sealmap_corpus["sealmap_corpus"]
  sealmap_corpus__contract["contract"]
  sealmap_corpus__document["document"]
  sealmap_corpus__index["index"]
  sealmap_corpus__naming["naming"]
  sealmap_corpus__overview["overview"]
  sealmap_corpus__sequence["sequence"]
  sealmap_corpus__structure["structure"]
  sealmap_corpus -->|"8"| sealmap_corpus__contract
  sealmap_corpus -->|"1"| sealmap_corpus__document
  sealmap_corpus -->|"8"| sealmap_corpus__index
  sealmap_corpus -->|"1"| sealmap_corpus__naming
  sealmap_corpus -->|"1"| sealmap_corpus__overview
  sealmap_corpus__contract -->|"7"| sealmap_corpus
  sealmap_corpus__contract -->|"4"| sealmap_corpus__document
  sealmap_corpus__document -->|"7"| sealmap_corpus
  sealmap_corpus__document -->|"5"| sealmap_corpus__index
  sealmap_corpus__document -->|"2"| sealmap_corpus__sequence
  sealmap_corpus__document -->|"2"| sealmap_corpus__structure
  sealmap_corpus__naming -->|"3"| sealmap_corpus
  sealmap_corpus__overview -->|"6"| sealmap_corpus
  sealmap_corpus__overview -->|"4"| sealmap_corpus__naming
  sealmap_corpus__sequence -->|"3"| sealmap_corpus
  sealmap_corpus__sequence -->|"9"| sealmap_corpus__naming
  sealmap_corpus__structure -->|"6"| sealmap_corpus
  sealmap_corpus__structure -->|"2"| sealmap_corpus__naming
```

## modules: sealmap_frontend
```mermaid
flowchart LR
  sealmap_frontend["sealmap_frontend"]
  sealmap_frontend__confidence["confidence"]
  sealmap_frontend__ids["ids"]
  sealmap_frontend__isolate["isolate"]
  sealmap_frontend__labels["labels"]
  sealmap_frontend__lower["lower"]
  sealmap_frontend__raw["raw"]
  sealmap_frontend -->|"2"| sealmap_frontend__confidence
  sealmap_frontend -->|"5"| sealmap_frontend__raw
  sealmap_frontend__lower -->|"5"| sealmap_frontend__raw
```

## modules: sealmap_mermaid
```mermaid
flowchart LR
  sealmap_mermaid["sealmap_mermaid"]
  sealmap_mermaid__class["class"]
  sealmap_mermaid__er["er"]
  sealmap_mermaid__escape["escape"]
  sealmap_mermaid__flowchart["flowchart"]
  sealmap_mermaid__sequence["sequence"]
  sealmap_mermaid__writer["writer"]
  sealmap_mermaid -->|"5"| sealmap_mermaid__class
  sealmap_mermaid -->|"2"| sealmap_mermaid__er
  sealmap_mermaid -->|"3"| sealmap_mermaid__escape
  sealmap_mermaid -->|"3"| sealmap_mermaid__flowchart
  sealmap_mermaid -->|"4"| sealmap_mermaid__sequence
  sealmap_mermaid -->|"1"| sealmap_mermaid__writer
  sealmap_mermaid__class -->|"14"| sealmap_mermaid__escape
  sealmap_mermaid__class -->|"8"| sealmap_mermaid__writer
  sealmap_mermaid__er -->|"11"| sealmap_mermaid__escape
  sealmap_mermaid__er -->|"5"| sealmap_mermaid__writer
  sealmap_mermaid__flowchart -->|"6"| sealmap_mermaid__class
  sealmap_mermaid__flowchart -->|"8"| sealmap_mermaid__escape
  sealmap_mermaid__flowchart -->|"5"| sealmap_mermaid__writer
  sealmap_mermaid__sequence -->|"16"| sealmap_mermaid__escape
  sealmap_mermaid__sequence -->|"8"| sealmap_mermaid__writer
```

## modules: sealmap_model
```mermaid
flowchart LR
  sealmap_model["sealmap_model"]
  sealmap_model__codebase["codebase"]
  sealmap_model__flow["flow"]
  sealmap_model__hash["hash"]
  sealmap_model__path["path"]
  sealmap_model__source["source"]
  sealmap_model__symbol["symbol"]
  sealmap_model -->|"2"| sealmap_model__codebase
  sealmap_model -->|"6"| sealmap_model__flow
  sealmap_model -->|"1"| sealmap_model__hash
  sealmap_model -->|"2"| sealmap_model__path
  sealmap_model -->|"3"| sealmap_model__source
  sealmap_model -->|"10"| sealmap_model__symbol
  sealmap_model__codebase -->|"1"| sealmap_model__flow
  sealmap_model__codebase -->|"4"| sealmap_model__path
  sealmap_model__codebase -->|"4"| sealmap_model__source
  sealmap_model__codebase -->|"28"| sealmap_model__symbol
  sealmap_model__flow -->|"6"| sealmap_model__symbol
  sealmap_model__source -->|"6"| sealmap_model__hash
  sealmap_model__source -->|"12"| sealmap_model__path
  sealmap_model__source -->|"3"| sealmap_model__symbol
  sealmap_model__symbol -->|"2"| sealmap_model__flow
  sealmap_model__symbol -->|"3"| sealmap_model__path
```

## modules: sealmap_rust
```mermaid
flowchart LR
  sealmap_rust["sealmap_rust"]
  sealmap_rust__collect["collect"]
  sealmap_rust__layout["layout"]
  sealmap_rust__raw["raw"]
  sealmap_rust__resolve["resolve"]
  sealmap_rust__tidy["tidy"]
  sealmap_rust -->|"2"| sealmap_rust__collect
  sealmap_rust -->|"2"| sealmap_rust__layout
  sealmap_rust -->|"1"| sealmap_rust__raw
  sealmap_rust -->|"1"| sealmap_rust__resolve
  sealmap_rust__collect -->|"3"| sealmap_rust
  sealmap_rust__collect -->|"3"| sealmap_rust__layout
  sealmap_rust__collect -->|"6"| sealmap_rust__raw
  sealmap_rust__collect -->|"11"| sealmap_rust__tidy
  sealmap_rust__raw -->|"2"| sealmap_rust__layout
  sealmap_rust__resolve -->|"5"| sealmap_rust
  sealmap_rust__resolve -->|"7"| sealmap_rust__raw
```

## data: sealmap_corpus
```mermaid
erDiagram
  sealmap_corpus__Corpus["Corpus"] {
    BTreeMap[SourcePath_String] files "BTreeMap<SourcePath, String>"
    Index index
  }
  sealmap_corpus__CorpusOptions["CorpusOptions"] {
    usize min_calls
    usize max_messages
    usize max_edges
    usize max_entities
    bool include_private
    ExternalLanes external_lanes
    bool emit_model
    bool pretty_json
  }
  sealmap_corpus__ExternalLanes["ExternalLanes"]
  sealmap_corpus__contract__Drift["Drift"]
  sealmap_corpus__contract__DriftEntry["DriftEntry"] {
    SourcePath path
    Drift drift
  }
  sealmap_corpus__contract__Report["Report"] {
    Vec[DriftEntry] entries
    usize checked
  }
  sealmap_corpus__index__CallRef["CallRef"] {
    SymbolId target
    Confidence confidence
    u32 line
    Option[String] expands
  }
  sealmap_corpus__index__DocumentEntry["DocumentEntry"] {
    SourcePath source
    SourcePath document
    SymbolId module
    ContentHash source_hash
    ContentHash document_hash
    Vec[FragmentEntry] fragments
  }
  sealmap_corpus__index__FragmentEntry["FragmentEntry"] {
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
  sealmap_corpus__index__FragmentKind["FragmentKind"]
  sealmap_corpus__index__Index["Index"] {
    u32 schema
    String generator
    String codebase
    CodebaseStats stats
    Vec[DocumentEntry] documents
  }
  sealmap_corpus__sequence__Ctx["Ctx"] {
    __aCodebase cb "&'a Codebase"
    __aCorpusOptions opts "&'a CorpusOptions"
    SymbolId caller
    Vec[(SymbolId_String_bool)] lanes "Vec<(SymbolId, String, bool)>"
    usize budget
    usize dropped
  }
  sealmap_corpus__Corpus ||--|| sealmap_corpus__index__Index : "index"
  sealmap_corpus__CorpusOptions ||--|| sealmap_corpus__ExternalLanes : "external_lanes"
  sealmap_corpus__contract__DriftEntry ||--|| sealmap_corpus__contract__Drift : "drift"
  sealmap_corpus__contract__Report ||--o{ sealmap_corpus__contract__DriftEntry : "entries"
  sealmap_corpus__index__DocumentEntry ||--o{ sealmap_corpus__index__FragmentEntry : "fragments"
  sealmap_corpus__index__FragmentEntry ||--|| sealmap_corpus__index__FragmentKind : "kind"
  sealmap_corpus__index__FragmentEntry ||--o{ sealmap_corpus__index__CallRef : "calls"
  sealmap_corpus__index__Index ||--o{ sealmap_corpus__index__DocumentEntry : "documents"
  sealmap_corpus__sequence__Ctx ||..|| sealmap_corpus__CorpusOptions : "opts"
```

## data: sealmap_frontend
```mermaid
erDiagram
  sealmap_frontend__Diagnostic["Diagnostic"] {
    SourcePath file
    String message
  }
  sealmap_frontend__Extraction["Extraction"] {
    Codebase codebase
    Vec[Diagnostic] diagnostics
  }
  sealmap_frontend__raw__Callee["Callee"]
  sealmap_frontend__raw__RawCall["RawCall"] {
    Callee callee
    String label
    CallKind kind
    bool awaited
    bool fallible
    u32 line
  }
  sealmap_frontend__raw__RawStep["RawStep"]
  sealmap_frontend__raw__Recv["Recv"]
  sealmap_frontend__Extraction ||--o{ sealmap_frontend__Diagnostic : "diagnostics"
  sealmap_frontend__raw__Callee ||--|| sealmap_frontend__raw__Recv : "Method"
  sealmap_frontend__raw__RawCall ||--|| sealmap_frontend__raw__Callee : "callee"
  sealmap_frontend__raw__RawStep ||--|| sealmap_frontend__raw__RawCall : "Call"
```

## data: sealmap_main
```mermaid
erDiagram
  sealmap_main__Cli["Cli"] {
    Cmd cmd
  }
  sealmap_main__Cmd["Cmd"]
  sealmap_main__Common["Common"] {
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
  sealmap_main__Cli ||--|| sealmap_main__Cmd : "cmd"
  sealmap_main__Cmd ||--|| sealmap_main__Common : "Generate"
  sealmap_main__Cmd ||--|| sealmap_main__Common : "Verify"
  sealmap_main__Cmd ||--|| sealmap_main__Common : "Model"
```

## data: sealmap_mermaid
```mermaid
erDiagram
  sealmap_mermaid__class__Class["Class"] {
    Ident id
    String label
    Option[String] annotation
    Vec[String] members
  }
  sealmap_mermaid__class__ClassDiagram["ClassDiagram"] {
    Option[Direction] direction_
    Vec[Class] classes
    Vec[ClassRelation] relations
  }
  sealmap_mermaid__class__ClassRelation["ClassRelation"] {
    Ident from
    Ident to
    ClassRelationKind kind
    Option[String] label
  }
  sealmap_mermaid__class__ClassRelationKind["ClassRelationKind"]
  sealmap_mermaid__class__Direction["Direction"]
  sealmap_mermaid__er__Attr["Attr"] {
    String ty
    String name
    Option[__staticstr] key "Option<&'static str>"
    Option[String] comment
  }
  sealmap_mermaid__er__Cardinality["Cardinality"]
  sealmap_mermaid__er__Entity["Entity"] {
    Ident id
    String label
    Vec[Attr] attrs
  }
  sealmap_mermaid__er__ErDiagram["ErDiagram"] {
    Vec[Entity] entities
    Vec[Rel] rels
  }
  sealmap_mermaid__er__Rel["Rel"] {
    Ident from
    Cardinality from_card
    Ident to
    Cardinality to_card
    bool identifying
    String label
  }
  sealmap_mermaid__escape__Ident["Ident"] {
    String _0
  }
  sealmap_mermaid__flowchart__EdgeStyle["EdgeStyle"]
  sealmap_mermaid__flowchart__Flowchart["Flowchart"] {
    Direction direction_
    Vec[Stmt] body
  }
  sealmap_mermaid__flowchart__NodeShape["NodeShape"]
  sealmap_mermaid__flowchart__Stmt["Stmt"]
  sealmap_mermaid__sequence__Arrow["Arrow"]
  sealmap_mermaid__sequence__BlockKind["BlockKind"]
  sealmap_mermaid__sequence__Item["Item"]
  sealmap_mermaid__sequence__SeqBuilder["SeqBuilder"] {
    Vec[Item] items
  }
  sealmap_mermaid__sequence__SequenceDiagram["SequenceDiagram"] {
    Option[String] title
    bool autonumber_
    Vec[(Ident_String_bool)] participants "Vec<(Ident, String, bool)>"
    SeqBuilder body
  }
  sealmap_mermaid__class__Class ||--|| sealmap_mermaid__escape__Ident : "id"
  sealmap_mermaid__class__ClassDiagram ||--o| sealmap_mermaid__class__Direction : "direction"
  sealmap_mermaid__class__ClassDiagram ||--o{ sealmap_mermaid__class__Class : "classes"
  sealmap_mermaid__class__ClassDiagram ||--o{ sealmap_mermaid__class__ClassRelation : "relations"
  sealmap_mermaid__class__ClassRelation ||--|| sealmap_mermaid__escape__Ident : "from"
  sealmap_mermaid__class__ClassRelation ||--|| sealmap_mermaid__escape__Ident : "to"
  sealmap_mermaid__class__ClassRelation ||--|| sealmap_mermaid__class__ClassRelationKind : "kind"
  sealmap_mermaid__er__Entity ||--|| sealmap_mermaid__escape__Ident : "id"
  sealmap_mermaid__er__Entity ||--o{ sealmap_mermaid__er__Attr : "attrs"
  sealmap_mermaid__er__ErDiagram ||--o{ sealmap_mermaid__er__Entity : "entities"
  sealmap_mermaid__er__ErDiagram ||--o{ sealmap_mermaid__er__Rel : "rels"
  sealmap_mermaid__er__Rel ||--|| sealmap_mermaid__escape__Ident : "from"
  sealmap_mermaid__er__Rel ||--|| sealmap_mermaid__er__Cardinality : "from_card"
  sealmap_mermaid__er__Rel ||--|| sealmap_mermaid__escape__Ident : "to"
  sealmap_mermaid__er__Rel ||--|| sealmap_mermaid__er__Cardinality : "to_card"
  sealmap_mermaid__flowchart__Flowchart ||--|| sealmap_mermaid__class__Direction : "direction"
  sealmap_mermaid__flowchart__Flowchart ||--o{ sealmap_mermaid__flowchart__Stmt : "body"
  sealmap_mermaid__flowchart__Stmt ||--|| sealmap_mermaid__escape__Ident : "Node"
  sealmap_mermaid__flowchart__Stmt ||--|| sealmap_mermaid__flowchart__NodeShape : "Node"
  sealmap_mermaid__flowchart__Stmt ||--|| sealmap_mermaid__escape__Ident : "Edge"
  sealmap_mermaid__flowchart__Stmt ||--|| sealmap_mermaid__flowchart__EdgeStyle : "Edge"
  sealmap_mermaid__flowchart__Stmt ||--|| sealmap_mermaid__escape__Ident : "Subgraph"
  sealmap_mermaid__flowchart__Stmt ||--|| sealmap_mermaid__class__Direction : "Subgraph"
  sealmap_mermaid__sequence__Item ||--|| sealmap_mermaid__escape__Ident : "Message"
  sealmap_mermaid__sequence__Item ||--|| sealmap_mermaid__sequence__Arrow : "Message"
  sealmap_mermaid__sequence__Item ||--|| sealmap_mermaid__escape__Ident : "Note"
  sealmap_mermaid__sequence__Item ||--|| sealmap_mermaid__sequence__BlockKind : "Block"
  sealmap_mermaid__sequence__SeqBuilder ||--o{ sealmap_mermaid__sequence__Item : "items"
  sealmap_mermaid__sequence__SequenceDiagram ||--o{ sealmap_mermaid__escape__Ident : "participants"
  sealmap_mermaid__sequence__SequenceDiagram ||--|| sealmap_mermaid__sequence__SeqBuilder : "body"
```

## data: sealmap_model
```mermaid
erDiagram
  sealmap_model__codebase__Codebase["Codebase"] {
    u32 schema
    String name
    BTreeMap[SourcePath_SourceFile] files "BTreeMap<SourcePath, SourceFile>"
    BTreeMap[SymbolId_Symbol] symbols "BTreeMap<SymbolId, Symbol>"
    BTreeSet[Relation] relations
  }
  sealmap_model__flow__Arm["Arm"] {
    String label
    Vec[Step] steps
  }
  sealmap_model__flow__Call["Call"] {
    SymbolId target
    String label
    CallKind kind
    Confidence confidence
    bool awaited
    bool fallible
    u32 line
  }
  sealmap_model__flow__CallKind["CallKind"]
  sealmap_model__flow__Exit["Exit"] {
    String label
    u32 line
  }
  sealmap_model__flow__Flow["Flow"] {
    Vec[Step] steps
  }
  sealmap_model__flow__Step["Step"]
  sealmap_model__hash__ContentHash["ContentHash"] {
    String _0
  }
  sealmap_model__path__SourcePath["SourcePath"] {
    String _0
  }
  sealmap_model__source__SourceFile["SourceFile"] {
    SourcePath path
    String language
    SymbolId module
    ContentHash hash
    u32 lines
  }
  sealmap_model__source__SourceSet["SourceSet"] {
    BTreeMap[SourcePath_String] files "BTreeMap<SourcePath, String>"
  }
  sealmap_model__symbol__Confidence["Confidence"]
  sealmap_model__symbol__Member["Member"] {
    String name
    MemberKind kind
    Option[String] ty
    Visibility visibility
    Span span
    Vec[SymbolId] refs
  }
  sealmap_model__symbol__MemberKind["MemberKind"]
  sealmap_model__symbol__Relation["Relation"] {
    SymbolId from
    SymbolId to
    RelationKind kind
    Confidence confidence
  }
  sealmap_model__symbol__RelationKind["RelationKind"]
  sealmap_model__symbol__Span["Span"] {
    u32 start_line
    u32 start_col
    u32 end_line
    u32 end_col
  }
  sealmap_model__symbol__Symbol["Symbol"] {
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
  sealmap_model__symbol__SymbolId["SymbolId"] {
    String _0
  }
  sealmap_model__symbol__SymbolKind["SymbolKind"]
  sealmap_model__symbol__Visibility["Visibility"]
  sealmap_model__codebase__Codebase ||--o{ sealmap_model__path__SourcePath : "files"
  sealmap_model__codebase__Codebase ||--o{ sealmap_model__source__SourceFile : "files"
  sealmap_model__codebase__Codebase ||--o{ sealmap_model__symbol__SymbolId : "symbols"
  sealmap_model__codebase__Codebase ||--o{ sealmap_model__symbol__Symbol : "symbols"
  sealmap_model__codebase__Codebase ||--o{ sealmap_model__symbol__Relation : "relations"
  sealmap_model__flow__Arm ||--o{ sealmap_model__flow__Step : "steps"
  sealmap_model__flow__Call ||--|| sealmap_model__symbol__SymbolId : "target"
  sealmap_model__flow__Call ||--|| sealmap_model__flow__CallKind : "kind"
  sealmap_model__flow__Call ||--|| sealmap_model__symbol__Confidence : "confidence"
  sealmap_model__flow__Flow ||--o{ sealmap_model__flow__Step : "steps"
  sealmap_model__flow__Step ||--|| sealmap_model__flow__Call : "Call"
  sealmap_model__flow__Step ||--|| sealmap_model__flow__Arm : "Branch"
  sealmap_model__flow__Step ||--|| sealmap_model__flow__Arm : "Parallel"
  sealmap_model__flow__Step ||--|| sealmap_model__flow__Exit : "Return"
  sealmap_model__source__SourceFile ||--|| sealmap_model__path__SourcePath : "path"
  sealmap_model__source__SourceFile ||--|| sealmap_model__symbol__SymbolId : "module"
  sealmap_model__source__SourceFile ||--|| sealmap_model__hash__ContentHash : "hash"
  sealmap_model__source__SourceSet ||--o{ sealmap_model__path__SourcePath : "files"
  sealmap_model__symbol__Member ||--|| sealmap_model__symbol__MemberKind : "kind"
  sealmap_model__symbol__Member ||--|| sealmap_model__symbol__Visibility : "visibility"
  sealmap_model__symbol__Member ||--|| sealmap_model__symbol__Span : "span"
  sealmap_model__symbol__Member ||--o{ sealmap_model__symbol__SymbolId : "refs"
  sealmap_model__symbol__Relation ||--|| sealmap_model__symbol__SymbolId : "from"
  sealmap_model__symbol__Relation ||--|| sealmap_model__symbol__SymbolId : "to"
  sealmap_model__symbol__Relation ||--|| sealmap_model__symbol__RelationKind : "kind"
  sealmap_model__symbol__Relation ||--|| sealmap_model__symbol__Confidence : "confidence"
  sealmap_model__symbol__Symbol ||--|| sealmap_model__symbol__SymbolId : "id"
  sealmap_model__symbol__Symbol ||--|| sealmap_model__symbol__SymbolKind : "kind"
  sealmap_model__symbol__Symbol ||--|| sealmap_model__symbol__Visibility : "visibility"
  sealmap_model__symbol__Symbol ||--|| sealmap_model__path__SourcePath : "file"
  sealmap_model__symbol__Symbol ||--|| sealmap_model__symbol__Span : "span"
  sealmap_model__symbol__Symbol ||--o| sealmap_model__symbol__SymbolId : "parent"
  sealmap_model__symbol__Symbol ||--o{ sealmap_model__symbol__Member : "members"
  sealmap_model__symbol__Symbol ||--o| sealmap_model__flow__Flow : "flow"
```

## data: sealmap_rust
```mermaid
erDiagram
  sealmap_rust__RustOptions["RustOptions"] {
    String name
    bool include_tests
    ExternalCalls external_calls
  }
  sealmap_rust__collect__Collector["Collector"] {
    __aRustOptions opts "&'a RustOptions"
    __amutRawFile raw "&'a mut RawFile"
  }
  sealmap_rust__layout__FileRole["FileRole"] {
    String crate_name
    Vec[String] module
    TargetKind target
  }
  sealmap_rust__layout__TargetKind["TargetKind"]
  sealmap_rust__raw__RawFile["RawFile"] {
    SourcePath path
    FileRole role
    u32 text_lines
    ContentHash hash
    Vec[RawModule] modules
    Vec[RawItem] items
    Vec[RawImpl] impls
    Option[String] error
  }
  sealmap_rust__raw__RawFn["RawFn"] {
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
  sealmap_rust__raw__RawImpl["RawImpl"] {
    Segs module
    Option[Segs] self_ty
    Option[(Segs_String)] trait_ "Option<(Segs, String)>"
    Vec[RawFn] methods
  }
  sealmap_rust__raw__RawItem["RawItem"] {
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
  sealmap_rust__raw__RawMember["RawMember"] {
    String name
    MemberKind kind
    Option[String] ty
    Visibility vis
    Span span
    Vec[Segs] refs
  }
  sealmap_rust__raw__RawModule["RawModule"] {
    Segs path
    Span span
    Option[String] doc
    Visibility vis
    Vec[RawUse] uses
    Vec[String] tags
  }
  sealmap_rust__raw__RawUse["RawUse"] {
    String alias
    Segs target
  }
  sealmap_rust__resolve__Resolver["Resolver"] {
    BTreeSet[String] crates
    BTreeMap[String_BTreeMap[String_SymbolId]] items "BTreeMap<String, BTreeMap<String, SymbolId>>"
    BTreeMap[String_Vec[RawUse]] uses "BTreeMap<String, Vec<RawUse>>"
    BTreeSet[SymbolId] internal
    BTreeMap[SymbolId_(String_BTreeMap[String_Vec[Segs]])] fields "BTreeMap<SymbolId, (String, BTreeMap<String, Vec<Segs>>)>"
    BTreeMap[SymbolId_BTreeMap[String_SymbolId]] methods "BTreeMap<SymbolId, BTreeMap<String, SymbolId>>"
    BTreeMap[String_BTreeSet[SymbolId]] by_name "BTreeMap<String, BTreeSet<SymbolId>>"
    BTreeMap[SymbolId_BTreeSet[SymbolId]] impls "BTreeMap<SymbolId, BTreeSet<SymbolId>>"
    BTreeMap[SymbolId_BTreeSet[String]] trait_methods "BTreeMap<SymbolId, BTreeSet<String>>"
    BTreeMap[String_Vec[String]] globs "BTreeMap<String, Vec<String>>"
  }
  sealmap_rust__collect__Collector ||..|| sealmap_rust__RustOptions : "opts"
  sealmap_rust__collect__Collector ||..|| sealmap_rust__raw__RawFile : "raw"
  sealmap_rust__layout__FileRole ||--|| sealmap_rust__layout__TargetKind : "target"
  sealmap_rust__raw__RawFile ||--|| sealmap_rust__layout__FileRole : "role"
  sealmap_rust__raw__RawFile ||--o{ sealmap_rust__raw__RawModule : "modules"
  sealmap_rust__raw__RawFile ||--o{ sealmap_rust__raw__RawItem : "items"
  sealmap_rust__raw__RawFile ||--o{ sealmap_rust__raw__RawImpl : "impls"
  sealmap_rust__raw__RawImpl ||--o{ sealmap_rust__raw__RawFn : "methods"
  sealmap_rust__raw__RawItem ||--o{ sealmap_rust__raw__RawMember : "members"
  sealmap_rust__raw__RawItem ||--o{ sealmap_rust__raw__RawFn : "methods"
  sealmap_rust__raw__RawModule ||--o{ sealmap_rust__raw__RawUse : "uses"
  sealmap_rust__resolve__Resolver ||--o{ sealmap_rust__raw__RawUse : "uses"
```
