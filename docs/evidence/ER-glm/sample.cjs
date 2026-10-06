#!/usr/bin/env node
'use strict';
// Second-opinion selection (#E7): every primary "unsure" plus ceil(25%) of the rest per case,
// drawn with one mulberry32(20261006) stream over cases in sorted order (gen, prefix, var).
// Routing: Gemini origin -> gpt-6-astra; Luna origin -> second Claude subagent; both -> both.
const fs = require('node:fs');
const path = require('node:path');
const E = __dirname;
function mulberry32(a) {
  return function () {
    a |= 0; a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const rnd = mulberry32(20261006);
const out = {};
for (const c of ['gen', 'prefix', 'var']) {
  const key = JSON.parse(fs.readFileSync(path.join(E, 'adjudication', c, 'key.json'), 'utf8'));
  const prim = JSON.parse(fs.readFileSync(path.join(E, 'adjudication', c, 'primary.json'), 'utf8'));
  const v = Object.fromEntries(prim.map((p) => [p.label, p.verdict]));
  if (key.some((k) => !v[k.label])) throw new Error(`${c}: primary verdict missing`);
  const unsure = key.filter((k) => v[k.label] === 'unsure').map((k) => k.label);
  const rest = key.filter((k) => v[k.label] !== 'unsure').map((k) => k.label);
  const n = Math.ceil(rest.length * 0.25);
  const pool = rest.slice();
  for (let i = pool.length - 1; i > 0; i--) { const j = Math.floor(rnd() * (i + 1)); [pool[i], pool[j]] = [pool[j], pool[i]]; }
  const sampled = pool.slice(0, n).sort();
  const chosen = [...unsure, ...sampled].sort();
  out[c] = chosen.map((label) => {
    const k = key.find((x) => x.label === label);
    const gem = k.members.some((m) => m.includes('-gemini:'));
    const luna = k.members.some((m) => m.includes('-luna:') || m.includes('-glm:')); // ER-glm: GLM origin routes to a second Claude, like Luna
    return { label, reason: unsure.includes(label) ? 'unsure' : 'sample', astra: gem, claude2: luna };
  });
}
fs.writeFileSync(path.join(E, 'adjudication', 'second-selection.json'), JSON.stringify(out, null, 2) + '\n');
console.log(JSON.stringify(out, null, 1));
