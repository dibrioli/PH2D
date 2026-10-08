// Gera as árvores de entrada (as MESMAS para os três apps) em entradas/T1..T6.json, mais as árvores
// de custo (N100, N1000). Determinístico: T3/N* usam um PRNG semeado (mulberry32). Formato de nó:
// { id, text, children: [...], collapsed?: true }. Uso: node gera_entradas.mjs
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const OUT = path.join(path.dirname(fileURLToPath(import.meta.url)), "entradas");
fs.mkdirSync(OUT, { recursive: true });
const n = (id, text, children = [], extra = {}) => ({ id, text, children, ...extra });
const leaf = (t) => n(t, t);
const rng = (seed) => () => {
  seed = (seed + 0x6d2b79f5) >>> 0;
  let t = seed;
  t = Math.imul(t ^ (t >>> 15), t | 1);
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};

const T1 = n("root", "Root", ["A", "B", "C"].map(leaf));
const T2 = n("root", "Root", [
  n("A", "A", [leaf("A1"), leaf("A2"), n("A3", "A3", ["A3a", "A3b", "A3c"].map(leaf)), leaf("A4"), leaf("A5")]),
  leaf("B"), leaf("C"), leaf("D"),
]);
// T3: raiz, 6 filhos, cada um com 2–4 filhos, e netos até somar 50 nós.
function geraT3() {
  const r = rng(3);
  const root = n("root", "Root", []);
  const nivel2 = [];
  for (let i = 1; i <= 6; i++) {
    const c = n(`t${i}`, `Topic ${i}`, []);
    root.children.push(c);
    const k = 2 + Math.floor(r() * 3);
    for (let j = 1; j <= k; j++) { const g = n(`t${i}.${j}`, `Topic ${i}.${j}`, []); c.children.push(g); nivel2.push(g); }
  }
  let total = 1 + 6 + nivel2.length;
  while (total < 50) {
    const g = nivel2[Math.floor(r() * nivel2.length)];
    const m = Math.min(1 + Math.floor(r() * 3), 50 - total);
    for (let q = 0; q < m; q++) { const id = `${g.id}.${g.children.length + 1}`; g.children.push(n(id, `Item ${id.slice(1)}`)); total++; }
  }
  return root;
}
const T3 = geraT3();
const T4 = n("root", "Root", [
  leaf("A"),
  n("B", "B is a much longer topic text, about eighty characters, to see wrapping and size"),
  n("C", "C first line\nC second line"),
]);
const T5 = structuredClone(T2);
T5.children[0].collapsed = true;
const T6 = n("root", "Root", ["C1", "C2", "C3", "C4", "C5", "C6", "C7"].map(leaf));
// Custo: árvores de N nós, largura primeiro, até 4 filhos por nó (o nº de filhos vem do PRNG).
function geraN(N, seed) {
  const r = rng(seed);
  const root = n("n0", "Node 0", []);
  const fila = [root];
  let k = 1;
  while (k < N) {
    const p = fila.shift();
    const m = 2 + Math.floor(r() * 3);
    for (let i = 0; i < m && k < N; i++, k++) { const c = n(`n${k}`, `Node ${k}`, []); p.children.push(c); fila.push(c); }
  }
  return root;
}
const conta = (t) => 1 + t.children.reduce((s, c) => s + conta(c), 0);
const todas = { T1, T2, T3, T4, T5, T6, N100: geraN(100, 100), N1000: geraN(1000, 1000) };
for (const [k, t] of Object.entries(todas)) {
  fs.writeFileSync(path.join(OUT, `${k}.json`), JSON.stringify(t) + "\n");
  console.log(k, conta(t), "nós");
}
