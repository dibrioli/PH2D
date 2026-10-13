// W20 — três perguntas ao Excalidraw 0.18.1 montado (corrido por script; o fonte não se lê), sobre um vídeo do
// YouTube colado, com a rede do YouTube deixada passar:
//   A — ARRASTAR × CLICAR: parado, seleccionado e activo; o activar decide-se no carregar ou no largar?
//   P — o PAN com o vídeo a tocar (espaço+arrastar, botão do meio, a mão, a roda) e o clique fora: desactiva?
//       o vídeo PAUSA? E meio fora da vista, todo fora da vista, de volta: o iframe fica? o tempo anda?
//   E — a DECORAÇÃO: que grupos o painel de propriedades mostra para um vídeo seleccionado; stroke/fundo/
//       tracejado/opacidade/cantos postos pela API — onde se desenham (à volta? por cima do iframe?).
// O estado do vídeo lê-se DENTRO do iframe do YouTube (o `<video>`: paused, currentTime).
// Uso: node sonda_w20.mjs [--so <prefixo>[,…]] [<pasta_saida>]
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
const outDir = args[0] || path.join(HERE, "saidas/w20");
fs.mkdirSync(outDir, { recursive: true });
const SCRIPT_SHA = createHash("sha256").update(fs.readFileSync(fileURLToPath(import.meta.url))).digest("hex");
const r3 = (v) => (typeof v === "number" ? Math.round(v * 1000) / 1000 : v);
const loadavg = () => os.loadavg().map((v) => r3(v));
const VID = "dQw4w9WgXcQ";
const LINK = `https://www.youtube.com/watch?v=${VID}`;

// ---- servidor local da página (o mesmo da W18) ----
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

// ---- lado da página ----
const pageMount = async () => {
  window.oracle.editorUnmount();
  await window.oracle.editorMount([]);
  window.oracle.editorApi().updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: 1 } } });
  return true;
};
const pageCola = ({ texto }) => {
  const dt = new DataTransfer();
  dt.setData("text/plain", texto);
  (document.activeElement || document).dispatchEvent(new ClipboardEvent("paste", { clipboardData: dt, bubbles: true, cancelable: true }));
  return true;
};
const pageSnap = () => {
  const api = window.oracle.editorApi();
  const a = api.getAppState();
  const iframes = [...document.querySelectorAll("iframe")].map((f) => {
    const r = f.getBoundingClientRect();
    const cs = getComputedStyle(f);
    const pai = f.parentElement ? getComputedStyle(f.parentElement) : null;
    return { src: (f.getAttribute("src") || "").slice(0, 120), rect: [r.x, r.y, r.width, r.height].map((v) => Math.round(v * 10) / 10), pointerEvents: cs.pointerEvents, pai_pointerEvents: pai?.pointerEvents ?? null, border: cs.border, borderRadius: cs.borderRadius, pai_borderRadius: pai?.borderRadius ?? null, pai_overflow: pai?.overflow ?? null };
  });
  // O painel de propriedades: os títulos dos grupos (as legendas) que aparecem.
  const grupos = [...document.querySelectorAll(".App-menu__left fieldset legend, .App-menu__left h3, .selected-shape-actions fieldset legend, .selected-shape-actions h3")]
    .map((n) => n.innerText.trim()).filter(Boolean);
  const e = api.getSceneElements().find((q) => q.type === "embeddable" && !q.isDeleted);
  return {
    el: e ? { id: e.id, x: r3(e.x), y: r3(e.y), w: r3(e.width), h: r3(e.height), strokeColor: e.strokeColor, backgroundColor: e.backgroundColor, strokeWidth: e.strokeWidth, strokeStyle: e.strokeStyle, fillStyle: e.fillStyle, opacity: e.opacity, roundness: e.roundness ?? null, roughness: e.roughness } : null,
    activeEmbeddable: a.activeEmbeddable ?? null, selected: Object.keys(a.selectedElementIds || {}),
    scroll: [r3(a.scrollX), r3(a.scrollY)], zoom: a.zoom?.value, tool: a.activeTool?.type ?? null,
    iframes, grupos_do_painel: grupos,
    cursor: document.querySelector("canvas.interactive")?.style.cursor ?? null,
  };
};
const pageRaf = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));

