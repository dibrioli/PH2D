// Oráculo de MAPA MENTAL: corre mind-elixir, simple-mind-map e markmap num Chromium headless sobre as
// árvores NOSSAS (entradas/*.json), mede o DOM/SVG em execução e carrega no teclado com o teclado e o
// rato REAIS do Playwright. Só API pública documentada (README / docs / .d.ts de cada pacote):
//   mind-elixir: new MindElixir({el,direction,toolBar,contextMenu}), init, getData, findEle, scaleVal,
//                currentNodes (Topic.nodeObj), allowUndo, refresh, destroy;
//   simple-mind-map: new MindMap({el,layout,data}), getData, getConfig, renderer.findNodeByUid,
//                renderer.activeNodeList, Node.left/top/width/height/uid/group/getRect, view.scale,
//                reRender, destroy, evento node_tree_render_end;
//   markmap: Transformer (markmap-lib), Markmap.create(svg,{duration,autoFit},data), state.data,
//                INode.state.rect/path, payload.fold, setData, destroy, defaultOptions.
// Uso: node corre.mjs [--so caso1,caso2] <pasta_saida>
// Por caso: <saida>/<caso>.json + <caso>.meta.json; no fim, se todos os casos de layout correram,
// <saida>/leis.json (as leis destiladas). Janela 1600×1000, deviceScaleFactor 1.
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { destilaLeis } from "./leis.mjs";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/mindmap-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const ver = (p) => JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules", p, "package.json"), "utf8")).version;
const VERSOES = { "mind-elixir": ver("mind-elixir"), "simple-mind-map": ver("simple-mind-map"), "markmap-view": ver("markmap-view"), "markmap-lib": ver("markmap-lib"), playwright: ver("playwright") };
const HERE = path.dirname(fileURLToPath(import.meta.url));
const SELF = fileURLToPath(import.meta.url);
const sha = (b) => createHash("sha256").update(b).digest("hex");
const SELF_SHA = sha(fs.readFileSync(SELF));
const LEIS_SHA = sha(fs.readFileSync(path.join(HERE, "leis.mjs")));
const ENTRADAS = Object.fromEntries(["T1", "T2", "T3", "T4", "T5", "T6", "N100", "N1000"].map((k) => [k, JSON.parse(fs.readFileSync(path.join(HERE, "entradas", `${k}.json`), "utf8"))]));
const ENTRADAS_SHA = Object.fromEntries(Object.keys(ENTRADAS).map((k) => [k, sha(fs.readFileSync(path.join(HERE, "entradas", `${k}.json`)))]));

let args = process.argv.slice(2);
const opt = (flag) => { const i = args.indexOf(flag); if (i < 0) return null; const v = args[i + 1]; args.splice(i, 2); return v; };
const so = opt("--so");
const [outDir] = args;
if (!outDir) { console.error("uso: node corre.mjs [--so caso1,caso2] <pasta_saida>"); process.exit(2); }
fs.mkdirSync(outDir, { recursive: true });

const r3 = (v) => (v == null ? v : Math.round(v * 1000) / 1000 || 0);

