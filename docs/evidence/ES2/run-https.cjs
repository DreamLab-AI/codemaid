'use strict';
// One EK review run: an arm's pack + the baked critical lens, sent exactly as external-review.cjs
// sends it (contents [pack, lens], thinking high, temperature 0, same retry policy). Independent
// request per run; nothing is shared between runs except Gemini's own implicit prefix cache.
//   node run.cjs <pack> <outdir>          review
//   node run.cjs --count <pack>           countTokens for pack alone and pack+lens
const fs = require('fs'); const path = require('path');
const ER = require('/home/devuser/workspace/project/agentbox/skills/sealmap-review/scripts/external-review.cjs');
const API = 'https://generativelanguage.googleapis.com/v1beta/models';
function httpsPost(url, key, data) { return new Promise((resolve, reject) => { const u = new URL(url); const rq = require('https').request({ hostname: u.hostname, path: u.pathname + u.search, method: 'POST', headers: { 'content-type': 'application/json', 'x-goog-api-key': key, 'content-length': Buffer.byteLength(data) }, timeout: 900_000 }, (rs) => { let b = ''; rs.setEncoding('utf8'); rs.on('data', (c) => (b += c)); rs.on('end', () => resolve({ ok: rs.statusCode >= 200 && rs.statusCode < 300, status: rs.statusCode, json: async () => JSON.parse(b) })); }); rq.on('timeout', () => rq.destroy(new Error('https timeout 900s'))); rq.on('error', reject); rq.end(data); }); }
const MODEL = process.env.DIAGRAM_REVIEW_MODEL || 'gemini-3.8-flash';
async function gemini(method, key, body) {
  for (let attempt = 0; ; attempt++) {
    const res = await httpsPost(`${API}/${MODEL}:${method}`, key, JSON.stringify(body));
    const json = await res.json().catch(() => ({}));
    if (res.ok) return json;
    if ((res.status === 429 || res.status >= 500) && attempt < 2) { await new Promise((r) => setTimeout(r, 5000 * (attempt + 1))); continue; }
    throw new Error(`${method} ${res.status}: ${JSON.stringify(json.error ?? json).slice(0, 400)}`);
  }
}
(async () => {
  const key = process.env.GEMINI_API_KEY; if (!key) throw new Error('no key');
  const lens = ER.loadLens('critical', 15);
  if (process.argv[2] === '--count') {
    const pack = fs.readFileSync(process.argv[3], 'utf8');
    const a = await gemini('countTokens', key, { contents: [{ role: 'user', parts: [{ text: pack }] }] });
    const b = await gemini('countTokens', key, { contents: [{ role: 'user', parts: [{ text: pack }, { text: lens }] }] });
    console.log(JSON.stringify({ pack: process.argv[3], pack_tokens: a.totalTokens, pack_plus_lens_tokens: b.totalTokens }));
    return;
  }
  const [packPath, out] = process.argv.slice(2);
  const pack = fs.readFileSync(packPath, 'utf8');
  fs.mkdirSync(out, { recursive: true });
  const started = Date.now();
  const res = await gemini('generateContent', key, { contents: [{ role: 'user', parts: [{ text: pack }, { text: lens }] }],
    generationConfig: { thinkingConfig: { thinkingLevel: process.env.DIAGRAM_REVIEW_THINKING || 'high' }, temperature: 0 } });
  const text = res.candidates?.[0]?.content?.parts?.filter((p) => p.text && !p.thought).map((p) => p.text).join('') ?? '';
  if (!text) throw new Error(`empty reply (finishReason ${res.candidates?.[0]?.finishReason ?? 'none'})`);
  fs.writeFileSync(path.join(out, 'critical.md'), text.endsWith('\n') ? text : `${text}\n`);
  const findings = ER.parseFindings(text, 'critical');
  fs.writeFileSync(path.join(out, 'findings.json'), JSON.stringify(findings, null, 2) + '\n');
  const u = res.usageMetadata || {};
  const m = { model: MODEL, pack: packPath, started: new Date(started).toISOString(), seconds: Math.round((Date.now() - started) / 100) / 10,
    findings: findings.length, finish: res.candidates?.[0]?.finishReason, prompt_tokens: u.promptTokenCount,
    cached_tokens: u.cachedContentTokenCount ?? 0, thinking_tokens: u.thoughtsTokenCount, output_tokens: u.candidatesTokenCount };
  fs.writeFileSync(path.join(out, 'manifest.json'), JSON.stringify(m, null, 2) + '\n');
  console.log(JSON.stringify(m));
})().catch((e) => { console.error(`run: ${e.message}`); process.exit(1); });
