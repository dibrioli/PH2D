// Guias inteligentes (snapping) do tldraw, medidas no EDITOR montado. Só API pública documentada:
// editor.createShapes / deleteShapes / select / selectNone / setCamera / setCurrentTool,
// editor.user.updateUserPreferences({ isSnapMode }), editor.snaps.getIndicators(),
// editor.getShapePageBounds / getShapePageTransform / getShape / getSelectionPageBounds,
// editor.getInstanceState().cursor, editor.pageToScreen. O gesto é o RATO REAL do Playwright.
// Uso: node corre_guias.mjs [--so G1,G4] <pasta_saida>
// Por caso: <saida>/guias_<caso>.json (as sondas: cena, gesto, indicadores ANTES de soltar, formas
// DEPOIS) + guias_<caso>.meta.json (versões, data, comando, sha256). Janela 1280×800, dpr 1.
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
const opt = (flag) => {
  const i = args.indexOf(flag);
  if (i < 0) return null;
  const v = args[i + 1];
  args.splice(i, 2);
  return v;
};
const so = opt("--so");
const [outDir] = args;
if (!outDir) {
  console.error("uso: node corre_guias.mjs [--so G1,G2] <pasta_saida>");
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });

// ---------------------------------------------------------------- geometria (lado do Node)
const rad = (d) => (d * Math.PI) / 180;
const deg = (r) => (r * 180) / Math.PI;
const rot = ([x, y], t) => [x * Math.cos(t) - y * Math.sin(t), x * Math.sin(t) + y * Math.cos(t)];
const add = (a, b) => [a[0] + b[0], a[1] + b[1]];
const sub = (a, b) => [a[0] - b[0], a[1] - b[1]];
const r9 = (v) => Math.round(v * 1e9) / 1e9;
const rv = (p) => [r9(p[0]), r9(p[1])];
// Retângulo geo com o CENTRO dado (a rotação é em torno da origem x,y do tldraw).
const geoC = (nome, cx, cy, w, h, rotDeg = 0, extra = {}) => {
  const o = sub([cx, cy], rot([w / 2, h / 2], rad(rotDeg)));
  return geo(nome, o[0], o[1], w, h, rotDeg, extra);
};
const geo = (nome, x, y, w, h, rotDeg = 0, extra = {}) => ({
  nome, type: "geo", x, y, rotation: rad(rotDeg), props: { w, h, geo: "rectangle", fill: "solid" }, ...extra,
});
const frame = (nome, x, y, w, h, rotDeg = 0) => ({ nome, type: "frame", x, y, rotation: rad(rotDeg), props: { w, h, name: nome } });

// ---------------------------------------------------------------- lado da página
const pageSetup = ({ shapes, z, snapMode }) => {
  const { editor, createShapeId } = window.oracle;
  editor.selectNone();
  editor.setCurrentTool("select");
  const all = [...editor.getCurrentPageShapeIds()];
  if (all.length) editor.deleteShapes(all);
  const id = (n) => createShapeId(n);
  editor.createShapes(
    shapes.map(({ nome, parent, ...s }) => ({ ...s, id: id(nome), ...(parent ? { parentId: id(parent) } : {}) })),
  );
  editor.setCamera({ x: 0, y: 0, z });
  editor.user.updateUserPreferences({ isSnapMode: snapMode });
  return true;
};
const pageGeom = (nomes) => {
  const { editor, createShapeId } = window.oracle;
  const out = {};
  for (const n of nomes) {
    const sid = n.startsWith("shape:") ? n : createShapeId(n);
    const s = editor.getShape(sid);
    if (!s) { out[n] = null; continue; }
    const b = editor.getShapePageBounds(sid);
    const m = editor.getShapePageTransform(sid);
    const ap = (x, y) => [m.a * x + m.c * y + m.e, m.b * x + m.d * y + m.f];
    const w = s.props.w, h = s.props.h;
    out[n] = {
      type: s.type, parentId: s.parentId, x: s.x, y: s.y, rotation: s.rotation,
      pageRotationDeg: (Math.atan2(m.b, m.a) * 180) / Math.PI,
      pageBounds: { x: b.x, y: b.y, w: b.w, h: b.h },
      ...(w !== undefined ? { corners: [ap(0, 0), ap(w, 0), ap(w, h), ap(0, h)], center: ap(w / 2, h / 2) } : {}),
    };
  }
  return out;
};
const pageState = () => {
  const { editor } = window.oracle;
  return {
    indicadores: JSON.parse(JSON.stringify(editor.snaps.getIndicators())),
    camera: editor.getCamera(),
    tool: editor.getPath(),
    selecionados: editor.getSelectedShapeIds(),
    cursor: editor.getInstanceState().cursor.type,
  };
};
const pageSelect = (nomes) => {
  const { editor, createShapeId } = window.oracle;
  editor.select(...nomes.map((n) => (n.startsWith("shape:") ? n : createShapeId(n))));
  return editor.getSelectedShapeIds();
};
const pageToScreen = (pts) => pts.map(([x, y]) => { const p = window.oracle.editor.pageToScreen({ x, y }); return [p.x, p.y]; });
const pageSelBounds = () => { const b = window.oracle.editor.getSelectionPageBounds(); return b && { x: b.x, y: b.y, w: b.w, h: b.h }; };
const pageCursor = () => window.oracle.editor.getInstanceState().cursor.type;
const pageTool = (t) => { window.oracle.editor.setCurrentTool(t); return window.oracle.editor.getPath(); };
const pageShapesByType = () => window.oracle.editor.getCurrentPageShapes().map((s) => ({ id: s.id, type: s.type })).sort((a, b) => a.id.localeCompare(b.id));

