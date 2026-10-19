// Sonda: Excalidraw 0.18.1 montado (caixa preta, rato real do Playwright). Arrasta o trecho do MEIO de uma
// seta elbowed ligada a R1/R2, regista fixedSegments e points, e depois move R2 (casos C*).
// Uso: node sonda_excalidraw_segmento.mjs <pasta_saida>
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const PKG = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/@excalidraw/excalidraw/package.json"), "utf8"));
const outDir = process.argv[2];
fs.mkdirSync(outDir, { recursive: true });
const SCRIPT_SHA = createHash("sha256").update(fs.readFileSync(fileURLToPath(import.meta.url))).digest("hex");
const r2 = (v) => Math.round(v * 100) / 100;

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
const seedScript = (seed) => `(() => { let a = ${seed} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a; t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61); return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next; Object.defineProperty(crypto, "getRandomValues", { value: (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength); for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; } }); })();`;

const ENTRADA = {
  elements: [
    { type: "rectangle", id: "R1", x: 0, y: 0, width: 100, height: 100, backgroundColor: "#a5d8ff", fillStyle: "solid" },
    { type: "rectangle", id: "R2", x: 400, y: 300, width: 100, height: 100, backgroundColor: "#ffc9c9", fillStyle: "solid" },
    { type: "arrow", id: "A", x: 100, y: 50, width: 300, height: 300, elbowed: true, start: { id: "R1" }, end: { id: "R2" } },
  ],
};
// Câmara: zoom 0.5, scroll (1000,700) ⇒ ecrã = (cena + scroll) * 0.5. Cena visível x -1000..1560, y -700..900.
const Z = 0.5, SX = 1000, SY = 700;
const toScreen = ([x, y]) => [(x + SX) * Z, (y + SY) * Z];

// Casos: cada um parte do ZERO (página nova): rota → arrasto do trecho do meio → movimentos de forma.
const CASOS_ALL = [
  { id: "C0_so_arrasto", moves: [] },
  { id: "C1_R2_desce_200", moves: [{ el: "R2", dx: 0, dy: 200 }] },
  { id: "C1b_R2_desce_200_e_volta", moves: [{ el: "R2", dx: 0, dy: 200 }, { el: "R2", dx: 0, dy: -200 }] },
  { id: "C2_R2_direita_300", moves: [{ el: "R2", dx: 300, dy: 0 }] },
  { id: "C3_R2_esquerda_do_fixo_x150", moves: [{ el: "R2", dx: -250, dy: 0 }] },
  { id: "C4_R2_outro_lado_x-400", moves: [{ el: "R2", dx: -800, dy: 0 }] },
  { id: "C5_R2_acima_y-300", moves: [{ el: "R2", dx: 0, dy: -600 }] },
  { id: "C6_R1_move_direita_150", moves: [{ el: "R1", dx: 150, dy: 0 }] },
  { id: "C7_R1_passa_o_fixo_x300", moves: [{ el: "R1", dx: 300, dy: 0 }] },
  { id: "C8_sem_arrasto_R2_desce_200", noDrag: true, moves: [{ el: "R2", dx: 0, dy: 200 }] },
  { id: "C5b_R2_acima_y-300_num_salto", moves: [{ el: "R2", dx: 0, dy: -600, steps: 1 }] },
  { id: "C9_R2_sobe_250_y50_sobrepoe_R1", moves: [{ el: "R2", dx: 0, dy: -250 }] },
  { id: "C10_R2_sobe_300_y0_alinhado", moves: [{ el: "R2", dx: 0, dy: -300 }] },
  { id: "C10b_R2_sobe_300_e_volta", moves: [{ el: "R2", dx: 0, dy: -300 }, { el: "R2", dx: 0, dy: 300 }] },
  { id: "C11_arrasta_1o_trecho_dy+60", seg: 1, moves: [{ el: "R2", dx: 0, dy: 200 }] },
  { id: "C12_duplo_clique_no_meio_do_fixo", dbl: true, moves: [] },
  { id: "C13_C3_e_volta", moves: [{ el: "R2", dx: -250, dy: 0 }, { el: "R2", dx: 250, dy: 0 }] },
];

