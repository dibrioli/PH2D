// W17 — dois defeitos das imagens contra o editor montado do Excalidraw 0.18.1:
//   D — a imagem com o cabeçalho bom e os dados ESTRAGADOS (entradas `gera_w17_imagens.py`): o que o
//       canvas mostra, que `status` o elemento fica, se quem a pôs é avisado (toast/diálogo), e o export;
//       pelos dois caminhos — o ficheiro que chega com a cena (addFiles) e o ficheiro LARGADO (drop).
//   E — o Shift num canto do recorte com a imagem ESTICADA (kx ≠ ky) e já recortada: a proporção que
//       fica é a da imagem natural, a do natural no ECRÃ, ou a da caixa de agora?
// Só API pública (excalidrawAPI + exportToSvg/exportToBlob) e o rato/teclado do Playwright — o fonte
// do Excalidraw não se lê. Infra-estrutura no molde da `sonda_imagem_w16.mjs` (mesma página, mesma
// geometria: janela 1280×800, zoom 1, scroll 0 ⇒ cena = ecrã).
// Uso: node sonda_imagem_w17.mjs [--so <prefixo>[,<prefixo>…]] [<pasta_entradas> <pasta_saida>]
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
const outDir = args[1] || path.join(HERE, "saidas/w17");
fs.mkdirSync(outDir, { recursive: true });

const ler = (n, mime) => { const b = fs.readFileSync(path.join(entDir, n)); return { nome: n, mime, b64: b.toString("base64"), sha256: createHash("sha256").update(b).digest("hex"), bytes: b.length }; };
const ENT = {
  bom: ler("w16_quadrantes.png", "image/png"),
  png_truncado: ler("w17_png_truncado.png", "image/png"),
  png_idat_lixo: ler("w17_png_idat_lixo.png", "image/png"),
  jpeg_lixo: ler("w17_jpeg_lixo.jpg", "image/jpeg"),
  jpeg_truncado: ler("w17_jpeg_truncado.jpg", "image/jpeg"),
};
const SCRIPT_SHA = createHash("sha256").update(fs.readFileSync(fileURLToPath(import.meta.url))).digest("hex");
const r3 = (v) => (typeof v === "number" ? Math.round(v * 1000) / 1000 : v);
const loadavg = () => os.loadavg().map((v) => r3(v));

const IMG = { x: 420, y: 180, w: 400, h: 300 };
const CLIP = { x: 300, y: 70, width: 680, height: 600 };
const fileOf = (k) => ({ id: `f_${k}`, mimeType: ENT[k].mime, dataURL: `data:${ENT[k].mime};base64,${ENT[k].b64}`, created: 1 });
const base = (over = {}) => ({
  id: "img", type: "image", x: IMG.x, y: IMG.y, width: IMG.w, height: IMG.h, angle: 0,
  strokeColor: "transparent", backgroundColor: "transparent", fillStyle: "solid", strokeWidth: 2,
  strokeStyle: "solid", roughness: 1, opacity: 100, groupIds: [], frameId: null, roundness: null,
  seed: 1, version: 1, versionNonce: 1, isDeleted: false, boundElements: null, updated: 1, link: null,
  locked: false, status: "saved", fileId: "f_bom", scale: [1, 1], crop: null, ...over,
});

const PEGAS = { nw: [-1, -1], n: [0, -1], ne: [1, -1], e: [1, 0], se: [1, 1], s: [0, 1], sw: [-1, 1], w: [-1, 0] };
const pegaGeo = (e, k) => {
  const [sx, sy] = PEGAS[k];
  const cx = e.x + e.width / 2, cy = e.y + e.height / 2;
  const n = Math.hypot(sx, sy);
  return { p: [cx + (sx * e.width) / 2, cy + (sy * e.height) / 2], dir: [sx / n, sy / n] };
};

