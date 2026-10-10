// Oráculo de CONTRASTE da colaboração ao vivo: tldraw sync 5.5.2 num servidor LOCAL (servidor.mjs),
// dois ou três contextos do Playwright na MESMA sala, gestos de rato/teclado como um utilizador, estado
// lido pela API do editor exposta em execução (window.oracle.editor) e pelo snapshot público da sala
// (TLSocketRoom.getCurrentSnapshot, com lastChangedClock por registo). Nenhum fonte do tldraw é lido.
//
// Concorrência: page.routeWebSocket entre a página e o servidor. «Ao mesmo tempo» = as mensagens de
// SAÍDA dos pares retidas enquanto ambos fazem o gesto, libertadas pela ordem do caso. Duplicar e
// reordenar (c16, c17) também por aí; sem rede = context.setOffline (c14, c15). Cada mensagem fica
// registada: par, direcção, bytes, instante relativo, tipo e chaves do diff.
//
// Uso: node corre_colab.mjs [--so c01,c05] <pasta_saida>
//   ORACLE_HOME (padrão ~/Apps/tldraw-sync-oracle). Lança o servidor como filho, mata-o pelo PID.
import { createRequire } from "node:module";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/tldraw-sync-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const HERE = path.dirname(fileURLToPath(import.meta.url));
const CASOS_PATH = path.join(HERE, "..", "casos.json");
const CASOS = JSON.parse(fs.readFileSync(CASOS_PATH, "utf8")).casos;
const sha = (b) => createHash("sha256").update(b).digest("hex");

let args = process.argv.slice(2);
const opt = (flag) => { const i = args.indexOf(flag); if (i < 0) return null; const v = args[i + 1]; args.splice(i, 2); return v; };
const so = opt("--so");
const [outDir] = args;
if (!outDir) { console.error("uso: node corre_colab.mjs [--so c01,c02] <pasta_saida>"); process.exit(2); }
fs.mkdirSync(outDir, { recursive: true });

const WEB = "http://127.0.0.1:5201";
const API = "http://127.0.0.1:5202";
const CORES = { A: "#e03131", B: "#1971c2", C: "#2f9e44" };
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const r3 = (v) => (typeof v === "number" ? Math.round(v * 1000) / 1000 : v);

// ------------------------------------------------------------------ servidor (filho, morto pelo PID)
const srv = spawn(process.execPath, [path.join(HERE, "servidor.mjs")], {
  env: { ...process.env, ORACLE_HOME, PRAZO_MS: String(40 * 60 * 1000) }, stdio: ["ignore", "pipe", "pipe"],
});
const SRV_PID = srv.pid;
let srvErr = "";
srv.stderr.on("data", (d) => { srvErr += d; });
const mataServidor = () => { try { process.kill(SRV_PID, "SIGTERM"); } catch {} };
process.on("exit", mataServidor);
await Promise.race([
  new Promise((r) => srv.stdout.on("data", (d) => { if (String(d).includes("PRONTO")) r(); })),
  wait(20000).then(() => { throw new Error("servidor não ficou pronto: " + srvErr); }),
]);

const browser = await chromium.launch({ headless: true });
const chromiumVersion = browser.version();

// ------------------------------------------------------------------ registo por subcorrida
let T0 = Date.now();
let MSGS = [];
let FASE = "preparo";
let EXTERNOS = [];
let CONSOLA = [];
let ROTULOS = new Map(); // id -> rótulo estável (X, Y, A#1…)
const rot = (id) => (ROTULOS.has(id) ? ROTULOS.get(id) : id);
const rotIds = (s) => (typeof s === "string" ? s.replace(/shape:[A-Za-z0-9_-]+/g, (m) => rot(m)) : s);

function resumo(m) {
  try {
    const j = JSON.parse(typeof m === "string" ? m : Buffer.from(m).toString("utf8"));
    const chaves = (diff) => Object.keys(diff || {}).map((k) => `${rotIds(k)}:${diff[k][0]}`);
    if (j.type === "push") {
      const corpo = j.diff && Object.keys(j.diff).some((k) => !k.startsWith("user:")) && JSON.stringify(j.diff).length < 600 ? rotIds(JSON.stringify(j.diff)) : undefined;
      return { tipo: "push", clientClock: j.clientClock, diff: chaves(j.diff), presenca: !!j.presence, ...(corpo ? { corpo } : {}) };
    }
    if (j.type === "data") return { tipo: "data", itens: j.data.map((d) => d.type === "push_result" ? `push_result:${d.action}:${d.clientClock}` : d.type === "patch" ? `patch@${d.serverClock}:[${chaves(d.diff).join(",")}]` : d.type) };
    if (j.type === "connect") return { tipo: "connect", hydration: j.hydrationType ?? null, diff: j.diff ? Object.keys(j.diff).length : undefined };
    return { tipo: j.type };
  } catch { return { tipo: "?" }; }
}
const ehDocMsg = (e) => {
  if (e.tipo === "push") return e.diff.some((k) => !k.startsWith("user:") && !k.startsWith("instance_presence"));
  if (e.tipo === "data") return e.itens.some((i) => i.startsWith("patch@") && i.slice(i.indexOf("[") + 1, -1).split(",").some((k) => k && !k.startsWith("user:") && !k.startsWith("instance_presence")));
  return false;
};

// Determinismo: PRNG semeado por par/caso (ids de formas criadas por gesto, ids de sessão).
const seedScript = (seed) => `(() => {
  let a = ${seed} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next;
  const gr = (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength);
    for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; };
  Object.defineProperty(crypto, "getRandomValues", { value: gr });
  Object.defineProperty(crypto, "randomUUID", { value: () => { const h = [...gr(new Uint8Array(16))].map((x) => x.toString(16).padStart(2, "0")).join(""); return h.slice(0,8)+"-"+h.slice(8,12)+"-4"+h.slice(13,16)+"-a"+h.slice(17,20)+"-"+h.slice(20,32); } });
})();`;

const LOCAL = /^(https?|wss?):\/\/(127\.0\.0\.1|localhost)(:\d+)?\//;
let PARES = [];
async function par(nome, sala, semente) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC" });
  await ctx.route((u) => !LOCAL.test(u.href) && !u.href.startsWith("data:") && !u.href.startsWith("blob:"), (r) => {
    EXTERNOS.push({ par: nome, url: r.request().url() }); r.abort();
  });
  await ctx.addInitScript(seedScript(parseInt(sha(semente + "/" + nome).slice(0, 8), 16)));
  const page = await ctx.newPage();
  const P = { nome, ctx, page, sala, holdOut: false, holdIn: false, qOut: [], qIn: [], conexoes: 0, eventos: [] };
  await page.routeWebSocket(/^ws:\/\/127\.0\.0\.1:5202\//, (ws) => {
    const s = ws.connectToServer();
    const conn = ++P.conexoes;
    P.eventos.push({ t: Date.now() - T0, ev: "abre", conn, fase: FASE });
    const reg = (dir, m) => {
      const e = { par: nome, dir: dir === "out" ? `${nome}→S` : `S→${nome}`, bytes: typeof m === "string" ? Buffer.byteLength(m) : m.length, t: Date.now() - T0, conn, fase: FASE, ...resumo(m) };
      MSGS.push(e);
      return e;
    };
    ws.onMessage((m) => { const e = reg("out", m); if (P.holdOut) { e.retida = true; P.qOut.push({ m, e, s }); } else s.send(m); });
    s.onMessage((m) => { const e = reg("in", m); if (P.holdIn) { e.retida = true; P.qIn.push({ m, e, ws }); } else ws.send(m); });
    ws.onClose((c, r) => P.eventos.push({ t: Date.now() - T0, ev: "fecha-página", conn, codigo: c }));
    s.onClose((c, r) => P.eventos.push({ t: Date.now() - T0, ev: "fecha-servidor", conn, codigo: c }));
  });
  page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") CONSOLA.push(`${nome} ${m.type()}: ${m.text().slice(0, 200)}`); });
  page.on("pageerror", (e) => CONSOLA.push(`${nome} pageerror: ${e.message.slice(0, 200)}`));
  const tGoto = Date.now();
  await page.goto(`${WEB}/?sala=${encodeURIComponent(sala)}&nome=${nome}&cor=${encodeURIComponent(CORES[nome])}`, { timeout: 60000 });
  await page.waitForFunction(() => window.oracleReady && window.oracle.status === "synced-remote" && window.oracle.connectionStatus === "online", null, { timeout: 60000 });
  P.tPronto = Date.now() - tGoto;
  await page.waitForTimeout(300);
  PARES.push(P);
  return P;
}
async function fechaPares() { for (const P of PARES) await P.ctx.close().catch(() => {}); PARES = []; }

// Liberta a fila retida. ordem: "fifo" | "reverso_doc" (só as que trazem documento, entre si, invertidas);
// duplica: as que trazem documento vão DUAS vezes seguidas.
function liberta(P, dir = "out", { ordem = "fifo", duplica = false } = {}) {
  const q = dir === "out" ? P.qOut.splice(0) : P.qIn.splice(0);
  let seq = q;
  if (ordem === "reverso_doc") {
    const docs = q.filter((x) => ehDocMsg(x.e)).reverse();
    let k = 0;
    seq = q.map((x) => (ehDocMsg(x.e) ? docs[k++] : x));
  }
  const t = Date.now() - T0;
  let n = 0;
  for (const x of seq) {
    const vezes = duplica && ehDocMsg(x.e) ? 2 : 1;
    for (let v = 0; v < vezes; v++) {
      try { (dir === "out" ? x.s : x.ws).send(x.m); n++; x.e.libertada = (x.e.libertada || []).concat(t); } catch (err) { x.e.descartada = String(err.message).slice(0, 80); }
    }
  }
  if (dir === "out") P.holdOut = false; else P.holdIn = false;
  return { par: P.nome, dir, ordem, duplica, mensagens: q.length, enviadas: n, t };
}

