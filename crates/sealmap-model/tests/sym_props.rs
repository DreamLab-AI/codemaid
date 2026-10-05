//! Property tests for the `sym:` grammar: printing is injective and parsing
//! is its exact inverse, including names that need escaping.

use proptest::prelude::*;
use sealmap_model::{Descriptor, Package, Suffix, SymbolId, Version};

/// Characters chosen to hit every escaping rule: grammar punctuation,
/// backticks, spaces, the simple set's edge members, control characters and
/// non-ASCII text.
fn tricky() -> impl Strategy<Value = String> {
    let ch = prop_oneof![
        Just('a'),
        Just('Z'),
        Just('0'),
        Just('_'),
        Just('+'),
        Just('-'),
        Just('$'),
        Just('`'),
        Just(' '),
        Just(':'),
        Just('/'),
        Just('#'),
        Just('.'),
        Just('('),
        Just(')'),
        Just('['),
        Just(']'),
        Just('!'),
        Just('<'),
        Just('?'),
        Just('\n'),
        Just('é'),
        Just('ß'),
    ];
    proptest::collection::vec(ch, 0..6).prop_map(|v| v.into_iter().collect())
}

fn field() -> impl Strategy<Value = String> {
    tricky().prop_filter_map("valid package field", |s| {
        let ok = !s.is_empty() && s != "." && !s.starts_with(' ') && !s.ends_with(' ') && !s.contains('\n');
        ok.then_some(s)
    })
}

fn package() -> impl Strategy<Value = Package> {
    let manager = prop_oneof![Just("cargo"), Just("npm"), Just("a-1"), Just("x")];
    let version = prop_oneof![Just(Version::Current), field().prop_map(Version::Release)];
    (manager, field(), version).prop_map(|(m, n, v)| Package::new(m, n, v).expect("generated fields are valid"))
}

fn descriptor() -> impl Strategy<Value = Descriptor> {
    let disambiguator = prop_oneof![Just(None), Just(Some("1".to_owned())), Just(Some("a+b$".to_owned()))];
    let suffix = prop_oneof![
        Just(Suffix::Namespace),
        Just(Suffix::Type),
        Just(Suffix::Term),
        disambiguator.prop_map(|disambiguator| Suffix::Method { disambiguator }),
        Just(Suffix::TypeParameter),
        Just(Suffix::Parameter),
        Just(Suffix::Meta),
        Just(Suffix::Macro),
    ];
    (tricky(), suffix).prop_map(|(n, s)| Descriptor::new(n, s).expect("generated disambiguators are valid"))
}

fn symbol_id() -> impl Strategy<Value = SymbolId> {
    prop_oneof![
        4 => (package(), proptest::collection::vec(descriptor(), 0..5)).prop_map(|(p, d)| SymbolId::global(p, d)),
        1 => proptest::collection::vec(tricky(), 1..4).prop_map(|s| SymbolId::path(s).expect("non-empty")),
        1 => tricky().prop_map(SymbolId::unresolved),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 4096, ..ProptestConfig::default() })]

    #[test]
    fn parse_inverts_print(id in symbol_id()) {
        let text = id.to_string();
        prop_assert_eq!(SymbolId::parse(&text), Ok(id), "text: {:?}", text);
    }

    #[test]
    fn print_is_injective(a in symbol_id(), b in symbol_id()) {
        if a != b {
            prop_assert_ne!(a.to_string(), b.to_string());
        }
    }

    #[test]
    fn parent_is_a_prefix(id in symbol_id()) {
        if let Some(p) = id.parent() {
            prop_assert!(p < id, "{} !< {}", p, id);
        }
    }
}

/// Pairs the old `::` → `__` mangling merged, and SCIP's own space escaping
/// would merge without the edge-space rule, stay distinct.
#[test]
fn historical_collisions_stay_distinct() {
    let pkg = |n: &str| Package::current("cargo", n).unwrap();
    let pairs = [
        (
            SymbolId::global(pkg("a"), vec![Descriptor::namespace("b__c")]),
            SymbolId::global(pkg("a"), vec![Descriptor::namespace("b"), Descriptor::namespace("c")]),
        ),
        (
            SymbolId::global(pkg("a"), vec![Descriptor::namespace("foo")]),
            SymbolId::global(pkg("a"), vec![Descriptor::method("foo")]),
        ),
        (
            SymbolId::global(pkg("a"), vec![Descriptor::r#type("T"), Descriptor::type_parameter("From<u8>")]),
            SymbolId::global(pkg("a"), vec![Descriptor::r#type("T"), Descriptor::type_parameter("From_u8_")]),
        ),
        (SymbolId::path(["a", "b"]).unwrap(), SymbolId::path(["a::b"]).unwrap()),
        (SymbolId::unresolved("x"), SymbolId::path(["?", "x"]).unwrap()),
    ];
    for (x, y) in pairs {
        assert_ne!(x.to_string(), y.to_string());
    }
    assert!(Package::current("cargo", "a ").is_err());
    assert!(Package::current("extern", "a").is_err());
}

proptest! {
    /// Any text the parser accepts is canonical: it prints back unchanged.
    #[test]
    fn accepted_text_is_canonical(text in "sym:(cargo|extern|\\?) [a-c` .:/#()\\[\\]!+]{0,12}") {
        if let Ok(id) = SymbolId::parse(&text) {
            prop_assert_eq!(id.to_string(), text);
        }
    }
}

/// The structural answer to `names()` and `parent()`, built from the parsed
/// parts rather than the text scanner.
fn structural(id: &SymbolId) -> (Vec<String>, Option<SymbolId>) {
    if let Some(p) = id.package() {
        let mut d = id.descriptors();
        let names = std::iter::once(p.name().to_owned()).chain(d.iter().map(|d| d.name().to_owned())).collect();
        let parent = d.pop().map(|_| {
            while d.last().is_some_and(|d| d.suffix() == &Suffix::TypeParameter) {
                d.pop();
            }
            SymbolId::global(p.clone(), d)
        });
        return (names, parent);
    }
    if let Some(s) = id.segments() {
        let parent = (s.len() > 1).then(|| SymbolId::path(s[..s.len() - 1].to_vec()).unwrap());
        return (s, parent);
    }
    (vec![id.name().into_owned()], None)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 4096, ..ProptestConfig::default() })]

    /// The borrowed text scanner behind `name`, `root`, `names`, `parent` and
    /// `child` agrees with the structured parse.
    #[test]
    fn scanner_agrees_with_structure(id in symbol_id(), d in descriptor()) {
        let (names, parent) = structural(&id);
        let got: Vec<String> = id.names().into_iter().map(|n| n.into_owned()).collect();
        prop_assert_eq!(&got, &names);
        prop_assert_eq!(id.name().into_owned(), names.last().cloned().unwrap_or_default());
        prop_assert_eq!(id.parent(), parent);
        if id.is_global() {
            prop_assert_eq!(id.root().map(|r| r.into_owned()), Some(names[0].clone()));
            let child = id.child(d.clone()).unwrap();
            let mut ds = id.descriptors();
            ds.push(d);
            prop_assert_eq!(child, SymbolId::global(id.package().unwrap(), ds));
        } else {
            prop_assert!(id.child(d).is_none());
        }
    }
}