const CASOS = process.env.SO ? CASOS_ALL.filter((c) => c.id.startsWith(process.env.SO)) : CASOS_ALL;
const browser = await chromium.launch({ headless: true });
const snap = async (page) => page.evaluate(() => {
  const els = window.oracle.editorApi().getSceneElements().filter((e) => !e.isDeleted);
  const pick = (e) => e.type === "arrow"
    ? { id: e.id, x: e.x, y: e.y, width: e.width, height: e.height, points: e.points, elbowed: e.elbowed, fixedSegments: e.fixedSegments ?? null,
        startIsSpecial: e.startIsSpecial ?? null, endIsSpecial: e.endIsSpecial ?? null, startBinding: e.startBinding, endBinding: e.endBinding, version: e.version }
    : { id: e.id, x: e.x, y: e.y, width: e.width, height: e.height, boundElements: e.boundElements };
  return els.map(pick);
});
const absPoints = (a) => a.points.map(([px, py]) => [Math.round((a.x + px) * 100) / 100, Math.round((a.y + py) * 100) / 100]);
const summarize = (els) => { const a = els.find((e) => e.id === "A"); return { abs: absPoints(a), fixedSegments: a.fixedSegments, startIsSpecial: a.startIsSpecial, endIsSpecial: a.endIsSpecial,
  R1: (({ x, y }) => ({ x, y }))(els.find((e) => e.id === "R1")), R2: (({ x, y }) => ({ x, y }))(els.find((e) => e.id === "R2")) }; };

async function drag(page, from, to, steps = 20) {
  await page.mouse.move(from[0], from[1]);
  await page.waitForTimeout(100);
  await page.mouse.down();
  await page.waitForTimeout(100);
  for (let i = 1; i <= steps; i++) {
    await page.mouse.move(from[0] + (to[0] - from[0]) * i / steps, from[1] + (to[1] - from[1]) * i / steps);
    await page.waitForTimeout(16);
  }
  await page.waitForTimeout(150);
  await page.mouse.up();
  await page.waitForTimeout(250);
}

