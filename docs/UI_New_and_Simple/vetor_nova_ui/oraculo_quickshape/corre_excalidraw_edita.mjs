// O ORÁCULO DO «EDIT SHAPE» (F7e) — o Excalidraw `next` editado à MÃO SIMULADA: o traço entra pela
// ferramenta de reconhecimento do EDITOR montado (`autoshape`, o «draw shape»), e depois cada alça que
// a forma reconhecida mostra é arrastada com o rato. Grava: o que o editor faz ao soltar (ferramenta,
// seleção), a forma nascida, as alças que ela tem e o que cada arrasto faz à forma.
// MIT, corre-se e não se lê: só a API pública (`setActiveTool`, `getSceneElements`, `updateScene`,
// `getTransformHandles` de `@excalidraw/element` para SABER onde estão as alças) e o rato do Playwright.
//
// uso:  PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh timeout 600 node <este> [saida_dir] [entrada...]
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const AQUI = path.dirname(new URL(import.meta.url).pathname);
const APP = path.join(os.homedir(), "Apps/excalidraw-oracle-next");
const SAI = process.argv[2] || path.join(AQUI, "fixtures_excalidraw_edita");
const PADRAO = [
  "rect_alinhado_400x250", "rect_rodado_25", "quadrado_300", "circulo_r150", "elipse_2para1_rodada_30",
  "poligono_3_lados", "trapezio", "linha_diagonal_500", "forma_L", "arco_meio",
  "seta_recta_v_aberto", "seta_curva_v_aberto", "seta_recta_duas_pontas",
];
const NOMES = process.argv.length > 3 ? process.argv.slice(3) : PADRAO;
const require = createRequire(path.join(APP, "package.json"));
const { chromium } = require("playwright");
const EL = await import(pathToFileURL(path.join(APP, "node_modules/@excalidraw/element/dist/prod/index.js")).href);
const pkg = (p) => JSON.parse(fs.readFileSync(path.join(APP, "node_modules", p, "package.json"), "utf8"));
const VERSAO = `@excalidraw/excalidraw ${pkg("@excalidraw/excalidraw").version} (${pkg("@excalidraw/excalidraw").license})`;

// As entradas são px de uma tela 1600×1000; o editor da página tem 1280×800 ⇒ escala 0,75 e
// a margem de cima livre da barra de ferramentas.
const ESCALA = 0.75;
const DY = 30;
const le = (nome) =>
  fs.readFileSync(path.join(AQUI, "entradas", nome + ".txt"), "utf8").split("\n")
    .filter((l) => l.trim() && !l.startsWith("#"))
    .map((l) => l.trim().split(/\s+/).map(Number))
    .map(([x, y]) => [x * ESCALA, y * ESCALA + DY]);

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

const r3 = (v) => Math.round(v * 1000) / 1000;
const forma = (e) => {
  if (!e) return null;
  const o = { type: e.type, x: r3(e.x), y: r3(e.y), w: r3(e.width), h: r3(e.height), angle: r3(e.angle) };
  if (e.points) o.points = e.points.map(([x, y]) => [r3(e.x + x), r3(e.y + y)]);
  if ("startArrowhead" in e) Object.assign(o, { startArrowhead: e.startArrowhead, endArrowhead: e.endArrowhead });
  if (e.roundness !== undefined) o.roundness = e.roundness;
  return o;
};

