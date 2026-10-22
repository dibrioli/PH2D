// W23 (decisão do dono 22/10: Rascunho/Final também na barra da FORMA) — o Excalidraw 0.18.1 montado, corrido
// por script (o fonte não se lê): para que tipos de selecção o painel de propriedades mostra «Sloppiness»,
// onde (a ordem dos grupos), que opção acende numa selecção MISTA, e o que o clique faz a uma selecção mista
// (com um tipo que não tem a opção).
// Uso: node sonda_w23_sloppiness.mjs [<pasta_saida>]
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle");
const outDir = process.argv[2] || path.join(HERE, "saidas/w23_sloppiness");
fs.mkdirSync(outDir, { recursive: true });
const SCRIPT_SHA = createHash("sha256").update(fs.readFileSync(fileURLToPath(import.meta.url))).digest("hex");

const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const PKG = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/@excalidraw/excalidraw/package.json"), "utf8"));
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

// A cena: um de cada tipo, dois rectângulos com roughness diferente (para a selecção mista).
const pageMount = async () => {
  const { convertToExcalidrawElements, restoreElements } = window.oracle.api;
  const sk = convertToExcalidrawElements([
    { type: "rectangle", id: "rect", x: 20, y: 120, width: 120, height: 80, roughness: 1 },
    { type: "rectangle", id: "rect0", x: 20, y: 240, width: 120, height: 80, roughness: 0 },
    { type: "ellipse", id: "ell", x: 180, y: 120, width: 120, height: 80 },
    { type: "diamond", id: "dia", x: 340, y: 120, width: 120, height: 80 },
    { type: "arrow", id: "arr", x: 500, y: 160, width: 120, height: 0 },
    { type: "line", id: "lin", x: 500, y: 260, width: 120, height: 40 },
    { type: "text", id: "txt", x: 680, y: 140, text: "nota" },
    { type: "frame", id: "frm", x: 20, y: 400, width: 200, height: 150, children: [] },
    { type: "embeddable", id: "emb", x: 680, y: 400, width: 200, height: 120, link: "https://example.com" },
  ], { regenerateIds: false });
  const extra = restoreElements([
    { type: "freedraw", id: "ink", x: 340, y: 260, width: 100, height: 40, points: [[0, 0], [30, 20], [60, 5], [100, 40]], pressures: [], simulatePressure: true, strokeColor: "#1e1e1e", backgroundColor: "transparent", fillStyle: "solid", strokeWidth: 2, strokeStyle: "solid", roughness: 1, opacity: 100, angle: 0, seed: 1, version: 1, versionNonce: 1, isDeleted: false, groupIds: [], frameId: null, roundness: null, boundElements: null, updated: 1, link: null, locked: false },
    { type: "image", id: "img", x: 340, y: 400, width: 120, height: 120, fileId: "f1", status: "saved", scale: [1, 1], strokeColor: "transparent", backgroundColor: "transparent", fillStyle: "solid", strokeWidth: 2, strokeStyle: "solid", roughness: 1, opacity: 100, angle: 0, seed: 2, version: 1, versionNonce: 2, isDeleted: false, groupIds: [], frameId: null, roundness: null, boundElements: null, updated: 1, link: null, locked: false },
  ], null);
  window.oracle.editorUnmount();
  await window.oracle.editorMount([...sk, ...extra]);
  window.oracle.editorApi().updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: 1 } } });
  return window.oracle.editorApi().getSceneElements().map((e) => e.id);
};
const pageSelect = (ids) => { window.oracle.editorSelect(ids); return true; };
const pageSnap = () => {
  const nome = (b) => b.closest("label")?.getAttribute("title") || b.getAttribute("title") || b.getAttribute("aria-label") || b.closest("label")?.innerText.trim() || b.value;

  const api = window.oracle.editorApi();
  const root = document.querySelector(".App-menu__left") || document.querySelector(".selected-shape-actions") || document;
  const grupos = [...root.querySelectorAll("fieldset legend, h3")].map((n) => n.innerText.trim()).filter(Boolean);
  const fs_ = [...root.querySelectorAll("fieldset")].find((f) => /sloppiness/i.test(f.querySelector("legend")?.innerText || ""));
  const opcoes = fs_ ? [...fs_.querySelectorAll("input, button")].map((b) => ({ titulo: nome(b), marcado: !!(b.checked || b.getAttribute("aria-checked") === "true" || b.classList.contains("active") || b.parentElement?.classList.contains("active")) })) : null;
  const sel = Object.keys(api.getAppState().selectedElementIds || {});
  const rough = Object.fromEntries(api.getSceneElements().filter((e) => sel.includes(e.id)).map((e) => [e.id, e.roughness]));
  return { grupos, sloppiness: opcoes, roughness_dos_seleccionados: rough };
};
const pageClick = (titulo) => {
  const nome = (b) => b.closest("label")?.getAttribute("title") || b.getAttribute("title") || b.getAttribute("aria-label") || b.closest("label")?.innerText.trim() || b.value;

  const root = document.querySelector(".App-menu__left") || document;
  const fs_ = [...root.querySelectorAll("fieldset")].find((f) => /sloppiness/i.test(f.querySelector("legend")?.innerText || ""));
  if (!fs_) return "sem Sloppiness no painel";
  const b = [...fs_.querySelectorAll("input, button")].find((q) => nome(q) === titulo);
  if (!b) return "sem a opção " + titulo;
  b.click();
  return "clicou";
};
const pageRaf = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
await context.route("**/*", (route) => {
  const u = route.request().url();
  if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue();
  return route.abort();
});
const page = await context.newPage();
await page.goto(origin + "/", { timeout: 120000 });
await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 120000 });
const settle = async () => { await page.evaluate(pageRaf); await page.waitForTimeout(150); };

