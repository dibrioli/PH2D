// Oráculo tldraw 5.5.2 (tldraw license — uso em Development Environment, LICENCA_tldraw_v5.5.2.md;
// corrido localmente, nenhum código lido nem copiado). Medido 2026-10-19.
// Pergunta: como o DESFAZER trata as operações de PÁGINA (o análogo das abas do nosso quadro).
// Só API pública documentada (DOCS.md do pacote = tldraw.dev, secções «Pages» e «History»):
// editor.createPage / renamePage / updatePage / duplicatePage / deletePage / setCurrentPage /
// getPages / getCurrentPageId / getPageShapeIds / createShapes / markHistoryStoppingPoint /
// undo / redo / canUndo / canRedo / clearHistory / selectNone.
// Cada caso: editor NOVO (contexto novo do Chromium), cena montada e `clearHistory()`; cada gesto do
// utilizador = `markHistoryStoppingPoint(<gesto>)` + a chamada (o idioma da secção «History»); undo e
// redo sem marca. Depois de CADA passo: páginas (id, nome, index, formas), corrente, canUndo/canRedo.
// Uso: node corre_paginas.mjs [--so P1,P7a] <pasta_saida>  →  <saida>/paginas_<caso>.json
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/tldraw-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const PKG = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/tldraw/package.json"), "utf8"));
const PW = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/playwright/package.json"), "utf8"));
const SELF = fileURLToPath(import.meta.url);
const SELF_SHA = createHash("sha256").update(fs.readFileSync(SELF)).digest("hex");

let args = process.argv.slice(2);
const iso = args.indexOf("--so");
const so = iso >= 0 ? args.splice(iso, 2)[1] : null;
const [outDir] = args;
if (!outDir) {
  console.error("uso: node corre_paginas.mjs [--so P1,P2] <pasta_saida>");
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });

// ---------------------------------------------------------------- lado da página
// Monta a cena: páginas `paginas` ([{id, nome, formas:[nome]}], a 1.ª é «A»), corrente `corrente`;
// a página por omissão do tldraw é apagada; o histórico fica LIMPO.
const pageSetup = ({ paginas, corrente }) => {
  const { editor: ed, createShapeId } = window.oracle;
  const def = ed.getCurrentPageId();
  let x = 100;
  for (const p of paginas) {
    ed.createPage({ id: p.id, name: p.nome });
    ed.setCurrentPage(p.id);
    for (const f of p.formas) {
      ed.createShapes([{ id: createShapeId(f), type: "geo", x, y: 100, props: { w: 120, h: 80, geo: "rectangle" } }]);
      x += 160;
    }
  }
  if (!paginas.some((p) => p.id === def)) ed.deletePage(def);
  ed.setCurrentPage(corrente);
  ed.selectNone();
  ed.clearHistory();
  return true;
};

const pageSnap = () => {
  const ed = window.oracle.editor;
  const can = (n) => (typeof ed[`getCan${n}`] === "function" ? ed[`getCan${n}`]() : ed[`can${n}`]());
  return {
    paginas: ed.getPages().map((p) => ({ id: p.id, nome: p.name, index: p.index, formas: [...ed.getPageShapeIds(p.id)].sort() })),
    corrente: ed.getCurrentPageId(),
    canUndo: can("Undo"),
    canRedo: can("Redo"),
  };
};

