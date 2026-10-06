'use strict';
// EK analysis: per-run recall from blind scores, five endpoints, decision rule, paired bootstrap.
const fs = require('fs');
const SEED = 20261006, B = 10000, ARMS = ['A', 'S', 'M', 'P'], CORPORA = ['campaignbuilder', 'sealmap'];
function mulberry32(a) { return () => { a |= 0; a = (a + 0x6D2B79F5) | 0; let t = Math.imul(a ^ (a >>> 15), 1 | a); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }; }
const mapping = JSON.parse(fs.readFileSync('mapping.json', 'utf8'));
const gold = Object.fromEntries(CORPORA.map((c) => [c, JSON.parse(fs.readFileSync(`${c}/gold.json`, 'utf8')).items]));
const tokens = Object.fromEntries(fs.readFileSync('tokens.jsonl', 'utf8').trim().split('\n').map(JSON.parse).map((t) => [t.pack, t]));
const runs = mapping.map((r) => {
  const s = JSON.parse(fs.readFileSync(`scores/${r.run_id}.json`, 'utf8'));
  const man = JSON.parse(fs.readFileSync(`runs/${r.run_id}/manifest.json`, 'utf8'));
  const ids = new Set(gold[r.corpus].map((g) => g.id));
  const byId = {};
  for (const i of s.issues || []) { if (!ids.has(i.id)) throw new Error(`${r.run_id}: unknown issue ${i.id}`);
    const v = String(i.score).toUpperCase(); if (v === 'MISS') continue; if (!['FULL', 'PARTIAL'].includes(v)) throw new Error(`${r.run_id}: bad score ${v}`);
    if (byId[i.id] !== 'FULL') byId[i.id] = v; }
  const full = Object.values(byId).filter((v) => v === 'FULL').length, partial = Object.values(byId).filter((v) => v === 'PARTIAL').length;
  const kindHits = {}; for (const g of gold[r.corpus]) if (byId[g.id]) kindHits[g.kind] = (kindHits[g.kind] || 0) + (byId[g.id] === 'FULL' ? 1 : 0.5);
  const f = s.findings || []; const cls = (c) => f.filter((x) => String(x.class).toUpperCase() === c).length;
  const ver = (v) => f.filter((x) => String(x.class).toUpperCase() === 'NOVEL' && String(x.verdict).toUpperCase() === v).length;
  if (f.length !== man.findings) console.error(`warn ${r.run_id}: scorer classified ${f.length} findings, parser found ${man.findings}`);
  const tk = tokens[`${r.corpus}/arms/pack-${r.arm}.txt`];
  return { ...r, gold_n: gold[r.corpus].length, full, partial, recall: (full + 0.5 * partial) / gold[r.corpus].length,
    recall_by_kind: kindHits, findings: f.length, reg: cls('REG'), novel: cls('NOVEL'), vague: cls('VAGUE'),
    real: ver('REAL'), partly: ver('PARTLY'), wrong: ver('WRONG'), pack_tokens: tk.pack_tokens,
    seconds: man.seconds, prompt_tokens: man.prompt_tokens, cached_tokens: man.cached_tokens, thinking_tokens: man.thinking_tokens, output_tokens: man.output_tokens };
});
const mean = (xs) => xs.reduce((a, b) => a + b, 0) / xs.length;
const sd = (xs) => { const m = mean(xs); return Math.sqrt(xs.reduce((a, b) => a + (b - m) ** 2, 0) / (xs.length - 1)); };
const sel = (arm, corpus) => runs.filter((r) => r.arm === arm && (!corpus || r.corpus === corpus)).sort((a, b) => a.k - b.k);
// Paired, corpus-stratified bootstrap: per resample and corpus, draw 3 replicate slots with
// replacement and apply the same slots to every arm; statistic = pooled mean over the 6 drawn runs.
const rnd = mulberry32(SEED);
const draws = Array.from({ length: B }, () => Object.fromEntries(CORPORA.map((c) => [c, [0, 1, 2].map(() => Math.floor(rnd() * 3))])));
function boot(fn) {
  const vals = draws.map((d) => fn((arm) => CORPORA.flatMap((c) => { const rs = sel(arm, c); return d[c].map((i) => rs[i]); })));
  vals.sort((a, b) => a - b);
  return { ci95: [vals[Math.floor(0.025 * B)], vals[Math.ceil(0.975 * B) - 1]] };
}
const pooled = (arm, key = 'recall') => mean(sel(arm).map((r) => r[key]));
const diff = (x, y) => ({ estimate: pooled(x) - pooled(y), ...boot((get) => mean(get(x).map((r) => r.recall)) - mean(get(y).map((r) => r.recall))) });
const armCI = (arm) => boot((get) => mean(get(arm).map((r) => r.recall))).ci95;
const perArm = Object.fromEntries(ARMS.map((a) => {
  const rs = sel(a); const tok = Object.fromEntries(CORPORA.map((c) => [c, sel(a, c)[0].pack_tokens]));
  return [a, { pooled_recall: pooled(a), pooled_recall_ci95: armCI(a), pooled_sd: sd(rs.map((r) => r.recall)),
    per_corpus: Object.fromEntries(CORPORA.map((c) => { const x = sel(a, c).map((r) => r.recall); return [c, { recalls: x, mean: mean(x), sd: sd(x),
      recall_per_100k_tokens: mean(x) / (tok[c] / 1e5) }]; })),
    pack_tokens: tok, recall_per_100k_pack_tokens_pooled: mean(CORPORA.map((c) => mean(sel(a, c).map((r) => r.recall)) / (tok[c] / 1e5))),
    wrong_per_run: pooled(a, 'wrong'), novel_per_run: pooled(a, 'novel'), real_per_run: pooled(a, 'real'), partly_per_run: pooled(a, 'partly'),
    vague_per_run: pooled(a, 'vague'), reg_per_run: pooled(a, 'reg'), findings_per_run: pooled(a, 'findings'),
    wrong_rate_of_findings: rs.reduce((s, r) => s + r.wrong, 0) / rs.reduce((s, r) => s + r.findings, 0),
    mean_seconds: pooled(a, 'seconds'), total_prompt_tokens: rs.reduce((s, r) => s + r.prompt_tokens, 0),
    total_cached_tokens: rs.reduce((s, r) => s + (r.cached_tokens || 0), 0), total_thinking_tokens: rs.reduce((s, r) => s + r.thinking_tokens, 0),
    total_output_tokens: rs.reduce((s, r) => s + r.output_tokens, 0) }];
}));
const e1 = diff('S', 'A'), e2 = diff('S', 'M'), e3 = diff('A', 'P');
const perCorpusDiff = Object.fromEntries(CORPORA.map((c) => [c, Object.fromEntries([['S-A', 'S', 'A'], ['S-M', 'S', 'M'], ['A-P', 'A', 'P']].map(([n, x, y]) =>
  [n, mean(sel(x, c).map((r) => r.recall)) - mean(sel(y, c).map((r) => r.recall))]))]));