// ---- corredor ----
const browser = await chromium.launch({ headless: true, args: ["--autoplay-policy=no-user-gesture-required"] });
const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
const DEIXA = /^https?:\/\/([a-z0-9-]+\.)*(youtube\.com|youtube-nocookie\.com|youtu\.be|ytimg\.com|ggpht\.com|googlevideo\.com|google\.com|gstatic\.com|googleapis\.com|doubleclick\.net|googleusercontent\.com)(\/|:|$)/i;
let bloqueados = 0;
// A página serve-se como `https://ph2d.local/` (a da W19): de `http://127.0.0.1` o YouTube recusa o vídeo
// («This video is unavailable», medido) — o embed exige um Referer de uma origem https.
const PAGINA = "https://ph2d.local";
await context.route(`${PAGINA}/**`, async (route) => {
  const p = path.normalize(decodeURIComponent(new URL(route.request().url()).pathname));
  const file = path.join(WWW, p === "/" ? "index.html" : p);
  if (!file.startsWith(WWW) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) return route.fulfill({ status: 404, body: "" });
  return route.fulfill({ status: 200, contentType: MIME[path.extname(file)] || "application/octet-stream", body: fs.readFileSync(file) });
});
await context.route("**/*", (route) => {
  const u = route.request().url();
  if (u.startsWith(PAGINA)) return route.fallback();
  if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:") || DEIXA.test(u)) return route.continue();
  bloqueados++;
  return route.abort();
});
const page = await context.newPage();
page.setDefaultTimeout(120000);
await page.goto(PAGINA + "/", { timeout: 120000 });
await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 120000 });
await page.addScriptTag({ content: `window.r3 = ${r3.toString()};` });

const settle = async (ms = 150) => { await page.evaluate(pageRaf); await page.waitForTimeout(ms); };
// O `<video>` do tocador: o iframe do YouTube (`null` = não há iframe montado).
const tocador = async () => {
  const f = page.frames().find((q) => /youtube(-nocookie)?\.com\/embed/.test(q.url()));
  if (!f) return null;
  try {
    return await f.evaluate(() => { const v = document.querySelector("video"); return v ? { paused: v.paused, t: Math.round(v.currentTime * 100) / 100, ready: v.readyState, ad: !!document.querySelector(".ad-showing") } : { sem_video: true }; });
  } catch (e) { return { erro: String(e.message || e).slice(0, 120) }; }
};
// O tempo anda? (duas leituras com `ms` entre elas)
const anda = async (ms = 1500) => {
  const a = await tocador(); await page.waitForTimeout(ms); const b = await tocador();
  return { antes: a, depois: b, anda: !!(a && b && typeof a.t === "number" && b.t > a.t + 0.2) };
};
let sondaAtual = null;
async function mede(nome, extra = {}, { foto = false } = {}) {
  await settle(150);
  const s = await page.evaluate(pageSnap);
  const passo = { passo: nome, ...extra, ...s };
  if (foto) {
    const fname = `w20_${sondaAtual.nome}_${String(sondaAtual.passos.length).padStart(2, "0")}_${nome}.png`;
    fs.writeFileSync(path.join(outDir, fname), await page.screenshot({ clip: { x: 0, y: 0, width: 1280, height: 800 } }));
    passo.foto = fname;
  }
  sondaAtual.passos.push(passo);
  return s;
}
const SONDAS = [];
const sonda = (nome, pergunta, fn) => SONDAS.push({ nome, pergunta, fn });