// ------------------------------------------------------------------ lado da página
const pgRaw = () => {
  const { editor } = window.oracle;
  const shapes = editor.getCurrentPageShapesSorted().map((s) => {
    const b = editor.getShapePageBounds(s.id);
    return { rec: s, bounds: b && { x: b.x, y: b.y, w: b.w, h: b.h }, bindings: editor.getBindingsFromShape(s.id, "arrow") };
  });
  return { shapes, pagina: editor.getCurrentPageId() };
};
const pgCriaCena = (cena) => {
  const { editor, createShapeId, toRichText } = window.oracle;
  editor.run(() => editor.createShapes(cena.map(({ nome, texto, ...s }) => ({ ...s, id: createShapeId(nome), props: { ...(s.props || {}), ...(texto !== undefined ? { richText: toRichText(texto) } : {}) } }))), { history: "ignore" });
  editor.selectNone();
  return true;
};
const pgIds = () => window.oracle.editor.getCurrentPageShapeIds ? [...window.oracle.editor.getCurrentPageShapeIds()] : [];
const pgToScreen = ([x, y]) => { const p = window.oracle.editor.pageToScreen({ x, y }); return [p.x, p.y]; };

const txt = (rt) => {
  if (!rt) return undefined;
  const ps = [];
  const walk = (n, acc) => { if (n.text) acc.push(n.text); (n.content || []).forEach((c) => walk(c, acc)); };
  for (const p of rt.content || []) { const acc = []; walk(p, acc); ps.push(acc.join("")); }
  return ps.join("\n");
};
// Normaliza um registo de forma (do cliente ou do servidor) nos campos do caso.
const normShape = (s) => ({
  id: rot(s.id), type: s.type, index: s.index, x: r3(s.x), y: r3(s.y), rot: r3(s.rotation),
  ...(s.props?.w !== undefined ? { w: r3(s.props.w), h: r3(s.props.h) } : {}),
  ...(s.props?.color !== undefined ? { cor: s.props.color } : {}),
  ...(s.props?.fill !== undefined ? { fill: s.props.fill } : {}),
  ...(s.props?.richText !== undefined ? { texto: txt(s.props.richText) } : {}),
  ...(s.type === "arrow" ? { start: s.props.start && { x: r3(s.props.start.x), y: r3(s.props.start.y) }, end: s.props.end && { x: r3(s.props.end.x), y: r3(s.props.end.y) } } : {}),
});
const normBinding = (b) => ({ id: `${rot(b.fromId)}→${rot(b.toId)}:${b.props?.terminal}`, type: b.type, de: rot(b.fromId), para: rot(b.toId), terminal: b.props?.terminal });

async function estadoPar(P) {
  const raw = await P.page.evaluate(pgRaw);
  const formas = raw.shapes.map((x, z) => ({ ...normShape(x.rec), z }));
  const bindings = raw.shapes.flatMap((x) => x.bindings.map(normBinding)).sort((a, b) => a.id.localeCompare(b.id));
  const caixas = Object.fromEntries(raw.shapes.map((x) => [rot(x.rec.id), x.bounds && { x: r3(x.bounds.x), y: r3(x.bounds.y), w: r3(x.bounds.w), h: r3(x.bounds.h) }]));
  return { formas, bindings, caixas };
}
async function estadoServidor(sala) {
  const res = await fetch(`${API}/snapshot/${encodeURIComponent(sala)}`);
  if (!res.ok) return null;
  const snap = await res.json();
  const docs = snap.documents.filter((d) => d.state.typeName === "shape");
  const formas = docs.map((d) => ({ ...normShape(d.state), clock: d.lastChangedClock }))
    .sort((a, b) => (a.index < b.index ? -1 : a.index > b.index ? 1 : a.id.localeCompare(b.id)));
  const bindings = snap.documents.filter((d) => d.state.typeName === "binding").map((d) => ({ ...normBinding(d.state), clock: d.lastChangedClock })).sort((a, b) => a.id.localeCompare(b.id));
  const lapides = Object.fromEntries(Object.entries(snap.tombstones || {}).filter(([k]) => k.startsWith("shape:") || k.startsWith("binding:")).map(([k, v]) => [rot(k), v]));
  return { relogio: snap.clock, relogioDocumento: snap.documentClock, formas, bindings, lapides };
}
// A chave de comparação: os registos (sem z nem caixas — o servidor não tem z; o índice manda).
const chaveDoc = (e) => JSON.stringify({ f: e.formas.map(({ z, clock, ...f }) => f).sort((a, b) => a.id.localeCompare(b.id)), b: e.bindings.map(({ clock, ...b }) => b) });

async function estados(pares, sala) {
  const out = {};
  for (const P of pares) out[P.nome] = await estadoPar(P);
  out.servidor = await estadoServidor(sala);
  return out;
}
async function convergencia(pares, sala, prazo = 8000) {
  const t0 = Date.now();
  let est;
  while (true) {
    est = await estados(pares, sala);
    const ks = Object.values(est).map((e) => (e ? chaveDoc(e) : "null"));
    if (ks.every((k) => k === ks[0])) return { convergiu: true, ms: Date.now() - t0, estado: est };
    if (Date.now() - t0 > prazo) return { convergiu: false, ms: Date.now() - t0, estado: est };
    await wait(60);
  }
}

// ------------------------------------------------------------------ gestos (rato/teclado do Playwright)
const scr = async (P, p) => P.page.evaluate(pgToScreen, p);
async function arrasta(P, de, para, { passos = 10, pausa = 16, soltar = true, agarra = true } = {}) {
  const a = await scr(P, de), b = await scr(P, para);
  if (agarra) { await P.page.mouse.move(a[0], a[1]); await P.page.mouse.down(); }
  for (let i = 1; i <= passos; i++) {
    await P.page.mouse.move(a[0] + ((b[0] - a[0]) * i) / passos, a[1] + ((b[1] - a[1]) * i) / passos);
    if (pausa) await P.page.waitForTimeout(pausa);
  }
  if (soltar) await P.page.mouse.up();
}
async function clica(P, p, opts = {}) { const a = await scr(P, p); await P.page.mouse.click(a[0], a[1], opts); }
async function tecla(P, k) { await P.page.keyboard.press(k); }
async function novasFormas(P, antes) { const depois = await P.page.evaluate(pgIds); return depois.filter((i) => !antes.includes(i)); }
const contador = {};
function rotula(id, dono) { contador[dono] = (contador[dono] || 0) + 1; ROTULOS.set(id, `${dono}#${contador[dono]}`); return ROTULOS.get(id); }
async function criaRect(P, de, para) {
  const antes = await P.page.evaluate(pgIds);
  await tecla(P, "r");
  await arrasta(P, de, para, { passos: 6 });
  await P.page.waitForTimeout(80);
  const n = await novasFormas(P, antes);
  return n.map((id) => rotula(id, P.nome));
}
async function criaSeta(P, de, para) {
  const antes = await P.page.evaluate(pgIds);
  await tecla(P, "a");
  await arrasta(P, de, para, { passos: 10 });
  await P.page.waitForTimeout(80);
  const n = await novasFormas(P, antes);
  return n.map((id) => rotula(id, P.nome));
}
async function mudaCor(P, cor) { await P.page.click(`[data-testid="style.color.${cor}"]`); }
async function editaTexto(P, p, { substitui, acrescenta }) {
  const a = await scr(P, p);
  await P.page.mouse.dblclick(a[0], a[1]);
  // pronto = a forma em edição E o ProseMirror com o foco e o texto todo seleccionado (como o produto entra)
  await P.page.waitForFunction(() => window.oracle.editor.getEditingShapeId() && document.activeElement?.classList.contains("ProseMirror") && String(getSelection()).length > 0, null, { timeout: 5000 });
  // medido: com 80 ms o produto ainda volta a seleccionar tudo DEPOIS da seta (o «aaa» substituía o texto)
  await P.page.waitForTimeout(400);
  if (substitui !== undefined) { await tecla(P, "Control+a"); await P.page.keyboard.type(substitui); }
  else {
    await tecla(P, "ArrowRight");
    await P.page.waitForFunction(() => getSelection().isCollapsed, null, { timeout: 2000 });
    await P.page.waitForTimeout(250); // o ProseMirror só lê a selecção do DOM depois do selectionchange (medido: sem isto o texto era substituído)
    await P.page.keyboard.type(acrescenta);
  }
  await P.page.waitForTimeout(100);
  await tecla(P, "Escape");
}
const idDe = (nome) => `shape:${nome}`;