// ================================================================ lado da página (injectado)
function pageLib() {
  const r3 = (v) => Math.round(v * 1000) / 1000 || 0;
  const frames = (n) => new Promise((res) => { const f = () => (n-- <= 0 ? res() : requestAnimationFrame(f)); requestAnimationFrame(f); });
  const H = (window.H = { app: null });
  const host = () => { const old = document.getElementById("host"); const h = document.createElement("div"); h.id = "host"; old.replaceWith(h); return h; };
  const toME = (t) => ({ id: t.id, topic: t.text, ...(t.collapsed ? { expanded: false } : {}), children: t.children.map(toME) });
  const toSMM = (t) => ({ data: { text: t.text, uid: t.id, ...(t.collapsed ? { expand: false } : {}) }, children: t.children.map(toSMM) });
  const toMM = (t) => ({ content: t.text.replace(/\n/g, "<br>"), payload: { pid: t.id, ...(t.collapsed ? { fold: 1 } : {}) }, children: t.children.map(toMM) });
  const strip = (s) => String(s ?? "").replace(/<br\s*\/?>/gi, "\n").replace(/<\/p>\s*<p>/gi, "\n").replace(/<[^>]+>/g, "").replace(/&nbsp;/g, " ");

  H.monta = async ({ app, tree, layout, opts = {} }) => {
    const el = host();
    if (app === "me") {
      const { MindElixir } = window.oracle;
      const mind = new MindElixir({ el, direction: MindElixir[layout], toolBar: false, contextMenu: false, ...opts });
      await mind.init({ nodeData: toME(tree) });
      H.app = { kind: "me", mind };
    } else if (app === "smm") {
      const { MindMap } = window.oracle;
      const mm = new MindMap({ el, layout, data: toSMM(tree), ...opts });
      H.app = { kind: "smm", mm };
    } else {
      const { Markmap } = window.oracle;
      const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
      svg.setAttribute("width", "1600"); svg.setAttribute("height", "1000");
      el.appendChild(svg);
      const mm = Markmap.create(svg, { duration: 0, autoFit: false, ...opts }, toMM(tree));
      H.app = { kind: "mm", mm, svg };
    }
    await H.estavel();
    return H.opcoes();
  };
  H.opcoes = () => {
    const a = H.app;
    if (a.kind === "me") return { direction: a.mind.direction, allowUndo: a.mind.allowUndo, keypress: !!a.mind.keypress, editable: a.mind.editable, scaleVal: a.mind.scaleVal, alignment: a.mind.alignment, compact: a.mind.compact };
    if (a.kind === "smm") { const c = a.mm.getConfig(); return { layout: c.layout, readonly: c.readonly, enableShortcutOnlyWhenMouseInSvg: c.enableShortcutOnlyWhenMouseInSvg, enableAutoEnterTextEditWhenKeydown: c.enableAutoEnterTextEditWhenKeydown, viewScale: a.mm.view.scale }; }
    const o = window.oracle.markmapDefaults;
    return { spacingHorizontal: o.spacingHorizontal, spacingVertical: o.spacingVertical, paddingX: o.paddingX, nodeMinHeight: o.nodeMinHeight, maxWidth: o.maxWidth, duration: 0, autoFit: false };
  };
  // Espera até a geometria não mudar durante 3 quadros seguidos (máx. 3 s).
  H.estavel = async () => {
    let last = "", same = 0;
    const t0 = performance.now();
    while (performance.now() - t0 < 3000) {
      await frames(1);
      let s;
      try { s = JSON.stringify(H.nos()); } catch { s = "erro"; }
      if (s === last && s !== "erro" && s !== "[]") { if (++same >= 3) return true; } else same = 0;
      last = s;
    }
    return false;
  };
  // Árvore de dados actual (pela API de cada app): [{id, texto, recolhido, filhos}].
  H.dados = () => {
    const a = H.app;
    if (a.kind === "me") {
      const w = (n) => ({ id: n.id, texto: n.topic, recolhido: n.expanded === false, filhos: (n.children || []).map(w) });
      return w(a.mind.getData().nodeData);
    }
    if (a.kind === "smm") {
      const d = a.mm.getData(false);
      const w = (n) => ({ id: n.data.uid, texto: strip(n.data.text), recolhido: n.data.expand === false, filhos: (n.children || []).map(w) });
      return w(d.root || d);
    }
    const w = (n) => ({ id: n.payload?.pid ?? `mm${n.state.path}`, texto: strip(n.content), recolhido: !!n.payload?.fold, filhos: n.children.map(w), _n: n });
    return w(a.mm.state.data);
  };
  // Caixa de cada nó no referencial do app (ME: ecrã/scale; SMM e markmap: referencial do layout).
  const caixaBruta = (d) => {
    const a = H.app;
    if (a.kind === "me") {
      let el = null;
      try { el = a.mind.findEle(d.id); } catch { el = null; }
      if (!el || !el.isConnected) return null;
      const r = el.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) return null;
      const k = a.mind.scaleVal;
      return [r.left / k, r.top / k, r.width / k, r.height / k];
    }
    if (a.kind === "smm") {
      const n = a.mm.renderer.findNodeByUid(d.id);
      if (!n || !n.group || !n.group.node || !n.group.node.isConnected) return null;
      return [n.left, n.top, n.width, n.height];
    }
    const n = d._n;
    if (!a.svg.querySelector(`g.markmap-node[data-path="${n.state.path}"]`)) return null;
    const r = n.state.rect;
    return [r.x, r.y, r.width, r.height];
  };
  // Lista em pré-ordem: [id, texto, pai, prof, x, y, w, h, lado, recolhido] com o centro da raiz = (0,0).
  H.nos = () => {
    const raiz = H.dados();
    const out = [];
    const rb = caixaBruta(raiz);
    if (!rb) return [];
    const cx = rb[0] + rb[2] / 2, cy = rb[1] + rb[3] / 2;
    H._centro = [cx, cy];
    const w = (d, pai, prof) => {
      const b = caixaBruta(d);
      let lado = null;
      if (b) { const dx = b[0] + b[2] / 2 - cx; lado = pai === null ? "raiz" : Math.abs(dx) < 1 ? "C" : dx > 0 ? "D" : "E"; }
      out.push([d.id, d.texto, pai, prof, ...(b ? [r3(b[0] - cx), r3(b[1] - cy), r3(b[2]), r3(b[3])] : [null, null, null, null]), lado, d.recolhido ? 1 : 0]);
      for (const f of d.filhos) w(f, d.id, prof + 1);
    };
    w(raiz, null, 0);
    return out;
  };
  // Ramos: cada <path> de ramo com as pontas no referencial normalizado e a matriz local→normalizado.
  H.ramos = () => {
    const a = H.app;
    H.nos();
    const [cx, cy] = H._centro;
    let paths, toN;
    if (a.kind === "me") {
      paths = [...a.mind.container.querySelectorAll("svg.lines path, svg.subLines path")];
      const k = a.mind.scaleVal;
      toN = (p) => { const S = p.getScreenCTM(); return [S.a / k, S.b / k, S.c / k, S.d / k, S.e / k - cx, S.f / k - cy]; };
    } else if (a.kind === "smm") {
      paths = [...a.mm.svg.node.querySelectorAll(".smm-line-container path")];
      const L = a.mm.renderer.root.group.node.parentNode.getScreenCTM().inverse();
      toN = (p) => { const M = L.multiply(p.getScreenCTM()); return [M.a, M.b, M.c, M.d, M.e - cx, M.f - cy]; };
    } else {
      paths = [...a.svg.querySelectorAll("path.markmap-link")];
      const L = a.svg.querySelector("g.markmap-node").parentNode.getScreenCTM().inverse();
      toN = (p) => { const M = L.multiply(p.getScreenCTM()); return [M.a, M.b, M.c, M.d, M.e - cx, M.f - cy]; };
    }
    const out = [];
    for (const p of paths) {
      const d = p.getAttribute("d");
      if (!d || !d.trim()) continue;
      const len = p.getTotalLength();
      if (!(len > 0)) continue;
      const m = toN(p);
      const ap = (q) => [r3(m[0] * q.x + m[2] * q.y + m[4]), r3(m[1] * q.x + m[3] * q.y + m[5])];
      out.push({ ini: ap(p.getPointAtLength(0)), fim: ap(p.getPointAtLength(len)), d: d.replace(/-?\d+\.\d+(e-?\d+)?/g, (s) => String(r3(+s))).replace(/\s+/g, " ").trim(), m: m.map(r3) });
    }
    return out;
  };
  H.selecao = () => {
    const a = H.app;
    if (a.kind === "me") return (a.mind.currentNodes || []).map((t) => t.nodeObj.id);
    if (a.kind === "smm") return a.mm.renderer.activeNodeList.map((n) => n.uid);
    return [];
  };
  H.foco = () => {
    const ae = document.activeElement;
    const editando = !!(ae && (ae.isContentEditable || /^(INPUT|TEXTAREA)$/.test(ae.tagName)));
    const cls = ae && typeof ae.className === "string" && ae.className.trim() ? "." + ae.className.trim().split(/\s+/).join(".") : "";
    const editorVisivel = [...document.querySelectorAll('[contenteditable="true"],[contenteditable=""],[contenteditable="plaintext-only"]')].some((e) => { const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0 && getComputedStyle(e).visibility !== "hidden"; });
    return { editando, editorVisivel, foco: ae ? ae.tagName.toLowerCase() + (ae.id ? "#" + ae.id : "") + cls : null };
  };
  H.estado = () => ({ nos: H.nos(), selecao: H.selecao(), ...H.foco() });
  // Centro do nó no ECRÃ (para o clique real).
  H.centroEcra = (id) => {
    const a = H.app;
    let r;
    if (a.kind === "me") r = a.mind.findEle(id).getBoundingClientRect();
    else { const n = a.mm.renderer.findNodeByUid(id); const g = n.getRect(); r = { left: g.x, top: g.y, width: g.width, height: g.height }; }
    return [r.left + r.width / 2, r.top + r.height / 2];
  };
  H.limpaSelecao = () => { const a = H.app; if (a.kind === "me") a.mind.clearSelection(); else a.mm.renderer.clearActiveNode(); return H.selecao(); };
  H.selecionaApi = (id) => {
    const a = H.app;
    if (a.kind === "me") a.mind.selectNode(a.mind.findEle(id));
    else a.mm.renderer.findNodeByUid(id).active();
    return H.selecao();
  };
  // Custo: montar (construtor + 1.º layout + render) e refazer (relayout da mesma árvore), em ms.
  H.custo = async ({ app, tree, layout }) => {
    // Relógio SÓ do trabalho do app: construtor + layout + DOM até à promessa/evento de fim de render,
    // mais um reflow forçado (o custo do browser conta). Sem a espera de estabilidade (ela mede quadros).
    const el = host();
    const flush = () => { el.getBoundingClientRect(); return document.body.offsetHeight; };
    const now = () => performance.now();
    let t0, t1, t2, t3;
    if (app === "me") {
      const { MindElixir } = window.oracle;
      const d = { nodeData: toME(tree) };
      t0 = now();
      const mind = new MindElixir({ el, direction: MindElixir[layout], toolBar: false, contextMenu: false });
      await mind.init(d);
      flush(); t1 = now();
      H.app = { kind: "me", mind };
      t2 = now(); mind.refresh(); flush(); t3 = now();
    } else if (app === "smm") {
      const { MindMap } = window.oracle;
      const d = toSMM(tree);
      t0 = now();
      const mm = new MindMap({ el, layout, data: d });
      await new Promise((res, rej) => { mm.on("node_tree_render_end", res); setTimeout(() => rej(new Error("smm: sem node_tree_render_end em 20 s")), 20000); });
      flush(); t1 = now();
      H.app = { kind: "smm", mm };
      t2 = now(); await new Promise((res) => mm.reRender(res)); flush(); t3 = now();
    } else {
      const { Markmap } = window.oracle;
      const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
      svg.setAttribute("width", "1600"); svg.setAttribute("height", "1000");
      el.appendChild(svg);
      const d1 = toMM(tree), d2 = toMM(tree);
      t0 = now();
      const mm = new Markmap(svg, { duration: 0, autoFit: false });
      await mm.setData(d1);
      flush(); t1 = now();
      H.app = { kind: "mm", mm, svg };
      t2 = now(); await mm.setData(d2); flush(); t3 = now();
    }
    await H.estavel();
    const a = H.app;
    const xs = H.nos().filter((n) => n[4] !== null);
    const bbox = xs.length ? [Math.min(...xs.map((n) => n[4])), Math.min(...xs.map((n) => n[5])), Math.max(...xs.map((n) => n[4] + n[6])), Math.max(...xs.map((n) => n[5] + n[7]))].map(r3) : null;
    if (a.kind === "me") a.mind.destroy?.(); else a.mm.destroy?.();
    return { montarMs: t1 - t0, refazerMs: t3 - t2, nosMedidos: xs.length, bbox };
  };
}

