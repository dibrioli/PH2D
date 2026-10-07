import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
const require = createRequire(path.join(process.env.HOME, "Apps/excalidraw-oracle/package.json"));
const gen = require("roughjs/bundled/rough.cjs.js").generator();
const els = JSON.parse(fs.readFileSync("saidas/rascunho_cantos.restored.json", "utf8"));
const svg = fs.readFileSync("saidas/rascunho_cantos.svg", "utf8");
const groups = [...svg.matchAll(/<g stroke-linecap="round"[^>]*>([\s\S]*?)<\/g>/g)].map((m) => [...m[1].matchAll(/<path d="([^"]*)"/g)].map((x) => x[1]));
els.forEach((e, i) => {
  if (e.type !== "rectangle") return;
  const { width: w, height: h } = e;
  const r = Math.min(Math.min(w, h) / 4, 32);
  const d = `M ${r} 0 L ${w - r} 0 Q ${w} 0, ${w} ${r} L ${w} ${h - r} Q ${w} ${h}, ${w - r} ${h} L ${r} ${h} Q 0 ${h}, 0 ${h - r} L 0 ${r} Q 0 0, ${r} 0`;
  const sm = Math.max(w, h) < 0 ? 0 : 1;
  for (const rr of [1, 0.5]) {
    const o = { seed: e.seed, strokeWidth: 2, roughness: rr, fill: e.backgroundColor, fillStyle: "solid", preserveVertices: true, fillWeight: 1, hachureGap: 8 };
    const got = gen.toPaths(gen.path(d, o)).map((p) => p.d);
    const got2 = gen.path(d, o).sets.map((s) => gen.opsToPath(s, 2));
    const want = groups[i];
    console.log(`${w}x${h} r=${rr}`, got2.length === want.length && got2.every((x, k) => x === want[k]) ? "BATE" : "não", "| quer", want[0].slice(0, 60), "| deu", got2[0].slice(0, 60));
  }
});
