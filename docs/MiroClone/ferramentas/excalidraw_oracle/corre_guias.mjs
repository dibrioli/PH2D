// W5b — guias e encaixe a objectos (objectsSnapModeEnabled) no EDITOR montado: rodados, molduras, vãos,
// lineares, rotação pela pega e a tolerância em px de ecrã.
// Uso: node corre_guias.mjs [--so <caso>] [--plano] <guias_casos.json> <pasta_saida>
// Cada SONDA monta a cena de novo (zoom do caso, scroll 0, janela 1280×800), liga/desliga o modo pelo
// updateScene público, arrasta com o rato do Playwright (desce no elemento, 2 px, 12 passos até ao
// alvo) e lê o appState.snapLines ANTES de largar e a cena DEPOIS. O alvo é calculado no Node a partir
// da cena NOSSA: «a feição `de` do móvel fica a `mais` px de ECRÃ da feição `a` do alvo».
// Feições: aabb.l|c|r (x) · aabb.t|m|b (y) · cantos.k (k-ésimo canto rodado por ordem no eixo; 0 = AABB)
// · ponto.k (vértice k de uma linha/traço). `centro_em: [x,y]` põe o centro da AABB do móvel num ponto.
// --plano: só imprime o plano de cada sonda e as feições próximas (sem navegador).
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle");
const FIXED_TIME = Date.UTC(2026, 0, 1, 0, 0, 0);

let args = process.argv.slice(2);
const opt = (flag, hasValue = true) => {
  const i = args.indexOf(flag);
  if (i < 0) return null;
  const v = hasValue ? args[i + 1] : true;
  args.splice(i, hasValue ? 2 : 1);
  return v;
};
const so = opt("--so");
const soPlano = opt("--plano", false);
const [casosPath, outDir] = args;
if (!casosPath || !outDir) {
  console.error("uso: node corre_guias.mjs [--so <caso>] [--plano] <guias_casos.json> <pasta_saida>");
  process.exit(2);
}
const casosBytes = fs.readFileSync(casosPath);
const casosSha = createHash("sha256").update(casosBytes).digest("hex");
const { cenas, casos } = JSON.parse(casosBytes.toString("utf8"));
const r6 = (v) => Math.round(v * 1e6) / 1e6;

// ---- geometria NOSSA (Node): AABB, cantos rodados, vértices ----
const isLinear = (e) => Array.isArray(e.points);
const pontos = (e) => {
  if (isLinear(e)) return e.points.map(([px, py]) => [e.x + px, e.y + py]);
  const cx = e.x + e.width / 2, cy = e.y + e.height / 2, c = Math.cos(e.angle || 0), s = Math.sin(e.angle || 0);
  return [[-1, -1], [1, -1], [1, 1], [-1, 1]].map(([sx, sy]) => {
    const dx = (sx * e.width) / 2, dy = (sy * e.height) / 2;
    return [cx + dx * c - dy * s, cy + dx * s + dy * c];
  });
};
const aabb = (e) => {
  const p = pontos(e), xs = p.map((q) => q[0]), ys = p.map((q) => q[1]);
  const l = Math.min(...xs), r = Math.max(...xs), t = Math.min(...ys), b = Math.max(...ys);
  return { l, c: (l + r) / 2, r, t, m: (t + b) / 2, b };
};
const AX = { x: { l: "l", c: "c", r: "r", t: "l", m: "c", b: "r" }, y: { l: "t", c: "m", r: "b", t: "t", m: "m", b: "b" } };
const feicao = (e, nome, eixo) => {
  const [kind, k] = nome.split(".");
  const i = eixo === "x" ? 0 : 1;
  if (kind === "aabb") return aabb(e)[AX[eixo][k]];
  if (kind === "cantos") return pontos(e).map((q) => q[i]).sort((a, b) => a - b)[Number(k)];
  if (kind === "ponto") return pontos(e)[Number(k)][i];
  throw new Error(`feição desconhecida ${nome}`);
};
const valor = (els, ref, eixo) => {
  if (typeof ref === "number") return ref;
  const j = ref.indexOf(".");
  const e = els.find((q) => q.id === ref.slice(0, j));
  if (!e) throw new Error(`alvo ${ref} não existe`);
  return feicao(e, ref.slice(j + 1), eixo);
};
const resumo = (e) => ({
  id: e.id, type: e.type, x: e.x, y: e.y, width: e.width, height: e.height,
  angle: e.angle, angulo_graus: r6(((e.angle || 0) * 180) / Math.PI), frameId: e.frameId ?? null,
  aabb: Object.fromEntries(Object.entries(aabb(e)).map(([k, v]) => [k, r6(v)])),
  cantos: pontos(e).map((q) => q.map(r6)),
});
// Todas as feições de eixo de um elemento (AABB + cantos/vértices), para listar candidatos próximos.
const feicoes = (e) => {
  const a = aabb(e), p = pontos(e);
  return {
    x: [["aabb.l", a.l], ["aabb.c", a.c], ["aabb.r", a.r], ...p.map((q, i) => [`${isLinear(e) ? "ponto" : "canto"}${i}`, q[0]])],
    y: [["aabb.t", a.t], ["aabb.m", a.m], ["aabb.b", a.b], ...p.map((q, i) => [`${isLinear(e) ? "ponto" : "canto"}${i}`, q[1]])],
  };
};
const moved = (e, dx, dy) => ({ ...e, x: e.x + dx, y: e.y + dy });

