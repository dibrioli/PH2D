// W5 — organizar: molduras (frames), grupos, alinhar/distribuir, ordem e cadeado no EDITOR montado.
// Uso: node corre_organizar.mjs [--fotos <pasta>] [--so <caso>] <casos.json> <pasta_saida>
// Cada caso: cena NOSSA → <Excalidraw> montado → passos de rato/teclado do Playwright (e selecção
// pelo excalidrawAPI público) → a cena lida ANTES e DEPOIS de cada passo em <saida>/organizar_<caso>.json
// (+ .meta.json). Determinístico como o corre.mjs: PRNG semeado pelo sha256 do caso, relógio fixo.
// Coordenadas dos passos em unidades de CENA; o editor fica em zoom 1, scroll 0, janela 1280×800.
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const { chromium } = require("playwright");
const WWW = path.join(ORACLE_HOME, "www");
const PKG = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/@excalidraw/excalidraw/package.json"), "utf8"));
const PW = JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/playwright/package.json"), "utf8"));
const FIXED_TIME = Date.UTC(2026, 0, 1, 0, 0, 0);

let args = process.argv.slice(2);
const opt = (flag) => {
  const i = args.indexOf(flag);
  if (i < 0) return null;
  const v = args[i + 1];
  args.splice(i, 2);
  return v;
};
const fotosDir = opt("--fotos");
const so = opt("--so");
const [casosPath, outDir] = args;
if (!casosPath || !outDir) {
  console.error("uso: node corre_organizar.mjs [--fotos <pasta>] [--so <caso>] <casos.json> <pasta_saida>");
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });
if (fotosDir) fs.mkdirSync(fotosDir, { recursive: true });
const casosBytes = fs.readFileSync(casosPath);
const casosSha = createHash("sha256").update(casosBytes).digest("hex");
const { cenas, casos } = JSON.parse(casosBytes.toString("utf8"));

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

