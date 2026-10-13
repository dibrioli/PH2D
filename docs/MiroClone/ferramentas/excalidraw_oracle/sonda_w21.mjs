// W21 — duas perguntas ao Excalidraw 0.18.1 montado (corrido por script; o fonte não se lê), sobre um vídeo do
// YouTube colado (a montagem da W20, `sonda_w20.mjs`):
//   T — o CLIQUE LONGO: vídeo seleccionado, carregar no meio parado N ms e largar — a partir de que N deixa de
//       activar? A duração conta-se no RELÓGIO DA PÁGINA (`event.timeStamp` do pointerdown/pointerup), não no
//       do Node: com a máquina carregada o `waitForTimeout` estica.
//   F — OUTRA FERRAMENTA na mão (rectângulo, elipse, losango, seta, linha, lápis, texto, borracha, laser, mão,
//       moldura): um clique sem mexer e um arrasto começados no meio de um vídeo PARADO, e um clique num vídeo
//       ACTIVO a tocar. Cria um elemento por cima? Activa? Pausa? Que ferramenta fica na mão depois?
// Uso: node sonda_w21.mjs [--so <prefixo>[,…]] [<pasta_saida>]
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
const outDir = args[0] || path.join(HERE, "saidas/w21");
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
    // Os OUTROS elementos (o que a ferramenta criou) e o editor de texto aberto.
    outros: api.getSceneElements().filter((q) => !q.isDeleted && q.type !== "embeddable")
      .map((q) => ({ type: q.type, x: r3(q.x), y: r3(q.y), w: r3(q.width), h: r3(q.height), pontos: q.points?.length ?? null, frameId: q.frameId ?? null })),
    embeddable_apagado: api.getSceneElements().some((q) => q.type === "embeddable" && q.isDeleted),
    editor_de_texto: !!document.querySelector("textarea.excalidraw-wysiwyg"),
    toque: window.__toque ?? null,
  };
};
// O relógio da página: a duração do último carregar→largar no canvas (`event.timeStamp`, ms).
const pageRelogio = () => {
  window.__toque = null;
  if (window.__relogio) return true;
  window.__relogio = true;
  let down = null;
  window.addEventListener("pointerdown", (e) => { down = e.timeStamp; }, true);
  window.addEventListener("pointerup", (e) => { if (down !== null) window.__toque = { ms: Math.round((e.timeStamp - down) * 10) / 10 }; down = null; }, true);
  return true;
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
    const fname = `w21_${sondaAtual.nome}_${String(sondaAtual.passos.length).padStart(2, "0")}_${nome}.png`;
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

const desactiva = async () => { await page.keyboard.press("Escape"); await page.mouse.click(...VAZIO); await settle(300); };
const estado = async () => page.evaluate(() => window.oracle.editorApi().getAppState().activeEmbeddable?.state ?? null);
// Carregar parado `ms` no ponto `p` e largar; a duração REAL vem do relógio da página.
const segura = async (p, ms) => {
  await page.evaluate(pageRelogio);
  await page.mouse.move(...p); await settle(150);
  await page.mouse.down(); await page.waitForTimeout(ms); await page.mouse.up();
  await settle(1200);
};

// ---- T ----
const TEMPOS = [80, 150, 200, 250, 280, 300, 320, 350, 400, 450, 500, 600, 800];
sonda("T0_nao_seleccionado", "Vídeo NÃO seleccionado: clique rápido no meio e carregar 600 ms parado — activa?", async () => {
  const e = await umVideo();
  for (const ms of [60, 600]) {
    await desactiva();
    await segura(meio(e), ms);
    await mede(`nao_seleccionado_${ms}ms`, { estado: await estado() });
  }
});
sonda("T1_tempo_parado", "Vídeo SELECCIONADO: carregar no meio parado N ms e largar (N a subir e depois a descer) — activa?", async () => {
  const e = await umVideo();
  for (const ms of [...TEMPOS, ...[...TEMPOS].reverse()]) {
    await desactiva();
    await page.mouse.click(...quarto(e)); await settle(250);
    await segura(meio(e), ms);
    await mede(`segurou_${ms}ms`, { pedido_ms: ms, estado: await estado() });
  }
});

// ---- F ----
const FERRAMENTAS = ["rectangle", "ellipse", "diamond", "arrow", "line", "freedraw", "text", "eraser", "laser", "hand", "frame"];
const naMao = (type) => page.evaluate((t) => window.oracle.editorApi().setActiveTool({ type: t }), type);
const limpaOutros = () => page.evaluate(() => {
  const api = window.oracle.editorApi();
  api.updateScene({ elements: api.getSceneElements().filter((q) => q.type === "embeddable") });
});
for (const f of FERRAMENTAS) {
  sonda(`F_${f}_parado`, `Vídeo PARADO, «${f}» na mão: clique sem mexer no meio; depois arrastar do meio +120,+60. Cria por cima? Activa?`, async () => {
    const e = await umVideo();
    await naMao(f); await settle(150);
    await mede("na_mao");
    await segura(meio(e), 60);
    await mede("clique", { estado: await estado() }, { foto: true });
    await page.keyboard.press("Escape"); await page.keyboard.press("Escape"); await settle(150);
    await limpaOutros(); await desactiva();
    await naMao(f); await settle(150);
    await arrasta(meio(e), [120, 60], { meioDoGesto: (q) => mede(`arrasto_${q}`, { estado_: null }) });
    await mede("arrasto_largou", { estado: await estado() }, { foto: true });
  });
  sonda(`F_${f}_activo`, `Vídeo ACTIVO a tocar, «${f}» na mão: clique no quarto; depois arrastar do quarto +120,+60. Cria por cima? Desactiva? Pausa?`, async () => {
    const e = await umVideo();
    const r = await aTocar(e);
    await mede("a_tocar", { ...r, anda: await anda() });
    await naMao(f); await settle(150);
    await mede("na_mao", { estado: await estado(), anda: await anda() });
    await segura(quarto(e), 60);
    await mede("clique", { estado: await estado(), anda: await anda() }, { foto: true });
    await arrasta(quarto(e), [120, 60]);
    await mede("arrasto_largou", { estado: await estado(), anda: await anda() }, { foto: true });
  });
}

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
const nomeSaida = `w21_sondas${so ? `.so_${so.replace(/,/g, "_")}` : ""}.json`;
fs.writeFileSync(path.join(outDir, nomeSaida), JSON.stringify({
  oraculo: "@excalidraw/excalidraw", versao: PKG.version, licenca: PKG.license, playwright: PW.version,
  data: new Date().toISOString(), loadavg_inicio: L0, loadavg_fim: loadavg(),
  comando: `node sonda_w21.mjs${so ? ` --so ${so}` : ""}`, script_sha256: SCRIPT_SHA, link: LINK,
  ambiente: { janela: [1280, 800], deviceScaleFactor: 1, rede: "só os domínios do YouTube/Google", pedidos_bloqueados: bloqueados },
  falhas, sondas: res,
}, null, 1) + "\n");
console.log(`gravado ${path.join(outDir, nomeSaida)} (${falhas} falhas)`);
process.exit(falhas ? 1 : 0);