// ------------------------------------------------------------------ subcorrida
let SUB = null;
async function subcorrida(caso, nome, nomesPares, cena, corpo) {
  ROTULOS = new Map(); for (const k of Object.keys(contador)) delete contador[k];
  for (const s of cena) ROTULOS.set(idDe(s.nome), s.nome);
  MSGS = []; EXTERNOS = []; CONSOLA = []; FASE = "preparo"; T0 = Date.now();
  const sala = `${caso}-${nome}-${Date.now().toString(36)}`;
  const pares = {};
  for (const n of nomesPares) pares[n] = await par(n, sala, `${caso}/${nome}`);
  const P0 = pares[nomesPares[0]];
  if (cena.length) await P0.page.evaluate(pgCriaCena, cena);
  const prep = await convergencia(Object.values(pares), sala);
  FASE = "caso"; T0 = Date.now();
  const gestos = [];
  SUB = { sala, pares, gestos, extra: {}, deterministico: {}, variavel: {} };
  const g = (s) => gestos.push({ t: Date.now() - T0, gesto: s });
  try {
    await corpo({ ...pares, sala, g, sub: SUB });
  } catch (err) {
    SUB.erro = String(err.stack || err).slice(0, 600);
  }
  const fim = SUB.semConvergencia ? null : await convergencia(Object.values(pares), sala, SUB.prazoConvergencia || 8000);
  const docs = MSGS.filter((e) => e.fase === "caso");
  const porDir = {};
  for (const e of docs) { const k = e.dir; porDir[k] = porDir[k] || { mensagens: 0, bytes: 0, com_documento: 0 }; porDir[k].mensagens++; porDir[k].bytes += e.bytes; if (ehDocMsg(e)) porDir[k].com_documento++; }
  const out = {
    subcorrida: nome, sala, pares: nomesPares,
    cena: cena.map((s) => ({ ...s })),
    gestos,
    deterministico: {
      ...(fim ? { convergiu: fim.convergiu, final: fim.estado.servidor ? { ...Object.fromEntries(nomesPares.map((n) => [n, fim.estado[n].formas.map(({ index, ...f }) => ({ index, ...f }))])), servidor: fim.estado.servidor.formas.map(({ clock, ...f }) => f), bindings: Object.fromEntries(nomesPares.map((n) => [n, fim.estado[n].bindings])), bindings_servidor: fim.estado.servidor.bindings.map(({ clock, ...b }) => b), lapides_servidor: Object.keys(fim.estado.servidor.lapides).map((k) => (k.startsWith("binding:") ? "binding (id aleatório)" : k)).sort() } : null } : {}),
      ...SUB.deterministico,
    },
    variavel: {
      ...(fim ? { ms_ate_convergir_depois_do_ultimo_gesto: fim.ms } : {}),
      preparo_convergiu_ms: prep.ms,
      mensagens_por_direccao: porDir,
      ...SUB.variavel,
      conexoes: Object.fromEntries(nomesPares.map((n) => [n, { conexoes: pares[n].conexoes, eventos: pares[n].eventos.filter((e) => e.fase !== "preparo" || e.ev !== "abre") }])),
      mensagens: docs.length > 1500 ? { truncado: true, total: docs.length, primeiras: docs.slice(0, 1500) } : docs,
    },
    ...(SUB.erro ? { erro: SUB.erro } : {}),
    externos_bloqueados: EXTERNOS,
    consola: CONSOLA.slice(0, 40),
  };
  if (fim) out.variavel.clocks_servidor = Object.fromEntries(fim.estado.servidor.formas.map((f) => [f.id, f.clock]));
  await fechaPares();
  return out;
}

// Retém a SAÍDA de todos os pares, corre os gestos, verifica que nada cruzou, liberta por `ordem`.
async function concorrente(pares, gestosPorPar, ordem, { pausaEntre = 500 } = {}) {
  for (const P of pares) P.holdOut = true;
  for (const [P, fn] of gestosPorPar) await fn(P);
  await wait(400);
  const antes = {};
  for (const P of pares) antes[P.nome] = (await estadoPar(P)).formas;
  SUB.deterministico.antes_de_libertar = antes;
  SUB.variavel.retidas = Object.fromEntries(pares.map((P) => [P.nome, P.qOut.length]));
  const lib = [];
  for (const nome of ordem) { lib.push(liberta(pares.find((P) => P.nome === nome), "out")); await wait(pausaEntre); }
  SUB.variavel.libertacoes = lib;
  SUB.deterministico.ordem_de_libertacao = ordem.join("→");
}

// ------------------------------------------------------------------ amostradores (no lado da página)
const pgAmostra = ({ dur, oque, alvo }) => new Promise((res) => {
  const { editor } = window.oracle;
  const out = [];
  const t0 = performance.now();
  const passo = () => {
    const t = performance.now() - t0;
    const amostra = { t: Math.round(t * 10) / 10, now: Date.now() };
    if (oque === "cursor") {
      const el = document.querySelector(".tl-collaborator__cursor");
      const m = el && /translate\(([-\d.]+)px,\s*([-\d.]+)px\)/.exec(el.style.transform);
      amostra.dom = m ? [Number(m[1]), Number(m[2])] : null;
      const c = editor.getCollaborators()[0];
      amostra.pres = c ? [c.cursor.x, c.cursor.y] : null;
    } else if (oque === "forma") {
      const s = editor.getShape(alvo);
      amostra.x = s ? s.x : null; amostra.y = s ? s.y : null;
    } else if (oque === "camara") {
      const c = editor.getCamera();
      amostra.cam = [c.x, c.y, c.z];
      amostra.seguir = editor.getInstanceState().followingUserId;
    }
    out.push(amostra);
    if (t < dur) requestAnimationFrame(passo); else res(out);
  };
  requestAnimationFrame(passo);
});

// ------------------------------------------------------------------ cenas e casos
const R = (nome, x, y, w = 160, h = 100, extra = {}) => ({ nome, type: "geo", x, y, props: { w, h, geo: "rectangle", fill: "solid", color: "black", ...(extra.props || {}) }, ...(extra.texto !== undefined ? { texto: extra.texto } : {}) });
const centro = (s) => [s.x + s.props.w / 2, s.y + s.props.h / 2];
const add = (a, b) => [a[0] + b[0], a[1] + b[1]];
const AB = [["A", "B"], ["B", "A"]];
const achaForma = (formas, id) => formas.find((f) => f.id === id);

const CASO = {};

// Quem fica por CIMA entre a forma de A e a de B: é a última libertada, ou a de maior índice
// fraccionário calculado LOCALMENTE (antes de libertar)? E A e B concordam na ordem?
function leiDoTopo(subs, formasDe) {
  let ultima = 0, indiceLocal = 0, concordam = 0, ok = 0;
  const linhas = subs.map((s) => {
    const d = s.deterministico, fs_ = formasDe(s);
    const zA = d.final.A.map((f) => f.id), zB = d.final.B.map((f) => f.id);
    const topo = zA.indexOf(fs_.A) > zA.indexOf(fs_.B) ? "A" : "B";
    const iA = d.antes_de_libertar.A.find((f) => f.id === fs_.A).index, iB = d.antes_de_libertar.B.find((f) => f.id === fs_.B).index;
    const maiorLocal = iA > iB ? "A" : "B";
    const ult = d.ordem_de_libertacao.split("→").pop();
    const indicesMantidos = d.final.servidor.find((f) => f.id === fs_.A).index === iA && d.final.servidor.find((f) => f.id === fs_.B).index === iB;
    if (topo === ult) ultima++; if (topo === maiorLocal) indiceLocal++; if (JSON.stringify(zA) === JSON.stringify(zB)) concordam++; if (indicesMantidos) ok++;
    return `${s.subcorrida}: por cima ${topo} (índices locais A=${iA} B=${iB}${indicesMantidos ? ", mantidos" : ", REESCRITOS"})`;
  });
  return `por cima = maior índice local em ${indiceLocal}/${subs.length}, = última libertada em ${ultima}/${subs.length}; A e B concordam em ${concordam}/${subs.length}; índices mantidos em ${ok}/${subs.length} — ${linhas.join(" · ")}`;
}
// Estado do servidor das formas nomeadas, depois de convergir (para casos em sequência).
async function passo(sub, rotulo, pares, sala, nomes) {
  const c = await convergencia(pares, sala);
  sub.deterministico.passos = sub.deterministico.passos || [];
  sub.deterministico.passos.push({ depois_de: rotulo, convergiu: c.convergiu, servidor: c.estado.servidor.formas.filter((f) => !nomes || nomes.includes(f.id)).map(({ clock, ...f }) => f) });
  return c;
}

CASO.c01_mesmo_campo = async () => {
  const X = R("X", 400, 250);
  const subs = [];
  for (const ordem of AB) subs.push(await subcorrida("c01", ordem.join(""), ["A", "B"], [X], async ({ A, B, g }) => {
    g("A arrasta X por (+300,+50); B arrasta X por (−200,+250); saídas retidas");
    await concorrente([A, B], [[A, (P) => arrasta(P, centro(X), add(centro(X), [300, 50]))], [B, (P) => arrasta(P, centro(X), add(centro(X), [-200, 250]))]], ordem);
  }));
  const res = subs.map((s) => { const f = achaForma(s.deterministico.final.servidor, "X"); const quem = f.x === 700 && f.y === 300 ? "A" : f.x === 200 && f.y === 500 ? "B" : `mistura (${f.x},${f.y})`; return `${s.deterministico.ordem_de_libertacao}: ganha ${quem}${s.deterministico.convergiu ? ", converge" : ", NÃO converge"}`; });
  return { existe: true, subcorridas: subs, resposta: res.join(" · ") };
};

CASO.c02_campos_diferentes_cor = async () => {
  const X = R("X", 400, 250);
  const subs = [];
  for (const ordem of AB) subs.push(await subcorrida("c02", ordem.join(""), ["A", "B"], [X], async ({ A, B, g }) => {
    await clica(B, add(centro(X), [60, 30])); await B.page.waitForTimeout(150);
    g("B selecciona X (clique, antes de reter); depois, retido: A arrasta X por (+300,+50); B clica a cor red no painel de estilo");
    await concorrente([A, B], [[A, (P) => arrasta(P, centro(X), add(centro(X), [300, 50]))], [B, (P) => mudaCor(P, "red")]], ordem);
  }));
  const res = subs.map((s) => { const f = achaForma(s.deterministico.final.servidor, "X"); return `${s.deterministico.ordem_de_libertacao}: X em (${f.x},${f.y}) cor ${f.cor}${f.x === 700 && f.cor === "red" ? " — FUNDE por campo" : " — perde uma edição"}`; });
  return { existe: true, subcorridas: subs, resposta: res.join(" · ") };
};