const VAZIO = [5, 790];
const umVideo = async () => {
  await page.mouse.move(640, 400); await page.mouse.click(640, 400); await settle(100);
  await page.evaluate(pageCola, { texto: LINK });
  await page.waitForTimeout(6000);
  await page.keyboard.press("Escape"); await page.mouse.click(...VAZIO); await settle(300);
  return (await page.evaluate(pageSnap)).el;
};
const meio = (e) => [Math.round(e.x + e.w / 2), Math.round(e.y + e.h / 2)];
// Um ponto DENTRO da caixa mas longe do play do meio (o quarto de cima à esquerda).
const quarto = (e) => [Math.round(e.x + e.w / 4), Math.round(e.y + e.h / 4)];
const arrasta = async (de, d, { button = "left", passos = 12, meioDoGesto = null } = {}) => {
  await page.mouse.move(...de); await page.mouse.down({ button }); await settle(60);
  if (meioDoGesto) await meioDoGesto("carregou");
  await page.mouse.move(de[0] + d[0], de[1] + d[1], { steps: passos }); await settle(60);
  if (meioDoGesto) await meioDoGesto("moveu");
  await page.mouse.up({ button }); await settle(200);
};
// Activa e põe a tocar: seleccionar, clicar no meio (activa), clicar no meio de novo (o play do YouTube).
const aTocar = async (e) => {
  await page.mouse.click(...quarto(e)); await settle(200);
  await page.mouse.click(...meio(e)); await settle(2500);
  const s = await page.evaluate(pageSnap);
  let t = await tocador();
  if (!t || t.paused !== false) { await page.mouse.click(...meio(e)); await page.waitForTimeout(4000); t = await tocador(); }
  return { activo: s.activeEmbeddable, tocador: t };
};

// ---- A ----
sonda("A1_arrasta_parado", "Vídeo NÃO seleccionado: carregar no play do meio e arrastar +150,+80. Move? Activa?", async () => {
  const e = await umVideo();
  await mede("antes", {}, { foto: true });
  await arrasta(meio(e), [150, 80], { meioDoGesto: (q) => mede(q) });
  await mede("largou", {}, { foto: true });
});
sonda("A2_clique_seleccionado", "Vídeo SELECCIONADO: carregar no meio (sem mexer), medir, largar, medir. O activar é no carregar ou no largar?", async () => {
  const e = await umVideo();
  await page.mouse.click(...quarto(e)); await settle(200);
  await mede("seleccionado");
  await page.mouse.move(...meio(e)); await settle(200);
  await mede("por_cima");
  // Carregar e medir SEM esperar (o estado logo a seguir ao Down), largar depressa.
  await page.mouse.down();
  const noDown = await page.evaluate(() => window.oracle.editorApi().getAppState().activeEmbeddable?.state ?? null);
  await page.mouse.up(); await settle(1500);
  await mede("rapido_largou", { estado_logo_apos_o_down: noDown, tocador: await tocador() }, { foto: true });
  // De novo, parado e seleccionado: carregar 600 ms quieto e largar.
  await page.keyboard.press("Escape"); await page.mouse.click(...VAZIO); await settle(300);
  await page.mouse.click(...quarto(e)); await settle(300);
  await page.mouse.move(...meio(e)); await settle(200);
  await page.mouse.down(); await page.waitForTimeout(600);
  await mede("lento_carregado");
  await page.mouse.up(); await settle(1500);
  await mede("lento_largou");
  // E o clique no quarto (fora do play do meio), seleccionado: activa?
  await page.keyboard.press("Escape"); await page.mouse.click(...VAZIO); await settle(300);
  await page.mouse.click(...quarto(e)); await settle(300);
  await page.mouse.click(...quarto(e)); await settle(1500);
  await mede("clique_no_quarto_seleccionado");
});
sonda("A3_arrasta_seleccionado", "Vídeo SELECCIONADO: carregar no meio e arrastar +150,+80 — move ou activa? E carregar fora do play (quarto) e arrastar?", async () => {
  const e = await umVideo();
  await page.mouse.click(...quarto(e)); await settle(200);
  await arrasta(meio(e), [150, 80], { meioDoGesto: (q) => mede(`meio_${q}`) });
  const e2 = (await mede("meio_largou")).el;
  await arrasta(quarto(e2), [-150, -80], { meioDoGesto: (q) => mede(`quarto_${q}`) });
  await mede("quarto_largou");
});
sonda("A4_arrasta_activo", "Vídeo ACTIVO e a tocar: arrastar de DENTRO (quarto) +150,+80; arrastar pela BORDA (2 px dentro da caixa); pelas pegas? Move?", async () => {
  const e = await umVideo();
  const r = await aTocar(e);
  await mede("a_tocar", r, { foto: true });
  await arrasta(quarto(e), [150, 80], { meioDoGesto: (q) => mede(`dentro_${q}`) });
  await mede("dentro_largou", { tocador: await tocador() });
  const e2 = (await page.evaluate(pageSnap)).el;
  // A borda de cima (2 px abaixo do topo, a meio): a única parte da caixa que pode não ser do iframe.
  const borda = [Math.round(e2.x + e2.w / 2), Math.round(e2.y + 2)];
  await arrasta(borda, [100, 60], { meioDoGesto: (q) => mede(`borda_${q}`) });
  await mede("borda_largou", { tocador: await tocador() }, { foto: true });
});

