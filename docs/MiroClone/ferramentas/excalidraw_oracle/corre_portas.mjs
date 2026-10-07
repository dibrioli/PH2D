// Corre as DUAS PORTAS do quadro (W4) como oráculo, sem navegador: o rough.js (traço à mão) e o
// perfect-freehand (contorno do desenho livre), sobre entradas NOSSAS, e grava a saída POR PASSO.
// Uso: node corre_portas.mjs <entradas/portas.json> <pasta_saida>
//   → <pasta_saida>/portas_rough.json     (por caso: os `sets` do gerador — tipo + ops com todos os dígitos)
//   → <pasta_saida>/portas_freehand.json  (por caso: passo 1 `getStrokePoints`, passo 2 `getStrokeOutlinePoints`)
// Cada ficheiro abre com `meta` (versões, licenças, data, sha256 da entrada, node). Os números saem
// pelo `JSON.stringify` do V8 (o mais curto que relê o MESMO double): a comparação do lado Rust é ao
// ulp-ish (tolerância no gate), nunca a olho.
//
// Determinismo: o rough.js só cai no `Math.random` com `seed: 0` (proibido nas entradas) ou num
// `randomizer.next() === 0` do preenchimento; mesmo assim o `Math.random` é semeado (mulberry32).
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const ORACLE_HOME = process.env.ORACLE_HOME || path.join(process.env.HOME, "Apps/excalidraw-oracle");
const require = createRequire(path.join(ORACLE_HOME, "package.json"));
const rough = require("roughjs/bundled/rough.cjs.js");
const pf = require("perfect-freehand");
const pkg = (name) => JSON.parse(fs.readFileSync(path.join(ORACLE_HOME, "node_modules", name, "package.json"), "utf8"));

const [inputPath, outDir] = process.argv.slice(2);
if (!inputPath || !outDir) {
  console.error("uso: node corre_portas.mjs <entradas/portas.json> <pasta_saida>");
  process.exit(2);
}
const inputBytes = fs.readFileSync(inputPath);
const input = JSON.parse(inputBytes);
const inputSha = createHash("sha256").update(inputBytes).digest("hex");

let s = parseInt(inputSha.slice(0, 8), 16) >>> 0;
Math.random = () => {
  s = (s + 0x6d2b79f5) >>> 0;
  let t = s;
  t = Math.imul(t ^ (t >>> 15), t | 1);
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};

const meta = (lib) => ({
  oracle: lib,
  package_version: pkg(lib).version,
  package_license: pkg(lib).license,
  deps: lib === "roughjs"
    ? Object.fromEntries(["hachure-fill", "path-data-parser", "points-on-path"].map((d) => [d, `${pkg(d).version} ${pkg(d).license}`]))
    : {},
  node: process.version,
  date: new Date().toISOString(),
  input: path.basename(inputPath),
  input_sha256: inputSha,
  runner: "docs/MiroClone/ferramentas/excalidraw_oracle/corre_portas.mjs",
});

const gen = rough.generator();
const roughCases = input.rough.map((c) => {
  if (!c.options.seed) throw new Error(`${c.name}: seed 0 cai no Math.random — proibido`);
  const d = gen[c.kind](...c.args, c.options);
  return { name: c.name, kind: c.kind, sets: d.sets.map((set) => ({ type: set.type, ops: set.ops.map((o) => [o.op, ...o.data]) })) };
});

const freehandCases = input.freehand.map((c) => {
  const points = c.points;
  const strokePoints = pf.getStrokePoints(points, c.options);
  const outline = pf.getStrokeOutlinePoints(strokePoints, c.options);
  return {
    name: c.name,
    stroke_points: strokePoints.map((p) => [p.point[0], p.point[1], p.pressure, p.vector[0], p.vector[1], p.distance, p.runningLength]),
    outline,
  };
});

fs.mkdirSync(outDir, { recursive: true });
const write = (name, obj) => fs.writeFileSync(path.join(outDir, name), JSON.stringify(obj) + "\n");
write("portas_rough.json", { meta: meta("roughjs"), cases: roughCases });
write("portas_freehand.json", { meta: { ...meta("perfect-freehand"), step_fields: "stroke_points = [x, y, pressure, vector.x, vector.y, distance, runningLength]" }, cases: freehandCases });
console.log(`rough ${roughCases.length} casos · freehand ${freehandCases.length} casos → ${outDir}`);