CASO.c03_campos_diferentes_texto = async () => {
  const X = R("X", 400, 250, 200, 120, { texto: "hello" });
  const subs = [];
  for (const ordem of AB) subs.push(await subcorrida("c03", ordem.join(""), ["A", "B"], [X], async ({ A, B, g }) => {
    g("retido: A arrasta X por (+300,+50); B duplo-clique em X, Ctrl+A, escreve «mundo novo», Escape");
    await concorrente([A, B], [[A, (P) => arrasta(P, add(centro(X), [-60, -40]), add(add(centro(X), [-60, -40]), [300, 50]))], [B, (P) => editaTexto(P, centro(X), { substitui: "mundo novo" })]], ordem);
  }));
  const res = subs.map((s) => { const f = achaForma(s.deterministico.final.servidor, "X"); return `${s.deterministico.ordem_de_libertacao}: X em (${f.x},${f.y}) texto «${f.texto}»${f.x === 700 && f.texto === "mundo novo" ? " — FUNDE por campo" : " — perde uma edição"}`; });
  return { existe: true, subcorridas: subs, resposta: res.join(" · ") };
};

CASO.c04_apagar_x_editar = async () => {
  const X = R("X", 400, 250);
  const subs = [];
  for (const ordem of AB) subs.push(await subcorrida("c04", ordem.join(""), ["A", "B"], [X], async ({ A, B, g }) => {
    g("retido: A clica X e carrega Delete; B arrasta X por (+200,+100)");
    await concorrente([A, B], [[A, async (P) => { await clica(P, centro(X)); await tecla(P, "Delete"); }], [B, (P) => arrasta(P, centro(X), add(centro(X), [200, 100]))]], ordem);
  }));
  const res = subs.map((s) => { const f = achaForma(s.deterministico.final.servidor, "X"); return `${s.deterministico.ordem_de_libertacao}: ${f ? `X VIVO em (${f.x},${f.y})` : "X apagado"}${s.deterministico.convergiu ? "" : " (NÃO converge)"}`; });
  return { existe: true, subcorridas: subs, resposta: res.join(" · ") };
};

CASO.c05_criar_no_mesmo_z = async () => {
  const subs = [];
  for (const ordem of AB) for (const k of [1, 2, 3]) subs.push(await subcorrida("c05", `${ordem.join("")}-s${k}`, ["A", "B"], [], async ({ A, B, g, sub }) => {
    g("retido: A cria um rectângulo (tecla r, arrasto (300,300)→(460,400)); B cria outro ((350,330)→(510,430))");
    const criados = {};
    await concorrente([A, B], [[A, async (P) => { criados.A = await criaRect(P, [300, 300], [460, 400]); }], [B, async (P) => { criados.B = await criaRect(P, [350, 330], [510, 430]); }]], ordem);
    sub.deterministico.criados = criados;
  }));
  const lei = leiDoTopo(subs, (s) => ({ A: s.deterministico.criados.A[0], B: s.deterministico.criados.B[0] }));
  return { existe: true, subcorridas: subs, resposta: lei };
};

CASO.c06_trazer_para_a_frente = async () => {
  const cena = [R("X", 300, 250), R("Y", 340, 280), R("Z", 380, 310)];
  const subs = [];
  for (const ordem of AB) for (const k of [1, 2, 3]) subs.push(await subcorrida("c06", `${ordem.join("")}-s${k}`, ["A", "B"], cena, async ({ A, B, g }) => {
    await clica(A, [310, 260]); await clica(B, [350, 290]); await A.page.waitForTimeout(150);
    g("A selecciona X, B selecciona Y (cliques, antes de reter); retido: A e B carregam ] (Bring to front, do menu Reorder)");
    await concorrente([A, B], [[A, (P) => tecla(P, "]")], [B, (P) => tecla(P, "]")]], ordem);
  }));
  const lei = leiDoTopo(subs, () => ({ A: "X", B: "Y" }));
  return { existe: true, subcorridas: subs, resposta: lei };
};

const handleFim = async (P, setaId) => P.page.evaluate((id) => {
  const { editor } = window.oracle;
  const h = editor.getShapeHandles(id).find((h) => h.id === "end");
  const m = editor.getShapePageTransform(id);
  const p = m.applyToPoint(h);
  return [p.x, p.y];
}, setaId);

CASO.c07_seta_e_forma_apagada = async () => {
  const cena = [R("P", 150, 300, 120, 100), R("Q", 600, 300, 120, 100)];
  const subs = [];
  for (const ordem of AB) subs.push(await subcorrida("c07", ordem.join(""), ["A", "B"], cena, async ({ A, B, g, sub }) => {
    const [seta] = await criaSeta(A, [210, 350], [660, 350]);
    const setaId = [...ROTULOS].find(([, v]) => v === seta)[0];
    const c = await convergencia([A, B], SUB.sala);
    sub.deterministico.seta_antes = { convergiu: c.convergiu, bindings: c.estado.servidor.bindings.map(({ clock, ...b }) => b) };
    await clica(A, [435, 352]); await A.page.waitForTimeout(150);
    const hf = await handleFim(A, setaId);
    sub.deterministico.ponta = hf.map(r3);
    g(`A cria a seta ${seta} de P a Q (tecla a, arrasto), converge; A selecciona a seta; retido: B clica Q e carrega Delete; A arrasta a ponta da seta (${hf.map(Math.round)}) para (690,380), dentro de Q`);
    await concorrente([A, B], [[A, (P) => arrasta(P, hf, [690, 380])], [B, async (P) => { await clica(P, [705, 390]); await tecla(P, "Delete"); }]], ordem);
  }));
  const res = subs.map((s) => { const f = s.deterministico.final; const seta = f.servidor.find((x) => x.type === "arrow"); return `${s.deterministico.ordem_de_libertacao}: Q ${achaForma(f.servidor, "Q") ? "vivo" : "apagado"}; seta ${seta ? `existe, bindings=${JSON.stringify(f.bindings_servidor.map((b) => b.terminal + "→" + b.para))}` : "DESAPARECEU"}`; });
  return { existe: true, subcorridas: subs, resposta: res.join(" · ") };
};

CASO.c08_seta_segue_forma_alheia = async () => {
  const cena = [R("P", 150, 300, 120, 100), R("Q", 600, 300, 120, 100)];
  const s = await subcorrida("c08", "seq", ["A", "B"], cena, async ({ A, B, g, sub, sala }) => {
    g("A cria a seta P→Q (tecla a, arrasto); converge");
    const [seta] = await criaSeta(A, [210, 350], [660, 350]);
    const c0 = await convergencia([A, B], sala);
    const caixa0 = (await estadoPar(A)).caixas[seta];
    const clk0 = Object.fromEntries(c0.estado.servidor.formas.map((f) => [f.id, f.clock]));
    const seta0 = c0.estado.servidor.formas.find((f) => f.id === seta);
    const nMsg0 = MSGS.length;
    g("B arrasta Q por (0,+200)");
    await arrasta(B, [700, 390], [700, 590]);
    const c1 = await convergencia([A, B], sala);
    const caixa1A = (await estadoPar(A)).caixas[seta];
    const caixa1B = (await estadoPar(B)).caixas[seta];
    const clk1 = Object.fromEntries(c1.estado.servidor.formas.map((f) => [f.id, f.clock]));
    const seta1 = c1.estado.servidor.formas.find((f) => f.id === seta);
    const quemEmpurraSeta = MSGS.slice(nMsg0).filter((e) => e.tipo === "push" && e.diff.some((k) => k.startsWith(seta))).map((e) => e.par);
    sub.variavel.clocks = { seta_antes: clk0[seta], seta_depois: clk1[seta], Q_antes: clk0.Q, Q_depois: clk1.Q };
    sub.deterministico.seta = {
      caixa_antes: caixa0, caixa_depois_em_A: caixa1A, caixa_depois_em_B: caixa1B,
      segue_em_A: JSON.stringify(caixa0) !== JSON.stringify(caixa1A),
      registo_da_seta_mudou: JSON.stringify(seta0) !== JSON.stringify(seta1) && JSON.stringify({ ...seta0, clock: 0 }) !== JSON.stringify({ ...seta1, clock: 0 }),
      clock_da_seta_inalterado: clk0[seta] === clk1[seta], clock_de_Q_sobe: clk1.Q > clk0.Q,
      pares_que_empurraram_a_seta_depois_do_move: [...new Set(quemEmpurraSeta)],
    };
  });
  const d = s.deterministico.seta;
  return { existe: true, subcorridas: [s], resposta: `em A a seta ${d.segue_em_A ? "SEGUE Q" : "NÃO segue"}; registo da seta ${d.registo_da_seta_mudou ? "muda" : "NÃO muda"} (clock da seta ${d.clock_da_seta_inalterado ? "parado" : "muda"}, o de Q ${d.clock_de_Q_sobe ? "sobe" : "parado"}); push da seta por: ${JSON.stringify(d.pares_que_empurraram_a_seta_depois_do_move)}` };
};

CASO.c09_texto_a_dois = async () => {
  const T = { nome: "T", type: "text", x: 400, y: 300, props: {}, texto: "hello" };
  const subs = [];
  for (const ordem of AB) subs.push(await subcorrida("c09", ordem.join(""), ["A", "B"], [T], async ({ A, B, g }) => {
    g("retido: A duplo-clique em T, seta para a direita (fim), escreve «aaa», Escape; B o mesmo com «bbb»");
    await concorrente([A, B], [[A, (P) => editaTexto(P, [415, 315], { acrescenta: "aaa" })], [B, (P) => editaTexto(P, [415, 315], { acrescenta: "bbb" })]], ordem);
  }));
  const res = subs.map((s) => `${s.deterministico.ordem_de_libertacao}: «${achaForma(s.deterministico.final.servidor, "T").texto}»`);
  return { existe: true, subcorridas: subs, resposta: res.join(" · ") };
};

