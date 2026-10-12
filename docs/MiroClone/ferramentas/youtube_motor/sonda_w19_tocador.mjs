// W19 — o tocador oficial no Chrome sem ecrã, conduzido como o PH2D o conduz (CDP: métricas da caixa, screencast,
// rato por Input.dispatch*), sobre a NOSSA página em https://ph2d.local/. Uma corrida responde às cinco perguntas
// do smoke do dono (12/10):
//   P — o que cada parâmetro oficial (controls=0, fs=0, disablekb=1, rel=0, iv_load_policy=3) tira e o que fica;
//   F — o botão de ecrã inteiro DO TOCADOR: que tamanho de quadro o screencast passa a mandar;
//   Z — um zoom contínuo 1×→2×→1× em 5 s: o tocador pausa/re-bufferiza? (variantes intercaladas no MESMO processo);
//   C — clicar no canal, no título, no «YouTube», nas sugestões: nasce um alvo? a moldura navega? fica preto?
//   A — o estado pela API oficial (listening → infoDelivery/onStateChange) e as ordens (mute, volume, seekTo…),
//       trazido por Runtime.addBinding; e se há anúncio (a linha de base do Premium).
// Uso: node … <exe> <id> <saida> [perfil]
import { createRequire } from "node:module";
import fs from "node:fs"; import path from "node:path"; import crypto from "node:crypto";
const [exe, vid, out, profile] = process.argv.slice(2);
fs.mkdirSync(out, { recursive: true });
const require = createRequire(path.join(process.env.HOME, "Apps/excalidraw-oracle/package.json"));
const { chromium } = require("playwright");
const BASE = `https://www.youtube-nocookie.com/embed/${vid}?autoplay=1&playsinline=1&enablejsapi=1&start=0`;
const OURS = `${BASE}&controls=0&fs=0&disablekb=1&rel=0&iv_load_policy=3`;
const args = ["--autoplay-policy=no-user-gesture-required", "--hide-scrollbars"];
const browser = profile
  ? null
  : await chromium.launch({ executablePath: exe, headless: true, args });
const persist = profile
  ? await chromium.launchPersistentContext(profile, { executablePath: exe, headless: true, args, viewport: null })
  : null;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const res = { o_que: "W19: o tocador oficial conduzido como o PH2D (sonda_w19_tocador.mjs)", exe, video: vid,
  data: new Date().toISOString(), versao: browser ? browser.version() : "perfil persistente" };

// A NOSSA página: o iframe a encher a aba; as mensagens do tocador vão para o binding `ph2dState`.
const html = (src) => `<!doctype html><html><body style="margin:0;background:#000;overflow:hidden">
<iframe style="border:0;width:100vw;height:100vh" allow="autoplay; encrypted-media; fullscreen"
 referrerpolicy="strict-origin-when-cross-origin" src="${src}"></iframe>
<script>
addEventListener('message', (e) => { if (typeof e.data !== 'string') return; if (self.ph2dState) self.ph2dState(e.data);
  if (e.data.includes('"event":"onReady"')) f.contentWindow.postMessage('{"event":"command","func":"addEventListener","args":["onStateChange"],"id":1,"channel":"widget"}', '*'); });
const f = document.querySelector('iframe');
let ouvir = setInterval(() => f.contentWindow.postMessage('{"event":"listening","id":1,"channel":"widget"}', '*'), 250);
self.ph2dCmd = (func, args) => f.contentWindow.postMessage(JSON.stringify({event:"command",func,args:args||[],id:1,channel:"widget"}), '*');
self.ph2dParar = () => clearInterval(ouvir);
</script></body></html>`;