// ================================================================ corredor
const MIME = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".woff2": "font/woff2", ".svg": "image/svg+xml" };
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
const chromiumVersion = browser.version();

// Determinismo: Math.random, crypto.getRandomValues e crypto.randomUUID semeados pelo sha256 do nome
// do caso (ids dos nós novos). Os ids novos são, além disso, NORMALIZADOS para novo1, novo2… (pré-ordem).
const seedScript = (seed) => `(() => {
  let a = ${seed} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next;
  const gr = (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength);
    for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; };
  Object.defineProperty(crypto, "getRandomValues", { value: gr });
  Object.defineProperty(crypto, "randomUUID", { value: () => { const b = gr(new Uint8Array(16)); const h = [...b].map((x) => x.toString(16).padStart(2, "0")).join(""); return h.slice(0,8)+"-"+h.slice(8,12)+"-4"+h.slice(13,16)+"-a"+h.slice(17,20)+"-"+h.slice(20,32); } });
})();`;
let consoleMsgs, external, context, page;
async function abrePagina(seedName) {
  context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1600, height: 1000 } });
  context.on("request", (rq) => { const u = rq.url(); if (!u.startsWith(origin) && !u.startsWith("data:") && !u.startsWith("blob:")) external.add(new URL(u).origin); });
  await context.addInitScript(seedScript(parseInt(sha(seedName).slice(0, 8), 16)));
  await context.addInitScript(`(${pageLib.toString()})();`);
  page = await context.newPage();
  page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") consoleMsgs.push(`${m.type()}: ${m.text().slice(0, 160)}`); });
  page.on("pageerror", (e) => consoleMsgs.push(`pageerror: ${e.message.slice(0, 160)}`));
  await page.goto(origin + "/", { timeout: 60000 });
  await page.waitForFunction(() => window.oracleReady === true && window.H, null, { timeout: 60000 });
}
const fechaPagina = async () => { try { await context.close(); } catch {} };
const monta = (app, arvore, layout, opts) => page.evaluate((a) => window.H.monta(a), { app, tree: ENTRADAS[arvore], layout, opts });
const estado = () => page.evaluate(() => window.H.estado());
const espera = (ms) => page.waitForTimeout(ms);

