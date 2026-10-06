//! Read-only git access. Every read names an explicit revision; nothing here
//! touches a working tree, an index or a ref.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A repository opened for revision-pinned reads.
#[derive(Debug, Clone)]
pub struct Repo {
    dir: PathBuf,
}

/// One entry of `git ls-tree -r`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TreeEntry {
    pub path: String,
    pub mode: String,
    pub oid: String,
}

/// A first-parent history entry: the commit and all of its parents.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub sha: String,
    pub parents: Vec<String>,
}

impl Repo {
    pub fn open(dir: &Path) -> Self {
        Self { dir: dir.to_path_buf() }
    }

    fn git(&self, args: &[&str]) -> Command {
        let mut c = Command::new("git");
        c.arg("-C").arg(&self.dir).args(args).env("GIT_OPTIONAL_LOCKS", "0").env("LC_ALL", "C");
        c
    }

    fn run(&self, args: &[&str]) -> Result<Vec<u8>, String> {
        let out = self.git(args).stderr(Stdio::piped()).output().map_err(|e| format!("git {args:?}: {e}"))?;
        if !out.status.success() {
            return Err(format!("git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(out.stdout)
    }

    fn run_str(&self, args: &[&str]) -> Result<String, String> {
        String::from_utf8(self.run(args)?).map_err(|e| format!("git {args:?}: non-UTF-8 output: {e}"))
    }

    /// The full sha of `rev` as a commit, or `None` if it names no commit here.
    pub fn resolve_commit(&self, rev: &str) -> Option<String> {
        let spec = format!("{rev}^{{commit}}");
        self.run_str(&["rev-parse", "--verify", "--quiet", &spec]).ok().map(|s| s.trim().to_string())
    }

    /// First-parent history from `head` (newest first), with every parent of each commit.
    pub fn first_parent_history(&self, head: &str) -> Result<Vec<HistoryEntry>, String> {
        let out = self.run_str(&["rev-list", "--first-parent", "--parents", head])?;
        Ok(out
            .lines()
            .map(|l| {
                let mut it = l.split_whitespace().map(String::from);
                let sha = it.next().unwrap_or_default();
                HistoryEntry { sha, parents: it.collect() }
            })
            .collect())
    }

    /// Paths changed between `from` and `to` (no rename detection: a rename is a
    /// delete plus an add, so both paths count as changed).
    pub fn changed_paths(&self, from: &str, to: &str) -> Result<Vec<String>, String> {
        let out = self.run(&["diff-tree", "-r", "-z", "--no-renames", "--name-only", "--no-commit-id", from, to])?;
        let mut v: Vec<String> =
            out.split(|b| *b == 0).filter(|s| !s.is_empty()).map(|s| String::from_utf8_lossy(s).into_owned()).collect();
        v.sort();
        v.dedup();
        Ok(v)
    }

    /// Every entry of the tree at `rev`, recursively.
    pub fn ls_tree(&self, rev: &str) -> Result<Vec<TreeEntry>, String> {
        let out = self.run(&["ls-tree", "-r", "-z", "--full-tree", rev])?;
        let mut v = Vec::new();
        for rec in out.split(|b| *b == 0).filter(|s| !s.is_empty()) {
            let rec = String::from_utf8_lossy(rec);
            let Some((meta, path)) = rec.split_once('\t') else { continue };
            let mut m = meta.split(' ');
            let (Some(mode), Some(_kind), Some(oid)) = (m.next(), m.next(), m.next()) else { continue };
            v.push(TreeEntry { path: path.to_string(), mode: mode.to_string(), oid: oid.to_string() });
        }
        v.sort();
        Ok(v)
    }

    /// The bytes of `path` at `rev`, or `None` if the file does not exist there.
    pub fn show(&self, rev: &str, path: &str) -> Option<Vec<u8>> {
        self.run(&["show", &format!("{rev}:{path}")]).ok()
    }

    /// `true` if `path` exists at `rev`.
    pub fn exists(&self, rev: &str, path: &str) -> bool {
        self.git(&["cat-file", "-e", &format!("{rev}:{path}")])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    /// `true` if `ancestor` is an ancestor of (or equal to) `of`.
    pub fn is_ancestor(&self, ancestor: &str, of: &str) -> bool {
        self.git(&["merge-base", "--is-ancestor", ancestor, of])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    /// Committer date of `sha` as an ISO-8601 string.
    pub fn commit_date(&self, sha: &str) -> Result<String, String> {
        Ok(self.run_str(&["show", "-s", "--format=%cI", sha])?.trim().to_string())
    }

    /// Subject line of `sha`.
    pub fn subject(&self, sha: &str) -> Result<String, String> {
        Ok(self.run_str(&["show", "-s", "--format=%s", sha])?.trim().to_string())
    }

    /// Unified diff between `from` and `to`, restricted to `paths`.
    pub fn diff(&self, from: &str, to: &str, paths: &[String]) -> Result<String, String> {
        let mut args: Vec<String> = ["diff", "--no-color", "--no-renames", "--no-ext-diff", from, to, "--"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        args.extend(paths.iter().map(|p| format!(":(top,literal){p}")));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        Ok(String::from_utf8_lossy(&self.run(&refs)?).into_owned())
    }

    /// Zero-context diff between two blobs, with every option that moves a hunk
    /// pinned on the command line (E0d amendment #1), so no user or repository
    /// configuration can change the result.
    pub fn diff_blobs_u0(&self, from: &str, to: &str) -> Result<String, String> {
        let out = self.run(&[
            "diff",
            "-U0",
            "--no-color",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--diff-algorithm=myers",
            "--indent-heuristic",
            from,
            to,
        ])?;
        Ok(String::from_utf8_lossy(&out).into_owned())
    }

    /// `git --version`, trimmed.
    pub fn git_version() -> String {
        Command::new("git")
            .arg("--version")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    }

    /// Read many blobs by object id through one `git cat-file --batch`.
    pub fn read_blobs(&self, oids: &[String]) -> Result<Vec<Vec<u8>>, String> {
        let mut child = self
            .git(&["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("git cat-file: {e}"))?;
        let mut stdin = child.stdin.take().ok_or("git cat-file: no stdin")?;
        let input: String = oids.iter().map(|o| format!("{o}\n")).collect();
        let writer = std::thread::spawn(move || {
            let r = stdin.write_all(input.as_bytes());
            drop(stdin);
            r
        });
        let mut out = BufReader::new(child.stdout.take().ok_or("git cat-file: no stdout")?);
        let mut blobs = Vec::with_capacity(oids.len());
        for oid in oids {
            let mut header = String::new();
            out.read_line(&mut header).map_err(|e| format!("git cat-file: {e}"))?;
            let parts: Vec<&str> = header.split_whitespace().collect();
            if parts.len() != 3 {
                return Err(format!("git cat-file: object {oid}: {}", header.trim()));
            }
            let size: usize = parts[2].parse().map_err(|_| format!("git cat-file: bad size in `{}`", header.trim()))?;
            let mut buf = vec![0u8; size + 1];
            out.read_exact(&mut buf).map_err(|e| format!("git cat-file: {e}"))?;
            buf.pop();
            blobs.push(buf);
        }
        writer.join().map_err(|_| "git cat-file: writer panicked")?.map_err(|e| format!("git cat-file: {e}"))?;
        child.wait().map_err(|e| format!("git cat-file: {e}"))?;
        Ok(blobs)
    }
}
