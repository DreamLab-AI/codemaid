//! E0d's overlap rule (`docs/evidence/E0d/PREREG.md` "The overlap rule",
//! amendment #1): relocate each citation from its topic's stamp to P through
//! the zero-context diff between the two blobs, then flag the topic when a
//! P→C hunk lies within k lines of a relocated citation, a cited file is
//! deleted, or a citation is lost or ambiguous and its file changed.
//!
//! Nothing here parses a language: every input is a git blob and a `-U0` diff.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;

use serde::Serialize;

use crate::git::Repo;

/// The pre-registered primary k and its two sensitivity values.
pub const KS: [u32; 3] = [0, 5, 20];

/// One `-U0` hunk header: old lines `old_start..old_start+old_len` became
/// new lines `new_start..new_start+new_len`. `old_len == 0` is an insertion
/// after old line `old_start` (0 = before the first line).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hunk {
    pub old_start: u32,
    pub old_len: u32,
    pub new_start: u32,
    pub new_len: u32,
}

/// A parsed blob diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diff {
    /// Hunks in old-line order (empty when the blobs are equal in content).
    Text(Vec<Hunk>),
    /// git judged either side binary, so no line can be relocated or tested.
    Binary,
}

fn range(s: &str) -> Option<(u32, u32)> {
    let s = s.get(1..)?;
    match s.split_once(',') {
        Some((a, n)) => Some((a.parse().ok()?, n.parse().ok()?)),
        None => Some((s.parse().ok()?, 1)),
    }
}

/// Parse `git diff -U0` output for one pair of blobs.
pub fn parse_u0(out: &str) -> Diff {
    let mut hunks = Vec::new();
    for line in out.lines() {
        if line.starts_with("Binary files ") && line.ends_with(" differ") {
            return Diff::Binary;
        }
        let Some(rest) = line.strip_prefix("@@ ") else { continue };
        let mut parts = rest.split(' ');
        let (Some(old), Some(new)) = (parts.next().and_then(range), parts.next().and_then(range)) else { continue };
        hunks.push(Hunk { old_start: old.0, old_len: old.1, new_start: new.0, new_len: new.1 });
    }
    Diff::Text(hunks)
}

/// Where old line `l` is after the hunks, or `None` when the line is deleted
/// (inside a hunk's old range).
pub fn relocate_line(hunks: &[Hunk], l: u32) -> Option<u32> {
    let mut delta = 0i64;
    for h in hunks {
        if h.old_len > 0 {
            let end = h.old_start + h.old_len - 1;
            if (h.old_start..=end).contains(&l) {
                return None;
            }
            if end < l {
                delta += i64::from(h.new_len) - i64::from(h.old_len);
            }
        } else if h.old_start < l {
            delta += i64::from(h.new_len);
        }
    }
    u32::try_from(i64::from(l) + delta).ok()
}

/// A cited span `[a, b]` after the hunks, or `None` when its first or last line is deleted.
pub fn relocate_span(hunks: &[Hunk], a: u32, b: u32) -> Option<(u32, u32)> {
    Some((relocate_line(hunks, a)?, relocate_line(hunks, b)?))
}

/// `true` when a hunk changes an old line within `k` of `[a, b]`, or inserts
/// into a gap whose either neighbour is within `k` (amendment #1).
pub fn touches(hunks: &[Hunk], a: u32, b: u32, k: u32) -> bool {
    let (lo, hi) = (i64::from(a) - i64::from(k), i64::from(b) + i64::from(k));
    hunks.iter().any(|h| {
        let s = i64::from(h.old_start);
        if h.old_len > 0 { s <= hi && s + i64::from(h.old_len) > lo } else { s + 1 >= lo && s <= hi }
    })
}

/// Why a citation's relocation could not be computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Why {
    /// The topic has no usable stamp in this repository.
    NoStamp,
    /// The file does not exist at the stamp.
    AbsentAtStamp,
    /// Line 0, or past the file's last line at the stamp.
    LineOutOfRange,
    /// git treats a blob on the way as binary.
    Binary,
}

/// One citation at P.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The relocated span at P.
    At(u32, u32),
    /// A first or last line was deleted between the stamp and P, or the file is absent at P.
    Lost,
    /// No relocation could be computed.
    Ambiguous(Why),
}