CASO.c10_desfazer_so_o_meu = async () => {
  const s = await subcorrida("c10", "seq", ["A", "B"], [], async ({ A, B, g, sub, sala }) => {
    g("A cria X (tecla r, arrasto)"); const [x] = await criaRect(A, [300, 300], [460, 400]); await passo(sub, "A cria X", [A, B], sala);
    g("B cria Y"); const [y] = await criaRect(B, [600, 300], [760, 400]); await passo(sub, "B cria Y", [A, B], sala);
    g("A carrega Ctrl+Z"); await tecla(A, "Control+z");
    sub.deterministico.rotulos = { X: x, Y: y };
  });
  const f = s.deterministico.final.servidor.map((x) => x.id);
  const { X, Y } = s.deterministico.rotulos;
  return { existe: true, subcorridas: [s], resposta: `depois do Ctrl+Z de A: ${X} (de A) ${f.includes(X) ? "fica" : "desfeito"}, ${Y} (de B) ${f.includes(Y) ? "fica" : "DESFEITO"} — desfazer ${!f.includes(X) && f.includes(Y) ? "é por par" : "NÃO é por par"}` };
};

CASO.c11_desfazer_depois_do_outro = async () => {
  const X = R("X", 400, 250);
  const s = await subcorrida("c11", "seq", ["A", "B"], [X], async ({ A, B, g, sub, sala }) => {
    g("A arrasta X por (+200,0)"); await arrasta(A, centro(X), add(centro(X), [200, 0])); await passo(sub, "A move X", [A, B], sala);
    g("B clica X e escolhe a cor red"); await clica(B, add(centro(X), [200, 0])); await mudaCor(B, "red"); await passo(sub, "B pinta X de red", [A, B], sala);
    g("A carrega Ctrl+Z"); await tecla(A, "Control+z");
    const c1 = await convergencia([A, B], sala);
    sub.deterministico.depois_ctrl_z = { convergiu: c1.convergiu, X: (({ clock, ...f }) => f)(achaForma(c1.estado.servidor.formas, "X")) };
    g("A carrega Ctrl+Shift+Z"); await tecla(A, "Control+Shift+z");
  });
  const u = s.deterministico.depois_ctrl_z.X, r = achaForma(s.deterministico.final.servidor, "X");
  return { existe: true, subcorridas: [s], resposta: `Ctrl+Z de A: X em (${u.x},${u.y}) cor ${u.cor}; Ctrl+Shift+Z: X em (${r.x},${r.y}) cor ${r.cor}` };
};

CASO.c12_desfazer_o_que_o_outro_apagou = async () => {
  const X = R("X", 400, 250);
  const s = await subcorrida("c12", "seq", ["A", "B"], [X], async ({ A, B, g, sub, sala }) => {
    g("A arrasta X por (+200,0)"); await arrasta(A, centro(X), add(centro(X), [200, 0])); await passo(sub, "A move X", [A, B], sala);
    g("B clica X e carrega Delete"); await clica(B, add(centro(X), [200, 0])); await tecla(B, "Delete"); await passo(sub, "B apaga X", [A, B], sala);
    g("A carrega Ctrl+Z"); await tecla(A, "Control+z");
  });
  const f = achaForma(s.deterministico.final.servidor, "X");
  return { existe: true, subcorridas: [s], resposta: f ? `X RESSUSCITA em (${f.x},${f.y})` : "X continua apagado (o Ctrl+Z de A não o traz)" };
};

CASO.c13_entrar_a_meio = async () => {
  const s = await subcorrida("c13", "seq", ["A"], [], async ({ A, g, sub, sala }) => {
    g("A desenha 50 rectângulos (tecla r + arrasto cada), grelha 10×5");
    for (let i = 0; i < 50; i++) { const x = 150 + (i % 10) * 80, y = 150 + Math.floor(i / 10) * 70; await criaRect(A, [x, y], [x + 60, y + 40]); }
    const cA = await convergencia([A], sala);
    const nA = cA.estado.A.formas.length;
    g("B entra na sala (página nova)");
    const n0 = MSGS.length;
    const tEntra = Date.now();
    const B = await par("B", sala, "c13/seq");
    const tSync = Date.now() - tEntra;
    let tIgual = null;
    const alvo = chaveDoc(cA.estado.A);
    for (let k = 0; k < 400; k++) { const e = await estadoPar(B); if (chaveDoc(e) === alvo) { tIgual = Date.now() - tEntra; break; } await wait(25); }
    await wait(1500);
    const mB = MSGS.slice(n0).filter((e) => e.par === "B");
    const ate = mB.filter((e) => e.t <= tIgual + (tEntra - T0));
    const soma = (xs) => xs.reduce((a, e) => a + e.bytes, 0);
    sub.deterministico.entrada = { formas_de_A: nA, B_ve_tudo: tIgual !== null };
    sub.variavel.entrada = {
      ms_ate_synced_remote: tSync, ms_ate_B_igual_a_A: tIgual,
      ate_igual: { mensagens: ate.length, bytes_recebidos: soma(ate.filter((e) => e.dir === "S→B")), bytes_enviados: soma(ate.filter((e) => e.dir === "B→S")) },
      primeira_resposta_connect_bytes: (mB.find((e) => e.dir === "S→B" && e.tipo === "connect") || {}).bytes,
      ate_1_5s_depois: { mensagens: mB.length, bytes: soma(mB) },
    };
  });
  const e = s.variavel.entrada;
  return { existe: true, subcorridas: [s], resposta: `B vê ${s.deterministico.entrada.B_ve_tudo ? "tudo" : "NÃO tudo"} (${s.deterministico.entrada.formas_de_A} formas); entrada: ${e.ate_igual.mensagens} mensagens, ${e.ate_igual.bytes_recebidos} B recebidos (connect ${e.primeira_resposta_connect_bytes} B), ${e.ate_igual.bytes_enviados} B enviados; igual em ${e.ms_ate_B_igual_a_A} ms` };
};

CASO.c14_sair_e_voltar = async () => {
  const X = R("X", 400, 250);
  const s = await subcorrida("c14", "seq", ["A", "B"], [X], async ({ A, B, g, sub, sala }) => {
    SUB.prazoConvergencia = 20000;
    g("B fica sem rede (context.setOffline(true))"); await B.ctx.setOffline(true); await wait(800);
    g("A arrasta X por (+200,0) e cria W"); await arrasta(A, centro(X), add(centro(X), [200, 0]));
    const [w] = await criaRect(A, [300, 500], [420, 580]);
    g("B (sem rede) clica X (onde o vê) e escolhe red; cria V"); await clica(B, add(centro(X), [60, 30])); await mudaCor(B, "red");
    const [v] = await criaRect(B, [700, 500], [820, 580]);
    await wait(600);
    sub.deterministico.durante = { A: (await estadoPar(A)).formas, B: (await estadoPar(B)).formas, B_estado: await B.page.evaluate(() => [window.oracle.status, window.oracle.connectionStatus]) };
    sub.deterministico.rotulos = { W: w, V: v };
    g("B volta (setOffline(false))"); await B.ctx.setOffline(false);
  });
  const f = s.deterministico.final.servidor, X1 = achaForma(f, "X");
  const { W, V } = s.deterministico.rotulos;
  return { existe: true, subcorridas: [s], resposta: `${s.deterministico.convergiu ? "converge" : "NÃO converge"}: X em (${X1?.x},${X1?.y}) cor ${X1?.cor}; W(A) ${achaForma(f, W) ? "fica" : "PERDIDO"}; V(B offline) ${achaForma(f, V) ? "fica" : "PERDIDO"}` };
};

CASO.c15_rede_cai_a_meio_do_arrasto = async () => {
  const X = R("X", 400, 250);
  const s = await subcorrida("c15", "seq", ["A", "B"], [X], async ({ A, B, g, sub }) => {
    SUB.prazoConvergencia = 20000;
    g("A agarra X e arrasta metade (+150,0)"); await arrasta(A, centro(X), add(centro(X), [150, 0]), { soltar: false, passos: 8 });
    await wait(300);
    sub.deterministico.B_a_meio = achaForma((await estadoPar(B)).formas, "X");
    g("a rede de A cai (setOffline(true)); A continua até (+300,+100) e larga"); await A.ctx.setOffline(true);
    await arrasta(A, add(centro(X), [150, 0]), add(centro(X), [300, 100]), { agarra: false, passos: 8 });
    await wait(800);
    sub.deterministico.durante = { A: achaForma((await estadoPar(A)).formas, "X"), B: achaForma((await estadoPar(B)).formas, "X") };
    g("a rede volta (setOffline(false))"); await A.ctx.setOffline(false);
  });
  const f = achaForma(s.deterministico.final.B, "X");
  return { existe: true, subcorridas: [s], resposta: `B acaba com X em (${f.x},${f.y}) — ${f.x === 700 && f.y === 350 ? "a posição FINAL de A" : "NÃO a posição final de A"}${s.deterministico.convergiu ? "; converge" : "; NÃO converge"} (a meio B via ${s.deterministico.B_a_meio?.x})` };
};

