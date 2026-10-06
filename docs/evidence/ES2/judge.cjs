'use strict';
// ES2 endpoint-2 fidelity judges: gpt-6-astra via `codex exec`, high effort, --sandbox read-only, cwd =
// read-only 44fef36 snapshot, prompt on stdin. Prompt = ES fidelity-template.md with ES2-3's two harness
// changes (inline file contents; reply with the JSON instead of writing it). One judge per topic.
//   node judge.cjs <template> <origDir> <newDir> <snapDir> <outDir> [concurrency]
const fs = require('fs'), path = require('path'), { spawn } = require('child_process');
const { topics } = require('./units.cjs');
const [TPL, O, N, SNAP, OUT, CONC = '5'] = process.argv.slice(2);
const tpl = fs.readFileSync(TPL, 'utf8');
for (const d of ['prompts', 'inputs', 'raw']) fs.mkdirSync(path.join(OUT, d), { recursive: true });
function prompt(t) {
  const slug = t.replace('/', '_').replace(/\.md$/, ''); const inDir = path.join(OUT, 'inputs', slug); fs.mkdirSync(inDir, { recursive: true });
  const orig = path.join(inDir, 'original.md'), neu = path.join(inDir, 'rewrite.md'), out = path.join(OUT, `${slug}.json`);
  fs.copyFileSync(path.join(O, t), orig); fs.copyFileSync(path.join(N, t), neu);
  let p = tpl.replaceAll('{{ORIG}}', orig).replaceAll('{{NEW}}', neu).replaceAll('{{TOPIC}}', t).replaceAll('{{OUT}}', out);
  const lines = p.replace(/\n$/, '').split('\n');
  const wi = lines.findIndex((l) => l.startsWith('Write exactly one JSON file to ')); const vi = lines.findIndex((l) => l.startsWith('Validate it with node -e'));
  if (wi < 0 || vi < 0) throw new Error('template shape changed');
  lines[wi] = 'Your answer is exactly one JSON object:'; lines[vi] = 'Reply with exactly that JSON object as your whole final message, and nothing else.';
  const fence = (f) => `\n----- ${f} -----\n\`\`\`\`markdown\n${fs.readFileSync(f, 'utf8')}\`\`\`\`\n`;
  p = `${lines.join('\n')}\n\nThe shell is unavailable in this sandbox; the two files' contents follow.\n${fence(orig)}${fence(neu)}`;
  fs.writeFileSync(path.join(OUT, 'prompts', `${slug}.md`), p);
  return { t, slug, p, out };
}
function run(j) { return new Promise((resolve) => {
  const t0 = Date.now();
  const c = spawn('codex', ['exec', '--json', '--skip-git-repo-check', '--sandbox', 'read-only', '--model', 'gpt-6-astra', '-c', 'model_reasoning_effort="high"', '-'], { cwd: SNAP, stdio: ['pipe', 'pipe', 'pipe'] });
  let so = '', se = ''; c.stdout.on('data', (d) => (so += d)); c.stderr.on('data', (d) => (se += d)); c.stdin.end(j.p);
  c.on('close', (code) => {
    fs.writeFileSync(path.join(OUT, 'raw', `${j.slug}.jsonl`), so); fs.writeFileSync(path.join(OUT, 'raw', `${j.slug}.err`), se);
    const ev = so.split('\n').filter(Boolean).map((l) => { try { return JSON.parse(l); } catch { return null; } }).filter(Boolean);
    const msgs = ev.filter((e) => e.type === 'item.completed' && e.item?.type === 'agent_message').map((e) => e.item.text);
    const usage = ev.filter((e) => e.type === 'turn.completed').map((e) => e.usage);
    const cmds = ev.filter((e) => e.type === 'item.completed' && e.item?.type === 'command_execution').length;
    let ok = false, err = null;
    const last = (msgs[msgs.length - 1] || '').trim();
    const body = last.replace(/^```(?:json)?\s*/, '').replace(/\s*```$/, '');
    try { const o = JSON.parse(body.slice(body.indexOf('{'), body.lastIndexOf('}') + 1)); fs.writeFileSync(j.out, JSON.stringify(o, null, 1) + '\n'); ok = true; } catch (e) { err = e.message; }
    resolve({ topic: j.t, slug: j.slug, ok, err, exit: code, seconds: (Date.now() - t0) / 1000, usage, commands_attempted: cmds });
  }); }); }
(async () => {
  const jobs = topics(N).map(prompt); const res = []; let i = 0;
  await Promise.all(Array.from({ length: Number(CONC) }, async () => { while (i < jobs.length) { const j = jobs[i++]; let r = await run(j);
    if (!r.ok) { r = { ...(await run(j)), retried_after: r.err }; } res.push(r); console.error(`${r.slug} ok=${r.ok} ${r.seconds}s`); } }));
  fs.writeFileSync(path.join(OUT, 'judge-manifest.json'), JSON.stringify(res.sort((a, b) => a.slug.localeCompare(b.slug)), null, 1) + '\n');
})();