// ---- servidor local da página ----
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
const SEED = parseInt(createHash("sha256").update(Object.values(ENT).map((e) => e.sha256).join("")).digest("hex").slice(0, 8), 16);
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
// Monta a cena e liga um observador que guarda TODO o texto que aparece no DOM (toasts que somem).
const pageMount = async ({ els, files }) => {
  const { restoreElements } = window.oracle.api;
  window.oracle.editorUnmount();
  window.__textos = [];
  window.__obs?.disconnect();
  window.__obs = new MutationObserver((ms) => {
    for (const m of ms) for (const n of m.addedNodes) {
      const t = (n.innerText ?? n.textContent ?? "").trim();
      if (t && t.length < 400) window.__textos.push({ classe: n.className?.toString?.() ?? n.nodeName, texto: t });
    }
  });
  window.__obs.observe(document.body, { childList: true, subtree: true });
  const elements = restoreElements(els, null, { refreshDimensions: false, repairBindings: true });
  await window.oracle.editorMount(elements);
  const api = window.oracle.editorApi();
  if (files.length) api.addFiles(files);
  api.updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: 1 } } });
  return true;
};
const pageSnap = () => {
  const api = window.oracle.editorApi();
  const a = api.getAppState();
  const vis = (el) => { const r = el.getBoundingClientRect(); return r.width > 0 && r.height > 0; };
  const avisos = [...document.querySelectorAll(".Toast, .ErrorDialog, .Modal, [role=dialog], [role=alert], .Dialog")]
    .filter(vis).map((el) => ({ classe: el.className?.toString?.() ?? "", texto: el.innerText.trim().slice(0, 400) }));
  const files = api.getFiles();
  return {
    elementos: api.getSceneElements().map((e) => ({
      id: e.id, type: e.type, x: e.x, y: e.y, width: e.width, height: e.height, scale: e.scale,
      crop: e.crop ?? null, status: e.status, fileId: e.fileId, isDeleted: e.isDeleted,
    })),
    ficheiros: Object.fromEntries(Object.entries(files).map(([k, f]) => [k, { mimeType: f.mimeType, dataURL_len: f.dataURL.length }])),
    appState: { errorMessage: a.errorMessage ?? null, toast: a.toast ?? null, openDialog: a.openDialog ?? null, croppingElementId: a.croppingElementId ?? null, isCropping: a.isCropping ?? null },
    avisos_visiveis: avisos,
    textos_que_apareceram: window.__textos.slice(),
    cursor: document.querySelector("canvas.interactive")?.style.cursor ?? null,
  };
};
const pageCursor = () => document.querySelector("canvas.interactive")?.style.cursor ?? null;
const pageRaf = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
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
// Larga um ficheiro no canvas como o gestor de ficheiros faria (dragover + drop com DataTransfer).
const pageDrop = ({ b64, mime, nome, x, y }) => {
  const bin = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
  const f = new File([bin], nome, { type: mime });
  const dt = new DataTransfer();
  dt.items.add(f);
  const alvo = document.elementFromPoint(x, y);
  for (const tipo of ["dragenter", "dragover", "drop"]) {
    alvo.dispatchEvent(new DragEvent(tipo, { bubbles: true, cancelable: true, clientX: x, clientY: y, dataTransfer: dt }));
  }
  return alvo.className?.toString?.() ?? alvo.nodeName;
};
const pageExport = async () => {
  const api = window.oracle.editorApi();
  const { exportToSvg, exportToBlob } = window.oracle.api;
  const elements = api.getSceneElements(), files = api.getFiles();
  const appState = { exportBackground: true, viewBackgroundColor: "#ffffff", exportWithDarkMode: false };
  const r = {};
  try {
    const svg = new XMLSerializer().serializeToString(await exportToSvg({ elements, appState, files, exportPadding: 10 }));
    const m = svg.match(/<image[^>]*href="(data:[^;"]+);base64,/);
    r.svg = { ok: true, bytes: svg.length, tem_image: /<image/.test(svg), image_mime: m ? m[1] : null, tem_use: /<use/.test(svg), symbols: (svg.match(/<symbol/g) || []).length };
  } catch (e) { r.svg = { ok: false, erro: String(e.message || e) }; }
  try {
    const blob = await exportToBlob({ elements, appState, files, mimeType: "image/png", exportPadding: 10 });
    r.png_b64 = await window.oracle.blobToBase64(blob);
  } catch (e) { r.png = { ok: false, erro: String(e.message || e) }; }
  return r;
};

