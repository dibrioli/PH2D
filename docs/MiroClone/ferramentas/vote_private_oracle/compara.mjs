// Compara duas corridas caso a caso, passo a passo, sobre a forma canónica (normaliza.mjs).
//   node compara.mjs saidas/scrumlr saidas/scrumlr/corrida2
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { canonico } from "./normaliza.mjs";
const [a, b] = process.argv.slice(2);
const res = {};
for (const f of readdirSync(a).filter((x) => /^[vp]\d+\w*\.json$/.test(x)).sort()) {
  const A = canonico(JSON.parse(readFileSync(join(a, f), "utf8")));
  const B = canonico(JSON.parse(readFileSync(join(b, f), "utf8")));
  const difs = [];
  for (let i = 0; i < Math.max(A.length, B.length); i++) {
    const x = A[i], y = B[i];
    if (!x || !y) { difs.push({ passo: i + 1, so_numa: true }); continue; }
    const hx = JSON.stringify(x.http), hy = JSON.stringify(y.http);
    if (hx !== hy) difs.push({ passo: x.passo, texto: x.texto, onde: "http", c1: x.http, c2: y.http });
    for (const q of new Set([...Object.keys(x.ws), ...Object.keys(y.ws)])) {
      const wx = JSON.stringify(x.ws[q] ?? []), wy = JSON.stringify(y.ws[q] ?? []);
      if (wx !== wy) difs.push({ passo: x.passo, texto: x.texto, onde: "ws:" + q, c1: x.ws[q] ?? [], c2: y.ws[q] ?? [] });
    }
  }
  res[f.replace(".json", "")] = { passos: A.length, iguais: difs.length === 0, difs };
  console.log(`${f.padEnd(10)} passos=${A.length} ${difs.length === 0 ? "IGUAIS" : "DIFEREM em " + difs.map((d) => d.passo + ":" + d.onde).join(", ")}`);
}
if (process.argv[4]) writeFileSync(process.argv[4], JSON.stringify(res, null, 1) + "\n");
