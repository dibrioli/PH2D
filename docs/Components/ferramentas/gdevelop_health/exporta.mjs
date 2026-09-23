// exporta.mjs — constrói um projecto GDevelop NOSSO com a extensão Health e exporta-o
// com o exportador do PRÓPRIO GDevelop (libGD.wasm 5.6.282 + carregador de extensões do IDE).
//
//   node exporta.mjs <pasta_de_saida>
//
// Nada aqui reimplementa a extensão: o código do comportamento é gerado pelo
// `gd.BehaviorCodeGenerator` (via o EventsFunctionsExtensionsLoader do IDE) e os
// eventos da cena pelo gerador de eventos do exportador.
import { createRequire } from "node:module";
import { readFileSync, writeFileSync, mkdirSync, rmSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { COMANDOS, RANHURAS, ID_DE } from "./comandos.mjs";

const AQUI = dirname(fileURLToPath(import.meta.url));
const VENDOR = join(AQUI, "vendor");
const GDJS_ROOT = join(VENDOR, "gd"); // contém Runtime/ (o GDJS instalado)
const require = createRequire(import.meta.url);

const saida = process.argv[2];
if (!saida) {
  console.error("uso: node exporta.mjs <pasta_de_saida>");
  process.exit(2);
}

// ---- 1. libGD (mesma sequência de arranque que o gdcore-tools/o IDE usam) ----
let logsGd = "";
const fetchGuardado = globalThis.fetch;
delete globalThis.fetch; // senão o Emscripten julga que está num browser
const initializeGDevelopJs = require(join(GDJS_ROOT, "lib", "libGD.cjs"));
const gd = await new Promise((resolve) => {
  initializeGDevelopJs({
    print: (m) => (logsGd += m + "\n"),
    printErr: (m) => (logsGd += m + "\n"),
    locateFile: (f) => join(GDJS_ROOT, "lib", f),
  }).then((g) => {
    delete g.then;
    resolve(g);
  });
});
globalThis.fetch = fetchGuardado;
gd.ProjectHelper.initializePlatforms();

global.gd = gd;
const loaders = require(join(VENDOR, "gdcore-tools", "dist", "loaders.cjs"));
delete global.gd;

const extensionsLoader = loaders.makeExtensionsLoader({
  gd,
  onFindGDJS: async () => ({ gdjsRoot: GDJS_ROOT }),
});
await extensionsLoader.loadAllExtensions((s) => s);

// ---- 2. O projecto (NOSSO) ----
const project = gd.ProjectHelper.createNewGDJSProject();
{
  const esqueleto = new gd.SerializerElement();
  project.serializeTo(esqueleto);
  const obj = JSON.parse(gd.Serializer.toJSON(esqueleto));
  esqueleto.delete();
  obj.properties.name = "PH2D Health Oracle";
  obj.properties.windowWidth = 320;
  obj.properties.windowHeight = 240;
  obj.properties.minFPS = 20; // o default; o driver passa sempre 1000/60 < 50 ms
  obj.properties.maxFPS = 60;
  obj.properties.projectFile = join(saida, "projecto.json");
  obj.eventsFunctionsExtensions = [JSON.parse(readFileSync(join(VENDOR, "Health.json"), "utf8"))];
  const s = gd.Serializer.fromJSObject(obj);
  project.unserializeFrom(s);
  s.delete();
}

// Gera o código dos comportamentos por eventos e declara-os na plataforma (código do IDE).
const escritos = [];
await loaders.loadProjectEventsFunctionsExtensions(
  project,
  loaders.makeLocalEventsFunctionCodeWriter({ onWriteFile: (f) => escritos.push(f) }),
  (s) => s
);

const layout = project.insertNewLayout("Cena", 0);
project.setFirstLayout("Cena");

// Variáveis de cena usadas pelos eventos (declaradas: o 5.6 recusa variáveis por declarar).
const vars = layout.getVariables();
const nomesVars = [];
for (let s = 1; s <= RANHURAS; s++) nomesVars.push(`Cmd${s}`, `A${s}`);
nomesVars.push("ChkHealth", "ChkShield", "ChkIsDead");
nomesVars.forEach((n, i) => {
  const v = new gd.Variable();
  v.setValue(0);
  vars.insert(n, v, i);
  v.delete();
});

// O objecto: um Sprite sem imagem, com o comportamento Health (valores de fábrica).
const objetos = layout.getObjects();
const hero = objetos.insertNewObject(project, "Sprite", "Hero", 0);
hero.addNewBehavior(project, "Health::Health", "Health");
// O IDE faz isto ao acrescentar um comportamento: cria os «shared data» da cena para ele
// (sem isto o runtime acusa «Can't find shared data for behavior with name: Health»).
layout.updateBehaviorsSharedData(project);
const inst = layout.getInitialInstances().insertNewInitialInstance();
inst.setObjectName("Hero");
inst.setX(100);
inst.setY(100);

// ---- 3. Os eventos da cena ----
const jsEvento = (codigo) => ({
  type: "BuiltinCommonInstructions::JsCode",
  inlineCode: codigo,
  parameterObjects: "Hero",
  useStrict: true,
  eventsSheetExpanded: false,
});
const instr = (tipo, parametros) => ({ type: { value: tipo }, parameters: parametros });
const eventos = [];
// (a) leitura ANTES das acções deste quadro (já depois do doStepPreEvents do comportamento)
eventos.push(jsEvento("window.__oraculo && window.__oraculo.antes(runtimeScene, objects);"));
// (b) despacho: RANHURAS × COMANDOS eventos standard, cada um com a acção pública da extensão
for (let s = 1; s <= RANHURAS; s++) {
  for (const c of COMANDOS) {
    eventos.push({
      type: "BuiltinCommonInstructions::Standard",
      conditions: [instr("BuiltinCommonInstructions::CompareNumbers", [`Cmd${s}`, "=", String(ID_DE[c.nome])])],
      actions: [instr(c.tipo, ["Hero", "Health", ...c.params(s)])],
      events: [],
    });
  }
}
// (c) verificação cruzada: as EXPRESSÕES lidas por eventos (e não por chamada directa)
eventos.push({
  type: "BuiltinCommonInstructions::Standard",
  conditions: [],
  actions: [
    instr("SetNumberVariable", ["ChkHealth", "=", "Hero.Health::Health()"]),
    instr("SetNumberVariable", ["ChkShield", "=", "Hero.Health::ShieldPoints()"]),
  ],
  events: [],
});
eventos.push({
  type: "BuiltinCommonInstructions::Standard",
  conditions: [instr("Health::Health::IsDead", ["Hero", "Health"])],
  actions: [instr("SetNumberVariable", ["ChkIsDead", "=", "1"])],
  events: [],
});
// (d) leitura DEPOIS das acções deste quadro
eventos.push(jsEvento("window.__oraculo && window.__oraculo.depois(runtimeScene, objects);"));
{
  const s = gd.Serializer.fromJSObject(eventos);
  layout.getEvents().unserializeFrom(project, s);
  s.delete();
}

// ---- 4. Exportar com o exportador do GDevelop ----
rmSync(saida, { recursive: true, force: true });
mkdirSync(saida, { recursive: true });
const fs = loaders.assignIn(new gd.AbstractFileSystemJS(), new loaders.LocalFileSystem());
const exporter = new gd.Exporter(fs, GDJS_ROOT);
const opts = new gd.ExportOptions(project, saida);
const ok = exporter.exportWholePixiProject(opts);
const erroExport = exporter.getLastError();
opts.delete();
exporter.delete();

// Injecta o driver do oráculo imediatamente antes do <script> de arranque do index.html.
if (ok) {
  const indexPath = join(saida, "index.html");
  const html = readFileSync(indexPath, "utf8");
  const ancora = "<body>";
  const n = html.split(ancora).length - 1;
  if (n !== 1) throw new Error(`âncora <body> casou ${n} vezes no index.html exportado`);
  writeFileSync(join(saida, "oraculo_driver.js"), readFileSync(join(AQUI, "oraculo_driver.js")));
  writeFileSync(indexPath, html.replace(ancora, `${ancora}\n<script src="oraculo_driver.js"></script>`));
}

// Guarda o projecto (para auditoria) e os metadados do export.
{
  const el = new gd.SerializerElement();
  project.serializeTo(el);
  writeFileSync(join(saida, "projecto.json"), gd.Serializer.toJSON(el));
  el.delete();
}
const meta = {
  gdevelop_libgd_version: gd.VersionWrapper.fullString(),
  export_ok: ok,
  export_error: erroExport,
  generated_extension_files: escritos.map((f) => f.includeFile),
  command_ids: ID_DE,
  slots: RANHURAS,
};
writeFileSync(join(saida, "oraculo_meta.json"), JSON.stringify(meta, null, 2));
if (logsGd.trim()) writeFileSync(join(saida, "libgd.log"), logsGd);
console.log(JSON.stringify(meta, null, 2));
if (!ok) process.exit(1);
