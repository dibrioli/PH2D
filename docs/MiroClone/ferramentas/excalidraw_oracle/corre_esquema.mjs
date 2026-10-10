// W8 — o esquema .excalidraw e a ida e volta, aprendidos pelas SAÍDAS do oráculo (API pública só).
// Uso: node corre_esquema.mjs <entrada.excalidraw> <sondas.json> <pasta_saida>
// Grava em <pasta_saida>, com o prefixo da entrada <n> (ex. w8_tipos):
//   <n>.restored.json      restore(ficheiro) — os elementos normalizados (o que ele preenche)
//   <n>.serialized.excalidraw   serializeAsJSON(…, "local") — o ficheiro que o Excalidraw grava
//   <n>.serialized_database.json serializeAsJSON(…, "database")
//   <n>.svg / <n>.png      exportToSvg (padding 10) / exportToBlob (png, escala 2, padding 10)
//   <n>.embed.svg / <n>.embed.png  exportEmbedScene:true (a cena dentro do ficheiro)
//   <n>.export.json        cada variante de export: dimensões, sha256, chunks PNG, <metadata> SVG,
//                          e a volta loadFromBlob(png/svg embutidos, .excalidraw gravado)
//   w8_sondas.json         restoreElements/restoreAppState sobre as sondas (entrada → saída)
//   w8_appstate_restaurado.json  restoreAppState({}) completo
//   <n>.meta.json          versões, data, sha256 das entradas e saídas
// Determinismo: o mesmo PRNG semeado + relógio fixo do corre.mjs. A página é a de $ORACLE_WWW
// (padrão $ORACLE_HOME/www — re-correr instala.sh depois de mudar src/oracle.js).
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle");
const WWW = process.env.ORACLE_WWW || path.join(ORACLE_HOME, "www");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const PKG = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/@excalidraw/excalidraw/package.json"), "utf8"));
const FIXED_TIME = Date.UTC(2026, 0, 1, 0, 0, 0);
const PNG_MAX_BYTES = 300 * 1024;

const [input, sondasPath, outDir] = process.argv.slice(2);
if (!input || !sondasPath || !outDir) {
  console.error("uso: node corre_esquema.mjs <entrada.excalidraw> <sondas.json> <pasta_saida>");
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });
const name = path.basename(input).replace(/\.excalidraw(\.json)?$/, "");
const inBytes = fs.readFileSync(input);
const sondasBytes = fs.readFileSync(sondasPath);
const sha = (b) => createHash("sha256").update(b).digest("hex");
const seed = parseInt(sha(Buffer.concat([inBytes, sondasBytes])).slice(0, 8), 16);