// ---- lado da página (corre dentro do Chromium) ----
const pageMount = async (raw) => {
  const { convertToExcalidrawElements, restoreElements } = window.oracle.api;
  const isSkeleton = (el) => el.version === undefined || el.seed === undefined;
  const conv = raw.some(isSkeleton) ? convertToExcalidrawElements(raw, { regenerateIds: false }) : raw;
  const elements = restoreElements(conv, null, { refreshDimensions: false, repairBindings: true });
  await window.oracle.editorMount(elements);
  const api = window.oracle.editorApi();
  api.updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: 1 } } });
  await document.fonts.ready;
  return true;
};
const pageSnap = () => {
  const api = window.oracle.editorApi();
  const a = api.getAppState();
  const keys = (o) => Object.keys(o || {}).filter((k) => o[k]).sort();
  return {
    elementos: api.getSceneElementsIncludingDeleted().map((e) => ({
      id: e.id, type: e.type, name: e.name ?? null, index: e.index,
      x: e.x, y: e.y, width: e.width, height: e.height, angle: e.angle,
      groupIds: e.groupIds, frameId: e.frameId, locked: e.locked, isDeleted: e.isDeleted,
    })),
    appState: {
      selectedElementIds: keys(a.selectedElementIds),
      selectedGroupIds: keys(a.selectedGroupIds),
      editingGroupId: a.editingGroupId ?? null,
      activeTool: a.activeTool?.type ?? null,
      scroll: [a.scrollX, a.scrollY], zoom: a.zoom?.value, offset: [a.offsetLeft, a.offsetTop],
    },
  };
};
const pageSelect = (ids) => {
  window.oracle.editorApi().updateScene({
    appState: { selectedElementIds: Object.fromEntries(ids.map((id) => [id, true])), selectedGroupIds: {}, editingGroupId: null },
  });
};
const pageView = () => {
  const a = window.oracle.editorApi().getAppState();
  return { sx: a.scrollX, sy: a.scrollY, z: a.zoom.value, ol: a.offsetLeft, ot: a.offsetTop };
};
// Botões visíveis com title/aria-label (o painel de acções do Excalidraw).
const pageButtons = () => {
  const out = new Set();
  for (const el of document.querySelectorAll("[title],[aria-label]")) {
    const r = el.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) continue;
    const t = el.getAttribute("title") || "";
    const l = el.getAttribute("aria-label") || "";
    out.add(`${l} | ${t}`);
  }
  return [...out].sort();
};
// O nome visível (o texto) do título de uma moldura: onde está no DOM e com que letra.
const pageDomText = (texto) => {
  const out = [];
  for (const el of document.querySelectorAll("body *")) {
    if (el.children.length || el.textContent !== texto) continue;
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    out.push({ tag: el.tagName.toLowerCase(), classe: el.className, rect: [r.x, r.y, r.width, r.height],
      fontSize: cs.fontSize, fontFamily: cs.fontFamily, fontWeight: cs.fontWeight, lineHeight: cs.lineHeight, color: cs.color });
  }
  return out;
};
// Píxeis de uma cor (± tol) numa imagem: caixa e contagem; e a caixa dos píxeis não brancos numa faixa.
const pagePixels = async ({ fonte, cor, tol, faixa }) => {
  let w, h, d, pngBase64 = null;
  if (fonte === "exportacao") {
    const api = window.oracle.editorApi();
    const els = api.getSceneElements();
    const blob = await window.oracle.api.exportToBlob({
      elements: els, appState: { exportBackground: true, viewBackgroundColor: "#ffffff", exportWithDarkMode: false },
      files: null, mimeType: "image/png", exportPadding: 10,
    });
    const buf = new Uint8Array(await blob.arrayBuffer());
    let s = "";
    for (let i = 0; i < buf.length; i += 0x8000) s += String.fromCharCode.apply(null, buf.subarray(i, i + 0x8000));
    pngBase64 = btoa(s);
    const bmp = await createImageBitmap(blob);
    w = bmp.width; h = bmp.height;
    const c = new OffscreenCanvas(w, h);
    const g = c.getContext("2d");
    g.drawImage(bmp, 0, 0);
    d = g.getImageData(0, 0, w, h).data;
  } else {
    const cv = [...document.querySelectorAll("canvas")].find((c) => c.classList.contains("static"));
    w = cv.width; h = cv.height;
    d = cv.getContext("2d").getImageData(0, 0, w, h).data;
  }
  const box = (pred, x0 = 0, y0 = 0, x1 = w, y1 = h) => {
    let n = 0, mx = Infinity, my = Infinity, Mx = -Infinity, My = -Infinity;
    for (let y = Math.max(0, y0); y < Math.min(h, y1); y++)
      for (let x = Math.max(0, x0); x < Math.min(w, x1); x++) {
        const i = (y * w + x) * 4;
        if (pred(d[i], d[i + 1], d[i + 2], d[i + 3])) { n++; mx = Math.min(mx, x); my = Math.min(my, y); Mx = Math.max(Mx, x); My = Math.max(My, y); }
      }
    return n ? { n, x0: mx, y0: my, x1: Mx, y1: My } : { n: 0 };
  };
  const out = { fonte, largura: w, altura: h };
  if (cor) out.cor = box((r, g, b) => Math.abs(r - cor[0]) <= tol && Math.abs(g - cor[1]) <= tol && Math.abs(b - cor[2]) <= tol);
  if (faixa) out.faixa = { rect: faixa, naoBranco: box((r, g, b, a) => a > 0 && (r < 235 || g < 235 || b < 235), ...faixa) };
  return { out, pngBase64 };
};
const pageSvgInfo = async () => {
  const els = window.oracle.editorApi().getSceneElements();
  const svgEl = await window.oracle.api.exportToSvg({
    elements: els, appState: { exportBackground: true, viewBackgroundColor: "#ffffff" }, files: null, exportPadding: 10,
  });
  return {
    markup: new XMLSerializer().serializeToString(svgEl),
    viewBox: svgEl.getAttribute("viewBox"),
    width: svgEl.getAttribute("width"), height: svgEl.getAttribute("height"),
    clipPaths: [...svgEl.querySelectorAll("clipPath")].map((c) => c.innerHTML),
    clipPathRefs: [...svgEl.querySelectorAll("[clip-path]")].length,
    texts: [...svgEl.querySelectorAll("text")].map((t) => ({ text: t.textContent, x: t.getAttribute("x"), y: t.getAttribute("y"), fontSize: t.getAttribute("font-size"), fontFamily: t.getAttribute("font-family") })),
  };
};

