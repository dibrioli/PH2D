// W23 bloco 6 — o LAÇO e o «Draw to shape» do Excalidraw (canal `next`), corridos pelo RATO real no editor
// montado (a página `www_w9` do instala_w9.sh; zoom 1, scroll 0 ⇒ ecrã = cena). Só a API pública e a UI:
// as ferramentas armam-se pelos botões que o editor DESENHA (data-testid/título lidos do DOM vivo).
//
// Uso:
//   node corre_bloco6.mjs laco  entradas/bloco6_laco.json  <pasta_saida> [--so <caso>] [--armar menu|selecao|ctrlalt] [--dt ms] [--relogio fixo|livre] [--fotos <dir>] [--porta n] [--sufixo s]
//   node corre_bloco6.mjs traco entradas/bloco6_traco.json <pasta_saida> [--so <traço>] [--dt ms] [--porta n]
// Saídas: <pasta_saida>/bloco6_laco.next[.<sufixo>].json · <pasta_saida>/bloco6_traco.next.json (cabeçalho:
// versão e licença do package.json instalado, sha256 da entrada, data, semente).
//
// LAÇO (por caso × modo de selecção `boxSelectionMode` contain|overlap, posto por updateScene — a UI do pacote
// não o expõe): monta os elementos, arma o laço, carrega no 1.º ponto do rasto, passa por todos, larga.
//   armar = menu     «More tools» → «Lasso selection» (data-testid toolbar-lasso)
//           selecao  clicar no botão «Selection» quando ele JÁ é a ferramenta activa (alterna para o laço)
//           ctrlalt  a ferramenta Selection com Ctrl+Alt carregados durante o arrasto
// TRAÇO (por traço × ferramenta): autoshape = «More tools» → «Draw to shape» (data-testid toolbar-autoshape);
//   freedraw = o botão «Draw» (o controlo). Toca os pontos com o rato, larga, espera a cena parar.
// Determinismo: Math.random/crypto semeados pelo sha256 (entrada + nome), ids novos normalizados (novo1…).
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const HOME = process.env.ORACLE_NEXT_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle-next");
const FIXED_TIME = Date.UTC(2026, 0, 1, 0, 0, 0);
let args = process.argv.slice(2);
const opt = (flag, def = null) => {
  const i = args.indexOf(flag);
  if (i < 0) return def;
  const v = args[i + 1];
  args.splice(i, 2);
  return v;
};
const so = opt("--so");
const armar = opt("--armar", "menu");
const dtOpt = opt("--dt");
const relogio = opt("--relogio", "fixo");
const fotos = opt("--fotos");
const porta = Number(opt("--porta", "47163"));
const sufixo = opt("--sufixo");
const [modo, entradaPath, outDir] = args;
if (!["laco", "traco"].includes(modo) || !entradaPath || !outDir) {
  console.error("uso: node corre_bloco6.mjs laco|traco <entrada.json> <pasta_saida> [opções]");
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });
if (fotos) fs.mkdirSync(fotos, { recursive: true });
const entBytes = fs.readFileSync(entradaPath);
const sha = (b) => createHash("sha256").update(b).digest("hex");
const ENT = JSON.parse(entBytes.toString("utf8"));
const [TW, TH] = ENT.tela;

const seedScript = (s) => `(() => {
  let a = ${s} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next;
  const gr = (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength);
    for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; };
  Object.defineProperty(crypto, "getRandomValues", { value: gr });
})();`;

