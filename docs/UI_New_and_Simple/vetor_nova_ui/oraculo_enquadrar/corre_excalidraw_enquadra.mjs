// O 2.º ORÁCULO DO «ENQUADRAR TUDO» (F8) — o Excalidraw `next` (MIT) montado COM a interface. Cada
// cena NOSSA (`cenas.mjs`) é enquadrada pela TECLA do produto (Shift+1 = tudo, Shift+2 = seleção) e
// a partir de três câmaras (a API `scrollToContent` saiu do canal `next` — só a tecla, que é o produto);
// grava a câmara final, as quatro margens no ecrã e a câmara a CADA QUADRO depois da ação.
// Corre-se e não se lê: só a API pública (`convertToExcalidrawElements`, `updateScene`,
// `getAppState`, `getViewportOffsets`) e o teclado do Playwright. Números INTEIROS.
//
// uso:  PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh timeout 900 node <este> [saida_dir]
//       ORACULO_VP=LxA mede outra janela (a saída ganha o sufixo).
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { CENAS, PARTIDAS, VIEWPORT } from "./cenas.mjs";

const AQUI = path.dirname(new URL(import.meta.url).pathname);
const APP = path.join(os.homedir(), "Apps/excalidraw-oracle-next");
const VP = process.env.ORACULO_VP ? Object.fromEntries(process.env.ORACULO_VP.split("x").map((v, i) => [i ? "height" : "width", +v])) : VIEWPORT;
const SAI = process.argv[2] || path.join(AQUI, "fixtures_excalidraw" + (process.env.ORACULO_VP ? "_" + process.env.ORACULO_VP : ""));
const require = createRequire(path.join(APP, "package.json"));
const { chromium } = require("playwright");
const pkg = (p) => JSON.parse(fs.readFileSync(path.join(APP, "node_modules", p, "package.json"), "utf8"));
const VERSAO = `@excalidraw/excalidraw ${pkg("@excalidraw/excalidraw").version} (${pkg("@excalidraw/excalidraw").license})`;

const WWW = path.join(APP, "www_w9");
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
fs.mkdirSync(SAI, { recursive: true });
try {
  const ctx = await browser.newContext({ viewport: VP });
  await ctx.route("**/*", (r) => (r.request().url().startsWith(origin) ? r.continue() : r.abort()));
  const page = await ctx.newPage();
  await page.goto(origin + "/");
  await page.waitForFunction(() => window.oracleReady === true);
  await page.evaluate(() => window.oracle.mount([]));
  // A ponte monta o editor numa caixa de 1280×800; noutra janela, a caixa passa a ocupar a janela.
  await page.evaluate(() => {
    const host = document.body.lastElementChild;
    host.style.width = "100vw";
    host.style.height = "100vh";
    window.dispatchEvent(new Event("resize"));
  });
  await page.waitForTimeout(500);

  const monta = (cena, partida) => page.evaluate(([cena, partida]) => {
    const api = window.oracle.editorApi();
    const EX = window.oracle.api;
    const els = EX.convertToExcalidrawElements(cena.objs.map(([x, y, w, h, t], i) => ({
      type: t === "ellipse" ? "ellipse" : "rectangle", id: "o" + i, x, y, width: w, height: h,
    }))).map((e, i) => ({ ...e, locked: (cena.tranca || []).includes(i) }));
    api.updateScene({
      elements: els,
      appState: {
        // O conversor dá ids PRÓPRIOS (medido: os nossos `o<i>` não ficam) — seleciona-se pelos dele.
        selectedElementIds: Object.fromEntries((cena.sel || []).map((i) => [els[i].id, true])),
        selectedGroupIds: {},
        scrollX: partida.x, scrollY: partida.y, zoom: { value: partida.z },
      },
    });
    document.querySelector(".excalidraw")?.focus();
    return els.length;
  }, [cena, partida]);

  const amostra = (ms) => page.evaluate((ms) => new Promise((ok) => {
    const api = window.oracle.editorApi();
    const s = [];
    const t0 = performance.now();
    const f = () => {
      const a = api.getAppState();
      s.push([performance.now() - t0, a.scrollX, a.scrollY, a.zoom.value]);
      if (performance.now() - t0 < ms) requestAnimationFrame(f); else ok(s);
    };
    requestAnimationFrame(f);
  }), ms);

  const le = () => page.evaluate(() => {
    const api = window.oracle.editorApi();
    const a = api.getAppState();
    const els = api.getSceneElements().filter((e) => !e.isDeleted);
    const caixa = (es) => {
      if (!es.length) return null;
      const x0 = Math.min(...es.map((e) => e.x)), y0 = Math.min(...es.map((e) => e.y));
      const x1 = Math.max(...es.map((e) => e.x + e.width)), y1 = Math.max(...es.map((e) => e.y + e.height));
      return { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
    };
    const z = a.zoom.value;
    const ecra = (b) => b && { x0: (b.x + a.scrollX) * z, y0: (b.y + a.scrollY) * z, x1: (b.x + b.w + a.scrollX) * z, y1: (b.y + b.h + a.scrollY) * z };
    const tudo = caixa(els);
    const sel = caixa(els.filter((e) => a.selectedElementIds[e.id]));
    return {
      camara: { x: a.scrollX, y: a.scrollY, z },
      viewport: { x: 0, y: 0, w: a.width, h: a.height, offset: [a.offsetLeft, a.offsetTop], ui: api.getViewportOffsets?.() ?? null },
      tudo_pagina: tudo, tudo_ecra: ecra(tudo), selecao_pagina: sel, selecao_ecra: ecra(sel),
      selecionados: Object.keys(a.selectedElementIds).length,
    };
  });

  const ACOES = {
    tecla_shift_1: async () => { await page.keyboard.press("Shift+Digit1"); },
    tecla_shift_2: async () => { await page.keyboard.press("Shift+Digit2"); },
  };

  for (const [nome, cena] of Object.entries(CENAS)) {
    const fix = {
      _cabecalho: {
        oraculo: VERSAO, cena: nome, data: new Date().toISOString().slice(0, 10),
        como: `Editor COM interface montado ${VP.width}×${VP.height}; elementos criados por convertToExcalidrawElements (locked onde «tranca»), seleção e câmara de partida por updateScene; a ação é a TECLA pelo teclado do Playwright; a câmara amostrada a cada requestAnimationFrame durante 1500 ms`,
        unidade: "px de cena e px de ecrã (ecrã = (cena + scroll) · zoom)",
        objectos: cena,
      },
      corridas: {},
    };
    for (const [pnome, partida] of Object.entries(PARTIDAS)) {
      for (const [anome, acao] of Object.entries(ACOES)) {
        await monta(cena, partida);
        await page.waitForTimeout(60);
        const antes = await le();
        const pAm = amostra(1500);
        pAm.catch(() => {});
        await acao();
        const quadros = await pAm;
        await page.waitForTimeout(100);
        const depois = await le();
        fix.corridas[`${pnome}/${anome}`] = { antes: antes.camara, depois, quadros };
      }
    }
    fs.writeFileSync(path.join(SAI, nome + ".json"), JSON.stringify(fix, null, 1) + "\n");
    const r = fix.corridas["z1_origem/tecla_shift_1"];
    console.log(`${nome.padEnd(18)} shift+1 → z ${r.depois.camara.z} · ${r.quadros.length} quadros`);
  }
} finally {
  await browser.close();
  server.close();
}