async function abre(src, w, h) {
  const ctx = persist ?? (await browser.newContext({ viewport: null }));
  const page = await ctx.newPage();
  const cdp = await ctx.newCDPSession(page);
  const msgs = [];
  const t0 = Date.now();
  await cdp.send("Runtime.enable");
  await cdp.send("Runtime.addBinding", { name: "ph2dState" });
  cdp.on("Runtime.bindingCalled", (e) => { if (e.name === "ph2dState") { try { msgs.push({ t: Date.now() - t0, m: JSON.parse(e.payload) }); } catch {} } });
  // ECRA=1: o «ecrã» do navegador passa a ser a caixa (o ecrã inteiro do tocador não tem outra proporção).
  const ecra = process.env.ECRA === "1" ? { screenWidth: w, screenHeight: h } : {};
  await cdp.send("Emulation.setDeviceMetricsOverride", { width: w, height: h, deviceScaleFactor: 1, mobile: false, ...ecra });
  await page.route("https://ph2d.local/**", (r) => r.fulfill({ contentType: "text/html", body: html(src) }));
  const quadros = [];
  cdp.on("Page.screencastFrame", async (f) => {
    const b = Buffer.from(f.data, "base64");
    quadros.push({ t: Date.now() - t0, w: f.metadata.deviceWidth, h: f.metadata.deviceHeight, jpeg: jpegSize(b), n: b.length,
      hash: crypto.createHash("md5").update(b).digest("hex").slice(0, 10) });
    await cdp.send("Page.screencastFrameAck", { sessionId: f.sessionId }).catch(() => {});
  });
  await page.goto("https://ph2d.local/");
  const go = async (sz) => cdp.send("Page.startScreencast", { format: "jpeg", quality: 85, maxWidth: sz[0], maxHeight: sz[1], everyNthFrame: 1 });
  await go([w, h]);
  const yt = async () => { for (let k = 0; k < 40; k++) { const f = page.frames().find((x) => x.url().includes("youtube")); if (f) return f; await sleep(100); } };
  return { ctx, page, cdp, msgs, quadros, t0, go, frame: await yt() };
}
const fecha = async (s) => { await s.cdp.send("Page.stopScreencast").catch(() => {}); await s.page.close(); if (!persist) await s.ctx.close(); };

// O tamanho de um JPEG pelo marcador SOF (o que o PH2D descodifica de facto).
function jpegSize(b) {
  for (let i = 2; i < b.length - 9;) {
    if (b[i] !== 0xff) return null;
    const m = b[i + 1], len = b.readUInt16BE(i + 2);
    if (m >= 0xc0 && m <= 0xc3) return [b.readUInt16BE(i + 7), b.readUInt16BE(i + 5)];
    i += 2 + len;
  }
  return null;
}
// O estado do tocador a partir das mensagens (infoDelivery.playerState e onStateChange).
const estados = (msgs) => msgs.flatMap(({ t, m }) => m.event === "onStateChange" ? [{ t, s: m.info }]
  : m.event === "infoDelivery" && m.info && "playerState" in m.info ? [{ t, s: m.info.playerState }] : []);
const ultimoInfo = (msgs) => Object.assign({}, ...msgs.flatMap((x) => (x.m.event === "infoDelivery" || x.m.event === "initialDelivery") && typeof x.m.info === "object" ? [x.m.info]
  : x.m.event === "onStateChange" ? [{ playerState: x.m.info }] : []));

// O que se vê dentro do tocador: cada seletor conhecido e cada link, com o retângulo e a visibilidade.
const SEL = { barra: ".ytp-chrome-bottom", ecra_inteiro: ".ytp-fullscreen-button", canal: ".ytp-title-channel-logo, .ytp-title-channel",
  titulo: ".ytp-title-link", logo_youtube: ".ytp-youtube-button", marca: ".ytp-watermark", sugestoes_pausa: ".ytp-pause-overlay",
  ecra_final: ".ytp-endscreen-content", anuncio: ".ad-showing, .ytp-ad-module :is(.ytp-ad-player-overlay, .ytp-ad-text)",
  cartoes: ".ytp-cards-button, .iv-branding", mais_videos: ".ytp-more-videos-view, .ytp-expand-pause-overlay" };