const WWW = path.join(HOME, "www_w9");
if (!fs.existsSync(path.join(WWW, "oracle.js"))) {
  console.error(`página W9 não construída em ${WWW}: corra instala_w9.sh`);
  process.exit(2);
}
const require = createRequire(path.join(HOME, "package.json"));
const { chromium } = require("playwright");
const PKG = JSON.parse(fs.readFileSync(path.join(HOME, "node_modules/@excalidraw/excalidraw/package.json"), "utf8"));
const MIME = { ".html": "text/html", ".js": "text/javascript", ".woff2": "font/woff2", ".ttf": "font/ttf", ".wasm": "application/wasm", ".css": "text/css" };
const server = createServer((req, res) => {
  const p = path.normalize(decodeURIComponent(new URL(req.url, "http://x").pathname));
  const file = path.join(WWW, p === "/" ? "index.html" : p);
  if (!file.startsWith(WWW) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404).end(); return; }
  res.writeHead(200, { "content-type": MIME[path.extname(file)] || "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(porta, "127.0.0.1", r));
const origin = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch({ headless: true });

// Uma página nova com o editor montado na tela da entrada (o host do oracle_w9 nasce 1280×800: estica-se).
async function abre(nome, elementos, relogioCaso) {
  const seed = parseInt(sha(Buffer.concat([entBytes, Buffer.from(nome)])).slice(0, 8), 16);
  const ctx = await browser.newContext({ viewport: { width: TW, height: TH }, deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC" });
  await ctx.route("**/*", (route) => {
    const u = route.request().url();
    if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue();
    return route.abort();
  });
  await ctx.addInitScript(seedScript(seed));
  const page = await ctx.newPage();
  if (relogioCaso === "fixo") await page.clock.setFixedTime(FIXED_TIME);
  const consola = [];
  page.on("pageerror", (e) => consola.push(`pageerror: ${e.message}`));
  page.on("console", (m) => { if (m.type() === "error") consola.push(`error: ${m.text()}`); });
  await page.goto(origin + "/", { timeout: 60000 });
  await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 60000 });
  const restaurados = await page.evaluate((els) => window.oracle.restoreEls(els), elementos);
  await page.evaluate((els) => window.oracle.mount(els), restaurados);
  await page.evaluate(([w, h]) => {
    const host = [...document.body.children].find((d) => d.tagName === "DIV" && d.style.position === "fixed");
    host.style.width = `${w}px`;
    host.style.height = `${h}px`;
    window.dispatchEvent(new Event("resize"));
  }, [TW, TH]);
  await page.waitForTimeout(400);
  await page.evaluate(() => window.oracle.editorApi().updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: 1 } } }));
  await page.waitForTimeout(200);
  const tela = await page.evaluate(() => { const a = window.oracle.editorApi().getAppState(); return { w: a.width, h: a.height, scroll: [a.scrollX, a.scrollY], zoom: a.zoom.value, offset: [a.offsetLeft, a.offsetTop] }; });
  return { ctx, page, consola, seed, tela };
}

const ferramenta = (page) => page.evaluate(() => window.oracle.editorApi().getAppState().activeTool.type);
const clicaTestid = async (page, id) => { await page.click(`[data-testid=${id}]`); await page.waitForTimeout(150); };
async function maisFerramentas(page, id) {
  await clicaTestid(page, "dropdown-menu-button");
  await clicaTestid(page, id);
}
// O rato percorre `pts` (cena = ecrã) com `dt` ms entre pontos; `meio(i)` corre a meio do caminho.
async function arrasta(page, pts, dt, meio) {
  await page.mouse.move(pts[0][0], pts[0][1]);
  await page.mouse.down();
  const h = Math.floor(pts.length / 2);
  for (let i = 1; i < pts.length; i++) {
    await page.mouse.move(pts[i][0], pts[i][1]);
    if (dt) await page.waitForTimeout(dt);
    if (i === h && meio) await meio();
  }
}
const cabecalho = (extra) => ({
  oraculo: "@excalidraw/excalidraw", versao: PKG.version, licenca: PKG.license, chromium: browser.version(),
  entrada: path.basename(entradaPath), entrada_sha256: sha(entBytes), data: new Date().toISOString().slice(0, 10),
  tela: ENT.tela, ...extra,
});

