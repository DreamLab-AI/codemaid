'use strict';
// Frozen issue list: Tension + Debt + Drift rows of REGISTER.md (Open and Invariant excluded per PREREG).
const fs = require('fs');
const [reg, out, label] = process.argv.slice(2);
const lines = fs.readFileSync(reg, 'utf8').split('\n');
const keep = { '## Tensions': 'Tension', '## Debt': 'Debt', '## Drifts': 'Drift' };
let kind = null; const items = [];
for (const l of lines) {
  if (l.startsWith('## ')) { kind = keep[l.trim()] || null; continue; }
  if (!kind || !l.startsWith('| ') || l.startsWith('| # ') ) continue;
  const cells = l.split(' | ');
  const id = cells[0].replace(/^\|\s*/, '').trim();
  if (!/^[A-Z]+-\d+/.test(id)) continue;
  const where = cells[1].replace(/\]\([^)]*\)/, ']');
  const note = cells.slice(2).join(' | ').replace(/\s*\|\s*$/, '');
  items.push({ id, kind, where, note });
}
fs.writeFileSync(out, JSON.stringify({ corpus: label, items }, null, 2) + '\n');
const c = items.reduce((a, i) => (a[i.kind] = (a[i.kind] || 0) + 1, a), {});
console.log(label, items.length, JSON.stringify(c));