// Um passo: [op, ...args]. Gesto = marca + chamada; undo/redo sem marca. `nomark:` antes da op = sem marca.
const pageStep = (passo) => {
  const { editor: ed, createShapeId } = window.oracle;
  let [op, ...a] = passo;
  let marca = !["undo", "redo"].includes(op);
  if (op.startsWith("nomark:")) { op = op.slice(7); marca = false; }
  const r = { passo, marca, retorno: null, erro: null };
  try {
    if (marca) r.marcaId = ed.markHistoryStoppingPoint(op);
    let v;
    switch (op) {
      case "createPage": v = ed.createPage(a[0]); break;
      case "renamePage": v = ed.renamePage(a[0], a[1]); break;
      case "updatePage": v = ed.updatePage(a[0]); break;
      case "duplicatePage": v = ed.duplicatePage(a[0]); break;
      case "deletePage": v = ed.deletePage(a[0]); break;
      case "setCurrentPage": v = ed.setCurrentPage(a[0]); break;
      case "createShape": {
        const sid = createShapeId(a[0]);
        v = ed.createShapes([{ id: sid, type: "geo", x: a[1] ?? 400, y: a[2] ?? 300, props: { w: 100, h: 60, geo: "rectangle" } }]);
        break;
      }
      // P4: index ANTES do primeiro (ZERO_INDEX_KEY 'a0' se couber; 'Zz' é válido e menor que 'a0').
      case "moverParaPrimeiro": {
        const first = ed.getPages()[0].index;
        const idx = "a0" < first ? "a0" : "Zz";
        r.indexPedido = idx;
        v = ed.updatePage({ id: a[0], index: idx });
        break;
      }
      case "moverParaUltimo": {
        const pages = ed.getPages();
        const last = pages[pages.length - 1].index;
        const idx = last + "V"; // sufixo fraccionário: maior que `last` e menor que o inteiro seguinte
        r.indexPedido = idx;
        v = ed.updatePage({ id: a[0], index: idx });
        break;
      }
      case "undo": v = ed.undo(); break;
      case "redo": v = ed.redo(); break;
      default: throw new Error(`op desconhecida ${op}`);
    }
    r.retorno = v === ed ? "editor" : v === undefined ? "undefined" : JSON.parse(JSON.stringify(v));
  } catch (e) {
    r.erro = `${e.name}: ${e.message}`;
  }
  return r;
};

// Superfície pública relevante PRESENTE no editor (só nomes de métodos, por `typeof`).
const pageSurface = () => {
  const ed = window.oracle.editor;
  const nomes = ["createPage", "renamePage", "updatePage", "duplicatePage", "deletePage", "setCurrentPage",
    "movePage", "movePages", "reorderPages", "getPages", "getPage", "getCurrentPageId", "getPageShapeIds",
    "markHistoryStoppingPoint", "undo", "redo", "canUndo", "canRedo", "getCanUndo", "getCanRedo", "clearHistory"];
  return Object.fromEntries(nomes.map((n) => [n, typeof ed[n]]));
};

// ---------------------------------------------------------------- casos
const A = "page:A", B = "page:B", C = "page:C";
const pg = (id, nome, formas) => ({ id, nome, formas });
const soA = [pg(A, "A", ["retA"])];
const AB = [pg(A, "A", ["retA"]), pg(B, "B", ["retB"])];
const ABvazia = [pg(A, "A", ["retA"]), pg(B, "B", [])];
const ABC = [pg(A, "A", ["retA"]), pg(B, "B", ["retB"]), pg(C, "C", ["retC"])];

