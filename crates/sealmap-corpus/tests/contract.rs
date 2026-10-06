//! The 1:1 contract and determinism guarantees, end to end.

use std::collections::BTreeMap;
use std::fs;

use sealmap_corpus::{
    Corpus, CorpusOptions, Drift, FragmentKind, corpus_hash, generate, verify, verify_against, write,
};
use sealmap_mermaid::Ident;
use sealmap_model::{SourcePath, SourceSet, SymbolId};
use sealmap_rust::{RustOptions, extract};

/// The diagram id of a canonical `sym:` id.
fn mid(sym: &str) -> String {
    let id = SymbolId::parse(sym).unwrap_or_else(|e| panic!("{sym}: {e}"));
    Ident::from_symbol(&id).as_str().to_owned()
}

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
    assert!(doc.starts_with("---\nsealmap: 2\nsource: src/orders.rs\nmodule: \"sym:cargo shop . orders/\"\n"));
    assert!(doc.contains("## structure\n```mermaid\nclassDiagram"));
    let (orders, db, module) =
        (mid("sym:cargo shop . orders/Orders#"), mid("sym:cargo shop . db/Db#"), mid("sym:cargo shop . orders/"));
    assert!(doc.contains(&format!("{orders} *-- {db} : db")));
    assert!(doc.contains("## `sym:cargo shop . orders/Orders#place().`"));
    let seq = doc.split("## `sym:cargo shop . orders/Orders#place().`").nth(1).unwrap();
    assert!(seq.contains(&format!("participant {orders} as Orders")));
    assert!(seq.contains(&format!("{orders}->>{db}: exists(id)")));
    assert!(seq.contains(&format!("Note over {orders}: return Err(#quot;dup#quot;.into())")));
    assert!(seq.contains(&format!("{orders}->>{db}: insert(id)?")));
    assert!(seq.contains(&format!("{orders}->>{module}: audit(id)")));
}

#[test]
fn index_links_calls_to_expanding_fragments() {
    let c = corpus(SHOP);
    let place = c.index.fragment("sym:cargo shop . orders/Orders#place().").unwrap();
    assert_eq!(place.kind, FragmentKind::Sequence);
    let by_target: BTreeMap<String, Option<&str>> =
        place.calls.iter().map(|c| (c.target.to_string(), c.expands.as_deref())).collect();
    // `insert` has its own sequence (it calls `push`... which is std and dropped),
    // so only callees with flows expand.
    assert_eq!(by_target.get("sym:cargo shop . db/Db#exists()."), Some(&None));
    assert!(by_target.contains_key("sym:cargo shop . orders/audit()."));
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
    let dir = std::env::temp_dir().join(format!("sealmap-contract-{}", std::process::id()));
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

/// `a::b__c` and `a::b::c` used to share the diagram id `a__b__c`, so a call
/// from one to the other rendered as a self-message on a single lane.
#[test]
fn colliding_paths_get_distinct_diagram_ids() {
    let mut src = SourceSet::new();
    src.insert("Cargo.toml", "[package]\nname = \"a\"").unwrap();
    src.insert("src/lib.rs", "pub mod b;\npub struct b__c;\nimpl b__c { pub fn go(&self) { b::c::run(); } }\n")
        .unwrap();
    src.insert("src/b.rs", "pub struct c;\nimpl c { pub fn run() {} }\n").unwrap();
    let cb = extract(&src, &RustOptions::default()).codebase;
    let doc = generate(&cb, &CorpusOptions::default()).document("src/lib.rs.md").unwrap().to_owned();
    let seq = &doc[doc.find("sequenceDiagram").expect("a sequence diagram")..];
    let seq = &seq[..seq.find("```").unwrap()];
    let lanes: std::collections::BTreeSet<&str> = seq
        .lines()
        .filter_map(|l| l.trim().strip_prefix("participant "))
        .map(|l| l.split_whitespace().next().unwrap())
        .collect();
    assert_eq!(lanes.len(), 2, "{seq}");
    let (outer, inner) = (mid("sym:cargo a . b__c#"), mid("sym:cargo a . b/c#"));
    assert_ne!(outer, inner);
    assert!(seq.contains(&format!("{outer}->>{inner}")), "{seq}");
}

/// Schema v2: every symbol in `_model.json` carries its `sym:` id, span and
/// both fingerprints; the index carries them per fragment; the JSON reads
/// back through the version check.
#[test]
fn schema_v2_json_carries_ids_spans_and_fingerprints() {
    let c = corpus(SHOP);
    let model: serde_json::Value = serde_json::from_str(c.document("_model.json").unwrap()).unwrap();
    assert_eq!(model["schema_version"], sealmap_model::MODEL_SCHEMA_VERSION);
    let symbols = model["symbols"].as_object().unwrap();
    assert!(!symbols.is_empty());
    for (id, s) in symbols {
        assert!(id.starts_with("sym:cargo shop . ") || id == "sym:cargo shop .", "{id}");
        assert_eq!(s["id"], id.as_str());
        assert!(s["span"]["start_line"].as_u64().unwrap() >= 1, "{id}");
        for h in ["sig_hash", "body_hash"] {
            let f = s[h].as_str().unwrap();
            assert!(f.starts_with("blake3-16:") && f.len() == 42 && !f.ends_with(&"0".repeat(32)), "{id} {h}");
        }
    }
    let index: serde_json::Value = serde_json::from_str(c.document("_index.json").unwrap()).unwrap();
    assert_eq!(index["schema_version"], sealmap_corpus::CORPUS_SCHEMA_VERSION);
    let place = c.index.fragment("sym:cargo shop . orders/Orders#place().").unwrap();
    let sym = &symbols["sym:cargo shop . orders/Orders#place()."];
    assert_eq!(place.sig_hash.to_string(), sym["sig_hash"].as_str().unwrap());
    assert_eq!(place.body_hash.to_string(), sym["body_hash"].as_str().unwrap());
    let back = sealmap_model::Codebase::from_json(c.document("_model.json").unwrap()).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap() + "\n", c.document("_model.json").unwrap());
}