CASO.c16_mensagem_duplicada = async () => {
  const X = R("X", 400, 250);
  const subs = [];
  for (const variante of ["S_para_B", "A_para_S"]) subs.push(await subcorrida("c16", variante, ["A", "B"], [X], async ({ A, B, g, sub }) => {
    if (variante === "S_para_B") B.holdIn = true; else A.holdOut = true;
    g(`${variante === "S_para_B" ? "retida a ENTRADA de B" : "retida a SAÍDA de A"}; A arrasta X por (+200,0) e cria W`);
    await arrasta(A, centro(X), add(centro(X), [200, 0]));
    const [w] = await criaRect(A, [300, 500], [420, 580]);
    await wait(400);
    g("liberta com cada mensagem com documento DUPLICADA");
    sub.variavel.libertacao = variante === "S_para_B" ? liberta(B, "in", { duplica: true }) : liberta(A, "out", { duplica: true });
    sub.deterministico.rotulos = { W: w };
    await wait(800);
  }));
  const res = subs.map((s) => { const f = s.deterministico.final; return `${s.subcorrida}: ${s.deterministico.convergiu ? "converge" : "NÃO converge"}, formas=${f.servidor.length}, B=${f.B.length}, reconexões A=${s.variavel.conexoes.A.conexoes - 1} B=${s.variavel.conexoes.B.conexoes - 1}`; });
  return { existe: true, subcorridas: subs, resposta: res.join(" · ") };
};

CASO.c17_mensagens_reordenadas = async () => {
  const X = R("X", 400, 250);
  const subs = [];
  for (const variante of ["S_para_B", "A_para_S"]) subs.push(await subcorrida("c17", variante, ["A", "B"], [X], async ({ A, B, g, sub }) => {
    if (variante === "S_para_B") B.holdIn = true; else A.holdOut = true;
    g(`${variante === "S_para_B" ? "retida a ENTRADA de B" : "retida a SAÍDA de A"}; A arrasta X por (+150,0), espera 400 ms, arrasta outra vez (+150,0)`);
    await arrasta(A, centro(X), add(centro(X), [150, 0]), { passos: 2 }); await wait(400);
    await arrasta(A, add(centro(X), [150, 0]), add(centro(X), [300, 0]), { passos: 2 }); await wait(400);
    g("liberta com as mensagens com documento em ordem INVERSA");
    sub.variavel.libertacao = variante === "S_para_B" ? liberta(B, "in", { ordem: "reverso_doc" }) : liberta(A, "out", { ordem: "reverso_doc" });
    await wait(300);
    sub.variavel.B_logo_depois = achaForma((await estadoPar(B)).formas, "X");
    const c = await convergencia([A, B], SUB.sala, 15000);
    sub.deterministico.converge_sozinho_em_15s = c.convergiu;
    sub.variavel.B_15s_depois = achaForma(c.estado.B.formas, "X");
    sub.deterministico.B_15s_depois_na_posicao_final_de_A = sub.variavel.B_15s_depois?.x === 700;
    if (!c.convergiu) {
      g("não convergiu em 15 s: A faz uma edição NOVA em X (seta para a direita, X seleccionado) para ver se B se cura");
      await A.page.mouse.click(1000, 650); await clica(A, add(centro(X), [300, 0])); await tecla(A, "ArrowRight");
    }
  }));
  const res = subs.map((s) => { const b = s.deterministico.B_logo_depois, f = achaForma(s.deterministico.final.B, "X"); return `${s.subcorrida}: B fica em x=${s.variavel.B_15s_depois?.x} (A acabou em 700; meio 550)${s.deterministico.converge_sozinho_em_15s ? ", converge" : `, NÃO converge em 15 s; depois de uma edição nova de A: B x=${f?.x}, ${s.deterministico.convergiu ? "converge" : "continua divergente"}`}`; });
  return { existe: true, subcorridas: subs, resposta: res.join(" · ") };
};

CASO.c18_cursor = async () => {
  const s = await subcorrida("c18", "seq", ["A", "B"], [], async ({ A, B, g, sub }) => {
    await A.page.mouse.move(790, 400); await wait(500);
    g("A move o rato em círculo (centro 640,400, raio 150, 2 s, um passo a cada ~16 ms), sem arrastar");
    const amostra = B.page.evaluate(pgAmostra, { dur: 2600, oque: "cursor" });
    const tIni = Date.now() - T0;
    const t0 = Date.now();
    let passos = 0;
    while (Date.now() - t0 < 2000) { const a = ((Date.now() - t0) / 2000) * 2 * Math.PI; await A.page.mouse.move(640 + 150 * Math.cos(a), 400 + 150 * Math.sin(a)); passos++; await wait(16); }
    const tFim = Date.now() - T0;
    const am = await amostra;
    const out = MSGS.filter((e) => e.dir === "A→S" && e.t >= tIni && e.t <= tFim);
    const inn = MSGS.filter((e) => e.dir === "S→B" && e.t >= tIni && e.t <= tFim + 300);
    const dom = am.filter((x) => x.dom).map((x) => x.dom);
    const distintas = dom.filter((p, i) => i === 0 || p[0] !== dom[i - 1][0] || p[1] !== dom[i - 1][1]).length;
    const saltos = dom.slice(1).map((p, i) => Math.hypot(p[0] - dom[i][0], p[1] - dom[i][1])).filter((d) => d > 0);
    const nome = await B.page.evaluate(() => { const c = window.oracle.editor.getCollaborators()[0]; const el = document.querySelector(".tl-collaborator__cursor .tl-nametag"); return { presenca: c && { userName: c.userName, color: c.color }, nametag: el && { texto: el.textContent, fundo: el.style.backgroundColor }, cor_do_cursor: document.querySelector(".tl-collaborator__cursor use")?.getAttribute("color") }; });
    sub.deterministico.B_ve = nome;
    sub.variavel.cursor = {
      passos_do_rato_em_A: passos, duracao_ms: tFim - tIni,
      A_envia: { mensagens: out.length, por_s: r3(out.length / ((tFim - tIni) / 1000)), bytes_por_mensagem: out.length ? r3(out.reduce((a, e) => a + e.bytes, 0) / out.length) : 0, bytes_por_s: r3(out.reduce((a, e) => a + e.bytes, 0) / ((tFim - tIni) / 1000)), com_presenca: out.filter((e) => e.presenca).length },
      B_recebe: { mensagens: inn.length, bytes_por_mensagem: inn.length ? r3(inn.reduce((a, e) => a + e.bytes, 0) / inn.length) : 0 },
      amostras_B_rAF: am.length, hz_de_amostragem: r3(am.length / 2.6), posicoes_distintas_no_DOM: distintas, posicoes_distintas_por_s: r3(distintas / 2.6),
      salto_px: saltos.length ? { mediana: r3(saltos.sort((a, b) => a - b)[Math.floor(saltos.length / 2)]), max: r3(Math.max(...saltos)) } : null,
      intervalo_entre_mudancas_ms: (() => { const ts = am.filter((x, i) => x.dom && i > 0 && am[i - 1].dom && (x.dom[0] !== am[i - 1].dom[0] || x.dom[1] !== am[i - 1].dom[1])).map((x) => x.t); const d = ts.slice(1).map((t, i) => t - ts[i]).sort((a, b) => a - b); return d.length ? { mediana: r3(d[Math.floor(d.length / 2)]), max: r3(d[d.length - 1]) } : null; })(),
      amostras: am.map((x) => [x.t, x.dom, x.pres]),
    };
  });
  const c = s.variavel.cursor;
  return { existe: true, subcorridas: [s], resposta: `A envia ${c.A_envia.por_s} msg/s, ${c.A_envia.bytes_por_mensagem} B/msg; B redesenha ${c.posicoes_distintas_por_s} pos/s (amostra ${c.hz_de_amostragem} Hz, salto mediano ${c.salto_px?.mediana} px); B vê nome «${s.deterministico.B_ve.nametag?.texto}» cor ${s.deterministico.B_ve.cor_do_cursor}` };
};

CASO.c19_arrastar_custo = async () => {
  const subs = [];
  for (const N of [1, 10, 50]) {
    const cena = Array.from({ length: N }, (_, i) => R(`S${i}`, 150 + (i % 10) * 70, 200 + Math.floor(i / 10) * 60, 50, 40));
    subs.push(await subcorrida("c19", `N${N}`, ["A", "B"], cena, async ({ A, B, g, sub }) => {
      await A.page.mouse.click(1000, 650); await tecla(A, "Control+a"); await wait(300);
      const nSel = await A.page.evaluate(() => window.oracle.editor.getSelectedShapeIds().length);
      g(`A selecciona tudo (Ctrl+A: ${nSel} formas) e arrasta S0 durante 2 s, 400 px para a direita`);
      const amostra = B.page.evaluate(pgAmostra, { dur: 2800, oque: "forma", alvo: "shape:S0" });
      const de = [175, 220];
      const a = await scr(A, de);
      await A.page.mouse.move(a[0], a[1]); await A.page.mouse.down();
      const tIni = Date.now() - T0, t0 = Date.now();
      let passos = 0;
      while (Date.now() - t0 < 2000) { const f = (Date.now() - t0) / 2000; await A.page.mouse.move(a[0] + 400 * f, a[1] + 40 * Math.sin(f * Math.PI)); passos++; await wait(16); }
      await A.page.mouse.move(a[0] + 400, a[1]); await wait(16);
      await A.page.mouse.up();
      const tFim = Date.now() - T0;
      const am = await amostra;
      const out = MSGS.filter((e) => e.dir === "A→S" && e.t >= tIni && e.t <= tFim);
      const xs = am.filter((x) => x.x !== null).map((x) => x.x);
      const distintas = xs.filter((x, i) => i === 0 || x !== xs[i - 1]).length;
      sub.deterministico.selecionadas = nSel;
      sub.variavel.arrasto = {
        passos_do_rato: passos, duracao_ms: tFim - tIni,
        A_envia: { mensagens: out.length, por_s: r3(out.length / ((tFim - tIni) / 1000)), bytes: out.reduce((a, e) => a + e.bytes, 0), bytes_por_s: r3(out.reduce((a, e) => a + e.bytes, 0) / ((tFim - tIni) / 1000)), bytes_por_mensagem: out.length ? r3(out.reduce((a, e) => a + e.bytes, 0) / out.length) : 0, com_documento: out.filter(ehDocMsg).length },
        B_ve: { amostras: am.length, posicoes_x_distintas_de_S0: distintas, primeira: xs[0], ultima: xs[xs.length - 1], caminho: distintas > 3 ? "o CAMINHO" : "só o fim" },
        amostras: am.map((x) => [x.t, x.x, x.y]),
      };
    }));
  }
  const res = subs.map((s) => { const a = s.variavel.arrasto; return `N=${s.deterministico.selecionadas}: ${a.A_envia.por_s} msg/s, ${a.A_envia.bytes_por_s} B/s (${a.A_envia.bytes_por_mensagem} B/msg); B vê ${a.B_ve.caminho} (${a.B_ve.posicoes_x_distintas_de_S0} posições)`; });
  return { existe: true, subcorridas: subs, resposta: res.join(" · ") };
};