/// Blob ids by revision and path, blob diffs and line counts, memoised.
pub struct Store {
    repo: Repo,
    trees: HashMap<String, Rc<HashMap<String, String>>>,
    diffs: HashMap<(String, String), Rc<Diff>>,
    lines: HashMap<String, Option<u32>>,
    /// `git diff` processes run.
    pub diff_calls: usize,
}

impl Store {
    pub fn new(repo: Repo) -> Self {
        Self { repo, trees: HashMap::new(), diffs: HashMap::new(), lines: HashMap::new(), diff_calls: 0 }
    }

    /// The blob id of `path` at `rev` (regular files and symlinks), or `None`.
    pub fn oid(&mut self, rev: &str, path: &str) -> Result<Option<String>, String> {
        if !self.trees.contains_key(rev) {
            let map: HashMap<String, String> = self
                .repo
                .ls_tree(rev)?
                .into_iter()
                .filter(|e| e.mode.starts_with("100") || e.mode == "120000")
                .map(|e| (e.path, e.oid))
                .collect();
            self.trees.insert(rev.to_string(), Rc::new(map));
        }
        Ok(self.trees[rev].get(path).cloned())
    }

    /// The `-U0` diff between two blobs.
    pub fn diff(&mut self, from: &str, to: &str) -> Result<Rc<Diff>, String> {
        let key = (from.to_string(), to.to_string());
        if let Some(d) = self.diffs.get(&key) {
            return Ok(d.clone());
        }
        self.diff_calls += 1;
        let d = Rc::new(parse_u0(&self.repo.diff_blobs_u0(from, to)?));
        self.diffs.insert(key, d.clone());
        Ok(d)
    }

    /// Lines in a blob, or `None` when git would call it binary (a NUL in the first 8,000 bytes).
    pub fn line_count(&mut self, oid: &str) -> Result<Option<u32>, String> {
        if let Some(n) = self.lines.get(oid) {
            return Ok(*n);
        }
        let blob = self.repo.read_blobs(&[oid.to_string()])?.pop().unwrap_or_default();
        let n = if blob[..blob.len().min(8000)].contains(&0) {
            None
        } else {
            let nl = blob.iter().filter(|b| **b == b'\n').count();
            let tail = usize::from(blob.last().is_some_and(|b| *b != b'\n'));
            Some(u32::try_from(nl + tail).unwrap_or(u32::MAX))
        };
        self.lines.insert(oid.to_string(), n);
        Ok(n)
    }

    /// Relocate one file's cited spans from `stamp` to `p`.
    pub fn relocate(
        &mut self,
        stamp: Option<&str>,
        p: &str,
        file: &str,
        spans: &[(u32, u32)],
    ) -> Result<Vec<Status>, String> {
        let all = |s: Status| Ok(vec![s; spans.len()]);
        let Some(stamp) = stamp else { return all(Status::Ambiguous(Why::NoStamp)) };
        let Some(os) = self.oid(stamp, file)? else { return all(Status::Ambiguous(Why::AbsentAtStamp)) };
        let Some(n) = self.line_count(&os)? else { return all(Status::Ambiguous(Why::Binary)) };
        let op = self.oid(p, file)?;
        let diff = match &op {
            Some(op) if *op != os => Some(self.diff(&os, op)?),
            _ => None,
        };
        Ok(spans
            .iter()
            .map(|&(a, b)| {
                if a == 0 || b > n {
                    return Status::Ambiguous(Why::LineOutOfRange);
                }
                match (&op, diff.as_deref()) {
                    (None, _) => Status::Lost,
                    (Some(_), None) => Status::At(a, b),
                    (Some(_), Some(Diff::Binary)) => Status::Ambiguous(Why::Binary),
                    (Some(_), Some(Diff::Text(h))) => match relocate_span(h, a, b) {
                        Some((x, y)) => Status::At(x, y),
                        None => Status::Lost,
                    },
                }
            })
            .collect())
    }
}

/// What one topic tracks in one repository for the overlap rule.
#[derive(Debug, Clone, Default)]
pub struct HTopic {
    pub topic: usize,
    pub stamp: Option<String>,
    /// Cited spans at the stamp, by cited file (repository-relative).
    pub cites: BTreeMap<String, Vec<(u32, u32)>>,
    /// `sources:` files no citation names (only the +uncited sensitivity row reads them).
    pub uncited: BTreeSet<String>,
}

