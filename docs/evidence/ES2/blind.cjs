'use strict';
// ES2 blind findings: runs/<id>/findings.json (the script's parseFindings) -> blind/<id>.json, as ES.
// ES2-4 (2): if any finding lacks Topics/Evidence/Failure, or fewer findings than F-NN headings, the
// critical.md is re-extracted with a tolerant parser (plain or bold labels) and that is used instead.
const fs = require('fs'), path = require('path');
const [E] = process.argv.slice(2); const log = [];
function tolerant(md) { const heads = [...md.matchAll(/^###\s+\**([FR]-\d+)\**\s*[—–-]\s*(.+)$/gm)];
  return heads.map((m, i) => { const end = i + 1 < heads.length ? heads[i + 1].index : md.length;
    const body = md.slice(m.index + m[0].length, end).split(/^#{1,2} /m)[0];
    const field = (name) => (body.match(new RegExp(`^\\s*[-*]\\s*\\**${name}\\**\\s*:\\s*\\**\\s*(.+)$`, 'mi')) || [])[1]?.trim() ?? null;
    return { id: `critical:${m[1]}`, title: m[2].replace(/\*+$/, '').trim(), topics: field('Topics'), evidence: field('Evidence'),
      failure: field('Failure') ?? field('Chain of events'), confidence: field('Confidence'), marked_by_authors: field('Marked by authors') }; }); }
fs.mkdirSync(path.join(E, 'blind'), { recursive: true });
for (const m of JSON.parse(fs.readFileSync(path.join(E, 'mapping.json'), 'utf8'))) {
  const dir = path.join(E, 'runs', m.run_id); let f = JSON.parse(fs.readFileSync(path.join(dir, 'findings.json'), 'utf8'));
  const md = fs.readFileSync(path.join(dir, 'critical.md'), 'utf8'); const nHeads = (md.match(/^###\s+\**F-\d+/gm) || []).length;
  const bad = f.filter((x) => !x.topics || !x.evidence || !x.failure).length; let used = 'parseFindings';
  if (bad || f.filter((x) => x.kind !== 'root-cause').length < nHeads) { f = tolerant(md); used = 'tolerant'; fs.writeFileSync(path.join(dir, 'findings.tolerant.json'), JSON.stringify(f, null, 2) + '\n'); }
  const blind = f.map((x, i) => ({ n: i + 1, title: x.title, topics: x.topics, evidence: x.evidence, failure: x.failure, confidence: x.confidence, marked_by_authors: x.marked_by_authors }));
  fs.writeFileSync(path.join(E, 'blind', `${m.run_id}.json`), JSON.stringify(blind, null, 2) + '\n');
  log.push({ run_id: m.run_id, headings: nHeads, parsed: f.length, null_fields_in_parseFindings: bad, used });
}
console.log(JSON.stringify(log));
