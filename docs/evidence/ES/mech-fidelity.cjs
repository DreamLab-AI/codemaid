'use strict';
// Mechanical fidelity of arm R (supplements endpoint 3's blind judge; not a substitute).
// Per topic: (1) is all non-mermaid text byte-identical once extra headings added by the rewrite are
// removed? (2) citations (path:line or path:a-b) inside mermaid blocks: original set vs rewrite set.
// (3) diagram-kind counts before and after.
const fs = require('fs'), path = require('path');
const [origDir, newDir, outFile] = process.argv.slice(2);
const topics = [];
(function walk(d) { for (const e of fs.readdirSync(d, { withFileTypes: true })) {
  if (e.isDirectory() && !['tools', 'rendered'].includes(e.name)) walk(path.join(d, e.name));
  else if (e.isFile() && /^\d\d-.*\.md$/.test(e.name)) topics.push(path.relative(origDir, path.join(d, e.name))); } })(origDir);
function split(text) {
  const lines = text.split('\n'); const prose = []; const blocks = []; let cur = null; const heads = [];
  for (const l of lines) {
    if (cur) { if (l.startsWith('```')) { blocks.push(cur); cur = null; } else cur.src.push(l); continue; }
    if (l.startsWith('```mermaid')) { cur = { src: [] }; prose.push('<<MERMAID>>'); continue; }
    if (/^## [A-Z]+-\d+\.\d+/.test(l)) heads.push(l);
    prose.push(l);
  }
  return { prose, blocks: blocks.map((b) => ({ kind: (b.src[0] || '').trim().split(/\s/)[0], src: b.src.join('\n') })), heads };
}
const CITE = /([A-Za-z0-9_.\-/]*[A-Za-z0-9_\-]\.(?:rs|toml|md|ya?ml|json|cjs|mjs|js|ts|sh|lock|txt)):(\d+(?:-\d+)?)/g;
const cites = (blocks) => { const s = new Map(); for (const b of blocks) for (const m of b.src.matchAll(CITE)) { const k = `${m[1]}:${m[2]}`; s.set(k, (s.get(k) || 0) + 1); } return s; };
const bareLines = (blocks) => blocks.reduce((n, b) => n + (b.src.match(/(?<![\w/.]):\d+(?:-\d+)?\b/g) || []).length, 0);
const report = [];
for (const t of topics) {
  const o = split(fs.readFileSync(path.join(origDir, t), 'utf8')), n = split(fs.readFileSync(path.join(newDir, t), 'utf8'));
  const added = n.heads.filter((h) => !o.heads.includes(h)), missing = o.heads.filter((h) => !n.heads.includes(h));
  // Remove added sections (heading through the line before the next H2) from the new prose.
  const np = []; let skipping = false;
  for (const l of n.prose) { if (/^## /.test(l)) skipping = added.includes(l); if (!skipping) np.push(l); }
  const collapse = (p) => p.join('\n').replace(/(<<MERMAID>>\n?)+/g, '<<MERMAID>>\n');
  const proseSame = collapse(o.prose) === collapse(np);
  const co = cites(o.blocks), cn = cites(n.blocks);
  const dropped = [...co.keys()].filter((k) => !cn.has(k)), invented = [...cn.keys()].filter((k) => !co.has(k));
  const kinds = (bs) => bs.reduce((a, b) => (a[b.kind] = (a[b.kind] || 0) + 1, a), {});
  report.push({ topic: t, prose_verbatim: proseSame, headings_added: added.length, headings_missing: missing,
    kinds_before: kinds(o.blocks), kinds_after: kinds(n.blocks), cites_before: co.size, cites_after: cn.size,
    bare_line_refs_before: bareLines(o.blocks), bare_line_refs_after: bareLines(n.blocks),
    cites_dropped: dropped, cites_invented: invented });
}
const tot = { topics: report.length, prose_verbatim: report.filter((r) => r.prose_verbatim).length,
  non_sequence_after: report.reduce((s, r) => s + Object.entries(r.kinds_after).filter(([k]) => k !== 'sequenceDiagram').reduce((a, [, v]) => a + v, 0), 0),
  blocks_before: report.reduce((s, r) => s + Object.values(r.kinds_before).reduce((a, b) => a + b, 0), 0),
  blocks_after: report.reduce((s, r) => s + Object.values(r.kinds_after).reduce((a, b) => a + b, 0), 0),
  headings_added: report.reduce((s, r) => s + r.headings_added, 0),
  cites_dropped: report.reduce((s, r) => s + r.cites_dropped.length, 0), cites_invented: report.reduce((s, r) => s + r.cites_invented.length, 0),
  topics_with_dropped_cites: report.filter((r) => r.cites_dropped.length).length, topics_with_invented_cites: report.filter((r) => r.cites_invented.length).length };
fs.writeFileSync(outFile, JSON.stringify({ totals: tot, per_topic: report }, null, 2) + '\n');
console.log(JSON.stringify(tot, null, 1));
