//! The `sealmap` binary end to end on a scratch repository: sign, verify,
//! seal-check, resolve, stale (with and without `--since`) and
//! `generate --check`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ACCOUNTS: &str = "pub struct LedgerAccount { pub balance: i64 }\n\nimpl LedgerAccount {\n    pub fn deposit(&mut self, amount: i64) {\n        self.balance += amount;\n    }\n\n    pub fn withdraw(&mut self, amount: i64) -> bool {\n        self.balance -= amount;\n        self.balance >= 0\n    }\n}\n";

const TOPIC: &str = "---\nid: LED-01\ntitle: Accounts\narea: ledger\n---\nMoney goes in through `sym:cargo ledger . accounts/LedgerAccount#deposit().` and out\nthrough `sym:cargo ledger . accounts/LedgerAccount#withdraw().`.\n";

const DEPOSIT: &str = "sym:cargo ledger . accounts/LedgerAccount#deposit().";

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("docs/diagrams/ledger")).unwrap();
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"ledger\"\n").unwrap();
    fs::write(dir.join("src/lib.rs"), "pub mod accounts;\n").unwrap();
    fs::write(dir.join("src/accounts.rs"), ACCOUNTS).unwrap();
    fs::write(dir.join("docs/diagrams/ledger/01-accounts.md"), TOPIC).unwrap();
    dir
}

fn sealmap(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sealmap")).current_dir(dir).args(args).output().unwrap()
}

