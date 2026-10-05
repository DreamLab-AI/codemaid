---
sealmap: 1
source: crates/sealmap/src/main.rs
module: sealmap_main
language: rust
source_hash: blake3:0e8c2311d0889a2e2d80d29925ab8fa4d8cbf5cf2cb4f991bbffb20137b3eb7f
lines: 182
fragments: 4
---
# `sealmap_main` · crates/sealmap/src/main.rs
> `sealmap` command-line tool.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_main__Cli["Cli"] {
    <<struct>>
    -cmd: Cmd
  }
  class sealmap_main__Cmd["Cmd"] {
    <<enum>>
    Generate#40;Common#41;
    Verify#40;Common#41;
    Model#40;Common#41;
  }
  class sealmap_main__Common["Common"] {
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
  class sealmap_main {
    <<module>>
    -dir_name(p: &Path) String
    -load(c: &Common, opts: &Options) Result#lt;#40;SourceSet, sealmap::rust::RustOptions#41;, String#gt;
    -main() ExitCode
    -run(cli: Cli) Result#lt;ExitCode, String#gt;
  }
  class sealmap__Options["Options"] {
    <<struct in crates/sealmap/src/lib.rs>>
  }
  class sealmap_model__source__SourceSet["SourceSet"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_rust__RustOptions["RustOptions"] {
    <<struct in crates/sealmap-rust/src/lib.rs>>
  }
  sealmap_main ..> sealmap__Options
  sealmap_main ..> sealmap_main__Cli
  sealmap_main ..> sealmap_main__Common
  sealmap_main ..> sealmap_model__source__SourceSet
  sealmap_main ..> sealmap_rust__RustOptions
  sealmap_main__Cli *-- sealmap_main__Cmd : cmd
  sealmap_main__Cmd *-- sealmap_main__Common : Generate, Verify, Model
```

## `sealmap_main::main`
`fn main() -> ExitCode` · L75-L84
```mermaid
sequenceDiagram
  participant sealmap_main as sealmap_main mod
  participant sealmap_main__Cli as Cli
  sealmap_main->>sealmap_main__Cli: ~Cli::parse()
  sealmap_main->>sealmap_main: run(cli)
```

## `sealmap_main::run`
`fn run(cli: Cli) -> Result<ExitCode, String>` · L86-L151
```mermaid
sequenceDiagram
  participant sealmap_main as sealmap_main mod
  participant sealmap__Options as Options
  participant sealmap_rust as sealmap_rust mod
  participant serde_json as serde_json ext
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_model__codebase__Codebase as Codebase
  participant sealmap_corpus__contract as contract mod
  participant sealmap_corpus__index__Index as Index
  sealmap_main->>sealmap__Options: ~Options::default()
  opt other
    Note over sealmap_main: return Err(format!(#quot;--external must be all, non-std or …
  end
  sealmap_main->>sealmap_main: load(c, &opts)?
  sealmap_main->>sealmap_rust: rust::extract(&sources, &rust_opts)
  opt let Cmd::Model(_) = cli.cmd
    sealmap_main->>serde_json: serde_json::to_string_pretty(&extraction.codebase)
    Note over sealmap_main: return Ok(ExitCode::SUCCESS)
  end
  sealmap_main->>sealmap_corpus: sealmap::generate(&extraction.codebase, &opts.corpus)
  sealmap_main->>sealmap_model__codebase__Codebase: ~stats()
  alt Cmd::Generate(_)
    sealmap_main->>sealmap_corpus__contract: corpus::write(&c.out, &corpus)
    sealmap_main->>sealmap_corpus__index__Index: ~fragments()
  else Cmd::Verify(_)
    sealmap_main->>sealmap_corpus__contract: corpus::verify(&c.out, &corpus)
  end
```

## `sealmap_main::load`
`fn load(c: &Common, opts: &Options) -> Result<(SourceSet, sealmap::rust::RustOptions), String>` · L153-L175
```mermaid
sequenceDiagram
  participant sealmap_main as sealmap_main mod
  participant sealmap_rust as sealmap_rust mod
  participant sealmap as sealmap mod
  opt c.repos.is_empty()
    sealmap_main->>sealmap_rust: rust::load_dir(&c.path)
    opt ro.name == #quot;codebase#quot;
      sealmap_main->>sealmap_main: dir_name(&c.path)
    end
    Note over sealmap_main: return Ok((sources, ro))
  end
  sealmap_main->>sealmap: sealmap::load_repos(&refs)
```
