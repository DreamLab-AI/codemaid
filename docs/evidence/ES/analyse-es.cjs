'use strict';
// ES analysis: arm R (GLM-5.3-Flash sequence-first rewrite) against EK's sealmap A and S runs.
// Recall = (FULL + 0.5 PARTIAL) / 45 per run. Paired bootstrap by replicate slot (EK Amendment 5,
// single stratum = sealmap): 10,000 resamples, mulberry32(20261006), 3 slots drawn with replacement
// and applied to both arms. Endpoint 3 from the per-topic fidelity judges.
//   node analyse-es.cjs <ES dir> <EK dir>
const fs = require('fs'), path = require('path');
const [ES, EK] = process.argv.slice(2);
const SEED = 20261006, B = 10000, MARGIN = -0.05;
function mulberry32(a) { return () => { a |= 0; a = (a + 0x6D2B79F5) | 0; let t = Math.imul(a ^ (a >>> 15), 1 | a); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }; }
const rd = (p) => JSON.parse(fs.readFileSync(p, 'utf8'));
const gold = rd(path.join(EK, 'sealmap/gold.json')).items; const ids = new Set(gold.map((g) => g.id));
const ek = rd(path.join(EK, 'results.json')).per_run.filter((r) => r.corpus === 'sealmap' && ['A', 'S'].includes(r.arm));
const mapping = rd(path.join(ES, 'mapping.json'));
const R = mapping.map((m) => {
  const s = rd(path.join(ES, 'scores', `${m.run_id}.json`)); const man = rd(path.join(ES, 'runs', m.run_id, 'manifest.json'));
  const byId = {};
  for (const i of s.issues || []) { if (!ids.has(i.id)) throw new Error(`${m.run_id}: unknown issue ${i.id}`);
    const v = String(i.score).toUpperCase(); if (v === 'MISS') continue; if (!['FULL', 'PARTIAL'].includes(v)) throw new Error(`bad score ${v}`);
    if (byId[i.id] !== 'FULL') byId[i.id] = v; }
  const full = Object.values(byId).filter((v) => v === 'FULL').length, partial = Object.values(byId).filter((v) => v === 'PARTIAL').length;
  const f = s.findings || []; const cls = (c) => f.filter((x) => String(x.class).toUpperCase() === c).length;
  const ver = (v) => f.filter((x) => String(x.class).toUpperCase() === 'NOVEL' && String(x.verdict).toUpperCase() === v).length;
  if (f.length !== man.findings) console.error(`warn ${m.run_id}: scorer classified ${f.length}, parser found ${man.findings}`);
  return { run_id: m.run_id, arm: 'R', k: m.k, full, partial, recall: (full + 0.5 * partial) / gold.length, findings: f.length,
    reg: cls('REG'), novel: cls('NOVEL'), vague: cls('VAGUE'), real: ver('REAL'), partly: ver('PARTLY'), wrong: ver('WRONG'),
    seconds: man.seconds, prompt_tokens: man.prompt_tokens, cached_tokens: man.cached_tokens, thinking_tokens: man.thinking_tokens, output_tokens: man.output_tokens, finish: man.finish };
});
const runs = [...R, ...ek.map((r) => ({ run_id: r.run_id, arm: r.arm, k: r.k, full: r.full, partial: r.partial, recall: r.recall, findings: r.findings, reg: r.reg, novel: r.novel, vague: r.vague, real: r.real, partly: r.partly, wrong: r.wrong, seconds: r.seconds, prompt_tokens: r.prompt_tokens, thinking_tokens: r.thinking_tokens, output_tokens: r.output_tokens }))];
const sel = (a) => runs.filter((r) => r.arm === a).sort((x, y) => x.k - y.k);
const mean = (xs) => xs.reduce((s, x) => s + x, 0) / xs.length;
const rnd = mulberry32(SEED); const draws = Array.from({ length: B }, () => [0, 1, 2].map(() => Math.floor(rnd() * 3)));
const ci = (vals) => { vals.sort((a, b) => a - b); return [vals[Math.floor(0.025 * B)], vals[Math.ceil(0.975 * B) - 1]]; };
const diff = (x, y) => { const rx = sel(x), ry = sel(y); return { estimate: mean(rx.map((r) => r.recall)) - mean(ry.map((r) => r.recall)),
  ci95: ci(draws.map((d) => mean(d.map((i) => rx[i].recall)) - mean(d.map((i) => ry[i].recall)))) }; };