// Ids que não existiam na árvore de entrada → novo1, novo2… pela ordem da pré-ordem do estado depois.
function normalizaIds(ids0, ...estados) {
  const map = new Map();
  for (const e of estados) for (const n of e.nos) if (!ids0.has(n[0]) && !map.has(n[0])) map.set(n[0], `novo${map.size + 1}`);
  const f = (id) => (id != null && map.has(id) ? map.get(id) : id);
  for (const e of estados) {
    e.nos = e.nos.map((n) => [f(n[0]), n[1], f(n[2]), ...n.slice(3)]);
    e.selecao = e.selecao.map(f);
  }
  return estados;
}
const idsDe = (t, s = new Set()) => { s.add(t.id); t.children.forEach((c) => idsDe(c, s)); return s; };

// Clique real no centro do nó; se o clique não selecionar só esse nó, cai para a API e regista-o.
async function seleciona(app, id) {
  // simple-mind-map só aceita atalhos com o rato DENTRO do svg (enableShortcutOnlyWhenMouseInSvg).
  // Medido: um 2.º clique no nó JÁ seleccionado entra em edição (sonda teclas_*_clique). Por isso,
  // se o nó já está seleccionado, limpa-se a selecção (API) e espera-se 800 ms antes do clique.
  if ((await page.evaluate(() => window.H.selecao())).includes(id)) { await page.evaluate(() => window.H.limpaSelecao()); await espera(800); }
  const [x, y] = await page.evaluate((i) => window.H.centroEcra(i), id);
  await page.mouse.click(x, y);
  await espera(120);
  let sel = await page.evaluate(() => window.H.selecao());
  if ((await page.evaluate(() => window.H.foco())).editando) { await page.keyboard.press("Escape"); await espera(100); return "clique+edicao"; }
  if (sel.length === 1 && sel[0] === id) return "clique";
  sel = await page.evaluate((i) => window.H.selecionaApi(i), id);
  await espera(80);
  return sel.length === 1 && sel[0] === id ? "api" : `falhou:${sel.join(",")}`;
}
async function carrega(teclas) {
  // 400 ms entre teclas (ritmo humano; medido: o simple-mind-map com 60 ms não registava o Delete
  // no histórico antes do Control+z).
  for (const k of teclas.split(" ")) { await page.keyboard.press(k); await espera(400); }
  await page.evaluate(() => window.H.estavel());
}

