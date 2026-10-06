'use strict';
// ES2 finalise: build arm R2 from the repaired corpus and the two gate rounds (ES2-2).
//  - round-1 pass: the rewrite1 block is kept (any repair-session edit to it is reverted).
//  - round-1 fail: kept as repaired only if the repaired block is a sequenceDiagram, differs from
//    rewrite1, and every round-2 unit for it (its heading and any added sections after it) passed;
//    otherwise the original block is restored and its added sections are dropped.
//   node finalise.cjs   (paths fixed under es2/)
const fs = require('fs'), path = require('path'); const E = __dirname;
const { topics, sections } = require('./units.cjs');
const O = path.join(E, 'original-diagrams'), W1 = path.join(E, 'rewrite1/docs/diagrams'), REP = path.join(E, 'repaired/docs/diagrams'), OUT = path.join(E, 'R2/corpus/docs/diagrams');
const r1 = require('./gate/round1/units.json').map((u) => ({ ...u, j: JSON.parse(fs.readFileSync(u.out, 'utf8')) }));
const r2p = path.join(E, 'gate/round2/units.json'); const r2 = fs.existsSync(r2p) ? require(r2p).map((u) => ({ ...u, j: JSON.parse(fs.readFileSync(u.out, 'utf8')) })) : [];
const fail1 = new Set(r1.filter((u) => u.j.verdict.toLowerCase() === 'fail').map((u) => u.head));
fs.rmSync(path.join(E, 'R2'), { recursive: true, force: true }); fs.cpSync(REP, OUT, { recursive: true }); require('child_process').execFileSync('chmod', ['-R', 'u+w', path.join(E, 'R2')]);
const record = []; const reverted = [];
for (const t of topics(O)) {
  const oT = fs.readFileSync(path.join(O, t), 'utf8'), w1T = fs.readFileSync(path.join(W1, t), 'utf8'), rT = fs.readFileSync(path.join(REP, t), 'utf8');
  const oS = Object.fromEntries(sections(oT).map((s) => [s.head, s])), w1S = Object.fromEntries(sections(w1T).map((s) => [s.head, s]));
  const L = rT.split('\n'); const oL = oT.split('\n'), w1L = w1T.split('\n');
  const rs = sections(rT); const edits = []; // {from,to,lines} or {drop:[start,end]}
  let parent = null; const decision = {};
  for (const s of rs) {
    if (oS[s.head]) parent = s.head;
    if (oS[s.head] && !fail1.has(s.head)) {               // round-1 pass or original sequence: rewrite1 block stands
      s.blocks.forEach((b, k) => { const w = w1S[s.head].blocks[k]; if (b.src !== w.src) { reverted.push(`${t} ${s.head}`); edits.push({ from: b.from, to: b.to, lines: w1L.slice(w.from, w.to + 1) }); } });
      continue; }
    const key = parent;
    if (decision[key] === undefined) {
      const units2 = r2.filter((u) => u.topic === t && (u.head === key || u.parent === key));
      const head = rs.find((x) => x.head === key); const blk = head.blocks[0]; const w = w1S[key].blocks[0];
      const changed = rs.filter((x) => x.head === key || (!oS[x.head] && units2.some((u) => u.head === x.head))).length > 1 || blk.src !== w.src;
      const seq = blk.kind === 'sequenceDiagram';
      const pass2 = units2.length > 0 && units2.every((u) => u.j.verdict.toLowerCase() === 'pass');
      decision[key] = { keep: changed && seq && pass2, changed, seq, pass2, units2: units2.map((u) => u.id) };
      record.push({ topic: t, head: key, orig_kind: oS[key].blocks[0].kind, ...decision[key] });
    }
    if (decision[key].keep) continue;
    if (oS[s.head]) { const ob = oS[s.head].blocks[0]; const b = s.blocks[0]; edits.push({ from: b.from, to: b.to, lines: oL.slice(ob.from, ob.to + 1) }); }
    else edits.push({ drop: [s.start, s.end] });
  }
  edits.sort((a, b) => (b.from ?? b.drop[0]) - (a.from ?? a.drop[0]));
  for (const e of edits) { if (e.drop) L.splice(e.drop[0], e.drop[1] - e.drop[0]); else L.splice(e.from, e.to - e.from + 1, ...e.lines); }
  fs.writeFileSync(path.join(OUT, t), L.join('\n'));
}
// Conversion: an original non-sequence diagram converts if its heading's final block is a sequenceDiagram.
const conv = [];
for (const t of topics(O)) { const f = Object.fromEntries(sections(fs.readFileSync(path.join(OUT, t), 'utf8')).map((s) => [s.head, s]));
  for (const s of sections(fs.readFileSync(path.join(O, t), 'utf8'))) if (s.blocks[0] && s.blocks[0].kind !== 'sequenceDiagram') conv.push({ topic: t, head: s.head, final_kind: f[s.head].blocks[0].kind }); }
const converted = conv.filter((c) => c.final_kind === 'sequenceDiagram').length;
const rounds = { round1: { units: r1.length, pass: r1.filter((u) => u.j.verdict.toLowerCase() === 'pass').length },
  round2: { units: r2.length, pass: r2.filter((u) => u.j.verdict.toLowerCase() === 'pass').length },
  repair_targets: fail1.size, repaired_and_kept: record.filter((r) => r.keep).length, restored: record.filter((r) => !r.keep).length };
rounds.round1.rate = rounds.round1.pass / rounds.round1.units; rounds.round2.rate = rounds.round2.units ? rounds.round2.pass / rounds.round2.units : null;
rounds.final_pass_rate_of_rewritten = (rounds.round1.pass + rounds.repaired_and_kept) / rounds.round1.units;
const out = { original_non_sequence: conv.length, converted, restored: record.filter((r) => !r.keep), repair_decisions: record, reverted_stray_edits: reverted, rounds, per_diagram: conv };
fs.writeFileSync(path.join(E, 'gate/final.json'), JSON.stringify(out, null, 1) + '\n');
console.log(JSON.stringify({ converted, of: conv.length, rate: converted / conv.length, rounds, reverted }, null, 1));