// ---------------------------------------------------------------- corredor
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
// Pedidos externos (ícones/traduções/fontes da CDN do tldraw) seguem e são REGISTADOS, não bloqueados:
// a licença proíbe interferir com a verificação de chave, e nada medido aqui depende deles (formas sem texto).
let external, consoleMsgs, context, page;
// Determinismo: Math.random e crypto.getRandomValues semeados pelo sha256 do nome do caso (ids dos
// indicadores e das formas criadas pelas ferramentas draw/arrow), uma página nova por caso.
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
  page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") consoleMsgs.push(`${m.type()}: ${m.text().slice(0, 200)}`); });
  page.on("pageerror", (e) => consoleMsgs.push(`pageerror: ${e.message}`));
  await page.goto(origin + "/", { timeout: 60000 });
  await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 60000 });
  await page.waitForTimeout(500);
}

const tick = (ms = 80) => page.waitForTimeout(ms);
const scr = async (p) => (await page.evaluate(pageToScreen, [p]))[0];

// Arrasta `mover` por `delta` (unidades de página) agarrando-o no centro (ou em `grab`). Lê os
// indicadores ANTES de soltar e a geometria depois.
async function drag({ mover, delta, grab, ctrl = false, shift = false, nomes }) {
  await page.evaluate(pageSelect, [mover]);
  const g0 = (await page.evaluate(pageGeom, nomes));
  const m0 = g0[mover];
  const gp = grab || m0.center || [m0.pageBounds.x + m0.pageBounds.w / 2, m0.pageBounds.y + m0.pageBounds.h / 2];
  const from = await scr(gp);
  const to = await scr(add(gp, delta));
  await page.mouse.move(from[0], from[1]);
  await tick(40);
  await page.mouse.down();
  await page.mouse.move(from[0] + Math.sign(to[0] - from[0] || 1) * 6, from[1] + Math.sign(to[1] - from[1] || 1) * 6, { steps: 2 });
  if (ctrl) await page.keyboard.down("Control");
  if (shift) await page.keyboard.down("Shift");
  await page.mouse.move(to[0], to[1], { steps: 10 });
  await tick(300);
  // Medido: se TODOS os movimentos caem no mesmo quadro em que o arrasto começa, a forma chega sem
  // guia (`chegadaRapida`); qualquer movimento num quadro seguinte prende (G8.velocidade: até 40 px
  // num passo de 16 ms prende) — não é a velocidade. Por isso: pausa, 0,5 px e de volta ao alvo exacto.
  const rapido = { indicadores: (await page.evaluate(pageState)).indicadores.length, pageBounds: (await page.evaluate(pageGeom, [mover]))[mover].pageBounds };
  await page.mouse.move(to[0] + 0.5, to[1]);
  await tick(60);
  await page.mouse.move(to[0], to[1]);
  await tick(120);
  const durante = { ...(await page.evaluate(pageState)), chegadaRapida: rapido };
  const gd = await page.evaluate(pageGeom, nomes);
  await page.mouse.up();
  if (shift) await page.keyboard.up("Shift");
  if (ctrl) await page.keyboard.up("Control");
  await tick(120);
  const g1 = await page.evaluate(pageGeom, nomes);
  const b0 = m0.pageBounds, b1 = g1[mover].pageBounds;
  const nudge = [r9(b1.x - b0.x - delta[0]), r9(b1.y - b0.y - delta[1])];
  return { gesto: { mover, deltaPagina: rv(delta), agarrePagina: rv(gp), deEcra: rv(from), paraEcra: rv(to), ctrl, shift }, antes: g0, durante: { ...durante, geom: gd[mover] }, depois: g1, nudge };
}