const ids = await page.evaluate(pageMount);
await settle();
const casos = [
  ["rectangulo", ["rect"]], ["elipse", ["ell"]], ["losango", ["dia"]], ["seta", ["arr"]], ["linha", ["lin"]],
  ["texto_nota", ["txt"]], ["tinta_livre", ["ink"]], ["imagem", ["img"]], ["frame", ["frm"]], ["video_embeddable", ["emb"]],
  ["misto_rough1_rough0", ["rect", "rect0"]], ["rect_mais_tinta", ["rect", "ink"]], ["rect_mais_texto", ["rect", "txt"]],
  ["rect_mais_imagem", ["rect", "img"]], ["rect_mais_frame", ["rect", "frm"]],
];
const passos = [];
for (const [nome, sel] of casos) {
  await page.evaluate(pageSelect, sel); await settle();
  const s = await page.evaluate(pageSnap);
  const passo = { caso: nome, seleccao: sel, ...s };
  if (nome === "rectangulo" || nome === "misto_rough1_rough0") {
    const f = `w23_${nome}.png`;
    fs.writeFileSync(path.join(outDir, f), await page.screenshot());
    passo.foto = f;
  }
  passos.push(passo);
}
// O clique numa selecção MISTA: o que fica? (re-montada a cada caso)
const cliques = [];
for (const [nome, sel, titulo] of [
  ["misto_clica_Artist", ["rect", "rect0"], "Artist"],
  ["misto_clica_Architect", ["rect", "rect0"], "Architect"],
  ["rect_mais_tinta_clica_Architect", ["rect", "ink"], "Architect"],
  ["rect_mais_texto_clica_Architect", ["rect", "txt"], "Architect"],
  ["rect_ja_Artist_clica_Artist", ["rect"], "Artist"],
]) {
  await page.evaluate(pageMount); await settle();
  await page.evaluate(pageSelect, sel); await settle();
  const antes = await page.evaluate(pageSnap);
  const clique = await page.evaluate(pageClick, titulo); await settle();
  const depois = await page.evaluate(pageSnap);
  cliques.push({ caso: nome, seleccao: sel, clicou: titulo, clique, antes, depois });
}
const out = {
  cabecalho: { sonda: "sonda_w23_sloppiness.mjs", sha256_do_script: SCRIPT_SHA, excalidraw: PKG.version, data: new Date().toISOString(), loadavg: os.loadavg(), ids_da_cena: ids },
  passos, cliques,
};
fs.writeFileSync(path.join(outDir, "w23_sloppiness.json"), JSON.stringify(out, null, 1));
console.log(JSON.stringify(out, null, 1));
await browser.close();
server.close();