let falhas = 0;
try {
  if (modo === "laco") {
    const dt = Number(dtOpt ?? 4);
    const resultados = [];
    for (const caso of ENT.casos) {
      if (so && caso.nome !== so) continue;
      for (const sel of ENT.modos) {
        const { ctx, page, consola, seed, tela } = await abre(caso.nome, caso.elementos, relogio);
        const r = { caso: caso.nome, boxSelectionMode: sel, armar, tela, semente: seed };
        try {
          await page.evaluate((m) => window.oracle.editorApi().updateScene({ appState: { boxSelectionMode: m } }), sel);
          await page.mouse.click(TW - 60, TH / 2); // foco no canvas, sem nada debaixo
          await page.keyboard.press("Escape");
          const cena0 = await page.evaluate(() => window.oracle.scene());
          r.ferramenta_antes = await ferramenta(page);
          if (armar === "menu") await maisFerramentas(page, "toolbar-lasso");
          else if (armar === "selecao") await clicaTestid(page, "toolbar-selection");
          r.ferramenta_armada = await ferramenta(page);
          const mods = armar === "ctrlalt" ? ["Control", "Alt"] : [];
          for (const m of mods) await page.keyboard.down(m);
          await arrasta(page, caso.laco, dt, async () => {
            r.a_meio = await page.evaluate(() => { const a = window.oracle.editorApi().getAppState(); return { ferramenta: a.activeTool.type, seleccionados: Object.keys(a.selectedElementIds).sort(), selectionElement: a.selectionElement ? a.selectionElement.type : null }; });
          });
          r.antes_de_largar = await page.evaluate(() => { const a = window.oracle.editorApi().getAppState(); return { ferramenta: a.activeTool.type, seleccionados: Object.keys(a.selectedElementIds).sort() }; });
          if (fotos) await page.screenshot({ path: path.join(fotos, `bloco6_laco_${caso.nome}.${sel}.${armar}.png`) });
          await page.mouse.up();
          for (const m of mods) await page.keyboard.up(m);
          await page.waitForTimeout(300);
          const fim = await page.evaluate(() => { const a = window.oracle.editorApi().getAppState(); return { ferramenta: a.activeTool.type, boxSelectionMode: a.boxSelectionMode, selectedElementIds: Object.keys(a.selectedElementIds).sort(), selectedGroupIds: Object.keys(a.selectedGroupIds || {}).filter((k) => a.selectedGroupIds[k]).sort() }; });
          Object.assign(r, fim);
          const ids = caso.elementos.map((e) => e.id);
          r.por_elemento = Object.fromEntries(ids.map((id) => [id, { seleccionado: fim.selectedElementIds.includes(id), fracao_dentro: caso.fracao_dentro[id] ?? null }]));
          const cena = await page.evaluate(() => window.oracle.scene());
          r.cena_mexeu = cena.some((e) => { const o = cena0.find((x) => x.id === e.id); return !o || Math.abs(o.x - e.x) > 1e-9 || Math.abs(o.y - e.y) > 1e-9; });
          r.elementos_na_cena = cena.length;
        } catch (e) {
          r.erro = e.message;
          falhas++;
        }
        r.consola = consola;
        await ctx.close();
        resultados.push(r);
        console.log(`laco ${caso.nome} ${sel} [${armar}] → ${r.erro ? "ERRO " + r.erro : (r.selectedElementIds || []).join(",") || "(nada)"} · ${r.ferramenta_armada}`);
      }
    }
    const nome = `bloco6_laco.next${sufixo ? "." + sufixo : ""}.json`;
    const cab = cabecalho({ armar, dt_ms: dt, relogio: relogio === "fixo" ? new Date(FIXED_TIME).toISOString() : "livre", modos: ENT.modos, nota_modos: "boxSelectionMode ('contain' | 'overlap', tipo BoxSelectionMode do .d.ts público) posto por updateScene; a UI do pacote não tem preferência para ele" });
    fs.writeFileSync(path.join(outDir, nome), JSON.stringify({ cabecalho: cab, resultados }, null, 1) + "\n");
    console.log(`→ ${path.join(outDir, nome)}`);
  } else {
    const dt = Number(dtOpt ?? 16);
    const API = path.resolve(path.dirname(entradaPath), "../../../../UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/fixtures_excalidraw");
    const tipoApi = (nome) => {
      const f = path.join(API, `${nome}.txt`);
      if (!fs.existsSync(f)) return null;
      const l = fs.readFileSync(f, "utf8").split("\n").find((x) => x.startsWith("# tipo:"));
      return l ? l.split(":")[1].trim() : null;
    };
    const resultados = [];
    for (const tool of ENT.ferramentas) {
      const { ctx, page, consola, seed, tela } = await abre(`traco_${tool}`, [], "livre");
      let estado = null;
      for (const t of ENT.tracos) {
        if (so && t.nome !== so) continue;
        const r = { traco: t.nome, ferramenta: tool, transformacao: t.transformacao };
        try {
          await page.evaluate(() => window.oracle.editorApi().updateScene({ elements: [], appState: { selectedElementIds: {}, selectedGroupIds: {} } }));
          await page.keyboard.press("Escape");
          await page.waitForTimeout(100);
          if (tool === "autoshape") await maisFerramentas(page, "toolbar-autoshape");
          else await clicaTestid(page, "toolbar-freedraw");
          r.ferramenta_armada = await page.evaluate(() => window.oracle.editorApi().getAppState().activeTool);
          if (!estado) estado = await page.evaluate(() => { const a = window.oracle.editorApi().getAppState(); return Object.fromEntries(Object.keys(a).filter((k) => k.startsWith("currentItem")).sort().map((k) => [k, a[k]])); });
          await arrasta(page, t.pontos, dt);
          await page.mouse.up();
          // a cena pode mudar depois de largar (o reconhecimento): espera-se que pare 3 leituras seguidas
          let prev = "", estavel = 0, ms = 0;
          while (estavel < 3 && ms < 5000) {
            await page.waitForTimeout(150);
            ms += 150;
            const s = await page.evaluate(() => JSON.stringify(window.oracle.editorApi().getSceneElements().map((e) => [e.type, e.x, e.y, e.width, e.height, e.angle, e.points?.length])));
            estavel = s === prev ? estavel + 1 : 0;
            prev = s;
          }
          r.ms_ate_parar = ms;
          const cena = await page.evaluate(() => window.oracle.scene());
          const mapa = new Map(cena.map((e, i) => [e.id, `novo${i + 1}`]));
          r.elementos = cena.map((e) => {
            const o = { id: mapa.get(e.id), type: e.type, x: e.x, y: e.y, width: e.width, height: e.height, angle: e.angle, roundness: e.roundness ?? null };
            if (e.points) {
              o.n_pontos = e.points.length;
              if (e.type === "freedraw") { o.primeiro = e.points[0]; o.ultimo = e.points[e.points.length - 1]; }
              else o.points = e.points;
            }
            if ("startArrowhead" in e) Object.assign(o, { startArrowhead: e.startArrowhead, endArrowhead: e.endArrowhead });
            return o;
          });
          r.tipo = r.elementos.map((e) => e.type).join("+") || "nenhum";
          r.tipo_api_recognizeShape = tool === "autoshape" ? tipoApi(t.nome) : undefined;
          r.ferramenta_depois = await ferramenta(page);
        } catch (e) {
          r.erro = e.message;
          falhas++;
        }
        resultados.push(r);
        console.log(`traco ${tool.padEnd(9)} ${t.nome.padEnd(28)} → ${r.erro ? "ERRO " + r.erro : r.tipo}${r.tipo_api_recognizeShape && r.tipo_api_recognizeShape !== r.tipo ? `  (API: ${r.tipo_api_recognizeShape})` : ""}`);
      }
      resultados.push({ ferramenta: tool, estado_currentItem: estado, semente: seed, tela, consola });
      await ctx.close();
    }
    const cab = cabecalho({ dt_ms: dt, relogio: "livre (ids novos normalizados por posição na cena)", ferramentas: ENT.ferramentas });
    fs.writeFileSync(path.join(outDir, "bloco6_traco.next.json"), JSON.stringify({ cabecalho: cab, resultados }, null, 1) + "\n");
    console.log(`→ ${path.join(outDir, "bloco6_traco.next.json")}`);
  }
} finally {
  await browser.close();
  server.close();
}
if (falhas) process.exit(1);