async function inventario(frame) {
  return frame.evaluate((SEL) => {
    const vis = (e) => { const r = e.getBoundingClientRect(); const cs = getComputedStyle(e);
      let op = 1; for (let x = e; x; x = x.parentElement) op *= Number(getComputedStyle(x).opacity);
      return { x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.width), h: Math.round(r.height),
        visivel: r.width > 0 && r.height > 0 && cs.display !== "none" && cs.visibility !== "hidden" && op > 0.01, opacidade: Math.round(op * 100) / 100 }; };
    const s = {};
    for (const [k, q] of Object.entries(SEL)) { const e = document.querySelector(q); s[k] = e ? vis(e) : null; }
    const links = [...document.querySelectorAll("a[href]")].map((a) => ({ href: a.href.slice(0, 90), classe: String(a.className).slice(0, 60), target: a.target, ...vis(a) }))
      .filter((l) => l.visivel);
    const mp = document.querySelector("#movie_player");
    // O tocador NOVO (2026: classes `ytm*`/`ytSpec*`) — tudo o que se clica, com o nome que aparece.
    const interativos = [...document.querySelectorAll("a[href], button, [role=button]")].map((e) => ({
      tag: e.tagName.toLowerCase(), aria: (e.getAttribute("aria-label") || e.getAttribute("title") || "").slice(0, 50),
      texto: (e.innerText || "").trim().replace(/\s+/g, " ").slice(0, 40), classe: String(e.className).slice(0, 60),
      href: e.href ? e.href.slice(0, 90) : null, ...vis(e) })).filter((l) => l.visivel);
    return { seletores: s, links_visiveis: links, interativos, classes_do_tocador: mp ? String(mp.className).split(/\s+/).filter((c) => /ad|autohide|paused|playing|fullscreen|ended/.test(c)) : null };
  }, SEL).catch((e) => ({ erro: String(e) }));
}
async function clica(cdp, x, y) {
  await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
  for (const type of ["mousePressed", "mouseReleased"]) await cdp.send("Input.dispatchMouseEvent", { type, x, y, button: "left", clickCount: 1 });
}
const centro = (r) => [r.x + r.w / 2, r.y + r.h / 2];
const FASES = (process.env.FASES || "P,F,Z,C,A").split(",");
// Os alvos no tocador NOVO, pelo que aparece na tela (o nome acessível, o texto, o link).
const ALVOS = {
  ecra_inteiro: (i) => /tela cheia|ecrã inteiro|full ?screen/i.test(i.aria),
  avatar: (i) => i.x < 80 && i.y < 70 && i.w < 80,
  canal: (i) => /ChannelTitle/.test(i.classe) || (/\/channel\//.test(i.href || "") && i.x >= 70),
  titulo: (i) => /VideoTitle/.test(i.classe),
  logo_youtube: (i) => i.x > 380 && i.y > 240 && /watch/.test(i.href || ""),
  mais_videos: (i) => /mais v[ií]deos|more videos/i.test(i.texto + " " + i.aria),
  copiar_link: (i) => /link|copiar|copy/i.test(i.aria) && i.x < 120 && i.y > 220,
};
const GEO = () => { const v = document.querySelector("video"); const r = v?.getBoundingClientRect();
  return { ecra: [screen.width, screen.height], janela: [innerWidth, innerHeight], video_rect: r && [r.x, r.y, r.width, r.height].map(Math.round),
    video_px: v && [v.videoWidth, v.videoHeight], object_fit: v && getComputedStyle(v).objectFit }; };
const acha = (inv, alvo) => (inv.interativos || []).find(ALVOS[alvo]) ?? null;
const shot = async (s, nome) => { await s.page.screenshot({ path: path.join(out, nome) }).catch(() => {}); return nome; };

// ── P: os parâmetros ──────────────────────────────────────────────────────────────────────────────
res.P = {};
if (FASES.includes("P")) for (const [nome, src] of [["hoje", BASE], ["nossos", OURS]]) {
  const s = await abre(src, 560, 315);
  await sleep(7000);
  const a_tocar = await inventario(s.frame); await shot(s, `P_${nome}_a_tocar.png`);
  await s.cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: 280, y: 290 }); await sleep(400);
  const rato_em_cima = await inventario(s.frame); await shot(s, `P_${nome}_rato.png`);
  await s.page.evaluate(() => self.ph2dCmd("pauseVideo")); await sleep(1500);
  const pausado = await inventario(s.frame); await shot(s, `P_${nome}_pausado.png`);
  res.P[nome] = { url: src, a_tocar, rato_em_cima, pausado, estado: estados(s.msgs).slice(-3) };
  await fecha(s);
}

