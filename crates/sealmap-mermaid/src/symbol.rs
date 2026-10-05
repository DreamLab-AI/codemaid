//! Injective diagram ids for `sym:` symbol ids (feature `model`).

use std::fmt::Write as _;

use sealmap_model::{DescriptorKind, IdView, SymbolId};

use crate::escape::{Ident, is_reserved};

impl Ident {
    /// The diagram id of a symbol, derived from the structure of its
    /// [`SymbolId`] by an **injective** encoding: two different ids never
    /// share a diagram id, by construction rather than by a collision check.
    ///
    /// The id is a head followed by components, one per descriptor:
    ///
    /// | Part | Encoding |
    /// |---|---|
    /// | package `p` (global ids) | `p` |
    /// | path id, first segment `s` | `_s` |
    /// | unresolved method `m` | `__um` |
    /// | namespace `n` (module) | `__n` |
    /// | path segment `s` | `__s` |
    /// | type / method / term | `___tName` / `___fname` / `___vNAME` |
    /// | type parameter, parameter, meta, macro | `___p…`, `___a…`, `___k…`, `___x…` |
    /// | method disambiguator | `___d…` after its method |
    /// | manager other than `cargo`, release version | `___g…`, `___r…` after the package |
    ///
    /// A name is written as is when it is *plain*: ASCII letters and digits
    /// in runs joined by single `_`. Any other name is escaped: letters and
    /// digits stay, every other UTF-8 byte becomes `_` plus two lower-case
    /// hex digits, and the tag letter is upper-case (`___T`, `___F`, `___N`
    /// for a namespace, `___S` for a segment, `__P` / `__S` / `__U` for a
    /// head), so the reader knows which form follows. A package or first
    /// segment that is not plain or starts with a digit is escaped too.
    ///
    /// No name ends with `_`, so a run of underscores after a name always
    /// starts the next component: two mean an untagged namespace or segment,
    /// three a tag letter. Inside an escaped name `_` is always followed by a
    /// hex digit. The id therefore splits back into exactly one sequence of
    /// components; the tests decode every generated id to check this. An id
    /// that would equal a Mermaid keyword gets `___z`, which nothing else
    /// produces.
    ///
    /// ```
    /// use sealmap_mermaid::Ident;
    /// use sealmap_model::SymbolId;
    ///
    /// let id = |s: &str| Ident::from_symbol(&SymbolId::parse(s).unwrap());
    /// assert_eq!(id("sym:cargo shop . orders/Orders#place().").as_str(), "shop__orders___tOrders___fplace");
    /// assert_eq!(id("sym:cargo shop .").as_str(), "shop");
    /// assert_eq!(id("sym:extern tokio::fs").as_str(), "_tokio__fs");
    /// assert_eq!(id("sym:? insert").as_str(), "__uinsert");
    /// assert_eq!(id("sym:cargo a . T#[`From<u8>`]from().").as_str(), "a___tT___PFrom_3cu8_3e___ffrom");
    /// // The pairs `::` → `__` mangling merged stay apart.
    /// assert_ne!(id("sym:cargo a . b__c#"), id("sym:cargo a . b/c#"));
    /// assert_ne!(id("sym:cargo a . foo/"), id("sym:cargo a . foo()."));
    /// ```
    pub fn from_symbol(id: &SymbolId) -> Self {
        let mut out = String::with_capacity(id.as_str().len());
        match id.view() {
            IdView::Global(g) => {
                head(&mut out, "__P", "", &g.package);
                if g.manager != "cargo" {
                    tagged(&mut out, 'g', g.manager);
                }
                if let Some(v) = &g.version {
                    tagged(&mut out, 'r', v);
                }
                for d in &g.descriptors {
                    match d.kind {
                        DescriptorKind::Namespace => untagged(&mut out, 'n', &d.name),
                        DescriptorKind::Type => tagged(&mut out, 't', &d.name),
                        DescriptorKind::Term => tagged(&mut out, 'v', &d.name),
                        DescriptorKind::Method => {
                            tagged(&mut out, 'f', &d.name);
                            if let Some(dis) = d.disambiguator {
                                tagged(&mut out, 'd', dis);
                            }
                        }
                        DescriptorKind::TypeParameter => tagged(&mut out, 'p', &d.name),
                        DescriptorKind::Parameter => tagged(&mut out, 'a', &d.name),
                        DescriptorKind::Meta => tagged(&mut out, 'k', &d.name),
                        DescriptorKind::Macro => tagged(&mut out, 'x', &d.name),
                    }
                }
            }
            IdView::Path(segments) => {
                let (first, rest) = segments.split_first().map_or(("", &[][..]), |(f, r)| (f.as_ref(), r));
                head(&mut out, "__S", "_", first);
                for s in rest {
                    untagged(&mut out, 's', s);
                }
            }
            IdView::Unresolved(name) => {
                if is_plain(&name) {
                    out.push_str("__u");
                    out.push_str(&name);
                } else {
                    out.push_str("__U");
                    escape(&name, &mut out);
                }
            }
        }
        if is_reserved(&out) {
            out.push_str("___z");
        }
        Ident::from_valid(out)
    }
}

