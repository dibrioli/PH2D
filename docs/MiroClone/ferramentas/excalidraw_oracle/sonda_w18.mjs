// W18 — duas perguntas a um navegador/editor que JÁ fazem isto, corridas por script (o fonte não se lê):
//   T — o `<text>` de um SVG NOSSO no Chromium (o motor que o Excalidraw usa para mostrar um SVG):
//       família presente/ausente/genérica, `text-anchor`, `tspan` (x/y/dx/dy), `transform`, peso/itálico,
//       unidades, espaços, contorno, `viewBox`. Geometria pela API do DOM (SVG em linha: getBBox,
//       getComputedTextLength, getStartPositionOfChar) e píxeis pelo `<img>` (como um ficheiro de imagem).
//   Y — um link do YouTube COLADO no editor montado do Excalidraw 0.18.1: que elemento nasce (type, link,
//       tamanho, proporção, validated), o iframe (src, onde, se recebe o rato), o que se desenha parado e
//       activado, o redimensionar, o pan/zoom, o export PNG/SVG e o `.excalidraw` gravado.
//       Corre DUAS vezes: `--rede nao` (tudo fora de 127.0.0.1 bloqueado) e `--rede youtube` (só os domínios
//       do YouTube/Google deixados passar).
// Uso: node sonda_w18.mjs --rede nao|youtube [--so <prefixo>[,…]] [<pasta_entradas> <pasta_saida>]
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
const rede = opt("--rede") || "nao";
if (!["nao", "youtube"].includes(rede)) throw new Error("--rede nao|youtube");
const entDir = args[0] || path.join(HERE, "entradas");
const outDir = args[1] || path.join(HERE, "saidas/w18");
fs.mkdirSync(outDir, { recursive: true });
const SCRIPT_SHA = createHash("sha256").update(fs.readFileSync(fileURLToPath(import.meta.url))).digest("hex");
const r3 = (v) => (typeof v === "number" ? Math.round(v * 1000) / 1000 : v);
const loadavg = () => os.loadavg().map((v) => r3(v));

// ---- entradas T: os SVG vivem em entradas/w18_texto/*.svg (gera_w18_svg_texto.py) ----
const TDIR = path.join(entDir, "w18_texto");
const SVGS = fs.existsSync(TDIR) ? fs.readdirSync(TDIR).filter((n) => n.endsWith(".svg")).sort() : [];
const svgDe = (n) => { const b = fs.readFileSync(path.join(TDIR, n)); return { nome: n, txt: b.toString("utf8"), sha256: createHash("sha256").update(b).digest("hex") }; };

// ---- entradas Y: os links ----
const VID = "dQw4w9WgXcQ";
const LINKS = [
  ["Y01_watch", `https://www.youtube.com/watch?v=${VID}`],
  ["Y02_youtu_be", `https://youtu.be/${VID}`],
  ["Y03_shorts", `https://www.youtube.com/shorts/${VID}`],
  ["Y04_embed", `https://www.youtube.com/embed/${VID}`],
  ["Y05_live", `https://www.youtube.com/live/${VID}`],
  ["Y06_watch_t_s", `https://www.youtube.com/watch?v=${VID}&t=90s`],
  ["Y07_watch_t_hms", `https://www.youtube.com/watch?v=${VID}&t=1m30s`],
  ["Y08_youtu_be_t", `https://youtu.be/${VID}?t=42`],
  ["Y09_embed_start", `https://www.youtube.com/embed/${VID}?start=30`],
  ["Y10_watch_list", `https://www.youtube.com/watch?v=${VID}&list=PLFgquLnL59alCl_2TQvOiD5Vgm1hCaGSI`],
  ["Y11_playlist", "https://www.youtube.com/playlist?list=PLFgquLnL59alCl_2TQvOiD5Vgm1hCaGSI"],
  ["Y12_canal", "https://www.youtube.com/@YouTube"],
  ["Y13_m_youtube", `https://m.youtube.com/watch?v=${VID}`],
  ["Y14_nocookie", `https://www.youtube-nocookie.com/embed/${VID}`],
  ["Y15_music", `https://music.youtube.com/watch?v=${VID}`],
  ["Y16_sem_www", `https://youtube.com/watch?v=${VID}`],
  ["Y17_http", `http://www.youtube.com/watch?v=${VID}`],
  ["Y18_maiusculas", `https://WWW.YOUTUBE.COM/watch?v=${VID}`],
  ["Y19_sem_esquema", `www.youtube.com/watch?v=${VID}`],
  ["Y20_id_curto", "https://www.youtube.com/watch?v=abc"],
  ["Y21_watch_v_depois", `https://www.youtube.com/watch?feature=share&v=${VID}`],
  ["Y22_texto_com_link", `look at this https://youtu.be/${VID}`],
  ["Y23_nao_video", "https://example.com/page"],
  ["Y24_vimeo", "https://vimeo.com/76979871"],
];

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
const SEED = parseInt(createHash("sha256").update(SVGS.join("|") + LINKS.flat().join("|")).digest("hex").slice(0, 8), 16);
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