// ---- lado do Node ----
const browser = await chromium.launch({ headless: true });
const chromiumVersion = browser.version();
let failures = 0;
try {
  for (const caso of casos) {
    if (so && caso.nome !== so) continue;
    const cena = cenas[caso.cena];
    const casoBytes = JSON.stringify({ caso, cena });
    const sha = createHash("sha256").update(casoBytes).digest("hex");
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

    const settle = () => page.waitForTimeout(250);
    const scr = async ([x, y]) => {
      const v = await page.evaluate(pageView);
      return [(x + v.sx) * v.z + v.ol, (y + v.sy) * v.z + v.ot];
    };
    const passos = [];
    const imagens = {};
    let foto = 0;
    const record = async (rotulo, acao, extra) => {
      const s = await page.evaluate(pageSnap);
      passos.push({ passo: passos.length, rotulo, acao, ...extra, ...s });
      if (fotosDir) await page.screenshot({ path: path.join(fotosDir, `${caso.nome}_${String(foto++).padStart(2, "0")}_${rotulo}.png`) });
    };
    try {
      await page.evaluate(pageMount, cena.elements);
      await settle();
      await record("montado", null);
      for (const p of caso.passos) {
        let extra = {};
        switch (p.do) {
          case "select": await page.evaluate(pageSelect, p.ids); break;
          case "key": for (let i = 0; i < (p.vezes || 1); i++) { await page.keyboard.press(p.key); await page.waitForTimeout(60); } break;
          case "tool": await page.keyboard.press(p.key); break;
          case "click": {
            const [x, y] = await scr(p.at);
            if (p.shift) await page.keyboard.down("Shift");
            await page.mouse.click(x, y);
            if (p.shift) await page.keyboard.up("Shift");
            break;
          }
          case "dblclick": { const [x, y] = await scr(p.at); await page.mouse.dblclick(x, y); break; }
          case "drag": {
            const [x0, y0] = await scr(p.from);
            const [x1, y1] = await scr(p.to);
            await page.mouse.move(x0, y0);
            await page.mouse.down();
            await page.mouse.move(x0 + Math.sign(x1 - x0) * 2, y0 + Math.sign(y1 - y0) * 2);
            await page.mouse.move(x1, y1, { steps: 12 });
            await page.waitForTimeout(80);
            await page.mouse.up();
            break;
          }
          case "botao": {
            // O painel de acções rola: o locator traz o botão à vista antes de clicar.
            const b = page.getByRole("button", { name: new RegExp(p.re, "i") }).first();
            extra.botao = `${await b.getAttribute("aria-label")} | ${await b.getAttribute("title")}`;
            await b.click({ timeout: 5000 });
            break;
          }
          case "botoes": extra.botoes = await page.evaluate(pageButtons); break;
          case "pixeis": {
            const { out, pngBase64 } = await page.evaluate(pagePixels, p);
            extra.pixeis = out;
            if (pngBase64) { imagens[".png"] = Buffer.from(pngBase64, "base64"); out.ficheiro = `organizar_${caso.nome}.png`; }
            break;
          }
          case "svg": {
            const { markup, ...info } = await page.evaluate(pageSvgInfo);
            imagens[".svg"] = Buffer.from(markup + "\n");
            extra.svg = { ...info, ficheiro: `organizar_${caso.nome}.svg` };
            break;
          }
          case "dom_texto": extra.dom = await page.evaluate(pageDomText, p.texto); break;
          default: throw new Error(`passo desconhecido ${p.do}`);
        }
        await settle();
        await record(p.rotulo || p.do, p, extra);
      }
    } catch (e) {
      console.error(`FALHA ${caso.nome}: ${e.message}`);
      failures++;
      await context.close();
      continue;
    }
    await page.evaluate(() => window.oracle.editorUnmount());
    await context.close();

    const base = path.join(outDir, `organizar_${caso.nome}`);
    const body = JSON.stringify({ caso: caso.nome, pergunta: caso.pergunta, cena: caso.cena, passos }, null, 2) + "\n";
    fs.writeFileSync(base + ".json", body);
    const outputs = { [path.basename(base + ".json")]: createHash("sha256").update(body).digest("hex") };
    for (const [suf, buf] of Object.entries(imagens)) {
      fs.writeFileSync(base + suf, buf);
      outputs[path.basename(base + suf)] = createHash("sha256").update(buf).digest("hex");
    }
    const meta = {
      oracle: "@excalidraw/excalidraw",
      package_version: PKG.version,
      package_license: PKG.license,
      chromium_version: chromiumVersion,
      playwright_version: PW.version,
      date: new Date().toISOString(),
      input: path.basename(casosPath),
      input_sha256: casosSha,
      case_sha256: sha,
      deterministic_env: { prng: "mulberry32", seed_from: "case_sha256[0..8]", fixed_time_utc: new Date(FIXED_TIME).toISOString() },
      editor: "<Excalidraw> montado (excalidrawAPI: updateScene para seleccionar/scroll 0/zoom 1, getSceneElementsIncludingDeleted, getAppState); rato e teclado do Playwright; janela 1280×800, deviceScaleFactor 1",
      external_requests_blocked: external,
      console: consoleErrors,
      outputs_sha256: outputs,
    };
    fs.writeFileSync(base + ".meta.json", JSON.stringify(meta, null, 2) + "\n");
    console.log(`ok ${caso.nome}: ${passos.length} passos`);
  }
} finally {
  await browser.close();
  server.close();
}
process.exit(failures ? 1 : 0);
