'use strict';
// Builds EK arms S, M, P from the register-stripped pack (arm A = that pack, unchanged).
// Removal unit (PREREG): a diagram's `## <ID>.<n>` heading line, its ```mermaid block, and its
// "**What it shows" / "**Why it is this way" paragraphs (each a run of non-blank lines), each
// span taking one following blank line with it. Everything else is kept.
const fs = require('fs');
const crypto = require('crypto');
const SEED = 20261006;
function mulberry32(a) { return () => { a |= 0; a = (a + 0x6D2B79F5) | 0; let t = Math.imul(a ^ (a >>> 15), 1 | a); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }; }

function units(lines) {
  const out = [];
  let cur = null;
  const isBoundary = (l) => l.startsWith('=== FILE:') || l.startsWith('# ') || l.startsWith('## ');
  for (let i = 0; i < lines.length; i++) {
    if (isBoundary(lines[i])) {
      if (cur) out.push(cur);
      cur = /^## [A-Z]+-\d+\.\d+/.test(lines[i]) ? { id: lines[i].slice(3).split(' ')[0], start: i, end: null } : null;
    }
    if (cur) cur.end = i;
  }
  if (cur) out.push(cur);
  return out.map((u) => {
    const del = new Set();
    const takeBlank = (j) => { if (j < lines.length && lines[j].trim() === '') del.add(j); };
    del.add(u.start); takeBlank(u.start + 1);
    let kind = null;
    for (let i = u.start + 1; i <= u.end; i++) {
      if (lines[i].startsWith('```mermaid')) {
        kind = (lines[i + 1] || '').trim().split(/\s/)[0];
        let j = i; while (j <= u.end && !(j > i && lines[j].startsWith('```'))) { del.add(j); j++; }
        del.add(j); takeBlank(j + 1); i = j;
      } else if (/^\*\*(What it shows|Why it is this way)/.test(lines[i]) && (i === 0 || lines[i - 1].trim() === '')) {
        let j = i; while (j <= u.end && lines[j].trim() !== '') { del.add(j); j++; }
        takeBlank(j); i = j;
      }
    }
    const bytes = [...del].reduce((s, j) => s + Buffer.byteLength(lines[j]) + 1, 0);
    return { id: u.id, kind, del, bytes };
  });
}

function apply(lines, chosen) {
  const del = new Set(); for (const u of chosen) for (const j of u.del) del.add(j);
  return lines.filter((_, j) => !del.has(j)).join('\n');
}

const [packPath, outDir] = process.argv.slice(2);
const pack = fs.readFileSync(packPath, 'utf8');
const lines = pack.split('\n');
const all = units(lines);
if (all.some((u) => !u.kind)) throw new Error('unit without a mermaid block');
const nonSeq = all.filter((u) => u.kind !== 'sequenceDiagram');
const target = nonSeq.reduce((s, u) => s + u.bytes, 0);
// M: Fisher-Yates shuffle of all units with mulberry32(SEED); take in order until >= target,
// then drop the last if that lands closer (so |M - S| < one diagram unit).
const rnd = mulberry32(SEED); const order = all.slice();
for (let i = order.length - 1; i > 0; i--) { const j = Math.floor(rnd() * (i + 1)); [order[i], order[j]] = [order[j], order[i]]; }
const m = []; let mb = 0;
for (const u of order) { m.push(u); mb += u.bytes; if (mb >= target) break; }
const last = m.at(-1);
if (mb - target > target - (mb - last.bytes)) { m.pop(); mb -= last.bytes; }
const arms = { A: pack, S: apply(lines, nonSeq), M: apply(lines, m), P: apply(lines, all) };
const kinds = (xs) => xs.reduce((a, u) => (a[u.kind] = (a[u.kind] || 0) + 1, a), {});
const report = { seed: SEED, units: all.length, kinds_all: kinds(all), S_removed_units: nonSeq.length, S_removed_bytes: target,
  M_removed_units: m.length, M_removed_bytes: mb, M_minus_S_bytes: mb - target, M_kinds: kinds(m), M_ids: m.map((u) => u.id).sort(),
  P_removed_units: all.length, arms: {} };
fs.mkdirSync(outDir, { recursive: true });
for (const [k, v] of Object.entries(arms)) {
  fs.writeFileSync(`${outDir}/pack-${k}.txt`, v);
  report.arms[k] = { bytes: Buffer.byteLength(v), removed_bytes: Buffer.byteLength(pack) - Buffer.byteLength(v), mermaid_blocks: (v.match(/^```mermaid/mg) || []).length,
    sha256: crypto.createHash('sha256').update(v).digest('hex') };
}
fs.writeFileSync(`${outDir}/arms.json`, JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ ...report, M_ids: undefined }, null, 1));