const armStats = (a) => { const rs = sel(a); return { recalls: rs.map((r) => r.recall), mean: mean(rs.map((r) => r.recall)),
  ci95: ci(draws.map((d) => mean(d.map((i) => rs[i].recall)))), wrong_per_run: mean(rs.map((r) => r.wrong)), novel_per_run: mean(rs.map((r) => r.novel)),
  real_per_run: mean(rs.map((r) => r.real)), partly_per_run: mean(rs.map((r) => r.partly)), vague_per_run: mean(rs.map((r) => r.vague)),
  wrong_total: rs.reduce((s, r) => s + r.wrong, 0), findings_total: rs.reduce((s, r) => s + r.findings, 0), mean_seconds: mean(rs.map((r) => r.seconds)) }; };
// Endpoint 3: fidelity judges.
const jdir = path.join(ES, 'fidelity'); const judges = fs.readdirSync(jdir).filter((f) => f.endsWith('.json')).sort().map((f) => rd(path.join(jdir, f)));
const n = (x) => (Array.isArray(x) ? x.length : 0);
const fid = judges.map((j) => ({ topic: j.topic, facts_dropped: n(j.facts_dropped), facts_invented: n(j.facts_invented), citations_dropped: n(j.citations_dropped), citations_added: n(j.citations_added) }));
const sum = (k) => fid.reduce((s, x) => s + x[k], 0);
const e3 = { topics: fid.length, per_topic: fid, totals: { facts_dropped: sum('facts_dropped'), facts_invented: sum('facts_invented'), citations_dropped: sum('citations_dropped'), citations_added: sum('citations_added') },
  topics_with_invented_facts: fid.filter((x) => x.facts_invented > 0).length, topics_with_dropped_facts: fid.filter((x) => x.facts_dropped > 0).length,
  topics_with_citation_changes: fid.filter((x) => x.citations_dropped + x.citations_added > 0).length };
const e1 = diff('R', 'A'), e2 = diff('R', 'S');
const holds = e1.ci95[0] >= MARGIN && e3.topics_with_invented_facts <= 2;
const out = { generated: new Date().toISOString(), seed: SEED, resamples: B, gold_n: gold.length,
  bootstrap: 'paired by replicate slot, single stratum (sealmap): each resample draws 3 slots with replacement and applies them to both arms',
  arms: { R: armStats('R'), A: armStats('A'), S: armStats('S') },
  endpoints: { '1_R_minus_A': { ...e1, margin: MARGIN, ci_lower_ge_margin: e1.ci95[0] >= MARGIN, point_above_margin: e1.estimate > MARGIN },
    '2_R_minus_S': e2, '3_fidelity': e3,
    '4_wrong_per_run': { R: armStats('R').wrong_per_run, A: armStats('A').wrong_per_run, S: armStats('S').wrong_per_run } },
  decision: { holds, rule: "endpoint 1 CI lower bound >= -0.05 AND invented facts in no more than 2 topics",
    text: holds ? 'Holds: a sequence-first rewrite reviews as well as the mixed corpus on sealmap; licenses a larger test.' : 'Does not hold: mixed corpora stay.' },
  per_run: runs };
fs.writeFileSync(path.join(ES, 'results.json'), JSON.stringify(out, null, 2) + '\n');
console.log(JSON.stringify({ arms: Object.fromEntries(Object.entries(out.arms).map(([k, v]) => [k, { mean: v.mean, recalls: v.recalls, wrong: v.wrong_per_run }])), e1, e2, e3: { ...e3, per_topic: undefined }, decision: out.decision }, null, 1));