// Leitura de distâncias (página) entre os traços x de `m` e os de cada alvo, para a sonda registar
// porque é que ela é discriminante (o que mais está dentro do limiar).
const featsX = (g) => {
  const b = g.pageBounds;
  const f = { aabbL: b.x, aabbC: b.x + b.w / 2, aabbR: b.x + b.w };
  if (g.corners) g.corners.forEach((c, i) => (f[`v${i}`] = c[0]));
  return f;
};
const featsY = (g) => {
  const b = g.pageBounds;
  const f = { aabbT: b.y, aabbM: b.y + b.h / 2, aabbB: b.y + b.h };
  if (g.corners) g.corners.forEach((c, i) => (f[`v${i}`] = c[1]));
  return f;
};
function proximos(geomMovedAt, alvos, axis, lim = 8.5) {
  const F = axis === "x" ? featsX : featsY;
  const fm = F(geomMovedAt);
  const out = [];
  for (const [n, g] of Object.entries(alvos)) {
    if (!g || !g.pageBounds) continue;
    const ft = F(g);
    for (const [km, vm] of Object.entries(fm)) for (const [kt, vt] of Object.entries(ft)) {
      const d = vt - vm;
      if (Math.abs(d) <= lim) out.push({ movedor: km, alvo: `${n}.${kt}`, d: r9(d) });
    }
  }
  return out.sort((a, b) => Math.abs(a.d) - Math.abs(b.d) || a.alvo.localeCompare(b.alvo));
}
const shiftGeom = (g, d) => ({
  ...g,
  pageBounds: { ...g.pageBounds, x: g.pageBounds.x + d[0], y: g.pageBounds.y + d[1] },
  ...(g.corners ? { corners: g.corners.map((c) => add(c, d)), center: add(g.center, d) } : {}),
});
// Classifica cada indicador de pontos: horizontal, vertical ou oblíquo.
const classifica = (inds) => inds.map((i) => {
  if (i.type !== "points") return { type: i.type, direction: i.direction, n: i.gaps?.length };
  const ps = i.points;
  const xs = ps.map((p) => p.x), ys = ps.map((p) => p.y);
  const dx = Math.max(...xs) - Math.min(...xs), dy = Math.max(...ys) - Math.min(...ys);
  return { type: "points", n: ps.length, linha: dx < 1e-6 ? "vertical" : dy < 1e-6 ? "horizontal" : "obliqua", dx: r9(dx), dy: r9(dy) };
});

async function sonda({ nome, cena, z = 1, snapMode = true, mover, alvoDelta, nomes, ctrl, shift, grab, pre }) {
  await page.evaluate(pageSetup, { shapes: cena, z, snapMode });
  await tick(60);
  if (pre) await pre();
  const nm = nomes || cena.map((s) => s.nome);
  const g0 = await page.evaluate(pageGeom, nm);
  const { delta, descr } = alvoDelta(g0);
  const alvos = Object.fromEntries(Object.entries(g0).filter(([k]) => k !== mover));
  const prev = shiftGeom(g0[mover], delta);
  const r = await drag({ mover, delta, ctrl, shift, grab: grab && grab(g0), nomes: nm });
  return {
    nome, descr, z, snapMode, cena,
    previsto: { semGuia: prev.pageBounds, proximosX: proximos(prev, alvos, "x"), proximosY: proximos(prev, alvos, "y") },
    ...r,
    indicadoresClass: classifica(r.durante.indicadores),
  };
}

// ---------------------------------------------------------------- casos
const casos = {};
const A0 = () => geo("A", 100, 100, 200, 120, 0);

casos.G1 = {
  pergunta: "Alvo A 200×120 a 0°; movedor B 160×100 rodado: guia pela AABB de B ou pelos cantos RODADOS? (a) AABB-esq de B a 3 px da esq de A; (b) um canto rodado INTERIOR de B a 3 px da esq de A com a AABB longe; (c) centros a 3 px.",
  sondas: async () => {
    const out = [];
    for (const t of [15, 30, 45, 37]) {
      const cena = [A0(), geoC("B", 600, 450, 160, 100, t)];
      out.push(await sonda({ nome: `t${t}_a_aabbL`, cena, mover: "B", alvoDelta: (g) => ({ delta: [g.A.pageBounds.x + 3 - g.B.pageBounds.x, 0], descr: "B.aabb.left → A.left + 3" }) }));
      for (const qual of ["topo", "base"]) {
        out.push(await sonda({
          nome: `t${t}_b_canto_${qual}`, cena, mover: "B",
          alvoDelta: (g) => {
            const cs = g.B.corners;
            const c = cs.reduce((m, p) => ((qual === "topo" ? p[1] < m[1] : p[1] > m[1]) ? p : m));
            return { delta: [g.A.pageBounds.x + 3 - c[0], 0], descr: `vértice rodado de ${qual} de B (x=${r9(c[0])}) → A.left + 3` };
          },
        }));
      }
      out.push(await sonda({ nome: `t${t}_c_centros`, cena, mover: "B", alvoDelta: (g) => ({ delta: [g.A.center[0] + 3 - g.B.center[0], 0], descr: "B.centro.x → A.centro.x + 3" }) }));
    }
    return out;
  },
};

