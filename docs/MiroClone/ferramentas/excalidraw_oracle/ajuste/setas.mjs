// As setas do Excalidraw: curva pelos pontos (arredondada) ou segmentos (sem arredondar), e o tremor.
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
const require = createRequire(path.join(process.env.HOME, "Apps/excalidraw-oracle/package.json"));
const gen = require("roughjs/bundled/rough.cjs.js").generator();
const els = JSON.parse(fs.readFileSync("saidas/rascunho_setas.restored.json", "utf8"));
const svg = fs.readFileSync("saidas/rascunho_setas.svg", "utf8");
const groups = [...svg.matchAll(/<g stroke-linecap="round"[^>]*>([\s\S]*?)<\/g>/g)].map((m) => [...m[1].matchAll(/<path d="([^"]*)"/g)].map((x) => x[1]));
console.log("grupos", groups.length, groups.map((g) => g.length));
els.forEach((e, i) => {
  const pts = e.points;
  for (const r of [1, 0.5])
    for (const pv of [true, false]) {
      const o = { seed: e.seed, strokeWidth: 2, roughness: r, preserveVertices: pv };
      const d = e.roundness ? gen.curve(pts, o) : gen.linearPath(pts, o);
      const got = gen.opsToPath(d.sets[0], 2);
      const hit = groups.findIndex((g) => g.includes(got));
      if (hit >= 0) console.log(e.type, pts.length, "pts", "roundness", JSON.stringify(e.roundness), "→ r", r, "preserve", pv, "grupo", hit, `(${e.width}x${e.height})`);
    }
});
