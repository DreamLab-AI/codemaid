//! Indented call trees.
//!
//! Every internal callable with a non-empty flow is *expanded* (its calls
//! listed beneath it) exactly once per output. A later call to it is written
//! as a reference (`^`), a call back into a callable still being expanded is
//! a cycle (`↺`), and a call reached beyond the depth limit is cut (`…`).
//! So the text grows with the number of call sites, never with the number of
//! paths through the graph.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use sealmap_model::{Call, CallKind, Codebase, Confidence, Step, SymbolId};

use crate::short::ShortNames;
use crate::skeleton::{one_line, order_key};

/// The call graph between internal symbols, read from the flows.
#[derive(Debug, Clone)]
pub(crate) struct Graph<'a> {
    /// Internal callables with at least one step, and their steps.
    pub(crate) flows: BTreeMap<&'a SymbolId, &'a [Step]>,
    /// Internal callees of each caller (self-calls included).
    pub(crate) callees: BTreeMap<&'a SymbolId, BTreeSet<&'a SymbolId>>,
    /// Internal callers of each callee, with the strongest confidence over
    /// all call sites (self-calls excluded).
    pub(crate) callers: BTreeMap<&'a SymbolId, BTreeMap<&'a SymbolId, Confidence>>,
}

impl<'a> Graph<'a> {
    pub(crate) fn new(cb: &'a Codebase) -> Self {
        let mut flows = BTreeMap::new();
        let mut callees: BTreeMap<&'a SymbolId, BTreeSet<&'a SymbolId>> = BTreeMap::new();
        let mut callers: BTreeMap<&'a SymbolId, BTreeMap<&'a SymbolId, Confidence>> = BTreeMap::new();
        for sym in cb.symbols.values() {
            let Some(flow) = sym.flow.as_ref().filter(|f| !f.is_empty()) else { continue };
            flows.insert(&sym.id, flow.steps.as_slice());
            for call in flow.calls() {
                let Some((target, _)) = cb.symbols.get_key_value(&call.target) else { continue };
                callees.entry(&sym.id).or_default().insert(target);
                if target != &sym.id {
                    let best = callers.entry(target).or_default().entry(&sym.id).or_insert(call.confidence);
                    *best = (*best).min(call.confidence);
                }
            }
        }
        Self { flows, callees, callers }
    }
}

/// Writes call trees into a string, remembering what it has expanded.
pub(crate) struct TreeWriter<'w, 'a> {
    cb: &'a Codebase,
    shorts: &'w ShortNames<'a>,
    graph: &'w Graph<'a>,
    pub(crate) out: String,
    max_depth: usize,
    expanded: BTreeSet<&'a SymbolId>,
    on_path: BTreeSet<&'a SymbolId>,
    /// `Some` in the full projection: cut callables queue up to become
    /// trees of their own. `None` in a slice: a cut is final.
    pending: Option<VecDeque<&'a SymbolId>>,
}

impl<'w, 'a> TreeWriter<'w, 'a> {
    pub(crate) fn new(
        cb: &'a Codebase,
        shorts: &'w ShortNames<'a>,
        graph: &'w Graph<'a>,
        max_depth: usize,
        requeue_cuts: bool,
    ) -> Self {
        Self {
            cb,
            shorts,
            graph,
            out: String::new(),
            max_depth,
            expanded: BTreeSet::new(),
            on_path: BTreeSet::new(),
            pending: requeue_cuts.then(VecDeque::new),
        }
    }

    pub(crate) fn is_expanded(&self, id: &SymbolId) -> bool {
        self.expanded.contains(id)
    }

    /// A tree rooted at `root`, its header followed by `mark` (`""` for an
    /// entry point or a slice seed, `" ↺"` for a callable only cycles
    /// reach); then, in the full projection, a tree headed `<name> …` for
    /// every callable it cut, in the order they were cut.
    pub(crate) fn root(&mut self, root: &'a SymbolId, mark: &str) {
        self.tree(root, mark);
        while let Some(next) = self.pending.as_mut().and_then(VecDeque::pop_front) {
            self.tree(next, " …");
        }
    }

    fn tree(&mut self, root: &'a SymbolId, mark: &str) {
        let Some(steps) = self.graph.flows.get(root).copied() else { return };
        if self.expanded.contains(root) {
            return;
        }
        self.out.push_str(self.shorts.get(root).unwrap_or("_"));
        self.out.push_str(mark);
        self.out.push('\n');
        self.expanded.insert(root);
        self.on_path.insert(root);
        self.steps(steps, 1, 1);
        self.on_path.remove(root);
    }

    fn steps(&mut self, steps: &'a [Step], indent: usize, depth: usize) {
        for step in steps {
            match step {
                Step::Call(call) => self.call(call, indent, depth),
                Step::Branch { arms } => {
                    for (i, arm) in arms.iter().enumerate() {
                        self.fragment(indent, if i == 0 { "alt" } else { "else" }, &arm.label);
                        self.steps(&arm.steps, indent + 1, depth);
                    }
                }
                Step::Parallel { arms } => {
                    for (i, arm) in arms.iter().enumerate() {
                        self.fragment(indent, if i == 0 { "par" } else { "and" }, &arm.label);
                        self.steps(&arm.steps, indent + 1, depth);
                    }
                }
                Step::Loop { label, body } => {
                    self.fragment(indent, "loop", label);
                    self.steps(body, indent + 1, depth);
                }
                Step::Optional { label, body } => {
                    self.fragment(indent, "opt", label);
                    self.steps(body, indent + 1, depth);
                }
                Step::Return(exit) => {
                    let label = one_line(&exit.label);
                    let keyword = ["return", "break", "continue"].iter().any(|k| label.starts_with(k));
                    self.fragment(indent, if keyword { "" } else { "return" }, &label);
                }
            }
        }
    }

