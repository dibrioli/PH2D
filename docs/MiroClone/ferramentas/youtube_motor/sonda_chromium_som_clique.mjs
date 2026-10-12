// W18 — o navegador da pessoa (Chrome) como JANELA VIRTUAL projectada no quadro: o SOM sai pelo sistema?
// e quanto tempo vai de um clique (CDP Input) a um quadro que o mostra? Uso: node … <exe> <id> <saida>
import { createRequire } from "node:module";
import fs from "node:fs"; import path from "node:path"; import { execSync } from "node:child_process";
const [exe, vid, out] = process.argv.slice(2);
fs.mkdirSync(out, { recursive: true });
const require = createRequire(path.join(process.env.HOME, "Apps/excalidraw-oracle/package.json"));
const { chromium } = require("playwright");
const browser = await chromium.launch({ executablePath: exe, headless: true, ignoreDefaultArgs: ["--mute-audio"], args: ["--autoplay-policy=no-user-gesture-required"] });
const ctx = await browser.newContext({ viewport: { width: 560, height: 315 } });
const page = await ctx.newPage();
const HTML = `<!doctype html><body style="margin:0;background:#000"><iframe width=560 height=315 frameborder=0 allow="autoplay; encrypted-media"
 referrerpolicy="strict-origin-when-cross-origin" src="https://www.youtube-nocookie.com/embed/${vid}?autoplay=1&playsinline=1"></iframe></body>`;
await page.route("https://ph2d.local/**", (r) => r.fulfill({ contentType: "text/html", body: HTML }));
await page.goto("https://ph2d.local/");
const cdp = await ctx.newCDPSession(page);
let ultimo = Date.now(), quadros = [];
cdp.on("Page.screencastFrame", async (f) => { quadros.push({ t: Date.now(), n: f.data.length, d: f.data.slice(2000, 2400) }); await cdp.send("Page.screencastFrameAck", { sessionId: f.sessionId }).catch(() => {}); });
await cdp.send("Page.startScreencast", { format: "jpeg", quality: 70 });
await page.waitForTimeout(9000);
const sinks = (() => { try { return execSync("pactl list sink-inputs").toString(); } catch (e) { return String(e); } })();
const app = [...sinks.matchAll(/application\.name = "([^"]+)"/g)].map((m) => m[1]);
const corked = [...sinks.matchAll(/Corked: (\S+)/g)].map((m) => m[1]);
// O clique: no meio do tocador (pausa), por CDP — o que o quadro faria com o rato do artista.
const lat = [];
for (let k = 0; k < 4; k++) {
  await page.waitForTimeout(1500);
  const n0 = quadros.length, t0 = Date.now();
  for (const type of ["mousePressed", "mouseReleased"]) await cdp.send("Input.dispatchMouseEvent", { type, x: 280, y: 150, button: "left", clickCount: 1 });
  // o 1.º quadro depois do clique cujo conteúdo muda de regime (pausa ⇒ deixa de mudar / retoma ⇒ volta a mudar)
  await page.waitForTimeout(1200);
  const depois = quadros.slice(n0);
  lat.push({ clique: k, quadros_em_1200ms: depois.length, primeiro_ms: depois.length ? depois[0].t - t0 : null });
}
await cdp.send("Page.stopScreencast");
fs.writeFileSync(path.join(out, "chromium_fim_som.png"), await page.screenshot());
const res = { o_que: "Chrome sem ecrã como janela virtual: som pelo sistema e clique por CDP", versao: browser.version(), data: new Date().toISOString(),
  som: { fluxos_no_sistema: app, corked }, cliques: lat };
fs.writeFileSync(path.join(out, "chromium_som_clique.json"), JSON.stringify(res, null, 1));
console.log(JSON.stringify(res));
await browser.close();