casos.G2 = {
  pergunta: "Alvo A 200×120 rodado 30°; movedor B 160×100 a 0°: o alvo oferece a AABB ou os vértices rodados?",
  sondas: async () => {
    const out = [];
    const A = geo("A", 300, 150, 200, 120, 30);
    const cx = [A, geoC("B", 700, 550, 160, 100, 0)];
    const vx = (g, i) => g.A.corners[i][0];
    out.push(await sonda({ nome: "x_aabbL", cena: cx, mover: "B", alvoDelta: (g) => ({ delta: [g.A.pageBounds.x + 3 - g.B.pageBounds.x, 0], descr: "B.left → A.aabb.left + 3 (= vértice v3)" }) }));
    out.push(await sonda({ nome: "x_vertice_topo_v0", cena: cx, mover: "B", alvoDelta: (g) => ({ delta: [vx(g, 0) + 3 - g.B.pageBounds.x, 0], descr: "B.left → x do vértice v0 (topo, interior à AABB) + 3" }) }));
    out.push(await sonda({ nome: "x_vertice_base_v2", cena: cx, mover: "B", alvoDelta: (g) => ({ delta: [vx(g, 2) + 3 - g.B.pageBounds.x, 0], descr: "B.left → x do vértice v2 (base, interior) + 3" }) }));
    out.push(await sonda({ nome: "x_aabbR", cena: cx, mover: "B", alvoDelta: (g) => ({ delta: [g.A.pageBounds.x + g.A.pageBounds.w + 3 - g.B.pageBounds.x, 0], descr: "B.left → A.aabb.right + 3 (= vértice v1)" }) }));
    const cy = [A, geoC("B", 960, 600, 160, 100, 0)];
    out.push(await sonda({ nome: "y_vertice_v1_menos3", cena: cy, mover: "B", alvoDelta: (g) => ({ delta: [0, g.A.corners[1][1] - 3 - g.B.pageBounds.y], descr: "B.top → y do vértice v1 (interior) − 3" }) }));
    out.push(await sonda({ nome: "y_aabbT", cena: cy, mover: "B", alvoDelta: (g) => ({ delta: [0, g.A.pageBounds.y + 3 - g.B.pageBounds.y], descr: "B.top → A.aabb.top + 3" }) }));
    return out;
  },
};

casos.G3 = {
  pergunta: "A e B ambos a 30°: há indicadores oblíquos (paralelos às arestas rodadas)?",
  sondas: async () => {
    const out = [];
    const t = rad(30);
    const u = [Math.cos(t), Math.sin(t)], n = [-Math.sin(t), Math.cos(t)];
    const A = geo("A", 400, 200, 200, 120, 30);
    // B (160×100) com a aresta de BAIXO paralela à aresta de cima de A, 3 px para fora, deslizada 20 px.
    const target = add(add([400, 200], [-(100 + 3) * n[0], -(100 + 3) * n[1]]), [20 * u[0], 20 * u[1]]);
    const B = geo("B", target[0] + 350, target[1] + 300, 160, 100, 30);
    out.push(await sonda({ nome: "aresta_paralela_3px", cena: [A, B], mover: "B", alvoDelta: () => ({ delta: [-350, -300], descr: "aresta de baixo de B paralela à de cima de A, a 3 px ao longo da normal, deslizada 20 px" }) }));
    // Canto contra canto: o vértice v3 (0,h) de B a (3, −3) do vértice v0 de A.
    out.push(await sonda({
      nome: "canto_canto_3_3", cena: [A, geo("B", 800, 500, 160, 100, 30)], mover: "B",
      alvoDelta: (g) => ({ delta: sub(add(g.A.corners[0], [3, -3]), g.B.corners[3]), descr: "vértice v3 de B → vértice v0 de A + (3, −3)" }),
    }));
    // E colinear: B deslizado AO LONGO da aresta de A (as mesmas linhas rodadas), 3 px fora da linha.
    const target2 = add(add([400, 200], [-(100 + 3) * n[0], -(100 + 3) * n[1]]), [0, 0]);
    out.push(await sonda({ nome: "aresta_paralela_cantos_alinhados", cena: [A, geo("B", target2[0] + 350, target2[1] + 300, 160, 100, 30)], mover: "B", alvoDelta: () => ({ delta: [-350, -300], descr: "como a 1.ª mas sem deslizar (vértice v3 de B sobre a normal de v0 de A, a 3 px)" }) }));
    return out;
  },
};