fn code(o: &Output) -> i32 {
    o.status.code().unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn edit(dir: &Path, file: &str, from: &str, to: &str) {
    let p = dir.join(file);
    let t = fs::read_to_string(&p).unwrap();
    assert!(t.contains(from), "{from}");
    fs::write(p, t.replace(from, to)).unwrap();
}

fn sign(dir: &Path) {
    let o = sealmap(
        dir,
        &["seal", "sign", "LED-01", "--reviewer", "zai:glm-5.3", "--model", "claude:sonnet", "--date", "2026-10-05"],
    );
    assert_eq!(code(&o), 0, "{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn sign_then_verify_goes_red_after_a_body_edit() {
    let dir = scratch("sign-verify");
    // Unsealed citations fail the gate before signing.
    assert_eq!(code(&sealmap(&dir, &["verify"])), 1);
    sign(&dir);
    let lock = fs::read_to_string(dir.join("docs/diagrams/seals.lock")).unwrap();
    assert!(lock.contains("id = \"LED-01\"\nfile = \"ledger/01-accounts.md\""), "{lock}");
    assert!(lock.contains("reviewer = \"zai:glm-5.3\""));
    let topic = fs::read_to_string(dir.join("docs/diagrams/ledger/01-accounts.md")).unwrap();
    assert!(topic.contains("area: ledger\nsealed: seals.lock\n---"), "{topic}");

    assert_eq!(code(&sealmap(&dir, &["verify"])), 0);
    assert_eq!(code(&sealmap(&dir, &["seal-check", "LED-01"])), 0);
    assert_eq!(code(&sealmap(&dir, &["seal-check", "LED-09"])), 1);
    // Signing again is idempotent.
    sign(&dir);
    assert_eq!(fs::read_to_string(dir.join("docs/diagrams/seals.lock")).unwrap(), lock);

    edit(&dir, "src/accounts.rs", "self.balance += amount;", "self.balance += amount * 2;");
    let o = sealmap(&dir, &["verify"]);
    assert_eq!(code(&o), 1);
    assert!(stdout(&o).contains(&format!("behaviour         LED-01   {DEPOSIT}")), "{}", stdout(&o));
    let o = sealmap(&dir, &["verify", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(v["findings"].as_array().unwrap().iter().any(|f| f["class"] == "behaviour" && f["symbol"] == DEPOSIT));

    // stale reports it and still exits 0.
    let o = sealmap(&dir, &["stale"]);
    assert_eq!(code(&o), 0);
    assert!(stdout(&o).contains(&format!("LED-01   behaviour  {DEPOSIT}")), "{}", stdout(&o));

    // A lock that is not canonical is a fault even when every seal holds.
    edit(&dir, "src/accounts.rs", "self.balance += amount * 2;", "self.balance += amount;");
    assert_eq!(code(&sealmap(&dir, &["verify"])), 0);
    fs::write(dir.join("docs/diagrams/seals.lock"), format!("{lock}\n")).unwrap();
    let o = sealmap(&dir, &["verify"]);
    assert_eq!(code(&o), 1);
    assert!(stdout(&o).contains("lock-fault"), "{}", stdout(&o));
}

#[test]
fn resolve_prints_span_and_hashes_or_absent() {
    let dir = scratch("resolve");
    let o = sealmap(&dir, &["resolve", DEPOSIT]);
    assert_eq!(code(&o), 0);
    assert!(stdout(&o).starts_with("found src/accounts.rs:4-6\nsig   blake3-16:"), "{}", stdout(&o));
    let o = sealmap(&dir, &["resolve", "--json", DEPOSIT]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["resolution"]["status"], "found");
    assert_eq!(v["resolution"]["span"]["start_line"], 4);

    // After a rename, the lock's sealed body names the candidate.
    sign(&dir);
    edit(&dir, "src/accounts.rs", "fn deposit(", "fn credit(");
    let o = sealmap(&dir, &["resolve", DEPOSIT]);
    assert_eq!(code(&o), 1);
    assert_eq!(stdout(&o), "absent\ncandidate sym:cargo ledger . accounts/LedgerAccount#credit().\n");
    assert_eq!(code(&sealmap(&dir, &["resolve", "not an id"])), 2);
}

#[test]
fn stale_since_a_git_revision_compares_two_trees() {
    if Command::new("git").arg("--version").output().is_err() {
        eprintln!("git not available; skipping");
        return;
    }
    let dir = scratch("stale-since");
    sign(&dir);
    // Change deposit, then commit: that change predates the revision.
    edit(&dir, "src/accounts.rs", "self.balance += amount;", "self.balance += amount + 0;");
    let git = |args: &[&str]| {
        let o = Command::new("git")
            .current_dir(&dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "base"]);
    // Then change withdraw's signature in the working tree.
    edit(&dir, "src/accounts.rs", "-> bool {", "-> i64 {");
    edit(&dir, "src/accounts.rs", "self.balance >= 0", "self.balance");

    let since = stdout(&sealmap(&dir, &["stale", "--since", "HEAD"]));
    assert!(since.contains("contract   sym:cargo ledger . accounts/LedgerAccount#withdraw()."), "{since}");
    assert!(!since.contains(DEPOSIT), "{since}");
    let against_lock = stdout(&sealmap(&dir, &["stale"]));
    assert!(against_lock.contains(DEPOSIT) && against_lock.contains("withdraw"), "{against_lock}");
    // The temporary tree is cleaned up and the checkout is untouched.
    assert_eq!(code(&sealmap(&dir, &["stale", "--since", "no-such-rev"])), 2);
}

#[test]
fn generate_check_compares_with_a_fresh_generation() {
    let dir = scratch("generate-check");
    assert_eq!(code(&sealmap(&dir, &["generate", "--check"])), 1);
    assert!(!dir.join(".sealmap").exists(), "--check writes nothing");
    assert_eq!(code(&sealmap(&dir, &["generate"])), 0);
    assert!(dir.join(".sealmap/src/accounts.rs.md").exists());
    assert_eq!(code(&sealmap(&dir, &["generate", "--check"])), 0);
    edit(&dir, "src/accounts.rs", "self.balance += amount;", "self.balance += amount * 2;");
    let o = sealmap(&dir, &["generate", "--check"]);
    assert_eq!(code(&o), 1);
    assert!(stdout(&o).contains("stale     src/accounts.rs.md"), "{}", stdout(&o));
}
