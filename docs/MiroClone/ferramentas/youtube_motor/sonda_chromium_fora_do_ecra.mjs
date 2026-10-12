// W18 — o motor Chromium (o mesmo do CEF) a tocar o embed do YouTube SEM ecrã, entregando quadros por
// CDP (Page.startScreencast = o caminho «off-screen rendering» que o CEF expõe). Mede: o vídeo anda?
// quadros/s entregues, CPU e RSS da árvore do navegador. Uso: node sonda_chromium_fora_do_ecra.mjs <exe> <id> <segundos> <saida>
import { createRequire } from "node:module";
import fs from "node:fs"; import path from "node:path"; import { execSync } from "node:child_process";
const [exe, vid, segs, out] = process.argv.slice(2);
fs.mkdirSync(out, { recursive: true });
const require = createRequire(path.join(process.env.HOME, "Apps/excalidraw-oracle/package.json"));
const { chromium } = require("playwright");
const browser = await chromium.launch({ executablePath: exe, headless: true, args: ["--autoplay-policy=no-user-gesture-required"] });
const ctx = await browser.newContext({ viewport: { width: 560, height: 315 }, deviceScaleFactor: 1 });
const page = await ctx.newPage();
const HTML = `<!doctype html><body style="margin:0;background:#000"><iframe width=560 height=315 frameborder=0 allow="autoplay; encrypted-media" referrerpolicy="strict-origin-when-cross-origin"
  src="https://www.youtube-nocookie.com/embed/${vid}?autoplay=1&mute=1&playsinline=1"></iframe></body>`;
await page.route("https://ph2d.local/**", (r) => r.fulfill({ contentType: "text/html", body: HTML }));
await page.goto("https://ph2d.local/");
const _unused = (`<body style="margin:0;background:#000"><iframe width=560 height=315 frameborder=0 allow="autoplay; encrypted-media"
  src="https://www.youtube-nocookie.com/embed/${vid}?autoplay=1&mute=1&playsinline=1"></iframe></body>`);
const cdp = await ctx.newCDPSession(page);
let n = 0, t0 = 0, fotos = [];
cdp.on("Page.screencastFrame", async (f) => {
  n++; const t = (Date.now() - t0) / 1000;
  if (t > 5 && fotos.length < 4 && n % 90 === 0) { const p = path.join(out, `chromium_${fotos.length}.jpg`); fs.writeFileSync(p, Buffer.from(f.data, "base64")); fotos.push(p); }
  await cdp.send("Page.screencastFrameAck", { sessionId: f.sessionId }).catch(() => {});
});
const pids = () => { try { return execSync(`pgrep -f -- '${process.env.ARVORE || exe}'`).toString().trim().split("\n").map(Number); } catch { return []; } };
const cpu = () => pids().reduce((a, p) => { try { const s = fs.readFileSync(`/proc/${p}/stat`, "utf8").split(") ")[1].split(" "); return a + (+s[11] + +s[12]); } catch { return a; } }, 0) / 100;
const rss = () => pids().reduce((a, p) => { try { return a + (+fs.readFileSync(`/proc/${p}/statm`, "utf8").split(" ")[1]) * 4096; } catch { return a; } }, 0) / 1048576;
await page.waitForTimeout(5000);
t0 = Date.now(); const c0 = cpu(); n = 0;
await cdp.send("Page.startScreencast", { format: "jpeg", quality: 70, everyNthFrame: 1 });
await page.waitForTimeout(+segs * 1000);
await cdp.send("Page.stopScreencast");
fs.writeFileSync(path.join(out, "chromium_fim.png"), await page.screenshot());
const dur = (Date.now() - t0) / 1000, c1 = cpu(), r = rss();
const res = { o_que: "Chromium sem ecrã + CDP screencast a tocar o embed do YouTube (proxy do CEF off-screen)", exe, versao: browser.version(),
  video: vid, data: new Date().toISOString(), segundos: dur, quadros: n, fps: +(n / dur).toFixed(2), cpu_nucleos: +((c1 - c0) / dur).toFixed(2), rss_mb_arvore: +r.toFixed(0), fotos };
fs.writeFileSync(path.join(out, `chromium_${path.basename(exe)}.json`), JSON.stringify(res, null, 1));
console.log(JSON.stringify(res));
await browser.close();
