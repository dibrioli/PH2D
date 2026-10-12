// W16 — a IMAGEM no EDITOR montado do Excalidraw 0.18.1: o recorte pelo gesto (A), o espelho (B) e o
// GIF animado (C). Só API pública (excalidrawAPI: addFiles, updateScene, getSceneElements, getAppState,
// getFiles; exportToSvg/exportToBlob) e o rato/teclado do Playwright — o fonte do Excalidraw não se lê.
// Uso: node sonda_imagem_w16.mjs [--so <prefixo>[,<prefixo>…]] [<pasta_entradas> <pasta_saida>]
//   (por omissão entradas/ e saidas/w16/ ao lado deste script; as entradas geram-se por
//    `python3 -I entradas/gera_w16_imagens.py`).
// Cada SONDA monta a cena de novo (janela 1280×800, zoom 1, scroll 0 ⇒ cena = ecrã), acrescenta o
// ficheiro pelo addFiles e espera a imagem aparecer no canvas. Depois de CADA passo grava o elemento
// (x, y, w, h, ângulo, scale, crop, status), o appState relevante (chaves com «crop», selecção, …) e
// TODAS as chaves do appState que mudaram desde o passo anterior, o cursor do canvas, uma captura
// recortada (CLIP) e, sobre ela, os píxeis em linhas/colunas fixas (corridas de cor: onde a imagem
// está no ecrã e com que alfa). As pegas acham-se pelo CURSOR (amostras ao longo da diagonal/normal).
// ⚠️ Relógio LIVRE (o duplo clique do editor precisa do tempo a andar); PRNG semeado pelas entradas.
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle");
let args = process.argv.slice(2);
const opt = (flag) => { const i = args.indexOf(flag); if (i < 0) return null; const v = args[i + 1]; args.splice(i, 2); return v; };
const so = opt("--so");
const entDir = args[0] || path.join(HERE, "entradas");
const outDir = args[1] || path.join(HERE, "saidas/w16");
fs.mkdirSync(outDir, { recursive: true });

const ler = (n) => { const b = fs.readFileSync(path.join(entDir, n)); return { nome: n, b64: b.toString("base64"), sha256: createHash("sha256").update(b).digest("hex"), bytes: b.length }; };
const PNG = ler("w16_quadrantes.png");
const GIF = ler("w16_anim.gif");
const SCRIPT_SHA = createHash("sha256").update(fs.readFileSync(fileURLToPath(import.meta.url))).digest("hex");
const r3 = (v) => (typeof v === "number" ? Math.round(v * 1000) / 1000 : v);
const loadavg = () => os.loadavg().map((v) => r3(v));

// ---- geometria da cena (zoom 1, scroll 0 ⇒ cena = ecrã; o editor está em (0,0)) ----
const IMG = { x: 420, y: 180, w: 400, h: 300 }; // natural 400×300, 1:1 ⇒ px de crop = px de ecrã
const CLIP = { x: 300, y: 70, width: 680, height: 550 };
const FORA = [1150, 700]; // vazio do canvas
const base = (over = {}) => ({
  id: "img", type: "image", x: IMG.x, y: IMG.y, width: IMG.w, height: IMG.h, angle: 0,
  strokeColor: "transparent", backgroundColor: "transparent", fillStyle: "solid", strokeWidth: 2,
  strokeStyle: "solid", roughness: 1, opacity: 100, groupIds: [], frameId: null, roundness: null,
  seed: 1, version: 1, versionNonce: 1, isDeleted: false, boundElements: null, updated: 1, link: null,
  locked: false, status: "saved", fileId: "w16png", scale: [1, 1], crop: null, ...over,
});
const FILE_PNG = { id: "w16png", mimeType: "image/png", dataURL: "data:image/png;base64," + PNG.b64, created: 1 };
const FILE_GIF = { id: "w16gif", mimeType: "image/gif", dataURL: "data:image/gif;base64," + GIF.b64, created: 1 };

// Cantos/meios da caixa (rodada) no ecrã e a direcção «para fora» de cada um.
const PEGAS = { nw: [-1, -1], n: [0, -1], ne: [1, -1], e: [1, 0], se: [1, 1], s: [0, 1], sw: [-1, 1], w: [-1, 0] };
const pegaGeo = (e, k) => {
  const [sx, sy] = PEGAS[k];
  const cx = e.x + e.width / 2, cy = e.y + e.height / 2, c = Math.cos(e.angle || 0), s = Math.sin(e.angle || 0);
  const dx = (sx * e.width) / 2, dy = (sy * e.height) / 2;
  const n = Math.hypot(sx, sy);
  return { p: [cx + dx * c - dy * s, cy + dx * s + dy * c], dir: [(sx * c - sy * s) / n, (sx * s + sy * c) / n] };
};
const rodar = ([x, y], th) => [x * Math.cos(th) - y * Math.sin(th), x * Math.sin(th) + y * Math.cos(th)];

// ---- servidor local da página (o mesmo dos outros corredores) ----
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const PKG = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/@excalidraw/excalidraw/package.json"), "utf8"));
const PW = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/playwright/package.json"), "utf8"));
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
const SEED = parseInt(createHash("sha256").update(PNG.sha256 + GIF.sha256).digest("hex").slice(0, 8), 16);
const seedScript = `(() => {
  let a = ${SEED} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next;
  const gr = (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength);
    for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; };
  Object.defineProperty(crypto, "getRandomValues", { value: gr });
})();`;

