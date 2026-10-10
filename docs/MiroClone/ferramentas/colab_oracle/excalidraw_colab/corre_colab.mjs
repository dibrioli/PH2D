// Oráculo da colaboração ao vivo do Excalidraw (app do excalidraw.com + excalidraw-room, tudo em 127.0.0.1).
// Corre os casos de ../casos.json com 2–3 contextos do Playwright na MESMA sala (o link que o próprio
// app gera em «Live collaboration… → Start session»). Gestos = rato e teclado do PRODUTO. Leitura do
// estado = observável em execução: a instância do editor achada pelo gancho padrão das devtools do
// React (__REACT_DEVTOOLS_GLOBAL_HOOK__) e a sua API pública (getSceneElementsIncludingDeleted,
// .state), o DOM (avatares, «Following …») e os pixels do canvas. Nunca se lê o fonte (CLAUDE.md §0.9).
//
// Concorrência controlada por page.routeWebSocket: cada mensagem socket.io (cabeçalho «45N-[…]» + N
// anexos binários encriptados) é UM grupo; o harness retém os grupos de saída de um par e liberta-os
// pela ordem do caso, duplica (c16) e inverte lotes (c17). Encriptado: só evento, bytes e instante.
//
// Uso: node corre_colab.mjs <pasta_saida> [id_caso ...]   (servidores: servidores.sh start)
import { chromium } from "playwright";
import fs from "fs";
import path from "path";
import crypto from "crypto";

const HERE = path.dirname(new URL(import.meta.url).pathname);
const COLAB_HOME = process.env.COLAB_HOME || path.join(process.env.HOME, "Apps/excalidraw-colab-oracle");
const BUILD = path.join(COLAB_HOME, "excalidraw/excalidraw-app/build");
const APP = "http://127.0.0.1:5101/";
const OUT = path.resolve(process.argv[2] || "saida");
const SO = process.argv.slice(3);
const CASOS = JSON.parse(fs.readFileSync(process.env.CASOS_JSON || path.join(HERE, "../casos.json"), "utf8")).casos;
fs.mkdirSync(OUT, { recursive: true });

const HOOK = `(() => { const roots = new Set();
  window.__REACT_DEVTOOLS_GLOBAL_HOOK__ = { supportsFiber: true, renderers: new Map(),
    inject(r) { this.renderers.set(this.renderers.size + 1, r); return this.renderers.size; },
    onCommitFiberRoot(id, root) { roots.add(root); }, onCommitFiberUnmount() {}, onPostCommitFiberRoot() {}, checkDCE() {}, isDisabled: false };
  window.__oracle_api = () => { for (const r of roots) { const st = [r.current]; while (st.length) { const f = st.pop(); if (!f) continue;
    const s = f.stateNode; if (s && typeof s.getSceneElementsIncludingDeleted === "function" && typeof s.updateScene === "function") return s;
    if (f.child) st.push(f.child); if (f.sibling) st.push(f.sibling); } } return null; }; })();`;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const r1 = (v) => (typeof v === "number" ? Math.round(v * 10) / 10 : v);

// ───────────────────────────── pares ─────────────────────────────
let browser, T0;
const now = () => Date.now() - T0;