casos.G4 = {
  pergunta: "Rodar pelo puxador de rotação: B a 0° perto de A a 30°; rodar a ~28°, ~2°, etc. Prende ao ângulo de outras formas / a 0? Com Shift: o passo.",
  sondas: async () => {
    const out = [];
    const cena = [geo("A", 200, 150, 200, 120, 30), geo("B", 640, 350, 160, 100, 0)];
    const plano = [
      ...[28, 29.5, 29.7, 30.3, 31, 32, 2, 1.4, 0.6, 0.5, 0.4, -1, 10.3, 10.7, 17.25, 44, 46, 89].map((a) => ({ a, shift: false })),
      ...[8, 20, 37, 50, 97].map((a) => ({ a, shift: true })),
    ];
    for (const { a, shift } of plano) {
      await page.evaluate(pageSetup, { shapes: cena, z: 1, snapMode: true });
      await tick(60);
      await page.evaluate(pageSelect, ["B"]);
      await tick(60);
      const sb = await page.evaluate(pageSelBounds);
      const c = await scr([sb.x + sb.w / 2, sb.y + sb.h / 2]);
      const tr = await scr([sb.x + sb.w, sb.y]);
      // Procura o puxador: anda na diagonal para fora do canto superior direito e lê o cursor.
      let handle = null;
      const visto = [];
      for (let k = 0; k <= 30 && !handle; k++) {
        for (const [dx, dy] of [[k, -k], [k, 0], [0, -k]]) {
          await page.mouse.move(tr[0] + dx, tr[1] + dy);
          await tick(30);
          const cur = await page.evaluate(pageCursor);
          visto.push([dx, dy, cur]);
          if (cur.includes("rotate")) { handle = [tr[0] + dx, tr[1] + dy, cur]; break; }
        }
      }
      if (!handle) { out.push({ nome: `rot_${a}${shift ? "_shift" : ""}`, erro: "puxador de rotação não encontrado pelo cursor", visto }); continue; }
      const v0 = [handle[0] - c[0], handle[1] - c[1]];
      await page.mouse.down();
      if (shift) await page.keyboard.down("Shift");
      const N = 12;
      for (let i = 1; i <= N; i++) {
        const p = add(c, rot(v0, rad((a * i) / N)));
        await page.mouse.move(p[0], p[1]);
        await tick(20);
      }
      await tick(300);
      const pf = add(c, rot(v0, rad(a)));
      const rapidoDeg = r9(deg((await page.evaluate(pageGeom, ["B"])).B.rotation));
      await page.mouse.move(pf[0] + 0.5, pf[1]);
      await tick(60);
      await page.mouse.move(pf[0], pf[1]);
      await tick(120);
      const durante = { ...(await page.evaluate(pageState)), chegadaRapidaRotacaoDeg: rapidoDeg };
      const gd = await page.evaluate(pageGeom, ["A", "B"]);
      await page.mouse.up();
      if (shift) await page.keyboard.up("Shift");
      await tick(120);
      const g1 = await page.evaluate(pageGeom, ["A", "B"]);
      out.push({
        nome: `rot_${a}${shift ? "_shift" : ""}`, descr: `arrastar o puxador ${handle[2]} em arco ${a}° em torno do centro da selecção${shift ? " com Shift" : ""}`,
        cena, puxadorEcra: rv(handle), centroEcra: rv(c), anguloPedidoDeg: a, shift,
        durante: { ...durante, rotacaoDeg: r9(deg(gd.B.rotation)) },
        depois: { rotacaoDeg: r9(deg(g1.B.rotation)), geom: g1.B },
        indicadoresClass: classifica(durante.indicadores),
      });
    }
    return out;
  },
};

casos.G5 = {
  pergunta: "Espaços iguais: A e C com espaço 40; B arrastado para espaço 43 / 37 / 48; e B entre dois com espaços desiguais (centrar).",
  sondas: async () => {
    const out = [];
    const A = geo("A", 300, 300, 100, 80), C = geo("C", 440, 300, 100, 80);
    const B = geo("B", 700, 600, 100, 80);
    const cena = [A, C, B];
    const toLeft = (x, y = 310) => (g) => ({ delta: [x - g.B.pageBounds.x, y - g.B.pageBounds.y], descr: `B.left → ${x}, B.top → ${y}` });
    out.push(await sonda({ nome: "dup_direita_43", cena, mover: "B", alvoDelta: toLeft(540 + 43) }));
    out.push(await sonda({ nome: "dup_direita_37", cena, mover: "B", alvoDelta: toLeft(540 + 37) }));
    out.push(await sonda({ nome: "dup_direita_48", cena, mover: "B", alvoDelta: toLeft(540 + 48) }));
    out.push(await sonda({ nome: "dup_esquerda_43", cena, mover: "B", alvoDelta: toLeft(300 - 43 - 100) }));
    const A2 = geo("A", 300, 300, 100, 80), C2 = geo("C", 600, 300, 100, 80);
    out.push(await sonda({ nome: "centrar_53_47", cena: [A2, C2, B], mover: "B", alvoDelta: toLeft(400 + 53) }));
    out.push(await sonda({ nome: "centrar_57_43", cena: [A2, C2, B], mover: "B", alvoDelta: toLeft(400 + 57) }));
    // Vertical: A e C empilhados com 40; B por baixo a 43.
    const Av = geo("A", 300, 100, 100, 80), Cv = geo("C", 300, 220, 100, 80);
    out.push(await sonda({ nome: "dup_vertical_43", cena: [Av, Cv, geo("B", 800, 600, 100, 80)], mover: "B", alvoDelta: (g) => ({ delta: [312 - g.B.pageBounds.x, 300 + 43 - g.B.pageBounds.y], descr: "B.top → 343 (espaço 43 sob C), B.left → 312" }) }));
    // Três em fila, B com espaço 43 MAS rodado 30° (a AABB conta?).
    out.push(await sonda({ nome: "dup_direita_43_B_rodado30", cena: [A, C, geoC("B", 760, 640, 100, 80, 30)], mover: "B", alvoDelta: (g) => ({ delta: [583 - g.B.pageBounds.x, 340 - (g.B.pageBounds.y + g.B.pageBounds.h / 2)], descr: "B.aabb.left → 583 (espaço AABB 43), centro y 340" }) }));
    return out;
  },
};

