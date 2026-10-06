#!/usr/bin/env node
'use strict';
// ER pooling: reads er/dedup.json ({case: [{members: ["gen-A-gemini:critical:F-01", ...]}]}),
// shuffles each case's items with mulberry32(20261006), relabels X-01.., writes
//   er/adjudication/<case>/blind.md   (no origins)  and  er/adjudication/<case>/key.json (origins).
const fs = require('node:fs');
const path = require('node:path');
const E = __dirname; // ER-glm copy of er/pool.cjs, unchanged logic

function mulberry32(a) {
  return function () {
    a |= 0; a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
function shuffle(arr, rnd) {
  const a = arr.slice();
  for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(rnd() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; }
  return a;
}

// Amendment #E12: tolerant field extraction from the raw review (accepts `- Evidence:` and
// `- **Evidence**:`, multi-line bodies with code blocks). Topics and "Marked by authors" are
// deliberately not carried into the blind text (topic ids would reveal the arm).
function blocks(md) {
  const re = /^### (F-\d+)\s*[—–-]\s*(.+)$/gm;
  const heads = [...md.matchAll(re)];
  return heads.map((m, i) => {
    const end = i + 1 < heads.length ? heads[i + 1].index : md.length;
    const body = md.slice(m.index + m[0].length, end).split(/^#{1,2} |^---\s*$/m)[0];
    const fieldRe = /^- \*{0,2}(Topics|Evidence|Failure|Chain of events|Confidence|Marked by authors)\*{0,2}\s*:\s*\*{0,2}\s*/gim;
    const marks = [...body.matchAll(fieldRe)];
    const fields = {};
    marks.forEach((f, j) => {
      const s = f.index + f[0].length;
      const e = j + 1 < marks.length ? marks[j + 1].index : body.length;
      fields[f[1].toLowerCase()] = body.slice(s, e).trim();
    });
    return { id: `critical:${m[1]}`, title: m[2].trim(), evidence: fields.evidence ?? null, failure: fields.failure ?? fields['chain of events'] ?? null, confidence: fields.confidence ?? null };
  });
}
const dedup = JSON.parse(fs.readFileSync(path.join(E, 'dedup.json'), 'utf8'));
const findings = {};
for (const dir of fs.readdirSync(path.join(E, 'reviews'))) {
  const f = path.join(E, 'reviews', dir, 'critical.md');
  if (!fs.existsSync(f)) continue;
  for (const x of blocks(fs.readFileSync(f, 'utf8'))) {
    if (!x.evidence || !x.failure) throw new Error(`unparsed field in ${dir}:${x.id}`);
    findings[`${dir}:${x.id}`] = x;
  }
}
const rnd = mulberry32(20261006);
for (const c of Object.keys(dedup).sort()) {
  const items = shuffle(dedup[c], rnd).map((it, i) => ({ label: `X-${String(i + 1).padStart(2, '0')}`, ...it }));
  const out = path.join(E, 'adjudication', c);
  fs.mkdirSync(out, { recursive: true });
  const blind = items.map((it) => {
    const parts = it.members.map((m) => {
      const x = findings[m];
      if (!x) throw new Error(`unknown member ${m}`);
      return `**${x.title}**\n- Evidence: ${x.evidence}\n- Failure: ${x.failure}\n- Reviewer confidence: ${x.confidence}`;
    });
    return `## ${it.label}\n\n${parts.length > 1 ? 'Merged from near-identical reports; each version follows.\n\n' : ''}${parts.join('\n\n')}\n`;
  }).join('\n');
  fs.writeFileSync(path.join(out, 'blind.md'), blind);
  fs.writeFileSync(path.join(out, 'key.json'), JSON.stringify(items, null, 2) + '\n');
  console.log(`${c}: ${items.length} items`);
}
// second-opinion sample is drawn later by sample.cjs from the same generator state persisted here
fs.writeFileSync(path.join(E, 'adjudication', 'rng-note.txt'), 'ER-glm (copy of er/pool.cjs): shuffle: mulberry32(20261006), cases in sorted order (gen, prefix, var); sampling continues from a fresh mulberry32(20261006) stream per sample.cjs\n');
