use crate::escape::{Ident, escape_text};
use crate::writer::CodeWriter;

/// Message arrow styles for [`SequenceDiagram`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Arrow {
    /// `->>` solid line, filled head: a synchronous call.
    Sync,
    /// `-->>` dashed line, filled head: a reply / return value.
    Reply,
    /// `-)` solid line, open head: an asynchronous message (spawn, send).
    Async,
    /// `-x` solid line, cross head: a call that propagates failure.
    Fail,
}

impl Arrow {
    fn token(self) -> &'static str {
        match self {
            Self::Sync => "->>",
            Self::Reply => "-->>",
            Self::Async => "-)",
            Self::Fail => "-x",
        }
    }
}

/// Combined fragment kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlockKind {
    /// `loop`.
    Loop,
    /// `opt`.
    Opt,
    /// `alt` with `else` arms.
    Alt,
    /// `par` with `and` arms.
    Par,
    /// `critical` with `option` arms.
    Critical,
    /// `break`.
    Break,
}

impl BlockKind {
    fn open(self) -> &'static str {
        match self {
            Self::Loop => "loop",
            Self::Opt => "opt",
            Self::Alt => "alt",
            Self::Par => "par",
            Self::Critical => "critical",
            Self::Break => "break",
        }
    }
    fn next_arm(self) -> &'static str {
        match self {
            Self::Alt => "else",
            Self::Par => "and",
            Self::Critical => "option",
            // Single-arm kinds: extra arms are rendered as `else`, which only
            // happens if a caller misuses `block`; keeps output parseable.
            _ => "else",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Item {
    Message { from: Ident, to: Ident, arrow: Arrow, text: String },
    Note { over: Vec<Ident>, text: String },
    Block { kind: BlockKind, arms: Vec<(String, Vec<Item>)> },
}

/// Statement builder shared by the diagram root and nested fragments.
///
/// Obtained from [`SequenceDiagram`] (which derefs to it) or inside a block
/// closure.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeqBuilder {
    items: Vec<Item>,
}

impl SeqBuilder {
    /// `from ->> to: text` (or another arrow).
    pub fn message(&mut self, from: &Ident, to: &Ident, arrow: Arrow, text: &str) -> &mut Self {
        self.items.push(Item::Message { from: from.clone(), to: to.clone(), arrow, text: escape_text(text) });
        self
    }

    /// `Note over a,b: text` (one or two participants; `right of` when only
    /// one is given would be ambiguous for merging, so `over` is always used).
    pub fn note(&mut self, over: &[&Ident], text: &str) -> &mut Self {
        self.items
            .push(Item::Note { over: over.iter().take(2).map(|i| (*i).clone()).collect(), text: escape_text(text) });
        self
    }

    /// A single-arm fragment (`loop`, `opt`, `break`).
    pub fn block(&mut self, kind: BlockKind, label: &str, f: impl FnOnce(&mut SeqBuilder)) -> &mut Self {
        let mut inner = SeqBuilder::default();
        f(&mut inner);
        self.items.push(Item::Block { kind, arms: vec![(escape_text(label), inner.items)] });
        self
    }

    /// Shorthand for `block(BlockKind::Loop, ..)`.
    pub fn loop_block(&mut self, label: &str, f: impl FnOnce(&mut SeqBuilder)) -> &mut Self {
        self.block(BlockKind::Loop, label, f)
    }

    /// Shorthand for `block(BlockKind::Opt, ..)`.
    pub fn opt_block(&mut self, label: &str, f: impl FnOnce(&mut SeqBuilder)) -> &mut Self {
        self.block(BlockKind::Opt, label, f)
    }