// ---- corredor ----
const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
const external = [];
let consola = [];
await context.route("**/*", (route) => {
  const u = route.request().url();
  if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue();
  external.push(u);
  return route.abort();
});
await context.addInitScript(seedScript);
const page = await context.newPage();
page.setDefaultTimeout(120000);
page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") consola.push(`${m.type()}: ${m.text().slice(0, 300)}`); });
page.on("pageerror", (e) => consola.push(`pageerror: ${e.message}`));
await page.goto(origin + "/", { timeout: 120000 });
await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 120000 });

const settle = async (ms = 150) => { await page.evaluate(pageRaf); await page.waitForTimeout(ms); };
const rd = (p) => [Math.round(p[0]), Math.round(p[1])];
async function arrasta(p0, p1, { shift = false } = {}) {
  p0 = rd(p0); p1 = rd(p1);
  await page.mouse.move(p0[0], p0[1]);
  await settle(60);
  await page.mouse.down();
  if (shift) await page.keyboard.down("Shift");
  await page.mouse.move(p0[0] + Math.sign(p1[0] - p0[0] || 1) * 2, p0[1] + Math.sign(p1[1] - p0[1] || 1) * 2);
  await page.mouse.move(p1[0], p1[1], { steps: 12 });
  await page.waitForTimeout(100); await page.mouse.move(p1[0], p1[1]); await settle(100);
  await page.mouse.up();
  if (shift) await page.keyboard.up("Shift");
  await settle(200);
  return { de: p0, para: p1 };
}
const imgDe = async (id = "img") => (await page.evaluate(pageSnap)).elementos.find((e) => e.id === id);
async function achaPega(k) {
  const e = await imgDe();
  const { p, dir } = pegaGeo(e, k);
  const rs = [];
  for (let d = -14; d <= 24; d += 2) {
    const q = rd([p[0] + dir[0] * d, p[1] + dir[1] * d]);
    await page.mouse.move(q[0], q[1]);
    await page.waitForTimeout(15);
    if (/resize/.test((await page.evaluate(pageCursor)) || "")) rs.push(d);
  }
  const d = rs.length ? rs[Math.floor(rs.length / 2)] : 0;
  return rd([p[0] + dir[0] * d, p[1] + dir[1] * d]);
}
async function arrastaPega(k, delta, opts = {}) {
  const p = await achaPega(k);
  return { pega: k, delta, shift: !!opts.shift, ...(await arrasta(p, [p[0] + delta[0], p[1] + delta[1]], opts)) };
}
const LINHAS = [
  { nome: "linha_y240", eixo: "x", fixo: 240 },
  { nome: "linha_y420", eixo: "x", fixo: 420 },
  { nome: "coluna_x520", eixo: "y", fixo: 520 },
  { nome: "coluna_x720", eixo: "y", fixo: 720 },
];
let sondaAtual = null;
async function mede(nome, extra = {}, { foto = true } = {}) {
  await settle(150);
  const s = await page.evaluate(pageSnap);
  const passo = { passo: nome, ...extra, ...s, consola: consola.splice(0) };
  if (foto) {
    const n = String(sondaAtual.passos.length).padStart(2, "0");
    const fname = `w17_${sondaAtual.nome}_${n}_${nome}.png`;
    const buf = await page.screenshot({ clip: CLIP });
    fs.writeFileSync(path.join(outDir, fname), buf);
    const linhas = LINHAS.map((l) => (l.eixo === "x"
      ? { nome: l.nome, eixo: "x", fixo: l.fixo - CLIP.y, de: 0, ate: CLIP.width - 1 }
      : { nome: l.nome, eixo: "y", fixo: l.fixo - CLIP.x, de: 0, ate: CLIP.height - 1 }));
    const px = await page.evaluate(pageDecode, { b64: buf.toString("base64"), pts: [], linhas });
    passo.foto = fname;
    passo.pixeis = px.linhas.map((l) => {
      const off = l.eixo === "x" ? CLIP.x : CLIP.y;
      return { nome: l.nome, corridas: l.corridas.map((c) => ({ de: c.de + off, ate: c.ate + off, n: c.ate - c.de + 1, rgb: c.rgba.slice(0, 3) })) };
    });
  }
  sondaAtual.passos.push(passo);
  return s;
}
// Espera o estado do elemento assentar: sai de `pending` ou passam 8 s.
async function esperaStatus(id = "img") {
  const t0 = Date.now();
  const serie = [];
  while (Date.now() - t0 < 8000) {
    await settle(200);
    const e = (await page.evaluate(pageSnap)).elementos.find((q) => q.id === id || q.type === "image");
    const st = e?.status ?? null;
    if (!serie.length || serie[serie.length - 1].status !== st) serie.push({ t_ms: Date.now() - t0, status: st });
    if (st && st !== "pending" && Date.now() - t0 > 1500) break;
  }
  await settle(800);
  return serie;
}
async function exporta(nome) {
  const r = await page.evaluate(pageExport);
  const out = { svg: r.svg };
  if (r.png_b64) {
    const png = Buffer.from(r.png_b64, "base64");
    fs.writeFileSync(path.join(outDir, `w17_${sondaAtual.nome}_${nome}.png`), png);
    const px = await page.evaluate(pageDecode, { b64: r.png_b64, pts: [], linhas: [] });
    const mid = await page.evaluate(pageDecode, { b64: r.png_b64, pts: [[Math.floor(px.w / 4), Math.floor(px.h / 4)], [Math.floor(px.w / 2), Math.floor(px.h / 2)]], linhas: [] });
    out.png = { ok: true, dim: [px.w, px.h], amostras: mid.pontos.map((q) => ({ p: q.p, rgb: q.rgba.slice(0, 3) })) };
  } else out.png = r.png;
  return out;
}
const dbl = async (p) => { await page.mouse.dblclick(p[0], p[1]); await settle(250); };
const clique = async (p) => { await page.mouse.click(p[0], p[1]); await settle(200); };