// ---- lado da página: T ----
// Geometria: o SVG em linha; cada <text>/<tspan> com id devolve bbox, comprimento e posição de cada letra
// (no sistema do utilizador do próprio elemento) e o CTM até ao ecrã.
const pageTextoGeo = ({ txt }) => {
  const host = document.createElement("div");
  host.style.cssText = "position:fixed;left:0;top:0;background:#fff";
  host.innerHTML = txt;
  document.body.appendChild(host);
  const svg = host.querySelector("svg");
  const out = [];
  for (const el of svg.querySelectorAll("text, tspan")) {
    const b = el.getBBox();
    const n = el.getNumberOfChars();
    const letras = [];
    for (let i = 0; i < n; i++) {
      const s = el.getStartPositionOfChar(i), e = el.getEndPositionOfChar(i);
      letras.push({ i, de: [r3(s.x), r3(s.y)], ate: [r3(e.x), r3(e.y)], rot: r3(el.getRotationOfChar(i)) });
    }
    const m = el.getCTM();
    const cs = getComputedStyle(el);
    out.push({
      tag: el.tagName, id: el.id || null, texto: el.textContent,
      bbox: [r3(b.x), r3(b.y), r3(b.width), r3(b.height)],
      comprimento: r3(el.getComputedTextLength()), n_letras: n, letras,
      ctm: [m.a, m.b, m.c, m.d, m.e, m.f].map(r3),
      estilo: { fontFamily: cs.fontFamily, fontSize: cs.fontSize, fontWeight: cs.fontWeight, fontStyle: cs.fontStyle, textAnchor: cs.textAnchor, letterSpacing: cs.letterSpacing, whiteSpace: cs.whiteSpace },
    });
  }
  host.remove();
  return out;
};
// Píxeis: o SVG como ficheiro de imagem (o que o Excalidraw faz) a escala `k`; caixa da tinta por cor-chave.
const pageTextoPix = async ({ txt, k, chaves }) => {
  const blob = new Blob([txt], { type: "image/svg+xml" });
  const url = URL.createObjectURL(blob);
  const img = new Image();
  img.src = url;
  await img.decode();
  const w = Math.round(img.naturalWidth * k), h = Math.round(img.naturalHeight * k);
  const c = new OffscreenCanvas(w, h);
  const g = c.getContext("2d");
  g.fillStyle = "#fff"; g.fillRect(0, 0, w, h);
  g.drawImage(img, 0, 0, w, h);
  URL.revokeObjectURL(url);
  const d = g.getImageData(0, 0, w, h).data;
  const caixas = {};
  for (const [nome, rgb] of Object.entries(chaves)) caixas[nome] = { x0: 1e9, y0: 1e9, x1: -1, y1: -1, n: 0 };
  let tinta = { x0: 1e9, y0: 1e9, x1: -1, y1: -1, n: 0 };
  const junta = (b, x, y) => { b.x0 = Math.min(b.x0, x); b.y0 = Math.min(b.y0, y); b.x1 = Math.max(b.x1, x); b.y1 = Math.max(b.y1, y); b.n++; };
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const i = (y * w + x) * 4;
    const p = [d[i], d[i + 1], d[i + 2]];
    if (p[0] > 245 && p[1] > 245 && p[2] > 245) continue;
    junta(tinta, x, y);
    // cor-chave: o canal dominante tem de ser o da chave, com margem (o anti-alias mistura com branco)
    for (const [nome, rgb] of Object.entries(chaves)) {
      const dist = Math.hypot(...rgb.map((q, j) => {
        const alfa = 1 - Math.min(p[0], p[1], p[2]) / 255; // quanto de tinta há
        return alfa > 0.35 ? (p[j] - rgb[j]) : 999;
      }));
      if (dist < 140) junta(caixas[nome], x, y);
    }
  }
  const png = await c.convertToBlob({ type: "image/png" });
  const buf = new Uint8Array(await png.arrayBuffer());
  let s = ""; for (let i = 0; i < buf.length; i += 0x8000) s += String.fromCharCode.apply(null, buf.subarray(i, i + 0x8000));
  return { dim: [w, h], tinta, caixas, png_b64: btoa(s) };
};

