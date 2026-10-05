//! The 1:1 contract and determinism guarantees, end to end.

use std::collections::BTreeMap;
use std::fs;

use codemaid_corpus::{
    Corpus, CorpusOptions, Drift, FragmentKind, corpus_hash, generate, verify, verify_against, write,
};
use codemaid_model::{SourcePath, SourceSet};
use codemaid_rust::{RustOptions, extract};

fn corpus(files: &[(&str, &str)]) -> Corpus {
    let mut src = SourceSet::new();
    for (p, t) in files {
        src.insert(p, t).unwrap();
    }
    let cb = extract(&src, &RustOptions { name: "shop".into(), ..Default::default() }).codebase;
    generate(&cb, &CorpusOptions::default())
}

const SHOP: &[(&str, &str)] = &[
    ("Cargo.toml", "[package]\nname = \"shop\""),
    ("src/lib.rs", "//! Shop.\npub mod db;\npub mod orders;\npub use orders::Orders;"),
    (
        "src/db.rs",
        "/// Storage.\npub struct Db { rows: Vec<Row> }\npub struct Row { pub id: u64 }\nimpl Db {\n  pub fn insert(&mut self, id: u64) -> Result<(), String> { self.rows.push(Row { id }); Ok(()) }\n  pub fn exists(&self, id: u64) -> bool { self.rows.iter().any(|r| r.id == id) }\n}",
    ),
    (
        "src/orders.rs",
        "use crate::db::Db;\npub struct Orders { db: Db }\nimpl Orders {\n  pub fn place(&mut self, id: u64) -> Result<(), String> {\n    if self.db.exists(id) { return Err(\"dup\".into()); }\n    self.db.insert(id)?;\n    audit(id);\n    Ok(())\n  }\n}\nfn audit(_id: u64) {}",
    ),
];

#[test]
fn one_document_per_source_file_plus_reserved_files() {
    let c = corpus(SHOP);
    let docs: Vec<&str> = c.files.keys().map(SourcePath::as_str).collect();
    assert_eq!(
        docs,
        [
            "_README.md",
            "_index.json",
            "_model.json",
            "_overview.md",
            "src/db.rs.md",
            "src/lib.rs.md",
            "src/orders.rs.md"
        ]
    );
    assert_eq!(c.index.documents.len(), 3);
}

#[test]
fn documents_carry_front_matter_structure_and_sequences() {
    let c = corpus(SHOP);
    let doc = c.document("src/orders.rs.md").unwrap();
    assert!(doc.starts_with("---\ncodemaid: 1\nsource: src/orders.rs\nmodule: shop::orders\n"));
    assert!(doc.contains("## structure\n```mermaid\nclassDiagram"));
    assert!(doc.contains("shop__orders__Orders *-- shop__db__Db : db"));
    assert!(doc.contains("## `shop::orders::Orders::place`"));
    let seq = doc.split("## `shop::orders::Orders::place`").nth(1).unwrap();
    assert!(seq.contains("participant shop__orders__Orders as Orders"));
    assert!(seq.contains("shop__orders__Orders->>shop__db__Db: exists(id)"));
    assert!(seq.contains("Note over shop__orders__Orders: return Err(#quot;dup#quot;.into())"));
    assert!(seq.contains("shop__orders__Orders->>shop__db__Db: insert(id)?"));
    assert!(seq.contains("shop__orders__Orders->>shop__orders: audit(id)"));
}

#[test]
fn index_links_calls_to_expanding_fragments() {
    let c = corpus(SHOP);
    let place = c.index.fragment("shop::orders::Orders::place").unwrap();
    assert_eq!(place.kind, FragmentKind::Sequence);
    let by_target: BTreeMap<&str, Option<&str>> =
        place.calls.iter().map(|c| (c.target.as_str(), c.expands.as_deref())).collect();
    // `insert` has its own sequence (it calls `push`... which is std and dropped),
    // so only callees with flows expand.
    assert_eq!(by_target.get("shop::db::Db::exists"), Some(&None));
    assert!(by_target.contains_key("shop::orders::audit"));
    assert!(c.index.fragment("structure:src/db.rs").is_some());
}

#[test]
fn generation_is_byte_identical_across_runs_and_input_order() {
    let a = corpus(SHOP);
    let reversed: Vec<(&str, &str)> = SHOP.iter().rev().copied().collect();
    let b = corpus(&reversed);
    assert_eq!(a, b);
    assert_eq!(corpus_hash(&a), corpus_hash(&b));
    // CRLF sources produce the same corpus (hashes are newline-normalised).
    let crlf: Vec<(String, String)> = SHOP.iter().map(|(p, t)| (p.to_string(), t.replace('\n', "\r\n"))).collect();
    let crlf_refs: Vec<(&str, &str)> = crlf.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
    assert_eq!(corpus_hash(&a), corpus_hash(&corpus(&crlf_refs)));
}

#[test]
fn verify_classifies_every_kind_of_drift() {
    let fresh = corpus(SHOP);
    let mut disk: BTreeMap<SourcePath, String> = fresh.files.clone();
    let p = |s: &str| SourcePath::new(s).unwrap();

    // Clean.
    assert!(verify_against(&fresh, &disk).is_clean());

    // Hand edit → modified.
    disk.get_mut(&p("src/db.rs.md")).unwrap().push_str("\nnote\n");
    // Missing document.
    disk.remove(&p("src/lib.rs.md"));
    // Orphan: a generated document whose source is gone.
    disk.insert(p("src/gone.rs.md"), fresh.files[&p("src/db.rs.md")].clone());
    // Unrelated files are ignored.
    disk.insert(p("notes.md"), "# mine".into());

    // Stale: the source of orders.rs changed.
    let mut changed: Vec<(&str, &str)> = SHOP.to_vec();
    changed[3].1 = "pub struct Orders;";
    let newer = corpus(&changed);

    let r = verify_against(&newer, &disk);
    let got: Vec<(&str, Drift)> = r.entries.iter().map(|e| (e.path.as_str(), e.drift)).collect();
    assert!(got.contains(&("src/db.rs.md", Drift::Modified)), "{got:?}");
    assert!(got.contains(&("src/lib.rs.md", Drift::Missing)), "{got:?}");
    assert!(got.contains(&("src/gone.rs.md", Drift::Orphaned)), "{got:?}");
    assert!(got.contains(&("src/orders.rs.md", Drift::Stale)), "{got:?}");
    assert!(!got.iter().any(|(p, _)| *p == "notes.md"));
}

#[test]
fn write_converges_and_is_idempotent() {
    let dir = std::env::temp_dir().join(format!("codemaid-contract-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let c = corpus(SHOP);

    let first = write(&dir, &c).unwrap();
    assert_eq!(first.count(Drift::Missing), c.files.len());
    assert!(verify(&dir, &c).unwrap().is_clean());

    // Nothing to do the second time.
    assert!(write(&dir, &c).unwrap().is_clean());

    // An orphan is removed, a user file is left alone.
    fs::create_dir_all(dir.join("src/old")).unwrap();
    fs::write(dir.join("src/old/x.rs.md"), c.document("src/db.rs.md").unwrap()).unwrap();
    fs::write(dir.join("keep.md"), "mine").unwrap();
    let r = write(&dir, &c).unwrap();
    assert_eq!(r.count(Drift::Orphaned), 1);
    assert!(!dir.join("src/old").exists());
    assert!(dir.join("keep.md").exists());

    fs::remove_dir_all(&dir).unwrap();
}
