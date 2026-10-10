// W9 — a fidelidade da importação/exportação `.excalidraw`, medida nas DUAS versões do oráculo: a 0.18.1
// (npm `latest`) e o canal `next` fixado em next/package.json (o do excalidraw.com, que grava as ligações
// `mode:"orbit"`). Só a API pública (src/oracle_w9.js) + rato/teclado do Playwright no editor montado.
//
// Uso: node corre_fidelidade.mjs [--so <caso>] [--versao 0181|next] [--prefixo w9] <casos.json> <pasta_saida>
// Por caso <c> e versão <v>: saidas/w9_<c>.<v>.json (cada passo: os elementos depois dele) e
// saidas/w9_<c>.<v>.svg (o export do último estado, cabeçalho com versão e sha da entrada).
// Passos (casos.json, `passos`):
//   restore            restoreElements(elementos, {refreshDimensions:false, repairBindings:true})
//   montar             <Excalidraw> com os restaurados (1280×800, zoom 1, scroll 0 ⇒ ecrã = cena)
//   empurrar {ids}     selecciona ids e carrega → depois ← (posição final igual): o editor RE-LIGA as setas
//   desenhar {tecla, de, a, via?, n?, dt?, espera?, mods?}  arma a ferramenta pela tecla e arrasta de `de`
//                      a `a` (cena = ecrã) pelos `via`, `n` passos por trecho, `espera` ms parado antes de largar;
//                      `manter` não limpa a selecção; `clique_clique` larga no início e clica no fim;
//                      `sem_escape` não carrega Escape no fim; `captura: <nome>` fotografa o ecrã e guarda
//                      o appState de ligação/realce ANTES de largar (W10: o realce a meio do arrasto)
//   serializar         serializeAsJSON(…, "local") → os elementos do ficheiro que ele GRAVA
//   svg                exportToSvg do estado actual (padding 10)
// Qualquer passo com `"guardar": false` corre mas não se escreve na saída.
// Determinismo: Math.random/crypto semeados pelo sha256 (casos.json + nome do caso), relógio fixo.
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const HOMES = {
  "0181": process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle"),
  next: process.env.ORACLE_NEXT_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle-next"),
};
const FIXED_TIME = Date.UTC(2026, 0, 1, 0, 0, 0);

let args = process.argv.slice(2);
const opt = (flag) => {
  const i = args.indexOf(flag);
  if (i < 0) return null;
  const v = args[i + 1];
  args.splice(i, 2);
  return v;
};
const so = opt("--so");
const soVersao = opt("--versao");
const prefixo = opt("--prefixo") || "w9";
const [casosPath, outDir] = args;
if (!casosPath || !outDir) {
  console.error("uso: node corre_fidelidade.mjs [--so <caso>] [--versao 0181|next] <casos.json> <pasta_saida>");
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });
const casosBytes = fs.readFileSync(casosPath);
const sha = (b) => createHash("sha256").update(b).digest("hex");
const { casos } = JSON.parse(casosBytes.toString("utf8"));

const seedScript = (s) => `(() => {
  let a = ${s} >>> 0;
  const next = () => { a = (a + 0x6D2B79F5) >>> 0; let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
  Math.random = next;
  const gr = (arr) => { const b = new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength);
    for (let i = 0; i < b.length; i++) b[i] = (next() * 256) | 0; return arr; };
  Object.defineProperty(crypto, "getRandomValues", { value: gr });
})();`;

const MIME = { ".html": "text/html", ".js": "text/javascript", ".woff2": "font/woff2", ".ttf": "font/ttf", ".wasm": "application/wasm", ".css": "text/css" };
// Campos voláteis que não dizem nada da lei e mudam com o relógio/PRNG do editor.
const VOLATEIS = new Set(["updated", "versionNonce", "version"]);
const limpa = (els) => els.map((e) => Object.fromEntries(Object.entries(e).filter(([k]) => !VOLATEIS.has(k))));
// Relógio LIVRE (desenho à mão): o PRNG semeado é consumido por temporizadores em número variável, por
// isso os ids, a `seed` e o `created` dos elementos NOVOS sorteiam-se. Normaliza-os: ids novos →
// `novo1, novo2…` pela ordem da cena (também dentro das ligações), `seed` = 7919·(posição+1), sem
// `created` — o resto (geometria, ligações) é a medida e fica tal e qual (duas corridas iguais).
const normaliza = (els, conhecidos) => {
  const mapa = new Map();
  for (const e of els) if (!conhecidos.has(e.id)) mapa.set(e.id, `novo${mapa.size + 1}`);
  const txt = JSON.stringify(els, (k, v) => (typeof v === "string" && mapa.has(v) ? mapa.get(v) : v));
  return JSON.parse(txt).map((e, i) => {
    const o = { ...e, seed: conhecidos.has(e.id) ? e.seed : 7919 * (i + 1) };
    delete o.created;
    return o;
  });
};