// ================================================================ casos
const LAYOUTS = {
  me: ["LEFT", "RIGHT", "SIDE", "DOWN"],
  smm: ["logicalStructure", "logicalStructureLeft", "mindMap", "catalogOrganization", "organizationStructure", "timeline", "timeline2", "fishbone", "fishbone2", "verticalTimeline", "verticalTimeline2", "verticalTimeline3"],
  mm: ["padrao"],
};
const COLUNAS = ["id", "texto", "pai", "prof", "x", "y", "w", "h", "lado", "recolhido"];
const ARVORES = ["T1", "T2", "T3", "T4", "T5", "T6"];
const casos = {};
for (const [app, ls] of Object.entries(LAYOUTS)) for (const L of ls) {
  casos[`layout_${app}_${L}`] = {
    tipo: "layout",
    pergunta: `posições de cada nó e ramos, ${app} layout ${L}, árvores T1..T6`,
    corre: async () => {
      const res = { app, layout: L, colunas: COLUNAS, arvores: {} };
      for (const t of ARVORES) {
        res.opcoes = await monta(app, t, app === "mm" ? undefined : L);
        const nos = await page.evaluate(() => window.H.nos());
        const ramos = await page.evaluate(() => window.H.ramos());
        res.arvores[t] = { nos, ramos: casaRamos(nos, ramos) };
      }
      return res;
    },
  };
}
// Cada ramo liga o nó mais próximo da ponta inicial ao mais próximo da final; ordenado pai→filho.
function casaRamos(nos, ramos) {
  const vis = nos.filter((n) => n[4] !== null);
  const dist = (p, n) => { const dx = Math.max(n[4] - p[0], 0, p[0] - (n[4] + n[6])); const dy = Math.max(n[5] - p[1], 0, p[1] - (n[5] + n[7])); return Math.hypot(dx, dy); };
  const perto = (p) => { let b = null, bd = Infinity; for (const n of vis) { const d = dist(p, n); if (d < bd) { bd = d; b = n; } } return [b, bd]; };
  const out = ramos.map((r) => {
    let [a, da] = perto(r.ini), [b, db] = perto(r.fim);
    let ini = r.ini, fim = r.fim;
    if (a && b && a[3] > b[3]) { [a, b] = [b, a]; [da, db] = [db, da]; [ini, fim] = [fim, ini]; }
    return { de: a?.[0] ?? null, para: b?.[0] ?? null, ini, fim, folga: [r3(da), r3(db)], d: r.d, m: r.m };
  });
  const ord = new Map(nos.map((n, i) => [n[0], i]));
  return out.sort((p, q) => (ord.get(p.para) ?? 1e9) - (ord.get(q.para) ?? 1e9) || (ord.get(p.de) ?? 1e9) - (ord.get(q.de) ?? 1e9) || p.d.localeCompare(q.d));
}

