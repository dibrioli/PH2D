// O ORÁCULO DO «ENQUADRAR TUDO» (F8) — o tldraw 5.5.2 COM a interface (a variante `www_ui`: as
// teclas vivem na UI, e `hideUi` desliga-as). Cada cena NOSSA (`cenas.mjs`) é enquadrada pela TECLA
// do produto (Shift+1 = tudo, Shift+2 = seleção) e pela API (`zoomToFit`, `zoomToSelection`), a
// partir de três câmaras; grava a câmara final, onde o conteúdo cai no ecrã (as quatro margens) e a
// câmara a CADA QUADRO depois da tecla (a animação). Números INTEIROS (lição da F7e).
// Licença: «tldraw license» (NÃO permissiva) ⇒ corre-se, nunca se lê nem se porta.
//
// uso:  PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh timeout 900 node <este> [saida_dir]
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { CENAS, PARTIDAS, VIEWPORT } from "./cenas.mjs";

const AQUI = path.dirname(new URL(import.meta.url).pathname);
const APP = path.join(os.homedir(), "Apps/tldraw-oracle");
// `ORACULO_VP=LxA` mede outra janela (a margem é fixa ou proporcional?); a saída ganha o sufixo.
const VP = process.env.ORACULO_VP ? Object.fromEntries(process.env.ORACULO_VP.split("x").map((v, i) => [i ? "height" : "width", +v])) : VIEWPORT;
const SAI = process.argv[2] || path.join(AQUI, "fixtures_tldraw" + (process.env.ORACULO_VP ? "_" + process.env.ORACULO_VP : ""));
const require = createRequire(path.join(APP, "package.json"));
const { chromium } = require("playwright");
const VERSAO = `tldraw ${require(path.join(APP, "node_modules/tldraw/package.json")).version} (tldraw license — corre-se, não se porta)`;

const WWW = path.join(APP, "www_ui");
const MIME = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".woff2": "font/woff2" };
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
fs.mkdirSync(SAI, { recursive: true });
try {
  const ctx = await browser.newContext({ viewport: VP });
  await ctx.route("**/*", (r) => (r.request().url().startsWith(origin) ? r.continue() : r.abort()));
  const page = await ctx.newPage();
  await page.goto(origin + "/");
  await page.waitForFunction(() => window.oracleReady === true, null, { timeout: 30000 });
  await page.waitForTimeout(500);

  const monta = (cena, partida) => page.evaluate(([cena, partida]) => {
    const { editor, createShapeId } = window.oracle;
    editor.setCurrentTool("select");
    const velhas = [...editor.getCurrentPageShapeIds()];
    if (velhas.length) {
      editor.updateShapes(velhas.map((id) => ({ id, type: editor.getShape(id).type, isLocked: false })));
      editor.deleteShapes(velhas);
    }
    const ids = cena.objs.map((_, i) => createShapeId("o" + i));
    editor.createShapes(cena.objs.map(([x, y, w, h, t], i) => ({
      id: ids[i], type: "geo", x, y, isLocked: (cena.tranca || []).includes(i),
      props: { geo: t === "ellipse" ? "ellipse" : "rectangle", w, h },
    })));
    editor.selectNone();
    if (cena.sel) editor.select(...cena.sel.map((i) => ids[i]));
    editor.stopCameraAnimation?.();
    editor.setCamera(partida);
    editor.focus();
    return ids;
  }, [cena, partida]);

  // A câmara a cada quadro, a partir do instante em que a ação é disparada.
  const amostra = (ms) => page.evaluate((ms) => new Promise((ok) => {
    const { editor } = window.oracle;
    const s = [];
    const t0 = performance.now();
    const f = () => {
      const c = editor.getCamera();
      s.push([performance.now() - t0, c.x, c.y, c.z]);
      if (performance.now() - t0 < ms) requestAnimationFrame(f); else ok(s);
    };
    window.__amostra = s;
    requestAnimationFrame(f);
  }), ms);

  const le = () => page.evaluate(() => {
    const { editor } = window.oracle;
    const c = editor.getCamera();
    const vp = editor.getViewportScreenBounds();
    const pb = editor.getCurrentPageBounds();
    const ids = editor.getSelectedShapeIds();
    const sb = ids.length ? editor.getSelectionPageBounds() : null;
    const ecra = (b) => b && { x0: (b.x + c.x) * c.z, y0: (b.y + c.y) * c.z, x1: (b.x + b.w + c.x) * c.z, y1: (b.y + b.h + c.y) * c.z };
    return {
      camara: { x: c.x, y: c.y, z: c.z },
      viewport: { x: vp.x, y: vp.y, w: vp.w, h: vp.h },
      tudo_pagina: pb && { x: pb.x, y: pb.y, w: pb.w, h: pb.h },
      tudo_ecra: ecra(pb),
      selecao_pagina: sb && { x: sb.x, y: sb.y, w: sb.w, h: sb.h },
      selecao_ecra: ecra(sb),
      selecionados: ids.length,
    };
  });

  const ACOES = {
    tecla_shift_1: async () => { await page.keyboard.press("Shift+Digit1"); },
    tecla_shift_2: async () => { await page.keyboard.press("Shift+Digit2"); },
    api_zoomToFit: () => page.evaluate(() => { window.oracle.editor.zoomToFit(); }),
    api_zoomToSelection: () => page.evaluate(() => { window.oracle.editor.zoomToSelection(); }),
  };

  for (const [nome, cena] of Object.entries(CENAS)) {
    const fix = {
      _cabecalho: {
        oraculo: VERSAO, cena: nome, data: new Date().toISOString().slice(0, 10),
        como: `Editor COM interface montado ${VP.width}×${VP.height}; formas geo criadas pela API (isLocked onde «tranca»), seleção pela API, câmara de partida por setCamera, editor.focus(); a ação é a TECLA pelo teclado do Playwright ou a função da API; a câmara amostrada a cada requestAnimationFrame durante 1500 ms`,
        unidade: "px de página e px de ecrã (ecrã = (página + câmara.xy) · câmara.z)",
        objectos: cena,
      },
      corridas: {},
    };
    for (const [pnome, partida] of Object.entries(PARTIDAS)) {
      for (const [anome, acao] of Object.entries(ACOES)) {
        await monta(cena, partida);
        await page.waitForTimeout(60);
        const antes = await le();
        const pAm = amostra(1500);
        pAm.catch(() => {}); // um erro da AÇÃO não pode ficar escondido atrás do da amostra
        await acao();
        const quadros = await pAm;
        await page.waitForTimeout(100);
        const depois = await le();
        fix.corridas[`${pnome}/${anome}`] = { antes: antes.camara, depois, quadros };
      }
    }
    fs.writeFileSync(path.join(SAI, nome + ".json"), JSON.stringify(fix, null, 1) + "\n");
    const r = fix.corridas["z1_origem/tecla_shift_1"];
    console.log(`${nome.padEnd(18)} shift+1 → z ${r.depois.camara.z} · ${r.quadros.length} quadros`);
  }
} finally {
  await browser.close();
  server.close();
}