// ---- lado da página: Y ----
const pageMount = async ({ els }) => {
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
  await window.oracle.editorMount(restoreElements(els, null, { refreshDimensions: false, repairBindings: true }));
  window.oracle.editorApi().updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: 1 } } });
  return true;
};
const pageCola = ({ texto }) => {
  const dt = new DataTransfer();
  dt.setData("text/plain", texto);
  const ev = new ClipboardEvent("paste", { clipboardData: dt, bubbles: true, cancelable: true });
  (document.activeElement || document).dispatchEvent(ev);
  return document.activeElement?.className?.toString?.() ?? document.activeElement?.nodeName ?? null;
};
const pageSnap = () => {
  const api = window.oracle.editorApi();
  const a = api.getAppState();
  const vis = (el) => { const r = el.getBoundingClientRect(); return r.width > 0 && r.height > 0; };
  const avisos = [...document.querySelectorAll(".Toast, .ErrorDialog, .Modal, [role=dialog], [role=alert], .Dialog")]
    .filter(vis).map((el) => ({ classe: el.className?.toString?.() ?? "", texto: el.innerText.trim().slice(0, 400) }));
  const iframes = [...document.querySelectorAll("iframe")].map((f) => {
    const r = f.getBoundingClientRect();
    const cs = getComputedStyle(f);
    let p = f.parentElement, pais = [];
    for (let k = 0; p && k < 4; k++, p = p.parentElement) {
      const c = getComputedStyle(p);
      pais.push({ classe: p.className?.toString?.() ?? p.nodeName, transform: c.transform, pointerEvents: c.pointerEvents, opacity: c.opacity, display: c.display, visibility: c.visibility });
    }
    return {
      src: f.getAttribute("src"), srcdoc: f.hasAttribute("srcdoc") ? f.getAttribute("srcdoc").slice(0, 300) : null,
      rect: [r.x, r.y, r.width, r.height].map((v) => Math.round(v * 10) / 10),
      pointerEvents: cs.pointerEvents, display: cs.display, visibility: cs.visibility, opacity: cs.opacity,
      sandbox: f.getAttribute("sandbox"), allow: f.getAttribute("allow"), referrerpolicy: f.getAttribute("referrerpolicy"),
      title: f.getAttribute("title"), pais,
    };
  });
  return {
    elementos: api.getSceneElements().map((e) => ({
      id: e.id, type: e.type, x: e.x, y: e.y, width: e.width, height: e.height, angle: e.angle,
      link: e.link ?? null, validated: e.validated ?? null, customData: e.customData ?? null,
      text: e.text ?? null, isDeleted: e.isDeleted, roundness: e.roundness ?? null,
      strokeColor: e.strokeColor, backgroundColor: e.backgroundColor, fillStyle: e.fillStyle,
    })),
    appState: {
      activeEmbeddable: a.activeEmbeddable ?? null, selectedElementIds: Object.keys(a.selectedElementIds || {}),
      scrollX: a.scrollX, scrollY: a.scrollY, zoom: a.zoom?.value, toast: a.toast ?? null, errorMessage: a.errorMessage ?? null,
      activeTool: a.activeTool?.type ?? null, showHyperlinkPopup: a.showHyperlinkPopup ?? null,
    },
    iframes,
    avisos_visiveis: avisos,
    textos_que_apareceram: window.__textos.slice(),
    cursor: document.querySelector("canvas.interactive")?.style.cursor ?? null,
  };
};
const pageRaf = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
const pageExport = async () => {
  const api = window.oracle.editorApi();
  const { exportToSvg, exportToBlob, serializeAsJSON } = window.oracle.api;
  const elements = api.getSceneElements(), files = api.getFiles();
  const appState = { exportBackground: true, viewBackgroundColor: "#ffffff", exportWithDarkMode: false };
  const r = {};
  try {
    const svg = new XMLSerializer().serializeToString(await exportToSvg({ elements, appState, files, exportPadding: 10 }));
    r.svg_txt = svg;
  } catch (e) { r.svg_erro = String(e.message || e); }
  try {
    const blob = await exportToBlob({ elements, appState, files, mimeType: "image/png", exportPadding: 10 });
    r.png_b64 = await window.oracle.blobToBase64(blob);
  } catch (e) { r.png_erro = String(e.message || e); }
  try { r.json = serializeAsJSON(elements, api.getAppState(), files, "local"); } catch (e) { r.json_erro = String(e.message || e); }
  return r;
};

