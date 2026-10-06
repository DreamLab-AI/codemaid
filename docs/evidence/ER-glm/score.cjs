#!/usr/bin/env node
'use strict';
// ER-glm scoring, same rules as er/score.cjs: final verdict = most conservative of primary and
// second opinion (unsupported > unsure > confirmed); hidden detected = final "yes" on an item that is
// not unsupported (#E8); cost per #E9 (primary cost split equally over a case's items, second-opinion
// cost split over its items). Also a primary-only sensitivity (#E13). Writes results.json.
const fs = require('node:fs'), path = require('node:path');
const E = __dirname; const rd = (p) => JSON.parse(fs.readFileSync(path.join(E, p), 'utf8'));
const RANK = { confirmed: 0, unsure: 1, unsupported: 2 }, HRANK = { yes: 0, partial: 1, no: 2 };
const cases = ['gen', 'var', 'prefix'];
const pcost = rd('adjudication/primary-cost.json'), c2cost = rd('adjudication/claude2-cost.json'), sel = rd('adjudication/second-selection.json');
const items = [];
for (const c of cases) {
  const key = rd(`adjudication/${c}/key.json`);
  const prim = Object.fromEntries(rd(`adjudication/${c}/primary.json`).map((p) => [p.label, p]));
  const c2 = fs.existsSync(path.join(E, `adjudication/${c}/claude2.json`)) ? Object.fromEntries(rd(`adjudication/${c}/claude2.json`).map((p) => [p.label, p])) : {};
  const c2labels = sel[c].map((s) => s.label);
  for (const k of key) {
    const ops = [{ who: 'claude-primary', ...prim[k.label] }];
    let tok = pcost[c].tokens / key.length, sec = pcost[c].seconds / key.length;
    if (c2labels.includes(k.label)) { if (!c2[k.label]) throw new Error(`${c} ${k.label}: second opinion missing`);
      ops.push({ who: 'claude-second', ...c2[k.label] }); tok += c2cost[c].tokens / c2labels.length; sec += c2cost[c].seconds / c2labels.length; }
    const final = ops.map((o) => o.verdict).reduce((a, b) => (RANK[b] > RANK[a] ? b : a));
    const hidden = ops.map((o) => o.hidden_defect).reduce((a, b) => (HRANK[b] > HRANK[a] ? b : a));
    items.push({ case: c, label: k.label, members: k.members, opinions: ops.map((o) => ({ who: o.who, verdict: o.verdict, hidden_defect: o.hidden_defect, rationale: o.rationale })),
      final, primary_only: ops[0].verdict, hidden_final: hidden, cost_tokens: Math.round(tok), cost_seconds: Math.round(sec * 10) / 10 });
  }
}
const arms = {};
for (const arm of ['A', 'B']) {
  const mine = items.filter((it) => it.members.some((m) => m.includes(`-${arm}-glm:`)));
  const per = Object.fromEntries(cases.map((c) => { const ci = mine.filter((i) => i.case === c);
    return [c, { items: ci.length, confirmed: ci.filter((i) => i.final === 'confirmed').length, confirmed_primary_only: ci.filter((i) => i.primary_only === 'confirmed').length,
      hidden_detected: ci.some((i) => i.hidden_final === 'yes' && i.final !== 'unsupported'), hidden_partial: ci.some((i) => i.hidden_final === 'partial') }]; }));
  arms[`${arm}-glm`] = { per_case: per, confirmed: mine.filter((i) => i.final === 'confirmed').length, confirmed_primary_only: mine.filter((i) => i.primary_only === 'confirmed').length,
    unsupported: mine.filter((i) => i.final === 'unsupported').length, unsure: mine.filter((i) => i.final === 'unsure').length,
    hidden_detected: cases.filter((c) => per[c].hidden_detected).length,
    adjudication_tokens: mine.reduce((s, i) => s + i.cost_tokens, 0), adjudication_seconds: Math.round(mine.reduce((s, i) => s + i.cost_seconds, 0) * 10) / 10 };
}
const pairs = items.flatMap((it) => it.opinions.slice(1).map((o) => ({ case: it.case, label: it.label, primary: it.opinions[0].verdict, second: o.verdict })));
const review_cost = {};
for (const d of fs.readdirSync(path.join(E, 'reviews'))) { const r = path.join(E, 'reviews', d);
  if (fs.existsSync(path.join(r, 'direct-manifest.json'))) { const m = rd(`reviews/${d}/direct-manifest.json`); review_cost[d] = { transport: 'direct Messages API (#G2)', seconds: m.seconds, input_tokens: m.usage.input_tokens, output_tokens: m.usage.output_tokens, claude_code_attempt: 'Prompt is too long (client-side, 0 API ms)' }; }
  else { const m = rd(`reviews/${d}/manifest.json`); review_cost[d] = { transport: 'claude -p --bare --tools ""', seconds: m.wall_seconds, input_tokens: m.usage.input_tokens, output_tokens: m.usage.output_tokens, modelUsage: m.modelUsage }; } }
const A = arms['A-glm'], Bm = arms['B-glm'];
const rule = (a, b) => ({ a_ge_b_plus_2: a.confirmed >= b.confirmed + 2, hidden_ok: a.hidden_detected >= b.hidden_detected, cost_ok: a.adjudication_tokens <= b.adjudication_tokens });
const out = { generated: new Date().toISOString(), exploratory: true, note: 'Dated exploratory third reviewer added after ER finished; cannot change ER decision.',
  items, arms, agreement: { pairs, agree: pairs.filter((p) => p.primary === p.second).length, total: pairs.length },
  decision_rule_glm: { ...rule(A, Bm), branch: (() => { const r = rule(A, Bm); return r.a_ge_b_plus_2 && r.hidden_ok && r.cost_ok ? 'review-pack improvement' : 'otherwise (hunk-overlap checker)'; })() },
  review_cost, primary_cost: pcost, second_cost: c2cost };
fs.writeFileSync(path.join(E, 'results.json'), JSON.stringify(out, null, 2) + '\n');
console.log(JSON.stringify({ arms, agreement: { agree: out.agreement.agree, total: out.agreement.total }, decision_rule_glm: out.decision_rule_glm, review_cost }, null, 1));