// Plano de uma sonda de arrasto: o delta em CENA que o ponteiro dá (arredondado a px de ecrã).
const planear = (els, s, zoom, ecraDe) => {
  const m = els.find((q) => q.id === s.mover);
  let dx = 0, dy = 0;
  const alvo = {};
  if (s.centro_em) {
    const a = aabb(m);
    dx = s.centro_em[0] - a.c; dy = s.centro_em[1] - a.m;
  }
  for (const eixo of ["x", "y"]) {
    const c = s[eixo];
    if (!c) continue;
    const T = valor(els, c.a, eixo);
    const f0 = feicao(m, c.de, eixo);
    const d = T + c.mais / zoom - f0;
    if (eixo === "x") dx = d; else dy = d;
    alvo[eixo] = { de: c.de, a: c.a, valor_alvo: r6(T), mais_px_ecra: c.mais };
  }
  let grab;
  if (s.pega === "titulo") grab = { cena: [m.x + 8, m.y], ecra_dy: -10 };
  else if (Array.isArray(s.pega)) grab = { cena: s.pega, ecra_dy: 0 };
  else { const a = aabb(m); grab = { cena: [a.c, a.m], ecra_dy: 0 }; }
  const g = ecraDe(grab.cena);
  const p0 = [Math.round(g[0]), Math.round(g[1] + grab.ecra_dy)];
  const p1 = [p0[0] + Math.round(dx * zoom), p0[1] + Math.round(dy * zoom)];
  const ddx = (p1[0] - p0[0]) / zoom, ddy = (p1[1] - p0[1]) / zoom;
  // Feições do móvel na posição PLANEADA perto (≤ 20 px de ecrã) de feições dos outros.
  const mp = moved(m, ddx, ddy), fm = feicoes(mp), perto = [];
  for (const o of els) {
    if (o.id === m.id || (m.type === "frame" && o.frameId === m.id)) continue;
    const fo = feicoes(o);
    for (const eixo of ["x", "y"])
      for (const [na, va] of fm[eixo])
        for (const [nb, vb] of fo[eixo]) {
          const dpx = (vb - va) * zoom;
          if (Math.abs(dpx) <= 20) perto.push({ eixo, movel: na, outro: `${o.id}.${nb}`, px_ecra: r6(dpx) });
        }
  }
  perto.sort((a, b) => Math.abs(a.px_ecra) - Math.abs(b.px_ecra) || (a.eixo + a.movel + a.outro < b.eixo + b.movel + b.outro ? -1 : 1));
  return { p0, p1, delta_cena: [r6(ddx), r6(ddy)], alvo, feicoes_perto_no_plano: perto };
};

if (soPlano) {
  for (const caso of casos) {
    if (so && caso.nome !== so) continue;
    const els = cenas[caso.cena].elements, zoom = caso.zoom || 1;
    for (const s of caso.sondas) {
      if (s.tipo !== "arrasta") continue;
      const p = planear(els, s, zoom, ([x, y]) => [x * zoom, y * zoom]);
      console.log(`${caso.nome}/${s.rotulo}: Δ=${p.delta_cena} perto=${p.feicoes_perto_no_plano.slice(0, 4).map((q) => `${q.eixo}:${q.movel}~${q.outro}=${q.px_ecra}`).join(" ")}`);
    }
  }
  process.exit(0);
}

