// Sonda: Excalidraw — o RÓTULO de uma seta (texto com containerId = seta), nas DUAS versões instaladas:
// 0.18.1 (npm latest) e o canal next (0.18.0-143b5b6, o do excalidraw.com, que tem `labelPosition` no texto —
// achado XV5 do estudo 07). Caixa preta: só a API pública da página W9 (`www_w9/oracle.js`: restoreEls, mount,
// select, scene, editorApi) + rato/teclado reais do Playwright no <Excalidraw> montado (1280×800, zoom 1,
// scroll 0 ⇒ ecrã = cena).
// Uso: node sonda_excalidraw_rotulo.mjs <pasta_saida>   (VERSAO=next|0181 · SO=<prefixo do caso>)
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HOMES = {
  next: process.env.ORACLE_NEXT_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle-next"),
  "0181": process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle"),
};
const outDir = process.argv[2];
fs.mkdirSync(outDir, { recursive: true });
const SCRIPT_SHA = createHash("sha256").update(fs.readFileSync(fileURLToPath(import.meta.url))).digest("hex");
const FIXED_TIME = Date.UTC(2026, 0, 1, 0, 0, 0);
const r2 = (v) => Math.round(v * 100) / 100;
const R = (p) => [r2(p[0]), r2(p[1])];
const MIME = { ".html": "text/html", ".js": "text/javascript", ".woff2": "font/woff2", ".ttf": "font/ttf", ".wasm": "application/wasm", ".css": "text/css" };
const seedScript = (seed) => `(() => { let a = ${seed} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a; t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61); return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next; Object.defineProperty(crypto, "getRandomValues", { value: (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength); for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; } }); })();`;

// Setas (pontos relativos); a seta nasce em (OX, OY) da cena — longe da barra de ferramentas do editor.
const OX = 450, OY = 300;   // à direita do painel de propriedades (x 16..211), que aparece com a seta seleccionada
const SETAS = {
  recta2: [[0, 0], [400, 0]],
  dobra3: [[0, 0], [200, 0], [200, 200]],
  dobra3_assim: [[0, 0], [300, 0], [300, 100]],
  quatro: [[0, 0], [100, 0], [100, 100], [400, 100]],
};
const LPS = [null, 0, 0.25, 0.5, 0.75, 1];
const CASOS_ALL = [];
for (const [s, pts] of Object.entries(SETAS)) for (const lp of LPS) CASOS_ALL.push({ id: `L_${s}_lp${lp ?? "null"}`, seta: s, pts, lp, tipo: "posicao" });
for (const lp of [null, 0.25, 0.75]) CASOS_ALL.push({ id: `L_dobra3_assim_redonda_lp${lp ?? "null"}`, seta: "dobra3_assim", pts: SETAS.dobra3_assim, lp, redonda: true, tipo: "posicao" });
// D_*: o rótulo parte de labelPosition 0.3 (longe das pegas); H_*: parte do «meio» (null), onde está a pega do
// ponto médio (2 pontos) ou o vértice do meio (3 pontos) — mede a precedência pega × rótulo
for (const s of ["recta2", "dobra3_assim"]) for (const [pre, lp] of [["D", 0.3], ["H", null]]) {
  CASOS_ALL.push({ id: `${pre}_${s}_a_ao_longo_3q`, seta: s, pts: SETAS[s], lp, tipo: "arrasto", arrasto: "ao_longo_3q" });
  CASOS_ALL.push({ id: `${pre}_${s}_b_alem_do_fim_200`, seta: s, pts: SETAS[s], lp, tipo: "arrasto", arrasto: "alem_fim_200" });
  CASOS_ALL.push({ id: `${pre}_${s}_b2_alem_do_inicio_200`, seta: s, pts: SETAS[s], lp, tipo: "arrasto", arrasto: "alem_inicio_200" });
  CASOS_ALL.push({ id: `${pre}_${s}_c_perpendicular_120`, seta: s, pts: SETAS[s], lp, tipo: "arrasto", arrasto: "perp_120" });
}
CASOS_ALL.push({ id: "M_dois_textos_na_mesma_seta", seta: "recta2", pts: SETAS.recta2, lp: null, tipo: "posicao", dois: true });
CASOS_ALL.push({ id: "K_duplo_clique_no_corpo_com_rotulo", seta: "recta2", pts: SETAS.recta2, lp: 0.25, tipo: "duplo", em: 0.8 });
const CASOS = process.env.SO ? CASOS_ALL.filter((c) => c.id.startsWith(process.env.SO)) : CASOS_ALL;