const results = [];
for (const caso of CASOS) {
  const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
  await context.route("**/*", (route) => { const u = route.request().url(); if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue(); return route.abort(); });
  await context.addInitScript(seedScript(0x5e9));
  const page = await context.newPage();
  await page.clock.setFixedTime(Date.UTC(2026, 0, 1));
  const errs = []; page.on("pageerror", (e) => errs.push(e.message));
  await page.goto(origin + "/");
  await page.waitForFunction(() => window.oracleReady === true);
  const restored = (await page.evaluate((inp) => window.oracle.run(inp), ENTRADA)).restored;
  await page.evaluate((els) => window.oracle.editorMount(els), restored);
  await page.evaluate(({ Z, SX, SY }) => window.oracle.editorApi().updateScene({ appState: { scrollX: SX, scrollY: SY, zoom: { value: Z } } }), { Z, SX, SY });
  // rota: empurra as pontas → ← (como o corre.mjs)
  await page.evaluate(() => window.oracle.editorSelect(["R1", "R2"]));
  await page.waitForTimeout(200); await page.keyboard.press("ArrowRight"); await page.waitForTimeout(200); await page.keyboard.press("ArrowLeft"); await page.waitForTimeout(300);
  const passos = [];
  const s0 = await snap(page); passos.push({ passo: "rota_inicial", ...summarize(s0) });
  if (!caso.noDrag) {
    const a = summarize(s0).abs;
    // o trecho do meio: com 4 pontos é o p1→p2; com n pontos, o índice floor(n/2)
    const k = caso.seg ?? Math.floor(a.length / 2);
    const p = a[k - 1], q = a[k];
    const mid = [(p[0] + q[0]) / 2, (p[1] + q[1]) / 2];
    const vertical = Math.abs(p[0] - q[0]) < 1e-6;
    const target = vertical ? [mid[0] + 60, mid[1]] : [mid[0], mid[1] + 60];
    // selecciona como o artista: clique no trecho do meio a 1/4 do comprimento (longe do ponto médio)
    await page.evaluate(() => window.oracle.editorApi().updateScene({ appState: { selectedElementIds: {} } }));
    const quarter = [p[0] + (q[0] - p[0]) * 0.25, p[1] + (q[1] - p[1]) * 0.25];
    await page.mouse.click(...toScreen(quarter));
    await page.waitForTimeout(300);
    if (process.env.DEBUG_SHOT) fs.writeFileSync(path.join(outDir, `dbg_${caso.id}_selected.png`), await page.screenshot());
    const selInfo = await page.evaluate(() => { const st = window.oracle.editorApi().getAppState(); return { selected: Object.keys(st.selectedElementIds || {}), selectedLinearElement: st.selectedLinearElement ? Object.keys(st.selectedLinearElement) : null }; });
    console.log("sel", JSON.stringify(selInfo));
    await drag(page, toScreen(mid), toScreen(target));
    const s1 = await snap(page);
    if (caso.dbl) {
      const sA = summarize(s1).abs; const pm = [(sA[k - 1][0] + sA[k][0]) / 2, (sA[k - 1][1] + sA[k][1]) / 2];
      passos.push({ passo: `arrasto_trecho_${k - 1}-${k}`, ...summarize(s1) });
      await page.mouse.dblclick(...toScreen(pm)); await page.waitForTimeout(300);
      const s2 = await snap(page);
      passos.push({ passo: "duplo_clique_ponto_medio_do_trecho_fixo", ponto_cena: pm, ...summarize(s2), arrow_raw: s2.find((e) => e.id === "A"), selectedLinearElement_editing: await page.evaluate(() => !!window.oracle.editorApi().getAppState().editingLinearElement) });
    } else passos.push({ passo: `arrasto_trecho_${k - 1}-${k}_${vertical ? "vertical_dx+60" : "horizontal_dy+60"}`, mid_cena: mid, alvo_cena: target, ...summarize(s1), arrow_raw: s1.find((e) => e.id === "A") });
  }
  for (const m of caso.moves) {
    await page.evaluate(() => window.oracle.editorApi().updateScene({ appState: { selectedElementIds: {} } }));
    await page.evaluate((id) => window.oracle.editorSelect([id]), m.el);
    await page.waitForTimeout(200);
    const cur = (await snap(page)).find((e) => e.id === m.el);
    const c = [cur.x + 30, cur.y + 30];
    await drag(page, toScreen(c), toScreen([c[0] + m.dx, c[1] + m.dy]), m.steps ?? 30);
    const s = await snap(page);
    passos.push({ passo: `move_${m.el}_${m.dx}_${m.dy}`, ...summarize(s), arrow_raw: s.find((e) => e.id === "A") });
  }
  const png = await page.screenshot();
  fs.writeFileSync(path.join(outDir, `excalidraw_${caso.id}.png`), png);
  results.push({ caso: caso.id, passos, pageerrors: errs });
  console.log(caso.id, JSON.stringify(passos.map((p) => ({ passo: p.passo, abs: p.abs, fixed: p.fixedSegments, R1: p.R1, R2: p.R2 }))));
  await context.close();
}
const out = {
  cabecalho: {
    oraculo: `@excalidraw/excalidraw ${PKG.version} (MIT), <Excalidraw> montado em Chromium ${browser.version()} headless, rato real do Playwright`,
    data: new Date().toISOString(), host: os.hostname(), loadavg: os.loadavg(),
    comando: `node ${fileURLToPath(import.meta.url)} ${outDir}`, script_sha256: SCRIPT_SHA,
    entrada: ENTRADA, camera: { zoom: Z, scrollX: SX, scrollY: SY, nota: "ecrã = (cena + scroll) * zoom" },
    metodo: "rota forçada empurrando R1+R2 → ← (como corre.mjs); seta seleccionada; arrasto do ponto médio do trecho do meio 60 px de cena na perpendicular; depois selecciona a forma e arrasta-a pelo rato (30 passos). Cada caso parte de página nova.",
    nota_fonte: "Caixa preta: nenhum código-fonte do Excalidraw lido.",
  },
  casos: results,
};
fs.writeFileSync(path.join(outDir, "excalidraw_segmento_fixo.json"), JSON.stringify(out, null, 2) + "\n");
await browser.close(); server.close();