class Par {
  constructor(nome, caso) { Object.assign(this, { nome, caso, msgs: [], bloqueados: [], ligacoes: 0, hold: false, outQ: [], inQ: [],
    buraco: false, dup: false, ws: null, server: null, pendOut: null, pendIn: null, fechos: [] }); }
  async abre(url) {
    this.ctx = await browser.newContext({ serviceWorkers: "block", viewport: { width: 1400, height: 900 }, locale: "en-US", deviceScaleFactor: 1 });
    await this.ctx.addInitScript(HOOK);
    await this.ctx.route(/.*/, (r) => {
      const u = new URL(r.request().url());
      if (["127.0.0.1", "localhost"].includes(u.hostname)) {
        if (u.port === "3102") this.caso.polling = (this.caso.polling || 0) + 1;
        return r.continue();
      }
      // fontes do CDN: servidas do próprio build local (mesmos ficheiros); nada sai da máquina.
      const m = u.pathname.match(/^\/oss\/(fonts\/.*)$/);
      if (u.hostname.endsWith("digitaloceanspaces.com") && m && fs.existsSync(path.join(BUILD, m[1]))) return r.fulfill({ path: path.join(BUILD, m[1]) });
      this.bloqueados.push(`${r.request().method()} ${u.origin}${u.pathname}`); return r.abort();
    });
    await this.ctx.routeWebSocket(/127\.0\.0\.1:3102\//, (ws) => this.liga(ws));
    this.p = await this.ctx.newPage();
    this.p.on("pageerror", (e) => this.caso.erros_pagina.push(`${this.nome}: ${String(e).slice(0, 200)}`));
    await this.p.goto(url || APP);
    await this.p.waitForFunction(() => !!window.__oracle_api?.(), null, { timeout: 20000 });
    return this;
  }
  liga(ws) {
    this.ligacoes++; this.caso.eventos.push({ t: now(), par: this.nome, ev: "ws_abre", n: this.ligacoes });
    const server = ws.connectToServer(); this.ws = ws; this.server = server;
    ws.onMessage((m) => this.dePagina(m));
    server.onMessage((m) => this.deServidor(m));
    ws.onClose((c) => { this.caso.eventos.push({ t: now(), par: this.nome, ev: "ws_fecha_pagina", code: c }); try { server.close(); } catch {} });
    server.onClose((c) => { this.caso.eventos.push({ t: now(), par: this.nome, ev: "ws_fecha_servidor", code: c }); try { ws.close(); } catch {} });
  }
  // agrupa quadros em mensagens socket.io (cabeçalho + anexos binários)
  agrupa(m, slot, fim) {
    const txt = typeof m === "string";
    if (this[slot]) { this[slot].frames.push(m); this[slot].bytes += txt ? Buffer.byteLength(m) : m.length;
      if (this[slot].frames.length === this[slot].need + 1) { const g = this[slot]; this[slot] = null; fim(g); } return; }
    if (!txt) { fim({ frames: [m], need: 0, bytes: m.length, ev: "?bin", ctl: false }); return; }
    const mb = m.match(/^45(\d+)-(\[.*)$/s), me = m.match(/^42(\[.*)$/s);
    let ev = null; try { ev = JSON.parse((mb ? mb[2] : me ? me[1] : "[]"))[0] ?? null; } catch { ev = "?"; }
    const g = { frames: [m], need: mb ? Number(mb[1]) : 0, bytes: Buffer.byteLength(m), ev: ev ?? `engine:${m.slice(0, 2)}`, ctl: !mb && !me };
    if (g.need > 0) this[slot] = g; else fim(g);
  }
  dePagina(m) { this.agrupa(m, "pendOut", (g) => {
    g.t = now(); this.msgs.push({ t: g.t, par: this.nome, dir: "envia", ev: g.ev, bytes: g.bytes, frames: g.frames.length });
    if (this.buraco) { this.outQ.push(g); return; }
    if (this.hold && !g.ctl) { this.outQ.push(g); return; }
    this.manda(g); if (this.dup && !g.ctl && g.ev === "server-broadcast") { this.manda(g); this.msgs.push({ t: now(), par: this.nome, dir: "duplicada", ev: g.ev, bytes: g.bytes }); }
  }); }
  deServidor(m) { this.agrupa(m, "pendIn", (g) => {
    g.t = now(); this.msgs.push({ t: g.t, par: this.nome, dir: "recebe", ev: g.ev, bytes: g.bytes, frames: g.frames.length });
    if (this.buraco) { this.inQ.push(g); return; }
    for (const f of g.frames) this.ws.send(f);
  }); }
  manda(g) { for (const f of g.frames) this.server.send(f); }
  liberta(grupos) { const gs = grupos ?? this.outQ.splice(0); for (const g of gs) this.manda(g); this.caso.eventos.push({ t: now(), par: this.nome, ev: "liberta", n: gs.length }); return gs.length; }
  async fecha() { try { await this.ctx.close(); } catch {} }

  // ── leitura do estado (API pública da instância em execução) ──
  async els() { return this.p.evaluate(() => window.__oracle_api().getSceneElementsIncludingDeleted().map((e) => ({
    id: e.id, type: e.type, x: e.x, y: e.y, width: e.width, height: e.height, strokeColor: e.strokeColor, backgroundColor: e.backgroundColor,
    text: e.text ?? null, originalText: e.originalText ?? null, containerId: e.containerId ?? null, isDeleted: e.isDeleted, index: e.index,
    version: e.version, versionNonce: e.versionNonce, updated: e.updated, boundElements: e.boundElements ?? null,
    startBinding: e.startBinding ?? null, endBinding: e.endBinding ?? null, points: e.points ?? null }))); }
  async app() { return this.p.evaluate(() => { const s = window.__oracle_api().state;
    return { scrollX: s.scrollX, scrollY: s.scrollY, zoom: s.zoom?.value, selectedElementIds: Object.keys(s.selectedElementIds || {}).filter((k) => s.selectedElementIds[k]),
      collaborators: [...(s.collaborators || new Map()).entries()].map(([k, v]) => ({ socketId: k, username: v.username ?? null, isCurrentUser: !!v.isCurrentUser,
        pointer: v.pointer ?? null, button: v.button ?? null, selectedElementIds: Object.keys(v.selectedElementIds || {}), userState: v.userState ?? null, color: v.color ?? null })) }; }); }
  async ids() { return (await this.els()).map((e) => e.id); }

  // ── gestos (rato e teclado) ──
  get m() { return this.p.mouse; }
  async esc() { await this.p.keyboard.press("Escape"); }
  // um diálogo que aparece sem ser pedido é registado (texto) e fechado pelo fundo, como um utilizador faria
  async semDialogo() {
    for (let i = 0; i < 10 && (await this.p.locator(".Modal__background").count()) > 0; i++) {
      if (i === 0) this.caso.eventos.push({ t: now(), par: this.nome, ev: "dialogo_inesperado", texto: await this.p.evaluate(() => document.querySelector(".excalidraw-modal-container")?.innerText?.slice(0, 200) ?? null) });
      await this.m.click(20, 880); await sleep(200);
    }
  }
  async ferramenta(n) { await this.semDialogo(); await this.p.getByTestId(`toolbar-${n}`).click(); }
  async rect(x, y, w, h, bg) {
    const antes = new Set(await this.ids()); await this.esc(); await this.ferramenta("rectangle");
    await this.m.move(x, y); await this.m.down(); await this.m.move(x + w, y + h, { steps: 4 }); await this.m.up();
    if (bg) { await this.semDialogo(); await this.p.getByTestId(`color-top-pick-${bg}`).first().click(); }
    const novo = (await this.ids()).filter((i) => !antes.has(i))[0];
    if (bg && novo) { const e = (await this.els()).find((e) => e.id === novo);
      if (e.backgroundColor !== bg) { this.caso.eventos.push({ t: now(), par: this.nome, ev: "cor_nao_aplicada_repete", pedida: bg, ficou: e.backgroundColor });
        await this.p.getByTestId(`color-top-pick-${bg}`).first().click(); } }
    await this.esc(); return novo;
  }
  async seta(x0, y0, x1, y1) {
    const antes = new Set(await this.ids()); await this.esc(); await this.ferramenta("arrow");
    await this.m.move(x0, y0); await this.m.down(); await this.m.move(x1, y1, { steps: 8 }); await this.m.up(); await this.esc();
    return (await this.ids()).filter((i) => !antes.has(i))[0];
  }
  async texto(x, y, s) {
    const antes = new Set(await this.ids()); await this.esc(); await this.ferramenta("text");
    await this.m.click(x, y); await this.p.waitForSelector("textarea"); await this.p.keyboard.type(s); await this.esc(); await sleep(100);
    return (await this.ids()).filter((i) => !antes.has(i))[0];
  }
  async clica(x, y) { await this.esc(); await this.m.click(x, y); }
  async arrasta(x0, y0, x1, y1, steps = 8) { await this.m.move(x0, y0); await this.m.down(); await this.m.move(x1, y1, { steps }); await this.m.up(); }
}

// ───────────────────────────── sala ─────────────────────────────
async function sala(caso, nomes = ["A", "B"], { entraTodos = true } = {}) {
  const A = await new Par(nomes[0], caso).abre();
  await A.p.getByRole("button", { name: /Live collaboration/i }).first().click();
  await A.p.getByRole("button", { name: /Start session/i }).click();
  await A.p.waitForFunction(() => location.hash.startsWith("#room="), null, { timeout: 15000 });
  const link = await A.p.evaluate(() => location.href);
  // o diálogo de partilha abre depois do «Start session»: espera-o e fecha-o pelo fundo até sumir
  await A.p.getByRole("button", { name: /Stop session/i }).waitFor({ timeout: 10000 }).catch(() => {});
  for (let i = 0; i < 25 && (await A.p.locator(".Modal__background").count()) > 0; i++) { await A.p.mouse.click(20, 880); await sleep(200); }
  if ((await A.p.locator(".Modal__background").count()) > 0) throw new Error("o diálogo de partilha não fechou");
  caso.link_forma = link.replace(/#room=[^,]+,.+$/, "#room=<id20hex>,<chave22b64>");
  const pares = [A];
  if (entraTodos) for (const n of nomes.slice(1)) pares.push(await entra(caso, n, link, pares.length + 1, pares));
  return { pares, link };
}
async function entra(caso, nome, link, total, outros) {
  const P = await new Par(nome, caso).abre(link);
  for (const q of [...outros, P]) await q.p.waitForFunction((n) => window.__oracle_api()?.state.collaborators?.size >= n, total, { timeout: 15000 });
  return P;
}

// convergência: lista por z (ordem do array), campos de estado; null ≡ [] em boundElements
function canon(els, nomes) {
  return els.map((e, z) => ({ papel: nomes[e.id] ?? "?" + e.type, type: e.type, x: r1(e.x), y: r1(e.y), w: r1(e.width), h: r1(e.height),
    bg: e.backgroundColor, stroke: e.strokeColor, text: e.text, apagado: e.isDeleted, index: e.index }));
}
const visiveis = (c) => c.filter((e) => !e.apagado);
async function converge(pares, nomes, ms = 25000) {
  const t0 = Date.now(); let ult;
  while (Date.now() - t0 < ms) {
    const ss = await Promise.all(pares.map((p) => p.els()));
    const cs = ss.map((s) => JSON.stringify(canon(s, nomes)));
    const vs = ss.map((s) => JSON.stringify(visiveis(canon(s, nomes))));
    ult = { ss, cs, vs };
    if (cs.every((c) => c === cs[0])) return { convergiu: true, convergiu_visiveis: true, ms: Date.now() - t0, ss };
    await sleep(100);
  }
  return { convergiu: false, convergiu_visiveis: ult.vs.every((v) => v === ult.vs[0]), ms: Date.now() - t0, ss: ult.ss };
}
function estado(ss, pares, nomes) {
  const o = {}; pares.forEach((p, i) => { o[p.nome] = ss[i].map((e, z) => ({ papel: nomes[e.id] ?? "?" + e.type, z, ...e, x: r1(e.x), y: r1(e.y), width: r1(e.width), height: r1(e.height) })); });
  return o;
}
const porPapel = (lista, papel) => lista.find((e) => e.papel === papel);
// libera os pares retidos pela ordem dada, com folga para o servidor ordenar
async function libertaPorOrdem(pares, ordem, gap = 200) {
  for (const n of ordem) { pares.find((p) => p.nome === n).liberta(); await sleep(gap); }
  for (const p of pares) { p.hold = false; p.liberta(); }
}
async function retem(pares, acoes) {
  for (const p of pares) p.hold = true;
  for (const a of acoes) await a();
  await sleep(600); // deixa sair (para a fila) o que o app atrasa
}
const vers = (els, nomes) => Object.fromEntries(els.filter((e) => nomes[e.id]).map((e) => [nomes[e.id], e.version]));

// ───────────────────────────── casos ─────────────────────────────
const C = {};

C.c01_mesmo_campo = async (k) => {
  const rodadas = [];
  for (const ordem of [["A", "B"], ["B", "A"], ["A", "B"], ["B", "A"], ["A", "B"], ["B", "A"]]) {
    const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
    const X = await A.rect(400, 300, 120, 80, "#a5d8ff"); const nomes = { [X]: "X" };
    await converge([A, B], nomes);
    await retem([A, B], [() => A.arrasta(460, 340, 660, 340), () => B.arrasta(460, 340, 460, 540)]);
    const local = { A: porPapel(canon(await A.els(), nomes), "X"), B: porPapel(canon(await B.els(), nomes), "X") };
    const xl = async (P) => { const e = (await P.els()).find((e) => e.id === X); return { version: e.version, versionNonce: e.versionNonce }; };
    const v0 = { A: (await xl(A)).version, B: (await xl(B)).version }; const n0 = { A: (await xl(A)).versionNonce, B: (await xl(B)).versionNonce };
    await libertaPorOrdem([A, B], ordem);
    const c = await converge([A, B], nomes); const x = porPapel(canon(c.ss[0], nomes), "X");
    const nonce = { A: c.ss[0].find((e) => e.id === X).versionNonce, B: c.ss[1].find((e) => e.id === X).versionNonce };
    rodadas.push({ ordem: ordem.join("→"), convergiu: c.convergiu, ms_ate_convergir: c.ms, local_antes_de_libertar: local, versao_local: v0, nonce_local: n0,
      ganhou_menor_nonce: null,
      final: { x: x.x, y: x.y }, ganhou: x.x === 600 ? "A" : x.y === 500 ? "B" : "nenhum", versao_final: c.ss[0].find((e) => e.id === X).version, versionNonce_final: nonce });
    const r = rodadas.at(-1); if (r.versao_local.A === r.versao_local.B && r.ganhou !== "nenhum") r.ganhou_menor_nonce = n0[r.ganhou] < n0[r.ganhou === "A" ? "B" : "A"];
    k.estado = estado(c.ss, [A, B], nomes);
    await A.fecha(); await B.fecha();
  }
  k.rodadas = rodadas;
  const g = rodadas.map((r) => r.ganhou);
  k.det = { convergiu_todas: rodadas.every((r) => r.convergiu), versoes_iguais_antes: rodadas.every((r) => r.versao_local.A === r.versao_local.B),
    empate_ganha_menor_nonce: rodadas.every((r) => r.ganhou_menor_nonce === true) ? true : rodadas.every((r) => r.ganhou_menor_nonce === false) ? false : "misto" };
  k.var = { ganhadores: g };
  k.resposta = `${rodadas.every((r) => r.convergiu) ? "convergem sempre" : "NÃO convergem"}; ganhadores por rodada ${g.join(",")} (ordens ${rodadas.map((r) => r.ordem).join(",")}); versões locais ${rodadas.map((r) => r.versao_local.A + "/" + r.versao_local.B).join(",")}; empate desfeito ${k.det.empate_ganha_menor_nonce === true ? "pelo MENOR versionNonce (aleatório por edição) — não pela ordem de chegada" : k.det.empate_ganha_menor_nonce === false ? "pelo MAIOR versionNonce" : "por critério misto"}`;
};

async function doisCampos(k, montar, acaoA, acaoB, ler) {
  const rodadas = [];
  for (const ordem of [["A", "B"], ["B", "A"]]) {
    const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
    const nomes = await montar(A, B); await converge([A, B], nomes);
    await retem([A, B], [() => acaoA(A), () => acaoB(B)]);
    const la = await A.els(), lb = await B.els();
    const local = { A: { ...ler(canon(la, nomes), null), versoes: vers(la, nomes) }, B: { ...ler(canon(lb, nomes), null), versoes: vers(lb, nomes) } };
    const retidas = { A: A.outQ.filter((g) => g.ev === "server-broadcast").length, B: B.outQ.filter((g) => g.ev === "server-broadcast").length };
    await libertaPorOrdem([A, B], ordem);
    const c = await converge([A, B], nomes);
    rodadas.push({ ordem: ordem.join("→"), convergiu: c.convergiu, ms_ate_convergir: c.ms, local_antes_de_libertar: local, fiaveis_retidas: retidas,
      ...ler(canon(c.ss[0], nomes), canon(c.ss[1], nomes)), versoes: { A: vers(c.ss[0], nomes), B: vers(c.ss[1], nomes) } });
    k.estado = estado(c.ss, [A, B], nomes); await A.fecha(); await B.fecha();
  }
  return rodadas;
}

C.c02_campos_diferentes_cor = async (k) => {
  let X;
  k.rodadas = await doisCampos(k, async (A) => { X = await A.rect(400, 300, 120, 80, "#a5d8ff"); return { [X]: "X" }; },
    (A) => A.arrasta(460, 340, 660, 340), async (B) => { await B.clica(460, 340); await B.p.getByTestId("color-top-pick-#ffc9c9").first().click(); await B.esc(); },
    (a, b) => { const x = porPapel(a, "X"); return { final: { x: x.x, y: x.y, bg: x.bg }, movido: x.x === 600, cor_nova: x.bg === "#ffc9c9" }; });
  k.det = k.rodadas.map((r) => ({ ordem: r.ordem, convergiu: r.convergiu, final: r.final }));
  k.resposta = k.rodadas.map((r) => `${r.ordem}: ${r.movido && r.cor_nova ? "funde (movido E cor nova)" : r.movido ? "fica só o MOVER (a cor de B perde-se inteira)" : r.cor_nova ? "fica só a COR (o mover de A perde-se inteiro)" : "nenhum"}${r.convergiu ? "" : " · NÃO converge"}`).join("; ");
};

C.c03_campos_diferentes_texto = async (k) => {
  k.rodadas = await doisCampos(k, async (A) => {
    const X = await A.rect(400, 300, 160, 80, "#a5d8ff");
    await A.m.dblclick(480, 340); await A.p.waitForSelector("textarea"); await A.p.keyboard.type("hello"); await A.esc(); await sleep(150);
    const t = (await A.els()).find((e) => e.type === "text"); return { [X]: "X", [t.id]: "texto" };
  }, (A) => A.arrasta(420, 310, 620, 310), async (B) => {
    await B.esc(); await B.m.dblclick(480, 340); await B.p.waitForSelector("textarea"); await B.p.keyboard.press("Control+a"); await B.p.keyboard.type("world"); await B.esc();
  }, (a) => { const x = porPapel(a, "X"), t = porPapel(a, "texto"); return { final: { x: x.x, y: x.y, texto: t?.text, texto_x: t?.x }, movido: x.x === 600, texto_novo: t?.text === "world", texto_acompanha: t ? t.x > 500 : null }; });
  k.det = k.rodadas.map((r) => ({ ordem: r.ordem, convergiu: r.convergiu, final: r.final }));
  k.resposta = k.rodadas.map((r) => `${r.ordem}: ${r.movido && r.texto_novo ? "funde (forma movida E texto novo)" : r.movido ? "só o mover" : r.texto_novo ? "só o texto" : "nenhum"}; texto ${r.texto_acompanha ? "acompanha a forma" : "NÃO acompanha a forma"}${r.convergiu ? "" : " · NÃO converge"}`).join("; ");
};

C.c04_apagar_x_editar = async (k) => {
  k.rodadas = await doisCampos(k, async (A) => ({ [await A.rect(400, 300, 120, 80, "#a5d8ff")]: "X" }),
    async (A) => { await A.clica(460, 340); await A.p.keyboard.press("Delete"); }, (B) => B.arrasta(460, 340, 460, 540),
    (a) => { const x = porPapel(a, "X"); return { final: { apagado: x.apagado, x: x.x, y: x.y }, ressuscita: !x.apagado }; });
  k.det = k.rodadas.map((r) => ({ ordem: r.ordem, convergiu: r.convergiu, final: r.final }));
  k.resposta = k.rodadas.map((r) => `${r.ordem}: ${r.ressuscita ? "RESSUSCITA (fica o mover de B)" : "fica apagado"}${r.convergiu ? "" : " · NÃO converge"}`).join("; ") + `; versões finais ${JSON.stringify(k.rodadas.map((r) => r.versoes.A.X))}`;
};

C.c05_criar_no_mesmo_z = async (k) => {
  const rodadas = [];
  for (const ordem of [["A", "B"], ["B", "A"]]) {
    const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
    const nomes = {}; let ia, ib;
    await retem([A, B], [async () => { ia = await A.rect(400, 300, 160, 100, "#ffc9c9"); }, async () => { ib = await B.rect(450, 340, 160, 100, "#b2f2bb"); }]);
    nomes[ia] = "XA"; nomes[ib] = "XB";
    const local = { A: canon(await A.els(), nomes).map((e) => e.papel + ":" + e.index), B: canon(await B.els(), nomes).map((e) => e.papel + ":" + e.index) };
    await libertaPorOrdem([A, B], ordem); const c = await converge([A, B], nomes);
    const za = canon(c.ss[0], nomes).map((e) => e.papel), zb = canon(c.ss[1], nomes).map((e) => e.papel);
    rodadas.push({ ordem: ordem.join("→"), convergiu: c.convergiu, ms_ate_convergir: c.ms, local_antes: local, z_A: za, z_B: zb, index: canon(c.ss[0], nomes).map((e) => e.papel + ":" + e.index), por_cima: { A: za.at(-1), B: zb.at(-1) } });
    k.estado = estado(c.ss, [A, B], nomes); await A.fecha(); await B.fecha();
  }
  k.rodadas = rodadas;
  k.det = rodadas.map((r) => ({ ordem: r.ordem, convergiu: r.convergiu, concordam: JSON.stringify(r.z_A) === JSON.stringify(r.z_B) }));
  k.var = { por_cima: rodadas.map((r) => r.por_cima), index: rodadas.map((r) => r.index) };
  k.resposta = rodadas.map((r) => `${r.ordem}: por cima em A=${r.por_cima.A}, em B=${r.por_cima.B} (${JSON.stringify(r.z_A) === JSON.stringify(r.z_B) ? "concordam" : "DISCORDAM"}; index ${r.index.join(" ")})`).join("; ");
};

C.c06_trazer_para_a_frente = async (k) => {
  const rodadas = [];
  for (const ordem of [["A", "B"], ["B", "A"]]) {
    const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
    const X = await A.rect(300, 300, 200, 150, "#ffc9c9"), Y = await A.rect(400, 350, 200, 150, "#b2f2bb"), Z = await A.rect(500, 400, 200, 150, "#a5d8ff");
    const nomes = { [X]: "X", [Y]: "Y", [Z]: "Z" }; await converge([A, B], nomes);
    await retem([A, B], [async () => { await A.clica(320, 320); await A.p.keyboard.press("Control+Shift+BracketRight"); await A.esc(); },
      async () => { await B.clica(580, 370); await B.p.keyboard.press("Control+Shift+BracketRight"); await B.esc(); }]);
    const local = { A: canon(await A.els(), nomes).map((e) => e.papel + ":" + e.index), B: canon(await B.els(), nomes).map((e) => e.papel + ":" + e.index) };
    await libertaPorOrdem([A, B], ordem); const c = await converge([A, B], nomes);
    rodadas.push({ ordem: ordem.join("→"), convergiu: c.convergiu, ms_ate_convergir: c.ms, local_antes: local, z_A: canon(c.ss[0], nomes).map((e) => e.papel), z_B: canon(c.ss[1], nomes).map((e) => e.papel),
      index: canon(c.ss[0], nomes).map((e) => e.papel + ":" + e.index), versoes: vers(c.ss[0], nomes) });
    k.estado = estado(c.ss, [A, B], nomes); await A.fecha(); await B.fecha();
  }
  // X e Y sobem para o MESMO index local (ver local_antes): a ordem final entre eles é um desempate, não a ordem de chegada
  k.rodadas = rodadas; k.det = rodadas.map((r) => ({ ordem: r.ordem, convergiu: r.convergiu, concordam: JSON.stringify(r.z_A) === JSON.stringify(r.z_B), Z_em_baixo: r.z_A[0] === "Z" }));
  k.var = { z_final: rodadas.map((r) => r.z_A.join("")) };
  k.resposta = rodadas.map((r) => `${r.ordem}: z final (baixo→cima) A=${r.z_A.join("")} B=${r.z_B.join("")}`).join("; ");
};

async function setaPQ(A) {
  const P = await A.rect(200, 300, 100, 100, "#ffc9c9"), Q = await A.rect(600, 300, 100, 100, "#a5d8ff");
  const S = await A.seta(250, 350, 650, 350); return { [P]: "P", [Q]: "Q", [S]: "seta" };
}
const lerSeta = (s) => s && ({ apagada: s.isDeleted, x: r1(s.x), y: r1(s.y), pontos: s.points?.map((p) => p.map(r1)), inicio: s.startBinding?.elementId ?? null, fim: s.endBinding?.elementId ?? null, version: s.version });

C.c07_seta_e_forma_apagada = async (k) => {
  const rodadas = [];
  for (const ordem of [["A", "B"], ["B", "A"]]) {
    const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
    const nomes = await setaPQ(A); await converge([A, B], nomes);
    const sid = Object.keys(nomes).find((i) => nomes[i] === "seta"), qid = Object.keys(nomes).find((i) => nomes[i] === "Q");
    const s0 = (await A.els()).find((e) => e.id === sid); const ponta = [s0.x + s0.points.at(-1)[0], s0.y + s0.points.at(-1)[1]];
    await retem([A, B], [async () => { await A.clica(450, s0.y); await sleep(100); await A.arrasta(ponta[0], ponta[1], 520, 600, 10); await A.esc(); },
      async () => { await B.clica(685, 385); await B.p.keyboard.press("Delete"); }]);
    await libertaPorOrdem([A, B], ordem); const c = await converge([A, B], nomes);
    const sa = c.ss[0].find((e) => e.id === sid), sb = c.ss[1].find((e) => e.id === sid), q = c.ss[0].find((e) => e.id === qid);
    const leg = (s) => ({ ...lerSeta(s), fim: s.endBinding ? (nomes[s.endBinding.elementId] ?? "?") : null, inicio: s.startBinding ? (nomes[s.startBinding.elementId] ?? "?") : null });
    rodadas.push({ ordem: ordem.join("→"), convergiu: c.convergiu, ms_ate_convergir: c.ms, ponta_antes: ponta.map(r1), seta_A: leg(sa), seta_B: leg(sb), Q_apagado: q.isDeleted,
      Q_boundElements: q.boundElements });
    k.estado = estado(c.ss, [A, B], nomes); await A.fecha(); await B.fecha();
  }
  k.rodadas = rodadas;
  k.det = rodadas.map((r) => ({ ordem: r.ordem, convergiu: r.convergiu, seta_apagada: r.seta_A.apagada, seta_fim: r.seta_A.fim, Q_apagado: r.Q_apagado, ponta: r.seta_A.pontos?.at(-1) }));
  k.resposta = rodadas.map((r) => `${r.ordem}: Q ${r.Q_apagado ? "apagado" : "vivo"}; seta ${r.seta_A.apagada ? "desaparece" : r.seta_A.fim ? `presa a ${r.seta_A.fim}${r.Q_apagado && r.seta_A.fim === "Q" ? " (APAGADO — presa a nada)" : ""}` : "solta"}${r.convergiu ? "" : " · NÃO converge"}`).join("; ");
};

C.c08_seta_segue_forma_alheia = async (k) => {
  const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
  const nomes = await setaPQ(A); await converge([A, B], nomes);
  const sid = Object.keys(nomes).find((i) => nomes[i] === "seta");
  const s0 = { A: lerSeta((await A.els()).find((e) => e.id === sid)), B: lerSeta((await B.els()).find((e) => e.id === sid)) };
  const t0 = now(); await B.arrasta(685, 385, 685, 585); await B.esc();
  const c = await converge([A, B], nomes, 6000); await sleep(800);
  const els = { A: await A.els(), B: await B.els() };
  const s1 = { A: lerSeta(els.A.find((e) => e.id === sid)), B: lerSeta(els.B.find((e) => e.id === sid)) };
  const depois = A.msgs.filter((m) => m.t >= t0 && m.dir === "envia" && m.ev === "server-broadcast");
  const bEnv = B.msgs.filter((m) => m.t >= t0 && m.dir === "envia" && m.ev === "server-broadcast");
  k.rodadas = [{ convergiu: c.convergiu, seta_antes: s0, seta_depois: s1, A_envia_broadcast_depois: depois.length, B_envia_broadcast: bEnv.length,
    seta_segue_em_A: JSON.stringify(s0.A.pontos) !== JSON.stringify(s1.A.pontos) || s0.A.y !== s1.A.y, seta_iguais: JSON.stringify(s1.A) === JSON.stringify(s1.B) }];
  k.estado = estado([els.A, els.B], [A, B], nomes);
  const r = k.rodadas[0];
  k.det = { convergiu: r.convergiu, segue: r.seta_segue_em_A, seta_final: s1.A.pontos, seta_muda_de_versao: s1.A.version > s0.A.version, A_reemite: r.A_envia_broadcast_depois > 0 };
  k.var = { versao_seta_antes_A_depois_A_depois_B: [s0.A.version, s1.A.version, s1.B.version] };
  k.resposta = `seta ${r.seta_segue_em_A ? "SEGUE Q do lado de A" : "NÃO segue"}; versão da seta ${s0.A.version}→${s1.A.version} (A) / ${s1.B.version} (B); A envia ${r.A_envia_broadcast_depois} broadcasts depois de receber ⇒ ${r.A_envia_broadcast_depois > 0 ? "A também recalcula/reemite" : "quem recalcula é B (o que moveu) e A só aplica"}`;
};

C.c09_texto_a_dois = async (k) => {
  const rodadas = [];
  for (const ordem of [["A", "B"], ["B", "A"]]) {
    const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
    const T = await A.texto(400, 300, "base"); const nomes = { [T]: "texto" }; await converge([A, B], nomes);
    const t = (await A.els()).find((e) => e.id === T);
    const edita = async (P, s) => { await P.esc(); await P.m.dblclick(t.x + 10, t.y + t.height / 2); await P.p.waitForSelector("textarea"); await P.p.keyboard.press("End"); await P.p.keyboard.type(s); await P.esc(); };
    await retem([A, B], [() => edita(A, "aaa"), () => edita(B, "bbb")]);
    const tl = async (P) => { const e = (await P.els()).find((e) => e.id === T); return { text: e.text, version: e.version, versionNonce: e.versionNonce }; };
    const local = { A: await tl(A), B: await tl(B) };
    await libertaPorOrdem([A, B], ordem); const c = await converge([A, B], nomes);
    rodadas.push({ ordem: ordem.join("→"), convergiu: c.convergiu, ms_ate_convergir: c.ms, local_antes: local, final_A: porPapel(canon(c.ss[0], nomes), "texto").text, final_B: porPapel(canon(c.ss[1], nomes), "texto").text });
    k.estado = estado(c.ss, [A, B], nomes); await A.fecha(); await B.fecha();
  }
  k.rodadas = rodadas;
  k.det = rodadas.map((r) => ({ ordem: r.ordem, convergiu: r.convergiu, intercala: /a.*b|b.*a/.test(r.final_A.replace("base", "")) }));
  k.var = { textos_finais: rodadas.map((r) => r.final_A) };
  k.resposta = rodadas.map((r) => `${r.ordem}: local A «${r.local_antes.A.text}» v${r.local_antes.A.version} · B «${r.local_antes.B.text}» v${r.local_antes.B.version}; final «${r.final_A}»/«${r.final_B}» ⇒ ${/aaa/.test(r.final_A) && /bbb/.test(r.final_A) ? "INTERCALA/funde" : "fica UM inteiro (o outro perde a edição toda)"}`).join("; ");
};

// Desfazer: o histórico do produto também guarda mudanças de SELECÇÃO (medido: depois de um clique no
// vazio, o 1.º Ctrl+Z só re-selecciona). Por isso cada tecla é registada (selecção de A + elementos) e
// a resposta usa a 1.ª tecla que mexe nos ELEMENTOS. Nenhum clique entre as teclas.
async function teclas(A, pares, nomes, tecla, n, rot) {
  const out = []; let ant = JSON.stringify(canon(await A.els(), nomes));
  for (let i = 1; i <= n; i++) {
    await A.p.keyboard.press(tecla); const c = await converge(pares, nomes, 4000); const cA = canon(c.ss[0], nomes); const ap = await A.app();
    const agora = JSON.stringify(cA);
    out.push({ tecla: rot, n: i, convergiu: c.convergiu, seleccao_A: ap.selectedElementIds.map((id) => nomes[id] ?? id), mexeu_elementos: agora !== ant,
      elementos: cA.map((e) => `${e.papel}${e.apagado ? "†" : ""}@(${e.x},${e.y})${e.bg}`) });
    ant = agora;
  }
  return out;
}
const primeiraQueMexe = (ps) => ps.find((p) => p.mexeu_elementos) ?? null;

C.c10_desfazer_so_o_meu = async (k) => {
  const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
  const X = await A.rect(300, 300, 120, 80, "#ffc9c9"); await sleep(300); const Y = await B.rect(600, 300, 120, 80, "#b2f2bb");
  const nomes = { [X]: "X", [Y]: "Y" }; await converge([A, B], nomes);
  const passos = await teclas(A, [A, B], nomes, "Control+KeyZ", 6, "Ctrl+Z");
  const c = await converge([A, B], nomes); k.estado = estado(c.ss, [A, B], nomes);
  k.rodadas = passos; k.det = passos.map(({ convergiu, mexeu_elementos, elementos, seleccao_A }) => ({ convergiu, mexeu_elementos, elementos, seleccao_A }));
  const yTocado = passos.some((p) => !p.elementos.some((e) => e.startsWith("Y@")));
  k.resposta = `A carrega Ctrl+Z 6× (A criou X e pintou-o; B criou Y depois): ${passos.map((p) => p.elementos.filter((e) => !e.includes("†")).map((e) => e.split("@")[0] + (e.endsWith("#ffc9c9") ? "(rosa)" : e.endsWith("transparent") ? "(sem fundo)" : "")).join("+") || "vazio").join(" → ")}; Y de B ${yTocado ? "É DESFEITO por A" : "NUNCA é tocado — desfaz só o de A"}`;
};

C.c11_desfazer_depois_do_outro = async (k) => {
  const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
  const X = await A.rect(400, 300, 120, 80, "#a5d8ff"); const nomes = { [X]: "X" }; await converge([A, B], nomes);
  await A.arrasta(460, 340, 660, 340); await converge([A, B], nomes);
  await B.clica(660, 340); await B.p.getByTestId("color-top-pick-#ffc9c9").first().click(); await B.esc(); await converge([A, B], nomes);
  const antes = canon(await A.els(), nomes);
  const z = await teclas(A, [A, B], nomes, "Control+KeyZ", 4, "Ctrl+Z");
  const y = await teclas(A, [A, B], nomes, "Control+Shift+KeyZ", 4, "Ctrl+Shift+Z");
  const c = await converge([A, B], nomes); k.estado = estado(c.ss, [A, B], nomes);
  k.rodadas = [{ antes: antes.map((e) => `${e.papel}@(${e.x},${e.y})${e.bg}`) }, ...z, ...y];
  k.det = k.rodadas.map(({ convergiu, mexeu_elementos, elementos, antes }) => antes ? { antes } : { convergiu, mexeu_elementos, elementos });
  const u = primeiraQueMexe(z), r = primeiraQueMexe(y);
  k.resposta = `antes: X@(600,300) rosa (movido por A, pintado por B); 1.º Ctrl+Z que mexe (tecla ${u?.n}): ${u?.elementos.join(" ")} ⇒ ${u ? (u.elementos[0].includes("(400,300)") && u.elementos[0].endsWith("#ffc9c9") ? "volta ao sítio E mantém a cor de B" : u.elementos[0].includes("(400,300)") ? "volta ao sítio e PERDE a cor de B" : "outra coisa") : "nenhum Ctrl+Z mexe"}; sequência Ctrl+Z: ${z.map((p) => p.elementos.join("")).join(" → ")}; 1.º Ctrl+Shift+Z que mexe (tecla ${r?.n}): ${r?.elementos.join(" ")}`;
};

C.c12_desfazer_o_que_o_outro_apagou = async (k) => {
  const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
  const X = await A.rect(400, 300, 120, 80, "#a5d8ff"); const nomes = { [X]: "X" }; await converge([A, B], nomes);
  await A.arrasta(460, 340, 660, 340); await converge([A, B], nomes);
  await B.clica(660, 340); await B.p.keyboard.press("Delete"); await converge([A, B], nomes);
  const z = await teclas(A, [A, B], nomes, "Control+KeyZ", 4, "Ctrl+Z");
  const y = await teclas(A, [A, B], nomes, "Control+Shift+KeyZ", 2, "Ctrl+Shift+Z");
  const c = await converge([A, B], nomes); k.estado = estado(c.ss, [A, B], nomes);
  k.rodadas = [...z, ...y]; k.det = k.rodadas.map(({ convergiu, mexeu_elementos, elementos }) => ({ convergiu, mexeu_elementos, elementos }));
  const vivo = z.find((p) => p.elementos.some((e) => e.startsWith("X@")));
  k.resposta = `Ctrl+Z ×4 de A: ${z.map((p) => p.elementos.join("")).join(" → ")} ⇒ ${vivo ? `X RESSUSCITA na tecla ${vivo.n} (${vivo.elementos.join("")}) — em B também: ${vivo.convergiu}` : "X NÃO ressuscita"}; Ctrl+Shift+Z ×2: ${y.map((p) => p.elementos.join("")).join(" → ")}`;
};

C.c13_entrar_a_meio = async (k) => {
  const { pares: [A], link } = await sala(k, ["A", "B"], { entraTodos: false }); k.pares.push(A);
  const ids = []; for (let i = 0; i < 50; i++) { const c = i % 10, l = Math.floor(i / 10); ids.push(await A.rect(300 + c * 100, 150 + l * 100, 70, 50, "#a5d8ff")); }
  const nomes = Object.fromEntries(ids.map((id, i) => [id, `F${String(i).padStart(2, "0")}`]));
  await sleep(1000); const tA = now();
  const B = new Par("B", k); k.pares.push(B);
  const tIni = now();
  const pB = B.abre(link); await pB;
  // B igual a A
  let tIgual = null; const fim = Date.now() + 15000; const alvo = JSON.stringify(canon(await A.els(), nomes));
  while (Date.now() < fim) { if (JSON.stringify(canon(await B.els(), nomes)) === alvo) { tIgual = now(); break; } await sleep(20); }
  await sleep(1500);
  const recB = B.msgs.filter((m) => m.dir === "recebe"), envA = A.msgs.filter((m) => m.dir === "envia" && m.t >= tA);
  const sum = (xs) => xs.reduce((s, m) => s + m.bytes, 0);
  const porEv = (xs) => Object.entries(xs.reduce((o, m) => { (o[m.ev] ??= { n: 0, bytes: 0 }); o[m.ev].n++; o[m.ev].bytes += m.bytes; return o; }, {})).map(([ev, v]) => ({ ev, ...v }));
  const visB = visiveis(canon(await B.els(), nomes)).length;
  k.medidas = { ms_ate_B_igual: tIgual === null ? null : tIgual - tIni, B_recebe_mensagens: recB.length, B_recebe_bytes: sum(recB), B_recebe_por_evento: porEv(recB),
    A_envia_mensagens: envA.length, A_envia_bytes: sum(envA), A_envia_por_evento: porEv(envA), B_envia_mensagens: B.msgs.filter((m) => m.dir === "envia").length };
  k.det = { B_ve_tudo: visB === 50, elementos_B: visB, igual: tIgual !== null };
  k.estado = { A_n: (await A.els()).length, B_n: (await B.els()).length };
  k.resposta = `B ${visB === 50 ? "vê as 50" : `vê ${visB}/50`}; entrada: B recebe ${recB.length} mensagens / ${sum(recB)} B; A envia ${envA.length} / ${sum(envA)} B; ${tIgual === null ? "B nunca ficou igual" : `${tIgual - tIni} ms do goto até B igual`}`;
};

// sem rede, dois regimes:
//  «offline» = context.setOffline (o navegador SABE: Wi-Fi desligado) — medido: o Chromium fecha o
//              WebSocket na hora e o app volta a ligar-se quando a rede volta;
//  «buraco»  = a ligação continua aberta e os quadros ficam retidos nas duas direcções, entregues ao
//              voltar (a ligação TCP que sobrevive a uma queda curta, sem o navegador saber).
async function semRede(P, modo, on) {
  if (modo === "offline") await P.ctx.setOffline(on);
  else if (on) P.buraco = true;
  else { P.buraco = false; for (const g of P.outQ.splice(0)) P.manda(g); for (const g of P.inQ.splice(0)) for (const f of g.frames) P.ws.send(f); }
  P.caso.eventos.push({ t: now(), par: P.nome, ev: on ? `rede_cai_${modo}` : `rede_volta_${modo}` });
}

C.c14_sair_e_voltar = async (k) => {
  const rodadas = [];
  // 3 repetições por regime: medido 10/10 que a cor final de V (criado por B sem rede) varia entre corridas
  for (const modo of ["offline", "offline", "offline", "buraco", "buraco", "buraco"]) {
    const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
    const X = await A.rect(400, 300, 120, 80, "#a5d8ff"); const nomes = { [X]: "X" }; await converge([A, B], nomes);
    const lig0 = B.ligacoes; await semRede(B, modo, true); await sleep(500);
    await A.arrasta(460, 340, 660, 340); await A.esc(); const W = await A.rect(300, 550, 100, 60, "#ffec99"); nomes[W] = "W";
    await B.clica(460, 340); await B.p.getByTestId("color-top-pick-#ffc9c9").first().click(); await B.esc(); const V = await B.rect(800, 550, 100, 60, "#b2f2bb"); nomes[V] = "V";
    await sleep(1500);
    const offlineUI = await B.p.evaluate(() => document.body.innerText.match(/offline|disconnect|reconnect/gi) ?? []);
    const lst = (els) => visiveis(canon(els, nomes)).map((e) => `${e.papel}@(${e.x},${e.y})${e.bg}`);
    const ea = await A.els(), eb = await B.els();
    const vn = (els) => Object.fromEntries(els.filter((e) => nomes[e.id]).map((e) => [nomes[e.id], `v${e.version}/${e.versionNonce}`]));
    const durante = { A: lst(ea), B: lst(eb), versoes_A: vn(ea), versoes_B: vn(eb) };
    await semRede(B, modo, false); const t0 = Date.now();
    const c = await converge([A, B], nomes, 30000);
    const x = { A: porPapel(canon(c.ss[0], nomes), "X"), B: porPapel(canon(c.ss[1], nomes), "X") };
    rodadas.push({ modo, convergiu: c.convergiu, ms_ate_convergir_apos_voltar: c.convergiu ? Date.now() - t0 : null, ligacoes_novas_de_B: B.ligacoes - lig0, aviso_na_interface: offlineUI,
      durante, final_A: visiveis(canon(c.ss[0], nomes)).map((e) => `${e.papel}@(${e.x},${e.y})${e.bg}`), final_B: visiveis(canon(c.ss[1], nomes)).map((e) => `${e.papel}@(${e.x},${e.y})${e.bg}`),
      X: x, versoes_finais: { A: vn(c.ss[0]), B: vn(c.ss[1]) }, cor_final_de_V: { A: porPapel(canon(c.ss[0], nomes), "V")?.bg, B: porPapel(canon(c.ss[1], nomes), "V")?.bg },
      perde: { mover_de_A: x.A.x !== 600 || x.B.x !== 600, cor_de_B_em_X: x.A.bg !== "#ffc9c9" || x.B.bg !== "#ffc9c9", W: !porPapel(visiveis(canon(c.ss[1], nomes)), "W"), V: !porPapel(visiveis(canon(c.ss[0], nomes)), "V") } });
    (k.estado ??= []).push(estado(c.ss, [A, B], nomes)); await A.fecha(); await B.fecha();
  }
  k.rodadas = rodadas;
  const semV = (l) => [...l].filter((e) => !e.startsWith("V@")).sort();
  k.det = rodadas.map((r) => ({ modo: r.modo, convergiu: r.convergiu, durante_A: r.durante.A, durante_B: r.durante.B, final_A_sem_cor_de_V: semV(r.final_A), final_B_sem_cor_de_V: semV(r.final_B), perde: r.perde }));
  k.var = { cor_final_de_V: rodadas.map((r) => `${r.modo}:${r.cor_final_de_V.A}`), z_final: rodadas.map((r) => r.final_A.map((e) => e.split("@")[0]).join("")) };
  const vRosa = (m) => rodadas.filter((r) => r.modo === m && r.cor_final_de_V.A === "#ffc9c9").length;
  k.resposta = ["offline", "buraco"].map((m) => { const rs = rodadas.filter((r) => r.modo === m); const r = rs[0];
    return `${m}: ${rs.every((r) => r.convergiu) ? "convergem" : "NÃO convergem"} (aos ${rs.map((r) => r.ms_ate_convergir_apos_voltar).join("/")} ms); perde-se ${Object.entries(r.perde).filter(([, v]) => v).map(([k]) => k).join(", ") || "nada"}; A e B acabam com ${semV(r.final_A).join(" ")} + V; V (verde em B antes de voltar) acaba ROSA (a cor que B deu a X) em ${vRosa(m)}/${rs.length}`; }).join(" | ");
};

C.c15_rede_cai_a_meio_do_arrasto = async (k) => {
  const rodadas = [];
  for (const modo of ["offline", "buraco"]) {
    const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
    const X = await A.rect(400, 300, 120, 80, "#a5d8ff"); const nomes = { [X]: "X" }; await converge([A, B], nomes);
    await A.m.move(460, 340); await A.m.down(); await A.m.move(560, 340, { steps: 6 }); await sleep(300);
    const meio = porPapel(canon(await B.els(), nomes), "X");
    await semRede(A, modo, true); await A.m.move(660, 440, { steps: 6 }); await A.m.up(); await A.esc(); await sleep(1000);
    const durante = porPapel(canon(await B.els(), nomes), "X");
    await semRede(A, modo, false); const c = await converge([A, B], nomes, 30000);
    const xa = porPapel(canon(c.ss[0], nomes), "X"), xb = porPapel(canon(c.ss[1], nomes), "X");
    rodadas.push({ modo, convergiu: c.convergiu, B_a_meio: { x: meio.x, y: meio.y }, B_durante_queda: { x: durante.x, y: durante.y }, final_A: { x: xa.x, y: xa.y }, final_B: { x: xb.x, y: xb.y },
      B_tem_final_de_A: xb.x === xa.x && xb.y === xa.y });
    k.estado = estado(c.ss, [A, B], nomes); await A.fecha(); await B.fecha();
  }
  k.rodadas = rodadas; k.det = rodadas.map(({ B_a_meio, ...r }) => r);
  k.resposta = rodadas.map((r) => `${r.modo}: B ${r.B_tem_final_de_A ? "acaba com a posição final de A" : `fica em (${r.final_B.x},${r.final_B.y}) ≠ A (${r.final_A.x},${r.final_A.y})`}`).join("; ");
};

C.c16_mensagem_duplicada = async (k) => {
  const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
  const X = await A.rect(400, 300, 120, 80, "#a5d8ff"); const nomes = { [X]: "X" }; await converge([A, B], nomes);
  A.dup = true; const t0 = now();
  await A.arrasta(460, 340, 660, 340); await A.esc(); const W = await A.rect(300, 550, 100, 60, "#ffec99"); nomes[W] = "W";
  await sleep(500); A.dup = false;
  const c = await converge([A, B], nomes); await sleep(500);
  const els = { A: await A.els(), B: await B.els() };
  const dups = A.msgs.filter((m) => m.dir === "duplicada" && m.t >= t0).length;
  const recB = B.msgs.filter((m) => m.dir === "recebe" && m.t >= t0 && m.ev === "client-broadcast").length;
  k.rodadas = [{ convergiu: c.convergiu, duplicadas: dups, B_recebeu_client_broadcast: recB, n_elementos: { A: els.A.length, B: els.B.length }, versoes: { A: vers(els.A, nomes), B: vers(els.B, nomes) } }];
  k.estado = estado([els.A, els.B], [A, B], nomes);
  k.det = { convergiu: c.convergiu, n_elementos: k.rodadas[0].n_elementos, mesmas_versoes: JSON.stringify(k.rodadas[0].versoes.A) === JSON.stringify(k.rodadas[0].versoes.B) };
  k.resposta = `${dups} mensagens fiáveis duplicadas; ${c.convergiu && els.B.length === els.A.length ? "IDEMPOTENTE: B igual a A, sem elemento repetido, mesmas versões" : "NÃO idempotente"} (A ${els.A.length} el., B ${els.B.length} el.)`;
};

C.c17_mensagens_reordenadas = async (k) => {
  const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
  const X = await A.rect(400, 300, 120, 80, "#a5d8ff"); const nomes = { [X]: "X" }; await converge([A, B], nomes);
  A.hold = true;
  await A.arrasta(460, 340, 560, 340); await A.esc(); await sleep(600); const lote1 = A.outQ.splice(0);
  await A.arrasta(560, 340, 560, 540); await A.esc(); await sleep(600); const lote2 = A.outQ.splice(0);
  const fiaveis = (l) => l.filter((g) => g.ev === "server-broadcast").length;
  A.liberta(lote2); await sleep(400); const aposLote2 = porPapel(canon(await B.els(), nomes), "X");
  A.liberta(lote1); await sleep(400); A.hold = false; A.liberta();
  const c = await converge([A, B], nomes, 4000);
  const xb = porPapel(canon(c.ss[1], nomes), "X"), xa = porPapel(canon(c.ss[0], nomes), "X");
  k.rodadas = [{ lote1: { mensagens: lote1.length, fiaveis: fiaveis(lote1) }, lote2: { mensagens: lote2.length, fiaveis: fiaveis(lote2) }, B_apos_lote2: { x: aposLote2.x, y: aposLote2.y },
    final_B: { x: xb.x, y: xb.y }, final_A: { x: xa.x, y: xa.y }, convergiu: c.convergiu }];
  k.estado = estado(c.ss, [A, B], nomes);
  k.det = { convergiu: c.convergiu, final_B: k.rodadas[0].final_B, final_A: k.rodadas[0].final_A };
  k.resposta = `B fica com ${xb.x === 500 && xb.y === 500 ? "a posição FINAL de A (o lote velho é ignorado)" : xb.x === 500 && xb.y === 300 ? "a posição do MEIO (o lote velho venceu)" : `(${xb.x},${xb.y})`}${c.convergiu ? "" : " · NÃO converge"}`;
};

const resumoEnvio = (msgs, t0, t1) => { const xs = msgs.filter((m) => m.dir === "envia" && m.t >= t0 && m.t <= t1); const s = (t1 - t0) / 1000;
  const por = {}; for (const m of xs) { (por[m.ev] ??= { n: 0, bytes: 0, min: Infinity, max: 0 }); const o = por[m.ev]; o.n++; o.bytes += m.bytes; o.min = Math.min(o.min, m.bytes); o.max = Math.max(o.max, m.bytes); }
  for (const o of Object.values(por)) { o.por_s = r1(o.n / s); o.bytes_por_s = Math.round(o.bytes / s); o.bytes_por_msg = Math.round(o.bytes / o.n); }
  return { janela_s: r1(s), total: xs.length, por_s: r1(xs.length / s), bytes: xs.reduce((a, m) => a + m.bytes, 0), bytes_por_s: Math.round(xs.reduce((a, m) => a + m.bytes, 0) / s), por_evento: por }; };

const AMOSTRADOR = `(() => { window.__amostras = []; const corBox = window.__corAlvo; let ativo = true; window.__paraAmostras = () => { ativo = false; };
  const cv = document.querySelector("canvas.interactive"); const ctx = cv && cv.getContext("2d", { willReadFrequently: true });
  const passo = (ts) => { if (!ativo) return; const s = window.__oracle_api().state; let ponteiro = null;
    for (const [, v] of s.collaborators) if (!v.isCurrentUser && v.pointer) ponteiro = { x: v.pointer.x, y: v.pointer.y };
    let desenhado = null;
    if (ctx && ponteiro) { const x0 = Math.max(0, Math.round(ponteiro.x + s.scrollX) - 120), y0 = Math.max(0, Math.round(ponteiro.y + s.scrollY) - 120);
      const w = Math.min(240, cv.width - x0), h = Math.min(240, cv.height - y0);
      if (w > 0 && h > 0) { const d = ctx.getImageData(x0, y0, w, h).data; let mx = 1e9, my = 1e9, n = 0;
        for (let i = 0; i < d.length; i += 4) if (d[i + 3] > 0) { const px = (i / 4) % w, py = Math.floor(i / 4 / w); if (px + py < mx + my) { mx = px; my = py; } n++; }
        if (n) desenhado = { x: x0 + mx, y: y0 + my, n }; } }
    window.__amostras.push({ t: Date.now(), ponteiro, desenhado }); requestAnimationFrame(passo); };
  requestAnimationFrame(passo); })();`;

async function corDoOutro(P) { await P.p.waitForSelector(".UserList__collaborator:not(.is-current-user) .Avatar", { timeout: 10000 }).catch(() => {}); return P.p.evaluate(() => { const a = document.querySelector(".UserList__collaborator:not(.is-current-user) .Avatar");
  const m = a && getComputedStyle(a).backgroundColor.match(/\d+/g); return m ? m.slice(0, 3).map(Number) : null; }); }

C.c18_cursor = async (k) => {
  const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
  const cor = await corDoOutro(B); await B.p.evaluate((c) => { window.__corAlvo = c; }, cor);
  await A.m.move(700, 300); await sleep(500); await B.p.evaluate(AMOSTRADOR); await sleep(200);
  const t0 = now(); const ini = Date.now(); let n = 0;
  while (Date.now() - ini < 2000) { const a = ((Date.now() - ini) / 2000) * 2 * Math.PI; await A.m.move(700 + 200 * Math.sin(a), 450 - 150 * Math.cos(a)); n++; await sleep(16); }
  const t1 = now(); await sleep(600);
  const amostras = await B.p.evaluate(() => { window.__paraAmostras(); return window.__amostras; });
  const T0abs = Date.now() - now();
  const janela = amostras.filter((s) => s.t - T0abs >= t0 && s.t - T0abs <= t1 + 300);
  let mud = 0, ult = null, saltos = [], ultD = null, mudD = 0, saltosD = [], difs = [];
  for (const s of janela) { if (s.ponteiro && (!ult || s.ponteiro.x !== ult.x || s.ponteiro.y !== ult.y)) { if (ult) saltos.push(Math.hypot(s.ponteiro.x - ult.x, s.ponteiro.y - ult.y)); mud++; ult = s.ponteiro; }
    if (s.desenhado && (!ultD || s.desenhado.x !== ultD.x || s.desenhado.y !== ultD.y)) { if (ultD) saltosD.push(Math.hypot(s.desenhado.x - ultD.x, s.desenhado.y - ultD.y)); mudD++; ultD = s.desenhado; }
    if (s.desenhado && s.ponteiro) difs.push(Math.hypot(s.desenhado.x - s.ponteiro.x, s.desenhado.y - s.ponteiro.y)); }
  const med = (xs) => xs.length ? r1(xs.slice().sort((a, b) => a - b)[Math.floor(xs.length / 2)]) : null;
  const ap = await B.app(); const outro = ap.collaborators.find((c) => !c.isCurrentUser);
  const nomeA = await A.p.evaluate(() => document.querySelector(".UserList__collaborator.is-current-user")?.closest(".excalidraw-tooltip-wrapper")?.textContent ?? null);
  const env = resumoEnvio(A.msgs, t0, t1);
  k.medidas = { movimentos_do_rato_em_A: n, A_envia: env, B_quadros_amostrados: janela.length,
    B_estado_ponteiro: { mudancas: mud, por_s: r1(mud / ((t1 - t0) / 1000)), salto_mediano_px: med(saltos), salto_max_px: r1(Math.max(0, ...saltos)) },
    B_desenhado: { quadros_com_cursor: janela.filter((s) => s.desenhado).length, mudancas: mudD, salto_mediano_px: med(saltosD), salto_max_px: r1(Math.max(0, ...saltosD)), dist_mediana_desenhado_vs_estado_px: med(difs) },
    movimento_real_px_por_evento: r1((2 * Math.PI * 175) / n) };
  k.det = { B_ve_nome: outro?.username ? true : false, cor_rgb_do_avatar_em_B: cor ? true : false, B_ve_ponteiro: !!outro?.pointer };
  k.var = { nome_visto_em_B: outro?.username, cor_rgb_avatar: cor };
  const vol = env.por_evento["server-volatile-broadcast"];
  const qps = r1(janela.length / ((t1 - t0 + 300) / 1000)), dps = r1(mudD / ((t1 - t0) / 1000));
  k.medidas.B_quadros_por_s = qps; k.medidas.B_desenhado.mudancas_por_s = dps; k.medidas.A_movimentos_por_s = r1(n / ((t1 - t0) / 1000));
  const interpola = mudD > mud * 1.5;
  k.det.B_interpola = interpola;
  k.resposta = `A move o rato ${k.medidas.A_movimentos_por_s}×/s e envia ${vol?.por_s ?? 0} msg/s (volatile, ${vol?.bytes_por_msg ?? "-"} B/msg, ${vol?.bytes_por_s ?? 0} B/s); B desenha o cursor ${dps}×/s em ${qps} quadros/s ⇒ ${interpola ? "INTERPOLA (suave)" : "SEM interpolação: aos saltos"} de ~${k.medidas.B_desenhado.salto_mediano_px}px (o rato real anda ${k.medidas.movimento_real_px_por_evento}px por evento); nome «${outro?.username}», cor do avatar rgb(${cor})`;
};

C.c19_arrastar_custo = async (k) => {
  const rodadas = [];
  for (const N of [1, 10, 50]) {
    const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
    const ids = []; for (let i = 0; i < N; i++) { const c = i % 10, l = Math.floor(i / 10); ids.push(await A.rect(300 + c * 60, 150 + l * 50, 40, 30, "#a5d8ff")); }
    const nomes = Object.fromEntries(ids.map((id, i) => [id, `F${i}`])); await converge([A, B], nomes, 15000);
    await A.m.click(1200, 750); if (N > 1) await A.p.keyboard.press("Control+a");
    const sel = (await A.app()).selectedElementIds.length;
    await B.p.evaluate((id) => { window.__trilho = []; let on = true; window.__paraTrilho = () => { on = false; };
      const f = () => { if (!on) return; const e = window.__oracle_api().getSceneElementsIncludingDeleted().find((x) => x.id === id); window.__trilho.push({ t: Date.now(), x: e.x, y: e.y }); requestAnimationFrame(f); }; requestAnimationFrame(f); }, ids[0]);
    await A.m.move(320, 165); await A.m.down(); const t0 = now(); const ini = Date.now(); let n = 0;
    while (Date.now() - ini < 2000) { const f = (Date.now() - ini) / 2000; await A.m.move(320 + 400 * f, 165 + 100 * Math.sin(f * Math.PI)); n++; await sleep(16); }
    await A.m.up(); const t1 = now(); await sleep(800);
    const trilho = await B.p.evaluate(() => { window.__paraTrilho(); return window.__trilho; });
    const T0abs = Date.now() - now(); const dur = trilho.filter((s) => s.t - T0abs >= t0 && s.t - T0abs <= t1);
    const pos = new Set(dur.map((s) => `${r1(s.x)},${r1(s.y)}`));
    const c = await converge([A, B], nomes, 8000);
    rodadas.push({ N, seleccionadas: sel, movimentos: n, A_envia: resumoEnvio(A.msgs, t0, t1), B_posicoes_distintas_durante: pos.size, B_quadros: dur.length, convergiu: c.convergiu,
      B_ve: pos.size > 3 ? "o caminho" : pos.size > 1 ? "poucos saltos" : "só o fim" });
    await A.fecha(); await B.fecha();
  }
  k.rodadas = rodadas;
  k.det = rodadas.map((r) => ({ N: r.N, seleccionadas: r.seleccionadas, convergiu: r.convergiu, B_ve: r.B_ve }));
  k.medidas = rodadas.map((r) => ({ N: r.N, msg_por_s: r.A_envia.por_s, bytes_por_s: r.A_envia.bytes_por_s, por_evento: r.A_envia.por_evento, B_posicoes_distintas: r.B_posicoes_distintas_durante }));
  k.resposta = rodadas.map((r) => `N=${r.N}: ${r.A_envia.por_s} msg/s, ${r.A_envia.bytes_por_s} B/s (fiáveis ${r.A_envia.por_evento["server-broadcast"]?.por_s ?? 0}/s × ${r.A_envia.por_evento["server-broadcast"]?.bytes_por_msg ?? "-"} B); B vê ${r.B_ve} (${r.B_posicoes_distintas_durante} posições)`).join("; ");
};

C.c20_seleccao_alheia = async (k) => {
  const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
  const X = await A.rect(400, 300, 160, 100, "#ffec99"); const nomes = { [X]: "X" }; await converge([A, B], nomes);
  await A.m.click(1200, 750); await sleep(500);
  const cor = await corDoOutro(B);
  // o canvas interactivo só leva sobreposições: pixels com alfa > 0 à volta de X = o contorno da selecção alheia; a cor média diz de quem é
  const contaCor = async () => B.p.evaluate(() => { const cv = document.querySelector("canvas.interactive"); const d = cv.getContext("2d").getImageData(380, 280, 200, 140).data; let n = 0, r = 0, g = 0, b = 0;
    for (let i = 0; i < d.length; i += 4) if (d[i + 3] > 0) { n++; r += d[i]; g += d[i + 1]; b += d[i + 2]; } return { n, rgb_medio: n ? [Math.round(r / n), Math.round(g / n), Math.round(b / n)] : null }; });
  const antes = await contaCor();
  await A.clica(480, 350); await A.m.move(1100, 750); await sleep(1200);
  const depois = await contaCor(); const ap = await B.app(); const outro = ap.collaborators.find((c) => !c.isCurrentUser);
  await B.p.screenshot({ path: path.join(OUT, "c20_B.png"), clip: { x: 350, y: 250, width: 400, height: 250 } }).catch(() => {});
  const perto = (a, b) => a && b && Math.abs(a[0] - b[0]) + Math.abs(a[1] - b[1]) + Math.abs(a[2] - b[2]) < 40;
  k.det = { B_ve_seleccao_no_estado: outro.selectedElementIds.includes(X), B_desenha_contorno: depois.n > 20 && antes.n < depois.n, contorno_na_cor_do_avatar_de_A: perto(depois.rgb_medio, cor) };
  k.var = { pixels_cor_A_antes: antes, pixels_cor_A_depois: depois, cor_rgb: cor, nome: outro.username };
  k.estado = { B_collaborators: ap.collaborators.map((c) => ({ ...c, selectedElementIds: c.selectedElementIds.map((i) => nomes[i] ?? i), socketId: "<id>" })) };
  k.resposta = `B ${k.det.B_ve_seleccao_no_estado ? "VÊ" : "não vê"} a selecção de A: contorno tracejado ${k.det.B_desenha_contorno ? "desenhado" : "NÃO desenhado"} (${antes.n}→${depois.n} px, cor média rgb(${depois.rgb_medio}) vs avatar de A rgb(${cor})${k.det.contorno_na_cor_do_avatar_de_A ? " — a MESMA cor" : ""}); o nome «${outro.username}» aparece na etiqueta do cursor e no avatar`;
};

C.c21_seguir = async (k) => {
  const { pares: [A, B] } = await sala(k); k.pares.push(A, B);
  await A.rect(400, 300, 160, 100, "#ffec99"); await sleep(500);
  const badge = (P) => P.p.evaluate(() => { const e = [...document.querySelectorAll("div, span")].find((x) => /^Following/.test(x.textContent.trim()) && x.textContent.length < 80); return e ? e.textContent.replace(/\s+/g, " ").trim() : null; });
  // clicar no avatar ALTERNA o seguir: só se clica quando B ainda não segue
  const segue = async () => { if (!(await badge(B))) { await B.semDialogo(); await B.p.locator(".UserList__collaborator:not(.is-current-user)").first().click(); await sleep(400); } return badge(B); };
  const xDoSelo = async () => { const r = await B.p.evaluate(() => { const e = [...document.querySelectorAll("div, span")].find((x) => /^Following/.test(x.textContent.trim()) && x.textContent.length < 80);
    const b = e && (e.querySelector("button") ?? e.parentElement?.querySelector("button")); if (!b) return null; const q = b.getBoundingClientRect(); return { x: q.x + q.width / 2, y: q.y + q.height / 2 }; });
    if (!r) throw new Error("selo sem botão ×"); await B.m.click(r.x, r.y); };
  const cam = (P) => P.p.evaluate(() => { const s = window.__oracle_api().state; return { sx: s.scrollX, sy: s.scrollY, z: s.zoom.value }; });
  const b0 = await segue();
  const sampler = (P) => P.p.evaluate(() => { window.__cam = []; let on = true; window.__paraCam = () => { on = false; return window.__cam; };
    const f = () => { if (!on) return; const s = window.__oracle_api().state; window.__cam.push({ t: Date.now(), sx: s.scrollX, sy: s.scrollY, z: s.zoom.value }); requestAnimationFrame(f); }; requestAnimationFrame(f); });
  await sampler(A); await sampler(B);
  await A.m.move(700, 450);
  for (let i = 0; i < 10; i++) { await A.m.wheel(30, 40); await sleep(30); }
  await sleep(800);
  await A.p.getByRole("button", { name: "Zoom in" }).click(); await sleep(300); await A.p.getByRole("button", { name: "Zoom in" }).click(); await sleep(1000);
  const ca = await A.p.evaluate(() => window.__paraCam()), cb = await B.p.evaluate(() => window.__paraCam());
  // atraso: para cada mudança de câmara de A, primeiro quadro de B com a mesma câmara
  const eq = (a, b) => Math.abs(a.sx - b.sx) < 0.5 && Math.abs(a.sy - b.sy) < 0.5 && Math.abs(a.z - b.z) < 1e-3;
  const mudA = []; for (let i = 1; i < ca.length; i++) if (!eq(ca[i], ca[i - 1])) mudA.push(ca[i]);
  const atrasos = []; for (const m of mudA) { const hit = cb.find((s) => s.t >= m.t - 5 && eq(s, m)); if (hit) atrasos.push(hit.t - m.t); }
  const fa = await cam(A), fb = await cam(B);
  const med = (xs) => xs.length ? xs.slice().sort((a, b) => a - b)[Math.floor(xs.length / 2)] : null;
  // o que interrompe
  const interrompe = {};
  const testes = [["B faz pan (roda do rato)", async () => { await B.m.move(700, 450); await B.m.wheel(0, 120); }],
    ["B clica no canvas", async () => { await B.m.click(1100, 700); }],
    ["B carrega em Zoom in", async () => { await B.p.getByRole("button", { name: "Zoom in" }).click(); }],
    ["B carrega no × do selo", xDoSelo],
    ["B arrasta no canvas", async () => { await B.arrasta(900, 650, 960, 700); }],
    ["A selecciona/desenha (B continua?)", async () => { await A.rect(800, 500, 80, 60); }]];
  for (const [rot, f] of testes) { const antes = await segue(); let err = null; try { await f(); } catch (e) { err = String(e).slice(0, 80); } await sleep(500); interrompe[rot] = { antes, depois: await badge(B), interrompeu: !!antes && !(await badge(B)), erro: err }; }
  // A vê quem o segue?
  const aVe = await A.p.evaluate(() => document.body.innerText.match(/follow[^\n]*/gi) ?? []);
  k.medidas = { mudancas_de_camara_A: mudA.length, atraso_ms_mediano: med(atrasos), atraso_ms_max: atrasos.length ? Math.max(...atrasos) : null, mudancas_acompanhadas: atrasos.length };
  k.det = { existe: !!b0, camara_final_igual: eq({ ...fa }, { ...fb }), interrompe: Object.fromEntries(Object.entries(interrompe).map(([a, v]) => [a, v.interrompeu])) };
  k.var = { selo: b0, camara_A: fa, camara_B: fb, A_ve_texto_follow: aVe, interrompe };
  k.resposta = `existe (clicar no avatar ⇒ selo «${b0}»); a câmara de B ${k.det.camara_final_igual ? "acompanha (pan e zoom)" : "NÃO fica igual"}; atraso mediano ${k.medidas.atraso_ms_mediano} ms (máx ${k.medidas.atraso_ms_max}); interrompem: ${Object.entries(k.det.interrompe).filter(([, v]) => v).map(([a]) => a).join(", ") || "nada"}`;
};

C.c22_trazer_todos = async (k) => {
  const { pares: [A, B, Cc] } = await sala(k, ["A", "B", "C"]); k.pares.push(A, B, Cc);
  const prova = {};
  await A.m.click(1200, 750); await A.p.keyboard.press("Control+/"); await sleep(500);
  const inp = A.p.locator("input").last();
  for (const q of ["summon", "bring", "everyone", "all users", "gather", "follow", "present", "collab", "viewport", "camera"]) {
    await inp.fill(q); await sleep(300);
    prova[`paleta:${q}`] = await A.p.evaluate(() => [...document.querySelectorAll(".command-palette-dialog .command-item, [class*=command-item]")].map((x) => x.textContent.trim().slice(0, 60)));
  }
  await A.esc(); await sleep(300); await A.esc();
  await A.p.locator(".UserList__pill, .UserList__pill-chevron, .UserList__more").first().click({ timeout: 3000 }).catch(() => {}); await sleep(400);
  prova.lista_de_utilizadores = await A.p.evaluate(() => (document.querySelector("[class*=UserList__collaborators], .UserList")?.closest("[class*=Island], [role=dialog], .UserList")?.innerText ?? "").split("\n").filter(Boolean));
  await A.esc(); await sleep(200);
  await A.p.getByTestId("main-menu-trigger").click({ timeout: 3000 }).catch(() => {}); await sleep(400);
  prova.menu_principal = await A.p.evaluate(() => [...document.querySelectorAll(".dropdown-menu-item, [role=menuitem]")].map((x) => x.textContent.trim().slice(0, 50)));
  await A.esc(); await sleep(200);
  await A.m.click(700, 450, { button: "right" }); await sleep(400);
  prova.menu_contexto_canvas = await A.p.evaluate(() => [...document.querySelectorAll(".context-menu li, .context-menu-item")].map((x) => x.textContent.trim().slice(0, 50)));
  await A.esc();
  await A.p.locator(".UserList__collaborator.is-current-user").first().click({ timeout: 3000 }).catch(() => {}); await sleep(400);
  prova.clicar_no_proprio_avatar = await A.p.evaluate(() => (document.body.innerText.match(/Following[^\n]*|summon[^\n]*|bring[^\n]*/gi) ?? []));
  const alvo = /summon|bring (all|every)|gather|trazer|everyone to me/i;
  const achou = Object.values(prova).flat().filter((s) => typeof s === "string" && alvo.test(s));
  k.existe = achou.length > 0;
  k.det = { existe: k.existe };
  k.prova = { procurado_por: String(alvo), achados: achou, ...prova };
  k.resposta = k.existe ? `existe: ${achou.join(" | ")}` : "NÃO existe: a paleta de comandos (10 termos), a lista de utilizadores, o menu principal, o menu de contexto e o próprio avatar não têm «trazer todos»; só existe SEGUIR (c21), iniciado por quem segue";
};

C.c23_tres_pares = async (k) => {
  const rodadas = [];
  for (const ordem of [["A", "B", "C"], ["A", "C", "B"], ["B", "A", "C"], ["B", "C", "A"], ["C", "A", "B"], ["C", "B", "A"]]) {
    const { pares } = await sala(k, ["A", "B", "C"]); k.pares.push(...pares); const [A, B, Cc] = pares;
    const X = await A.rect(400, 300, 120, 80, "#a5d8ff"); const nomes = { [X]: "X" }; await converge(pares, nomes);
    await retem(pares, [() => A.arrasta(460, 340, 660, 340), () => B.arrasta(460, 340, 460, 540), () => Cc.arrasta(460, 340, 310, 340)]);
    const local = {}; for (const p of pares) { const e = (await p.els()).find((e) => e.id === X); local[p.nome] = { version: e.version, versionNonce: e.versionNonce }; }
    await libertaPorOrdem(pares, ordem); const tLib = Date.now(); await sleep(1000);
    const quem0 = (x) => (x.x === 600 ? "A" : x.y === 500 ? "B" : x.x === 250 ? "C" : `(${x.x},${x.y})`);
    const aos1s = {}; for (const p of pares) aos1s[p.nome] = quem0(porPapel(canon(await p.els(), nomes), "X"));
    const c = await converge(pares, nomes, 40000); c.ms = Date.now() - tLib;
    const fimV = Object.fromEntries(pares.map((p, i) => { const e = c.ss[i].find((e) => e.id === X); return [p.nome, { version: e.version, versionNonce: e.versionNonce }]; }));
    const xs = c.ss.map((s) => porPapel(canon(s, nomes), "X"));
    const quem = (x) => (x.x === 600 ? "A" : x.y === 500 ? "B" : x.x === 250 ? "C" : `(${x.x},${x.y})`);
    rodadas.push({ ordem: ordem.join("→"), convergiu: c.convergiu, ms_ate_convergir: c.ms, um_segundo_depois: aos1s, final: Object.fromEntries(pares.map((p, i) => [p.nome, { x: xs[i].x, y: xs[i].y, ganhou: quem(xs[i]) }])), local_antes: local, final_versoes: fimV });
    k.estado = estado(c.ss, pares, nomes); for (const p of pares) await p.fecha();
  }
  k.rodadas = rodadas; k.det = { convergem_todas_em_40s: rodadas.every((r) => r.convergiu) };
  k.var = { por_ordem: rodadas.map((r) => ({ ordem: r.ordem, ganhou: r.final.A.ganhou, ms_ate_convergir: r.ms_ate_convergir, imediato: r.ms_ate_convergir < 1000 })) };
  k.resposta = rodadas.map((r) => `${r.ordem}: ${r.convergiu ? `1 s depois A/B/C em ${Object.values(r.um_segundo_depois).join("/")}; convergem em ${r.final.A.ganhou} aos ${r.ms_ate_convergir} ms` : `NÃO convergem em 40 s (A/B/C ficam em ${Object.values(r.final).map((f) => f.ganhou).join("/")})`}`).join("; ");
};

// ───────────────────────────── corrida ─────────────────────────────
browser = await chromium.launch();
const ver = { chromium: browser.version() };
const resumo = [];
for (const caso of CASOS) {
  if (SO.length && !SO.includes(caso.id)) continue;
  T0 = Date.now();
  const k = { id: caso.id, o_que: caso.o_que, pergunta: caso.pergunta, existe: true, pares: [], eventos: [], erros_pagina: [] };
  process.stdout.write(`${caso.id} … `);
  try { await C[caso.id](k); } catch (e) { k.erro = String(e.stack || e).slice(0, 1500);
    for (const p of k.pares) { try { if (p.p.isClosed()) continue; const txt = await p.p.evaluate(() => document.querySelector(".excalidraw-modal-container")?.innerText?.slice(0, 400) ?? null);
      if (txt !== null) k.erro += `\n[${p.nome}] diálogo aberto: ${txt}`; await p.p.screenshot({ path: path.join(OUT, `${caso.id}_${p.nome}_erro.png`) }); } catch {} } }
  for (const p of k.pares) await p.fecha();
  const mensagens = k.pares.flatMap((p) => p.msgs).sort((a, b) => a.t - b.t);
  const bloqueados = [...new Set(k.pares.flatMap((p) => p.bloqueados))].sort();
  const out = { id: k.id, o_que: k.o_que, pergunta: k.pergunta, existe: k.existe, resposta: k.resposta ?? null, erro: k.erro ?? null,
    deterministico: k.det ?? null,
    variavel: (() => { const ms = Array.isArray(k.rodadas) ? k.rodadas.map((r) => r.ms_ate_convergir ?? r.ms_ate_convergir_apos_voltar).filter((v) => v !== undefined) : [];
      return ms.length ? { ...(k.var ?? {}), ms_ate_convergir_por_rodada: ms } : (k.var ?? null); })(), medidas: k.medidas ?? null, rodadas: k.rodadas ?? null, prova: k.prova ?? null, estado_final: k.estado ?? null,
    link_forma: k.link_forma, eventos_ws: k.eventos, polling_http: k.polling ?? 0, erros_pagina: k.erros_pagina, pedidos_bloqueados: bloqueados,
    mensagens: mensagens.length > 4000 ? { truncado: true, n: mensagens.length, primeiras: mensagens.slice(0, 4000) } : mensagens };
  fs.writeFileSync(path.join(OUT, `${k.id}.json`), JSON.stringify(out, null, 1));
  resumo.push({ id: k.id, resposta: out.resposta, erro: out.erro });
  console.log(out.erro ? `ERRO ${out.erro.split("\n")[0]}` : out.resposta);
}
await browser.close();
fs.writeFileSync(path.join(OUT, "_corrida.json"), JSON.stringify({ quando: new Date().toISOString(), versoes: ver, resumo }, null, 1));