const entrada = (caso) => {
  const pts = caso.pts;
  const xs = pts.map((p) => p[0]), ys = pts.map((p) => p[1]);
  const texto = (id, txt, lp) => ({ type: "text", id, x: OX, y: OY, width: 30, height: 25, text: txt, originalText: txt, fontSize: 20, fontFamily: 5,
    textAlign: "center", verticalAlign: "middle", containerId: "A", lineHeight: 1.25, autoResize: true, ...(lp === null ? {} : { labelPosition: lp }) });
  const els = [
    { type: "arrow", id: "A", x: OX, y: OY, width: Math.max(...xs) - Math.min(...xs), height: Math.max(...ys) - Math.min(...ys), points: pts,
      roundness: caso.redonda ? { type: 2 } : null, startArrowhead: null, endArrowhead: "arrow",
      boundElements: caso.dois ? [{ type: "text", id: "T" }, { type: "text", id: "T2" }] : [{ type: "text", id: "T" }] },
    texto("T", "sim", caso.lp),
  ];
  if (caso.dois) els.push(texto("T2", "dois", null));
  return els;
};

// ---- geometria do lado do Node ----
const polyLen = (P) => { let L = 0; for (let i = 1; i < P.length; i++) L += Math.hypot(P[i][0] - P[i - 1][0], P[i][1] - P[i - 1][1]); return L; };
const atArc = (P, f) => { const L = polyLen(P); let s = f * L; for (let i = 1; i < P.length; i++) { const d = Math.hypot(P[i][0] - P[i - 1][0], P[i][1] - P[i - 1][1]); if (s <= d || i === P.length - 1) { const u = d ? Math.min(1, s / d) : 0; return [P[i - 1][0] + (P[i][0] - P[i - 1][0]) * u, P[i - 1][1] + (P[i][1] - P[i - 1][1]) * u]; } s -= d; } return P[P.length - 1]; };
const atParam = (P, f) => { const n = P.length - 1; const x = f * n; const i = Math.min(n - 1, Math.floor(x)); const u = x - i; return [P[i][0] + (P[i + 1][0] - P[i][0]) * u, P[i][1] + (P[i + 1][1] - P[i][1]) * u]; };
const project = (P, q) => { let best = null, acc = 0; for (let i = 1; i < P.length; i++) { const a = P[i - 1], b = P[i]; const dx = b[0] - a[0], dy = b[1] - a[1]; const d2 = dx * dx + dy * dy; const len = Math.sqrt(d2); let u = d2 ? ((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / d2 : 0; u = Math.max(0, Math.min(1, u)); const p = [a[0] + dx * u, a[1] + dy * u]; const dist = Math.hypot(q[0] - p[0], q[1] - p[1]); if (!best || dist < best.d - 1e-9) best = { s: acc + u * len, d: dist, ponto: p, tangente: len ? [dx / len, dy / len] : [0, 0] }; acc += len; } best.f = best.s / acc; return best; };
const dist = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);
// «meio» à 0.18.1 (hipótese a confrontar): nº ímpar de pontos ⇒ o vértice do meio; par ⇒ meio do trecho do meio
const meioPorIndice = (P) => { const n = P.length; if (n % 2 === 1) return P[(n - 1) / 2]; const a = P[n / 2 - 1], b = P[n / 2]; return [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2]; };

function analisa(els, lpPedido) {
  const A = els.find((e) => e.id === "A");
  const T = els.find((e) => e.id === "T");
  const T2 = els.find((e) => e.id === "T2");
  const P = A.points.map(([x, y]) => [A.x + x, A.y + y]);
  const pick = (t) => t && ({ x: r2(t.x), y: r2(t.y), width: r2(t.width), height: r2(t.height), angle: t.angle, containerId: t.containerId, labelPosition: Object.prototype.hasOwnProperty.call(t, "labelPosition") ? t.labelPosition : "AUSENTE", isDeleted: !!t.isDeleted, text: t.text });
  const out = { seta: { x: A.x, y: A.y, points: A.points, abs: P.map(R), comprimento: r2(polyLen(P)), boundElements: A.boundElements, roundness: A.roundness, angle: A.angle }, texto: pick(T), texto2: pick(T2) ?? null };
  if (T) {
    const c = [T.x + T.width / 2, T.y + T.height / 2];
    const pr = project(P, c);
    out.texto.centro = R(c);
    out.texto.projeccao = { s: r2(pr.s), fraccao_arco: Math.round(pr.f * 10000) / 10000, distancia: r2(pr.d), ponto: R(pr.ponto) };
    const lp = lpPedido ?? 0.5;
    const pa = atArc(P, lp), pp = atParam(P, lp), pm = meioPorIndice(P);
    out.texto.comparacao = { lp_usado: lp, ponto_fraccao_arco: R(pa), ponto_parametrico_indice: R(pp), meio_por_indice: R(pm), dist_ao_ponto_arco: r2(dist(c, pa)), dist_ao_parametrico: r2(dist(c, pp)), dist_ao_meio_por_indice: r2(dist(c, pm)) };
  }
  return out;
}

async function drag(page, from, to, steps = 25) {
  await page.mouse.move(from[0], from[1]); await page.waitForTimeout(100);
  await page.mouse.down(); await page.waitForTimeout(100);
  for (let i = 1; i <= steps; i++) { await page.mouse.move(from[0] + (to[0] - from[0]) * i / steps, from[1] + (to[1] - from[1]) * i / steps); await page.waitForTimeout(16); }
  await page.waitForTimeout(150); await page.mouse.up(); await page.waitForTimeout(300);
}

const porVersao = {};
for (const [versao, home] of Object.entries(HOMES)) {
  if (process.env.VERSAO && process.env.VERSAO !== versao) continue;
  const WWW = path.join(home, "www_w9");
  const require = createRequire(path.join(home, "package.json"));
  const { chromium } = require("playwright");
  const PKG = JSON.parse(fs.readFileSync(path.join(home, "node_modules/@excalidraw/excalidraw/package.json"), "utf8"));
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
  const bloqueados = new Set();
  const results = [];
  for (const caso of CASOS) {
    const ctx = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
    await ctx.route("**/*", (route) => { const u = route.request().url(); if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue(); bloqueados.add(u.split("?")[0]); return route.abort(); });
    await ctx.addInitScript(seedScript(0x1a6e1));
    const page = await ctx.newPage();
    // arrasto e duplo clique com o relógio LIVRE (o editor next decide gestos por temporizadores — README W9)
    if (caso.tipo === "posicao") await page.clock.setFixedTime(FIXED_TIME);
    const errs = []; page.on("pageerror", (e) => errs.push(e.message));
    await page.goto(origin + "/", { timeout: 60000 });
    await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 60000 });
    const raw = entrada(caso);
    const passos = [];
    const restored0 = await page.evaluate((els) => window.oracle.restoreEls(els), raw);
    passos.push({ passo: "restoreElements(refreshDimensions:false, repairBindings:true)", ...analisa(restored0, caso.lp), texto_restaurado_json: restored0.find((e) => e.id === "T") });
    // refreshDimensions:true — o restore volta a medir o texto e a COLOCÁ-LO no contentor (medido: com false fica onde o ficheiro diz)
    const restored = await page.evaluate((els) => window.oracle.api.restoreElements(els, null, { refreshDimensions: true, repairBindings: true }), raw);
    passos.push({ passo: "restoreElements(refreshDimensions:true, repairBindings:true)", ...analisa(restored, caso.lp), texto_restaurado_json: restored.find((e) => e.id === "T") });
    await page.evaluate((els) => window.oracle.mount(els), restored);
    await page.waitForTimeout(400);
    let cena = await page.evaluate(() => window.oracle.scene());
    passos.push({ passo: "montado", ...analisa(cena, caso.lp),
      serializado_texto: await page.evaluate((els) => { const t = JSON.parse(window.oracle.api.serializeAsJSON(els, {}, {}, "local")).elements.find((e) => e.id === "T"); return { labelPosition: t && Object.prototype.hasOwnProperty.call(t, "labelPosition") ? t.labelPosition : "AUSENTE", x: t?.x, y: t?.y }; }, cena) });
    // empurra a seta → ← (posição final igual): o editor volta a colocar o rótulo pela lei dele
    await page.evaluate(() => window.oracle.select(["A"]));
    await page.waitForTimeout(150); await page.keyboard.press("ArrowRight"); await page.waitForTimeout(150); await page.keyboard.press("ArrowLeft"); await page.waitForTimeout(300);
    cena = await page.evaluate(() => window.oracle.scene());
    passos.push({ passo: "empurrado_seta_direita_esquerda", ...analisa(cena, caso.lp) });
    // o gesto que obriga o EDITOR a recolocar o rótulo: arrastar a ponta final 40 px na perpendicular do último
    // trecho e devolvê-la (dois arrastos reais; a seta acaba igual)
    {
      const P0 = analisa(cena, caso.lp).seta.abs; const n = P0.length; const a = P0[n - 2], b = P0[n - 1]; const l = dist(a, b);
      const nrm = [-(b[1] - a[1]) / l * 40, (b[0] - a[0]) / l * 40];
      await page.evaluate(() => window.oracle.select([]));
      await page.keyboard.press("Escape"); await page.waitForTimeout(100);
      await page.mouse.click(...atArc(P0, 0.125)); await page.waitForTimeout(250);
      await drag(page, b, [b[0] + nrm[0], b[1] + nrm[1]], 10);
      cena = await page.evaluate(() => window.oracle.scene());
      passos.push({ passo: "ponta_final_arrastada_40_na_perpendicular", ...analisa(cena, caso.lp), recolocado_por_restore: analisa(await page.evaluate((els) => window.oracle.api.restoreElements(els, null, { refreshDimensions: true, repairBindings: true }), cena), caso.lp) });
      if (caso.tipo === "posicao") fs.writeFileSync(path.join(outDir, `excalidraw_${versao}_${caso.id}_ponta40.png`), await page.screenshot());
      await drag(page, [b[0] + nrm[0], b[1] + nrm[1]], b, 10);
      cena = await page.evaluate(() => window.oracle.scene());
    }
    const aE = analisa(cena, caso.lp);
    passos.push({ passo: "ponta_final_devolvida", ...aE });
    if (caso.tipo === "arrasto") {
      await page.evaluate(() => window.oracle.select([]));
      await page.keyboard.press("Escape"); await page.waitForTimeout(100);
      const P = aE.seta.abs;
      // selecciona como o artista: clique no corpo a 1/8 do arco
      await page.mouse.click(...atArc(P, 0.125)); await page.waitForTimeout(300);
      const selecao = await page.evaluate(() => window.oracle.appState().selected);
      const from = aE.texto.centro;
      let to;
      if (caso.arrasto === "ao_longo_3q") to = atArc(P, 0.75);
      else if (caso.arrasto === "alem_fim_200") { const n = P.length; const a = P[n - 2], b = P[n - 1]; const l = dist(a, b); to = [b[0] + (b[0] - a[0]) / l * 200, b[1] + (b[1] - a[1]) / l * 200]; }
      else if (caso.arrasto === "alem_inicio_200") { const a = P[1], b = P[0]; const l = dist(a, b); to = [b[0] + (b[0] - a[0]) / l * 200, b[1] + (b[1] - a[1]) / l * 200]; }
      else { const pr = project(P, from); const [tx, ty] = pr.tangente; to = [from[0] - ty * 120, from[1] + tx * 120]; }
      await drag(page, from, to);
      cena = await page.evaluate(() => window.oracle.scene());
      const a1 = analisa(cena, null);
      // o x,y gravado do texto pode ficar para trás do que o canvas desenha (medido no next: a ponta movida não o actualiza);
      // o restore com refreshDimensions recoloca-o pela lei, a partir do labelPosition gravado
      const a1r = analisa(await page.evaluate((els) => window.oracle.api.restoreElements(els, null, { refreshDimensions: true, repairBindings: true }), cena), cena.find((e) => e.id === "T")?.labelPosition ?? null);
      const prTo = project(P, to);
      passos.push({ passo: `arrasto_${caso.arrasto}`, selecao_antes: selecao, de: R(from), para: R(to), alvo_projectado: { fraccao_arco: Math.round(prTo.f * 10000) / 10000, distancia: r2(prTo.d), ponto: R(prTo.ponto) },
        seta_mexeu: JSON.stringify(a1.seta.abs) !== JSON.stringify(aE.seta.abs), ...a1, recolocado_por_restore: a1r, selecao_depois: await page.evaluate(() => window.oracle.appState().selected) });
    } else if (caso.tipo === "duplo") {
      await page.evaluate(() => window.oracle.select([]));
      await page.keyboard.press("Escape"); await page.waitForTimeout(100);
      const q = atArc(aE.seta.abs, caso.em);
      await page.mouse.dblclick(...q); await page.waitForTimeout(400);
      const st = await page.evaluate(() => { const a = window.oracle.editorApi().getAppState(); return { editingTextElement: a.editingTextElement?.id ?? a.editingElement?.id ?? null, selected: Object.keys(a.selectedElementIds || {}) }; });
      await page.keyboard.type("X"); await page.waitForTimeout(200); await page.keyboard.press("Escape"); await page.waitForTimeout(300);
      cena = await page.evaluate(() => window.oracle.scene());
      passos.push({ passo: `duplo_clique_no_corpo_a_${caso.em}_e_escreve_X_e_Escape`, ponto: R(q), estado_apos_duplo_clique: st, ...analisa(cena, null), textos_na_cena: cena.filter((e) => e.type === "text" && !e.isDeleted).map((e) => ({ id: e.id, text: e.text, containerId: e.containerId, labelPosition: e.labelPosition ?? "AUSENTE" })) });
    }
    fs.writeFileSync(path.join(outDir, `excalidraw_${versao}_${caso.id}.png`), await page.screenshot());
    results.push({ caso: caso.id, entrada: raw, passos, pageerrors: errs });
    const last = passos[passos.length - 1];
    console.log(versao, caso.id, JSON.stringify({ T: last.texto && { lp: last.texto.labelPosition, c: last.texto.centro, f: last.texto.projeccao?.fraccao_arco, d: last.texto.projeccao?.distancia, ang: last.texto.angle, cmp: last.texto.comparacao }, mexeu: last.seta_mexeu, T2: last.texto2 }));
    await ctx.close();
  }
  porVersao[versao] = { versao: PKG.version, licenca: PKG.license, chromium: browser.version(), pedidos_externos_bloqueados: [...bloqueados].sort(), casos: results };
  await browser.close(); server.close();
}
fs.writeFileSync(path.join(outDir, "excalidraw_rotulo.json"), JSON.stringify({
  cabecalho: {
    oraculo: "@excalidraw/excalidraw nas duas versões instaladas (0.18.1 em ~/Apps/excalidraw-oracle, next 0.18.0-143b5b6 em ~/Apps/excalidraw-oracle-next), página W9 (www_w9) em Chromium headless, rato real do Playwright",
    data: new Date().toISOString(), host: os.hostname(), loadavg: os.loadavg(),
    comando: `node ${fileURLToPath(import.meta.url)} ${outDir}`, script_sha256: SCRIPT_SHA,
    entrada: `seta id A em (${OX},${OY}) com roundness null (cantos vivos; *_redonda = {type:2}), endArrowhead arrow, boundElements [{type:text,id:T}]; texto T «sim» fontSize 20 fontFamily 5 containerId A, x,y = origem da seta, labelPosition só quando ≠ null; setas: ${JSON.stringify(SETAS)}`,
    camera: "zoom 1, scroll 0 ⇒ ecrã = cena (oracle.mount)",
    metodo: "restoreElements → monta → empurra a seta → ← (o editor recoloca o rótulo) → [arrasto do rótulo pelo rato com a seta seleccionada por clique no corpo a 1/8 | duplo clique no corpo]; relógio fixo nos casos de posição, livre nos de gesto",
    comparacao: "ponto_fraccao_arco = labelPosition·L do comprimento de arco da polilinha; ponto_parametrico_indice = fracção do índice dos vértices; meio_por_indice = vértice do meio (nº ímpar de pontos) ou meio do trecho do meio (par); projeccao = ponto da polilinha mais perto do centro do texto",
    nota_fonte: "Caixa preta: nenhum código-fonte do Excalidraw lido.",
  },
  versoes: porVersao,
}, null, 1) + "\n");
