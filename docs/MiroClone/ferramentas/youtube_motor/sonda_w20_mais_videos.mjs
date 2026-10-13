// W20 — o dono (13/10): «"Mais vídeos" abre um vídeo no navegador e ambos ficam tocando sobrepostos. Seria possível
// navegar no próprio quadro?». Pergunta ao tocador oficial, conduzido como o PH2D (a nossa página em
// https://ph2d.local/, os nossos parâmetros): o botão «Mais vídeos» abre o quê? e o clique numa sugestão — abre uma
// ABA NOVA (com que endereço) ou a moldura navega? o vídeo de origem PAUSA? Fotos directas a cada passo.
// Uso: node sonda_w20_mais_videos.mjs <exe-do-chrome> <id> <saida>
import { createRequire } from "node:module";
import fs from "node:fs"; import path from "node:path";
const [exe, vid, out] = process.argv.slice(2);
fs.mkdirSync(out, { recursive: true });
const require = createRequire(path.join(process.env.HOME, "Apps/excalidraw-oracle/package.json"));
const { chromium } = require("playwright");
const src = `https://www.youtube-nocookie.com/embed/${vid}?autoplay=1&playsinline=1&enablejsapi=1&start=0&controls=0&fs=0&disablekb=1&rel=0&iv_load_policy=3`;
const html = `<!doctype html><html><body style="margin:0;background:#000;overflow:hidden">
<iframe style="border:0;width:100vw;height:100vh" allow="autoplay; encrypted-media" referrerpolicy="strict-origin-when-cross-origin" src="${src}"></iframe></body></html>`;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const browser = await chromium.launch({ executablePath: exe, headless: true, args: ["--autoplay-policy=no-user-gesture-required", "--hide-scrollbars"] });
const ctx = await browser.newContext({ viewport: null, locale: "pt-BR" });
const abas = [];
ctx.on("page", (p) => abas.push({ t: Date.now(), url: p.url() }));
const page = await ctx.newPage();
const cdp = await ctx.newCDPSession(page);
await cdp.send("Emulation.setDeviceMetricsOverride", { width: 560, height: 315, deviceScaleFactor: 1, mobile: false, screenWidth: 560, screenHeight: 315 });
await page.route("https://ph2d.local/**", (r) => r.fulfill({ contentType: "text/html", body: html }));
await page.goto("https://ph2d.local/");
const yt = () => page.frames().find((x) => /youtube(-nocookie)?\.com/.test(x.url()));
const res = { o_que: "W20: o «Mais vídeos» do tocador (sonda_w20_mais_videos.mjs)", video: vid, versao: browser.version(), data: new Date().toISOString(), passos: [] };
const estado = async () => {
  const f = yt();
  if (!f) return { moldura: null };
  return f.evaluate(() => {
    const v = document.querySelector("video");
    const vis = (b) => { const r = b.getBoundingClientRect(); const cs = getComputedStyle(b); return r.width > 0 && r.height > 0 && cs.visibility !== "hidden" && cs.display !== "none" && Number(cs.opacity) > 0.05; };
    const nomes = [...document.querySelectorAll("a, button, [role=button]")].filter(vis)
      .map((b) => { const r = b.getBoundingClientRect(); return { tag: b.tagName, nome: (b.getAttribute("aria-label") || b.innerText || b.title || "").trim().slice(0, 60), href: b.getAttribute("href")?.slice(0, 120) ?? null, alvo: b.getAttribute("target"), x: Math.round(r.x + r.width / 2), y: Math.round(r.y + r.height / 2) }; });
    return { url: location.href.slice(0, 160), paused: v?.paused ?? null, t: v ? Math.round(v.currentTime * 10) / 10 : null, nomes };
  });
};
const passo = async (nome, extra = {}) => {
  const s = await cdp.send("Page.captureScreenshot", { format: "png" });
  fs.writeFileSync(path.join(out, `${nome}.png`), Buffer.from(s.data, "base64"));
  const e = await estado();
  res.passos.push({ passo: nome, ...e, abas: abas.slice(), ...extra });
  return e;
};
await sleep(5000);
await page.mouse.move(280, 157); await sleep(600);
let e = await passo("0_a_tocar_rato_por_cima");
const mais = e.nomes?.find((n) => /mais v[ií]deos|more videos/i.test(n.nome));
if (!mais) { await passo("sem_mais_videos"); }
else {
  await page.mouse.click(mais.x, mais.y); await sleep(1500);
  e = await passo("1_painel_aberto", { clicou: mais });
  // A 1.ª sugestão: um link para um vídeo (watch / embed / youtu.be) que não seja o próprio.
  const sug = e.nomes?.find((n) => n.href && /watch\?v=|\/embed\/|youtu\.be\//.test(n.href) && !n.href.includes(vid));
  if (sug) {
    await page.mouse.click(sug.x, sug.y); await sleep(4000);
    await passo("2_clicou_sugestao", { clicou: sug });
    await sleep(3000);
    await passo("3_depois_3s");
  } else {
    res.passos.push({ passo: "sem_sugestao_com_link", nomes: e.nomes });
  }
}
fs.writeFileSync(path.join(out, "w20_mais_videos.json"), JSON.stringify(res, null, 1) + "\n");
for (const p of res.passos) console.log(JSON.stringify({ p: p.passo, url: p.url, paused: p.paused, t: p.t, abas: p.abas?.map((a) => a.url.slice(0, 100)), clicou: p.clicou ? { nome: p.clicou.nome, href: p.clicou.href, alvo: p.clicou.alvo } : undefined }));
await browser.close();
