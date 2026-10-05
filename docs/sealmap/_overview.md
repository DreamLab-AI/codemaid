---
sealmap: 2
kind: overview
codebase: sealmap
---
# sealmap overview
41 files · 619 symbols · 1691 relations · 256 flows · 1177 calls

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
  toml{{"toml"}}
  sealmap -->|"9"| sealmap_corpus
  sealmap -->|"1"| sealmap_frontend
  sealmap -->|"1"| sealmap_mermaid
  sealmap -->|"8"| sealmap_model
  sealmap -->|"4"| sealmap_rust
  sealmap_corpus -->|"69"| sealmap_mermaid
  sealmap_corpus -->|"148"| sealmap_model
  sealmap_corpus -->|"4"| serde
  sealmap_corpus -->|"3"| serde_json
  sealmap_frontend -->|"4"| blake3
  sealmap_frontend -->|"2"| rayon
  sealmap_frontend -->|"68"| sealmap_model
  sealmap_main -->|"3"| clap
  sealmap_main -->|"4"| sealmap
  sealmap_main -->|"7"| sealmap_corpus
  sealmap_main -->|"1"| sealmap_frontend
  sealmap_main -->|"3"| sealmap_model
  sealmap_main -->|"3"| sealmap_rust
  sealmap_main -->|"1"| serde_json
  sealmap_mermaid -->|"6"| sealmap_model
  sealmap_model -->|"6"| blake3
  sealmap_model -->|"1"| ignore
  sealmap_model -->|"21"| serde
  sealmap_model -->|"2"| serde_json
  sealmap_rust -->|"1"| Callee
  sealmap_rust -->|"1"| RawCall
  sealmap_rust -->|"11"| RawStep
  sealmap_rust -->|"7"| Recv
  sealmap_rust -->|"15"| Segs
  sealmap_rust -->|"23"| proc_macro2
  sealmap_rust -->|"6"| quote
  sealmap_rust -->|"130"| sealmap_frontend
  sealmap_rust -->|"130"| sealmap_model
  sealmap_rust -->|"123"| syn
  sealmap_rust -->|"1"| toml
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
  sealmap_frontend__fingerprint["fingerprint"]
  sealmap_frontend__ids["ids"]
  sealmap_frontend__isolate["isolate"]
  sealmap_frontend__labels["labels"]
  sealmap_frontend__lower["lower"]
  sealmap_frontend__raw["raw"]
  sealmap_frontend -->|"2"| sealmap_frontend__confidence
  sealmap_frontend -->|"5"| sealmap_frontend__raw
  sealmap_frontend__lower -->|"5"| sealmap_frontend__raw
  sealmap_frontend__raw -->|"5"| sealmap_frontend__labels
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
  sealmap_mermaid__symbol["symbol"]
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
  sealmap_mermaid__symbol -->|"4"| sealmap_mermaid__escape
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
  sealmap_model__sym["sym"]
  sealmap_model__symbol["symbol"]
  sealmap_model -->|"3"| sealmap_model__codebase
  sealmap_model -->|"6"| sealmap_model__flow
  sealmap_model -->|"2"| sealmap_model__hash
  sealmap_model -->|"2"| sealmap_model__path
  sealmap_model -->|"3"| sealmap_model__source
  sealmap_model -->|"10"| sealmap_model__sym
  sealmap_model -->|"9"| sealmap_model__symbol
  sealmap_model__codebase -->|"1"| sealmap_model__flow
  sealmap_model__codebase -->|"1"| sealmap_model__hash
  sealmap_model__codebase -->|"4"| sealmap_model__path
  sealmap_model__codebase -->|"4"| sealmap_model__source
  sealmap_model__codebase -->|"10"| sealmap_model__sym
  sealmap_model__codebase -->|"18"| sealmap_model__symbol
  sealmap_model__flow -->|"3"| sealmap_model__sym
  sealmap_model__flow -->|"3"| sealmap_model__symbol
  sealmap_model__source -->|"6"| sealmap_model__hash
  sealmap_model__source -->|"12"| sealmap_model__path
  sealmap_model__source -->|"3"| sealmap_model__sym
  sealmap_model__symbol -->|"2"| sealmap_model__flow
  sealmap_model__symbol -->|"2"| sealmap_model__hash
  sealmap_model__symbol -->|"3"| sealmap_model__path
  sealmap_model__symbol -->|"14"| sealmap_model__sym
