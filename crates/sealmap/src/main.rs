//! `sealmap` command-line tool.
//!
//! ```text
//! sealmap generate [PATH] -o docs/sealmap       # write / update the corpus
//! sealmap verify   [PATH] -o docs/sealmap       # exit 1 on any drift (CI)
//! sealmap model    [PATH]                        # print the model as JSON
//! sealmap generate --repo api=../api --repo core=../core -o corpus
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use sealmap::corpus::{self, Drift, ExternalLanes};
use sealmap::rust::ExternalCalls;
use sealmap::{Options, SourceSet};

#[derive(Parser)]
#[command(name = "sealmap", version, about = "Deterministic codebase → Mermaid corpus generator")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate the corpus and write it into the output directory.
    Generate(Common),
    /// Check the output directory against the sources. Exits 1 on drift.
    Verify(Common),
    /// Print the extracted model as JSON to stdout.
    Model(Common),
}

#[derive(Args)]
struct Common {
    /// Source root (ignored when --repo is given).
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Add a repository as NAME=PATH (repeatable) to analyse several together.
    #[arg(long = "repo", value_name = "NAME=PATH")]
    repos: Vec<String>,
    /// Corpus directory.
    #[arg(short, long, default_value = ".sealmap")]
    out: PathBuf,
    /// Codebase name (defaults to the directory or repo names).
    #[arg(long)]
    name: Option<String>,
    /// Include tests, examples and benches.
    #[arg(long)]
    tests: bool,
    /// External calls to keep in flows: all, non-std, none.
    #[arg(long, default_value = "non-std")]
    external: String,
    /// One sequence lane per external owner instead of per crate.
    #[arg(long)]
    owner_lanes: bool,
    /// Minimum calls for a function to get a sequence diagram.
    #[arg(long, default_value_t = 1)]
    min_calls: usize,
    /// Leave private items out of structure diagrams.
    #[arg(long)]
    public_only: bool,
    /// Do not write _model.json.
    #[arg(long)]
    no_model: bool,
    /// Pretty-print _index.json and _model.json.
    #[arg(long)]
    pretty: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("sealmap: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    let (Cmd::Generate(c) | Cmd::Verify(c) | Cmd::Model(c)) = &cli.cmd;
    let mut opts = Options::default();
    if let Some(n) = &c.name {
        opts.rust.name = n.clone();
    }
    opts.rust.include_tests = c.tests;
    opts.rust.external_calls = match c.external.as_str() {
        "all" => ExternalCalls::All,
        "non-std" => ExternalCalls::NonStd,
        "none" => ExternalCalls::None,
        other => return Err(format!("--external must be all, non-std or none (got `{other}`)")),
    };
    opts.corpus.min_calls = c.min_calls;
    opts.corpus.include_private = !c.public_only;
    opts.corpus.emit_model = !c.no_model;
    opts.corpus.pretty_json = c.pretty;
    if c.owner_lanes {
        opts.corpus.external_lanes = ExternalLanes::Owner;
    }

    let (sources, rust_opts) = load(c, &opts)?;
    let extraction = sealmap::rust::extract(&sources, &rust_opts);
    for d in &extraction.diagnostics {
        eprintln!("warning: {}: {}", d.file, d.message);
    }

    if let Cmd::Model(_) = cli.cmd {
        let json = serde_json::to_string_pretty(&extraction.codebase).map_err(|e| e.to_string())?;
        println!("{json}");
        return Ok(ExitCode::SUCCESS);
    }

    let corpus = sealmap::generate(&extraction.codebase, &opts.corpus);
    let stats = extraction.codebase.stats();
    match cli.cmd {
        Cmd::Generate(_) => {
            let report = corpus::write(&c.out, &corpus).map_err(|e| e.to_string())?;
            eprintln!(
                "sealmap: {} files, {} symbols, {} sequences → {} ({} written, {} removed, {} unchanged)",
                stats.files,
                stats.symbols,
                corpus.index.fragments().filter(|f| f.kind == corpus::FragmentKind::Sequence).count(),
                c.out.display(),
                report.entries.len() - report.count(Drift::Orphaned),
                report.count(Drift::Orphaned),
                report.checked - (report.entries.len() - report.count(Drift::Orphaned)),
            );
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Verify(_) => {
            let report = corpus::verify(&c.out, &corpus).map_err(|e| e.to_string())?;
            for e in &report.entries {
                println!("{:<9} {}", e.drift, e.path);
            }
            if report.is_clean() {
                eprintln!("sealmap: corpus in {} is up to date ({} files)", c.out.display(), report.checked);
                Ok(ExitCode::SUCCESS)
            } else {
                eprintln!("sealmap: {} files drifted; run `sealmap generate`", report.entries.len());
                Ok(ExitCode::from(1))
            }
        }
        Cmd::Model(_) => unreachable!("handled above"),
    }
}

fn load(c: &Common, opts: &Options) -> Result<(SourceSet, sealmap::rust::RustOptions), String> {
    let mut ro = opts.rust.clone();
    if c.repos.is_empty() {
        // Load only: `run` extracts once. (Calling `extract_dir` here ran the
        // whole frontend twice and threw the first result away.)
        let sources = sealmap::rust::load_dir(&c.path).map_err(|e| format!("{}: {e}", c.path.display()))?;
        if ro.name == "codebase" {
            ro.name = dir_name(&c.path);
        }
        return Ok((sources, ro));
    }
    let mut repos = Vec::new();
    for r in &c.repos {
        let (name, path) = r.split_once('=').ok_or_else(|| format!("--repo expects NAME=PATH, got `{r}`"))?;
        repos.push((name.to_owned(), PathBuf::from(path)));
    }
    let refs: Vec<(&str, &Path)> = repos.iter().map(|(n, p)| (n.as_str(), p.as_path())).collect();
    let sources = sealmap::load_repos(&refs).map_err(|e| e.to_string())?;
    if ro.name == "codebase" {
        ro.name = repos.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join("+");
    }
    Ok((sources, ro))
}

fn dir_name(p: &Path) -> String {
    p.canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "codebase".into())
}