casos.G6 = {
  pergunta: "Molduras: a moldura é alvo? arrastar a moldura; arrastar um FILHO perto do irmão e de formas de fora; moldura RODADA: guias nos eixos locais ou da página?",
  sondas: async () => {
    const out = [];
    const F = frame("F", 100, 100, 400, 300);
    const c1 = geo("c1", 40, 60, 80, 60, 0, { parent: "F" });
    const c2 = geo("c2", 260, 170, 100, 60, 0, { parent: "F" });
    const O = geo("O", 700, 500, 120, 80), O2 = geo("O2", 230, 600, 100, 60);
    const R = geo("R", 900, 150, 80, 60);
    const cena = [F, c1, c2, O, O2, R];
    out.push(await sonda({ nome: "livre_R_esq_a_dir_da_moldura", cena, mover: "R", alvoDelta: (g) => ({ delta: [g.F.pageBounds.x + g.F.pageBounds.w + 3 - g.R.pageBounds.x, 0], descr: "R.left → F.right + 3 (R fora, à direita)" }) }));
    out.push(await sonda({ nome: "livre_R_alinha_esq_da_moldura", cena, mover: "R", alvoDelta: (g) => ({ delta: [g.F.pageBounds.x + 3 - g.R.pageBounds.x, 450 - g.R.pageBounds.y], descr: "R.left → F.left + 3, R por baixo da moldura (top 450)" }) }));
    out.push(await sonda({ nome: "arrastar_moldura", cena, mover: "F", grab: (g) => [g.F.pageBounds.x + 200, g.F.pageBounds.y + 30], alvoDelta: (g) => ({ delta: [g.O.pageBounds.x + g.O.pageBounds.w + 3 - (g.F.pageBounds.x + g.F.pageBounds.w), 0], descr: "F.right → O.right + 3 (agarrada no interior vazio, F seleccionada antes)" }) }));
    out.push(await sonda({ nome: "filho_c1_topo_a_irmao", cena, mover: "c1", alvoDelta: (g) => ({ delta: [0, g.c2.pageBounds.y + 3 - g.c1.pageBounds.y], descr: "c1.top → c2.top + 3 (dentro da moldura)" }) }));
    out.push(await sonda({ nome: "filho_c1_esq_a_forma_de_fora", cena, mover: "c1", alvoDelta: (g) => ({ delta: [g.O2.pageBounds.x + 3 - g.c1.pageBounds.x, 0], descr: "c1.left → O2.left + 3 (O2 fora da moldura)" }) }));
    out.push(await sonda({ nome: "filho_c1_esq_a_moldura_mae", cena, mover: "c1", alvoDelta: (g) => ({ delta: [g.F.pageBounds.x + 3 - g.c1.pageBounds.x, 0], descr: "c1.left → F.left + 3 (a própria mãe)" }) }));
    // Moldura rodada 30° com dois filhos.
    const Fr = frame("Fr", 300, 120, 400, 300, 30);
    const d1 = geo("d1", 40, 60, 80, 60, 0, { parent: "Fr" });
    const d2 = geo("d2", 240, 170, 100, 60, 0, { parent: "Fr" });
    const cr = [Fr, d1, d2];
    const t = rad(30);
    out.push(await sonda({ nome: "rodada_d1_topo_local_a_d2", cena: cr, mover: "d1", alvoDelta: () => ({ delta: rot([0, 113], t), descr: "d1 deslocado (0, 113) LOCAL: topo local 173 contra o topo local 170 de d2" }) }));
    out.push(await sonda({ nome: "rodada_d1_esq_local_a_d2", cena: cr, mover: "d1", alvoDelta: () => ({ delta: rot([203, 0], t), descr: "d1 deslocado (203, 0) LOCAL: esquerda local 243 contra 240 de d2" }) }));
    out.push(await sonda({ nome: "rodada_d1_aabbL_pagina_a_d2", cena: cr, mover: "d1", alvoDelta: (g) => ({ delta: [g.d2.pageBounds.x + 3 - g.d1.pageBounds.x, 0], descr: "d1.aabb.left (página) → d2.aabb.left + 3" }) }));
    return out;
  },
};

