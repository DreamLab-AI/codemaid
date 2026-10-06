'use strict';
// Build gate prompts for one round. node mkgate.cjs <round> <origDiagrams> <frozenNewDiagrams> [unitsFilter.json]
const fs = require('fs'), path = require('path'), { execFileSync } = require('child_process');
const E = __dirname; const [round, O, N, filt] = process.argv.slice(2);
const tpl = fs.readFileSync(path.join(E, 'gate-template.md'), 'utf8');
let { units } = JSON.parse(execFileSync('node', [path.join(E, 'units.cjs'), O, N], { maxBuffer: 1 << 26 }));
units = units.filter((u) => u.new_kind === 'sequenceDiagram');
if (filt) { const keep = new Set(JSON.parse(fs.readFileSync(filt, 'utf8'))); units = units.filter((u) => keep.has(u.id)); }
const dir = path.join(E, 'gate', round); fs.mkdirSync(path.join(dir, 'prompts'), { recursive: true });
const list = units.map((u) => {
  const f = u.id.replace('#', '_'); const out = path.join(dir, `${f}.json`);
  const unit = u.added ? `This heading is not in the ORIGINAL: the rewrite added it as an extra diagram; the nearest preceding original heading is \`${u.parent}\` (a \`${u.orig_kind}\`), and the extra diagram must still state only what the original topic states.`
    : `It replaces the original diagram under the same heading in the ORIGINAL (a \`${u.orig_kind}\`).`;
  const p = tpl.replaceAll('{{ORIG}}', path.join(O, u.topic)).replaceAll('{{NEW}}', path.join(N, u.topic)).replaceAll('{{CODE}}', path.join(E, 'snap'))
    .replaceAll('{{HEAD}}', u.head + (u.id.includes('#') ? ` (block ${u.id.split('#')[1]} under that heading)` : '')).replaceAll('{{UNIT}}', unit).replaceAll('{{OUT}}', out).replaceAll('{{ID}}', u.id);
  fs.writeFileSync(path.join(dir, 'prompts', `${f}.md`), p); return { ...u, file: f, prompt: path.join(dir, 'prompts', `${f}.md`), out };
});
fs.writeFileSync(path.join(dir, 'units.json'), JSON.stringify(list, null, 1) + '\n');
console.log(`${round}: ${list.length} units; kinds ${JSON.stringify(list.reduce((a, u) => (a[u.new_kind] = (a[u.new_kind] || 0) + 1, a), {}))}; added ${list.filter((u) => u.added).length}`);
