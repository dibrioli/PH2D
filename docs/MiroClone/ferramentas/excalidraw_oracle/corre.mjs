// Corre o oráculo Excalidraw sobre entradas .excalidraw NOSSAS e grava as fixtures.
// Uso: node corre.mjs [--sem-semente] <pasta_saida> <entrada.excalidraw>...
// Para cada entrada <n>: <n>.svg · <n>.png (escala 2) · <n>.restored.json · <n>.meta.json.
// Por omissão o ambiente é DETERMINÍSTICO: Math.random e crypto.getRandomValues semeados por
// entrada (sha256 da entrada) e relógio fixo — controla ids/seeds/versionNonce que o
// convertToExcalidrawElements sorteia para esqueletos. `--sem-semente` desliga isso (diagnóstico).
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
const FIXED_TIME = Date.UTC(2026, 0, 1, 0, 0, 0);
const PNG_MAX_BYTES = 300 * 1024;

let args = process.argv.slice(2);
const seeded = !args.includes("--sem-semente");
args = args.filter((a) => a !== "--sem-semente");
const [outDir, ...inputs] = args;
if (!outDir || inputs.length === 0) {
  console.error("uso: node corre.mjs [--sem-semente] <pasta_saida> <entrada.excalidraw>...");
  process.exit(2);
}
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

// Injectado antes de qualquer script da página: PRNG mulberry32 semeado.
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

const browser = await chromium.launch({ headless: true });
const chromiumVersion = browser.version();
let failures = 0;
try {
  for (const input of inputs) {
    const name = path.basename(input).replace(/\.excalidraw(\.json)?$/, "");
    const bytes = fs.readFileSync(input);
    const sha = createHash("sha256").update(bytes).digest("hex");
    const context = await browser.newContext({ deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC" });
    const external = [];
    const consoleErrors = [];
    await context.route("**/*", (route) => {
      const u = route.request().url();
      if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue();
      external.push(u);
      return route.abort();
    });
    if (seeded) await context.addInitScript(seedScript(parseInt(sha.slice(0, 8), 16)));
    const page = await context.newPage();
    if (seeded) await page.clock.setFixedTime(FIXED_TIME);
    page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") consoleErrors.push(`${m.type()}: ${m.text()}`); });
    page.on("pageerror", (e) => consoleErrors.push(`pageerror: ${e.message}`));
    await page.goto(origin + "/", { timeout: 60000 });
    await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 60000 });
    let result;
    try {
      result = await page.evaluate((txt) => window.oracle.run(JSON.parse(txt)), bytes.toString("utf8"));
      // Setas em cotovelo: só o EDITOR as encaminha. Monta-o, empurra as formas das pontas → e ←
      // (posição final igual à entrada) e lê a cena que ele guardou.
      if (result.restored.some((e) => e.type === "arrow" && e.elbowed)) {
        await page.evaluate((els) => window.oracle.editorMount(els), result.restored);
        const ends = await page.evaluate(() => window.oracle.elbowEndpoints());
        await page.evaluate((ids) => window.oracle.editorSelect(ids), ends);
        await page.waitForTimeout(200);
        await page.keyboard.press("ArrowRight");
        await page.waitForTimeout(200);
        await page.keyboard.press("ArrowLeft");
        await page.waitForTimeout(300);
        result.editor = await page.evaluate(() => window.oracle.editorRead());
        result.editor.nudged = ends;
      }
    } catch (e) {
      console.error(`FALHA ${name}: ${e.message}`);
      failures++;
      await context.close();
      continue;
    }
    await context.close();

    const base = path.join(outDir, name);
    const outputs = {};
    const write = (suffix, data) => {
      fs.writeFileSync(base + suffix, data);
      outputs[path.basename(base + suffix)] = createHash("sha256").update(data).digest("hex");
    };
    const svgHeader = `<!-- ph2d excalidraw-oracle: @excalidraw/excalidraw ${PKG.version} · chromium ${chromiumVersion} · entrada ${path.basename(input)} sha256 ${sha} · ver ${name}.meta.json -->\n`;
    write(".svg", svgHeader + result.svg + "\n");
    const png = Buffer.from(result.pngBase64, "base64");
    let pngOmitted = false;
    if (png.length <= PNG_MAX_BYTES) write(".png", png);
    else { pngOmitted = true; fs.rmSync(base + ".png", { force: true }); }
    write(".restored.json", JSON.stringify(result.restored, null, 2) + "\n");
    if (result.editor) {
      write(".editor.svg", svgHeader + result.editor.svg + "\n");
      const epng = Buffer.from(result.editor.pngBase64, "base64");
      if (epng.length <= PNG_MAX_BYTES) write(".editor.png", epng);
      write(".editor.json", JSON.stringify(result.editor.elements, null, 2) + "\n");
    }
    const meta = {
      oracle: "@excalidraw/excalidraw",
      package_version: PKG.version,
      package_license: PKG.license,
      chromium_version: chromiumVersion,
      playwright_version: JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules/playwright/package.json"), "utf8")).version,
      date: new Date().toISOString(),
      input: path.basename(input),
      input_sha256: sha,
      deterministic_env: seeded ? { prng: "mulberry32", seed_from: "input_sha256[0..8]", fixed_time_utc: new Date(FIXED_TIME).toISOString() } : null,
      api: ["convertToExcalidrawElements(regenerateIds:false) se houver esqueletos", "restoreElements(repairBindings:true)", "exportToSvg(exportPadding:10)", "exportToBlob(image/png, exportScale:2, exportPadding:10)"],
      editor_step: result.editor ? { what: "<Excalidraw> montado; formas das pontas das setas elbowed seleccionadas e empurradas ArrowRight+ArrowLeft; cena lida por excalidrawAPI.getSceneElements", nudged: result.editor.nudged } : null,
      png_bytes: png.length,
      png_omitted_over_300KB: pngOmitted,
      fonts_loaded: result.fontsLoaded,
      external_requests_blocked: external,
      console: consoleErrors,
      outputs_sha256: outputs,
    };
    fs.writeFileSync(base + ".meta.json", JSON.stringify(meta, null, 2) + "\n");
    console.log(`ok ${name}: svg ${result.svg.length} B · png ${png.length} B${pngOmitted ? " (omitido >300KB)" : ""} · ${result.restored.length} elementos · fontes [${result.fontsLoaded.join(", ")}]${external.length ? ` · BLOQUEADOS ${external.length}` : ""}`);
  }
} finally {
  await browser.close();
  server.close();
}
process.exit(failures ? 1 : 0);
