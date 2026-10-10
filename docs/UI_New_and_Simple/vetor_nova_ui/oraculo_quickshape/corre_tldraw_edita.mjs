// O 2.º ORÁCULO DO «EDIT SHAPE» (F7e) — o tldraw 5.5.2: não reconhece traços, mas as formas NATIVAS
// dele são as que o QuickShape faz (rectângulo, elipse, triângulo, linha de vários nós, seta com
// curva) e cada uma tem as alças PRÓPRIAS do tipo. Grava, por forma, as alças que mostra e o que
// arrastar cada uma (com e sem Shift) faz à forma — e, nas lineares, se a curva passa pelo ponto
// largado. Licença: «tldraw license» (NÃO permissiva) ⇒ corre-se, nunca se lê nem se porta.
// Só API pública do Editor (`createShapes`, `select`, `getShape`, `getShapeHandles`,
// `getShapePageTransform`, `getSelectionRotatedPageBounds`) e o rato do Playwright.
//
// uso:  PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh timeout 600 node <este> [saida_dir]
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createServer } from "node:http";
import { createRequire } from "node:module";

const AQUI = path.dirname(new URL(import.meta.url).pathname);
const APP = path.join(os.homedir(), "Apps/tldraw-oracle");
const SAI = process.argv[2] || path.join(AQUI, "fixtures_tldraw_edita");
const require = createRequire(path.join(APP, "package.json"));
const { chromium } = require("playwright");
const VERSAO = `tldraw ${require(path.join(APP, "node_modules/tldraw/package.json")).version} (tldraw license — corre-se, não se porta)`;

// As formas, em px de página (câmara 0,0 zoom 1 ⇒ página = ecrã), do tamanho das da cena 12.
const FORMAS = [
  ["rectangulo", { type: "geo", x: 450, y: 300, props: { geo: "rectangle", w: 300, h: 190 } }],
  ["rectangulo_rodado_25", { type: "geo", x: 500, y: 250, rotation: 25 * Math.PI / 180, props: { geo: "rectangle", w: 300, h: 190 } }],
  ["circulo", { type: "geo", x: 480, y: 280, props: { geo: "ellipse", w: 220, h: 220 } }],
  ["elipse", { type: "geo", x: 420, y: 300, props: { geo: "ellipse", w: 360, h: 180 } }],
  ["triangulo", { type: "geo", x: 480, y: 280, props: { geo: "triangle", w: 260, h: 220 } }],
  ["linha", { type: "line", x: 380, y: 400, props: { points: { a1: { id: "a1", index: "a1", x: 0, y: 0 }, a2: { id: "a2", index: "a2", x: 400, y: -120 } } } }],
  ["polilinha", { type: "line", x: 380, y: 450, props: { points: { a1: { id: "a1", index: "a1", x: 0, y: 0 }, a2: { id: "a2", index: "a2", x: 200, y: -200 }, a3: { id: "a3", index: "a3", x: 400, y: 0 } } } }],
  ["seta", { type: "arrow", x: 0, y: 0, props: { start: { x: 380, y: 420 }, end: { x: 780, y: 380 }, arrowheadEnd: "arrow" } }],
  ["seta_curva", { type: "arrow", x: 0, y: 0, props: { start: { x: 380, y: 450 }, end: { x: 780, y: 450 }, bend: 120, arrowheadEnd: "arrow" } }],
];
const ARRASTO = [40, 25];