// ── F: o ecrã inteiro do tocador (com os parâmetros de hoje), numa caixa 16:9 e numa 4:3 ─────────────
res.F = {};
if (FASES.includes("F")) for (const [w, h] of [[560, 315], [480, 360]]) {
  const s = await abre(BASE, w, h);
  await sleep(6000);
  await s.cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: w / 2, y: h - 20 }); await sleep(400);
  const inv = await inventario(s.frame);
  const geo_antes = await s.frame.evaluate(GEO).catch((e) => String(e));
  await s.page.evaluate(() => self.ph2dCmd("pauseVideo")); await sleep(1200);
  await s.cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: w / 2, y: h - 20 }); await sleep(400);
  await shot(s, `F_${w}x${h}_antes.png`);
  const b = acha(inv, "ecra_inteiro");
  const n0 = s.quadros.length;
  if (b) await clica(s.cdp, ...centro(b));
  await sleep(2500);
  const fs_el = await s.frame.evaluate(() => !!document.fullscreenElement).catch(() => null);
  const geo_depois = await s.frame.evaluate(GEO).catch((e) => String(e));
  const pagina_fs = await s.page.evaluate(() => !!document.fullscreenElement).catch(() => null);
  const antes = s.quadros.slice(Math.max(0, n0 - 3), n0).map(({ w, h, jpeg }) => ({ w, h, jpeg }));
  const depois = s.quadros.slice(n0);
  const tamanhos = [...new Set(depois.map((q) => `${q.w}x${q.h} jpeg ${q.jpeg?.join("x")}`))];
  const janela = await s.cdp.send("Browser.getWindowForTarget").catch((e) => ({ erro: String(e) }));
  await shot(s, `F_${w}x${h}_depois.png`);
  res.F[`${w}x${h}`] = { botao: b, clicou: !!b, quadros_antes: antes, tamanhos_depois: tamanhos, quadros_depois: depois.length,
    fullscreen_no_iframe: fs_el, fullscreen_na_pagina: pagina_fs, janela, geo_antes, geo_depois };
  await fecha(s);
}
// F2 — com os NOSSOS parâmetros: o duplo-clique e a tecla «f» ainda põem o tocador em ecrã inteiro?
if (FASES.includes("F")) for (const gesto of ["duplo_clique", "tecla_f"]) {
  const s = await abre(OURS, 560, 315);
  await sleep(6000);
  if (gesto === "duplo_clique") for (const c of [1, 2]) for (const type of ["mousePressed", "mouseReleased"])
    await s.cdp.send("Input.dispatchMouseEvent", { type, x: 140, y: 160, button: "left", clickCount: c });
  else { await clica(s.cdp, 140, 160); await sleep(300); await clica(s.cdp, 140, 160);
    for (const type of ["keyDown", "keyUp"]) await s.cdp.send("Input.dispatchKeyEvent", { type, key: "f", code: "KeyF", windowsVirtualKeyCode: 70, text: type === "keyDown" ? "f" : undefined }); }
  await sleep(1500);
  res.F[`nossos_${gesto}`] = { fullscreen_no_iframe: await s.frame.evaluate(() => !!document.fullscreenElement).catch(() => null), geo: await s.frame.evaluate(GEO).catch((e) => String(e)) };
  await shot(s, `F_nossos_${gesto}.png`);
  await fecha(s);
}

// ── Z: zoom contínuo 1×→2×→1× em 5 s, variantes intercaladas em blocos rodados ───────────────────────
// fixo: nada muda · hoje: a cada 16 ms métricas + parar/recomeçar o screencast (o que o PH2D faz) ·
// so_metricas: a cada 16 ms só as métricas · assenta: o tamanho só muda 250 ms depois do último passo.
res.Z = [];
const ordem = ["fixo", "hoje", "so_metricas", "assenta", "assenta", "so_metricas", "hoje", "fixo"];
if (FASES.includes("Z")) for (const v of ordem) {
  const s = await abre(OURS, 560, 315);
  await sleep(6000);
  const m0 = s.msgs.length, q0 = s.quadros.length, T = 5000, ini = Date.now();
  let ultimo = null, aplicado = [560, 315], passos = 0;
  while (Date.now() - ini < T) {
    const k = 1 + Math.sin(Math.PI * (Date.now() - ini) / T);
    const sz = [Math.round(560 * k), Math.round(315 * k)];
    if (v === "hoje" || v === "so_metricas") {
      await s.cdp.send("Emulation.setDeviceMetricsOverride", { width: sz[0], height: sz[1], deviceScaleFactor: 1, mobile: false });
      if (v === "hoje") { await s.cdp.send("Page.stopScreencast"); await s.go(sz); }
      passos++;
    }
    ultimo = sz;
    await sleep(16);
  }
  if (v === "assenta") {
    await sleep(250);
    await s.cdp.send("Emulation.setDeviceMetricsOverride", { width: ultimo[0], height: ultimo[1], deviceScaleFactor: 1, mobile: false });
    passos++;
  }
  await sleep(1500);
  const est = estados(s.msgs.slice(m0));
  const qs = s.quadros.slice(q0);
  const distintos = new Set(qs.map((q) => q.hash)).size;
  res.Z.push({ variante: v, passos, estados: [...new Set(est.map((e) => e.s))], pausas: est.filter((e) => e.s === 2).length,
    buffering: est.filter((e) => e.s === 3).length, quadros: qs.length, distintos_por_s: Math.round(distintos / ((T + 1500) / 1000) * 10) / 10,
    tempo_final: ultimoInfo(s.msgs).currentTime });
  await fecha(s);
}