// ---- lado da página ----
const pageMount = async ({ els, files, zoom }) => {
  const { restoreElements } = window.oracle.api;
  window.oracle.editorUnmount();
  const elements = restoreElements(els, null, { refreshDimensions: false, repairBindings: true });
  await window.oracle.editorMount(elements);
  const api = window.oracle.editorApi();
  api.addFiles(files);
  api.updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: zoom || 1 } } });
  return true;
};
const pageSnap = () => {
  const api = window.oracle.editorApi();
  const a = api.getAppState();
  const plain = {};
  for (const [k, v] of Object.entries(a)) {
    if (typeof v === "function") continue;
    try { plain[k] = JSON.parse(JSON.stringify(v ?? null)); } catch { plain[k] = String(v); }
  }
  return {
    elementos: api.getSceneElements().map((e) => ({
      id: e.id, type: e.type, x: e.x, y: e.y, width: e.width, height: e.height, angle: e.angle,
      scale: e.scale, crop: e.crop ?? null, status: e.status, fileId: e.fileId, version: e.version, isDeleted: e.isDeleted,
    })),
    appState: plain,
    cursor: document.querySelector("canvas.interactive")?.style.cursor ?? null,
    dica: document.querySelector(".HintViewer")?.innerText ?? null,
  };
};
const pageCursor = () => document.querySelector("canvas.interactive")?.style.cursor ?? null;
const pageSelect = (ids) => {
  window.oracle.editorApi().updateScene({ appState: { selectedElementIds: Object.fromEntries(ids.map((id) => [id, true])) } });
};
const pageScroll = (sx) => { window.oracle.editorApi().updateScene({ appState: { scrollX: sx } }); };
const pageRaf = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
// Píxeis de um PNG (base64) em pontos e corridas de cor ao longo de linhas/colunas (coordenadas locais).
const pageDecode = async ({ b64, pts, linhas }) => {
  const blob = await (await fetch("data:image/png;base64," + b64)).blob();
  const bm = await createImageBitmap(blob);
  const c = new OffscreenCanvas(bm.width, bm.height);
  const g = c.getContext("2d");
  g.drawImage(bm, 0, 0);
  const d = g.getImageData(0, 0, bm.width, bm.height).data;
  const at = (x, y) => { const i = (y * bm.width + x) * 4; return [d[i], d[i + 1], d[i + 2], d[i + 3]]; };
  const pontos = pts.map(([x, y]) => ({ p: [x, y], rgba: at(x, y) }));
  const corr = linhas.map(({ nome, eixo, fixo, de, ate }) => {
    const out = [];
    for (let t = de; t <= ate; t++) {
      const v = eixo === "x" ? at(t, fixo) : at(fixo, t);
      const last = out[out.length - 1];
      if (last && last.rgba.every((q, k) => Math.abs(q - v[k]) <= 6)) last.ate = t;
      else out.push({ de: t, ate: t, rgba: v });
    }
    return { nome, eixo, fixo, corridas: out };
  });
  return { w: bm.width, h: bm.height, pontos, linhas: corr };
};
const pageUI = () => {
  const vis = (el) => { const r = el.getBoundingClientRect(); return r.width > 0 && r.height > 0; };
  const t = [...document.querySelectorAll("button, label, [role=button], input[type=radio]")]
    .filter(vis).map((el) => el.getAttribute("title") || el.getAttribute("aria-label") || "").filter(Boolean);
  return [...new Set(t)];
};
const pageMenu = () => {
  const m = document.querySelector(".context-menu");
  if (!m) return null;
  return [...m.querySelectorAll("li")].map((li) => {
    const lab = li.querySelector(".context-menu-item__label")?.innerText ?? li.innerText;
    const sc = li.querySelector(".context-menu-item__shortcut")?.innerText ?? "";
    return li.classList.contains("context-menu-item-separator") ? "—" : (lab + (sc ? `  [${sc}]` : "")).trim();
  });
};
const pageMenuClick = (re) => {
  const m = document.querySelector(".context-menu");
  if (!m) return null;
  const li = [...m.querySelectorAll("li")].find((q) => new RegExp(re, "i").test(q.innerText));
  if (!li) return null;
  (li.querySelector("button") || li).click();
  return li.innerText;
};
const pageExport = async () => {
  const api = window.oracle.editorApi();
  const { exportToSvg, exportToBlob } = window.oracle.api;
  const elements = api.getSceneElements(), files = api.getFiles();
  const appState = { exportBackground: true, viewBackgroundColor: "#ffffff", exportWithDarkMode: false };
  const svgEl = await exportToSvg({ elements, appState, files, exportPadding: 10 });
  const svg = new XMLSerializer().serializeToString(svgEl);
  const blob = await exportToBlob({ elements, appState, files, mimeType: "image/png", exportPadding: 10 });
  const fl = Object.fromEntries(Object.entries(files).map(([k, f]) => [k, { mimeType: f.mimeType, dataURL_prefixo: f.dataURL.slice(0, 40), dataURL_len: f.dataURL.length }]));
  return { svg, png: await window.oracle.blobToBase64(blob), files: fl };
};

