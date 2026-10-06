'use strict';
// ES2 gate units. Parses topic files into H2 sections and their mermaid blocks; a unit is every
// sequenceDiagram block in NEW that is not byte-identical to ORIG's block under the same heading.
//   node units.cjs <origDiagrams> <newDiagrams>            -> JSON on stdout
// Also exported: topics(), sections(), for the restore and conversion steps.
const fs = require('fs'), path = require('path');
function topics(dir) { const out = [];
  (function walk(d) { for (const e of fs.readdirSync(d, { withFileTypes: true })) {
    if (e.isDirectory() && !['tools', 'rendered'].includes(e.name)) walk(path.join(d, e.name));
    else if (e.isFile() && /^\d\d-.*\.md$/.test(e.name)) out.push(path.relative(dir, path.join(d, e.name))); } })(dir);
  return out.sort(); }
// Returns [{head, id, start, end, blocks:[{kind, src, from, to}]}]; line indices into text.split('\n').
function sections(text) { const L = text.split('\n'); const secs = []; let cur = null, inB = null;
  L.forEach((l, i) => {
    if (inB) { if (l.startsWith('```')) { inB.to = i; inB.src = L.slice(inB.from + 1, i).join('\n'); inB.kind = (L[inB.from + 1] || '').trim().split(/\s/)[0]; cur.blocks.push(inB); inB = null; } return; }
    if (l.startsWith('```mermaid')) { inB = { from: i }; if (!cur) throw new Error('mermaid before heading'); return; }
    if (/^## /.test(l)) { if (cur) cur.end = i; const m = l.match(/^## ([A-Z]+-\d+\.\d+)/); cur = { head: l, id: m ? m[1] : null, start: i, blocks: [] }; secs.push(cur); }
  }); if (cur) cur.end = L.length; return secs; }
module.exports = { topics, sections };
if (require.main === module) {
  const [O, N] = process.argv.slice(2); const units = []; const conv = [];
  for (const t of topics(O)) {
    const os = sections(fs.readFileSync(path.join(O, t), 'utf8')), ns = sections(fs.readFileSync(path.join(N, t), 'utf8'));
    const oBy = Object.fromEntries(os.map((s) => [s.head, s]));
    let prevOrig = null;
    for (const s of ns) {
      const o = oBy[s.head]; if (o) prevOrig = o;
      s.blocks.forEach((b, k) => {
        const ob = o && o.blocks[k];
        if (ob && ob.src === b.src) return;              // untouched (incl. pre-existing sequence diagrams)
        units.push({ id: `${s.id}${s.blocks.length > 1 ? `#${k + 1}` : ''}`, topic: t, head: s.head, added: !o,
          parent: o ? s.head : prevOrig && prevOrig.head, orig_kind: (o ? ob : prevOrig && prevOrig.blocks[0])?.kind ?? null, new_kind: b.kind });
      });
    }
    for (const o of os) for (const b of o.blocks) if (b.kind !== 'sequenceDiagram') conv.push({ topic: t, head: o.head, kind: b.kind });
  }
  console.log(JSON.stringify({ units, original_non_sequence: conv }, null, 1));
}