// ---------------------------------------------------------------- teclado
const LAYOUT_TECLAS = { me: "SIDE", smm: "mindMap" };
const LAYOUT_TECLAS_1LADO = { me: "RIGHT", smm: "logicalStructure" };
// Sondas que MUDAM a árvore: cada uma numa página nova. [árvore, nó inicial, teclas]
const SONDAS = {
  tab: [["T1", "A", "Tab"], ["T1", "B", "Tab"], ["T1", "root", "Tab"], ["T2", "A", "Tab"], ["T2", "A3", "Tab"], ["T2", "A3b", "Tab"], ["T2", "D", "Tab"]],
  enter: [["T1", "A", "Enter"], ["T1", "B", "Enter"], ["T1", "C", "Enter"], ["T1", "root", "Enter"], ["T2", "A1", "Enter"], ["T2", "A3", "Enter"], ["T2", "A3b", "Enter"], ["T2", "A5", "Enter"], ["T2", "B", "Enter"]],
  shift_enter: [["T1", "B", "Shift+Enter"], ["T1", "A", "Shift+Enter"], ["T1", "root", "Shift+Enter"], ["T2", "A3", "Shift+Enter"], ["T2", "A3b", "Shift+Enter"]],
  shift_tab: [["T1", "B", "Shift+Tab"], ["T1", "root", "Shift+Tab"], ["T2", "A3", "Shift+Tab"], ["T2", "A3b", "Shift+Tab"]],
  ctrl_enter: [["T1", "B", "Control+Enter"], ["T2", "A3", "Control+Enter"], ["T2", "A3b", "Control+Enter"]],
  apagar: [["T1", "B", "Delete"], ["T1", "root", "Delete"], ["T2", "A3", "Delete"], ["T2", "A1", "Delete"], ["T2", "A5", "Delete"], ["T2", "A3b", "Delete"], ["T2", "A3c", "Delete"], ["T2", "B", "Delete"], ["T2", "A3", "Backspace"], ["T2", "A1", "Backspace"], ["T2", "A3b", "Backspace"], ["T2", "A3", "Shift+Backspace"], ["T2", "A3", "Shift+Delete"]],
  recolher: [["T2", "A", "Space"], ["T2", "A", "Slash"], ["T2", "A", "Control+Slash"], ["T2", "A", "Alt+Slash"], ["T2", "A", "Minus"], ["T2", "A", "Control+k Control+0"], ["T2", "A3", "Space"], ["T2", "A3", "Slash"], ["T5", "A", "Space"], ["T5", "A", "Slash"]],
  desfazer: [["T1", "A", "Tab Control+z"], ["T1", "A", "Tab Escape Control+z"], ["T1", "A", "Tab Enter Control+z"], ["T1", "A", "Tab Escape Control+z Control+y"], ["T1", "A", "Tab Escape Control+z Control+Shift+z"], ["T2", "A3", "Delete Control+z"], ["T1", "A", "Enter Escape Control+z"]],
};
for (const app of ["me", "smm"]) {
  for (const [nome, lista] of Object.entries(SONDAS)) {
    casos[`teclas_${app}_${nome}`] = {
      tipo: "teclas",
      porSonda: true,
      pergunta: `${app}: o que fazem as teclas (${nome}) — estado antes/depois (árvore, selecção, edição, posições); layout ${LAYOUT_TECLAS[app]}`,
      sondas: lista.map(([arv, ini, teclas]) => ({ arv, ini, teclas })),
      correSonda: async ({ arv, ini, teclas }) => {
        const opcoes = await monta(app, arv, LAYOUT_TECLAS[app]);
        const selPor = await seleciona(app, ini);
        const antes = await estado();
        await carrega(teclas);
        const depois = await estado();
        normalizaIds(idsDe(ENTRADAS[arv]), antes, depois);
        return { arvore: arv, layout: LAYOUT_TECLAS[app], inicio: ini, selecionadoPor: selPor, teclas, opcoes, antes, depois };
      },
    };
  }
  // Clique: o que faz um 2.º clique (rato real) no nó já seleccionado, com 150 ms e 700 ms de intervalo.
  casos[`teclas_${app}_clique`] = {
    tipo: "teclas",
    pergunta: `${app}: 2.º clique no nó já seleccionado (150 ms e 700 ms depois do 1.º) — entra em edição?`,
    corre: async () => {
      const out = [];
      for (const [id, ms] of [["A", 150], ["A", 700], ["root", 150], ["root", 700]]) {
        await monta(app, "T1", LAYOUT_TECLAS[app]);
        const [x, y] = await page.evaluate((i) => window.H.centroEcra(i), id);
        await page.mouse.click(x, y);
        await espera(ms);
        const s1 = await estado();
        await page.mouse.click(x, y);
        await espera(300);
        const s2 = await estado();
        out.push({ no: id, intervaloMs: ms, depois1: { selecao: s1.selecao, editando: s1.editando }, depois2: { selecao: s2.selecao, editando: s2.editando, foco: s2.foco } });
      }
      return { resultados: out };
    },
  };
  // Setas: não mudam a árvore — uma página por (árvore, layout), clique no nó e 1 tecla.
  casos[`teclas_${app}_setas`] = {
    tipo: "teclas",
    pergunta: `${app}: para onde vai a selecção com ← → ↑ ↓ a partir de CADA nó de T1, T2 e T6, no layout de dois lados (${LAYOUT_TECLAS[app]}) e no de um lado (${LAYOUT_TECLAS_1LADO[app]}); mapa remontado, clique real no nó e UMA tecla por sonda`,
    corre: async () => {
      const out = [];
      for (const L of [LAYOUT_TECLAS[app], LAYOUT_TECLAS_1LADO[app]]) for (const arv of ["T1", "T2", "T6"]) {
        await monta(app, arv, L);
        const st = await estado();
        const r = { arvore: arv, layout: L, nos: st.nos, destino: {} };
        for (const n of st.nos) {
          if (n[4] === null) continue;
          r.destino[n[0]] = {};
          for (const k of ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"]) {
            await monta(app, arv, L); // mapa novo por sonda: nenhum clique cai num nó já seleccionado
            const por = await seleciona(app, n[0]);
            const ed0 = (await page.evaluate(() => window.H.foco())).editando;
            await page.keyboard.press(k);
            await espera(100);
            const s = await estado();
            r.destino[n[0]][k] = { sel: s.selecao, ...(s.editando ? { editando: true } : {}), ...(ed0 ? { editandoAntes: true } : {}), ...(por !== "clique" ? { selecionadoPor: por } : {}) };
            if (s.editando) await page.keyboard.press("Escape");
          }
        }
        out.push(r);
      }
      return { colunas: COLUNAS, resultados: out };
    },
  };
}

