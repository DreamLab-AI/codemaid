/// A tiny indentation-aware line writer, in the spirit of ts-morph's
/// `CodeBlockWriter` (MIT, © David Sherret), reduced to what Mermaid needs.
///
/// Guarantees: `\n` line endings, two-space indentation, no trailing
/// whitespace, exactly one trailing newline from [`CodeWriter::finish`].
///
/// ```
/// use codemaid_mermaid::CodeWriter;
///
/// let mut w = CodeWriter::new();
/// w.line("loop retry");
/// w.indented(|w| {
///     w.line("a->>b: ping   ");
/// });
/// w.line("end");
/// assert_eq!(w.finish(), "loop retry\n  a->>b: ping\nend\n");
/// ```
#[derive(Debug, Default, Clone)]
pub struct CodeWriter {
    buf: String,
    depth: usize,
}

impl CodeWriter {
    /// An empty writer at depth 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Write one line at the current depth. Trailing whitespace is trimmed;
    /// embedded newlines are the caller's bug and are replaced with spaces.
    pub fn line(&mut self, text: impl AsRef<str>) -> &mut Self {
        let text = text.as_ref().trim_end();
        if text.is_empty() {
            self.buf.push('\n');
            return self;
        }
        for _ in 0..self.depth {
            self.buf.push_str("  ");
        }
        if text.contains('\n') {
            self.buf.push_str(&text.replace(['\n', '\r'], " "));
        } else {
            self.buf.push_str(text);
        }
        self.buf.push('\n');
        self
    }

    /// Run `f` one level deeper.
    pub fn indented(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.depth += 1;
        f(self);
        self.depth -= 1;
        self
    }

    /// Current indentation depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Return the text, guaranteeing exactly one trailing newline.
    pub fn finish(mut self) -> String {
        while self.buf.ends_with("\n\n") {
            self.buf.pop();
        }
        if !self.buf.ends_with('\n') {
            self.buf.push('\n');
        }
        self.buf
    }
}
