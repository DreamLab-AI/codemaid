// Validate every ```mermaid block in the given Markdown files with real Mermaid in headless Chromium.
// Usage: npm i mermaid puppeteer && sealmap generate . -o .sealmap && node tools/validate-mermaid.mjs $(find .sealmap -name '*.md')
// Env: CHROME=/path/to/chromium (default: puppeteer's bundled browser), MODE=render for a full render.
import fs from 'fs';
import { createRequire } from 'module';
import puppeteer from 'puppeteer';
const files = process.argv.slice(2);
const blocks = [];
for (const f of files) {
  const t = fs.readFileSync(f, 'utf8');
  const re = /```mermaid\n([\s\S]*?)```/g; let m;
  while ((m = re.exec(t))) blocks.push({ f, code: m[1] });
}
const browser = await puppeteer.launch({ executablePath: process.env.CHROME || undefined, args: ['--no-sandbox'], protocolTimeout: 0 });
const MODE = process.env.MODE || 'parse';
const page = await browser.newPage();
await page.setContent('<html><body></body></html>');
await page.addScriptTag({ path: createRequire(import.meta.url).resolve('mermaid/dist/mermaid.min.js') });
await page.evaluate(() => mermaid.initialize({ startOnLoad: false, maxTextSize: 1e7, maxEdges: 5000 }));
const results = [];
for (let k = 0; k < blocks.length; k += 100) {
  const chunk = blocks.slice(k, k + 100);
  const r = await page.evaluate(async (chunk, k, mode) => {
    const out = [];
    let i = k;
    for (const b of chunk) {
      try {
        if (mode === 'render') { await mermaid.render('d' + (i++), b.code); document.body.innerHTML = ''; }
        else { await mermaid.parse(b.code); }
        out.push(null);
      } catch (e) { out.push(String(e && e.message || e).slice(0, 300)); }
    }
    return out;
  }, chunk, k, MODE);
  results.push(...r);
}
await browser.close();
let bad = 0;
results.forEach((r, i) => { if (r) { bad++; if (bad <= 15) console.log(`FAIL ${blocks[i].f}\n${r}\n---\n${blocks[i].code.split('\n').slice(0, 40).join('\n')}\n`); } });
console.log(`${blocks.length} diagrams, ${bad} failed`);
process.exit(bad ? 1 : 0);