casos.G7 = {
  pergunta: "Setas e traços livres (draw) são alvo das guias?",
  sondas: async () => {
    const out = [];
    const B = geo("B", 800, 600, 100, 60);
    let ids = null;
    const pre = async () => {
      await page.evaluate(pageTool, "draw");
      const pts = [[180, 150], [230, 170], [280, 160], [330, 190], [380, 200]];
      let p = await scr(pts[0]);
      await page.mouse.move(p[0], p[1]);
      await page.mouse.down();
      for (const q of pts.slice(1)) { p = await scr(q); await page.mouse.move(p[0], p[1], { steps: 5 }); await tick(20); }
      await page.mouse.up();
      await tick(80);
      await page.evaluate(pageTool, "arrow");
      p = await scr([450, 350]);
      await page.mouse.move(p[0], p[1]);
      await page.mouse.down();
      p = await scr([650, 420]);
      await page.mouse.move(p[0], p[1], { steps: 10 });
      await page.mouse.up();
      await tick(80);
      await page.keyboard.press("Escape");
      await page.evaluate(pageTool, "select");
      await page.evaluate(() => { window.oracle.editor.selectNone(); return true; });
      const s = await page.evaluate(pageShapesByType);
      ids = { draw: s.find((x) => x.type === "draw")?.id, arrow: s.find((x) => x.type === "arrow")?.id };
    };
    const probes = [
      ["draw_esq", (g, d) => ({ delta: [d.pageBounds.x + 3 - g.B.pageBounds.x, 0], descr: "B.left → draw.aabb.left + 3" }), "draw"],
      ["draw_dir", (g, d) => ({ delta: [d.pageBounds.x + d.pageBounds.w + 3 - g.B.pageBounds.x, 0], descr: "B.left → draw.aabb.right + 3" }), "draw"],
      ["seta_esq", (g, d) => ({ delta: [d.pageBounds.x + 3 - g.B.pageBounds.x, 0], descr: "B.left → arrow.aabb.left + 3" }), "arrow"],
      ["seta_dir", (g, d) => ({ delta: [d.pageBounds.x + d.pageBounds.w + 3 - g.B.pageBounds.x, 0], descr: "B.left → arrow.aabb.right + 3" }), "arrow"],
      ["seta_centro", (g, d) => ({ delta: [d.pageBounds.x + d.pageBounds.w / 2 + 3 - (g.B.pageBounds.x + 50), 0], descr: "B.centro.x → arrow.aabb.centro + 3" }), "arrow"],
    ];
    for (const [nome, f, alvo] of probes) {
      await page.evaluate(pageSetup, { shapes: [B], z: 1, snapMode: true });
      await tick(60);
      await pre();
      const nm = ["B", ids.draw, ids.arrow].filter(Boolean);
      const g0 = await page.evaluate(pageGeom, nm);
      const { delta, descr } = f(g0, g0[ids[alvo]]);
      const prev = shiftGeom(g0.B, delta);
      const alvos = Object.fromEntries(Object.entries(g0).filter(([k]) => k !== "B"));
      const r = await drag({ mover: "B", delta, nomes: nm });
      out.push({ nome, descr, idsCriados: ids, previsto: { semGuia: prev.pageBounds, proximosX: proximos(prev, alvos, "x"), proximosY: proximos(prev, alvos, "y") }, ...r, indicadoresClass: classifica(r.durante.indicadores) });
    }
    return out;
  },
};