// ---- as sondas ----
const SONDAS = [];
const sonda = (nome, pergunta, fn, extra = {}) => { SONDAS.push({ nome, pergunta, fn, ...extra }); };
const ESTRAGADAS = ["png_truncado", "png_idat_lixo", "jpeg_lixo", "jpeg_truncado"];

for (const k of ["bom", ...ESTRAGADAS]) {
  sonda(`D01_cena_${k}`, `O ficheiro «${ENT[k].nome}» chega COM a cena (addFiles, como ao abrir um quadro gravado): o que o canvas mostra, que status fica, há aviso? E o export?`, async () => {
    const serie = await esperaStatus();
    await mede("assentou", { status_no_tempo: serie });
    await clique([IMG.x + 100, IMG.y + 75]);
    await mede("seleccionada");
    await mede("export", { export: await exporta("export") }, { foto: false });
  }, { els: [base({ fileId: `f_${k}` })], files: [k] });
  sonda(`D02_largada_${k}`, `O ficheiro «${ENT[k].nome}» é LARGADO no canvas vazio (drop, como do gestor de ficheiros): entra um elemento? que status? quem largou é avisado?`, async () => {
    await mede("vazio", {}, { foto: false });
    const alvo = await page.evaluate(pageDrop, { b64: ENT[k].b64, mime: ENT[k].mime, nome: ENT[k].nome, x: 620, y: 330 });
    const serie = await esperaStatus();
    await mede("largou", { alvo_do_drop: alvo, status_no_tempo: serie });
    await page.waitForTimeout(3000);
    await mede("3s_depois");
  }, { els: [], files: [] });
}