// ---------------------------------------------------------------- custo (os tempos vão para o .meta)
const CUSTO_VARIANTES = [["me", "SIDE"], ["me", "RIGHT"], ["smm", "mindMap"], ["smm", "logicalStructure"], ["mm", "padrao"]];
let temposCusto = null;
casos.custo = {
  tipo: "custo",
  pergunta: "tempo de montar (construtor+layout+render) e de refazer o layout a 100 e 1000 nós; 5 rodadas intercaladas, o MÍNIMO; os tempos vão para custo.meta.json (não são determinísticos)",
  corre: async () => {
    const RODADAS = 5;
    const vs = [];
    for (const N of ["N100", "N1000"]) for (const [app, L] of CUSTO_VARIANTES) vs.push({ N, app, L });
    const amostras = vs.map(() => ({ montar: [], refazer: [] }));
    const det = {};
    for (let r = 0; r < RODADAS; r++) {
      // página nova por rodada (medido: no simple-mind-map a 1000 nós o tempo subia de rodada em
      // rodada na mesma página — 448 → 589 ms — acumulação, não o layout).
      if (r > 0) { await fechaPagina(); await abrePagina(`custo#${r}`); }
      for (let j = 0; j < vs.length; j++) {
        const i = (j + r) % vs.length; // rodadas rodadas: cada variante muda de posição
        const v = vs[i];
        const o = await page.evaluate((a) => window.H.custo(a), { app: v.app, tree: ENTRADAS[v.N], layout: v.app === "mm" ? undefined : v.L });
        amostras[i].montar.push(o.montarMs);
        amostras[i].refazer.push(o.refazerMs);
        det[`${v.N}_${v.app}_${v.L}`] = { nosMedidos: o.nosMedidos, bbox: o.bbox };
      }
    }
    temposCusto = Object.fromEntries(vs.map((v, i) => [`${v.N}_${v.app}_${v.L}`, {
      montarMinMs: Math.round(Math.min(...amostras[i].montar) * 10) / 10,
      refazerMinMs: Math.round(Math.min(...amostras[i].refazer) * 10) / 10,
      montarMs: amostras[i].montar.map((x) => Math.round(x * 10) / 10),
      refazerMs: amostras[i].refazer.map((x) => Math.round(x * 10) / 10),
    }]));
    return { rodadas: RODADAS, variantes: det };
  },
};