casos.G8 = {
  pergunta: "Limiar da guia em px de ECRÃ a zoom 1 e 2 (varrimento 0,5..15 px); e os modos: isSnapMode × Ctrl.",
  sondas: async () => {
    const out = [];
    const cena = [geo("A", 60, 60, 100, 60), geo("B", 300, 260, 130, 50)];
    for (const z of [1, 2]) {
      for (let o2 = 1; o2 <= 30; o2++) {
        const o = o2 / 2;
        const s = await sonda({ nome: `z${z}_o${o}`, cena, z, mover: "B", alvoDelta: (g) => ({ delta: [g.A.pageBounds.x + o / z - g.B.pageBounds.x, 0], descr: `B.left → A.left + ${o} px de ecrã (${o / z} de página)` }) });
        out.push({ nome: s.nome, z, ofsEcraPx: o, ofsPagina: o / z, nudge: s.nudge, prendeu: Math.abs(s.nudge[0] + o / z) < 1e-6, nIndicadores: s.durante.indicadores.length, camera: s.durante.camera, proximosX: s.previsto.proximosX });
      }
    }
    for (const snapMode of [true, false]) for (const ctrl of [false, true]) {
      const s = await sonda({ nome: `modo_snap${snapMode}_ctrl${ctrl}`, cena, snapMode, ctrl, mover: "B", alvoDelta: (g) => ({ delta: [g.A.pageBounds.x + 3 - g.B.pageBounds.x, 0], descr: "B.left → A.left + 3" }) });
      out.push({ nome: s.nome, isSnapMode: snapMode, ctrl, nudge: s.nudge, prendeu: Math.abs(s.nudge[0] + 3) < 1e-6, indicadores: s.durante.indicadores });
    }
    // Velocidade: a aproximação final de 40 px de ecrã a v px por passo, 16 ms entre passos; lê-se
    // 20 ms depois de chegar (sem mais movimento) e de novo 400 ms depois (parado).
    for (const v of [0.5, 1, 2, 3, 4, 6, 8, 12, 20, 40]) {
      await page.evaluate(pageSetup, { shapes: cena, z: 1, snapMode: true });
      await tick(60);
      await page.evaluate(pageSelect, ["B"]);
      const g0 = await page.evaluate(pageGeom, ["A", "B"]);
      const from = await scr(g0.B.center);
      const dx = g0.A.pageBounds.x + 3 - g0.B.pageBounds.x;
      const to = [from[0] + dx, from[1]];
      await page.mouse.move(from[0], from[1]);
      await tick(40);
      await page.mouse.down();
      await page.mouse.move(from[0] - 6, from[1], { steps: 2 });
      await page.mouse.move(to[0] + 40, to[1], { steps: 10 });
      await tick(300);
      const n = Math.round(40 / v);
      for (let i = 1; i <= n; i++) { await page.mouse.move(to[0] + 40 - (40 * i) / n, to[1]); await tick(16); }
      await tick(4);
      const ga = (await page.evaluate(pageGeom, ["B"])).B.pageBounds.x;
      const ia = (await page.evaluate(pageState)).indicadores.length;
      await tick(400);
      const gb = (await page.evaluate(pageGeom, ["B"])).B.pageBounds.x;
      await page.mouse.up();
      await tick(120);
      const gc = (await page.evaluate(pageGeom, ["B"])).B.pageBounds.x;
      const alvoPreso = g0.A.pageBounds.x;
      out.push({ nome: `velocidade_${v}px_por_passo`, pxPorPasso16ms: v, passos: n, prendeuAoChegar: Math.abs(ga - alvoPreso) < 1e-6, indicadoresAoChegar: ia, prendeuParado400ms: Math.abs(gb - alvoPreso) < 1e-6, prendeuAoSoltar: Math.abs(gc - alvoPreso) < 1e-6, xAoChegar: r9(ga), xDepoisDeSoltar: r9(gc) });
    }
    return out;
  },
};

let failures = 0;
const escolhidos = so ? so.split(",") : Object.keys(casos);
try {
  for (const nome of escolhidos) {
    const caso = casos[nome];
    let sondas;
    try {
      await abrePagina(nome);
      sondas = await caso.sondas();
      await context.close();
    } catch (e) {
      console.error(`FALHA ${nome}: ${e.stack}`);
      failures++;
      try { await context.close(); } catch {}
      continue;
    }
    const base = path.join(outDir, `guias_${nome}`);
    const body = JSON.stringify({ caso: nome, pergunta: caso.pergunta, sondas }, null, 2) + "\n";
    fs.writeFileSync(base + ".json", body);
    const meta = {
      oracle: "tldraw",
      package_version: PKG.version,
      package_license: PKG.license,
      license_note: "tldraw license (LICENCA_tldraw_v5.5.2.md): uso em Development Environment permitido; corrido localmente como oráculo de medição, nenhum código copiado",
      chromium_version: chromiumVersion,
      playwright_version: PW.version,
      date: new Date().toISOString(),
      command: `node docs/MiroClone/ferramentas/tldraw_oracle/corre_guias.mjs --so ${nome} <saida>`,
      runner_sha256: SELF_SHA,
      inputs: "as cenas e os gestos de cada sonda estão no próprio .json (campos cena, gesto, descr)",
      measured: "editor.snaps.getIndicators() com o botão do rato ainda em baixo (durante.indicadores); geometria das formas depois de soltar (depois); nudge = deslocamento real − deslocamento pedido (unidades de página)",
      editor: "<Tldraw hideUi onMount> montado; câmara {x:0,y:0,z}; isSnapMode por sonda; rato/teclado reais do Playwright; janela 1280×800, deviceScaleFactor 1",
      external_requests_seen: [...external].sort(),
      console: [...new Set(consoleMsgs)].sort(),
      outputs_sha256: { [path.basename(base + ".json")]: createHash("sha256").update(body).digest("hex") },
    };
    fs.writeFileSync(base + ".meta.json", JSON.stringify(meta, null, 2) + "\n");
    console.log(`ok ${nome}: ${sondas.length} sondas`);
  }
} finally {
  await browser.close();
  server.close();
}
process.exit(failures ? 1 : 0);