const casos = {
  P1: { pergunta: "criar página B → undo → redo; a corrente muda com createPage? e com undo/redo?",
    cena: { paginas: soA, corrente: A },
    passos: [["createPage", { id: B, name: "B" }], ["undo"], ["redo"]] },
  P1b: { pergunta: "P1 sem id dado (createPage({name})): que id e index recebe?",
    cena: { paginas: soA, corrente: A },
    passos: [["createPage", { name: "B" }], ["undo"], ["redo"]] },
  P2: { pergunta: "renomear A→A2 por updatePage({id,name}) → undo → redo",
    cena: { paginas: soA, corrente: A },
    passos: [["updatePage", { id: A, name: "A2" }], ["undo"], ["redo"]] },
  P2b: { pergunta: "renomear A→A2 por renamePage(id,name) → undo → redo",
    cena: { paginas: soA, corrente: A },
    passos: [["renamePage", A, "A2"], ["undo"], ["redo"]] },
  P3: { pergunta: "duplicar A → undo → redo; a cópia fica corrente?",
    cena: { paginas: soA, corrente: A },
    passos: [["duplicatePage", A], ["undo"], ["redo"]] },
  P4: { pergunta: "A,B,C: mover C para primeiro (updatePage index) → undo → redo",
    cena: { paginas: ABC, corrente: A },
    passos: [["moverParaPrimeiro", C], ["undo"], ["redo"]] },
  P4b: { pergunta: "A,B,C: mover A para último (updatePage index), estando em A → undo → redo",
    cena: { paginas: ABC, corrente: A },
    passos: [["moverParaUltimo", A], ["undo"], ["redo"]] },
  P5: { pergunta: "apagar B estando em B → undo: B volta com as formas? volta a corrente? → redo",
    cena: { paginas: AB, corrente: B },
    passos: [["deletePage", B], ["undo"], ["redo"]] },
  P5b: { pergunta: "A,B,C: apagar B (do MEIO) estando em B → para onde vai a corrente? → undo → redo",
    cena: { paginas: ABC, corrente: B },
    passos: [["deletePage", B], ["undo"], ["redo"]] },
  P6: { pergunta: "apagar B estando em A (não corrente) → undo → corrente? → redo",
    cena: { paginas: AB, corrente: A },
    passos: [["deletePage", B], ["undo"], ["redo"]] },
  P7a: { pergunta: "cruzar páginas (marca em TODO gesto, também na troca de página): forma em A; ir para B; forma em B; voltar a A; undo ×4",
    cena: { paginas: ABvazia, corrente: A },
    passos: [["createShape", "novaA", 400, 300], ["setCurrentPage", B], ["createShape", "novaB", 400, 300], ["setCurrentPage", A],
      ["undo"], ["undo"], ["undo"], ["undo"]] },
  P7b: { pergunta: "P7 com marca SÓ nos gestos que mudam o documento (troca de página sem marca)",
    cena: { paginas: ABvazia, corrente: A },
    passos: [["createShape", "novaA", 400, 300], ["nomark:setCurrentPage", B], ["createShape", "novaB", 400, 300], ["nomark:setCurrentPage", A],
      ["undo"], ["undo"], ["undo"]] },
  P8a: { pergunta: "estando em B: apagar B (vai a A sozinho); ir para A; forma em A; undo ×3 (marca em todo gesto)",
    cena: { paginas: AB, corrente: B },
    passos: [["deletePage", B], ["setCurrentPage", A], ["createShape", "novaA", 400, 300], ["undo"], ["undo"], ["undo"]] },
  P8b: { pergunta: "A,B,C em A: apagar B; ir para C; forma em C; undo ×3 (marca em todo gesto)",
    cena: { paginas: ABC, corrente: A },
    passos: [["deletePage", B], ["setCurrentPage", C], ["createShape", "novaC", 400, 300], ["undo"], ["undo"], ["undo"]] },
  P8c: { pergunta: "P8b sem marca na troca de página",
    cena: { paginas: ABC, corrente: A },
    passos: [["deletePage", B], ["nomark:setCurrentPage", C], ["createShape", "novaC", 400, 300], ["undo"], ["undo"]] },
  P9a: { pergunta: "em A: apagar B; undo (B volta); forma nova na corrente A (sem trocar de página); o redo do apagar ainda existe?",
    cena: { paginas: AB, corrente: A },
    passos: [["deletePage", B], ["undo"], ["createShape", "novaA", 400, 300], ["redo"], ["undo"], ["undo"]] },
  P9b: { pergunta: "em A: apagar B; undo (B volta); ir para B (com marca); redo existe?; forma em B; redo existe?",
    cena: { paginas: AB, corrente: A },
    passos: [["deletePage", B], ["undo"], ["setCurrentPage", B], ["createShape", "novaB", 400, 300], ["redo"], ["undo"], ["undo"]] },
  P9c: { pergunta: "P9b sem marca na troca de página",
    cena: { paginas: AB, corrente: A },
    passos: [["deletePage", B], ["undo"], ["nomark:setCurrentPage", B], ["createShape", "novaB", 400, 300], ["redo"], ["undo"], ["undo"]] },
  P10a: { pergunta: "só trocar de página (com marca): entra no histórico? undo volta de página?",
    cena: { paginas: AB, corrente: A },
    passos: [["setCurrentPage", B], ["undo"], ["redo"]] },
  P10b: { pergunta: "só trocar de página (sem marca)",
    cena: { paginas: AB, corrente: A },
    passos: [["nomark:setCurrentPage", B], ["undo"], ["redo"]] },
  P10c: { pergunta: "trocar de página DEPOIS de um gesto que muda o documento: forma em A; ir para B; undo → a forma sai e a corrente volta a A?; redo",
    cena: { paginas: AB, corrente: A },
    passos: [["createShape", "novaA", 400, 300], ["setCurrentPage", B], ["undo"], ["undo"], ["redo"], ["redo"]] },
  P11: { pergunta: "A,B em A: apagar B (fica 1); tentar apagar A, a última → o tldraw deixa? undo ×2",
    cena: { paginas: AB, corrente: A },
    passos: [["deletePage", B], ["deletePage", A], ["undo"], ["undo"]] },
  P11b: { pergunta: "só A: apagar A (a única)",
    cena: { paginas: soA, corrente: A },
    passos: [["deletePage", A], ["undo"]] },
};