const browser = await chromium.launch({ headless: true });
fs.mkdirSync(SAI, { recursive: true });
try {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await ctx.route("**/*", (r) => (r.request().url().startsWith(origin) ? r.continue() : r.abort()));
  const page = await ctx.newPage();
  await page.goto(origin + "/");
  await page.waitForFunction(() => window.oracleReady === true);
  await page.evaluate(() => window.oracle.mount([]));

  const cena = () => page.evaluate(() => window.oracle.scene().filter((e) => !e.isDeleted));
  const estado = () => page.evaluate(() => window.oracle.appState());
  const repoe = (els, sel) => page.evaluate(([els, sel]) => {
    const api = window.oracle.editorApi();
    api.updateScene({ elements: els, appState: { selectedElementIds: Object.fromEntries(sel.map((i) => [i, true])), selectedGroupIds: {} } });
  }, [els, sel]);
  const ferramenta = (type) => page.evaluate((type) => window.oracle.editorApi().setActiveTool({ type }), type);
  const arrasta = async ([x0, y0], [dx, dy], tecla) => {
    await page.mouse.move(x0, y0);
    if (tecla) await page.keyboard.down(tecla);
    await page.mouse.down();
    await page.mouse.move(x0 + dx / 2, y0 + dy / 2, { steps: 4 });
    await page.mouse.move(x0 + dx, y0 + dy, { steps: 4 });
    await page.mouse.up();
    if (tecla) await page.keyboard.up(tecla);
    await page.waitForTimeout(120);
  };

  for (const nome of NOMES) {
    await repoe([], []);
    await ferramenta("autoshape");
    const pts = le(nome);
    await page.mouse.move(...pts[0]);
    await page.mouse.down();
    for (const p of pts.slice(1)) await page.mouse.move(...p);
    await page.mouse.up();
    await page.waitForTimeout(300);
    const depois = await estado();
    const els = await cena();
    const el = els[els.length - 1];
    const fix = {
      _cabecalho: {
        oraculo: VERSAO, entrada: nome, data: new Date().toISOString().slice(0, 10),
        como: "rato do Playwright no <Excalidraw> montado (1280×800, zoom 1); traço pela ferramenta autoshape; cada alça arrastada a partir da forma nascida, reposta entre arrastos",
        unidade: `px da página = px da entrada × ${ESCALA} (+${DY} em y)`,
      },
      ao_soltar: { ferramenta: depois.activeTool, selecionados: depois.selected.length, elementos: els.length },
      nascida: forma(el),
      alcas: {},
    };
    if (!el) { fs.writeFileSync(path.join(SAI, nome + ".json"), JSON.stringify(fix, null, 1)); continue; }
    const orig = els;
    // Que alças: as de transformação (o editor pinta-as quando a forma está selecionada) e, nas
    // lineares, os pontos e os meios dos segmentos.
    const map = new Map(els.map((e) => [e.id, e]));
    const th = EL.getTransformHandles(el, { value: 1 }, map, "mouse");
    const alvos = Object.entries(th).filter(([, b]) => b).map(([k, [x, y, w, h]]) => [k, [x + w / 2, y + h / 2]]);
    if (el.points) {
      el.points.forEach(([px, py], i) => alvos.push([`ponto_${i}`, [el.x + px, el.y + py]]));
      for (let i = 0; i + 1 < el.points.length; i++) {
        const [a, b] = [el.points[i], el.points[i + 1]];
        alvos.push([`meio_${i}`, [el.x + (a[0] + b[0]) / 2, el.y + (a[1] + b[1]) / 2]]);
      }
    }
    for (const [k, c] of alvos) {
      for (const [rot, tecla] of [["livre", null], ["shift", "Shift"]]) {
        await repoe(orig, [el.id]);
        await ferramenta("selection");
        // Um clique PARADO primeiro: a seleção pela API não arma o editor de pontos das lineares
        // (medido: sem ele o 1.º ponto arrastava a linha inteira).
        await page.mouse.click(...c);
        await page.waitForTimeout(80);
        await arrasta(c, [40, 25], tecla);
        const novos = await cena();
        const mesmo = novos.find((e) => e.id === el.id);
        fix.alcas[`${k}/${rot}`] = {
          em: c.map(r3), arrasto: [40, 25],
          elementos: novos.length, forma: forma(mesmo),
        };
      }
    }
    // A entrada do Edit Shape: com a ferramenta de reconhecimento AINDA na mão, um arrasto que começa
    // na alça de canto edita a forma ou desenha outra?
    {
      await repoe(orig, [el.id]);
      await ferramenta("autoshape");
      const canto = alvos.find(([k]) => k === "se") || alvos.find(([k]) => k.startsWith("ponto_"));
      if (canto) {
        await arrasta(canto[1], [40, 25], null);
        const novos = await cena();
        fix.com_a_ferramenta_na_mao = {
          alca: canto[0], elementos: novos.length,
          forma: forma(novos.find((e) => e.id === el.id)),
        };
      }
    }
    fs.writeFileSync(path.join(SAI, nome + ".json"), JSON.stringify(fix, null, 1) + "\n");
    console.log(`${nome.padEnd(26)} ${String(el.type).padEnd(10)} solta→${depois.activeTool}/${depois.selected.length} alças ${alvos.map(([k]) => k).join(",")}`);
  }
} finally {
  await browser.close();
  server.close();
}
