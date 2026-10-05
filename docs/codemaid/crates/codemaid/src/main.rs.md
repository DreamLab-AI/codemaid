---
codemaid: 1
source: crates/codemaid/src/main.rs
module: codemaid_main
language: rust
source_hash: blake3:36111ee157a75a218e151a6b0095bdc2df1daeb4d28754c549d799aeffe02ee8
lines: 178
fragments: 4
---
# `codemaid_main` · crates/codemaid/src/main.rs
> `codemaid` command-line tool.

## structure
```mermaid
classDiagram
  direction LR
  class codemaid_main__Cli["Cli"] {
    <<struct>>
    -cmd: Cmd
  }
  class codemaid_main__Cmd["Cmd"] {
    <<enum>>
    Generate#40;Common#41;
    Verify#40;Common#41;
    Model#40;Common#41;
  }
  class codemaid_main__Common["Common"] {
    <<struct>>
    -path: PathBuf
    -repos: Vec#lt;String#gt;
    -out: PathBuf
    -name: Option#lt;String#gt;
    -tests: bool
    -external: String
    -owner_lanes: bool
    -min_calls: usize
    -public_only: bool
    -no_model: bool
    -pretty: bool
  }
  class codemaid_main {
    <<module>>
    -dir_name(p: &Path) String
    -load(c: &Common, opts: &Options) Result#lt;#40;SourceSet, codemaid::rust::RustOptions#41;, String#gt;
    -main() ExitCode
    -run(cli: Cli) Result#lt;ExitCode, String#gt;
  }
  class codemaid__Options["Options"] {
    <<struct in crates/codemaid/src/lib.rs>>
  }
  class codemaid_model__source__SourceSet["SourceSet"] {
    <<struct in crates/codemaid-model/src/source.rs>>
  }
  class codemaid_rust__RustOptions["RustOptions"] {
    <<struct in crates/codemaid-rust/src/lib.rs>>
  }
  codemaid_main ..> codemaid__Options
  codemaid_main ..> codemaid_main__Cli
  codemaid_main ..> codemaid_main__Common
  codemaid_main ..> codemaid_model__source__SourceSet
  codemaid_main ..> codemaid_rust__RustOptions
  codemaid_main__Cli *-- codemaid_main__Cmd : cmd
  codemaid_main__Cmd *-- codemaid_main__Common : Generate, Verify, Model
```

## `codemaid_main::main`
`fn main() -> ExitCode` · L72-L81
```mermaid
sequenceDiagram
  participant codemaid_main as codemaid_main mod
  participant codemaid_main__Cli as Cli
  codemaid_main->>codemaid_main__Cli: ~Cli::parse()
  codemaid_main->>codemaid_main: run(cli)
```

## `codemaid_main::run`
`fn run(cli: Cli) -> Result<ExitCode, String>` · L83-L148
```mermaid
sequenceDiagram
  participant codemaid_main as codemaid_main mod
  participant codemaid__Options as Options
  participant codemaid_rust as codemaid_rust mod
  participant serde_json as serde_json ext
  participant codemaid_corpus as codemaid_corpus mod
  participant codemaid_model__codebase__Codebase as Codebase
  participant codemaid_corpus__contract as contract mod
  participant codemaid_corpus__index__Index as Index
  codemaid_main->>codemaid__Options: ~Options::default()
  opt other
    Note over codemaid_main: return Err(format!(#quot;--external must be all, non-std or …
  end
  codemaid_main->>codemaid_main: load(c, &opts)?
  codemaid_main->>codemaid_rust: rust::extract(&sources, &rust_opts)
  opt let Cmd::Model(_) = cli.cmd
    codemaid_main->>serde_json: serde_json::to_string_pretty(&extraction.codebase)
    Note over codemaid_main: return Ok(ExitCode::SUCCESS)
  end
  codemaid_main->>codemaid_corpus: codemaid::generate(&extraction.codebase, &opts.corpus)
  codemaid_main->>codemaid_model__codebase__Codebase: ~stats()
  alt Cmd::Generate(_)
    codemaid_main->>codemaid_corpus__contract: corpus::write(&c.out, &corpus)
    codemaid_main->>codemaid_corpus__index__Index: ~fragments()
  else Cmd::Verify(_)
    codemaid_main->>codemaid_corpus__contract: corpus::verify(&c.out, &corpus)
  end
```

## `codemaid_main::load`
`fn load(c: &Common, opts: &Options) -> Result<(SourceSet, codemaid::rust::RustOptions), String>` · L150-L171
```mermaid
sequenceDiagram
  participant codemaid_main as codemaid_main mod
  participant codemaid_rust as codemaid_rust mod
  participant codemaid as codemaid mod
  opt c.repos.is_empty()
    codemaid_main->>codemaid_rust: rust::extract_dir(&c.path, &ro)
    opt ro.name == #quot;codebase#quot;
      codemaid_main->>codemaid_main: dir_name(&c.path)
    end
    Note over codemaid_main: return Ok((sources, ro))
  end
  codemaid_main->>codemaid: codemaid::load_repos(&refs)
```