const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const PKG = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/@excalidraw/excalidraw/package.json"), "utf8"));
const PW = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/playwright/package.json"), "utf8"));
fs.mkdirSync(outDir, { recursive: true });

const MIME = { ".html": "text/html", ".js": "text/javascript", ".woff2": "font/woff2", ".ttf": "font/ttf", ".wasm": "application/wasm", ".css": "text/css" };
const server = createServer((req, res) => {
  const p = path.normalize(decodeURIComponent(new URL(req.url, "http://x").pathname));
  const file = path.join(WWW, p === "/" ? "index.html" : p);
  if (!file.startsWith(WWW) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { "content-type": MIME[path.extname(file)] || "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const origin = `http://127.0.0.1:${server.address().port}`;

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

// ---- lado da página ----
const pageMount = async ({ raw, zoom, snap }) => {
  const { convertToExcalidrawElements, restoreElements } = window.oracle.api;
  const isSkeleton = (el) => el.version === undefined || el.seed === undefined;
  const conv = raw.some(isSkeleton) ? convertToExcalidrawElements(raw, { regenerateIds: false }) : raw;
  const elements = restoreElements(conv, null, { refreshDimensions: false, repairBindings: true });
  await window.oracle.editorMount(elements);
  const api = window.oracle.editorApi();
  api.updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: zoom }, objectsSnapModeEnabled: snap } });
  await document.fonts.ready;
  return true;
};
const pageSnap = () => {
  const api = window.oracle.editorApi();
  const a = api.getAppState();
  return {
    elementos: api.getSceneElements().map((e) => ({
      id: e.id, type: e.type, x: e.x, y: e.y, width: e.width, height: e.height, angle: e.angle,
      frameId: e.frameId ?? null, ...(e.points ? { points: e.points } : {}),
    })),
    appState: {
      objectsSnapModeEnabled: a.objectsSnapModeEnabled, snapLines: a.snapLines,
      gridModeEnabled: a.gridModeEnabled, isRotating: a.isRotating,
      selectedElementIds: Object.keys(a.selectedElementIds || {}).filter((k) => a.selectedElementIds[k]).sort(),
      zoom: a.zoom?.value, scroll: [a.scrollX, a.scrollY], offset: [a.offsetLeft, a.offsetTop],
    },
  };
};
const pageView = () => {
  const a = window.oracle.editorApi().getAppState();
  return { sx: a.scrollX, sy: a.scrollY, z: a.zoom.value, ol: a.offsetLeft, ot: a.offsetTop };
};
const pageSelect = (ids) => {
  window.oracle.editorApi().updateScene({
    appState: { selectedElementIds: Object.fromEntries(ids.map((id) => [id, true])), selectedGroupIds: {}, editingGroupId: null },
  });
};
const pageCursor = () => {
  const c = document.querySelector("canvas.interactive");
  return c ? c.style.cursor : null;
};

