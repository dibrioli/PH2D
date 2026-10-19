// Sonda: tldraw 5.5.2 montado (<Tldraw hideUi>, caixa preta, rato real do Playwright). Seta kind:'elbow' ligada
// a R1/R2; arrasta a pega do trecho do MEIO; regista elbowMidPoint e a rota; depois move as caixas.
// Só API pública do Editor (createShapes, createBindings, select, setCamera, getShape, getShapeGeometry,
// getShapeHandles, getShapePageTransform). Uso: node sonda_tldraw_segmento.mjs <pasta_saida>
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/tldraw-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const PKG = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/tldraw/package.json"), "utf8"));
const outDir = process.argv[2];
fs.mkdirSync(outDir, { recursive: true });
const SCRIPT_SHA = createHash("sha256").update(fs.readFileSync(fileURLToPath(import.meta.url))).digest("hex");
const MIME = { ".html": "text/html", ".js": "text/javascript", ".woff2": "font/woff2", ".ttf": "font/ttf", ".css": "text/css" };
const server = createServer((req, res) => {
  const p = path.normalize(decodeURIComponent(new URL(req.url, "http://x").pathname));
  const file = path.join(WWW, p === "/" ? "index.html" : p);
  if (!file.startsWith(WWW) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404).end(); return; }
  res.writeHead(200, { "content-type": MIME[path.extname(file)] || "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const origin = `http://127.0.0.1:${server.address().port}`;
const seedScript = (seed) => `(() => { let a = ${seed} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a; t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61); return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next; Object.defineProperty(crypto, "getRandomValues", { value: (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength); for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; } }); })();`;

// câmara: ecrã = (página + camera) * z
const Z = 0.5, CX = 1000, CY = 700;
const toScreen = ([x, y]) => [(x + CX) * Z, (y + CY) * Z];

const CASOS_ALL = [
  { id: "T0_so_arrasto", moves: [] },
  { id: "T1_R2_desce_200", moves: [{ el: "R2", dx: 0, dy: 200 }] },
  { id: "T1b_R2_desce_200_e_volta", moves: [{ el: "R2", dx: 0, dy: 200 }, { el: "R2", dx: 0, dy: -200 }] },
  { id: "T2_R2_direita_300", moves: [{ el: "R2", dx: 300, dy: 0 }] },
  { id: "T3_R2_esquerda_x150", moves: [{ el: "R2", dx: -250, dy: 0 }] },
  { id: "T4_R2_outro_lado_x-400", moves: [{ el: "R2", dx: -800, dy: 0 }] },
  { id: "T5_R2_acima_y-300", moves: [{ el: "R2", dx: 0, dy: -600 }] },
  { id: "T6_R1_direita_150", moves: [{ el: "R1", dx: 150, dy: 0 }] },
  { id: "T7_R1_passa_x300", moves: [{ el: "R1", dx: 300, dy: 0 }] },
  { id: "T8_sem_arrasto_R2_desce_200", noDrag: true, moves: [{ el: "R2", dx: 0, dy: 200 }] },
  { id: "T10_R2_sobe_300_alinhado_e_volta", moves: [{ el: "R2", dx: 0, dy: -300 }, { el: "R2", dx: 0, dy: 300 }] },
  { id: "T13_T3_e_volta", moves: [{ el: "R2", dx: -250, dy: 0 }, { el: "R2", dx: 250, dy: 0 }] },
];
const CASOS = process.env.SO ? CASOS_ALL.filter((c) => c.id.startsWith(process.env.SO)) : CASOS_ALL;

const pageSetup = ({ Z, CX, CY }) => {
  const ed = window.oracle.editor; const id = window.oracle.createShapeId;
  const R1 = id("R1"), R2 = id("R2"), A = id("A");
  ed.createShapes([
    { id: R1, type: "geo", x: 0, y: 0, props: { geo: "rectangle", w: 100, h: 100, fill: "solid" } },
    { id: R2, type: "geo", x: 400, y: 300, props: { geo: "rectangle", w: 100, h: 100, fill: "solid" } },
    { id: A, type: "arrow", x: 50, y: 50, props: { kind: "elbow", start: { x: 0, y: 0 }, end: { x: 400, y: 300 } } },
  ]);
  ed.createBindings([
    { type: "arrow", fromId: A, toId: R1, props: { terminal: "start", normalizedAnchor: { x: 0.5, y: 0.5 }, isExact: false, isPrecise: false } },
    { type: "arrow", fromId: A, toId: R2, props: { terminal: "end", normalizedAnchor: { x: 0.5, y: 0.5 }, isExact: false, isPrecise: false } },
  ]);
  ed.setCamera({ x: CX, y: CY, z: Z });
  ed.selectNone();
  return true;
};
const pageSnap = () => {
  const ed = window.oracle.editor; const id = window.oracle.createShapeId;
  const A = ed.getShape(id("A"));
  const t = ed.getShapePageTransform(A);
  const g = ed.getShapeGeometry(A);
  const verts = (g.vertices || []).map((v) => { const p = t.applyToPoint(v); return [Math.round(p.x * 100) / 100, Math.round(p.y * 100) / 100]; });
  const handles = (ed.getShapeHandles(A) || []).map((h) => { const p = t.applyToPoint(h); return { id: h.id, type: h.type, x: Math.round(p.x * 100) / 100, y: Math.round(p.y * 100) / 100 }; });
  const b = (k) => { const s = ed.getShape(id(k)); return { x: s.x, y: s.y }; };
  const bindings = ed.getBindingsFromShape(A, "arrow").map((x) => ({ toId: x.toId, ...x.props }));
  return JSON.parse(JSON.stringify({ arrow: { x: A.x, y: A.y, props: A.props }, rota_pagina: verts, handles, bindings, R1: b("R1"), R2: b("R2") }));
};

async function drag(page, from, to, steps = 20) {
  await page.mouse.move(from[0], from[1]); await page.waitForTimeout(100);
  await page.mouse.down(); await page.waitForTimeout(100);
  for (let i = 1; i <= steps; i++) { await page.mouse.move(from[0] + (to[0] - from[0]) * i / steps, from[1] + (to[1] - from[1]) * i / steps); await page.waitForTimeout(16); }
  await page.waitForTimeout(150); await page.mouse.up(); await page.waitForTimeout(300);
}
const dedup = (pts) => pts.filter((p, i) => i === 0 || Math.hypot(p[0] - pts[i - 1][0], p[1] - pts[i - 1][1]) > 1e-6);

const browser = await chromium.launch({ headless: true });
const results = [];
const external = new Set();
for (const caso of CASOS) {
  const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
  context.on("request", (r) => { if (!r.url().startsWith(origin)) external.add(r.url().split("?")[0]); });
  await context.addInitScript(seedScript(0x7d1));
  const page = await context.newPage();
  const errs = []; page.on("pageerror", (e) => errs.push(e.message));
  await page.goto(origin + "/", { timeout: 60000 });
  await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 60000 });
  await page.evaluate(pageSetup, { Z, CX, CY });
  await page.waitForTimeout(300);
  const passos = [];
  const s0 = await page.evaluate(pageSnap); passos.push({ passo: "rota_inicial", ...s0 });
  if (!caso.noDrag) {
    await page.evaluate(() => { window.oracle.editor.select(window.oracle.createShapeId("A")); return true; });
    await page.waitForTimeout(250);
    const sSel = await page.evaluate(pageSnap);
    const mids = sSel.handles.filter((h) => h.id.includes("mid") || h.type === "virtual");
    const h = mids[0] || null;
    const rota = dedup(sSel.rota_pagina);
    // o trecho do meio da rota: verificar se é vertical
    const k = Math.floor(rota.length / 2); const p = rota[k - 1], q = rota[k];
    const vertical = Math.abs(p[0] - q[0]) < 1e-6;
    const from = h ? [h.x, h.y] : [(p[0] + q[0]) / 2, (p[1] + q[1]) / 2];
    const to = vertical ? [from[0] + 60, from[1]] : [from[0], from[1] + 60];
    await drag(page, toScreen(from), toScreen(to));
    const s1 = await page.evaluate(pageSnap);
    passos.push({ passo: `arrasto_pega_${h ? h.id : "sem_pega"}_${vertical ? "dx+60" : "dy+60"}`, handles_antes: sSel.handles, de_pagina: from, para_pagina: to, ...s1 });
  }
  for (const m of caso.moves) {
    await page.evaluate((k) => { const ed = window.oracle.editor; ed.selectNone(); ed.select(window.oracle.createShapeId(k)); return true; }, m.el);
    await page.waitForTimeout(200);
    const cur = (await page.evaluate(pageSnap))[m.el];
    const c = [cur.x + 30, cur.y + 30];
    await drag(page, toScreen(c), toScreen([c[0] + m.dx, c[1] + m.dy]), m.steps ?? 30);
    passos.push({ passo: `move_${m.el}_${m.dx}_${m.dy}`, ...(await page.evaluate(pageSnap)) });
  }
  fs.writeFileSync(path.join(outDir, `tldraw_${caso.id}.png`), await page.screenshot());
  results.push({ caso: caso.id, passos, pageerrors: errs });
  console.log(caso.id, JSON.stringify(passos.map((p) => ({ passo: p.passo, rota: dedup(p.rota_pagina), mid: p.arrow.props.elbowMidPoint, R1: p.R1, R2: p.R2 }))));
  await context.close();
}
fs.writeFileSync(path.join(outDir, "tldraw_segmento_meio.json"), JSON.stringify({
  cabecalho: {
    oraculo: `tldraw ${PKG.version} (tldraw license — uso de desenvolvimento/teste, local), <Tldraw hideUi> em Chromium ${browser.version()} headless, rato real do Playwright`,
    data: new Date().toISOString(), host: os.hostname(), loadavg: os.loadavg(),
    comando: `node ${fileURLToPath(import.meta.url)} ${outDir}`, script_sha256: SCRIPT_SHA,
    entrada: "R1 geo rectangle (0,0,100,100); R2 (400,300,100,100); seta kind:'elbow' com bindings start→R1, end→R2 (normalizedAnchor 0.5,0.5, imprecisas)",
    camera: { x: CX, y: CY, z: Z, nota: "ecrã = (página + camera) * z" },
    rota_pagina: "editor.getShapeGeometry(seta).vertices transformados por getShapePageTransform (pode repetir pontos)",
    pedidos_externos: [...external].sort(),
    nota_fonte: "Caixa preta: nenhum código-fonte do tldraw lido; só DOCS.md do pacote (= tldraw.dev).",
  },
  casos: results,
}, null, 2) + "\n");
await browser.close(); server.close();
