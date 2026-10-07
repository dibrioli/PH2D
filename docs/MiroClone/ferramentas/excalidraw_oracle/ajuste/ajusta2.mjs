import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
const require = createRequire(path.join(process.env.HOME, "Apps/excalidraw-oracle/package.json"));
const rough = require("roughjs/bundled/rough.cjs.js");
const gen = rough.generator();
const [name] = process.argv.slice(2);
const els = JSON.parse(fs.readFileSync(`saidas/${name}.restored.json`, "utf8"));
const svg = fs.readFileSync(`saidas/${name}.svg`, "utf8");
const groups = [...svg.matchAll(/<g stroke-linecap="round"[^>]*>([\s\S]*?)<\/g>/g)].map((m) => [...m[1].matchAll(/<path d="([^"]*)"/g)].map((x) => x[1]));
const shapes = els.filter((e) => ["rectangle", "ellipse", "diamond"].includes(e.type));
const ds = (d) => d.sets.map((s) => gen.opsToPath(s, 2));
function draw(e, o) {
  const w = e.width, h = e.height;
  if (e.type === "rectangle") return gen.rectangle(0, 0, w, h, o);
  if (e.type === "ellipse") return gen.ellipse(w / 2, h / 2, w, h, { ...o, curveFitting: 1 });
  const tx = Math.floor(w / 2) + 1, ry = Math.floor(h / 2) + 1;
  return gen.polygon([[tx, 0], [w, ry], [tx, h], [0, ry]], o);
}
const cands = [];
for (let k = 1; k <= 400; k++) cands.push(k / 400 * e0());
function e0() { return 1; }
shapes.forEach((e, i) => {
  const want = groups[i] || [];
  const tryR = (r) => {
    const o = { seed: e.seed, strokeWidth: e.strokeWidth, roughness: r, fill: e.backgroundColor, fillStyle: e.fillStyle,
      preserveVertices: e.roughness < 2, fillWeight: e.strokeWidth / 2, hachureGap: e.strokeWidth * 4, disableMultiStroke: e.strokeStyle !== "solid" };
    const got = ds(draw(e, o));
    return got.length === want.length && got.every((d, k) => d === want[k]);
  };
  let hit = tryR(e.roughness) ? e.roughness : tryR(0) ? 0 : null;
  if (hit === null) for (let k = 1; k <= 2000 && hit === null; k++) { const r = k / 1000 * e.roughness; if (tryR(r)) hit = r; }
  console.log(e.type, `${e.width}x${e.height}`, "sw", e.strokeWidth, "r", e.roughness, "roundness", JSON.stringify(e.roundness), "→ tremor efectivo", hit);
});