// ================================================================ execução
let failures = 0;
const escolhidos = so ? so.split(",") : Object.keys(casos);
const layoutsMedidos = {};
try {
  for (const nome of escolhidos) {
    const caso = casos[nome];
    if (!caso) { console.error(`caso desconhecido: ${nome}`); failures++; continue; }
    consoleMsgs = [];
    external = new Set();
    const loadAntes = os.loadavg()[0];
    let res;
    try {
      if (caso.porSonda) {
        const sondas = [];
        for (const [i, s] of caso.sondas.entries()) {
          await abrePagina(`${nome}#${i}`);
          sondas.push(await caso.correSonda(s));
          await fechaPagina();
        }
        res = { sondas };
      } else {
        await abrePagina(nome);
        res = await caso.corre();
        await fechaPagina();
      }
    } catch (e) {
      console.error(`FALHA ${nome}: ${e.stack}`);
      failures++;
      await fechaPagina();
      continue;
    }
    if (caso.tipo === "layout") layoutsMedidos[nome] = res;
    escreve(nome, { caso: nome, pergunta: caso.pergunta, ...res }, { loadavg_1min: [r3(loadAntes), r3(os.loadavg()[0])], ...(nome === "custo" ? { tempos_ms: temposCusto } : {}) });
    console.log(`ok ${nome}`);
  }
  const todosLayouts = Object.keys(casos).filter((k) => casos[k].tipo === "layout");
  if (todosLayouts.every((k) => layoutsMedidos[k])) {
    escreve("leis", { caso: "leis", pergunta: "leis destiladas dos casos layout_* (leis.mjs)", ...destilaLeis(layoutsMedidos) }, {});
    console.log("ok leis");
  }
} finally {
  await browser.close();
  server.close();
}

function escreve(nome, obj, metaExtra) {
  const base = path.join(outDir, nome);
  const body = JSON.stringify(obj) + "\n";
  fs.writeFileSync(base + ".json", body);
  const meta = {
    oracle: "mindmap (mind-elixir, simple-mind-map, markmap)",
    versions: VERSOES,
    licenses: "todas MIT (licencas.mjs; simple-mind-map: campo license MIT + LICENSE do repositório, sem ficheiro no artefacto)",
    chromium_version: chromiumVersion,
    node_version: process.version,
    date: new Date().toISOString(),
    command: `node docs/MiroClone/ferramentas/mindmap_oracle/corre.mjs --so ${nome} <saida>`,
    runner_sha256: SELF_SHA,
    leis_sha256: LEIS_SHA,
    inputs_sha256: ENTRADAS_SHA,
    window: "1600×1000, deviceScaleFactor 1, locale en-US, UTC",
    normalizacao: "caixas relativas ao CENTRO da raiz, zoom 1 (ME: DOM/scaleVal; SMM: Node.left/top/width/height; markmap: state.rect); números a 1e-3; ids de nós novos → novo1, novo2… (pré-ordem); números no atributo d dos ramos a 1e-3",
    external_requests_seen: [...external].sort(),
    console: [...new Set(consoleMsgs)].sort(),
    outputs_sha256: { [nome + ".json"]: sha(body) },
    ...metaExtra,
  };
  fs.writeFileSync(base + ".meta.json", JSON.stringify(meta, null, 2) + "\n");
}
process.exit(failures ? 1 : 0);