CASO.c20_seleccao_alheia = async () => {
  const X = R("X", 400, 250);
  const s = await subcorrida("c20", "seq", ["A", "B"], [X], async ({ A, B, g, sub }) => {
    g("A clica X (selecciona) e leva o rato para fora do recorte"); await clica(A, centro(X)); await A.page.mouse.move(1000, 650); await wait(800);
    sub.deterministico.B_ve = await B.page.evaluate(() => {
      const { editor } = window.oracle;
      const c = editor.getCollaborators()[0];
      const indicadores = [...document.querySelectorAll("[class*=collaborator]")].filter((e) => !e.classList.contains("tl-collaborators")).map((e) => ({ tag: e.tagName, classe: e.getAttribute("class"), stroke: e.getAttribute("stroke") || getComputedStyle(e).stroke, cor: getComputedStyle(e).color }));
      const comCor = [...document.querySelectorAll("svg *")].filter((e) => { const st = e.getAttribute("stroke") || ""; const cs = getComputedStyle(e).stroke; return /e03131|224, 49, 49/i.test(st + cs); }).map((e) => ({ tag: e.tagName, classe: e.getAttribute("class") || e.parentElement?.getAttribute("class") }));
      return { presenca: c && { userName: c.userName, color: c.color, selectedShapeIds: c.selectedShapeIds }, elementos_collaborator: indicadores, elementos_com_a_cor_de_A: comCor.slice(0, 10) };
    });
    // Prova por PIXEL: o recorte de X em B com a selecção de A contra o mesmo recorte depois de A a largar
    // (controlo); conta os píxeis que mudam e dá a cor média deles.
    const clip = { x: 330, y: 180, width: 340, height: 240 };
    const comPng = await B.page.screenshot({ clip });
    fs.writeFileSync(path.join(outDir, "c20_B.png"), comPng);
    g("controlo: A clica no vazio e carrega Escape (larga a selecção)"); await A.page.mouse.click(1000, 650); await tecla(A, "Escape"); await wait(800);
    const semPng = await B.page.screenshot({ clip });
    sub.deterministico.pixeis_que_mudam_no_recorte = await B.page.evaluate(async ([a, b]) => {
      const dec = async (b64) => { const img = await createImageBitmap(await (await fetch("data:image/png;base64," + b64)).blob()); const c = new OffscreenCanvas(img.width, img.height); const x = c.getContext("2d"); x.drawImage(img, 0, 0); return x.getImageData(0, 0, img.width, img.height).data; };
      const da = await dec(a), db = await dec(b);
      let n = 0, r = 0, g = 0, bl = 0;
      for (let i = 0; i < da.length; i += 4) if (Math.abs(da[i] - db[i]) + Math.abs(da[i + 1] - db[i + 1]) + Math.abs(da[i + 2] - db[i + 2]) > 60) { n++; r += da[i]; g += da[i + 1]; bl += da[i + 2]; }
      return { pixeis: n, cor_media_com_seleccao: n ? [Math.round(r / n), Math.round(g / n), Math.round(bl / n)] : null };
    }, [comPng.toString("base64"), semPng.toString("base64")]);
  });
  const v = s.deterministico.B_ve;
  const px = s.deterministico.pixeis_que_mudam_no_recorte;
  return { existe: true, subcorridas: [s], resposta: `B ${v.presenca?.selectedShapeIds?.length ? "VÊ" : "não vê"} a selecção de A (presença selectedShapeIds=${JSON.stringify(v.presenca?.selectedShapeIds)}); contorno desenhado: ${px.pixeis} píxeis mudam quando A larga, cor média rgb(${px.cor_media_com_seleccao}) (A = ${v.presenca?.color}) — ver c20_B.png; nome «${v.presenca?.userName}» só na presença/cursor` };
};

// Abre o menu de pessoas de P e devolve os botões da linha de `alvo`.
async function menuPessoas(P) { await P.page.click(".tlui-people-menu__avatars-button"); await P.page.waitForTimeout(400); }
async function seguir(P, alvoNome) {
  // o botão da linha ALTERNA (medido): só se carrega se ainda não se segue alguém
  if (await seguindo(P)) return "já seguia";
  await menuPessoas(P);
  const linhas = await P.page.$$(".tlui-people-menu__item");
  for (const l of linhas) {
    if ((await l.innerText()).trim() === alvoNome) {
      await l.hover(); await P.page.waitForTimeout(200);
      const bt = await l.$("button[title], .tlui-people-menu__item__follow, button:last-child");
      const botoes = await l.$$("button");
      const info = [];
      for (const b of botoes) info.push({ texto: (await b.innerText()).trim(), titulo: await b.getAttribute("title"), aria: await b.getAttribute("aria-label"), classe: (await b.getAttribute("class"))?.slice(0, 80), testid: await b.getAttribute("data-testid") });
      const follow = botoes[botoes.length - 1];
      await follow.click(); await P.page.waitForTimeout(300);
      await P.page.keyboard.press("Escape"); await P.page.waitForTimeout(150);
      return info;
    }
  }
  return null;
}
const seguindo = (P) => P.page.evaluate(() => window.oracle.editor.getInstanceState().followingUserId);

CASO.c21_seguir = async () => {
  const s = await subcorrida("c21", "seq", ["A", "B"], [R("X", 400, 250)], async ({ A, B, g, sub }) => {
    g("B abre o menu de pessoas (avatares no canto superior direito) e carrega o botão de seguir da linha de A");
    const botoes = await seguir(B, "A");
    // o followingUserId guarda o userId da PRESENÇA ("user:…"), não o editor.user.getId() (sem prefixo) — medido
    const idA = await B.page.evaluate(() => window.oracle.editor.getCollaborators().find((c) => c.userName === "A")?.userId);
    const f0 = await seguindo(B);
    sub.deterministico.seguir = { botoes_da_linha: botoes, B_segue: f0 === idA, followingUserId_e_A: f0 === idA };
    g("A faz pan (roda do rato, 6 × (dx 80, dy 50)) e zoom (Ctrl + roda, 3 × −100)");
    const amB = B.page.evaluate(pgAmostra, { dur: 3500, oque: "camara" });
    const amA = A.page.evaluate(pgAmostra, { dur: 3500, oque: "camara" });
    await A.page.mouse.move(640, 400);
    for (let i = 0; i < 6; i++) { await A.page.mouse.wheel(80, 50); await wait(60); }
    await wait(400);
    await A.page.keyboard.down("Control");
    for (let i = 0; i < 3; i++) { await A.page.mouse.wheel(0, -100); await wait(80); }
    await A.page.keyboard.up("Control");
    const [a, b] = await Promise.all([amA, amB]);
    const camA = a[a.length - 1].cam, camB = b[b.length - 1].cam;
    const quando = (am, cam) => { const x = am.find((s) => Math.abs(s.cam[0] - cam[0]) < 0.5 && Math.abs(s.cam[1] - cam[1]) < 0.5 && Math.abs(s.cam[2] - cam[2]) < 1e-3); return x ? x.now : null; };
    const tA = quando(a, camA), tB = quando(b, camA);
    const mudancasB = b.filter((s, i) => i > 0 && JSON.stringify(s.cam) !== JSON.stringify(b[i - 1].cam)).length;
    sub.deterministico.camara = { A_final: camA.map(r3), B_final: camB.map(r3), B_alcanca_A: tB !== null };
    sub.variavel.camara = { atraso_ms_B_alcanca_a_camara_final_de_A: tA && tB ? tB - tA : null, mudancas_de_camara_em_B: mudancasB, amostras_A: a.length, amostras_B: b.length, serie_A: a.map((s) => [s.now - a[0].now, ...s.cam.map(r3)]), serie_B: b.map((s) => [s.now - a[0].now, ...s.cam.map(r3), s.seguir ? 1 : 0]) };
    // o que interrompe o seguir
    const interrupcoes = {};
    const testes = [
      ["B roda o rato (pan)", async () => { await B.page.mouse.move(640, 400); await B.page.mouse.wheel(0, 120); }],
      ["B clica no canvas vazio", async () => { await B.page.mouse.click(900, 600); }],
      ["B carrega Escape", async () => { await B.page.mouse.move(640, 400); await tecla(B, "Escape"); }],
      ["B escolhe uma ferramenta (tecla r)", async () => { await B.page.mouse.move(640, 400); await tecla(B, "r"); await tecla(B, "v"); }],
      ["B arrasta X (que vê)", async () => { const c = await B.page.evaluate(() => { const b = window.oracle.editor.getShapePageBounds("shape:X"); return [b.x + b.w / 2, b.y + b.h / 2]; }); await arrasta(B, c, add(c, [30, 0]), { passos: 3 }); }],
      ["B faz zoom (Ctrl + roda)", async () => { await B.page.mouse.move(640, 400); await B.page.keyboard.down("Control"); await B.page.mouse.wheel(0, -100); await B.page.keyboard.up("Control"); }],
    ];
    for (const [nome, fn] of testes) {
      await seguir(B, "A"); await wait(200);
      const antes = (await seguindo(B)) === idA;
      await fn(); await wait(400);
      const depois = (await seguindo(B)) === idA;
      interrupcoes[nome] = { seguia_antes: antes, segue_depois: depois, interrompe: antes && !depois };
    }
    sub.deterministico.interrupcoes = interrupcoes;
  });
  const d = s.deterministico, v = s.variavel.camara;
  return { existe: true, subcorridas: [s], resposta: `B ${d.seguir.B_segue ? "segue" : "NÃO segue"} A; câmara de B ${d.camara.B_alcanca_A ? "alcança" : "NÃO alcança"} a de A (atraso ${v.atraso_ms_B_alcanca_a_camara_final_de_A} ms); interrompem: ${Object.entries(d.interrupcoes).filter(([, x]) => x.interrompe).map(([k]) => k).join("; ") || "nada"}; não interrompem: ${Object.entries(d.interrupcoes).filter(([, x]) => x.seguia_antes && !x.interrompe).map(([k]) => k).join("; ")}` };
};