/// Letters and digits in runs joined by single `_`.
fn is_plain(s: &str) -> bool {
    !s.is_empty()
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && !s.starts_with('_')
        && !s.ends_with('_')
        && !s.contains("__")
}

fn escape(s: &str, out: &mut String) {
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() {
            out.push(b as char);
        } else {
            let _ = write!(out, "_{b:02x}");
        }
    }
}

/// The first name: `prefix` + the name when plain and starting with a
/// letter, otherwise `escaped_tag` + the escaped name.
fn head(out: &mut String, escaped_tag: &str, prefix: &str, name: &str) {
    if is_plain(name) && name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        out.push_str(prefix);
        out.push_str(name);
    } else {
        out.push_str(escaped_tag);
        escape(name, out);
    }
}

/// `___` + tag + plain name, or `___` + upper-case tag + escaped name.
fn tagged(out: &mut String, tag: char, name: &str) {
    out.push_str("___");
    if is_plain(name) {
        out.push(tag);
        out.push_str(name);
    } else {
        out.push(tag.to_ascii_uppercase());
        escape(name, out);
    }
}

/// `__` + plain name, or the escaped tagged form (`tag` upper-cased).
fn untagged(out: &mut String, tag: char, name: &str) {
    if is_plain(name) {
        out.push_str("__");
        out.push_str(name);
    } else {
        out.push_str("___");
        out.push(tag.to_ascii_uppercase());
        escape(name, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sealmap_model::{Descriptor, Package, Suffix, Version};

    /// The inverse of `from_symbol`, written independently from the
    /// encoding rules: if every generated id decodes back to its symbol, the
    /// encoding is injective.
    fn decode(text: &str) -> Option<SymbolId> {
        let text = match text.strip_suffix("___z") {
            Some(rest) if is_reserved(rest) => rest,
            _ => text,
        };
        let mut r = Reader { s: text.as_bytes(), pos: 0 };
        enum Head {
            Global(String),
            Path(String),
        }
        let head = if r.eat("__P") {
            Head::Global(r.escaped()?)
        } else if r.eat("__S") {
            Head::Path(r.escaped()?)
        } else if r.eat("__u") {
            let n = r.plain()?;
            return r.done().then(|| SymbolId::unresolved(n));
        } else if r.eat("__U") {
            let n = r.escaped()?;
            return r.done().then(|| SymbolId::unresolved(n));
        } else if r.eat("_") {
            Head::Path(r.plain()?)
        } else {
            Head::Global(r.plain()?)
        };
        let mut comps: Vec<(char, String)> = Vec::new();
        while !r.done() {
            if r.eat("___") {
                let tag = *r.s.get(r.pos)? as char;
                r.pos += 1;
                let name = if tag.is_ascii_uppercase() { r.escaped()? } else { r.plain()? };
                comps.push((tag.to_ascii_lowercase(), name));
            } else if r.eat("__") {
                comps.push(('_', r.plain()?));
            } else {
                return None;
            }
        }
        match head {
            Head::Path(first) => {
                let mut segs = vec![first];
                for (tag, name) in comps {
                    if tag != '_' && tag != 's' {
                        return None;
                    }
                    segs.push(name);
                }
                SymbolId::path(segs).ok()
            }
            Head::Global(name) => {
                let mut manager = "cargo".to_owned();
                let mut version = Version::Current;
                let mut ds: Vec<Descriptor> = Vec::new();
                for (tag, n) in comps {
                    match tag {
                        'g' if ds.is_empty() => manager = n,
                        'r' if ds.is_empty() => version = Version::Release(n),
                        '_' | 'n' => ds.push(Descriptor::namespace(n)),
                        't' => ds.push(Descriptor::r#type(n)),
                        'v' => ds.push(Descriptor::term(n)),
                        'f' => ds.push(Descriptor::method(n)),
                        'd' => {
                            let m = ds.pop()?;
                            ds.push(Descriptor::new(m.name(), Suffix::Method { disambiguator: Some(n) }).ok()?);
                        }
                        'p' => ds.push(Descriptor::type_parameter(n)),
                        'a' => ds.push(Descriptor::parameter(n)),
                        'k' => ds.push(Descriptor::meta(n)),
                        'x' => ds.push(Descriptor::r#macro(n)),
                        _ => return None,
                    }
                }
                Some(SymbolId::global(Package::new(manager, name, version).ok()?, ds))
            }
        }
    }

    struct Reader<'a> {
        s: &'a [u8],
        pos: usize,
    }

    impl Reader<'_> {
        fn eat(&mut self, lit: &str) -> bool {
            let ok = self.s[self.pos..].starts_with(lit.as_bytes());
            if ok {
                self.pos += lit.len();
            }
            ok
        }

        fn done(&self) -> bool {
            self.pos == self.s.len()
        }

        fn alnum(&self, at: usize) -> bool {
            self.s.get(at).is_some_and(u8::is_ascii_alphanumeric)
        }

        fn plain(&mut self) -> Option<String> {
            let start = self.pos;
            while self.alnum(self.pos) || (self.s.get(self.pos) == Some(&b'_') && self.alnum(self.pos + 1)) {
                self.pos += 1;
            }
            (self.pos > start).then(|| String::from_utf8_lossy(&self.s[start..self.pos]).into_owned())
        }

        fn escaped(&mut self) -> Option<String> {
            let mut bytes = Vec::new();
            loop {
                match self.s.get(self.pos) {
                    Some(b) if b.is_ascii_alphanumeric() => {
                        bytes.push(*b);
                        self.pos += 1;
                    }
                    Some(b'_') if self.s.get(self.pos + 1).is_some_and(u8::is_ascii_hexdigit) => {
                        let hex = std::str::from_utf8(self.s.get(self.pos + 1..self.pos + 3)?).ok()?;
                        bytes.push(u8::from_str_radix(hex, 16).ok()?);
                        self.pos += 3;
                    }
                    _ => return String::from_utf8(bytes).ok(),
                }
            }
        }
    }

    fn check(id: &SymbolId) {
        let m = Ident::from_symbol(id);
        let s = m.as_str();
        assert!(s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'), "{s}");
        assert!(!s.starts_with(|c: char| c.is_ascii_digit()), "{s}");
        assert!(!is_reserved(s), "{s}");
        assert_eq!(decode(s).as_ref(), Some(id), "{id} -> {s}");
    }

    #[test]
    fn decodes_back_on_edge_cases() {
        for text in [
            "sym:cargo a .",
            "sym:cargo end .",
            "sym:cargo Actor .",
            "sym:cargo 9x .",
            "sym:cargo _a .",
            "sym:cargo a_ .",
            "sym:cargo a__b .",
            "sym:npm @x/y 1.0.0 index/`POST /api`().",
            "sym:cargo a . b__c#",
            "sym:cargo a . b/c#",
            "sym:cargo a . _b/",
            "sym:cargo a . b_/",
            "sym:cargo a . foo/",
            "sym:cargo a . foo().",
            "sym:cargo a . foo(+1).",
            "sym:cargo a . T#[`From<u8>`]from().",
            "sym:cargo a . T#[From_u8_]from().",
            "sym:cargo a . T#[Store]put().",
            "sym:cargo a . impl#[Vec][Codec]encode().",
            "sym:cargo a . `r#type`#",
            "sym:cargo a . r_type#",
            "sym:cargo a . `é`/",
            "sym:cargo a . ``#",
            "sym:cargo a . f().(x)[T]m:k!v.",
            "sym:extern tokio",
            "sym:extern Tokio::fs",
            "sym:extern 9::x",
            "sym:extern a::b",
            "sym:extern `a::b`",
            "sym:extern `?`::x",
            "sym:? x",
            "sym:? ``",
            "sym:? `a b`",
        ] {
            check(&SymbolId::parse(text).unwrap_or_else(|e| panic!("{text}: {e}")));
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig { cases: 4096, ..Default::default() })]

        #[test]
        fn every_id_decodes_back(text in "sym:(cargo|npm) [a-c_9]{1,3} (\\.|1) [a-c_`/#.()\\[\\]!:+ é]{0,14}") {
            if let Ok(id) = SymbolId::parse(&text) {
                check(&id);
            }
        }

        #[test]
        fn every_path_and_unresolved_id_decodes_back(segs in proptest::collection::vec("[a-c_9 :?é]{0,4}", 1..4)) {
            check(&SymbolId::path(segs.clone()).unwrap());
            check(&SymbolId::unresolved(segs.join("")));
        }
    }
}
