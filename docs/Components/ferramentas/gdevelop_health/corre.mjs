// corre.mjs — corre o jogo EXPORTADO no Chrome headless do sistema e grava uma fixture por cenário.
//
//   node corre.mjs <pasta_do_jogo_exportado> <pasta_de_fixtures> [nomeDoCenario...]
//
// Três controlos correm em cada cenário, e a fixture regista o veredito de cada um:
//   · REPRODUTÍVEL — o cenário corre duas vezes em páginas novas e as duas saídas são iguais ao byte;
//   · LER NÃO PERTURBA — uma terceira corrida SEM a leitura «antes» dá o mesmo «depois»/«fim»;
//   · EXPRESSÕES POR EVENTOS — o Health/ShieldPoints/IsDead lidos por eventos do GDevelop
//     (SetNumberVariable) são iguais aos lidos por chamada directa ao método gerado.
// Um controlo que falha aborta (exit 1): uma fixture de um oráculo instável não é um oráculo.
import http from "node:http";
import { readFileSync, writeFileSync, mkdirSync, existsSync, statSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { gzipSync } from "node:zlib";
import { join, dirname, extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { execSync } from "node:child_process";
import { CENARIOS, DT_MS } from "./cenarios.mjs";
import { RANHURAS, COMANDOS } from "./comandos.mjs";

const AQUI = dirname(fileURLToPath(import.meta.url));
const require = createRequire(join(AQUI, "vendor", "package.json"));
const puppeteer = require("puppeteer-core");

const [jogoDir, fixturesDir, ...filtro] = process.argv.slice(2);
if (!jogoDir || !fixturesDir) {
  console.error("uso: node corre.mjs <jogo_exportado> <pasta_fixtures> [cenario...]");
  process.exit(2);
}
const JOGO = resolve(jogoDir);
mkdirSync(fixturesDir, { recursive: true });
const meta = JSON.parse(readFileSync(join(JOGO, "oraculo_meta.json"), "utf8"));
const health = JSON.parse(readFileSync(join(AQUI, "vendor", "Health.json"), "utf8"));
const shaHealth = execSync(`sha256sum ${JSON.stringify(join(AQUI, "vendor", "Health.json"))}`).toString().split(" ")[0];
const pacote = (() => {
  try { return execSync("pacman -Q gdevelop-bin").toString().trim(); } catch { return "desconhecido"; }
})();
const CHROME = process.env.CHROME || "/usr/bin/google-chrome-stable";

const pedidos404 = []; // ficam no cabeçalho: um 404 de favicon não é um 404 de script
// ---- servidor estático local (127.0.0.1, porta efémera) ----
const MIME = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".wasm": "application/wasm", ".css": "text/css", ".png": "image/png" };
const servidor = http.createServer((req, res) => {
  // O browser pede sempre /favicon.ico; o jogo exportado não tem. 204 = «nada», sem 404 a sujar o controlo.
  if (req.url === "/favicon.ico") { res.writeHead(204); res.end(); return; }
  const caminho = join(JOGO, decodeURIComponent(new URL(req.url, "http://x").pathname));
  if (!caminho.startsWith(JOGO) || !existsSync(caminho) || statSync(caminho).isDirectory()) {
    pedidos404.push(req.url);
    res.writeHead(404); res.end(); return;
  }
  res.writeHead(200, { "Content-Type": MIME[extname(caminho)] || "application/octet-stream" });
  res.end(readFileSync(caminho));
});
await new Promise((r) => servidor.listen(0, "127.0.0.1", r));
const URL_JOGO = `http://127.0.0.1:${servidor.address().port}/index.html`;

// Perfil descartável num caminho CURTO (o socket do Chrome tem limite de comprimento); apagado no fim.
const perfil = mkdtempSync(join(tmpdir(), "gdh-"));
const browser = await puppeteer.launch({
  executablePath: CHROME,
  headless: true, // o modo headless NOVO do Chrome: nenhuma janela, nenhum DISPLAY
  userDataDir: perfil,
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--no-first-run", "--no-default-browser-check", "--mute-audio"],
  env: { ...process.env, DISPLAY: "", WAYLAND_DISPLAY: "" },
});
const versaoChrome = await browser.version();

const ids = meta.command_ids;

async function correUmaVez(cenario, opcoes) {
  const page = await browser.newPage();
  const erros = [];
  page.on("pageerror", (e) => erros.push("pageerror: " + e.message));
  page.on("console", (m) => { if (m.type() === "error") erros.push("console.error: " + m.text()); });
  await page.goto(URL_JOGO, { waitUntil: "load" });
  await page.waitForFunction("window.__pronto === true", { timeout: 30000 });
  const out = await page.evaluate(
    (c, ids, r, o) => window.__correr(c, ids, r, o),
    { ...cenario, dt_ms: DT_MS },
    ids,
    RANHURAS,
    opcoes
  );
  const runtimeInfo = await page.evaluate(() => ({
    renderer: (() => { try { return gdjs.RuntimeGame.prototype.getRenderer ? typeof PIXI !== "undefined" ? "PIXI " + PIXI.VERSION : "?" : "?"; } catch (e) { return "?"; } })(),
    behaviorClass: typeof gdjs.evtsExt__Health__Health !== "undefined" ? "gdjs.evtsExt__Health__Health.Health" : null,
  }));
  await page.close();
  // Filtra só avisos benignos conhecidos? Não: qualquer erro de página aborta.
  return { out, erros, runtimeInfo };
}

const semAntes = (passos) => passos.map((p) => ({ ...p, antes: null }));

const escolhidos = filtro.length ? CENARIOS.filter((c) => filtro.includes(c.nome)) : CENARIOS;
let falhou = false;
const resumo = [];
for (const cenario of escolhidos) {
  const r1 = await correUmaVez(cenario, { lerAntes: true });
  const r2 = await correUmaVez(cenario, { lerAntes: true });
  const r3 = await correUmaVez(cenario, { lerAntes: false });
  const erros = [...r1.erros, ...r2.erros, ...r3.erros];
  const reprodutivel = JSON.stringify(r1.out) === JSON.stringify(r2.out);
  const lerNaoPerturba = JSON.stringify(semAntes(r1.out.passos)) === JSON.stringify(r3.out.passos);
  const discrepanciasChk = r1.out.passos.filter(
    (p) =>
      p.chk_por_eventos.Health !== p.depois.publico.Health ||
      p.chk_por_eventos.ShieldPoints !== p.depois.publico.ShieldPoints ||
      p.chk_por_eventos.IsDead !== p.depois.publico.IsDead
  ).map((p) => p.passo);
  const ok = erros.length === 0 && reprodutivel && lerNaoPerturba && discrepanciasChk.length === 0;
  if (!ok) falhou = true;

  const fixture = {
    cabecalho: {
      oraculo: "GDevelop Health extension, executada pelo runtime GDJS do próprio GDevelop (headless)",
      gdevelop: {
        pacote,
        libgd: meta.gdevelop_libgd_version,
        runtime_gdjs: "/usr/lib/gdevelop/GDJS/Runtime (copiado para vendor/gd/Runtime pelo setup.sh)",
        codegen: "gd.BehaviorCodeGenerator via EventsFunctionsExtensionsLoader do IDE (gdcore-tools loaders.cjs) + gd.Exporter.exportWholePixiProject",
      },
      extensao: {
        nome: health.name,
        versao: health.version,
        autor: health.author,
        licenca: "MIT",
        url: "https://github.com/GDevelopApp/GDevelop-extensions/blob/3f71acc8f3d68cea9839654ea8ff606162a4cb39/extensions/reviewed/Health.json",
        commit: "3f71acc8f3d68cea9839654ea8ff606162a4cb39",
        sha256: shaHealth,
      },
      navegador: versaoChrome + " (headless, SwiftShader)",
      runtime_info: r1.runtimeInfo,
      data: new Date().toISOString(),
      comando: "bash docs/Components/ferramentas/gdevelop_health/oraculo.sh",
      tempo: {
        dt_ms: DT_MS,
        como: "passos manuais SceneStack.step(dt_ms) em vez do requestAnimationFrame; TimeManager.update usa min(dt, 1000/minFPS=50) — dt fixo 1000/60 ms",
      },
      fases_de_leitura: {
        antes: "dentro do quadro, na fase de EVENTOS, antes das acções do quadro (já depois do doStepPreEvents do comportamento)",
        depois: "dentro do quadro, na fase de EVENTOS, depois das acções do quadro",
        fim: "entre quadros: depois do doStepPostEvents e do render (o comportamento não tem doStepPostEvents)",
        publico: "expressões/condições PÚBLICAS geradas, chamadas como um evento as chama: getBehavior('Health').X(null)",
        bruto: "os getters _get<Propriedade>() gerados (estado interno do comportamento)",
        timers: "timers do objecto usados pela extensão (getTimerElapsedTimeInSecondsOrNaN)",
      },
      acoes: "cada acção é executada por um EVENTO standard gerado pelo GDevelop (CompareNumbers(CmdN=id) -> Health::Health::<Acção>); ranhuras executam por ordem 1..N no mesmo quadro",
      comandos: Object.fromEntries(COMANDOS.map((c) => [c.nome, `${c.tipo}(Hero, Health, ${c.params("N").join(", ")})`])),
      aleatorio: cenario.semente === null
        ? "Math.random NÃO semeado (o cenário não depende dele: chance de esquiva 0 de fábrica)"
        : `Math.random substituído pelo HARNESS por mulberry32(${cenario.semente}); a extensão chama gdjs.randomFloatInRange(0,1) -> Math.random. A extensão em si não tem semente.`,
      controlos: {
        reprodutivel,
        ler_antes_nao_perturba: lerNaoPerturba,
        expressoes_por_eventos_iguais_as_chamadas: discrepanciasChk.length === 0,
        passos_com_discrepancia: discrepanciasChk,
        erros_da_pagina: erros,
        pedidos_404: [...new Set(pedidos404)],
      },
      cenario: { nome: cenario.nome, descricao: cenario.descricao, semente: cenario.semente, passos: cenario.passos },
    },
    inicial: r1.out.inicial,
    passos: r1.out.passos,
  };
  // Uma linha por passo: legível por diff e por grep.
  const texto =
    "{\n" +
    `"cabecalho": ${JSON.stringify(fixture.cabecalho, null, 1)},\n` +
    `"inicial": ${JSON.stringify(fixture.inicial)},\n` +
    `"passos": [\n${fixture.passos.map((p) => JSON.stringify(p)).join(",\n")}\n]\n}\n`;
  JSON.parse(texto);
  // Comprimida, como os outros corpora de oráculo do repo (o cleanroom-sweep lê .gz pelos bytes
  // mágicos). O cabeçalho gzip do zlib do Node leva mtime 0 ⇒ a mesma saída dá os mesmos bytes.
  writeFileSync(join(fixturesDir, `${cenario.nome}.json.gz`), gzipSync(texto, { level: 9 }));
  resumo.push({ cenario: cenario.nome, passos: fixture.passos.length, ok, reprodutivel, lerNaoPerturba, chk: discrepanciasChk.length, erros: erros.length });
}

await browser.close();
servidor.close();
rmSync(perfil, { recursive: true, force: true });
console.table(resumo);
if (falhou) {
  console.error("FALHOU um controlo — ver cabecalho.controlos nas fixtures");
  process.exit(1);
}