// ---- corredor ----
const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
const external = [], consoleErrors = [];
await context.route("**/*", (route) => {
  const u = route.request().url();
  if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue();
  external.push(u);
  return route.abort();
});
await context.addInitScript(seedScript);
const page = await context.newPage();
page.setDefaultTimeout(120000);
page.on("console", (m) => { if (m.type() === "error") consoleErrors.push(m.text()); });
page.on("pageerror", (e) => consoleErrors.push(`pageerror: ${e.message}`));
await page.goto(origin + "/", { timeout: 120000 });
await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 120000 });

const settle = async (ms = 150) => { await page.evaluate(pageRaf); await page.waitForTimeout(ms); };
const assenta = async ([x, y]) => { await page.waitForTimeout(100); await page.mouse.move(x, y); await settle(100); };
const rd = (p) => [Math.round(p[0]), Math.round(p[1])];
async function arrasta(p0, p1, { shift = false } = {}) {
  p0 = rd(p0); p1 = rd(p1);
  await page.mouse.move(p0[0], p0[1]);
  await settle(60);
  await page.mouse.down();
  if (shift) await page.keyboard.down("Shift");
  await page.mouse.move(p0[0] + Math.sign(p1[0] - p0[0] || 1) * 2, p0[1] + Math.sign(p1[1] - p0[1] || 1) * 2);
  await page.mouse.move(p1[0], p1[1], { steps: 12 });
  await assenta(p1);
  await page.mouse.up();
  if (shift) await page.keyboard.up("Shift");
  await settle(200);
  return { de: p0, para: p1 };
}
const imgDe = async (id = "img") => (await page.evaluate(pageSnap)).elementos.find((e) => e.id === id);
// Acha a pega `k` pelo cursor: amostra ao longo da direcção «para fora» de −14 a +24 px.
async function achaPega(k, id = "img") {
  const e = await imgDe(id);
  const { p, dir } = pegaGeo(e, k);
  const amostras = [];
  for (let d = -14; d <= 24; d += 2) {
    const q = rd([p[0] + dir[0] * d, p[1] + dir[1] * d]);
    await page.mouse.move(q[0], q[1]);
    await page.waitForTimeout(15);
    amostras.push([d, await page.evaluate(pageCursor)]);
  }
  const rs = amostras.filter(([, c]) => /resize/.test(c || "")).map(([d]) => d);
  const d = rs.length ? rs[Math.floor(rs.length / 2)] : 0;
  return { k, canto: p.map(r3), dir: dir.map(r3), amostras, offsets_resize: rs, usado: d, ponto: rd([p[0] + dir[0] * d, p[1] + dir[1] * d]) };
}
// Arrasto CRU: N movimentos iguais, sem o passo de 2 px nem o movimento final repetido; lê o appState
// e o elemento ANTES de largar.
async function arrastaN(p0, p1, n) {
  p0 = rd(p0); p1 = rd(p1);
  await page.mouse.move(p0[0], p0[1]);
  await settle(60);
  await page.mouse.down();
  for (let i = 1; i <= n; i++) {
    await page.mouse.move(p0[0] + ((p1[0] - p0[0]) * i) / n, p0[1] + ((p1[1] - p0[1]) * i) / n);
    await settle(30);
  }
  await settle(100);
  const s = await page.evaluate(pageSnap);
  const a = Object.fromEntries(Object.entries(s.appState).filter(([k]) => RELEVANTE.test(k) || OUTRAS.includes(k)));
  await page.mouse.up();
  await settle(200);
  return { de: p0, para: p1, n_movimentos: n, antes_de_largar: { appState: a, elemento: s.elementos[0], cursor: s.cursor } };
}
async function arrastaPega(k, delta, opts = {}) {
  const pg = await achaPega(k, opts.id);
  const mv = await arrasta(pg.ponto, [pg.ponto[0] + delta[0], pg.ponto[1] + delta[1]], opts);
  return { pega: { k, canto: pg.canto, offsets_resize: pg.offsets_resize, cursores: [...new Set(pg.amostras.map(([, c]) => c))], usado: pg.usado }, delta, shift: !!opts.shift, ...mv };
}
const LINHAS = [
  { nome: "linha_y240", eixo: "x", fixo: 240 }, // quadrantes de cima (vermelho|verde)
  { nome: "linha_y420", eixo: "x", fixo: 420 }, // quadrantes de baixo (azul|preto)
  { nome: "coluna_x520", eixo: "y", fixo: 520 }, // quadrantes da esquerda (vermelho|azul)
  { nome: "coluna_x720", eixo: "y", fixo: 720 }, // quadrantes da direita (verde|preto)
];
const RELEVANTE = /crop/i;
const OUTRAS = ["selectedElementIds", "activeTool", "editingGroupId", "selectedElementsAreBeingDragged", "isResizing", "isRotating", "resizingElement", "newElement", "editingTextElement", "editingLinearElement", "openMenu", "openPopup", "contextMenu", "showHyperlinkPopup"];

