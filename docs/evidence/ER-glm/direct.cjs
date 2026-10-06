'use strict';
// Direct Anthropic-Messages call to Z.AI for packs Claude Code refuses client-side ("Prompt is too long":
// it assumes a 200K window for the unrecognised model id). Same prompt.txt on the wire, no tools, no
// thinking parameter (GLM default = max effort), max_tokens 128000, streamed.
//   node direct.cjs <prompt.txt> <outdir>
const fs = require('fs'), path = require('path');
const [pf, out] = process.argv.slice(2);
const base = process.env.ZAI_URL.replace(/\/$/, ''), key = process.env.ZAI_ANTHROPIC_API_KEY;
(async () => {
  const t0 = Date.now();
  const res = await fetch(`${base}/v1/messages`, { method: 'POST',
    headers: { 'content-type': 'application/json', 'x-api-key': key, 'anthropic-version': '2023-06-01' },
    body: JSON.stringify({ model: 'glm-5.3-flash', max_tokens: 128000, stream: true,
      messages: [{ role: 'user', content: fs.readFileSync(pf, 'utf8') }] }) });
  if (!res.ok) { const t = await res.text(); throw new Error(`${res.status} ${t.slice(0, 500)}`); }
  let buf = '', text = '', thinking = '', usage = {}, stop = null;
  const dec = new TextDecoder();
  for await (const chunk of res.body) {
    buf += dec.decode(chunk, { stream: true });
    let i; while ((i = buf.indexOf('\n')) >= 0) { const line = buf.slice(0, i).trim(); buf = buf.slice(i + 1);
      if (!line.startsWith('data:')) continue; let ev; try { ev = JSON.parse(line.slice(5)); } catch { continue; }
      if (ev.type === 'message_start') Object.assign(usage, ev.message.usage || {});
      if (ev.type === 'content_block_delta') { if (ev.delta.type === 'text_delta') text += ev.delta.text; if (ev.delta.type === 'thinking_delta') thinking += ev.delta.thinking; }
      if (ev.type === 'message_delta') { Object.assign(usage, ev.usage || {}); stop = ev.delta?.stop_reason ?? stop; }
      if (ev.type === 'error') throw new Error(JSON.stringify(ev.error)); } }
  fs.mkdirSync(out, { recursive: true });
  fs.writeFileSync(path.join(out, 'critical.md'), text);
  const m = { transport: 'direct-messages-api', model: 'glm-5.3-flash', seconds: (Date.now() - t0) / 1000, stop_reason: stop, usage, thinking_chars: thinking.length };
  fs.writeFileSync(path.join(out, 'direct-manifest.json'), JSON.stringify(m, null, 2) + '\n'); console.log(JSON.stringify(m));
})().catch((e) => { console.error('direct:', e.message); process.exit(1); });
