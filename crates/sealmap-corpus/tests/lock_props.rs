//! The lock writer is the format: any lock it writes parses back to the
//! same value and re-serialises to the same bytes.

use proptest::prelude::*;
use sealmap_corpus::seal::{Lock, SymbolSeal, TopicSeal};
use sealmap_model::{Descriptor, Fingerprint, Package, SourcePath, SymbolId};

fn fingerprint() -> impl Strategy<Value = Fingerprint> {
    any::<[u8; 16]>().prop_map(Fingerprint::from_bytes)
}

fn symbol() -> impl Strategy<Value = SymbolId> {
    // Names outside the simple set are quoted (with doubled backticks), so
    // arbitrary text exercises the id's own escaping inside TOML strings.
    ("[a-z][a-z0-9_-]{0,8}", proptest::collection::vec("\\PC{1,6}", 1..4)).prop_map(|(pkg, names)| {
        let package = Package::current("cargo", pkg).unwrap();
        let mut ds: Vec<Descriptor> =
            names[..names.len() - 1].iter().map(|n| Descriptor::namespace(n.clone())).collect();
        ds.push(Descriptor::method(names[names.len() - 1].clone()));
        SymbolId::global(package, ds)
    })
}

fn topic(n: usize) -> impl Strategy<Value = TopicSeal> {
    (
        "[A-Z]{2,4}-[0-9]{2}[^\\p{Cc}]{0,4}",
        fingerprint(),
        ".{0,12}",
        "(?s).{0,12}",
        proptest::collection::btree_map(symbol(), (fingerprint(), fingerprint()), 0..4),
    )
        .prop_filter("ids have no edge whitespace", |(id, ..)| id.trim() == id)
        .prop_map(move |(id, topic_hash, reviewer, model, symbols)| TopicSeal {
            id,
            file: SourcePath::new(format!("area/{n:02}-topic.md")).unwrap(),
            topic_hash,
            reviewer,
            model,
            date: "2026-10-05".into(),
            symbols: symbols.into_iter().map(|(k, (sig, body))| (k, SymbolSeal { sig, body })).collect(),
        })
}

proptest! {
    #[test]
    fn canonical_round_trip(topics in (topic(1), topic(2), topic(3)), generator in "[^\\p{Cc}]{0,16}") {
        let mut lock = Lock::new();
        lock.generator = generator;
        for t in [topics.0, topics.1, topics.2] {
            lock.insert(t);
        }
        let text = lock.to_toml();
        let back = Lock::parse_canonical(&text).unwrap();
        prop_assert_eq!(&back, &lock);
        prop_assert_eq!(back.to_toml(), text);
    }
}