// ---------------------------------------------------------------- corredor (igual ao de corre_guias.mjs)
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
let external, consoleMsgs, context, page;
// Determinismo: Math.random/crypto.getRandomValues semeados pelo sha256 do nome do caso (ids gerados
// por createPage sem id e por duplicatePage).
const seedScript = (seed) => `(() => {
  let a = ${seed} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next;
  const gr = (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength);
    for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; };
  Object.defineProperty(crypto, "getRandomValues", { value: gr });
})();`;
async function abrePagina(nomeCaso) {
  external = new Set();
  consoleMsgs = [];
  context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
  context.on("request", (rq) => { const u = rq.url(); if (!u.startsWith(origin) && !u.startsWith("data:") && !u.startsWith("blob:")) external.add(new URL(u).origin + new URL(u).pathname.split("/").slice(0, 2).join("/")); });
  await context.addInitScript(seedScript(parseInt(createHash("sha256").update(nomeCaso).digest("hex").slice(0, 8), 16)));
  page = await context.newPage();
  page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") consoleMsgs.push(`${m.type()}: ${m.text().slice(0, 300)}`); });
  page.on("pageerror", (e) => consoleMsgs.push(`pageerror: ${e.message}`));
  await page.goto(origin + "/", { timeout: 60000 });
  await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 60000 });
  await page.waitForTimeout(300);
}

let failures = 0;
const escolhidos = so ? so.split(",") : Object.keys(casos);
try {
  for (const nome of escolhidos) {
    const caso = casos[nome];
    const out = {
      cabecalho: {
        oraculo: "tldraw", versao: PKG.version, licenca: PKG.license,
        nota_licenca: "tldraw license (LICENCA_tldraw_v5.5.2.md): uso em Development Environment; corrido localmente como oráculo de medição, nenhum código lido nem copiado",
        chromium: chromiumVersion, playwright: PW.version, data: new Date().toISOString(),
        comando: `node docs/MiroClone/ferramentas/tldraw_oracle/corre_paginas.mjs --so ${nome} <saida>`,
        corredor_sha256: SELF_SHA,
        api: "editor.createPage/renamePage/updatePage/duplicatePage/deletePage/setCurrentPage/getPages/getCurrentPageId/getPageShapeIds/createShapes/markHistoryStoppingPoint/undo/redo/canUndo|getCanUndo/canRedo|getCanRedo/clearHistory",
        idioma: "gesto = markHistoryStoppingPoint(<op>) + chamada; 'nomark:' = chamada sem marca; undo/redo sem marca; cena montada e clearHistory() antes do 1.º passo",
      },
      caso: nome, pergunta: caso.pergunta, cena: caso.cena,
    };
    try {
      await abrePagina(nome);
      out.superficie = await page.evaluate(pageSurface);
      await page.evaluate(pageSetup, caso.cena);
      out.inicio = await page.evaluate(pageSnap);
      out.passos = [];
      for (const p of caso.passos) {
        const c0 = consoleMsgs.length;
        const r = await page.evaluate(pageStep, p);
        await page.waitForTimeout(30);
        r.depois = await page.evaluate(pageSnap);
        const cons = consoleMsgs.slice(c0);
        if (cons.length) r.consola = cons;
        out.passos.push(r);
      }
      out.pedidos_externos = [...external].sort();
      out.consola = [...new Set(consoleMsgs)].sort();
      await context.close();
    } catch (e) {
      console.error(`FALHA ${nome}: ${e.stack}`);
      failures++;
      try { await context.close(); } catch {}
      out.falha = String(e.stack);
    }
    fs.writeFileSync(path.join(outDir, `paginas_${nome}.json`), JSON.stringify(out, null, 2) + "\n");
    console.log(`ok ${nome}: ${out.passos?.length ?? 0} passos`);
  }
} finally {
  await browser.close();
  server.close();
}
process.exit(failures ? 1 : 0);