const MIME = { ".html": "text/html", ".js": "text/javascript", ".woff2": "font/woff2", ".ttf": "font/ttf", ".wasm": "application/wasm", ".css": "text/css" };
const server = createServer((req, res) => {
  const p = path.normalize(decodeURIComponent(new URL(req.url, "http://x").pathname));
  const file = path.join(WWW, p === "/" ? "index.html" : p);
  if (!file.startsWith(WWW) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404).end(); return; }
  res.writeHead(200, { "content-type": MIME[path.extname(file)] || "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
// Porta FIXA: o serializeAsJSON grava `source` = window.location.origin (medido) — porta 0 sorteava-a.
await new Promise((r) => server.listen(Number(process.env.ORACLE_PORT || 47118), "127.0.0.1", r));
const origin = `http://127.0.0.1:${server.address().port}`;

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

// ---- lado da página -------------------------------------------------------------------------
async function pageSide({ inputTxt, sondasTxt }) {
  const A = window.oracle.api;
  const b64 = window.oracle.blobToBase64;
  const input = JSON.parse(inputTxt);
  const sondas = JSON.parse(sondasTxt);
  const famValues = Object.values(A.FONT_FAMILY);
  // Aquecer as fontes (o run() do oracle.js regista as FontFace pelo canvas e espera-as).
  await window.oracle.run({ elements: famValues.map((f, i) => ({ type: "text", x: 0, y: i * 40, text: "Olá caixa Texto solto duas linhas abc xyz 123", fontFamily: f })) });
  const consts = { FONT_FAMILY: A.FONT_FAMILY, ROUNDNESS: A.ROUNDNESS, MIME_TYPES: A.MIME_TYPES };
  const cfg = { refreshDimensions: false, repairBindings: true };
  const clone = (x) => JSON.parse(JSON.stringify(x));

  // 1. restore do ficheiro inteiro + serialize + volta
  const restored = A.restore({ elements: input.elements, appState: input.appState, files: input.files }, null, null, cfg);
  const serialized = A.serializeAsJSON(restored.elements, restored.appState, restored.files, "local");
  const serializedDb = A.serializeAsJSON(restored.elements, restored.appState, restored.files, "database");
  const loadedJson = await A.loadFromBlob(new Blob([serialized], { type: "application/json" }), null, null);
  const reserialized = A.serializeAsJSON(loadedJson.elements, loadedJson.appState, loadedJson.files, "local");

  // 2. exports
  const elements = restored.elements.filter((e) => !e.isDeleted);
  const files = restored.files;
  const base = { exportBackground: true, viewBackgroundColor: "#ffffff", exportWithDarkMode: false };
  const svgOf = async (opts) => new XMLSerializer().serializeToString(await A.exportToSvg({ elements, files, exportPadding: 10, ...opts, appState: { ...base, ...(opts.appState || {}) } }));
  const pngOf = async (opts) => {
    const blob = await A.exportToBlob({ elements, files, mimeType: "image/png", exportPadding: 10, ...opts, appState: { ...base, exportScale: 2, ...(opts.appState || {}) } });
    return { blob, b64: await b64(blob), type: blob.type };
  };
  const frame = elements.find((e) => e.type === "frame");
  const svgVariants = {
    base: {},
    padding0: { exportPadding: 0 },
    padding50: { exportPadding: 50 },
    sem_fundo: { appState: { exportBackground: false } },
    escuro: { appState: { exportWithDarkMode: true } },
    fundo_cor: { appState: { viewBackgroundColor: "#ffec99" } },
    embed: { appState: { exportEmbedScene: true } },
    sem_fontes_inline: { skipInliningFonts: true },
    so_moldura: { exportingFrame: frame },
    render_embeddables: { renderEmbeddables: true },
  };
  const pngVariants = {
    base: {},
    escala1: { appState: { exportScale: 1 } },
    escala3: { appState: { exportScale: 3 } },
    sem_fundo: { appState: { exportBackground: false } },
    escuro: { appState: { exportWithDarkMode: true } },
    embed: { appState: { exportEmbedScene: true } },
    maxWidthOrHeight300: { maxWidthOrHeight: 300 },
    getDimensions_fixo: { getDimensions: () => ({ width: 400, height: 300, scale: 0.5 }) },
    getDimensions_escala2: { getDimensions: (w, h) => ({ width: w * 2, height: h * 2, scale: 2 }) },
    so_moldura: { exportingFrame: frame },
    jpeg: { mimeType: "image/jpeg", quality: 0.9 },
  };
  const svgs = {};
  for (const [k, o] of Object.entries(svgVariants)) {
    try { svgs[k] = await svgOf(o); } catch (e) { svgs[k] = { erro: String(e) }; }
  }
  const pngs = {};
  for (const [k, o] of Object.entries(pngVariants)) {
    try { const r = await pngOf(o); pngs[k] = { b64: r.b64, type: r.type }; } catch (e) { pngs[k] = { erro: String(e) }; }
  }
  // volta dos embutidos
  const volta = {};
  const summarize = (r) => ({
    n: r.elements.length,
    ids: r.elements.map((e) => e.id),
    igual_ao_restore: JSON.stringify(r.elements) === JSON.stringify(restored.elements),
    files: Object.keys(r.files || {}),
    appState_view: r.appState.viewBackgroundColor,
  });
  volta.json = summarize(loadedJson);
  volta.json.reserialize_igual = reserialized === serialized;
  for (const k of ["embed"]) {
    try {
      const pngBlob = await (await fetch("data:image/png;base64," + pngs[k].b64)).blob();
      volta.png_embed = summarize(await A.loadFromBlob(pngBlob, null, null));
    } catch (e) { volta.png_embed = { erro: String(e) }; }
    try { volta.svg_embed = summarize(await A.loadFromBlob(new Blob([svgs[k]], { type: "image/svg+xml" }), null, null)); }
    catch (e) { volta.svg_embed = { erro: String(e) }; }
    try { await A.loadFromBlob(new Blob([svgs.base], { type: "image/svg+xml" }), null, null); volta.svg_sem_embed = "carregou?!"; }
    catch (e) { volta.svg_sem_embed = { erro: String(e) }; }
  }
  // canvas: dimensões
  const canvasDims = {};
  for (const [k, o] of Object.entries({ padding10: { exportPadding: 10 }, padding0: { exportPadding: 0 } })) {
    const c = await A.exportToCanvas({ elements, files, ...o, appState: base });
    canvasDims[k] = [c.width, c.height];
  }

  // 3. sondas
  const sondaOut = [];
  for (const s of sondas.sondas_elementos) {
    const o = { nome: s.nome, opts: s.opts || cfg, entrada: s.elementos };
    if (s.via) o.via = s.via;
    try {
      o.saida = clone(s.via === "convert"
        ? A.convertToExcalidrawElements(clone(s.elementos), { regenerateIds: false })
        : A.restoreElements(clone(s.elementos), null, s.opts || cfg));
    }
    catch (e) { o.erro = String(e); }
    sondaOut.push(o);
  }
  const appDefault = clone(A.restoreAppState({}, null));
  const appOut = [];
  for (const s of sondas.sondas_appstate) {
    const o = { nome: s.nome, entrada: s.appState };
    try {
      const r = clone(A.restoreAppState(clone(s.appState), null));
      o.diferente_da_omissao = Object.fromEntries(Object.keys({ ...r, ...appDefault })
        .filter((k) => JSON.stringify(r[k]) !== JSON.stringify(appDefault[k])).map((k) => [k, r[k]]));
      o.serializado = JSON.parse(A.serializeAsJSON([], r, {}, "local")).appState;
      o.serializado_database = JSON.parse(A.serializeAsJSON([], r, {}, "database")).appState;
    } catch (e) { o.erro = String(e); }
    appOut.push(o);
  }
  // restore() inteiro de um ficheiro mínimo (sem appState, sem files, sem type)
  const minimo = clone(A.restore({ elements: [{ type: "rectangle", x: 0, y: 0, width: 10, height: 10 }] }, null, null));
  const minimoSer = JSON.parse(A.serializeAsJSON(minimo.elements, minimo.appState, minimo.files, "local"));

  return {
    consts, restoredElements: clone(restored.elements), restoredFiles: clone(restored.files),
    serialized, serializedDb, volta, svgs, pngs, canvasDims,
    sondas: sondaOut, appDefault, appOut, minimoSer,
  };
}

// ---- utilitários Node ------------------------------------------------------------------------
function pngInfo(buf) {
  if (buf.slice(0, 8).toString("hex") !== "89504e470d0a1a0a") return { formato: buf.slice(0, 4).toString("hex") };
  const chunks = [];
  let o = 8, w, h;
  while (o < buf.length) {
    const len = buf.readUInt32BE(o), type = buf.slice(o + 4, o + 8).toString("latin1");
    const data = buf.slice(o + 8, o + 8 + len);
    const c = { type, len };
    if (type === "IHDR") { w = data.readUInt32BE(0); h = data.readUInt32BE(4); c.bitDepth = data[8]; c.colorType = data[9]; }
    if (type === "tEXt" || type === "iTXt" || type === "zTXt") {
      const z = data.indexOf(0);
      c.keyword = data.slice(0, z).toString("latin1");
      const rest = data.slice(z + 1).toString("latin1");
      c.inicio = rest.slice(0, 160);
    }
    if (type === "pHYs") c.ppu = [data.readUInt32BE(0), data.readUInt32BE(4), data[8]];
    chunks.push(c);
    o += 12 + len;
  }
  return { width: w, height: h, chunks };
}
function svgInfo(s) {
  const root = s.match(/<svg[^>]*>/)?.[0] || "";
  const attr = (n) => root.match(new RegExp(`\\s${n}="([^"]*)"`))?.[1];
  const meta = s.match(/<metadata>([\s\S]*?)<\/metadata>/)?.[1];
  return {
    width: attr("width"), height: attr("height"), viewBox: attr("viewBox"), filter: attr("filter"),
    bytes: Buffer.byteLength(s), font_faces: (s.match(/@font-face/g) || []).length,
    rect_fundo: (s.match(/<rect [^>]*fill="[^"]*"[^>]*>/) || [null])[0],
    metadata_inicio: meta ? meta.slice(0, 300) : null, metadata_bytes: meta ? meta.length : 0,
    clipPaths: (s.match(/<clipPath/g) || []).length, images: (s.match(/<image /g) || []).length,
    foreignObjects: (s.match(/<foreignObject/g) || []).length,
  };
}

const browser = await chromium.launch({ headless: true });
const chromiumVersion = browser.version();
const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC" });
const external = [], consoleMsgs = [];
await context.route("**/*", (route) => {
  const u = route.request().url();
  if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue();
  external.push(u);
  return route.abort();
});
await context.addInitScript(seedScript(seed));
const page = await context.newPage();
await page.clock.setFixedTime(FIXED_TIME);
page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") consoleMsgs.push(`${m.type()}: ${m.text()}`.slice(0, 300)); });
page.on("pageerror", (e) => consoleMsgs.push(`pageerror: ${e.message}`));
let R;
try {
  await page.goto(origin + "/", { timeout: 60000 });
  await page.waitForFunction(() => window.oracleReady === true && window.oracle.api.serializeAsJSON, null, { timeout: 60000 });
  R = await page.evaluate(pageSide, { inputTxt: inBytes.toString("utf8"), sondasTxt: sondasBytes.toString("utf8") });
} finally {
  await browser.close();
  server.close();
}

const outputs = {};
const write = (file, data) => {
  fs.writeFileSync(path.join(outDir, file), data);
  outputs[file] = sha(Buffer.from(data));
};
const hdr = `<!-- ph2d excalidraw-oracle W8: @excalidraw/excalidraw ${PKG.version} · chromium ${chromiumVersion} · entrada ${path.basename(input)} sha256 ${sha(inBytes)} · ver ${name}.meta.json -->\n`;
write(`${name}.restored.json`, JSON.stringify({ elements: R.restoredElements, files: R.restoredFiles }, null, 2) + "\n");
write(`${name}.serialized.excalidraw`, R.serialized);
write(`${name}.serialized_database.json`, R.serializedDb);
write(`${name}.svg`, hdr + R.svgs.base + "\n");
write(`${name}.embed.svg`, hdr + R.svgs.embed + "\n");
const exportSummary = { svg: {}, png: {}, canvas: R.canvasDims, volta: R.volta };
for (const [k, s] of Object.entries(R.svgs)) {
  exportSummary.svg[k] = typeof s === "string" ? { sha256: sha(Buffer.from(s)), ...svgInfo(s) } : s;
}
for (const [k, p] of Object.entries(R.pngs)) {
  if (p.erro) { exportSummary.png[k] = p; continue; }
  const buf = Buffer.from(p.b64, "base64");
  exportSummary.png[k] = { type: p.type, bytes: buf.length, sha256: sha(buf), ...pngInfo(buf) };
  if (k === "base" || k === "embed" || k === "escuro" || k === "sem_fundo") {
    const f = k === "base" ? `${name}.png` : `${name}.${k}.png`;
    if (buf.length <= PNG_MAX_BYTES) write(f, buf);
  }
}
write(`${name}.export.json`, JSON.stringify(exportSummary, null, 2) + "\n");
write("w8_sondas.json", JSON.stringify({ consts: R.consts, minimo_serializado: R.minimoSer, sondas_appstate: R.appOut, sondas_elementos: R.sondas }, null, 1) + "\n");
write("w8_appstate_restaurado.json", JSON.stringify(R.appDefault, null, 2) + "\n");
const meta = {
  oracle: "@excalidraw/excalidraw", package_version: PKG.version, package_license: PKG.license,
  chromium_version: chromiumVersion,
  playwright_version: JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/playwright/package.json"), "utf8")).version,
  date: new Date().toISOString(),
  input: path.basename(input), input_sha256: sha(inBytes),
  sondas: path.basename(sondasPath), sondas_sha256: sha(sondasBytes),
  deterministic_env: { prng: "mulberry32", seed_from: "sha256(entrada+sondas)[0..8]", fixed_time_utc: new Date(FIXED_TIME).toISOString() },
  api: ["restore(cfg {refreshDimensions:false, repairBindings:true})", "serializeAsJSON(local|database)", "loadFromBlob(json|png embed|svg embed)",
    "exportToSvg/exportToBlob/exportToCanvas (variantes no .export.json)", "restoreElements/restoreAppState nas sondas"],
  external_requests_blocked: external, console: consoleMsgs, outputs_sha256: outputs,
};
fs.writeFileSync(path.join(outDir, `${name}.meta.json`), JSON.stringify(meta, null, 2) + "\n");
console.log(`ok ${name}: ${R.restoredElements.length} elementos · ${R.sondas.length} sondas · ${Object.keys(outputs).length} ficheiros${external.length ? ` · BLOQUEADOS ${external.length}` : ""} · consola ${consoleMsgs.length}`);