// ---- corredor ----
const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
const DEIXA = /^https?:\/\/([a-z0-9-]+\.)*(youtube\.com|youtube-nocookie\.com|youtu\.be|ytimg\.com|ggpht\.com|googlevideo\.com|google\.com|gstatic\.com|googleapis\.com|doubleclick\.net|googleusercontent\.com)(\/|:|$)/i;
const pedidos = { bloqueados: [], deixados: [] };
await context.route("**/*", (route) => {
  const u = route.request().url();
  if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue();
  if (rede === "youtube" && DEIXA.test(u)) { pedidos.deixados.push(u.slice(0, 160)); return route.continue(); }
  pedidos.bloqueados.push(u.slice(0, 160));
  return route.abort();
});
await context.addInitScript(seedScript);
const page = await context.newPage();
page.setDefaultTimeout(120000);
let consola = [];
page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") consola.push(`${m.type()}: ${m.text().slice(0, 300)}`); });
page.on("pageerror", (e) => consola.push(`pageerror: ${e.message}`));
await page.goto(origin + "/", { timeout: 120000 });
await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 120000 });
await page.addScriptTag({ content: `window.r3 = ${r3.toString()};` });

const settle = async (ms = 150) => { await page.evaluate(pageRaf); await page.waitForTimeout(ms); };
const CLIP = { x: 0, y: 0, width: 1280, height: 800 };
const sufixo = rede === "nao" ? "offline" : "online";
let sondaAtual = null;
async function mede(nome, extra = {}, { foto = true } = {}) {
  await settle(150);
  const s = await page.evaluate(pageSnap);
  const passo = { passo: nome, ...extra, ...s, consola: consola.splice(0) };
  if (foto) {
    const n = String(sondaAtual.passos.length).padStart(2, "0");
    const fname = `w18_${sufixo}_${sondaAtual.nome}_${n}_${nome}.png`;
    fs.writeFileSync(path.join(outDir, fname), await page.screenshot({ clip: CLIP }));
    passo.foto = fname;
  }
  sondaAtual.passos.push(passo);
  return s;
}
const SONDAS = [];
const sonda = (nome, pergunta, fn) => { SONDAS.push({ nome, pergunta, fn }); };

// ---- T (só na corrida sem rede: não depende de rede) ----
const CHAVES = { vermelho: [220, 0, 0], verde: [0, 160, 0], azul: [0, 0, 220] };
if (rede === "nao") {
  for (const n of SVGS) {
    sonda(`T_${n.replace(/\.svg$/, "")}`, `O SVG «${n}» no Chromium: geometria de cada <text>/<tspan> (DOM) e a tinta no <img> a 1× e 2×.`, async () => {
      const e = svgDe(n);
      const geo = await page.evaluate(pageTextoGeo, { txt: e.txt });
      const pix = {};
      for (const k of [1, 2]) {
        const r = await page.evaluate(pageTextoPix, { txt: e.txt, k, chaves: CHAVES });
        const fname = `w18_texto_${n.replace(/\.svg$/, "")}_${k}x.png`;
        fs.writeFileSync(path.join(outDir, fname), Buffer.from(r.png_b64, "base64"));
        delete r.png_b64;
        for (const b of [r.tinta, ...Object.values(r.caixas)]) if (b.n === 0) Object.assign(b, { x0: null, y0: null, x1: null, y1: null });
        pix[`${k}x`] = { ...r, foto: fname };
      }
      sondaAtual.passos.push({ passo: "mede", entrada: { nome: n, sha256: e.sha256 }, geometria: geo, pixeis: pix, consola: consola.splice(0) });
    });
  }
}