// Per-corpus bootstrap of each contrast (same draws, one stratum).
const perCorpusCI = Object.fromEntries(CORPORA.map((c) => [c, Object.fromEntries([['S-A','S','A'],['S-M','S','M'],['A-P','A','P']].map(([n,x,y]) => { const v = draws.map((d) => { const rx = sel(x,c), ry = sel(y,c); return mean(d[c].map((i)=>rx[i].recall)) - mean(d[c].map((i)=>ry[i].recall)); }).sort((a,b)=>a-b); return [n, [v[Math.floor(0.025*B)], v[Math.ceil(0.975*B)-1]]]; }))]));
const nonInferior = e1.estimate > -0.05, sGeM = e2.estimate >= 0;
const result = { generated: new Date().toISOString(), seed: SEED, resamples: B,
  bootstrap: 'paired, corpus-stratified: each resample draws 3 replicate slots with replacement per corpus and applies the same slots to every arm; the statistic is the pooled mean over the 6 drawn runs',
  gold_sizes: Object.fromEntries(CORPORA.map((c) => [c, gold[c].length])),
  endpoints: { '1_S_minus_A': { ...e1, margin: -0.05, non_inferior: nonInferior, ci_lower_above_margin: e1.ci95[0] > -0.05 },
    '2_S_minus_M': { ...e2, S_ge_M: sGeM }, '3_A_minus_P': e3,
    '4_cost': Object.fromEntries(ARMS.map((a) => [a, { pack_tokens: perArm[a].pack_tokens, recall_per_100k_pack_tokens: perArm[a].recall_per_100k_pack_tokens_pooled }])),
    '5_precision_wrong_per_run': Object.fromEntries(ARMS.map((a) => [a, perArm[a].wrong_per_run])) },
  per_corpus_differences: perCorpusDiff, per_corpus_ci95: perCorpusCI,
  decision: { S_non_inferior_to_A: nonInferior, S_ge_M: sGeM, adopt_sequence_first: nonInferior && sGeM,
    text: nonInferior && sGeM ? 'S is non-inferior to A and S >= M: diagrams-as-code moves to sequence-first authoring.' : 'Decision rule not met: the mixed corpus stays and the result is published as found.' },
  per_arm: perArm, per_run: runs.sort((a, b) => a.corpus.localeCompare(b.corpus) || a.arm.localeCompare(b.arm) || a.k - b.k) };
fs.writeFileSync('results.json', JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ endpoints: result.endpoints, decision: result.decision, per_corpus_differences: perCorpusDiff }, null, 1));