CASO.c22_trazer_todos = async () => {
  const PADRAO = /bring (every|all)|summon|everyone|gather|all users|to me|follow me|ask .*follow|come here/i;
  const s = await subcorrida("c22", "seq", ["A", "B", "C"], [R("X", 400, 250)], async ({ A, g, sub }) => {
    const visiveis = () => A.page.evaluate(() => [...document.querySelectorAll('[role=menuitem],[role=menuitemcheckbox],[role=menuitemradio],button')].filter((e) => { const r = e.getBoundingClientRect(); return r.width > 0 && e.closest('[role=menu],[data-radix-popper-content-wrapper],.tlui-popover__content,[role=dialog],.tlui-people-menu__wrapper'); }).map((e) => `${e.dataset.testid || ""}|${e.textContent.trim().slice(0, 50)}|${e.getAttribute("title") || ""}`));
    const procurados = {};
    g("A abre cada menu e submenu e lista os itens: menu principal (+ cada submenu), menu de pessoas (+ linhas de B e C), menu de contexto sobre X e sobre o vazio, menu de acções, diálogo de atalhos");
    await A.page.click('[data-testid="main-menu.button"]'); await wait(300);
    procurados.menu_principal = await visiveis();
    const subs = await A.page.$$('[data-testid^="main-menu-sub."]');
    for (const sb of subs) { const t = await sb.getAttribute("data-testid"); await sb.hover(); await wait(300); procurados[t] = await visiveis(); }
    await tecla(A, "Escape"); await tecla(A, "Escape"); await wait(200);
    await menuPessoas(A); procurados.menu_pessoas = await visiveis();
    const linhas = await A.page.$$(".tlui-people-menu__item");
    for (const l of linhas) { await l.hover(); await wait(200); }
    procurados.menu_pessoas_hover = await visiveis();
    await tecla(A, "Escape"); await wait(200);
    await clica(A, [480, 300], { button: "right" }); await wait(300);
    procurados.contexto_forma = await visiveis();
    const csubs = await A.page.$$('[data-testid^="context-menu-sub."]');
    for (const sb of csubs) { const t = await sb.getAttribute("data-testid"); await sb.hover(); await wait(250); procurados[t] = await visiveis(); }
    await tecla(A, "Escape"); await tecla(A, "Escape"); await wait(200);
    await A.page.mouse.click(900, 600, { button: "right" }); await wait(300);
    procurados.contexto_vazio = await visiveis();
    await tecla(A, "Escape"); await wait(200);
    const am = await A.page.$('[data-testid="actions-menu.button"]');
    if (am) { await am.click(); await wait(300); procurados.menu_accoes = await visiveis(); await tecla(A, "Escape"); }
    await A.page.click('[data-testid="main-menu.button"]'); await wait(250);
    const ks = await A.page.$('[data-testid="main-menu.keyboard-shortcuts-button"]');
    if (ks) { await ks.click(); await wait(500); procurados.atalhos = await A.page.evaluate(() => { const d = document.querySelector("[role=dialog]"); return d ? d.innerText.split("\n").filter(Boolean) : []; }); await tecla(A, "Escape"); }
    const achados = Object.entries(procurados).flatMap(([k, xs]) => xs.filter((x) => PADRAO.test(x)).map((x) => `${k}: ${x}`));
    sub.deterministico.procura = { padrao: String(PADRAO), achados, itens_por_menu: Object.fromEntries(Object.entries(procurados).map(([k, v]) => [k, v.length])) };
    sub.variavel.itens = procurados;
  });
  const a = s.deterministico.procura.achados;
  return { existe: a.length > 0, subcorridas: [s], resposta: a.length ? `achado: ${a.join("; ")}` : `NÃO existe: nenhum item que case ${s.deterministico.procura.padrao} em ${Object.keys(s.deterministico.procura.itens_por_menu).length} menus/diálogos (${Object.values(s.deterministico.procura.itens_por_menu).reduce((x, y) => x + y, 0)} itens)` };
};

CASO.c23_tres_pares = async () => {
  const X = R("X", 400, 250);
  const ordens = [["A", "B", "C"], ["A", "C", "B"], ["B", "A", "C"], ["B", "C", "A"], ["C", "A", "B"], ["C", "B", "A"]];
  const subs = [];
  for (const ordem of ordens) subs.push(await subcorrida("c23", ordem.join(""), ["A", "B", "C"], [X], async ({ A, B, C, g }) => {
    g("retido: A arrasta X (+300,+50), B (−200,+250), C (+150,+300)");
    await concorrente([A, B, C], [[A, (P) => arrasta(P, centro(X), add(centro(X), [300, 50]))], [B, (P) => arrasta(P, centro(X), add(centro(X), [-200, 250]))], [C, (P) => arrasta(P, centro(X), add(centro(X), [150, 300]))]], ordem);
  }));
  const quem = (f) => (f.x === 700 && f.y === 300 ? "A" : f.x === 200 && f.y === 500 ? "B" : f.x === 550 && f.y === 550 ? "C" : `(${f.x},${f.y})`);
  const res = subs.map((s) => `${s.deterministico.ordem_de_libertacao}: ${quem(achaForma(s.deterministico.final.servidor, "X"))}${s.deterministico.convergiu ? "" : " NÃO converge"}`);
  return { existe: true, subcorridas: subs, resposta: `${subs.every((s) => s.deterministico.convergiu) ? "os três convergem sempre" : "NEM sempre convergem"}; ganha: ${res.join(" · ")}` };
};

// ------------------------------------------------------------------ corrida
const versao = (p) => JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules", p, "package.json"), "utf8")).version;
const lista = so ? so.split(",") : CASOS.map((c) => c.id);
const resumoCorrida = {};
try {
  for (const c of CASOS) {
    if (!lista.some((x) => c.id.startsWith(x))) continue;
    const t = Date.now();
    process.stdout.write(`${c.id} … `);
    let r;
    try { r = await CASO[c.id](); } catch (err) { r = { existe: null, erro: String(err.stack || err).slice(0, 800), subcorridas: [] }; await fechaPares(); }
    const saida = { caso: c.id, o_que: c.o_que, pergunta: c.pergunta, existe: r.existe, resposta: r.resposta ?? null, ...(r.erro ? { erro: r.erro } : {}), subcorridas: r.subcorridas };
    fs.writeFileSync(path.join(outDir, `${c.id}.json`), JSON.stringify(saida, null, 1) + "\n");
    resumoCorrida[c.id] = { resposta: saida.resposta, ms: Date.now() - t };
    console.log(`${Date.now() - t} ms — ${saida.resposta ?? saida.erro}`);
  }
} finally {
  await browser.close().catch(() => {});
  mataServidor();
}
const meta = {
  oraculo: "tldraw sync (contraste)",
  data: new Date().toISOString(),
  versoes: { tldraw: versao("tldraw"), "@tldraw/sync": versao("@tldraw/sync"), "@tldraw/sync-core": versao("@tldraw/sync-core"), "@tldraw/tlschema": versao("@tldraw/tlschema"), "@tldraw/assets": versao("@tldraw/assets"), ws: versao("ws"), react: versao("react"), playwright: versao("playwright"), chromium: chromiumVersion, node: process.version },
  casos_json_sha256: sha(fs.readFileSync(CASOS_PATH)),
  corredor_sha256: sha(fs.readFileSync(fileURLToPath(import.meta.url))),
  servidor_sha256: sha(fs.readFileSync(path.join(HERE, "servidor.mjs"))),
  pagina_sha256: sha(fs.readFileSync(path.join(HERE, "src", "oracle.js"))),
  maquina: { loadavg_no_fim: os.loadavg().map(r3), cpus: os.cpus().length },
  reproduzir: "cd <worktree> && bash docs/MiroClone/ferramentas/colab_oracle/tldraw_sync/instala.sh && PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/colab_oracle/tldraw_sync/corre_colab.mjs <saida>",
  casos: resumoCorrida,
};
fs.writeFileSync(path.join(outDir, so ? `meta.parcial.json` : "meta.json"), JSON.stringify(meta, null, 2) + "\n");
console.log("servidor (pid " + SRV_PID + ") terminado; saída em " + outDir);