    /// `<indent><keyword> <label>`; an `else` labelled `else` is written once.
    fn fragment(&mut self, indent: usize, keyword: &str, label: &str) {
        let label = one_line(label);
        indent_by(&mut self.out, indent);
        self.out.push_str(keyword);
        if !label.is_empty() && label != keyword {
            if !keyword.is_empty() {
                self.out.push(' ');
            }
            self.out.push_str(&label);
        }
        self.out.push('\n');
    }

    fn call(&mut self, call: &'a Call, indent: usize, depth: usize) {
        indent_by(&mut self.out, indent);
        call_text(&mut self.out, self.cb, self.shorts, call);
        let target = match self.cb.symbols.get_key_value(&call.target) {
            Some((id, _)) if self.graph.flows.contains_key(id) => id,
            _ => {
                self.out.push('\n');
                return;
            }
        };
        if self.on_path.contains(target) {
            self.out.push_str(" ↺\n");
        } else if self.expanded.contains(target) {
            self.out.push_str(" ^\n");
        } else if depth >= self.max_depth {
            self.out.push_str(" …\n");
            if let Some(queue) = self.pending.as_mut() {
                if !queue.contains(&target) {
                    queue.push_back(target);
                }
            }
        } else {
            self.out.push('\n');
            self.expanded.insert(target);
            self.on_path.insert(target);
            if let Some(steps) = self.graph.flows.get(target).copied() {
                self.steps(steps, indent + 1, depth + 1);
            }
            self.on_path.remove(target);
        }
    }
}

/// `<confidence><name><args><.await><?>`: `~` inferred, `?` external,
/// nothing for exact. Internal targets are named by short name, others by
/// their path as written.
pub(crate) fn call_text(out: &mut String, cb: &Codebase, shorts: &ShortNames<'_>, call: &Call) {
    match call.confidence {
        Confidence::Exact => {}
        Confidence::Inferred => out.push('~'),
        Confidence::External => out.push('?'),
    }
    let name = match shorts.get(&call.target) {
        Some(short) if cb.is_internal(&call.target) => short.to_owned(),
        _ => {
            let mut path = call.target.display_path();
            path.retain(|c| !c.is_whitespace());
            if call.kind == CallKind::Macro && !path.ends_with('!') {
                path.push('!');
            }
            path
        }
    };
    out.push_str(&name);
    let label = one_line(&call.label);
    if let Some(open) = label.find('(') {
        out.push_str(&label[open..]);
    }
    if call.awaited {
        out.push_str(".await");
    }
    if call.fallible {
        out.push('?');
    }
}

/// Walks callers upwards from a seed, as an indented reverse tree.
pub(crate) struct CallerWriter<'w, 'a> {
    cb: &'a Codebase,
    shorts: &'w ShortNames<'a>,
    graph: &'w Graph<'a>,
    max_depth: usize,
    listed: BTreeSet<&'a SymbolId>,
    path: Vec<&'a SymbolId>,
    pub(crate) out: String,
}

impl<'w, 'a> CallerWriter<'w, 'a> {
    pub(crate) fn new(cb: &'a Codebase, shorts: &'w ShortNames<'a>, graph: &'w Graph<'a>, max_depth: usize) -> Self {
        Self { cb, shorts, graph, max_depth, listed: BTreeSet::new(), path: Vec::new(), out: String::new() }
    }

    /// The callers of `seed` to `max_depth` levels; nothing if it has none
    /// or was already listed.
    pub(crate) fn root(&mut self, seed: &'a SymbolId) {
        if !self.graph.callers.contains_key(seed) || self.listed.contains(seed) {
            return;
        }
        self.out.push_str(self.shorts.get(seed).unwrap_or("_"));
        self.out.push('\n');
        self.listed.insert(seed);
        self.path.push(seed);
        self.level(seed, 1);
        self.path.pop();
    }

    fn level(&mut self, of: &'a SymbolId, depth: usize) {
        let Some(direct) = self.graph.callers.get(of) else { return };
        let mut direct: Vec<(&'a SymbolId, Confidence)> = direct.iter().map(|(id, c)| (*id, *c)).collect();
        direct.sort_by(|a, b| match (self.cb.symbol(a.0), self.cb.symbol(b.0)) {
            (Some(x), Some(y)) => order_key(x).cmp(&order_key(y)),
            _ => a.0.cmp(b.0),
        });
        for (caller, confidence) in direct {
            indent_by(&mut self.out, depth);
            match confidence {
                Confidence::Exact => {}
                Confidence::Inferred => self.out.push('~'),
                Confidence::External => self.out.push('?'),
            }
            self.out.push_str(self.shorts.get(caller).unwrap_or("_"));
            if self.path.contains(&caller) {
                self.out.push_str(" ↺\n");
            } else if !self.graph.callers.contains_key(caller) {
                self.out.push('\n');
            } else if self.listed.contains(caller) {
                self.out.push_str(" ^\n");
            } else if depth >= self.max_depth {
                self.out.push_str(" …\n");
            } else {
                self.out.push('\n');
                self.listed.insert(caller);
                self.path.push(caller);
                self.level(caller, depth + 1);
                self.path.pop();
            }
        }
    }
}

fn indent_by(out: &mut String, n: usize) {
    out.extend(std::iter::repeat_n(' ', n));
}