```

## modules: sealmap_rust
```mermaid
flowchart LR
  sealmap_rust["sealmap_rust"]
  sealmap_rust__collect["collect"]
  sealmap_rust__fingerprint["fingerprint"]
  sealmap_rust__layout["layout"]
  sealmap_rust__raw["raw"]
  sealmap_rust__resolve["resolve"]
  sealmap_rust__tidy["tidy"]
  sealmap_rust -->|"2"| sealmap_rust__collect
  sealmap_rust -->|"2"| sealmap_rust__layout
  sealmap_rust -->|"1"| sealmap_rust__raw
  sealmap_rust -->|"1"| sealmap_rust__resolve
  sealmap_rust__collect -->|"3"| sealmap_rust
  sealmap_rust__collect -->|"22"| sealmap_rust__fingerprint
  sealmap_rust__collect -->|"3"| sealmap_rust__layout
  sealmap_rust__collect -->|"7"| sealmap_rust__raw
  sealmap_rust__collect -->|"12"| sealmap_rust__tidy
  sealmap_rust__raw -->|"2"| sealmap_rust__layout
  sealmap_rust__resolve -->|"5"| sealmap_rust
  sealmap_rust__resolve -->|"8"| sealmap_rust__raw
```

## data: sealmap_corpus
```mermaid
erDiagram
  sealmap_corpus___tCorpus["Corpus"] {
    BTreeMap[SourcePath_String] files "BTreeMap<SourcePath, String>"
    Index index
  }
  sealmap_corpus___tCorpusOptions["CorpusOptions"] {
    usize min_calls
    usize max_messages
    usize max_edges
    usize max_entities
    bool include_private
    ExternalLanes external_lanes
    bool emit_model
    bool pretty_json
  }
  sealmap_corpus___tExternalLanes["ExternalLanes"]
  sealmap_corpus__contract___tDrift["Drift"]
  sealmap_corpus__contract___tDriftEntry["DriftEntry"] {
    SourcePath path
    Drift drift
  }
  sealmap_corpus__contract___tReport["Report"] {
    Vec[DriftEntry] entries
    usize checked
  }
  sealmap_corpus__index___tCallRef["CallRef"] {
    SymbolId target
    Confidence confidence
    u32 line
    Option[String] expands
  }
  sealmap_corpus__index___tDocumentEntry["DocumentEntry"] {
    SourcePath source
    SourcePath document
    SymbolId module
    ContentHash source_hash
    ContentHash document_hash
    Vec[FragmentEntry] fragments
  }
  sealmap_corpus__index___tFragmentEntry["FragmentEntry"] {
    String id
    FragmentKind kind
    SourcePath document
    SymbolId symbol
    Span span
    Fingerprint sig_hash
    Fingerprint body_hash
    Vec[SymbolId] participants
    Vec[CallRef] calls
    usize truncated
    ContentHash hash
  }
  sealmap_corpus__index___tFragmentKind["FragmentKind"]
  sealmap_corpus__index___tIndex["Index"] {
    u32 schema_version
    String generator
    String codebase
    CodebaseStats stats
    Vec[DocumentEntry] documents
  }
  sealmap_corpus__sequence___tCtx["Ctx"] {
    __aCodebase cb "&'a Codebase"
    __aCorpusOptions opts "&'a CorpusOptions"
    SymbolId caller
    Vec[(SymbolId_String_bool)] lanes "Vec<(SymbolId, String, bool)>"
    usize budget
    usize dropped
  }
  sealmap_corpus___tCorpus ||--|| sealmap_corpus__index___tIndex : "index"
  sealmap_corpus___tCorpusOptions ||--|| sealmap_corpus___tExternalLanes : "external_lanes"
  sealmap_corpus__contract___tDriftEntry ||--|| sealmap_corpus__contract___tDrift : "drift"
  sealmap_corpus__contract___tReport ||--o{ sealmap_corpus__contract___tDriftEntry : "entries"
  sealmap_corpus__index___tDocumentEntry ||--o{ sealmap_corpus__index___tFragmentEntry : "fragments"
  sealmap_corpus__index___tFragmentEntry ||--|| sealmap_corpus__index___tFragmentKind : "kind"
  sealmap_corpus__index___tFragmentEntry ||--o{ sealmap_corpus__index___tCallRef : "calls"
  sealmap_corpus__index___tIndex ||--o{ sealmap_corpus__index___tDocumentEntry : "documents"
  sealmap_corpus__sequence___tCtx ||..|| sealmap_corpus___tCorpusOptions : "opts"
```

## data: sealmap_frontend
```mermaid
erDiagram
  sealmap_frontend___tDiagnostic["Diagnostic"] {
    SourcePath file
    String message
  }
  sealmap_frontend___tExtraction["Extraction"] {
    Codebase codebase
    Vec[Diagnostic] diagnostics
  }
  sealmap_frontend__fingerprint___tDelim["Delim"]
  sealmap_frontend__fingerprint___tToken["Token"]
  sealmap_frontend__raw___tCallee["Callee"]
  sealmap_frontend__raw___tRawCall["RawCall"] {
    Callee callee
    String label
    CallKind kind
    bool awaited
    bool fallible
    u32 line
  }
  sealmap_frontend__raw___tRawStep["RawStep"]
  sealmap_frontend__raw___tRecv["Recv"]
  sealmap_frontend___tExtraction ||--o{ sealmap_frontend___tDiagnostic : "diagnostics"
  sealmap_frontend__fingerprint___tToken ||--|| sealmap_frontend__fingerprint___tDelim : "Open"
  sealmap_frontend__fingerprint___tToken ||--|| sealmap_frontend__fingerprint___tDelim : "Close"
  sealmap_frontend__raw___tCallee ||--|| sealmap_frontend__raw___tRecv : "Method"
  sealmap_frontend__raw___tRawCall ||--|| sealmap_frontend__raw___tCallee : "callee"
  sealmap_frontend__raw___tRawStep ||--|| sealmap_frontend__raw___tRawCall : "Call"
```

## data: sealmap_main
```mermaid
erDiagram
  sealmap_main___tCli["Cli"] {
    Cmd cmd
  }
  sealmap_main___tCmd["Cmd"]
  sealmap_main___tCommon["Common"] {
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
  sealmap_main___tCli ||--|| sealmap_main___tCmd : "cmd"
  sealmap_main___tCmd ||--|| sealmap_main___tCommon : "Generate"
  sealmap_main___tCmd ||--|| sealmap_main___tCommon : "Verify"
  sealmap_main___tCmd ||--|| sealmap_main___tCommon : "Model"
```

## data: sealmap_mermaid
```mermaid
erDiagram
  sealmap_mermaid__class___tClass["Class"] {
    Ident id
    String label
    Option[String] annotation
    Vec[String] members
  }
  sealmap_mermaid__class___tClassDiagram["ClassDiagram"] {
    Option[Direction] direction_
    Vec[Class] classes
    Vec[ClassRelation] relations
  }
  sealmap_mermaid__class___tClassRelation["ClassRelation"] {
    Ident from
    Ident to
    ClassRelationKind kind
    Option[String] label
  }
  sealmap_mermaid__class___tClassRelationKind["ClassRelationKind"]
  sealmap_mermaid__class___tDirection["Direction"]
  sealmap_mermaid__er___tAttr["Attr"] {
    String ty
    String name
    Option[__staticstr] key "Option<&'static str>"
    Option[String] comment
  }
  sealmap_mermaid__er___tCardinality["Cardinality"]
  sealmap_mermaid__er___tEntity["Entity"] {
    Ident id
    String label
    Vec[Attr] attrs
  }
  sealmap_mermaid__er___tErDiagram["ErDiagram"] {
    Vec[Entity] entities
    Vec[Rel] rels
  }
  sealmap_mermaid__er___tRel["Rel"] {
    Ident from
    Cardinality from_card
    Ident to
    Cardinality to_card
    bool identifying
    String label
  }
  sealmap_mermaid__escape___tIdent["Ident"] {
    String _0
  }
  sealmap_mermaid__flowchart___tEdgeStyle["EdgeStyle"]
  sealmap_mermaid__flowchart___tFlowchart["Flowchart"] {
    Direction direction_
    Vec[Stmt] body
  }
  sealmap_mermaid__flowchart___tNodeShape["NodeShape"]
  sealmap_mermaid__flowchart___tStmt["Stmt"]
  sealmap_mermaid__sequence___tArrow["Arrow"]
  sealmap_mermaid__sequence___tBlockKind["BlockKind"]
  sealmap_mermaid__sequence___tItem["Item"]
  sealmap_mermaid__sequence___tSeqBuilder["SeqBuilder"] {
    Vec[Item] items
  }
  sealmap_mermaid__sequence___tSequenceDiagram["SequenceDiagram"] {
    Option[String] title
    bool autonumber_
    Vec[(Ident_String_bool)] participants "Vec<(Ident, String, bool)>"
    SeqBuilder body
  }
  sealmap_mermaid__class___tClass ||--|| sealmap_mermaid__escape___tIdent : "id"
  sealmap_mermaid__class___tClassDiagram ||--o| sealmap_mermaid__class___tDirection : "direction"
  sealmap_mermaid__class___tClassDiagram ||--o{ sealmap_mermaid__class___tClass : "classes"
  sealmap_mermaid__class___tClassDiagram ||--o{ sealmap_mermaid__class___tClassRelation : "relations"
  sealmap_mermaid__class___tClassRelation ||--|| sealmap_mermaid__escape___tIdent : "from"
  sealmap_mermaid__class___tClassRelation ||--|| sealmap_mermaid__escape___tIdent : "to"
  sealmap_mermaid__class___tClassRelation ||--|| sealmap_mermaid__class___tClassRelationKind : "kind"
  sealmap_mermaid__er___tEntity ||--|| sealmap_mermaid__escape___tIdent : "id"
  sealmap_mermaid__er___tEntity ||--o{ sealmap_mermaid__er___tAttr : "attrs"
  sealmap_mermaid__er___tErDiagram ||--o{ sealmap_mermaid__er___tEntity : "entities"
  sealmap_mermaid__er___tErDiagram ||--o{ sealmap_mermaid__er___tRel : "rels"
  sealmap_mermaid__er___tRel ||--|| sealmap_mermaid__escape___tIdent : "from"
  sealmap_mermaid__er___tRel ||--|| sealmap_mermaid__er___tCardinality : "from_card"
  sealmap_mermaid__er___tRel ||--|| sealmap_mermaid__escape___tIdent : "to"
  sealmap_mermaid__er___tRel ||--|| sealmap_mermaid__er___tCardinality : "to_card"
  sealmap_mermaid__flowchart___tFlowchart ||--|| sealmap_mermaid__class___tDirection : "direction"
  sealmap_mermaid__flowchart___tFlowchart ||--o{ sealmap_mermaid__flowchart___tStmt : "body"
  sealmap_mermaid__flowchart___tStmt ||--|| sealmap_mermaid__escape___tIdent : "Node"
  sealmap_mermaid__flowchart___tStmt ||--|| sealmap_mermaid__flowchart___tNodeShape : "Node"
  sealmap_mermaid__flowchart___tStmt ||--|| sealmap_mermaid__escape___tIdent : "Edge"
  sealmap_mermaid__flowchart___tStmt ||--|| sealmap_mermaid__flowchart___tEdgeStyle : "Edge"
  sealmap_mermaid__flowchart___tStmt ||--|| sealmap_mermaid__escape___tIdent : "Subgraph"
  sealmap_mermaid__flowchart___tStmt ||--|| sealmap_mermaid__class___tDirection : "Subgraph"
  sealmap_mermaid__sequence___tItem ||--|| sealmap_mermaid__escape___tIdent : "Message"
  sealmap_mermaid__sequence___tItem ||--|| sealmap_mermaid__sequence___tArrow : "Message"
  sealmap_mermaid__sequence___tItem ||--|| sealmap_mermaid__escape___tIdent : "Note"
  sealmap_mermaid__sequence___tItem ||--|| sealmap_mermaid__sequence___tBlockKind : "Block"
  sealmap_mermaid__sequence___tSeqBuilder ||--o{ sealmap_mermaid__sequence___tItem : "items"
  sealmap_mermaid__sequence___tSequenceDiagram ||--o{ sealmap_mermaid__escape___tIdent : "participants"
  sealmap_mermaid__sequence___tSequenceDiagram ||--|| sealmap_mermaid__sequence___tSeqBuilder : "body"
```

## data: sealmap_model
```mermaid
erDiagram
  sealmap_model__codebase___tCodebase["Codebase"] {
    u32 schema_version
    String name
    BTreeMap[SourcePath_SourceFile] files "BTreeMap<SourcePath, SourceFile>"
    BTreeMap[SymbolId_Symbol] symbols "BTreeMap<SymbolId, Symbol>"
    BTreeSet[Relation] relations
  }
  sealmap_model__flow___tArm["Arm"] {
    String label
    Vec[Step] steps
  }
  sealmap_model__flow___tCall["Call"] {
    SymbolId target
    String label
    CallKind kind
    Confidence confidence
    bool awaited
    bool fallible
    u32 line
  }
  sealmap_model__flow___tCallKind["CallKind"]
  sealmap_model__flow___tExit["Exit"] {
    String label
    u32 line
  }
  sealmap_model__flow___tFlow["Flow"] {
    Vec[Step] steps
  }
  sealmap_model__flow___tStep["Step"]
  sealmap_model__hash___tContentHash["ContentHash"] {
    String _0
  }
  sealmap_model__hash___tFingerprint["Fingerprint"] {
    _[u8_16] _0 "[u8#59; 16]"
  }
  sealmap_model__path___tSourcePath["SourcePath"] {
    String _0
  }
  sealmap_model__source___tSourceFile["SourceFile"] {
    SourcePath path
    String language
    SymbolId module
    ContentHash hash
    u32 lines
  }
  sealmap_model__source___tSourceSet["SourceSet"] {
    BTreeMap[SourcePath_String] files "BTreeMap<SourcePath, String>"
  }
  sealmap_model__sym___tDescriptor["Descriptor"] {
    String name
    Suffix suffix
  }
  sealmap_model__sym___tDescriptorKind["DescriptorKind"]
  sealmap_model__sym___tDescriptorView["DescriptorView"] {
    Cow[_a_str] name "Cow<'a, str>"
    DescriptorKind kind
    Option[__astr] disambiguator "Option<&'a str>"
  }
  sealmap_model__sym___tGlobalView["GlobalView"] {
    __astr manager "&'a str"
    Cow[_a_str] package "Cow<'a, str>"
    Option[Cow[_a_str]] version "Option<Cow<'a, str>>"
    Vec[DescriptorView[_a]] descriptors "Vec<DescriptorView<'a>>"
  }
  sealmap_model__sym___tIdView["IdView"]
  sealmap_model__sym___tPackage["Package"] {
    String manager
    String name
    Version version
  }
  sealmap_model__sym___tRepr["Repr"]
  sealmap_model__sym___tSuffix["Suffix"]
  sealmap_model__sym___tSymbolId["SymbolId"] {
    Arc[str] _0
  }
  sealmap_model__sym___tVersion["Version"]
  sealmap_model__symbol___tConfidence["Confidence"]
  sealmap_model__symbol___tMember["Member"] {
    String name
    MemberKind kind
    Option[String] ty
    Visibility visibility
    Span span
    Vec[SymbolId] refs
  }
  sealmap_model__symbol___tMemberKind["MemberKind"]
  sealmap_model__symbol___tRelation["Relation"] {
    SymbolId from
    SymbolId to
    RelationKind kind
    Confidence confidence
  }
  sealmap_model__symbol___tRelationKind["RelationKind"]
  sealmap_model__symbol___tSpan["Span"] {
    u32 start_line
    u32 start_col
    u32 end_line
    u32 end_col
  }
  sealmap_model__symbol___tSymbol["Symbol"] {
    SymbolId id
    String name
    SymbolKind kind
    Visibility visibility
    SourcePath file
    Span span
    Fingerprint sig_hash
    Fingerprint body_hash
    Option[SymbolId] parent
    Option[String] signature
    Option[String] doc
    Vec[String] generics
    Vec[String] tags
    Vec[Member] members
    Option[Flow] flow
  }
  sealmap_model__symbol___tSymbolKind["SymbolKind"]
  sealmap_model__symbol___tVisibility["Visibility"]
  sealmap_model__codebase___tCodebase ||--o{ sealmap_model__path___tSourcePath : "files"
  sealmap_model__codebase___tCodebase ||--o{ sealmap_model__source___tSourceFile : "files"
  sealmap_model__codebase___tCodebase ||--o{ sealmap_model__sym___tSymbolId : "symbols"
  sealmap_model__codebase___tCodebase ||--o{ sealmap_model__symbol___tSymbol : "symbols"
  sealmap_model__codebase___tCodebase ||--o{ sealmap_model__symbol___tRelation : "relations"
  sealmap_model__flow___tArm ||--o{ sealmap_model__flow___tStep : "steps"
  sealmap_model__flow___tCall ||--|| sealmap_model__sym___tSymbolId : "target"
  sealmap_model__flow___tCall ||--|| sealmap_model__flow___tCallKind : "kind"
  sealmap_model__flow___tCall ||--|| sealmap_model__symbol___tConfidence : "confidence"
  sealmap_model__flow___tFlow ||--o{ sealmap_model__flow___tStep : "steps"
  sealmap_model__flow___tStep ||--|| sealmap_model__flow___tCall : "Call"
  sealmap_model__flow___tStep ||--|| sealmap_model__flow___tArm : "Branch"
  sealmap_model__flow___tStep ||--|| sealmap_model__flow___tArm : "Parallel"
  sealmap_model__flow___tStep ||--|| sealmap_model__flow___tExit : "Return"
  sealmap_model__source___tSourceFile ||--|| sealmap_model__path___tSourcePath : "path"
  sealmap_model__source___tSourceFile ||--|| sealmap_model__sym___tSymbolId : "module"
  sealmap_model__source___tSourceFile ||--|| sealmap_model__hash___tContentHash : "hash"
  sealmap_model__source___tSourceSet ||--o{ sealmap_model__path___tSourcePath : "files"
  sealmap_model__sym___tDescriptor ||--|| sealmap_model__sym___tSuffix : "suffix"
  sealmap_model__sym___tDescriptorView ||--|| sealmap_model__sym___tDescriptorKind : "kind"
  sealmap_model__sym___tGlobalView ||--o{ sealmap_model__sym___tDescriptorView : "descriptors"
  sealmap_model__sym___tIdView ||--|| sealmap_model__sym___tGlobalView : "Global"
  sealmap_model__sym___tPackage ||--|| sealmap_model__sym___tVersion : "version"
  sealmap_model__sym___tRepr ||--|| sealmap_model__sym___tPackage : "Global"
  sealmap_model__sym___tRepr ||--|| sealmap_model__sym___tDescriptor : "Global"
  sealmap_model__symbol___tMember ||--|| sealmap_model__symbol___tMemberKind : "kind"
  sealmap_model__symbol___tMember ||--|| sealmap_model__symbol___tVisibility : "visibility"
  sealmap_model__symbol___tMember ||--|| sealmap_model__symbol___tSpan : "span"
  sealmap_model__symbol___tMember ||--o{ sealmap_model__sym___tSymbolId : "refs"
  sealmap_model__symbol___tRelation ||--|| sealmap_model__sym___tSymbolId : "from"
  sealmap_model__symbol___tRelation ||--|| sealmap_model__sym___tSymbolId : "to"
  sealmap_model__symbol___tRelation ||--|| sealmap_model__symbol___tRelationKind : "kind"
  sealmap_model__symbol___tRelation ||--|| sealmap_model__symbol___tConfidence : "confidence"
  sealmap_model__symbol___tSymbol ||--|| sealmap_model__sym___tSymbolId : "id"
  sealmap_model__symbol___tSymbol ||--|| sealmap_model__symbol___tSymbolKind : "kind"
  sealmap_model__symbol___tSymbol ||--|| sealmap_model__symbol___tVisibility : "visibility"
  sealmap_model__symbol___tSymbol ||--|| sealmap_model__path___tSourcePath : "file"
  sealmap_model__symbol___tSymbol ||--|| sealmap_model__symbol___tSpan : "span"
  sealmap_model__symbol___tSymbol ||--|| sealmap_model__hash___tFingerprint : "sig_hash"
  sealmap_model__symbol___tSymbol ||--|| sealmap_model__hash___tFingerprint : "body_hash"
  sealmap_model__symbol___tSymbol ||--o| sealmap_model__sym___tSymbolId : "parent"
  sealmap_model__symbol___tSymbol ||--o{ sealmap_model__symbol___tMember : "members"
  sealmap_model__symbol___tSymbol ||--o| sealmap_model__flow___tFlow : "flow"
```

## data: sealmap_rust
```mermaid
erDiagram
  sealmap_rust___tRustOptions["RustOptions"] {
    String name
    bool include_tests
    ExternalCalls external_calls
  }
  sealmap_rust__collect___tCollector["Collector"] {
    __aRustOptions opts "&'a RustOptions"
    __amutRawFile raw "&'a mut RawFile"
  }
  sealmap_rust__layout___tFileRole["FileRole"] {
    String crate_name
    Vec[String] module
    TargetKind target
  }
  sealmap_rust__layout___tTargetKind["TargetKind"]
  sealmap_rust__raw___tRawFile["RawFile"] {
    SourcePath path
    FileRole role
    u32 text_lines
    ContentHash hash
    Vec[RawModule] modules
    Vec[RawItem] items
    Vec[RawImpl] impls
    Option[String] error
  }
  sealmap_rust__raw___tRawFn["RawFn"] {
    String name
    Visibility vis
    Span span
    String signature
    Option[String] doc
    Vec[String] generics
    Vec[String] tags
    Vec[Segs] sig_refs
    Vec[RawStep] flow
    Fingerprint sig_hash
    Fingerprint body_hash
  }
  sealmap_rust__raw___tRawImpl["RawImpl"] {
    Segs module
    Option[Segs] self_ty
    Option[(Segs_String)] trait_ "Option<(Segs, String)>"
    Vec[RawFn] methods
  }
  sealmap_rust__raw___tRawItem["RawItem"] {
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
    Fingerprint sig_hash
    Fingerprint body_hash
  }
  sealmap_rust__raw___tRawMember["RawMember"] {
    String name
    MemberKind kind
    Option[String] ty
    Visibility vis
    Span span
    Vec[Segs] refs
  }
  sealmap_rust__raw___tRawModule["RawModule"] {
    Segs path
    Span span
    Option[String] doc
    Visibility vis
    Vec[RawUse] uses
    Vec[String] tags
    Fingerprint sig_hash
    Fingerprint body_hash
  }
  sealmap_rust__raw___tRawUse["RawUse"] {
    String alias
    Segs target
  }
  sealmap_rust__resolve___tResolver["Resolver"] {
    BTreeSet[String] crates
    BTreeMap[SymbolId_BTreeMap[String_Slots]] items "BTreeMap<SymbolId, BTreeMap<String, Slots>>"
    BTreeMap[SymbolId_Vec[RawUse]] uses "BTreeMap<SymbolId, Vec<RawUse>>"
    BTreeSet[SymbolId] internal
    BTreeMap[SymbolId_(SymbolId_BTreeMap[String_Vec[Segs]])] fields "BTreeMap<SymbolId, (SymbolId, BTreeMap<String, Vec<Segs>>)>"
    BTreeMap[SymbolId_BTreeMap[String_SymbolId]] methods "BTreeMap<SymbolId, BTreeMap<String, SymbolId>>"
    BTreeMap[String_BTreeSet[SymbolId]] by_name "BTreeMap<String, BTreeSet<SymbolId>>"
    BTreeMap[SymbolId_BTreeSet[SymbolId]] impls "BTreeMap<SymbolId, BTreeSet<SymbolId>>"
    BTreeMap[SymbolId_BTreeSet[String]] trait_methods "BTreeMap<SymbolId, BTreeSet<String>>"
    BTreeMap[SymbolId_Vec[SymbolId]] globs "BTreeMap<SymbolId, Vec<SymbolId>>"
  }
  sealmap_rust__resolve___tSlots["Slots"] {
    Option[SymbolId] ty
    Option[SymbolId] value
  }
  sealmap_rust__collect___tCollector ||..|| sealmap_rust___tRustOptions : "opts"
  sealmap_rust__collect___tCollector ||..|| sealmap_rust__raw___tRawFile : "raw"
  sealmap_rust__layout___tFileRole ||--|| sealmap_rust__layout___tTargetKind : "target"
  sealmap_rust__raw___tRawFile ||--|| sealmap_rust__layout___tFileRole : "role"
  sealmap_rust__raw___tRawFile ||--o{ sealmap_rust__raw___tRawModule : "modules"
  sealmap_rust__raw___tRawFile ||--o{ sealmap_rust__raw___tRawItem : "items"
  sealmap_rust__raw___tRawFile ||--o{ sealmap_rust__raw___tRawImpl : "impls"
  sealmap_rust__raw___tRawImpl ||--o{ sealmap_rust__raw___tRawFn : "methods"
  sealmap_rust__raw___tRawItem ||--o{ sealmap_rust__raw___tRawMember : "members"
  sealmap_rust__raw___tRawItem ||--o{ sealmap_rust__raw___tRawFn : "methods"
  sealmap_rust__raw___tRawModule ||--o{ sealmap_rust__raw___tRawUse : "uses"
  sealmap_rust__resolve___tResolver ||--o{ sealmap_rust__resolve___tSlots : "items"
  sealmap_rust__resolve___tResolver ||--o{ sealmap_rust__raw___tRawUse : "uses"
```