/// An external call through a qualified path is labelled once with its
/// owner (`Hasher::new_derive_key`), not twice (`Hasher::Hasher::…`), which
/// the self-corpus showed after the `sym:` ids landed.
#[test]
fn external_path_calls_are_not_double_qualified() {
    let c = corpus(&[("src/lib.rs", "pub fn f() { blake3::Hasher::new_derive_key(\"c\"); }")]);
    let doc = c.document("src/lib.rs.md").unwrap();
    assert!(doc.contains(": Hasher::new_derive_key("), "{doc}");
    assert!(!doc.contains("Hasher::Hasher::"), "{doc}");
}

/// Review finding `critical:F-06` claimed that a call dispatched through a
/// trait never links to an expanding fragment. Calls on a concrete type
/// resolve to the impl method (`Type#[Trait]m().`) and calls to a trait's
/// default method resolve to its own symbol; both have bodies and so both
/// expand. Only a call through `dyn Trait` to a *required* method names the
/// trait's declaration, which has no body to expand.
#[test]
fn trait_calls_link_to_impl_and_default_bodies() {
    let c = corpus(&[
        ("Cargo.toml", "[package]\nname = \"shop\""),
        (
            "src/lib.rs",
            "pub trait Job { fn run(&self); fn dflt(&self) { helper(); } }\n\
             pub struct Worker;\n\
             impl Job for Worker { fn run(&self) { helper(); } }\n\
             fn helper() {}\n\
             pub fn concrete(w: Worker) { w.run(); w.dflt(); }\n\
             pub fn dynamic(j: &dyn Job) { j.run(); }",
        ),
    ]);
    let expands = |frag: &str| -> BTreeMap<String, Option<String>> {
        c.index
            .fragment(frag)
            .unwrap_or_else(|| panic!("no fragment {frag}"))
            .calls
            .iter()
            .map(|c| (c.target.to_string(), c.expands.clone()))
            .collect()
    };
    let concrete = expands("sym:cargo shop . concrete().");
    let imp = "sym:cargo shop . Worker#[Job]run().";
    let dflt = "sym:cargo shop . Job#dflt().";
    assert_eq!(concrete.get(imp), Some(&Some(imp.to_owned())));
    assert_eq!(concrete.get(dflt), Some(&Some(dflt.to_owned())));
    // Dynamic dispatch has no single callee: the declaration is the target and does not expand.
    let dynamic = expands("sym:cargo shop . dynamic().");
    assert_eq!(dynamic.get("sym:cargo shop . Job#run()."), Some(&None));
}

/// Review finding `premortem:F-01` claimed `write` purges any Markdown file
/// that begins `sealmap: `. Only a file whose *front matter* opens with that
/// key is generated; authored files that mention it are left alone.
#[test]
fn write_leaves_authored_markdown_that_mentions_the_marker() {
    let dir = std::env::temp_dir().join(format!("sealmap-authored-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let c = corpus(SHOP);
    write(&dir, &c).unwrap();
    let authored = [
        ("plain.md", "sealmap: my notes\n"),
        ("other-fm.md", "---\ntitle: x\nsealmap: y\n---\nbody\n"),
        ("late.md", "# Heading\n---\nsealmap: z\n"),
    ];
    for (name, text) in authored {
        fs::write(dir.join(name), text).unwrap();
    }
    write(&dir, &c).unwrap();
    for (name, text) in authored {
        assert_eq!(fs::read_to_string(dir.join(name)).unwrap(), text, "{name} was touched");
    }
    fs::remove_dir_all(&dir).unwrap();
}

/// Review finding ER `gen X-12` / `var X-03` / `prefix X-03`: `read_rec` does
/// not follow symlinks, so a link at an expected path read as `Missing`, and
/// `write` then wrote through it, outside the output directory. A link to a
/// directory let `write` create documents under the link's target.
#[cfg(unix)]
#[test]
fn write_never_writes_through_a_symlink() {
    use std::os::unix::fs::symlink;
    let root = std::env::temp_dir().join(format!("sealmap-symlink-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let (dir, outside) = (root.join("out"), root.join("outside"));
    fs::create_dir_all(&dir).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let c = corpus(SHOP);

    // A link at an expected file path.
    fs::write(outside.join("victim.txt"), "precious").unwrap();
    symlink(outside.join("victim.txt"), dir.join("_README.md")).unwrap();
    let err = write(&dir, &c).unwrap_err();
    assert!(err.to_string().contains("_README.md"), "{err}");
    assert_eq!(fs::read_to_string(outside.join("victim.txt")).unwrap(), "precious");
    // Refused before anything was written.
    assert!(!dir.join("src").exists());

    // A link to a directory on the way to an expected path.
    fs::remove_file(dir.join("_README.md")).unwrap();
    symlink(&outside, dir.join("src")).unwrap();
    let err = write(&dir, &c).unwrap_err();
    assert!(err.to_string().contains("src"), "{err}");
    let mut leaked: Vec<_> = fs::read_dir(&outside).unwrap().map(|e| e.unwrap().file_name()).collect();
    leaked.sort();
    assert_eq!(leaked, ["victim.txt"]);

    // With the link gone, `write` converges as usual.
    fs::remove_file(dir.join("src")).unwrap();
    write(&dir, &c).unwrap();
    assert!(verify(&dir, &c).unwrap().is_clean());
    fs::remove_dir_all(&root).unwrap();
}