    /// A multi-arm fragment (`alt`/`else`, `par`/`and`, `critical`/`option`).
    /// Each arm is `(label, builder)`. Empty `arms` emits nothing.
    ///
    /// ```
    /// use sealmap_mermaid::{Arrow, BlockKind, Ident, SequenceDiagram};
    ///
    /// let (a, b) = (Ident::new("a"), Ident::new("b"));
    /// let mut seq = SequenceDiagram::new();
    /// seq.arms(BlockKind::Alt, vec![
    ///     ("Ok(v)".into(), Box::new(|s: &mut sealmap_mermaid::SeqBuilder| { s.message(&a, &b, Arrow::Sync, "use(v)"); })),
    ///     ("Err(e)".into(), Box::new(|s: &mut sealmap_mermaid::SeqBuilder| { s.message(&a, &b, Arrow::Fail, "log(e)"); })),
    /// ]);
    /// assert!(seq.render().contains("  alt Ok(v)\n    a->>b: use(v)\n  else Err(e)\n    a-xb: log(e)\n  end\n"));
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn arms(&mut self, kind: BlockKind, arms: Vec<(String, Box<dyn FnOnce(&mut SeqBuilder) + '_>)>) -> &mut Self {
        if arms.is_empty() {
            return self;
        }
        let arms = arms
            .into_iter()
            .map(|(label, f)| {
                let mut inner = SeqBuilder::default();
                f(&mut inner);
                (escape_text(&label), inner.items)
            })
            .collect();
        self.items.push(Item::Block { kind, arms });
        self
    }

    /// Append the statements of another builder (used when composing
    /// fragments built separately).
    pub fn extend(&mut self, other: SeqBuilder) -> &mut Self {
        self.items.extend(other.items);
        self
    }

    /// `true` if no statements were added.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Number of message arrows, recursively.
    pub fn message_count(&self) -> usize {
        fn count(items: &[Item]) -> usize {
            items
                .iter()
                .map(|i| match i {
                    Item::Message { .. } => 1,
                    Item::Note { .. } => 0,
                    Item::Block { arms, .. } => arms.iter().map(|(_, a)| count(a)).sum(),
                })
                .sum()
        }
        count(&self.items)
    }

    fn write(items: &[Item], w: &mut CodeWriter) {
        for item in items {
            match item {
                Item::Message { from, to, arrow, text } => {
                    w.line(format!("{from}{}{to}: {text}", arrow.token()));
                }
                Item::Note { over, text } => {
                    let over: Vec<&str> = over.iter().map(Ident::as_str).collect();
                    w.line(format!("Note over {}: {text}", over.join(",")));
                }
                Item::Block { kind, arms } => {
                    for (i, (label, body)) in arms.iter().enumerate() {
                        let kw = if i == 0 { kind.open() } else { kind.next_arm() };
                        w.line(format!("{kw} {label}"));
                        w.indented(|w| Self::write(body, w));
                    }
                    w.line("end");
                }
            }
        }
    }
}

/// A Mermaid `sequenceDiagram`.
///
/// Participants are declared in the order you add them (that order is the
/// left-to-right lane order). Messages to an undeclared participant are
/// legal Mermaid; declaring them gives you a stable lane order and a short
/// alias.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SequenceDiagram {
    title: Option<String>,
    autonumber: bool,
    participants: Vec<(Ident, String, bool)>,
    body: SeqBuilder,
}

impl SequenceDiagram {
    /// An empty diagram.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the diagram title.
    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = Some(escape_text(title));
        self
    }

    /// Number messages (`autonumber`).
    pub fn autonumber(&mut self, on: bool) -> &mut Self {
        self.autonumber = on;
        self
    }

    /// Declare a participant lane with a display alias. Re-declaring an id is
    /// a no-op (the first alias wins).
    pub fn participant(&mut self, id: Ident, alias: &str) -> &mut Self {
        self.declare(id, alias, false)
    }

    /// Declare an actor lane (stick figure), e.g. for an external caller.
    pub fn actor(&mut self, id: Ident, alias: &str) -> &mut Self {
        self.declare(id, alias, true)
    }

    fn declare(&mut self, id: Ident, alias: &str, actor: bool) -> &mut Self {
        if !self.participants.iter().any(|(p, _, _)| *p == id) {
            self.participants.push((id, escape_text(alias), actor));
        }
        self
    }

    /// Declared participant ids, in lane order.
    pub fn participants(&self) -> impl Iterator<Item = &Ident> {
        self.participants.iter().map(|(id, _, _)| id)
    }

    /// Render to Mermaid text.
    pub fn render(&self) -> String {
        let mut w = CodeWriter::new();
        w.line("sequenceDiagram");
        w.indented(|w| {
            if let Some(t) = &self.title {
                w.line(format!("title {t}"));
            }
            if self.autonumber {
                w.line("autonumber");
            }
            for (id, alias, actor) in &self.participants {
                let kw = if *actor { "actor" } else { "participant" };
                if alias.is_empty() || alias == id.as_str() {
                    w.line(format!("{kw} {id}"));
                } else {
                    w.line(format!("{kw} {id} as {alias}"));
                }
            }
            SeqBuilder::write(&self.body.items, w);
        });
        w.finish()
    }
}

impl std::ops::Deref for SequenceDiagram {
    type Target = SeqBuilder;
    fn deref(&self) -> &SeqBuilder {
        &self.body
    }
}

impl std::ops::DerefMut for SequenceDiagram {
    fn deref_mut(&mut self) -> &mut SeqBuilder {
        &mut self.body
    }
}