// ---- lado do Node ----
const browser = await chromium.launch({ headless: true });
const chromiumVersion = browser.version();
let failures = 0;
try {
  for (const caso of casos) {
    if (so && caso.nome !== so) continue;
    const cena = cenas[caso.cena];
    const zoom = caso.zoom || 1;
    const sha = createHash("sha256").update(JSON.stringify({ caso, cena })).digest("hex");
    const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC", viewport: { width: 1280, height: 800 } });
    const external = [];
    const consoleErrors = [];
    await context.route("**/*", (route) => {
      const u = route.request().url();
      if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue();
      external.push(u);
      return route.abort();
    });
    await context.addInitScript(seedScript(parseInt(sha.slice(0, 8), 16)));
    const page = await context.newPage();
    await page.clock.setFixedTime(FIXED_TIME);
    page.on("console", (m) => { if (m.type() === "error") consoleErrors.push(`${m.type()}: ${m.text()}`); });
    page.on("pageerror", (e) => consoleErrors.push(`pageerror: ${e.message}`));
    await page.goto(origin + "/", { timeout: 60000 });
    await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 60000 });

    // Medido 08/10 (corrida 2 de G8 d-1): sob carga o ÚLTIMO pointermove pode não ser aplicado (o móvel
    // ficou num passo intermédio, 40 px antes). Cura: repetir o movimento final e esperar 2 quadros.
    const assenta = async ([x, y]) => {
      await page.waitForTimeout(100);
      await page.mouse.move(x, y);
      await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
      await page.waitForTimeout(100);
    };
    const sondas = [];
    try {
      for (const s of caso.sondas) {
        const snap = s.snap !== false;
        await page.evaluate(pageMount, { raw: cena.elements, zoom, snap });
        await page.waitForTimeout(250);
        for (const k of s.teclas_antes || []) { await page.keyboard.press(k); await page.waitForTimeout(120); }
        const v = await page.evaluate(pageView);
        const ecraDe = ([x, y]) => [(x + v.sx) * v.z + v.ol, (y + v.sy) * v.z + v.ot];
        const inicio = await page.evaluate(pageSnap);
        const els0 = inicio.elementos;
        const out = { rotulo: s.rotulo, sonda: s, modo_ligado_antes: inicio.appState.objectsSnapModeEnabled };
        if (s.tipo === "arrasta") {
          const plano = planear(els0, s, zoom, ecraDe);
          const { p0, p1 } = plano;
          await page.mouse.move(p0[0], p0[1]);
          await page.mouse.down();
          await page.mouse.move(p0[0] + Math.sign(p1[0] - p0[0] || 1) * 2, p0[1] + Math.sign(p1[1] - p0[1] || 1) * 2);
          await page.mouse.move(p1[0], p1[1], { steps: 12 });
          await assenta(p1);
          const antes = await page.evaluate(pageSnap);
          await page.mouse.up();
          await page.waitForTimeout(150);
          const depois = await page.evaluate(pageSnap);
          const m0 = els0.find((e) => e.id === s.mover);
          const mA = antes.elementos.find((e) => e.id === s.mover);
          const mD = depois.elementos.find((e) => e.id === s.mover);
          const medido = {
            arrastou: [r6(mD.x - m0.x), r6(mD.y - m0.y)],
            encaixe_desvio_cena: [r6(mD.x - (m0.x + plano.delta_cena[0])), r6(mD.y - (m0.y + plano.delta_cena[1]))],
            igual_antes_e_depois_de_largar: mA.x === mD.x && mA.y === mD.y,
            seleccao_depois: depois.appState.selectedElementIds,
          };
          for (const eixo of ["x", "y"]) {
            const c = s[eixo];
            if (!c) continue;
            const T = valor(depois.elementos, c.a, eixo);
            medido[`residuo_${eixo}_px_ecra`] = r6((feicao(mD, c.de, eixo) - T) * zoom);
          }
          Object.assign(out, {
            plano,
            antes_de_largar: { snapLines: antes.appState.snapLines, objectsSnapModeEnabled: antes.appState.objectsSnapModeEnabled, movel: resumo(mA) },
            depois: { elementos: depois.elementos.map(resumo), snapLines: depois.appState.snapLines },
            medido,
          });
        } else if (s.tipo === "roda") {
          await page.evaluate(pageSelect, [s.el]);
          await page.waitForTimeout(150);
          const e = els0.find((q) => q.id === s.el);
          const a = aabb(e);
          const th = e.angle || 0;
          const c = ecraDe([a.c, a.m]);
          const up = (k) => [c[0] + Math.sin(th) * (e.height / 2 + k) * zoom, c[1] - Math.cos(th) * (e.height / 2 + k) * zoom];
          const grabs = [];
          for (let k = 0; k <= 60; k++) {
            const [hx, hy] = up(k / zoom);
            await page.mouse.move(Math.round(hx), Math.round(hy));
            await page.waitForTimeout(25);
            if ((await page.evaluate(pageCursor)) === "grab") grabs.push(k);
          }
          const kmid = grabs.length ? (grabs[0] + grabs[grabs.length - 1]) / 2 : 20;
          const r = (e.height / 2) * zoom + kmid;
          const at = (phi) => [Math.round(c[0] + Math.sin(phi) * r), Math.round(c[1] - Math.cos(phi) * r)];
          const alvo = (s.graus * Math.PI) / 180;
          const p0 = at(th);
          await page.mouse.move(p0[0], p0[1]);
          await page.mouse.down();
          if (s.shift) await page.keyboard.down("Shift");
          const N = 16;
          for (let i = 1; i <= N; i++) { const q = at(th + ((alvo - th) * i) / N); await page.mouse.move(q[0], q[1]); }
          await assenta(at(alvo));
          const antes = await page.evaluate(pageSnap);
          await page.mouse.up();
          if (s.shift) await page.keyboard.up("Shift");
          await page.waitForTimeout(150);
          const depois = await page.evaluate(pageSnap);
          const pf = at(alvo);
          const ang = (p) => r6((Math.atan2(p[0] - c[0], -(p[1] - c[1])) * 180) / Math.PI);
          const mD = depois.elementos.find((q) => q.id === s.el);
          Object.assign(out, {
            pega: { px_ecra_acima_da_aresta_com_cursor_grab: grabs, usado_k: kmid, raio_px: r, ponteiro_inicio: p0, ponteiro_fim: pf, angulo_do_ponteiro_graus: ang(pf) },
            antes_de_largar: { snapLines: antes.appState.snapLines, isRotating: antes.appState.isRotating, el: resumo(antes.elementos.find((q) => q.id === s.el)) },
            depois: { elementos: depois.elementos.map(resumo) },
            medido: { angulo_final_graus: r6((((mD.angle || 0) * 180) / Math.PI)), pedido_graus: s.graus, rodou: mD.angle !== e.angle },
          });
        } else throw new Error(`sonda desconhecida ${s.tipo}`);
        await page.evaluate(() => window.oracle.editorUnmount());
        sondas.push(out);
      }
    } catch (e) {
      console.error(`FALHA ${caso.nome}: ${e.stack || e.message}`);
      failures++;
      await context.close();
      continue;
    }
    await context.close();

    const base = path.join(outDir, `guias_${caso.nome}`);
    const body = JSON.stringify({ caso: caso.nome, pergunta: caso.pergunta, cena: caso.cena, zoom, sondas }, null, 2) + "\n";
    fs.writeFileSync(base + ".json", body);
    const meta = {
      oracle: "@excalidraw/excalidraw",
      package_version: PKG.version,
      package_license: PKG.license,
      chromium_version: chromiumVersion,
      playwright_version: PW.version,
      date: new Date().toISOString(),
      command: `node corre_guias.mjs ${path.basename(casosPath)} <saidas>`,
      input: path.basename(casosPath),
      input_sha256: casosSha,
      case_sha256: sha,
      inputs: { cena: cena.elements, zoom, sondas: caso.sondas },
      measured: "Por sonda: a cena montada de novo; arrasto (desce no centro do móvel ou no título da moldura, 2 px, 12 passos, o movimento final repetido e 2 quadros de espera) até a feição `de` ficar a `mais` px de ECRÃ da feição `a`; appState.snapLines e o móvel ANTES de largar; x,y,w,h,ângulo, AABB e cantos de todos DEPOIS; o desvio do encaixe = posição final − posição sem encaixe. Rotação: a pega achada pelo cursor «grab» do canvas.interactive, arco de 16 passos até ao ângulo pedido.",
      deterministic_env: { prng: "mulberry32", seed_from: "case_sha256[0..8]", fixed_time_utc: new Date(FIXED_TIME).toISOString() },
      editor: "<Excalidraw> montado (excalidrawAPI público: updateScene para zoom/scroll/objectsSnapModeEnabled/selecção, getSceneElements, getAppState); rato e teclado do Playwright; janela 1280×800, deviceScaleFactor 1",
      external_requests_blocked: external,
      console: consoleErrors,
      outputs_sha256: { [path.basename(base + ".json")]: createHash("sha256").update(body).digest("hex") },
    };
    fs.writeFileSync(base + ".meta.json", JSON.stringify(meta, null, 2) + "\n");
    console.log(`ok ${caso.nome}: ${sondas.length} sondas`);
  }
} finally {
  await browser.close();
  server.close();
}
process.exit(failures ? 1 : 0);
