---
sealmap: 2
source: crates/sealmap/src/main.rs
module: "sym:cargo sealmap_main ."
language: rust
source_hash: blake3:0e8c2311d0889a2e2d80d29925ab8fa4d8cbf5cf2cb4f991bbffb20137b3eb7f
lines: 182
fragments: 4
---
# `sym:cargo sealmap_main .` · crates/sealmap/src/main.rs
> `sealmap` command-line tool.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_main___tCli["Cli"] {
    <<struct>>
    -cmd: Cmd
  }
  class sealmap_main___tCmd["Cmd"] {
    <<enum>>
    Generate#40;Common#41;
    Verify#40;Common#41;
    Model#40;Common#41;
  }
  class sealmap_main___tCommon["Common"] {
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
  class sealmap___tOptions["Options"] {
    <<struct in crates/sealmap/src/lib.rs>>
  }
  class sealmap_model__source___tSourceSet["SourceSet"] {
    <<struct in crates/sealmap-model/src/source.rs>>
  }
  class sealmap_rust___tRustOptions["RustOptions"] {
    <<struct in crates/sealmap-rust/src/lib.rs>>
  }
  sealmap_main ..> sealmap___tOptions
  sealmap_main ..> sealmap_main___tCli
  sealmap_main ..> sealmap_main___tCommon
  sealmap_main ..> sealmap_model__source___tSourceSet
  sealmap_main ..> sealmap_rust___tRustOptions
  sealmap_main___tCli *-- sealmap_main___tCmd : cmd
  sealmap_main___tCmd *-- sealmap_main___tCommon : Generate, Verify, Model
```

## `sym:cargo sealmap_main . main().`
`fn main() -> ExitCode` · L75-L84
```mermaid
sequenceDiagram
  participant sealmap_main as sealmap_main mod
  participant _sealmap_main as sealmap_main ext
  sealmap_main->>_sealmap_main: ~Cli::Cli::parse()
  sealmap_main->>sealmap_main: run(cli)
```

## `sym:cargo sealmap_main . run().`
`fn run(cli: Cli) -> Result<ExitCode, String>` · L86-L151
```mermaid
sequenceDiagram
  participant sealmap_main as sealmap_main mod
  participant _sealmap as sealmap ext
  participant sealmap_rust as sealmap_rust mod
  participant _serde_json as serde_json ext
  participant sealmap_corpus as sealmap_corpus mod
  participant sealmap_model__codebase___tCodebase as Codebase
  participant sealmap_corpus__contract as contract mod
  participant sealmap_corpus__index___tIndex as Index
  sealmap_main->>_sealmap: ~Options::Options::default()
  opt other
    Note over sealmap_main: return Err(format!(#quot;--external must be all, non-std or …
  end
  sealmap_main->>sealmap_main: load(c, &opts)?
  sealmap_main->>sealmap_rust: rust::extract(&sources, &rust_opts)
  opt let Cmd::Model(_) = cli.cmd
    sealmap_main->>_serde_json: serde_json::to_string_pretty(&extraction.codebase)
    Note over sealmap_main: return Ok(ExitCode::SUCCESS)
  end
  sealmap_main->>sealmap_corpus: sealmap::generate(&extraction.codebase, &opts.corpus)
  sealmap_main->>sealmap_model__codebase___tCodebase: ~stats()
  alt Cmd::Generate(_)
    sealmap_main->>sealmap_corpus__contract: corpus::write(&c.out, &corpus)
    sealmap_main->>sealmap_corpus__index___tIndex: ~fragments()
  else Cmd::Verify(_)
    sealmap_main->>sealmap_corpus__contract: corpus::verify(&c.out, &corpus)
  end
```

## `sym:cargo sealmap_main . load().`
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
