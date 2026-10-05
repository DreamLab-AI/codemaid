//! CLI HOOK for `sealmap dense`, kept out of `src/main.rs` so this branch
//! merges cleanly with the seal-surface work editing the CLI. At the merge,
//! fold [`Dense`] into `main.rs`'s `Cmd` enum and move [`run`] behind it;
//! nothing else here is needed.
//!
//! ```text
//! cargo run --release -p sealmap --example dense -- [PATH] [-o DIR] [--depth N] [--stats]
//! ```
//!
//! Writes `_index.txt` and `dense.txt` (see `sealmap_dense`) into `DIR`
//! (default `.sealmap/dense`). `--stats` prints source, dense and call-edge
//! counts to stderr.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use sealmap::dense::{self, DenseOptions};

/// Write the dense agent projection of a Rust tree.
#[derive(Parser)]
#[command(name = "sealmap dense")]
struct Dense {
    /// Source root.
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Output directory.
    #[arg(short, long, default_value = ".sealmap/dense")]
    out: PathBuf,
    /// Codebase name (defaults to the directory name).
    #[arg(long)]
    name: Option<String>,
    /// Call levels expanded under a tree's root before a callee is cut.
    #[arg(long, default_value_t = DenseOptions::default().max_depth)]
    depth: usize,
    /// Include tests, examples and benches.
    #[arg(long)]
    tests: bool,
    /// Print sizes and call-edge counts to stderr.
    #[arg(long)]
    stats: bool,
}

fn main() -> ExitCode {
    match run(&Dense::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("sealmap dense: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Dense) -> Result<(), String> {
    let sources = sealmap::rust::load_dir(&args.path).map_err(|e| format!("{}: {e}", args.path.display()))?;
    let mut opts = sealmap::rust::RustOptions { include_tests: args.tests, ..Default::default() };
    opts.name = args.name.clone().unwrap_or_else(|| dir_name(&args.path));
    let extraction = sealmap::rust::extract(&sources, &opts);
    let cb = &extraction.codebase;
    let out = dense::render(cb, &DenseOptions::with_max_depth(args.depth));
    fs::create_dir_all(&args.out).map_err(|e| format!("{}: {e}", args.out.display()))?;
    for (name, text) in out.files() {
        let path = args.out.join(name);
        fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if args.stats {
        let source: usize = cb.files.keys().filter_map(|p| sources.get(p)).map(str::len).sum();
        let calls_at = out.text.find("\n# calls\n").map_or(out.text.len(), |i| i + 1);
        let stats = cb.stats();
        eprintln!(
            "source={source} dense={} index={} skeleton={} calls={} call_edges={} symbols={} ratio={:.3}",
            out.text.len(),
            out.index.len(),
            calls_at,
            out.text.len() - calls_at,
            stats.calls,
            stats.symbols,
            out.text.len() as f64 / source.max(1) as f64,
        );
    }
    Ok(())
}

fn dir_name(p: &Path) -> String {
    p.canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "codebase".into())
}