// ---- Y ----
const PONTO = [640, 400];
for (const [nome, url] of LINKS) {
  sonda(nome, `Colar «${url}» no canvas vazio (rato em ${PONTO}): que elemento nasce?`, async () => {
    await page.mouse.move(...PONTO); await page.mouse.click(...PONTO); await settle(100);
    const alvo = await page.evaluate(pageCola, { texto: url });
    await page.waitForTimeout(rede === "nao" ? 1500 : 6000);
    await mede("colou", { colado: url, alvo_do_colar: alvo });
  });
}
// O caminho inteiro sobre um vídeo: parado, seleccionado, activado, redimensionar (livre/Shift), pan, zoom, export.
const umVideo = async () => {
  await page.mouse.move(...PONTO); await page.mouse.click(...PONTO); await settle(100);
  await page.evaluate(pageCola, { texto: `https://www.youtube.com/watch?v=${VID}` });
  await page.waitForTimeout(rede === "nao" ? 1500 : 8000);
  return (await page.evaluate(pageSnap)).elementos.find((e) => !e.isDeleted);
};
sonda("Z01_caminho", "Um vídeo do YouTube colado: parado → clique (selecção) → clique fora → duplo clique / clique no meio estando seleccionado (activar) → Esc.", async () => {
  const e = await umVideo();
  const meio = [Math.round(e.x + e.width / 2), Math.round(e.y + e.height / 2)];
  await page.keyboard.press("Escape"); await settle(200);
  await mede("parado", { elemento: e });
  await page.mouse.click(5, 790); await settle(200);
  await mede("desseleccionado");
  await page.mouse.click(...meio); await settle(400);
  await mede("clique_1");
  await page.mouse.click(...meio); await settle(rede === "nao" ? 800 : 5000);
  await mede("clique_2");
  await page.mouse.dblclick(...meio); await settle(rede === "nao" ? 800 : 5000);
  await mede("duplo_clique");
  await page.keyboard.press("Escape"); await settle(300);
  await mede("esc");
  await page.mouse.click(5, 790); await settle(300);
  await mede("clique_fora");
});
sonda("Z02_redimensiona", "Pega SE do vídeo +200,+40 sem Shift e com Shift: a proporção mantém-se?", async () => {
  const e = await umVideo();
  await page.keyboard.press("Escape"); await page.mouse.click(5, 790); await settle(150);
  await page.mouse.click(Math.round(e.x + e.width / 2), Math.round(e.y + 8)); await settle(200);
  await mede("seleccionado", {}, { foto: false });
  for (const shift of [false, true]) {
    const s = (await page.evaluate(pageSnap)).elementos.find((q) => !q.isDeleted);
    let p = [Math.round(s.x + s.width), Math.round(s.y + s.height)];
    // acha a pega pelo cursor
    let achou = null;
    for (let d = -10; d <= 16 && !achou; d += 2) {
      const q = [p[0] + d, p[1] + d];
      await page.mouse.move(...q); await page.waitForTimeout(15);
      if (/resize/.test((await page.evaluate(() => document.querySelector("canvas.interactive")?.style.cursor)) || "")) achou = q;
    }
    p = achou || p;
    await page.mouse.move(...p); await page.mouse.down();
    if (shift) await page.keyboard.down("Shift");
    await page.mouse.move(p[0] + 200, p[1] + 40, { steps: 12 }); await settle(100);
    await page.mouse.up();
    if (shift) await page.keyboard.up("Shift");
    await settle(300);
    await mede(shift ? "se_shift" : "se_livre", { pega: p, delta: [200, 40] });
  }
});
sonda("Z03_vista", "Pan (scroll −200,−100) e zoom 0,5 e 2: o iframe segue? some? Rotação 30°?", async () => {
  const e = await umVideo();
  await page.keyboard.press("Escape"); await page.mouse.click(5, 790); await settle(150);
  await mede("antes");
  await page.evaluate(() => window.oracle.editorApi().updateScene({ appState: { scrollX: -200, scrollY: -100 } }));
  await settle(400); await mede("pan");
  for (const z of [0.5, 2]) {
    await page.evaluate((zz) => window.oracle.editorApi().updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: zz } } }), z);
    await settle(400); await mede(`zoom_${z}`);
  }
  await page.evaluate(() => window.oracle.editorApi().updateScene({ appState: { zoom: { value: 1 } } }));
  await page.evaluate((id) => {
    const api = window.oracle.editorApi();
    api.updateScene({ elements: api.getSceneElements().map((q) => (q.id === id ? { ...q, angle: Math.PI / 6, version: q.version + 1 } : q)) });
  }, e.id);
  await settle(400); await mede("rodado_30");
});
sonda("Z04_export", "Export PNG/SVG e o .excalidraw gravado de um vídeo colado.", async () => {
  await umVideo();
  await page.keyboard.press("Escape"); await page.mouse.click(5, 790); await settle(150);
  const r = await page.evaluate(pageExport);
  const out = {};
  if (r.svg_txt) { const f = `w18_${sufixo}_Z04_export.svg`; fs.writeFileSync(path.join(outDir, f), r.svg_txt); out.svg = { ficheiro: f, bytes: r.svg_txt.length, tem_a: /<a[\s>]/.test(r.svg_txt), tem_image: /<image/.test(r.svg_txt), tem_foreignObject: /<foreignObject/.test(r.svg_txt), textos: [...r.svg_txt.matchAll(/<text[^>]*>([^<]*)</g)].map((m) => m[1]).slice(0, 20), hrefs: [...r.svg_txt.matchAll(/href="([^"]{0,200})/g)].map((m) => m[1].slice(0, 120)) }; } else out.svg = { erro: r.svg_erro };
  if (r.png_b64) { const f = `w18_${sufixo}_Z04_export.png`; fs.writeFileSync(path.join(outDir, f), Buffer.from(r.png_b64, "base64")); out.png = { ficheiro: f }; } else out.png = { erro: r.png_erro };
  if (r.json) { const f = `w18_${sufixo}_Z04_gravado.excalidraw`; fs.writeFileSync(path.join(outDir, f), r.json); out.excalidraw = { ficheiro: f, elementos: JSON.parse(r.json).elements }; } else out.excalidraw = { erro: r.json_erro };
  await mede("exportou", { export: out }, { foto: false });
});
sonda("Z05_abre_gravado", "Abrir um .excalidraw com um embeddable do YouTube escrito à mão (validated ausente, true e false).", async () => {
  const base = (id, x, over) => ({ id, type: "embeddable", x, y: 120, width: 400, height: 225, angle: 0, strokeColor: "#1e1e1e", backgroundColor: "transparent", fillStyle: "solid", strokeWidth: 2, strokeStyle: "solid", roughness: 1, opacity: 100, groupIds: [], frameId: null, roundness: null, seed: 1, version: 1, versionNonce: 1, isDeleted: false, boundElements: null, updated: 1, link: `https://youtu.be/${VID}`, locked: false, ...over });
  await page.evaluate(pageMount, { els: [base("sem", 20, {}), base("sim", 440, { validated: true }), base("nao", 860, { validated: false })] });
  await settle(rede === "nao" ? 1500 : 8000);
  await mede("abriu");
});

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
    if (!s.nome.startsWith("T_")) { await page.evaluate(pageMount, { els: [] }); await settle(300); }
    await s.fn();
    res.push({ sonda: s.nome, pergunta: s.pergunta, passos: sondaAtual.passos, ms: Date.now() - t, load: loadavg() });
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

