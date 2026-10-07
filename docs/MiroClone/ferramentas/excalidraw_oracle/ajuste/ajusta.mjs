// Ajusta as opcoes do rough.js que o Excalidraw usa: para cada elemento gravado (semente fixa), gera
// o traco com candidatos de opcoes e compara o `d` (2 casas) com o do SVG do oraculo.
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
const require = createRequire(path.join(process.env.HOME, "Apps/excalidraw-oracle/package.json"));
const rough = require("roughjs/bundled/rough.cjs.js");
const gen = rough.generator();
const [name] = process.argv.slice(2);
const els = JSON.parse(fs.readFileSync(`saidas/${name}.restored.json`, "utf8"));
const svg = fs.readFileSync(`saidas/${name}.svg`, "utf8");
// grupos por elemento: <g ...> ... </g> com paths
const groups = [...svg.matchAll(/<g stroke-linecap="round"[^>]*>([\s\S]*?)<\/g>/g)].map((m) => [...m[1].matchAll(/<path d="([^"]*)"/g)].map((x) => x[1]));
const shapes = els.filter((e) => ["rectangle", "ellipse", "diamond"].includes(e.type));
const ds = (drawable) => drawable.sets.map((s) => gen.opsToPath(s, 2));
function draw(e, o) {
  const w = e.width, h = e.height;
  if (e.type === "rectangle") return gen.rectangle(0, 0, w, h, o);
  if (e.type === "ellipse") return gen.ellipse(w / 2, h / 2, w, h, o);
  const tx = Math.floor(w / 2) + 1, ry = Math.floor(h / 2) + 1;
  const pts = [[tx, 0], [w, ry], [tx, h], [0, ry]];
  return gen.polygon(pts, o);
}
const grid = [];
for (const preserveVertices of [false, true])
  for (const curveFitting of [0.95, 1])
    for (const rk of ["raw", "half"])
      for (const fw of ["half", "def"])
        for (const gap of ["x4", "def"])
          grid.push({ preserveVertices, curveFitting, rk, fw, gap });
shapes.forEach((e, i) => {
  const want = groups[i] || [];
  const hits = grid.filter((g) => {
    const o = {
      seed: e.seed, strokeWidth: e.strokeWidth, roughness: g.rk === "raw" ? e.roughness : e.roughness / 2,
      fill: e.backgroundColor, fillStyle: e.fillStyle, preserveVertices: g.preserveVertices, curveFitting: g.curveFitting,
      ...(g.fw === "half" ? { fillWeight: e.strokeWidth / 2 } : {}), ...(g.gap === "x4" ? { hachureGap: e.strokeWidth * 4 } : {}),
      disableMultiStroke: e.strokeStyle !== "solid",
    };
    const got = ds(draw(e, o));
    return got.length === want.length && got.every((d, k) => d === want[k]);
  });
  console.log(e.type, "r" + e.roughness, e.fillStyle, "paths", want.length, "→", hits.length ? JSON.stringify(hits[0]) + ` (+${hits.length - 1})` : "NENHUM");
});