let sondaAtual = null;
async function mede(nome, extra = {}, { foto = true } = {}) {
  await settle(150);
  const s = await page.evaluate(pageSnap);
  const prev = sondaAtual.prevApp;
  const mudou = {};
  if (prev) for (const k of new Set([...Object.keys(prev), ...Object.keys(s.appState)])) {
    const a = JSON.stringify(prev[k]), b = JSON.stringify(s.appState[k]);
    if (a !== b) mudou[k] = b && b.length > 300 ? b.slice(0, 300) + "…" : s.appState[k];
  }
  sondaAtual.prevApp = s.appState;
  const rel = {};
  for (const k of Object.keys(s.appState)) if (RELEVANTE.test(k) || OUTRAS.includes(k)) rel[k] = s.appState[k];
  const passo = { passo: nome, ...extra, elementos: s.elementos, appState: rel, appState_mudou: prev ? mudou : "(1.º passo)", cursor: s.cursor, dica: s.dica };
  if (foto) {
    const n = String(sondaAtual.passos.length).padStart(2, "0");
    const fname = `w16_${sondaAtual.nome}_${n}_${nome}.png`;
    const buf = await page.screenshot({ clip: CLIP });
    fs.writeFileSync(path.join(outDir, fname), buf);
    const linhas = LINHAS.map((l) => (l.eixo === "x"
      ? { nome: l.nome, eixo: "x", fixo: l.fixo - CLIP.y, de: 0, ate: CLIP.width - 1 }
      : { nome: l.nome, eixo: "y", fixo: l.fixo - CLIP.x, de: 0, ate: CLIP.height - 1 }));
    const px = await page.evaluate(pageDecode, { b64: buf.toString("base64"), pts: [], linhas });
    passo.foto = fname;
    // De volta a coordenadas de ECRÃ (= cena).
    passo.pixeis = px.linhas.map((l) => {
      const off = l.eixo === "x" ? CLIP.x : CLIP.y;
      return { nome: l.nome, corridas: l.corridas.map((c) => ({ de: c.de + off, ate: c.ate + off, n: c.ate - c.de + 1, rgb: c.rgba.slice(0, 3) })) };
    });
  }
  sondaAtual.passos.push(passo);
  return s;
}
async function monta(els, files, zoom = 1) {
  await page.mouse.move(5, 790);
  await page.evaluate(pageMount, { els, files, zoom });
  // Espera a imagem chegar ao canvas: o centro do 1.º elemento deixa de ser branco/cinzento.
  const e0 = els[0];
  const c = rd([(e0.x + e0.width / 4) * zoom, (e0.y + e0.height / 4) * zoom]);
  const t0 = Date.now();
  let rgb = null;
  while (Date.now() - t0 < 30000) {
    await settle(100);
    const buf = await page.screenshot({ clip: { x: c[0], y: c[1], width: 1, height: 1 } });
    rgb = (await page.evaluate(pageDecode, { b64: buf.toString("base64"), pts: [[0, 0]], linhas: [] })).pontos[0].rgba;
    const [r, g, b] = rgb;
    const cinza = Math.abs(r - g) < 12 && Math.abs(g - b) < 12 && r > 150;
    if (!cinza) break;
  }
  return { carregou_em_ms: Date.now() - t0, rgb_quarto: rgb };
}
const dbl = async (p) => { await page.mouse.dblclick(p[0], p[1]); await settle(250); };
const clique = async (p, o = {}) => { await page.mouse.click(p[0], p[1], o); await settle(200); };
const tecla = async (k) => { await page.keyboard.press(k); await settle(200); };
const CENTRO = [IMG.x + IMG.w / 2, IMG.y + IMG.h / 2];

// ---- as sondas ----
const SONDAS = [];
const sonda = (nome, pergunta, fn) => { const o = { nome, pergunta, fn }; SONDAS.push(o); return o; };