let falhas = 0;
for (const [versao, home] of Object.entries(HOMES)) {
  if (soVersao && soVersao !== versao) continue;
  const WWW = path.join(home, "www_w9");
  if (!fs.existsSync(path.join(WWW, "oracle.js"))) {
    console.error(`página W9 não construída em ${WWW}: corra instala_w9.sh`);
    process.exit(2);
  }
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
  await new Promise((r) => server.listen(Number(process.env.ORACLE_PORT || 47119), "127.0.0.1", r));
  const origin = `http://127.0.0.1:${server.address().port}`;
  const browser = await chromium.launch({ headless: true });
  try {
    for (const caso of casos) {
      if (so && caso.nome !== so) continue;
      const seed = parseInt(sha(Buffer.concat([casosBytes, Buffer.from(caso.nome)])).slice(0, 8), 16);
      const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: "en-US", timezoneId: "UTC" });
      await ctx.route("**/*", (route) => {
        const u = route.request().url();
        if (u.startsWith(origin) || u.startsWith("data:") || u.startsWith("blob:")) return route.continue();
        return route.abort();
      });
      await ctx.addInitScript(seedScript(seed));
      const page = await ctx.newPage();
      // ⚠️ O relógio fixo CONGELA o Date.now — e o editor next decide orbit × inside por um TEMPORIZADOR
      // (`bindModeHandler`): os casos de desenho à mão levam `"relogio": "livre"` (medido: com o relógio
      // fixo toda a ponta largada dentro de uma forma saía `inside`).
      if (caso.relogio !== "livre") await page.clock.setFixedTime(FIXED_TIME);
      const consola = [];
      page.on("pageerror", (e) => consola.push(`pageerror: ${e.message}`));
      page.on("console", (m) => { if (m.type() === "error") consola.push(`error: ${m.text()}`); });
      await page.goto(origin + "/", { timeout: 60000 });
      await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 60000 });
      const passos = [];
      let atuais = caso.elementos;
      const conhecidos = new Set(caso.elementos.map((e) => e.id));
      let svg = null;
      try {
        for (const passo of caso.passos) {
          const t = passo.tipo;
          if (t === "restore") {
            atuais = await page.evaluate((els) => window.oracle.restoreEls(els), atuais);
          } else if (t === "montar") {
            await page.evaluate((els) => window.oracle.mount(els), atuais);
            await page.waitForTimeout(300);
            atuais = await page.evaluate(() => window.oracle.scene());
          } else if (t === "empurrar") {
            await page.evaluate((ids) => window.oracle.select(ids), passo.ids);
            await page.waitForTimeout(150);
            await page.keyboard.press("ArrowRight");
            await page.waitForTimeout(150);
            await page.keyboard.press("ArrowLeft");
            await page.waitForTimeout(300);
            atuais = await page.evaluate(() => window.oracle.scene());
          } else if (t === "desenhar") {
            if (!passo.manter) {
              await page.evaluate(() => window.oracle.select([]));
              await page.keyboard.press("Escape");
            }
            if (passo.tecla) await page.keyboard.press(passo.tecla);
            await page.waitForTimeout(100);
            for (const m of passo.mods || []) await page.keyboard.down(m);
            await page.mouse.move(passo.de[0], passo.de[1]);
            await page.mouse.down();
            // `clique_clique`: larga logo no início (seta por cliques: o 2.º clique fecha-a).
            if (passo.clique_clique) await page.mouse.up();
            // `via`: pontos intermédios (cena); `n` passos por trecho (12), `dt` ms entre passos (16),
            // `espera` ms parado no fim antes de largar (50).
            const n = passo.n || 12;
            const pts = [passo.de, ...(passo.via || []), passo.a];
            for (let k = 1; k < pts.length; k++) {
              const [p, q] = [pts[k - 1], pts[k]];
              for (let i = 1; i <= n; i++) {
                await page.mouse.move(p[0] + ((q[0] - p[0]) * i) / n, p[1] + ((q[1] - p[1]) * i) / n);
                await page.waitForTimeout(passo.dt ?? 16);
              }
            }
            await page.mouse.move(passo.a[0], passo.a[1]);
            await page.waitForTimeout(passo.espera ?? 50);
            // W10 `captura`: a meio do arrasto (antes de largar) o ecrã em PNG e as chaves do appState
            // que falam de ligação/realce — o que o editor MOSTRA enquanto se arrasta.
            if (passo.captura) {
              await page.screenshot({ path: path.join(outDir, `${prefixo}_${caso.nome}.${versao}.${passo.captura}.png`) });
              const meio = await page.evaluate(() => {
                const a = window.oracle.editorApi().getAppState();
                const ks = Object.keys(a).filter((k) => /bind|suggest|hover|highlight/i.test(k)).sort();
                return JSON.parse(JSON.stringify(Object.fromEntries(ks.map((k) => [k, a[k] ?? null]))));
              });
              // Os voláteis saem também de DENTRO do appState (o `suggestedBinding` leva o elemento inteiro).
              const semVolateis = JSON.parse(JSON.stringify(meio, (k, v) => (VOLATEIS.has(k) ? undefined : v)));
              let cena = await page.evaluate(() => window.oracle.scene());
              if (caso.relogio === "livre") cena = normaliza(cena, conhecidos);
              passos.push({ passo: { tipo: "captura", nome: passo.captura }, appState: semVolateis, elementos: limpa(cena) });
            }
            await page.mouse.up();
            if (passo.clique_final) { await page.waitForTimeout(100); await page.mouse.down(); await page.mouse.up(); }
            for (const m of passo.mods || []) await page.keyboard.up(m);
            await page.waitForTimeout(300);
            if (!passo.sem_escape) await page.keyboard.press("Escape");
            await page.waitForTimeout(100);
            atuais = await page.evaluate(() => window.oracle.scene());
          } else if (t === "estado") {
            passos.push({ passo, appState: await page.evaluate((ks) => Object.fromEntries(ks.map((k) => [k, window.oracle.editorApi().getAppState()[k] ?? null])), passo.chaves) });
            continue;
          } else if (t === "serializar") {
            atuais = await page.evaluate((els) => JSON.parse(window.oracle.api.serializeAsJSON(els, {}, {}, "local")).elements, atuais);
          } else if (t === "svg") {
            if (caso.relogio === "livre") atuais = normaliza(atuais, conhecidos);
            svg = await page.evaluate((els) => window.oracle.svg(els), atuais);
          } else {
            throw new Error(`passo desconhecido ${t}`);
          }
          // `guardar: false`: o passo corre mas não vai para a saída (W10: a varredura só guarda o fim).
          if (passo.guardar === false) continue;
          passos.push({ passo, elementos: limpa(caso.relogio === "livre" ? normaliza(atuais, conhecidos) : atuais) });
        }
      } catch (e) {
        console.error(`FALHA ${caso.nome} (${versao}): ${e.message}`);
        falhas++;
      }
      await ctx.close();
      const base = path.join(outDir, `${prefixo}_${caso.nome}.${versao}`);
      const cab = { oraculo: "@excalidraw/excalidraw", versao: PKG.version, licenca: PKG.license, chromium: browser.version(), casos: path.basename(casosPath), casos_sha256: sha(casosBytes), caso: caso.nome, descricao: caso.descricao || null, semente: seed, relogio_fixo_utc: new Date(FIXED_TIME).toISOString(), consola };
      // `"compacto": true` no caso: a saída sem indentação (a varredura W10).
      fs.writeFileSync(base + ".json", JSON.stringify({ cabecalho: cab, passos }, null, caso.compacto ? undefined : 1) + "\n");
      if (svg) fs.writeFileSync(base + ".svg", `<!-- ph2d excalidraw-oracle W9: @excalidraw/excalidraw ${PKG.version} · caso ${caso.nome} · ${path.basename(casosPath)} sha256 ${cab.casos_sha256} -->\n${svg}\n`);
      console.log(`ok ${versao} ${caso.nome}: ${passos.length} passos${consola.length ? ` · consola ${consola.length}` : ""}`);
    }
  } finally {
    await browser.close();
    server.close();
  }
}
if (falhas) process.exit(1);