const nomeSaida = `w18_sondas_${sufixo}${so ? `.so_${so.replace(/,/g, "_")}` : ""}.json`;
fs.writeFileSync(path.join(outDir, nomeSaida), JSON.stringify({
  oraculo: "@excalidraw/excalidraw", versao: PKG.version, licenca: PKG.license,
  chromium: browser.version?.() ?? null, playwright: PW.version,
  data: new Date().toISOString(), loadavg_inicio: L0, loadavg_fim: loadavg(),
  comando: `node sonda_w18.mjs --rede ${rede}${so ? ` --so ${so}` : ""}`,
  script_sha256: SCRIPT_SHA, rede,
  entradas_svg: Object.fromEntries(SVGS.map((n) => [n, svgDe(n).sha256])), links: LINKS,
  ambiente: { janela: [1280, 800], deviceScaleFactor: 1, zoom: 1, scroll: [0, 0], cena_igual_ecra: true, prng: `mulberry32 semente ${SEED}`, fontconfig_sans: "Noto Sans (fc-match sans-serif, 12/10)" },
  pedidos, falhas, sondas: res,
}, null, 1) + "\n");
console.log(`gravado ${path.join(outDir, nomeSaida)} (${falhas} falhas)`);
process.exit(falhas ? 1 : 0);
