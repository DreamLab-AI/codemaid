'use strict';
// ES2 analysis: arm R2 (GLM-5.3-Flash sequence-first rewrite + Claude fidelity gate + one repair) against EK's sealmap A and S runs and ES's R.
// Recall = (FULL + 0.5 PARTIAL) / 45 per run. Paired bootstrap by replicate slot (EK Amendment 5,
// single stratum = sealmap): 10,000 resamples, mulberry32(20261006), 3 slots drawn with replacement
// and applied to both arms. Endpoint 3 from the per-topic fidelity judges.
//   node analyse-es2.cjs <ES2 dir> <EK dir> <ES dir>
const fs = require('fs'), path = require('path');
const [ES, EK, ESR] = process.argv.slice(2);
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
  return { run_id: m.run_id, arm: 'R2', k: m.k, full, partial, recall: (full + 0.5 * partial) / gold.length, findings: f.length,
    reg: cls('REG'), novel: cls('NOVEL'), vague: cls('VAGUE'), real: ver('REAL'), partly: ver('PARTLY'), wrong: ver('WRONG'),
    seconds: man.seconds, prompt_tokens: man.prompt_tokens, cached_tokens: man.cached_tokens, thinking_tokens: man.thinking_tokens, output_tokens: man.output_tokens, finish: man.finish };
});
const esr = rd(path.join(ESR, 'results.json')).per_run.filter((r) => r.arm === 'R');
const runs = [...R, ...esr, ...ek.map((r) => ({ run_id: r.run_id, arm: r.arm, k: r.k, full: r.full, partial: r.partial, recall: r.recall, findings: r.findings, reg: r.reg, novel: r.novel, vague: r.vague, real: r.real, partly: r.partly, wrong: r.wrong, seconds: r.seconds, prompt_tokens: r.prompt_tokens, thinking_tokens: r.thinking_tokens, output_tokens: r.output_tokens }))];
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
const jdir = path.join(ES, 'fidelity'); const judges = fs.readdirSync(jdir).filter((f) => f.endsWith('.json') && f !== 'judge-manifest.json').sort().map((f) => rd(path.join(jdir, f)));
const n = (x) => (Array.isArray(x) ? x.length : 0);
const fid = judges.map((j) => ({ topic: j.topic, facts_dropped: n(j.facts_dropped), facts_invented: n(j.facts_invented), citations_dropped: n(j.citations_dropped), citations_added: n(j.citations_added) }));
const sum = (k) => fid.reduce((s, x) => s + x[k], 0);
const e3 = { topics: fid.length, per_topic: fid, totals: { facts_dropped: sum('facts_dropped'), facts_invented: sum('facts_invented'), citations_dropped: sum('citations_dropped'), citations_added: sum('citations_added') },
  topics_with_invented_facts: fid.filter((x) => x.facts_invented > 0).length, topics_with_dropped_facts: fid.filter((x) => x.facts_dropped > 0).length,
  topics_with_citation_changes: fid.filter((x) => x.citations_dropped + x.citations_added > 0).length };
// Endpoint 3: conversion, and the gate record (gate/final.json written by finalise.cjs).
const g = rd(path.join(ES, 'gate', 'final.json'));
const conv = { original_non_sequence: g.original_non_sequence, ended_sequence: g.converted, restored: g.restored.length, rate: g.converted / g.original_non_sequence, threshold: 0.70 };
const e1 = diff('R2', 'A'), xS = diff('R2', 'S'), xR = diff('R2', 'R');
const pass1 = e1.ci95[0] >= MARGIN, pass2 = e3.topics_with_invented_facts <= 2, pass3 = conv.rate >= 0.70;
const holds = pass1 && pass2 && pass3;
const out = { generated: new Date().toISOString(), seed: SEED, resamples: B, gold_n: gold.length,
  bootstrap: 'paired by replicate slot, single stratum (sealmap): each resample draws 3 slots with replacement and applies them to both arms',
  arms: { R2: armStats('R2'), A: armStats('A'), S: armStats('S'), R: armStats('R') },
  endpoints: { '1_recall_R2_minus_A': { ...e1, margin: MARGIN, pass: pass1 },
    '2_fidelity': { ...e3, threshold: 'invented or changed facts in at most 2 of 20 topics', pass: pass2 },
    '3_conversion': { ...conv, pass: pass3 },
    '4_reported': { wrong_per_run: { R2: armStats('R2').wrong_per_run, A: armStats('A').wrong_per_run, S: armStats('S').wrong_per_run, R: armStats('R').wrong_per_run },
      gate: g.rounds, exploratory_R2_minus_S: xS, exploratory_R2_minus_R: xR } },
  decision: { holds, rule: 'endpoints 1, 2 and 3 all pass',
    text: holds ? 'Holds: sequence-first authoring with a different-family fidelity gate becomes the recommended corpus style.' : 'Does not hold: mixed corpora stay.' },
  per_run: runs };
fs.writeFileSync(path.join(ES, 'results.json'), JSON.stringify(out, null, 2) + '\n');
console.log(JSON.stringify({ arms: Object.fromEntries(Object.entries(out.arms).map(([k, v]) => [k, { mean: v.mean, ci95: v.ci95, recalls: v.recalls, wrong: v.wrong_per_run }])), e1, e2: { ...e3, per_topic: undefined }, e3: conv, xS, xR, gate: g.rounds, decision: out.decision }, null, 1));
