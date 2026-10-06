// Sonda: o roundness que o EDITOR escreve ao DESENHAR rectângulo/losango/elipse com os padrões.
// Uso: PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh timeout 300 node docs/MiroClone/ferramentas/excalidraw_oracle/sonda_editor_arredondamento.mjs
// Medido 2026-10-06 (0.18.1): rectangle {type:3} · diamond {type:2} · ellipse {type:2}.
import { createRequire } from "node:module";
import { createServer } from "node:http";
import fs from "node:fs"; import path from "node:path";
const ORACLE_HOME = path.join(process.env.HOME, "Apps/excalidraw-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const MIME = { ".html": "text/html", ".js": "text/javascript", ".woff2": "font/woff2", ".ttf": "font/ttf", ".wasm": "application/wasm", ".css": "text/css" };
const server = createServer((req, res) => {
  const p = path.normalize(decodeURIComponent(new URL(req.url, "http://x").pathname));
  const file = path.join(WWW, p === "/" ? "index.html" : p);
  if (!file.startsWith(WWW) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404).end(); return; }
  res.writeHead(200, { "content-type": MIME[path.extname(file)] || "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const origin = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch({ headless: true });
try {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await ctx.route("**/*", (r) => (r.request().url().startsWith(origin) ? r.continue() : r.abort()));
  const page = await ctx.newPage();
  await page.goto(origin + "/");
  await page.waitForFunction(() => window.oracleReady === true);
  await page.evaluate(() => window.oracle.editorMount([]));
  await page.waitForTimeout(500);
  for (const [key, x0] of [["r", 300], ["d", 600], ["o", 900]]) {
    await page.keyboard.press("Escape");
    await page.keyboard.press(key);
    await page.mouse.move(x0, 300); await page.mouse.down();
    await page.mouse.move(x0 + 80, 350, { steps: 5 }); await page.mouse.move(x0 + 160, 420, { steps: 5 });
    await page.mouse.up(); await page.waitForTimeout(200);
  }
  const els = await page.evaluate(() => window.oracle.editorRead());
  for (const e of els.elements) console.log(JSON.stringify({ type: e.type, roundness: e.roundness, roughness: e.roughness, strokeWidth: e.strokeWidth, w: e.width, h: e.height }));
} finally { await browser.close(); server.close(); }