sonda("A01_entra", "Duplo clique no centro entra no modo recorte? O que muda no appState; que pegas existem (cursor); Enter com a imagem seleccionada também entra?", async () => {
  await mede("montada");
  await dbl(CENTRO);
  const pegas = {};
  for (const k of Object.keys(PEGAS)) pegas[k] = await achaPega(k);
  await page.mouse.move(CENTRO[0], CENTRO[1]); await settle(60);
  const cursorDentro = await page.evaluate(pageCursor);
  await mede("duplo_clique_no_centro", { pegas_no_modo_recorte: pegas, cursor_no_centro: cursorDentro });
  await tecla("Escape");
  await mede("escape");
  await clique(FORA);
  await page.evaluate(pageSelect, ["img"]); await settle();
  const pegasNormais = {};
  for (const k of Object.keys(PEGAS)) pegasNormais[k] = await achaPega(k);
  await mede("seleccionada_pela_api", { pegas_fora_do_modo: pegasNormais });
  await tecla("Enter");
  await mede("enter_com_a_imagem_seleccionada");
});
sonda("A02_canto_se", "Pega SE −80,−60 no modo recorte (e o esbatido fora do recorte, A12).", async () => {
  await dbl(CENTRO); await mede("entrou");
  await mede("se_menos80_menos60", await arrastaPega("se", [-80, -60]));
});
sonda("A02b_canto_se_livre", "Pega SE −80,−30 sem Shift: proporção livre?", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("se_menos80_menos30", await arrastaPega("se", [-80, -30]));
});
sonda("A02c_canto_se_shift", "Pega SE −80,−30 COM Shift.", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("se_menos80_menos30_shift", await arrastaPega("se", [-80, -30], { shift: true }));
});
sonda("A02d_escala_metade", "Imagem 200×150 (metade do natural): pega SE −40,−30 — o crop vem em px naturais?", async () => {
  const c = [IMG.x + 100, IMG.y + 75];
  await dbl(c); await mede("entrou");
  await mede("se_menos40_menos30", await arrastaPega("se", [-40, -30]));
}).els = [base({ width: 200, height: 150 })];
sonda("A03_canto_nw", "Pega NW +80,+60: o elemento move-se? A imagem fica parada no ecrã?", async () => {
  await dbl(CENTRO); await mede("entrou");
  await mede("nw_mais80_mais60", await arrastaPega("nw", [80, 60]));
});
sonda("A03b_canto_nw_livre", "Pega NW +80,+30.", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("nw_mais80_mais30", await arrastaPega("nw", [80, 30]));
});
sonda("A04_meios", "Pegas nos meios dos lados: E −80, N +60, S −40, W +40 (em sequência).", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("e_menos80", await arrastaPega("e", [-80, 0]));
  await mede("n_mais60", await arrastaPega("n", [0, 60]));
  await mede("s_menos40", await arrastaPega("s", [0, -40]));
  await mede("w_mais40", await arrastaPega("w", [40, 0]));
});
sonda("A05_pan", "Arrastar DENTRO do recorte: desliza a imagem sob a janela ou move o elemento? Limites.", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("se_menos160_menos120", await arrastaPega("se", [-160, -120]));
  const meio = async () => { const e = await imgDe(); return [e.x + e.width / 2, e.y + e.height / 2]; };
  let m = await meio();
  await page.mouse.move(m[0], m[1]); await settle(60);
  const cur = await page.evaluate(pageCursor);
  await mede("arrasta_dentro_menos50_menos40", { cursor_antes: cur, ...(await arrasta(m, [m[0] - 50, m[1] - 40])) });
  m = await meio();
  await mede("arrasta_dentro_mais30_mais20", await arrasta(m, [m[0] + 30, m[1] + 20]));
  m = await meio();
  await mede("arrasta_dentro_mais300_mais300", await arrasta(m, [m[0] + 300, m[1] + 300]));
  m = await meio();
  await mede("arrasta_dentro_menos600_menos600", await arrasta(m, [m[0] - 600, m[1] - 600]));
});
sonda("A05b_pan_passos", "O pan depende do número de pointermove? (−20,−10) com 1, 4 e 20 movimentos, cada um a partir do crop 0,0 (repõe com +300,+300); e a leitura ANTES de largar.", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("se_menos160_menos120", await arrastaPega("se", [-160, -120]), { foto: false });
  for (const n of [1, 4, 20]) {
    const m = [IMG.x + 120, IMG.y + 90];
    await mede(`repoe_${n}`, await arrasta(m, [m[0] + 300, m[1] + 300]), { foto: false });
    await mede(`pan_menos20_menos10_n${n}`, await arrastaN(m, [m[0] - 20, m[1] - 10], n));
  }
  const pg = await achaPega("se");
  await mede("pega_se_menos40_menos30_n4", await arrastaN(pg.ponto, [pg.ponto[0] - 40, pg.ponto[1] - 30], 4));
});
sonda("A05c_pan_metade", "Pan numa imagem a METADE do natural (200×150): (−20,−10) com 1 movimento — o Δcrop em px naturais.", async () => {
  const c = [IMG.x + 100, IMG.y + 75];
  await dbl(c); await mede("entrou", {}, { foto: false });
  await mede("se_menos80_menos60", await arrastaPega("se", [-80, -60]));
  const e = await imgDe();
  const m = [e.x + e.width / 2, e.y + e.height / 2];
  await mede("pan_menos20_menos10_n1", await arrastaN(m, [m[0] - 20, m[1] - 10], 1));
  await mede("pan_menos20_menos10_n4", await arrastaN(m, [m[0] - 20, m[1] - 10], 4));
}).els = [base({ width: 200, height: 150 })];
// O factor do pan: crop JÁ feito pela API (0,0,240,180 de 400×300), o elemento a 1×, 1,5× e 2× do
// recorte e o zoom 1 ou 2; um só pointermove de (−20,−10) px de ECRÃ a partir do meio.
const CROP0 = { x: 0, y: 0, width: 240, height: 180, naturalWidth: 400, naturalHeight: 300 };
for (const [nome, k, zoom, x0, y0] of [["A05d_pan_escala1_zoom1", 1, 1, IMG.x, IMG.y], ["A05e_pan_escala1_5_zoom1", 1.5, 1, IMG.x, IMG.y], ["A05f_pan_escala1_zoom2", 1, 2, 150, 80], ["A05g_pan_escala2_zoom1", 2, 1, IMG.x, IMG.y]]) {
  const o = sonda(nome, `Pan (−20,−10) px de ecrã, 1 movimento, elemento a ${k}× do recorte, zoom ${zoom}: Δcrop em px naturais.`, async () => {
    const m = [(x0 + (240 * k) / 2) * zoom, (y0 + (180 * k) / 2) * zoom];
    await dbl(m); await mede("entrou", {}, { foto: false });
    await mede("pan_menos20_menos10_n1", await arrastaN(m, [m[0] - 20, m[1] - 10], 1), { foto: false });
  });
  o.els = [base({ x: x0, y: y0, width: 240 * k, height: 180 * k, crop: CROP0 })];
  o.zoom = zoom;
}
sonda("A06_para_fora", "Pegas arrastadas para FORA da imagem natural: presas à borda?", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("se_mais100_mais80_inteira", await arrastaPega("se", [100, 80]));
  await mede("nw_menos100_menos80_inteira", await arrastaPega("nw", [-100, -80]));
  await mede("se_menos160_menos120", await arrastaPega("se", [-160, -120]));
  await mede("se_mais300_mais300_recortada", await arrastaPega("se", [300, 300]));
  await mede("e_mais200_recortada", await arrastaPega("e", [200, 0]));
});
sonda("A06b_minimo", "Pega SE até quase fechar (−395,−295): o tamanho mínimo.", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("se_menos395_menos295", await arrastaPega("se", [-395, -295]));
});
sonda("A06d_minimo_metade", "Imagem a METADE do natural (200×150): pega SE até quase fechar (−195,−145) — o mínimo é em px da caixa ou naturais?", async () => {
  await dbl([IMG.x + 100, IMG.y + 75]); await mede("entrou", {}, { foto: false });
  await mede("se_menos195_menos145", await arrastaPega("se", [-195, -145]));
}).els = [base({ width: 200, height: 150 })];
sonda("A06c_cruza", "Pega SE para lá do canto oposto (−500,−400).", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("se_menos500_menos400", await arrastaPega("se", [-500, -400]));
});
for (const [nome, acao] of [
  ["A07a_sai_enter", async () => tecla("Enter")],
  ["A07b_sai_escape", async () => tecla("Escape")],
  ["A07c_sai_clique_fora", async () => clique(FORA)],
  ["A07d_sai_duplo_clique", async () => { const e = await imgDe(); await dbl([e.x + e.width / 2, e.y + e.height / 2]); }],
]) {
  sonda(nome, "Recorta (SE −80,−60 e NW +40,+30), sai pelo gesto, e depois Ctrl+Z, Ctrl+Z, Ctrl+Shift+Z (A07 + A08).", async () => {
    await dbl(CENTRO); await mede("entrou", {}, { foto: false });
    await mede("se_menos80_menos60", await arrastaPega("se", [-80, -60]), { foto: false });
    await mede("nw_mais40_mais30", await arrastaPega("nw", [40, 30]));
    await acao();
    await mede("saiu");
    await tecla("Control+z"); await mede("ctrl_z_1");
    await tecla("Control+z"); await mede("ctrl_z_2");
    await tecla("Control+Shift+z"); await mede("ctrl_shift_z");
  });
}
sonda("A08b_desfazer_dentro", "Ctrl+Z DENTRO do modo recorte, depois de dois arrastos.", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("se_menos80_menos60", await arrastaPega("se", [-80, -60]), { foto: false });
  await mede("nw_mais40_mais30", await arrastaPega("nw", [40, 30]), { foto: false });
  await tecla("Control+z"); await mede("ctrl_z_1");
  await tecla("Control+z"); await mede("ctrl_z_2");
});
sonda("A09_redimensiona_recortada", "Recortada (SE −80,−60, sai pelo Enter); depois a pega NORMAL: o crop muda ou a imagem estica?", async () => {
  await dbl(CENTRO); await mede("entrou", {}, { foto: false });
  await mede("se_menos80_menos60", await arrastaPega("se", [-80, -60]), { foto: false });
  await tecla("Enter"); await mede("enter");
  await clique(FORA);
  let e = await imgDe();
  await clique([e.x + e.width / 2, e.y + e.height / 2]); await mede("seleccionada");
  await mede("se_normal_mais80_mais60", await arrastaPega("se", [80, 60]));
  await mede("se_normal_mais80_0", await arrastaPega("se", [80, 0]));
  await mede("e_normal_mais60", await arrastaPega("e", [60, 0]));
  await mede("se_normal_mais80_0_shift", await arrastaPega("se", [80, 0], { shift: true }));
});
sonda("A09b_redimensiona_inteira", "Controlo: a pega NORMAL numa imagem SEM recorte (+80,0 e E +60).", async () => {
  await clique(CENTRO); await mede("seleccionada", {}, { foto: false });
  await mede("se_normal_mais80_0", await arrastaPega("se", [80, 0]));
  await mede("e_normal_mais60", await arrastaPega("e", [60, 0]));
});
const TH = Math.PI / 6;
sonda("A10_rodada", "Rodada 30°: duplo clique entra? As pegas seguem a rotação? SE (−80,−60 no referencial da imagem).", async () => {
  await dbl(CENTRO);
  const pegas = {};
  for (const k of ["nw", "se", "e"]) pegas[k] = await achaPega(k);
  await mede("duplo_clique", { pegas_rodadas: pegas });
  await mede("se_local_menos80_menos60", await arrastaPega("se", rodar([-80, -60], TH)));
}).els = [base({ angle: TH })];
sonda("A11_espelhada", "Shift+H e depois recorte pela pega SE (−80,−60): crop em px da imagem NÃO espelhada?", async () => {
  await clique(CENTRO);
  await tecla("Shift+H"); await mede("shift_h");
  await dbl(CENTRO); await mede("entrou");
  await mede("se_menos80_menos60", await arrastaPega("se", [-80, -60]));
  await mede("nw_mais40_mais30", await arrastaPega("nw", [40, 30]));
  await tecla("Enter"); await mede("enter");
});
sonda("B01_espelho", "Shift+H / Shift+V: o que muda; ×2 volta? botões; menu de contexto.", async () => {
  await clique(CENTRO);
  await mede("seleccionada", { ui_titulos: await page.evaluate(pageUI) });
  await tecla("Shift+H"); await mede("shift_h_1");
  await tecla("Shift+H"); await mede("shift_h_2");
  await tecla("Shift+V"); await mede("shift_v_1");
  await tecla("Shift+V"); await mede("shift_v_2");
  await clique(CENTRO, { button: "right" });
  const menu = await page.evaluate(pageMenu);
  await mede("menu_contexto", { menu_contexto: menu });
  const flip = await page.evaluate(pageMenuClick, "flip horizontal");
  await settle(200);
  await mede("menu_flip_horizontal", { clicado: flip });
  await clique(CENTRO, { button: "right" });
  const crop = await page.evaluate(pageMenuClick, "crop");
  await settle(250);
  await mede("menu_crop", { clicado: crop });
  await tecla("Escape");
  await clique(FORA, { button: "right" });
  await mede("menu_contexto_no_vazio", { menu_contexto: await page.evaluate(pageMenu) }, { foto: false });
  await tecla("Escape");
});
sonda("B02_espelho_recortada", "Shift+H numa imagem JÁ recortada (crop 0,0,320,240 pela API): o crop muda?", async () => {
  await clique([IMG.x + 160, IMG.y + 120]);
  await mede("seleccionada");
  await tecla("Shift+H"); await mede("shift_h");
  await tecla("Shift+V"); await mede("shift_v");
}).els = [base({ width: 320, height: 240, crop: { x: 0, y: 0, width: 320, height: 240, naturalWidth: 400, naturalHeight: 300 } })];
sonda("B03_espelho_rodada", "Shift+H numa imagem rodada 30°: x/y/ângulo?", async () => {
  await clique(CENTRO); await mede("seleccionada", {}, { foto: false });
  await tecla("Shift+H"); await mede("shift_h");
  await tecla("Shift+H"); await mede("shift_h_2", {}, { foto: false });
}).els = [base({ angle: TH })];

