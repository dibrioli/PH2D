// Sonda: tldraw 5.5.2 montado (<Tldraw hideUi>, caixa preta, rato real do Playwright). O RÓTULO de uma seta
// ao longo da linha: onde fica para cada `labelPosition` (recta, curva, cotovelo), o arrasto do rótulo pelo
// rato (ao longo, além da ponta, na perpendicular), o duplo clique no corpo, e o rótulo longo numa seta curta
// (o vão da linha atrás do rótulo, medido nos píxeis da captura).
// Só API pública do Editor (createShapes, createBindings, select, setCamera, getShape, getShapeGeometry,
// getShapePageTransform, getEditingShapeId, getCurrentPageShapes) + DOCS.md do pacote.
// Uso: node sonda_tldraw_rotulo.mjs <pasta_saida>        (SO=<prefixo> corre só os casos com esse prefixo)
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/tldraw-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const PKG = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/tldraw/package.json"), "utf8"));
const outDir = process.argv[2];
fs.mkdirSync(outDir, { recursive: true });
const SCRIPT_SHA = createHash("sha256").update(fs.readFileSync(fileURLToPath(import.meta.url))).digest("hex");
const MIME = { ".html": "text/html", ".js": "text/javascript", ".woff2": "font/woff2", ".ttf": "font/ttf", ".css": "text/css" };
const server = createServer((req, res) => {
  const p = path.normalize(decodeURIComponent(new URL(req.url, "http://x").pathname));
  const file = path.join(WWW, p === "/" ? "index.html" : p);
  if (!file.startsWith(WWW) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404).end(); return; }
  res.writeHead(200, { "content-type": MIME[path.extname(file)] || "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const origin = `http://127.0.0.1:${server.address().port}`;
const seedScript = (seed) => `(() => { let a = ${seed} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a; t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61); return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next; Object.defineProperty(crypto, "getRandomValues", { value: (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength); for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; } }); })();`;

// câmara: ecrã = (página + camera) * z  (zoom 1: 1 px de ecrã = 1 un. de página)
const Z = 1, CX = 250, CY = 200;
const toScreen = ([x, y]) => [(x + CX) * Z, (y + CY) * Z];
const r2 = (v) => Math.round(v * 100) / 100;

const KINDS = {
  recta: { kind: "arc", bend: 0 },
  curva: { kind: "arc", bend: 120 },
  cotovelo: { kind: "elbow", bend: 0 },
};
const POSICOES = [0, 0.1, 0.25, 0.5, 0.75, 0.9, 1];
const CASOS_ALL = [];
for (const [k, v] of Object.entries(KINDS)) {
  for (const lp of POSICOES) CASOS_ALL.push({ id: `P_${k}_lp${lp}`, ...v, text: "sim", labelPosition: lp, tipo: "posicao" });
  // arrastos: o rótulo parte de 0.3 — a 0.5 o centro do rótulo coincide com a pega do meio (bend / elbowMidPoint)
  CASOS_ALL.push({ id: `D_${k}_a_ao_longo_3q`, ...v, text: "sim", labelPosition: 0.3, tipo: "arrasto", arrasto: "ao_longo_3q" });
  CASOS_ALL.push({ id: `D_${k}_b_alem_do_fim_200`, ...v, text: "sim", labelPosition: 0.3, tipo: "arrasto", arrasto: "alem_fim_200" });
  CASOS_ALL.push({ id: `D_${k}_b2_alem_do_inicio_200`, ...v, text: "sim", labelPosition: 0.3, tipo: "arrasto", arrasto: "alem_inicio_200" });
  CASOS_ALL.push({ id: `D_${k}_c_perpendicular_120`, ...v, text: "sim", labelPosition: 0.3, tipo: "arrasto", arrasto: "perp_120" });
  // a precedência: premir no centro do rótulo a 0.5 (onde está a pega do meio) e arrastar na perpendicular
  CASOS_ALL.push({ id: `H_${k}_lp0.5_centro_perpendicular_120`, ...v, text: "sim", labelPosition: 0.5, tipo: "arrasto", arrasto: "perp_120" });
  // varrimento do labelPosition pela API (mesmo contexto): a lei de corte junto às pontas
  for (const [tn, tx] of [["sim", "sim"], ["longo", "a long label here"]]) CASOS_ALL.push({ id: `V_${k}_${tn}`, ...v, text: tx, labelPosition: 0, tipo: "varrimento" });
}
CASOS_ALL.push({ id: "V_curta_recta_sim", ...KINDS.recta, r2: [160, 0], text: "sim", labelPosition: 0, tipo: "varrimento" });
CASOS_ALL.push({ id: "V_curta_recta_longo", ...KINDS.recta, r2: [160, 0], text: "a long label here", labelPosition: 0, tipo: "varrimento" });
for (const size of ["s", "m", "l", "xl"]) CASOS_ALL.push({ id: `V_horizontal_sim_size_${size}`, ...KINDS.recta, r2: [400, 0], size, text: "sim", labelPosition: 0, tipo: "varrimento" });
for (const size of ["m"]) CASOS_ALL.push({ id: `V_horizontal_longo_size_${size}`, ...KINDS.recta, r2: [400, 0], size, text: "a long label here", labelPosition: 0, tipo: "varrimento" });
CASOS_ALL.push({ id: "W_largura_do_rotulo_x_comprimento", ...KINDS.recta, r2: [160, 0], text: "a long label here", labelPosition: 0.5, tipo: "largura" });
CASOS_ALL.push({ id: "D_recta_a2_ao_longo_3q_100_passos", ...KINDS.recta, text: "sim", labelPosition: 0.3, tipo: "arrasto", arrasto: "ao_longo_3q", passos: 100 });
CASOS_ALL.push({ id: "K_recta_duplo_clique_corpo_sem_rotulo_lp0.3", ...KINDS.recta, text: "", labelPosition: 0.3, tipo: "duplo", em: 0.8 });
CASOS_ALL.push({ id: "K_cotovelo_duplo_clique_corpo_com_rotulo_lp0.3", ...KINDS.cotovelo, text: "sim", labelPosition: 0.3, tipo: "duplo", em: 0.8 });
CASOS_ALL.push({ id: "V_media_recta_sim_R2_260", ...KINDS.recta, r2: [260, 0], text: "sim", labelPosition: 0, tipo: "varrimento" });
CASOS_ALL.push({ id: "K_recta_duplo_clique_corpo_com_rotulo", ...KINDS.recta, text: "sim", labelPosition: 0.5, tipo: "duplo", em: 0.2 });
CASOS_ALL.push({ id: "K_recta_duplo_clique_corpo_sem_rotulo", ...KINDS.recta, text: "", labelPosition: 0.5, tipo: "duplo", em: 0.2 });
CASOS_ALL.push({ id: "K_cotovelo_duplo_clique_corpo_sem_rotulo", ...KINDS.cotovelo, text: "", labelPosition: 0.5, tipo: "duplo", em: 0.2 });
CASOS_ALL.push({ id: "G_recta_sim_vao", ...KINDS.recta, text: "sim", labelPosition: 0.5, tipo: "vao" });
CASOS_ALL.push({ id: "S_curta_rotulo_longo", ...KINDS.recta, r2: [160, 0], text: "a long label here", labelPosition: 0.5, tipo: "vao" });
CASOS_ALL.push({ id: "S_curta_rotulo_longo_lp0", ...KINDS.recta, r2: [160, 0], text: "a long label here", labelPosition: 0, tipo: "vao" });
CASOS_ALL.push({ id: "S_recta_rotulo_longo", ...KINDS.recta, text: "a long label here", labelPosition: 0.5, tipo: "vao" });
const CASOS = process.env.SO ? CASOS_ALL.filter((c) => c.id.startsWith(process.env.SO)) : CASOS_ALL;

const pageSetup = ({ caso, Z, CX, CY }) => {
  const ed = window.oracle.editor; const id = window.oracle.createShapeId;
  const R1 = id("R1"), R2 = id("R2"), A = id("A");
  const [r2x, r2y] = caso.r2 || [400, 300];
  // TLRichText à mão (o toRichText não está exposto na página): doc → paragraph → text (DOCS.md §Rich text)
  const richText = { type: "doc", content: [caso.text ? { type: "paragraph", content: [{ type: "text", text: caso.text }] } : { type: "paragraph" }] };
  ed.createShapes([
    { id: R1, type: "geo", x: 0, y: 0, props: { geo: "rectangle", w: 100, h: 100, fill: "none" } },
    { id: R2, type: "geo", x: r2x, y: r2y, props: { geo: "rectangle", w: 100, h: 100, fill: "none" } },
    { id: A, type: "arrow", x: 50, y: 50, props: { ...(caso.size ? { size: caso.size } : {}), kind: caso.kind, bend: caso.bend, color: "red", labelColor: "black", start: { x: 0, y: 0 }, end: { x: r2x, y: r2y }, richText, labelPosition: caso.labelPosition } },
  ]);
  ed.createBindings([
    { type: "arrow", fromId: A, toId: R1, props: { terminal: "start", normalizedAnchor: { x: 0.5, y: 0.5 }, isExact: false, isPrecise: false } },
    { type: "arrow", fromId: A, toId: R2, props: { terminal: "end", normalizedAnchor: { x: 0.5, y: 0.5 }, isExact: false, isPrecise: false } },
  ]);
  ed.setCamera({ x: CX, y: CY, z: Z });
  ed.selectNone();
  return true;
};
const pageSnap = () => {
  const ed = window.oracle.editor; const id = window.oracle.createShapeId;
  const A = ed.getShape(id("A"));
  const t = ed.getShapePageTransform(A);
  const g = ed.getShapeGeometry(A);
  const tp = (v) => { const p = t.applyToPoint(v); return [Math.round(p.x * 1000) / 1000, Math.round(p.y * 1000) / 1000]; };
  const kids = (g.children || [g]).map((c) => ({ classe: c.constructor?.name ?? null, isLabel: !!c.isLabel, vertices: (c.vertices || []).map(tp) }));
  const ignored = (g.ignoredChildren || []).map((c) => ({ classe: c.constructor?.name ?? null, isLabel: !!c.isLabel, vertices: (c.vertices || []).map(tp) }));
  const shapes = ed.getCurrentPageShapes().map((s) => ({ id: s.id, type: s.type }));
  const rt = A.props.richText;
  const plain = JSON.stringify(rt).match(/"text":"([^"]*)"/g)?.map((m) => m.slice(8, -1)).join("") ?? "";
  return JSON.parse(JSON.stringify({
    arrow: { x: A.x, y: A.y, rotation: A.rotation, props: A.props },
    texto_plano: plain,
    geometria_filhos: kids, geometria_ignorados: ignored,
    corpo_pagina_por_omissao: (g.vertices || []).map(tp),
    editing: ed.getEditingShapeId(), selected: ed.getSelectedShapeIds(), formas_na_pagina: shapes,
    estado: ed.getPath ? ed.getPath() : null,
  }));
};

// ---- geometria do lado do Node (comparação) ----
const dedup = (pts) => pts.filter((p, i) => i === 0 || Math.hypot(p[0] - pts[i - 1][0], p[1] - pts[i - 1][1]) > 1e-6);
const polyLen = (P) => { let L = 0; for (let i = 1; i < P.length; i++) L += Math.hypot(P[i][0] - P[i - 1][0], P[i][1] - P[i - 1][1]); return L; };
const atArc = (P, f) => { const L = polyLen(P); let s = f * L; for (let i = 1; i < P.length; i++) { const d = Math.hypot(P[i][0] - P[i - 1][0], P[i][1] - P[i - 1][1]); if (s <= d || i === P.length - 1) { const u = d ? Math.min(1, s / d) : 0; return [P[i - 1][0] + (P[i][0] - P[i - 1][0]) * u, P[i - 1][1] + (P[i][1] - P[i - 1][1]) * u]; } s -= d; } return P[P.length - 1]; };
// paramétrico = fracção do ÍNDICE dos vértices (cada trecho da polilinha vale o mesmo, seja qual for o comprimento)
const atParam = (P, f) => { const n = P.length - 1; const x = f * n; const i = Math.min(n - 1, Math.floor(x)); const u = x - i; return [P[i][0] + (P[i + 1][0] - P[i][0]) * u, P[i][1] + (P[i + 1][1] - P[i][1]) * u]; };
const atChord = (P, f) => [P[0][0] + (P[P.length - 1][0] - P[0][0]) * f, P[0][1] + (P[P.length - 1][1] - P[0][1]) * f];
// projecção no corpo: devolve {s (comprimento de arco), f = s/L, d (distância), ponto, tangente}
const project = (P, q) => { let best = null, acc = 0; for (let i = 1; i < P.length; i++) { const a = P[i - 1], b = P[i]; const dx = b[0] - a[0], dy = b[1] - a[1]; const d2 = dx * dx + dy * dy; const len = Math.sqrt(d2); let u = d2 ? ((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / d2 : 0; u = Math.max(0, Math.min(1, u)); const p = [a[0] + dx * u, a[1] + dy * u]; const dist = Math.hypot(q[0] - p[0], q[1] - p[1]); if (!best || dist < best.d - 1e-9) best = { s: acc + u * len, d: dist, ponto: p, tangente: len ? [dx / len, dy / len] : [0, 0] }; acc += len; } best.f = best.s / acc; return best; };
const dist = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const R = (p) => [r2(p[0]), r2(p[1])];

function analisa(s, lp) {
  const corpoK = s.geometria_filhos.find((k) => !k.isLabel);
  const labelK = s.geometria_filhos.find((k) => k.isLabel) || s.geometria_ignorados.find((k) => k.isLabel) || null;
  const corpo = dedup(corpoK ? corpoK.vertices : s.corpo_pagina_por_omissao);
  const L = polyLen(corpo);
  const out = { corpo_n: corpo.length, corpo_comprimento: r2(L), corpo_inicio: R(corpo[0]), corpo_fim: R(corpo[corpo.length - 1]) };
  if (!labelK) return { ...out, rotulo: null, corpo_pagina: corpo.map(R) };
  const V = labelK.vertices;
  const c = [V.reduce((a, v) => a + v[0], 0) / V.length, V.reduce((a, v) => a + v[1], 0) / V.length];
  const xs = V.map((v) => v[0]), ys = V.map((v) => v[1]);
  const alinhado = V.every((v, i) => { const w = V[(i + 1) % V.length]; return Math.abs(v[0] - w[0]) < 1e-6 || Math.abs(v[1] - w[1]) < 1e-6; });
  const pr = project(corpo, c);
  const rot = { vertices: V.map(R), centro: R(c), caixa: { x: r2(Math.min(...xs)), y: r2(Math.min(...ys)), w: r2(Math.max(...xs) - Math.min(...xs)), h: r2(Math.max(...ys) - Math.min(...ys)) }, alinhado_aos_eixos: alinhado,
    projeccao_no_corpo: { s: r2(pr.s), fraccao_arco: Math.round(pr.f * 10000) / 10000, distancia: r2(pr.d), ponto: R(pr.ponto) },
    distancia_ao_inicio_do_corpo_ao_longo: r2(pr.s), distancia_ao_fim_do_corpo_ao_longo: r2(L - pr.s) };
  if (lp != null) {
    const pa = atArc(corpo, lp), pp = atParam(corpo, lp), pc = atChord(corpo, lp);
    rot.comparacao_lp = { labelPosition: lp, ponto_fraccao_arco: R(pa), ponto_parametrico_indice: R(pp), ponto_corda: R(pc), dist_ao_ponto_arco: r2(dist(c, pa)), dist_ao_parametrico: r2(dist(c, pp)), dist_a_corda: r2(dist(c, pc)) };
  }
  return { ...out, rotulo: rot, corpo_pagina: corpo.map(R) };
}

// vão: amostra a captura ao longo do corpo, 1 px de página por amostra; vermelho = a linha (color red)
const pixelProbe = async (page, png, pts) => page.evaluate(async ({ b64, pts }) => {
  const img = new Image(); img.src = "data:image/png;base64," + b64; await img.decode();
  const cv = document.createElement("canvas"); cv.width = img.width; cv.height = img.height;
  const cx = cv.getContext("2d"); cx.drawImage(img, 0, 0);
  const d = cx.getImageData(0, 0, img.width, img.height).data;
  return pts.map(([x, y]) => { let red = false; for (let oy = -1; oy <= 1; oy++) for (let ox = -1; ox <= 1; ox++) { const X = Math.round(x) + ox, Y = Math.round(y) + oy; const i = (Y * img.width + X) * 4; if (d[i] > 150 && d[i + 1] < 110 && d[i + 2] < 110) red = true; } return red; });
}, { b64: png.toString("base64"), pts });

async function drag(page, from, to, steps = 25) {
  await page.mouse.move(from[0], from[1]); await page.waitForTimeout(100);
  await page.mouse.down(); await page.waitForTimeout(100);
  for (let i = 1; i <= steps; i++) { await page.mouse.move(from[0] + (to[0] - from[0]) * i / steps, from[1] + (to[1] - from[1]) * i / steps); await page.waitForTimeout(16); }
  await page.waitForTimeout(150); await page.mouse.up(); await page.waitForTimeout(300);
}

const browser = await chromium.launch({ headless: true });
const results = [];
const external = new Set();
for (const caso of CASOS) {
  const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
  context.on("request", (r) => { const u = r.url(); if (!u.startsWith(origin) && !u.startsWith("blob:") && !u.startsWith("data:")) external.add(u.split("?")[0]); });
  await context.addInitScript(seedScript(0x1a6e1));
  const page = await context.newPage();
  const errs = []; page.on("pageerror", (e) => errs.push(e.message));
  await page.goto(origin + "/", { timeout: 60000 });
  await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 60000 });
  // aquece a letra «draw»: sem isto a 1.ª medida do rótulo sai com a letra de recurso (39,61 × 41,5 de largura, medido)
  await page.evaluate(async () => { const ed = window.oracle.editor; const id = window.oracle.createShapeId("aquece");
    ed.createShapes([{ id, type: "text", x: -900, y: -900, props: { richText: { type: "doc", content: [{ type: "paragraph", content: [{ type: "text", text: "sim a long label here novo" }] }] } } }]);
    await document.fonts.ready; await new Promise((r) => setTimeout(r, 600)); ed.deleteShapes([id]); return true; });
  await page.evaluate(pageSetup, { caso, Z, CX, CY });
  await page.waitForTimeout(400);
  const passos = [];
  const s0 = await page.evaluate(pageSnap);
  const a0 = analisa(s0, caso.labelPosition);
  passos.push({ passo: "montado", ...a0, labelPosition: s0.arrow.props.labelPosition, editing: s0.editing, n_formas: s0.formas_na_pagina.length, bruto: s0 });
  if (caso.tipo === "arrasto") {
    await page.evaluate(() => { window.oracle.editor.select(window.oracle.createShapeId("A")); return true; });
    await page.waitForTimeout(250);
    const corpo = dedup(a0.corpo_pagina);
    const from = a0.rotulo.centro;
    let to;
    if (caso.arrasto === "ao_longo_3q") to = atArc(corpo, 0.75);
    else if (caso.arrasto === "alem_fim_200") { const n = corpo.length; const a = corpo[n - 2], b = corpo[n - 1]; const l = dist(a, b); to = [b[0] + (b[0] - a[0]) / l * 200, b[1] + (b[1] - a[1]) / l * 200]; }
    else if (caso.arrasto === "alem_inicio_200") { const a = corpo[1], b = corpo[0]; const l = dist(a, b); to = [b[0] + (b[0] - a[0]) / l * 200, b[1] + (b[1] - a[1]) / l * 200]; }
    else { const pr = project(corpo, from); const [tx, ty] = pr.tangente; to = [from[0] - ty * 120, from[1] + tx * 120]; }
    await drag(page, toScreen(from), toScreen(to), caso.passos ?? 25);
    const s1 = await page.evaluate(pageSnap);
    const a1 = analisa(s1, s1.arrow.props.labelPosition);
    const prTo = project(corpo, to);
    passos.push({ passo: `arrasto_${caso.arrasto}`, de_pagina: R(from), para_pagina: R(to), alvo_projectado_no_corpo: { fraccao_arco: Math.round(prTo.f * 10000) / 10000, distancia: r2(prTo.d), ponto: R(prTo.ponto) },
      ...a1, labelPosition: s1.arrow.props.labelPosition, selected: s1.selected, editing: s1.editing, n_formas: s1.formas_na_pagina.length, corpo_mudou: JSON.stringify(a1.corpo_pagina) !== JSON.stringify(a0.corpo_pagina), bend_depois: s1.arrow.props.bend });
  } else if (caso.tipo === "varrimento") {
    const tabela = [];
    for (let i = 0; i <= 50; i++) {
      const lp = Math.round(i * 0.02 * 100) / 100;
      await page.evaluate((lp) => { const ed = window.oracle.editor; ed.updateShapes([{ id: window.oracle.createShapeId("A"), type: "arrow", props: { labelPosition: lp } }]); return true; }, lp);
      const s = await page.evaluate(pageSnap);
      const a = analisa(s, lp);
      tabela.push({ labelPosition: lp, prop_gravada: s.arrow.props.labelPosition, centro: a.rotulo?.centro, s: a.rotulo?.projeccao_no_corpo.s, fraccao_arco: a.rotulo?.projeccao_no_corpo.fraccao_arco, d_corpo: a.rotulo?.projeccao_no_corpo.distancia, caixa: a.rotulo?.caixa, alinhado: a.rotulo?.alinhado_aos_eixos });
    }
    passos.push({ passo: "varrimento_labelPosition_0_a_1_passo_0.02_por_updateShapes", corpo_comprimento: a0.corpo_comprimento, corpo_pagina: a0.corpo_pagina, tabela });
  } else if (caso.tipo === "largura") {
    const tabela = [];
    for (const x of [160, 180, 200, 230, 260, 300, 350, 400, 500, 600, 800]) {
      await page.evaluate((x) => { const ed = window.oracle.editor; ed.updateShapes([{ id: window.oracle.createShapeId("R2"), type: "geo", x }]); return true; }, x);
      await page.waitForTimeout(100);
      const s = await page.evaluate(pageSnap);
      const a = analisa(s, 0.5);
      tabela.push({ R2_x: x, corpo_comprimento: a.corpo_comprimento, caixa: a.rotulo?.caixa, centro: a.rotulo?.centro });
    }
    passos.push({ passo: "R2_x_varia_rotulo_longo_horizontal", tabela });
  } else if (caso.tipo === "duplo") {
    const corpo = dedup(a0.corpo_pagina);
    const q = atArc(corpo, caso.em);
    await page.mouse.dblclick(...toScreen(q)); await page.waitForTimeout(400);
    const s1 = await page.evaluate(pageSnap);
    const a1 = analisa(s1, s1.arrow.props.labelPosition);
    passos.push({ passo: `duplo_clique_no_corpo_a_${caso.em}_do_arco`, ponto_pagina: R(q), ...a1, labelPosition: s1.arrow.props.labelPosition, editing: s1.editing, selected: s1.selected, formas_na_pagina: s1.formas_na_pagina, estado: s1.estado });
    await page.keyboard.type("novo"); await page.waitForTimeout(200);
    await page.keyboard.press("Escape"); await page.waitForTimeout(300);
    const s2 = await page.evaluate(pageSnap);
    const a2 = analisa(s2, s2.arrow.props.labelPosition);
    passos.push({ passo: "escreve_novo_e_Escape", ...a2, labelPosition: s2.arrow.props.labelPosition, texto_plano: s2.texto_plano, editing: s2.editing, formas_na_pagina: s2.formas_na_pagina });
  } else if (caso.tipo === "vao") {
    const png = await page.screenshot();
    const corpo = dedup(a0.corpo_pagina);
    const L = polyLen(corpo); const N = Math.floor(L);
    const amostras = []; for (let i = 0; i <= N; i++) amostras.push(atArc(corpo, i / N));
    const red = await pixelProbe(page, png, amostras.map(toScreen));
    // corridas de «sem vermelho» ao longo do arco
    const runs = []; let st = null;
    red.forEach((v, i) => { if (!v && st === null) st = i; if ((v || i === red.length - 1) && st !== null) { runs.push({ s0: r2(st * L / N), s1: r2((v ? i - 1 : i) * L / N) }); st = null; } });
    const box = a0.rotulo?.caixa;
    const dentro = box ? amostras.map((p, i) => ({ p, v: red[i] })).filter(({ p }) => p[0] >= box.x && p[0] <= box.x + box.w && p[1] >= box.y && p[1] <= box.y + box.h) : [];
    passos.push({ passo: "vao_da_linha_na_captura", metodo: "seta color red, rótulo labelColor black; amostra 1 un./px ao longo do corpo (comprimento de arco), vizinhança 3×3, vermelho = R>150,G<110,B<110", amostras: red.length,
      corridas_sem_linha_no_arco: runs, amostras_dentro_da_caixa_do_rotulo: dentro.length, vermelhas_dentro_da_caixa: dentro.filter((x) => x.v).length,
      vermelhas_fora_da_caixa: red.filter((v, i) => !dentro.some((x) => x.p === amostras[i]) && v).length, fora_da_caixa: red.length - dentro.length });
  }
  fs.writeFileSync(path.join(outDir, `tldraw_${caso.id}.png`), await page.screenshot());
  results.push({ caso: caso.id, entrada: caso, passos, pageerrors: errs });
  const last = passos[passos.length - 1];
  console.log(caso.id, JSON.stringify({ lp: last.labelPosition, centro: last.rotulo?.centro, f: last.rotulo?.projeccao_no_corpo?.fraccao_arco, d: last.rotulo?.projeccao_no_corpo?.distancia, cmp: last.rotulo?.comparacao_lp, ed: last.editing, runs: last.corridas_sem_linha_no_arco }));
  await context.close();
}
fs.writeFileSync(path.join(outDir, "tldraw_rotulo.json"), JSON.stringify({
  cabecalho: {
    oraculo: `tldraw ${PKG.version} (tldraw license — uso de desenvolvimento/teste, local), <Tldraw hideUi> em Chromium ${browser.version()} headless, rato real do Playwright`,
    data: new Date().toISOString(), host: os.hostname(), loadavg: os.loadavg(),
    comando: `node ${fileURLToPath(import.meta.url)} ${outDir}`, script_sha256: SCRIPT_SHA,
    entrada: "R1 geo rectangle (0,0,100,100) fill none; R2 (400,300,100,100) [S_curta*: (160,0)]; seta color red, labelColor black, richText de um parágrafo, ligada start→R1, end→R2 (normalizedAnchor 0.5,0.5, imprecisas); kind arc bend 0 (recta) · arc bend 120 (curva) · elbow (cotovelo)",
    camera: { x: CX, y: CY, z: Z, nota: "ecrã = (página + camera) * z" },
    geometria: "editor.getShapeGeometry(seta).children (Group2d): o filho isLabel é o rectângulo do rótulo; o outro é o corpo; vértices por getShapePageTransform",
    comparacao: "ponto_fraccao_arco = ponto a labelPosition·L do comprimento de arco do corpo; ponto_parametrico_indice = fracção do índice dos vértices (cada trecho pesa igual); ponto_corda = interpolação recta entre as pontas do corpo; projeccao_no_corpo = ponto do corpo mais perto do centro do rótulo (s, s/L, distância)",
    pedidos_externos: [...external].sort(),
    nota_fonte: "Caixa preta: nenhum código-fonte do tldraw lido; só DOCS.md do pacote (= tldraw.dev).",
  },
  casos: results,
}, null, 2) + "\n");
await browser.close(); server.close();