// ── C: os links dentro do tocador ─────────────────────────────────────────────────────────────────
res.C = [];
const ctxEventos = async (s) => {
  const ev = [];
  s.ctx.on("page", (p) => ev.push({ t: Date.now() - s.t0, novo_alvo: p.url() }));
  s.page.on("framenavigated", (f) => ev.push({ t: Date.now() - s.t0, navegou: f === s.page.mainFrame() ? "pagina" : "moldura", url: f.url().slice(0, 100) }));
  s.page.on("popup", (p) => ev.push({ t: Date.now() - s.t0, popup: p.url() }));
  return ev;
};
if (FASES.includes("C")) for (const [params, src] of [["hoje", BASE], ["nossos", OURS]]) {
  for (const alvo of ["avatar", "canal", "titulo", "logo_youtube", "mais_videos", "copiar_link"]) {
    const s = await abre(src, 560, 315);
    const ev = await ctxEventos(s);
    await sleep(6000);
    await s.cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: 280, y: 160 }); await sleep(400);
    const inv = await inventario(s.frame);
    const r = acha(inv, alvo);
    const q0 = s.quadros.length;
    if (r) await clica(s.cdp, ...centro(r));
    await sleep(3000);
    const qs = s.quadros.slice(q0);
    const pags = s.ctx.pages().map((p) => p.url().slice(0, 100));
    const moldura = s.page.frames().map((f) => f.url().slice(0, 100));
    const est = estados(s.msgs);
    await shot(s, `C_${params}_${alvo}.png`);
    res.C.push({ params, alvo, achado: r ?? null, eventos: ev, paginas_depois: pags, molduras_depois: moldura,
      quadros_depois: qs.length, jpeg_medio_bytes: qs.length ? Math.round(qs.reduce((a, q) => a + q.n, 0) / qs.length) : null,
      ultimo_estado: est.at(-1)?.s ?? null });
    for (const p of s.ctx.pages()) if (p !== s.page) await p.close().catch(() => {});
    await fecha(s);
  }
}

// ── A: o estado e as ordens pela API oficial ──────────────────────────────────────────────────────
if (FASES.includes("A")) {
  const s = await abre(OURS, 560, 315);
  await sleep(6000);
  const eventos = [...new Set(s.msgs.map((x) => x.m.event))];
  const chaves = [...new Set(s.msgs.filter((x) => x.m.event === "infoDelivery").flatMap((x) => Object.keys(x.m.info || {})))];
  const n3 = s.msgs.filter((x) => x.t > 3000 && x.t <= 6000 && x.m.event === "infoDelivery").length;
  const espera = async (pred, ms = 3000) => { const t = Date.now(); while (Date.now() - t < ms) { if (pred(ultimoInfo(s.msgs))) return Date.now() - t; await sleep(5); } return null; };
  const ordens = [];
  for (const [func, args, pred] of [
    ["mute", [], (i) => i.muted === true], ["unMute", [], (i) => i.muted === false],
    ["setVolume", [30], (i) => i.volume === 30], ["seekTo", [120, true], (i) => i.currentTime >= 119 && i.currentTime < 125],
    ["pauseVideo", [], (i) => i.playerState === 2], ["playVideo", [], (i) => i.playerState === 1],
    ["setPlaybackRate", [1.5], (i) => i.playbackRate === 1.5],
  ]) {
    const t = Date.now();
    await s.page.evaluate(([f, a]) => self.ph2dCmd(f, a), [func, args]);
    ordens.push({ func, args, ms_ate_o_estado: await espera(pred) ?? `não chegou (${Date.now() - t} ms)` });
  }
  const info = ultimoInfo(s.msgs);
  const inv = await inventario(s.frame);
  res.A = { eventos, chaves_infoDelivery: chaves, infoDelivery_por_s: Math.round(n3 / 3 * 10) / 10, ordens,
    exemplo: { currentTime: info.currentTime, duration: info.duration, playerState: info.playerState, volume: info.volume,
      muted: info.muted, videoData: info.videoData, availablePlaybackRates: info.availablePlaybackRates },
    anuncio: { classes: inv.classes_do_tocador, seletor: inv.seletores?.anuncio, adState: s.msgs.filter((x) => typeof x.m.info === "object" && x.m.info && "adState" in x.m.info).map((x) => x.m.info.adState).slice(-1) } };
  await fecha(s);
}

fs.writeFileSync(path.join(out, "w19_tocador.json"), JSON.stringify(res, null, 1));
console.log(JSON.stringify({ P: Object.fromEntries(Object.entries(res.P).map(([k, v]) => [k, v.rato_em_cima?.seletores])), F: res.F, Z: res.Z,
  C: res.C.map((c) => ({ p: c.params, a: c.alvo, achado: !!c.achado, ev: c.eventos, pags: c.paginas_depois.length, q: c.quadros_depois, b: c.jpeg_medio_bytes, e: c.ultimo_estado })), A: res.A }, null, 0));
if (persist) await persist.close(); else await browser.close();