const WWW = path.join(APP, "www");
const MIME = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".woff2": "font/woff2" };
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
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await ctx.route("**/*", (r) => (r.request().url().startsWith(origin) ? r.continue() : r.abort()));
  const page = await ctx.newPage();
  await page.goto(origin + "/");
  await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 30000 });

  // Repõe a forma (apaga tudo, cria, seleciona) e devolve-a lida + as alças em coordenadas de página.
  const repoe = (spec) => page.evaluate((spec) => {
    const { editor, createShapeId } = window.oracle;
    editor.setCurrentTool("select");
    editor.deleteShapes([...editor.getCurrentPageShapeIds()]);
    editor.setCamera({ x: 0, y: 0, z: 1 });
    const id = createShapeId("f");
    editor.createShapes([{ id, ...spec }]);
    editor.select(id);
    return id;
  }, spec);
  const le = (id) => page.evaluate((id) => {
    const { editor } = window.oracle;
    const s = editor.getShape(id);
    const r = (v) => Math.round(v * 1000) / 1000;
    const tr = editor.getShapePageTransform(id);
    const hs = (editor.getShapeHandles(s) || []).map((h) => {
      const p = tr.applyToPoint({ x: h.x, y: h.y });
      return { id: h.id, type: h.type, x: r(p.x), y: r(p.y) };
    });
    const b = editor.getSelectionRotatedPageBounds();
    const rot = editor.getSelectionRotation();
    const caixa = b ? { x: r(b.x), y: r(b.y), w: r(b.w), h: r(b.h), rot: r(rot) } : null;
    // A rotação vai INTEIRA: arredondada a 1e-3 rad ela erra 0,02 px num lado de 300 (medido).
    return { shape: JSON.parse(JSON.stringify({ x: r(s.x), y: r(s.y), rotation: s.rotation, props: s.props })), alcas: hs, caixa };
  }, id);
  const arrasta = async ([x0, y0], [dx, dy], tecla) => {
    await page.mouse.move(x0, y0);
    await page.waitForTimeout(30);
    if (tecla) await page.keyboard.down(tecla);
    await page.mouse.down();
    for (let k = 1; k <= 8; k++) await page.mouse.move(x0 + dx * k / 8, y0 + dy * k / 8);
    await page.mouse.up();
    if (tecla) await page.keyboard.up(tecla);
    await page.waitForTimeout(80);
  };
  // Os cantos e os meios dos lados da caixa rodada (as alças de redimensionar que o tldraw pinta).
  const cantos = (c) => {
    const [cs, sn] = [Math.cos(c.rot), Math.sin(c.rot)];
    const at = (u, v) => [c.x + u * cs - v * sn, c.y + u * sn + v * cs];
    return [
      ["nw", at(0, 0)], ["ne", at(c.w, 0)], ["se", at(c.w, c.h)], ["sw", at(0, c.h)],
      ["n", at(c.w / 2, 0)], ["e", at(c.w, c.h / 2)], ["s", at(c.w / 2, c.h)], ["w", at(0, c.h / 2)],
    ];
  };

  for (const [nome, spec] of FORMAS) {
    const id = await repoe(spec);
    const base = await le(id);
    const fix = {
      _cabecalho: {
        oraculo: VERSAO, forma: nome, data: new Date().toISOString().slice(0, 10),
        como: "Editor montado 1280×800, câmara 0,0 zoom 1; forma criada pela API e seleccionada; cada alça arrastada (+40,+25) pelo rato do Playwright, com e sem Shift, e a forma reposta entre arrastos",
        unidade: "px de página = px de ecrã",
      },
      base,
      arrastos: {},
    };
    const alvos = spec.type === "geo"
      ? cantos(base.caixa)
      : base.alcas.map((h) => [`${h.type}:${h.id}`, [h.x, h.y]]);
    for (const [k, c] of alvos) {
      for (const [rot, tecla] of [["livre", null], ["shift", "Shift"]]) {
        const id2 = await repoe(spec);
        await arrasta(c, ARRASTO, tecla);
        const depois = await le(id2);
        const n = await page.evaluate(() => window.oracle.editor.getCurrentPageShapeIds().size);
        fix.arrastos[`${k}/${rot}`] = { em: c, largado_em: [c[0] + ARRASTO[0], c[1] + ARRASTO[1]], formas: n, ...depois };
      }
    }
    fs.writeFileSync(path.join(SAI, nome + ".json"), JSON.stringify(fix, null, 1) + "\n");
    console.log(`${nome.padEnd(22)} alças ${alvos.map(([k]) => k).join(",")}`);
  }
} finally {
  await browser.close();
  server.close();
}