// ---- P ----
const PAN = [
  ["espaco", async (de, d) => { await page.keyboard.down(" "); await settle(60); await arrasta(de, d); await page.keyboard.up(" "); }],
  ["meio", async (de, d) => { await arrasta(de, d, { button: "middle" }); }],
  ["mao", async (de, d) => { await page.keyboard.press("h"); await settle(100); await arrasta(de, d); await page.keyboard.press("v"); }],
  ["roda", async (_de, d) => { await page.mouse.move(640, 400); await page.mouse.wheel(-d[0], -d[1]); }],
];
for (const [nome, faz] of PAN) {
  for (const onde of ["vazio", "sobre"]) {
    sonda(`P_${nome}_${onde}`, `Vídeo activo a tocar; pan «${nome}» começado ${onde === "vazio" ? "no quadro vazio" : "SOBRE o vídeo (quarto)"} −120,−60: desactiva? pausa?`, async () => {
      const e = await umVideo();
      const r = await aTocar(e);
      await mede("a_tocar", { ...r, anda: await anda() });
      const de = onde === "vazio" ? [Math.max(20, e.x - 80), Math.max(80, e.y - 40)] : quarto(e);
      await faz(de, [-120, -60]);
      await mede("depois_do_pan", { anda: await anda(2000) }, { foto: true });
    });
  }
}
sonda("P_clique_fora", "Vídeo activo a tocar; clique no quadro vazio: desactiva? o vídeo PAUSA?", async () => {
  const e = await umVideo();
  const r = await aTocar(e);
  await mede("a_tocar", { ...r, anda: await anda() });
  await page.mouse.click(...VAZIO); await settle(300);
  await mede("clicou_fora", { anda: await anda(2000) }, { foto: true });
});
sonda("P_fora_da_vista", "Vídeo a tocar; a vista leva-o meio fora, todo fora, e de volta (scroll pela API): o iframe fica? o tempo anda?", async () => {
  const e = await umVideo();
  const r = await aTocar(e);
  await mede("a_tocar", { ...r, anda: await anda() });
  const s0 = (await page.evaluate(pageSnap)).scroll;
  const vai = async (dx) => page.evaluate(([x, y]) => window.oracle.editorApi().updateScene({ appState: { scrollX: x, scrollY: y } }), [s0[0] + dx, s0[1]]);
  // A caixa começa em e.x; deixar metade à vista: a borda direita no x da metade da largura.
  await vai(-(e.x + e.w / 2)); await settle(300);
  await mede("meio_fora", { anda: await anda(2000) }, { foto: true });
  await vai(-(e.x + e.w - 150)); await settle(300);
  await mede("so_150px_a_vista", { anda: await anda(2000) });
  await vai(-(e.x + e.w + 200)); await settle(500);
  await mede("todo_fora", { anda: await anda(2000) }, { foto: true });
  await page.waitForTimeout(3000);
  await vai(0); await settle(800);
  await mede("de_volta", { anda: await anda(2500) }, { foto: true });
});