// E — o Shift no recorte com a imagem esticada. Natural 400×300 (4:3).
const meio = (e) => [e.x + e.width / 2, e.y + e.height / 2];
const CROP_ALTO = { x: 0, y: 0, width: 200, height: 300, naturalWidth: 400, naturalHeight: 300 }; // natural 2:3
for (const [nome, pergunta, els, pega, delta] of [
  ["E01_larga_livre", "Esticada 600×300 (kx 1,5, ky 1): SE −80,−30 SEM Shift (controlo).", [base({ width: 600 })], "se", [-80, -30]],
  ["E02_larga_shift_se", "Esticada 600×300: SE −80,−30 COM Shift — 4:3 natural (crop 360×270 ⇒ caixa 540×270) ou 4:3 no ecrã (caixa 360×270)?", [base({ width: 600 })], "se", [-80, -30]],
  ["E03_larga_shift_nw", "Esticada 600×300: NW +80,+30 COM Shift.", [base({ width: 600 })], "nw", [80, 30]],
  ["E04_alta_shift_se", "Esticada 400×450 (kx 1, ky 1,5): SE −80,−30 COM Shift.", [base({ y: 120, height: 450 })], "se", [-80, -30]],
  ["E05_larga_shift_se_y", "Esticada 600×300: SE −20,−100 COM Shift (o eixo vertical manda).", [base({ width: 600 })], "se", [-20, -100]],
  ["E06_recortada_shift", "SEM esticar mas JÁ recortada (crop 200×300 natural, caixa 200×300): SE −40,−30 COM Shift — a proporção do natural (4:3) ou a do recorte de agora (2:3)?", [base({ width: 200, height: 300, crop: CROP_ALTO })], "se", [-40, -30]],
  ["E07_recortada_esticada_shift", "Recortada (crop 200×300) E esticada (caixa 300×300, kx 1,5): SE −40,−30 COM Shift — 4:3 natural, 4:3 ecrã, 2:3 natural ou 1:1 ecrã?", [base({ width: 300, height: 300, crop: CROP_ALTO })], "se", [-40, -30]],
]) {
  const shift = !nome.includes("livre");
  sonda(nome, pergunta, async () => {
    const e = await imgDe();
    await dbl(meio(e)); await mede("entrou", {}, { foto: false });
    await mede(`${pega}_${delta.join("_")}${shift ? "_shift" : ""}`, await arrastaPega(pega, delta, { shift }));
  }, { els, files: ["bom"] });
}
sonda("E08_esticada_pelo_gesto", "O caminho do artista: pega NORMAL SE +200,0 com Shift (estica a 600×300, A09b da W16), depois duplo clique e SE −80,−30 com Shift.", async () => {
  await clique([IMG.x + 200, IMG.y + 150]);
  await mede("estica", await arrastaPega("se", [200, 0], { shift: true }));
  const e = await imgDe();
  await dbl(meio(e)); await mede("entrou", {}, { foto: false });
  await mede("se_-80_-30_shift", await arrastaPega("se", [-80, -30], { shift: true }));
}, { els: [base()], files: ["bom"] });

// ---- corre ----
const L0 = loadavg();
const res = [];
let falhas = 0;
for (const s of SONDAS) {
  if (so && !so.split(",").some((q) => s.nome.startsWith(q))) continue;
  sondaAtual = { nome: s.nome, passos: [] };
  consola = [];
  const t = Date.now();
  try {
    await page.mouse.move(5, 790);
    await page.evaluate(pageMount, { els: s.els, files: s.files.map(fileOf) });
    await settle(300);
    await s.fn();
    res.push({ sonda: s.nome, pergunta: s.pergunta, entrada: s.els, ficheiros: s.files.map((k) => ENT[k].nome), passos: sondaAtual.passos, ms: Date.now() - t, load: loadavg() });
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

const nomeSaida = so ? `w17_sondas.so_${so.replace(/,/g, "_")}.json` : "w17_sondas.json";
const corpo = {
  oraculo: "@excalidraw/excalidraw", versao: PKG.version, licenca: PKG.license,
  chromium: browser.version?.() ?? null, playwright: PW.version,
  data: new Date().toISOString(), loadavg_inicio: L0, loadavg_fim: loadavg(),
  comando: `node sonda_imagem_w17.mjs${so ? ` --so ${so}` : ""}`,
  script_sha256: SCRIPT_SHA,
  entradas: Object.fromEntries(Object.values(ENT).map((e) => [e.nome, { sha256: e.sha256, bytes: e.bytes, mime: e.mime }])),
  ambiente: { janela: [1280, 800], deviceScaleFactor: 1, zoom: 1, scroll: [0, 0], cena_igual_ecra: true, imagem: IMG, clip_das_fotos: CLIP, relogio: "livre", prng: `mulberry32 semente ${SEED} (sha das entradas)` },
  pedidos_externos_bloqueados: external, falhas,
  sondas: res,
};
fs.writeFileSync(path.join(outDir, nomeSaida), JSON.stringify(corpo, null, 1) + "\n");
console.log(`gravado ${path.join(outDir, nomeSaida)} (${falhas} falhas)`);
process.exit(falhas ? 1 : 0);