/// One topic's overlap flags for one commit.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct HFlags {
    /// T_hunk(k) for k = 0, 5, 20 ([`KS`]).
    pub k: [bool; 3],
    /// Sensitivity: T_hunk(5) plus every citation lost or ambiguous at P, changed or not.
    pub literal_lost: bool,
    /// Sensitivity: T_hunk(5) plus every changed uncited source.
    pub uncited: bool,
    /// A cited file was present at P and absent at C.
    pub by_deleted: bool,
    /// A citation lost at P, in a file that changed.
    pub by_lost: bool,
    /// A citation ambiguous at P, in a file that changed.
    pub by_ambiguous: bool,
    /// A cited file changed with a binary blob at P or C.
    pub by_binary: bool,
    /// A hunk within 5 lines of a relocated citation.
    pub by_overlap5: bool,
}

/// Flag one topic for the commit P→C.
pub fn flag(store: &mut Store, t: &HTopic, p: &str, c: &str, changed: &BTreeSet<String>) -> Result<HFlags, String> {
    let mut f = HFlags::default();
    let mut lost_anywhere = false;
    let mut overlap = [false; 3];
    for (file, spans) in &t.cites {
        let st = store.relocate(t.stamp.as_deref(), p, file, spans)?;
        let bad_lost = st.contains(&Status::Lost);
        let bad_amb = st.iter().any(|s| matches!(s, Status::Ambiguous(_)));
        lost_anywhere |= bad_lost || bad_amb;
        if !changed.contains(file) {
            continue;
        }
        f.by_lost |= bad_lost;
        f.by_ambiguous |= bad_amb;
        let (op, oc) = (store.oid(p, file)?, store.oid(c, file)?);
        match (op, oc) {
            (Some(_), None) => f.by_deleted = true,
            (Some(op), Some(oc)) if op != oc => match &*store.diff(&op, &oc)? {
                Diff::Binary => f.by_binary = true,
                Diff::Text(h) => {
                    for s in &st {
                        if let Status::At(a, b) = *s {
                            for (i, k) in KS.iter().enumerate() {
                                overlap[i] |= touches(h, a, b, *k);
                            }
                        }
                    }
                }
            },
            // Absent at P: every citation is lost (set above). Same blob: no line changed.
            _ => {}
        }
    }
    let whole = f.by_deleted || f.by_lost || f.by_ambiguous || f.by_binary;
    for (flag, hit) in f.k.iter_mut().zip(overlap) {
        *flag = whole || hit;
    }
    f.by_overlap5 = overlap[1];
    f.literal_lost = f.k[1] || lost_anywhere;
    f.uncited = f.k[1] || t.uncited.iter().any(|s| changed.contains(s));
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::process::Command;

    fn h(os: u32, ol: u32, ns: u32, nl: u32) -> Hunk {
        Hunk { old_start: os, old_len: ol, new_start: ns, new_len: nl }
    }

    #[test]
    fn parses_headers_with_and_without_counts() {
        let out = "diff --git a/x b/y\nindex 1..2\n--- a/x\n+++ b/y\n@@ -3 +3 @@ fn x\n-a\n+b\n@@ -10,0 +11,2 @@\n+c\n+d\n@@ -20,2 +22,0 @@\n-e\n-f\n";
        assert_eq!(parse_u0(out), Diff::Text(vec![h(3, 1, 3, 1), h(10, 0, 11, 2), h(20, 2, 22, 0)]));
        assert_eq!(parse_u0("diff --git a/x b/x\nBinary files a/x and b/x differ\n"), Diff::Binary);
        assert_eq!(parse_u0(""), Diff::Text(vec![]));
    }

    #[test]
    fn relocation_moves_survivors_and_loses_deleted_lines() {
        // Insert 2 after line 10, delete 20-21, replace 3.
        let hs = [h(3, 1, 3, 1), h(10, 0, 11, 2), h(20, 2, 22, 0)];
        assert_eq!(relocate_line(&hs, 1), Some(1));
        assert_eq!(relocate_line(&hs, 3), None, "replaced line is deleted");
        assert_eq!(relocate_line(&hs, 10), Some(10), "insertion after 10 does not move 10");
        assert_eq!(relocate_line(&hs, 11), Some(13));
        assert_eq!(relocate_line(&hs, 20), None);
        assert_eq!(relocate_line(&hs, 22), Some(22), "+2 then −2");
        assert_eq!(relocate_span(&hs, 5, 12), Some((5, 14)));
        assert_eq!(relocate_span(&hs, 19, 21), None, "last line deleted");
        // Insertion at the top of the file moves everything.
        assert_eq!(relocate_line(&[h(0, 0, 1, 3)], 1), Some(4));
    }

    #[test]
    fn overlap_respects_k_and_insertion_gaps() {
        let replace = [h(10, 1, 10, 1)];
        assert!(touches(&replace, 10, 10, 0));
        assert!(!touches(&replace, 12, 12, 0));
        assert!(touches(&replace, 15, 15, 5) && !touches(&replace, 16, 16, 5));
        assert!(touches(&replace, 3, 5, 5) && !touches(&replace, 3, 4, 5));
        // Insertion after line 9 (gap 9|10): adjacent to line 10 at k = 0.
        let ins = [h(9, 0, 10, 1)];
        assert!(touches(&ins, 10, 10, 0));
        assert!(touches(&ins, 9, 9, 0));
        assert!(!touches(&ins, 11, 11, 0) && touches(&ins, 11, 11, 1));
        assert!(!touches(&ins, 3, 3, 5) && touches(&ins, 4, 4, 5));
        // Insertion at the top (gap 0|1).
        assert!(touches(&[h(0, 0, 1, 1)], 1, 1, 0));
    }

    /// A throwaway repository with commits, for the git-backed paths.
    struct Tmp {
        dir: std::path::PathBuf,
    }

    impl Tmp {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("e0d-hunk-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            let t = Tmp { dir };
            t.git(&["init", "-q"]);
            t
        }
        fn git(&self, args: &[&str]) -> String {
            let o = Command::new("git")
                .arg("-C")
                .arg(&self.dir)
                .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"])
                .args(args)
                .output()
                .unwrap();
            assert!(o.status.success(), "{args:?}: {}", String::from_utf8_lossy(&o.stderr));
            String::from_utf8(o.stdout).unwrap().trim().to_string()
        }
        fn commit(&self, files: &[(&str, Option<&str>)]) -> String {
            for (p, t) in files {
                let path = self.dir.join(p);
                match t {
                    Some(t) => std::fs::write(&path, t).unwrap(),
                    None => {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
            self.git(&["add", "-A"]);
            self.git(&["commit", "-q", "--allow-empty", "-m", "x"]);
            self.git(&["rev-parse", "HEAD"])
        }
        fn path(&self) -> &Path {
            &self.dir
        }
    }

    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn lines(n: u32) -> String {
        (1..=n).map(|i| format!("line {i}\n")).collect()
    }

    fn topic(stamp: &str, cites: &[(&str, u32, u32)], uncited: &[&str]) -> HTopic {
        let mut t = HTopic {
            topic: 0,
            stamp: Some(stamp.into()),
            uncited: uncited.iter().map(|s| s.to_string()).collect(),
            ..HTopic::default()
        };
        for (f, a, b) in cites {
            t.cites.entry(f.to_string()).or_default().push((*a, *b));
        }
        t
    }

    fn set(v: &[&str]) -> BTreeSet<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn end_to_end_over_a_real_history() {
        let r = Tmp::new("e2e");
        let base = lines(60);
        let s = r.commit(&[("a.txt", Some(&base)), ("b.txt", Some("x\n")), ("u.txt", Some("u\n"))]);
        // P: 3 lines inserted at the top of a.txt (pure movement relative to the stamp).
        let p_text = format!("new 1\nnew 2\nnew 3\n{base}");
        let p = r.commit(&[("a.txt", Some(&p_text))]);
        let mut store = Store::new(Repo::open(r.path()));
        let t = topic(&s, &[("a.txt", 30, 30)], &["u.txt"]);
        assert_eq!(store.relocate(Some(&s), &p, "a.txt", &[(30, 30)]).unwrap(), [Status::At(33, 33)]);

        // C1: edit line 40 at P (old line 37), 7 lines from the citation at 33.
        let c1 = r.commit(&[("a.txt", Some(&p_text.replace("line 37\n", "LINE 37\n")))]);
        let f = flag(&mut store, &t, &p, &c1, &set(&["a.txt"])).unwrap();
        assert_eq!(f.k, [false, false, true], "{f:?}");
        assert!(!f.literal_lost && !f.uncited);

        // C2 from P: edit P-line 36 (3 away) → k = 5 and 20.
        let c2 = r.commit(&[("a.txt", Some(&p_text.replace("line 33\n", "LINE 33\n")))]);
        let p2 = r.git(&["rev-parse", "HEAD~1"]);
        let f = flag(&mut store, &t, &p2, &c2, &set(&["a.txt"])).unwrap();
        // HEAD~1 is c1 (line 37 edited), so the citation at 33 survives; line 36 in c1 is "line 33".
        assert_eq!(f.k, [false, true, true], "{f:?}");
        assert!(f.by_overlap5);

        // A pure insertion far away only moves the citation: no flag.
        let far = format!("{p_text}tail\n");
        let c3 = r.commit(&[("a.txt", Some(&far))]);
        let f = flag(&mut store, &t, &p, &c3, &set(&["a.txt"])).unwrap();
        assert_eq!(f.k, [false, false, false]);

        // An uncited source changing sets only the +uncited row.
        let f = flag(&mut store, &t, &p, &p, &set(&["u.txt"])).unwrap();
        assert!(!f.k[1] && f.uncited);

        // Deleting the cited file flags every k.
        let c4 = r.commit(&[("a.txt", None)]);
        let p4 = r.git(&["rev-parse", "HEAD~1"]);
        let f = flag(&mut store, &t, &p4, &c4, &set(&["a.txt"])).unwrap();
        assert!(f.by_deleted && f.k == [true, true, true]);
    }

    #[test]
    fn lost_and_ambiguous_flag_on_change_and_literally_everywhere() {
        let r = Tmp::new("lost");
        let base = lines(20);
        let s = r.commit(&[("a.txt", Some(&base)), ("o.txt", Some("o\n"))]);
        // P deletes the cited line 10.
        let p = r.commit(&[("a.txt", Some(&base.replace("line 10\n", "")))]);
        let c_other = r.commit(&[("o.txt", Some("o2\n"))]);
        let mut store = Store::new(Repo::open(r.path()));
        let t = topic(&s, &[("a.txt", 10, 10)], &[]);
        // The commit touches only o.txt: the gated reading does not flag, the literal one does.
        let f = flag(&mut store, &t, &p, &c_other, &set(&["o.txt"])).unwrap();
        assert!(!f.k[1] && f.literal_lost, "{f:?}");
        // A far edit to a.txt flags through the lost citation.
        let pa = c_other.clone();
        let ca = r.commit(&[("a.txt", Some(&base.replace("line 10\n", "").replace("line 1\n", "LINE 1\n")))]);
        let f = flag(&mut store, &t, &pa, &ca, &set(&["a.txt"])).unwrap();
        assert!(f.by_lost && f.k == [true, true, true], "{f:?}");
        // Past the end of the file at the stamp, and no stamp: ambiguous.
        assert_eq!(
            store.relocate(Some(&s), &p, "a.txt", &[(25, 25)]).unwrap(),
            [Status::Ambiguous(Why::LineOutOfRange)]
        );
        assert_eq!(store.relocate(None, &p, "a.txt", &[(1, 1)]).unwrap(), [Status::Ambiguous(Why::NoStamp)]);
        assert_eq!(
            store.relocate(Some(&s), &p, "zzz.txt", &[(1, 1)]).unwrap(),
            [Status::Ambiguous(Why::AbsentAtStamp)]
        );
        // Backwards relocation (P before the stamp): a line the stamp added is lost at P.
        let later = r.commit(&[("a.txt", Some(&format!("added\n{base}")))]);
        assert_eq!(store.relocate(Some(&later), &s, "a.txt", &[(1, 1)]).unwrap(), [Status::Lost]);
        assert_eq!(store.relocate(Some(&later), &s, "a.txt", &[(5, 6)]).unwrap(), [Status::At(4, 5)]);
    }

    #[test]
    fn binary_changes_count_as_ambiguous() {
        let r = Tmp::new("bin");
        let s = r.commit(&[("b.bin", Some("a\0b\n"))]);
        let c = r.commit(&[("b.bin", Some("a\0c\n"))]);
        let mut store = Store::new(Repo::open(r.path()));
        let oid = store.oid(&s, "b.bin").unwrap().unwrap();
        assert_eq!(store.line_count(&oid).unwrap(), None);
        let t = topic(&s, &[("b.bin", 1, 1)], &[]);
        let f = flag(&mut store, &t, &s, &c, &set(&["b.bin"])).unwrap();
        assert!(f.by_ambiguous && f.k[0]);
    }
}
