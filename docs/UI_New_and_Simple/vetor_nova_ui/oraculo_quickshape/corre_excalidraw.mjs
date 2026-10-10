// ⭐ O 2.º ORÁCULO DO QUICKSHAPE — o reconhecedor de traços do Excalidraw (canal `next`, a ferramenta
// «draw shape»: um traço livre vira rectângulo, elipse, losango, SETA ou linha). MIT, mas corre-se e
// não se lê (o método: o fonte diz COMO, o gate precisa de O QUÊ). Só a API pública exportada por
// `@excalidraw/element`: `recognizeShape` (o tipo) e `convertToShape` (o elemento que entra na cena).
//
// uso:  node corre_excalidraw.mjs [entradas_dir] [fixtures_dir] [app_dir]
//       app_dir = onde está o `node_modules` (por omissão ~/Apps/excalidraw-oracle-next)
//
// ⚠️ MEDIDO (10/10): `recognizeShape` devolve os pontos CRUS (só o tipo é a resposta); a geometria é
// a do `convertToShape`, que pede um `appState` — o mínimo abaixo basta (o elemento sai com o estilo
// «corrente»). A FORMA da ponta vem do estado (`currentItemEndArrowhead`), não do desenho: por isso
// o cabeçalho grava as duas corridas, com a ponta corrente `"arrow"` e com `null`.
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { pathToFileURL } from "node:url";

const AQUI = path.dirname(new URL(import.meta.url).pathname);
const ENT = process.argv[2] || path.join(AQUI, "entradas");
const FIX = process.argv[3] || path.join(AQUI, "fixtures_excalidraw");
const APP = process.argv[4] || path.join(os.homedir(), "Apps/excalidraw-oracle-next");
const pkg = (p) => JSON.parse(fs.readFileSync(path.join(APP, "node_modules", p, "package.json"), "utf8"));
const EL = await import(pathToFileURL(path.join(APP, "node_modules/@excalidraw/element/dist/prod/index.js")).href);
const VERSAO = `@excalidraw/element ${pkg("@excalidraw/element").version} (${pkg("@excalidraw/element").license})`;

const appState = (endHead) => ({
  zoom: { value: 1 }, scrollX: 0, scrollY: 0, currentItemStrokeColor: "#1e1e1e",
  currentItemBackgroundColor: "transparent", currentItemFillStyle: "solid", currentItemStrokeWidth: 2,
  currentItemStrokeStyle: "solid", currentItemRoughness: 1, currentItemOpacity: 100,
  currentItemRoundness: "round", currentItemStartArrowhead: null, currentItemEndArrowhead: endHead,
  currentItemArrowType: "round", currentItemFontFamily: 5, currentItemFontSize: 20,
  currentItemTextAlign: "left", frameRendering: { enabled: true, clip: true, name: true, outline: true },
  gridSize: 20, gridStep: 5, gridModeEnabled: false, editingGroupId: null, selectedElementIds: {},
  objectsSnapModeEnabled: false,
});

const lePontos = (f) => {
  const cab = [];
  const pts = [];
  for (const l of fs.readFileSync(f, "utf8").split("\n")) {
    if (l.startsWith("#")) cab.push(l);
    else if (l.trim()) pts.push(l.trim().split(/\s+/).map(Number));
  }
  return { cab, pts };
};

const elemento = (pts, endHead) => {
  const el = EL.convertToShape(pts.map((p) => [...p]), appState(endHead), new Map(), null, []);
  if (!el) return { type: "nenhum" };
  const out = { type: el.type, x: el.x, y: el.y, w: el.width, h: el.height, angle: el.angle };
  if (el.points) out.points = el.points.map(([x, y]) => [+(el.x + x).toFixed(3), +(el.y + y).toFixed(3)]);
  if ("startArrowhead" in el) Object.assign(out, { startArrowhead: el.startArrowhead, endArrowhead: el.endArrowhead });
  if (el.roundness !== undefined) out.roundness = el.roundness;
  return out;
};

fs.mkdirSync(FIX, { recursive: true });
let n = 0;
const tipos = {};
for (const nome of fs.readdirSync(ENT).filter((f) => f.endsWith(".txt")).sort()) {
  const { cab, pts } = lePontos(path.join(ENT, nome));
  const tipo = EL.recognizeShape(pts.map((p) => [...p]), null, 1).type;
  const comPonta = elemento(pts, "arrow");
  const semPonta = elemento(pts, null);
  const linhas = [
    ...cab,
    `# oraculo: ${VERSAO} · recognizeShape(points, null, zoom 1) + convertToShape(appState minimo)`,
    `# unidade: a da entrada (px da tela virtual) — o Excalidraw trabalha em coordenadas de cena = ecra a zoom 1`,
    `# tipo: ${tipo}`,
    `# elemento_ponta_corrente_arrow: ${JSON.stringify(comPonta)}`,
    `# elemento_ponta_corrente_null: ${JSON.stringify(semPonta)}`,
  ];
  linhas.push("# --- entrada (px) ---");
  for (const [x, y] of pts) linhas.push(`in ${x} ${y}`);
  linhas.push("# --- elemento do oraculo (px) ---");
  for (const [x, y] of comPonta.points || []) linhas.push(`out ${x} ${y}`);
  fs.writeFileSync(path.join(FIX, nome), linhas.join("\n") + "\n");
  tipos[tipo] = (tipos[tipo] || 0) + 1;
  n++;
  const cabos = comPonta.points ? `${comPonta.points.length} pts` : "";
  console.log(`${nome.replace(".txt", "").padEnd(28)} ${tipo.padEnd(10)} ${cabos.padEnd(7)} ${comPonta.startArrowhead ?? ""}→${comPonta.endArrowhead ?? ""}`);
}
console.log(`${n} traços · ${JSON.stringify(tipos)} · fixtures em ${FIX}`);