const GIFEL = { x: 560, y: 270, w: 120, h: 120 };
sonda("C01_gif", "O canvas anima o GIF? (amostras no tempo, com e sem re-render) O SVG leva o GIF original? Que quadro sai no PNG?", async () => {
  await mede("montado");
  const c = [GIFEL.x + 60, GIFEL.y + 60];
  // CONTROLO: o MESMO GIF num <img> do DOM (anima?) e esse <img> desenhado num canvas novo (drawImage).
  await page.evaluate((src) => {
    const im = document.createElement("img");
    im.id = "controlo-gif"; im.src = src;
    im.style.cssText = "position:fixed;left:1100px;top:100px;width:120px;height:120px;z-index:99999";
    document.body.appendChild(im);
  }, FILE_GIF.dataURL);
  await settle(300);
  const pageDraw = () => {
    const im = document.getElementById("controlo-gif");
    const cv = document.createElement("canvas"); cv.width = 120; cv.height = 120;
    const g = cv.getContext("2d"); g.drawImage(im, 0, 0);
    return [...g.getImageData(60, 60, 1, 1).data].slice(0, 3);
  };
  // centro (620,330) e borda esquerda (560,330) da imagem no canvas: a borda prova que houve re-render
  // (com scrollX 1 a imagem anda 1 px e o píxel 560 fica branco); e o centro do <img> de controlo.
  const amostra = async () => {
    const buf = await page.screenshot({ clip: { x: 555, y: 325, width: 70, height: 10 } });
    const a = (await page.evaluate(pageDecode, { b64: buf.toString("base64"), pts: [[65, 5], [5, 5]], linhas: [] })).pontos.map((q) => q.rgba.slice(0, 3));
    const bc = await page.screenshot({ clip: { x: 1160, y: 160, width: 1, height: 1 } });
    const ctl = (await page.evaluate(pageDecode, { b64: bc.toString("base64"), pts: [[0, 0]], linhas: [] })).pontos[0].rgba.slice(0, 3);
    return { rgb: a[0], borda_x560: a[1], controlo_img_dom: ctl, controlo_drawImage: await page.evaluate(pageDraw) };
  };
  const semRender = [];
  const t0 = Date.now();
  for (let i = 0; i < 12; i++) {
    semRender.push({ t_ms: Date.now() - t0, ...(await amostra()) });
    if (i === 0 || i === 3 || i === 6) {
      const buf = await page.screenshot({ clip: CLIP });
      fs.writeFileSync(path.join(outDir, `w16_C01_gif_t${i}.png`), buf);
    }
    await page.waitForTimeout(150);
  }
  const comRender = [];
  const t1 = Date.now();
  for (let i = 0; i < 12; i++) {
    await page.evaluate(pageScroll, i % 2); // força o canvas estático a redesenhar (1 px)
    await settle(10);
    comRender.push({ t_ms: Date.now() - t1, scrollX: i % 2, ...(await amostra()) });
    await page.waitForTimeout(150);
  }
  await page.evaluate(pageScroll, 0);
  await page.evaluate(() => document.getElementById("controlo-gif")?.remove());
  const ex = [];
  for (let i = 0; i < 3; i++) {
    const t = Date.now();
    const r = await page.evaluate(pageExport);
    const png = Buffer.from(r.png, "base64");
    const px = await page.evaluate(pageDecode, { b64: r.png, pts: [[70, 70]], linhas: [] });
    const m = r.svg.match(/<image[^>]*href="(data:[^;"]+);base64,([^"]*)"/);
    const hrefB64 = m ? m[2] : null;
    ex.push({
      t_ms: t - t1, files: r.files, png_dim: [px.w, px.h], png_centro_rgb: px.pontos[0].rgba.slice(0, 3), png_sha256: createHash("sha256").update(png).digest("hex"),
      svg_image_mime: m ? m[1] : null, svg_href_igual_ao_gif_de_entrada: hrefB64 === GIF.b64, svg_href_len: hrefB64?.length ?? null,
    });
    if (i === 0) {
      fs.writeFileSync(path.join(outDir, "w16_C01_gif_export.svg"), r.svg);
      fs.writeFileSync(path.join(outDir, "w16_C01_gif_export.png"), png);
    }
    await page.waitForTimeout(250);
  }
  await mede("depois", { canvas_sem_rerender: semRender, canvas_com_rerender: comRender, exportacoes: ex }, { foto: false });
}).els = [base({ id: "gif", fileId: "w16gif", x: GIFEL.x, y: GIFEL.y, width: GIFEL.w, height: GIFEL.h })];

