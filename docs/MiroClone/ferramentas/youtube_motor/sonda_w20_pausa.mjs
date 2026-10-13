// W20 — o dono (13/10): «ao apertar o botão de pause, o ícone de play deveria aparecer». O nosso `Player` mostrou que,
// PAUSADO, o screencast não manda NENHUM quadro novo (o desenho fica com o último, o «‖» de quando tocava). Pergunta:
// a PÁGINA muda ao pausar (e é o screencast que não a manda) ou o tocador com os nossos parâmetros não muda nada?
// Conduzido como o PH2D (a nossa página em https://ph2d.local/, CDP screencast com ack), com fotos DIRECTAS
// (`Page.captureScreenshot`, que não depende do screencast) e o botão do meio lido pelo DOM do tocador.
// Uso: node sonda_w20_pausa.mjs <exe-do-chrome> <id> <saida>
import { createRequire } from "node:module";
import fs from "node:fs"; import path from "node:path";
const [exe, vid, out] = process.argv.slice(2);
fs.mkdirSync(out, { recursive: true });
const require = createRequire(path.join(process.env.HOME, "Apps/excalidraw-oracle/package.json"));
const { chromium } = require("playwright");
const src = `https://www.youtube-nocookie.com/embed/${vid}?autoplay=1&playsinline=1&enablejsapi=1&start=0&controls=0&fs=0&disablekb=1&rel=0&iv_load_policy=3`;
const html = `<!doctype html><html><body style="margin:0;background:#000;overflow:hidden">
<iframe style="border:0;width:100vw;height:100vh" allow="autoplay; encrypted-media" referrerpolicy="strict-origin-when-cross-origin" src="${src}"></iframe>
<script>const f=document.querySelector('iframe');
self.ph2dCmd=(func,args)=>f.contentWindow.postMessage(JSON.stringify({event:"command",func,args:args||[],id:1,channel:"widget"}),'*');
setInterval(()=>f.contentWindow.postMessage('{"event":"listening","id":1,"channel":"widget"}','*'),250);</script></body></html>`;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
// MONTAGEM=ph2d: como o nosso `Player` — o contexto POR OMISSÃO (perfil persistente, com o about:blank inicial) e a
// aba do tocador como SEGUNDA aba; senão o contexto novo do Playwright.
const ARGS = ["--autoplay-policy=no-user-gesture-required", "--hide-scrollbars"];
const PH2D = process.env.MONTAGEM === "ph2d";
const persist = PH2D ? await chromium.launchPersistentContext(fs.mkdtempSync(path.join(process.env.TMPDIR || "/tmp", "w20p-")), { executablePath: exe, headless: true, args: ARGS, viewport: null }) : null;
const browser = PH2D ? persist.browser() ?? { version: () => "persistente", close: () => persist.close() } : await chromium.launch({ executablePath: exe, headless: true, args: ARGS });
const ctx = PH2D ? persist : await browser.newContext({ viewport: null });
if (PH2D) console.error(`abas antes: ${ctx.pages().length}`);
const page = await ctx.newPage();
const cdp = await ctx.newCDPSession(page);
await cdp.send("Emulation.setDeviceMetricsOverride", { width: 560, height: 315, deviceScaleFactor: 1, mobile: false, screenWidth: 560, screenHeight: 315 });
await page.route("https://ph2d.local/**", (r) => r.fulfill({ contentType: "text/html", body: html }));
let quadros = 0;
cdp.on("Page.screencastFrame", async (f) => { quadros++; await cdp.send("Page.screencastFrameAck", { sessionId: f.sessionId }).catch(() => {}); });
await page.goto("https://ph2d.local/");
await cdp.send("Page.startScreencast", { format: "jpeg", quality: 85, maxWidth: 560, maxHeight: 315, everyNthFrame: 1 });
let yt; for (let k = 0; k < 60 && !yt; k++) { yt = page.frames().find((x) => x.url().includes("youtube")); await sleep(100); }
const estado = async () => yt.evaluate(() => {
  const v = document.querySelector("video");
  // Os botões visíveis ao meio (o nome acessível diz se é play ou pausa).
  const meio = [...document.querySelectorAll("button, [role=button]")].map((b) => { const r = b.getBoundingClientRect(); const cs = getComputedStyle(b);
    return { nome: b.getAttribute("aria-label") || b.getAttribute("title") || b.className.toString().slice(0, 60), x: Math.round(r.x + r.width / 2), y: Math.round(r.y + r.height / 2), w: Math.round(r.width), vis: cs.visibility !== "hidden" && cs.display !== "none" && Number(cs.opacity) > 0.05 && r.width > 0 }; })
    .filter((b) => b.vis && Math.abs(b.x - 280) < 80 && Math.abs(b.y - 157) < 80);
  return { paused: v?.paused ?? null, t: v ? Math.round(v.currentTime * 10) / 10 : null, meio };
});
const res = { o_que: "W20: o que o tocador mostra ao PAUSAR (sonda_w20_pausa.mjs)", video: vid, versao: browser.version(), data: new Date().toISOString(), passos: [] };
const foto = async (nome, extra = {}) => {
  const q0 = quadros;
  const s = await cdp.send("Page.captureScreenshot", { format: "png" });
  fs.writeFileSync(path.join(out, `${nome}.png`), Buffer.from(s.data, "base64"));
  res.passos.push({ passo: nome, ...(await estado()), quadros_screencast_ate_aqui: q0, ...extra });
};
await sleep(5000);
await page.mouse.move(5, 5); await page.mouse.move(-1, -1).catch(() => {});
await foto("0_a_tocar");
// A: pela API (a nossa barra). O rato FORA do tocador.
let q = quadros; await page.evaluate(() => self.ph2dCmd("pauseVideo")); await sleep(1000);
await foto("1_api_1s", { quadros_no_segundo: quadros - q });
q = quadros; await sleep(4000); await foto("2_api_5s", { quadros_nos_4s: quadros - q });
await page.evaluate(() => self.ph2dCmd("playVideo")); await sleep(4000);
// B: o clique ao meio (o clique no vídeo activo), depois o rato sai.
q = quadros; await page.mouse.click(280, 157); await sleep(1000);
await foto("3_clique_1s", { quadros_no_segundo: quadros - q });
await page.mouse.move(-1, -1).catch(() => {}); q = quadros; await sleep(4000);
await foto("4_clique_5s_rato_fora", { quadros_nos_4s: quadros - q });
// C: o rato volta por cima, pausado.
q = quadros; await page.mouse.move(280, 157); await sleep(800);
await foto("5_rato_por_cima_pausado", { quadros: quadros - q });
fs.writeFileSync(path.join(out, "w20_pausa.json"), JSON.stringify(res, null, 1) + "\n");
console.log(JSON.stringify(res.passos.map((p) => ({ p: p.passo, paused: p.paused, meio: p.meio.map((b) => b.nome), q: p.quadros_no_segundo ?? p.quadros_nos_4s ?? p.quadros }))));
await (PH2D ? persist.close() : browser.close());