// ---- E ----
sonda("E1_painel", "Vídeo seleccionado: que grupos tem o painel de propriedades? (comparado com um rectângulo)", async () => {
  const e = await umVideo();
  await page.mouse.click(...quarto(e)); await settle(300);
  await mede("video_seleccionado", {}, { foto: true });
  await page.keyboard.press("Escape"); await page.mouse.click(...VAZIO);
  await page.keyboard.press("r"); await arrasta([100, 100], [120, 80]); await settle(200);
  await mede("rectangulo_seleccionado", {}, { foto: true });
});
sonda("E2_estilo", "Vídeo com stroke vermelho 4 tracejado, fundo amarelo sólido, opacidade 50, cantos rectos/redondos (API): onde se desenha cada um? E activo?", async () => {
  const e = await umVideo();
  const poe = (p) => page.evaluate(([id, p]) => {
    const api = window.oracle.editorApi();
    api.updateScene({ elements: api.getSceneElements().map((q) => (q.id === id ? { ...q, ...p, version: q.version + 1, versionNonce: q.versionNonce + 1 } : q)) });
  }, [e.id, p]);
  await poe({ strokeColor: "#e03131", strokeWidth: 4, strokeStyle: "dashed", backgroundColor: "#ffec99", fillStyle: "solid" });
  await mede("stroke_fundo", {}, { foto: true });
  await poe({ roundness: null });
  await mede("cantos_rectos", {}, { foto: true });
  await poe({ roundness: { type: 3 }, opacity: 50 });
  await mede("opacidade_50", {}, { foto: true });
  // O `.excalidraw` que ELE grava com o vídeo decorado (stroke vermelho 4 tracejado, fundo amarelo, opacidade 60,
  // cantos rectos): a entrada do gate de leitura (`ph2d-board-excalidraw::video_tests`).
  await poe({ opacity: 60, roundness: null });
  const json = await page.evaluate(() => {
    const api = window.oracle.editorApi();
    return window.oracle.api.serializeAsJSON(api.getSceneElements(), api.getAppState(), api.getFiles(), "local");
  });
  fs.writeFileSync(path.join(outDir, "w20_E2_gravado.excalidraw"), json);
  await mede("gravado", { ficheiro: "w20_E2_gravado.excalidraw" });
  await poe({ opacity: 100, roundness: { type: 3 } });
  const r = await aTocar((await page.evaluate(pageSnap)).el);
  await mede("activo_a_tocar", r, { foto: true });
});

// ---- corre ----
const L0 = loadavg();
const res = [];
let falhas = 0;
for (const s of SONDAS) {
  if (so && !so.split(",").some((q) => s.nome.startsWith(q))) continue;
  sondaAtual = { nome: s.nome, passos: [] };
  const t = Date.now();
  try {
    await page.mouse.move(...VAZIO);
    await page.evaluate(pageMount); await settle(300);
    await s.fn();
    res.push({ sonda: s.nome, pergunta: s.pergunta, passos: sondaAtual.passos, ms: Date.now() - t, load: loadavg() });
    console.log(`ok ${s.nome} (${sondaAtual.passos.length} passos, ${Date.now() - t} ms, load ${loadavg()[0]})`);
  } catch (e) {
    falhas++;
    console.error(`FALHA ${s.nome}: ${e.stack || e.message}`);
    res.push({ sonda: s.nome, pergunta: s.pergunta, falha: String(e.stack || e.message), passos: sondaAtual.passos });
  }
  for (const k of [" ", "Shift"]) await page.keyboard.up(k).catch(() => {});
  await page.keyboard.press("Escape").catch(() => {});
}
await browser.close();
server.close();
const nomeSaida = `w20_sondas${so ? `.so_${so.replace(/,/g, "_")}` : ""}.json`;
fs.writeFileSync(path.join(outDir, nomeSaida), JSON.stringify({
  oraculo: "@excalidraw/excalidraw", versao: PKG.version, licenca: PKG.license, playwright: PW.version,
  data: new Date().toISOString(), loadavg_inicio: L0, loadavg_fim: loadavg(),
  comando: `node sonda_w20.mjs${so ? ` --so ${so}` : ""}`, script_sha256: SCRIPT_SHA, link: LINK,
  ambiente: { janela: [1280, 800], deviceScaleFactor: 1, rede: "só os domínios do YouTube/Google", pedidos_bloqueados: bloqueados },
  falhas, sondas: res,
}, null, 1) + "\n");
console.log(`gravado ${path.join(outDir, nomeSaida)} (${falhas} falhas)`);
process.exit(falhas ? 1 : 0);