// ---- corre ----
const L0 = loadavg();
const res = [];
let falhas = 0;
for (const s of SONDAS) {
  if (so && !so.split(",").some((q) => s.nome.startsWith(q))) continue;
  const els = s.els || [base()];
  const files = els.some((e) => e.fileId === "w16gif") ? [FILE_GIF] : [FILE_PNG];
  sondaAtual = { nome: s.nome, passos: [], prevApp: null };
  const t = Date.now();
  try {
    const carga = await monta(els, files, s.zoom || 1);
    await s.fn();
    res.push({ sonda: s.nome, pergunta: s.pergunta, entrada: els, carga_da_imagem: carga, passos: sondaAtual.passos, ms: Date.now() - t, load: loadavg() });
    console.log(`ok ${s.nome} (${sondaAtual.passos.length} passos, ${Date.now() - t} ms, load ${loadavg()[0]})`);
  } catch (e) {
    falhas++;
    console.error(`FALHA ${s.nome}: ${e.stack || e.message}`);
    res.push({ sonda: s.nome, pergunta: s.pergunta, falha: String(e.stack || e.message), passos: sondaAtual.passos });
  }
  await page.keyboard.press("Escape").catch(() => {});
}
await browser.close();
server.close();

const nomeSaida = so ? `w16_sondas.so_${so.replace(/,/g, "_")}.json` : "w16_sondas.json";
const corpo = {
  oraculo: "@excalidraw/excalidraw", versao: PKG.version, licenca: PKG.license,
  chromium: browser.version?.() ?? null, playwright: PW.version,
  data: new Date().toISOString(), loadavg_inicio: L0, loadavg_fim: loadavg(),
  comando: `node sonda_imagem_w16.mjs${so ? ` --so ${so}` : ""}`,
  script_sha256: SCRIPT_SHA,
  entradas: { [PNG.nome]: { sha256: PNG.sha256, bytes: PNG.bytes, natural: [400, 300], quadrantes: "TL (255,0,0) · TR (0,160,0) · BL (0,0,255) · BR (0,0,0)" },
    [GIF.nome]: { sha256: GIF.sha256, bytes: GIF.bytes, natural: [120, 120], quadros: "3 × 200 ms em ciclo: (255,0,0) · (0,160,0) · (0,0,255)" } },
  ambiente: { janela: [1280, 800], deviceScaleFactor: 1, zoom: 1, scroll: [0, 0], cena_igual_ecra: true, imagem: IMG, clip_das_fotos: CLIP, clique_fora: FORA, relogio: "livre (o duplo clique precisa do tempo a andar)", prng: `mulberry32 semente ${SEED} (sha das entradas)` },
  editor: "<Excalidraw> montado (excalidrawAPI: addFiles, updateScene, getSceneElements, getAppState, getFiles; exportToSvg/exportToBlob); rato e teclado do Playwright; pegas achadas pelo cursor do canvas.interactive",
  pixeis: "corridas de cor (tolerância 6/canal) nas linhas y=240 e y=420 e colunas x=520 e x=720 do ECRÃ, lidas da captura recortada; sobre branco, um quadrante esbatido a alfa α dá v = (1−α)·255 no canal que é 0",
  pedidos_externos_bloqueados: external, consola: consoleErrors, falhas,
  sondas: res,
};
fs.writeFileSync(path.join(outDir, nomeSaida), JSON.stringify(corpo, null, 1) + "\n");
console.log(`gravado ${path.join(outDir, nomeSaida)} (${falhas} falhas)`);
process.exit(falhas ? 1 : 0);
