// Destila as LEIS de layout a partir das saídas layout_* (cada nó: [id, texto, pai, prof, x, y, w, h,
// lado, recolhido], centro da raiz = (0,0)). Para cada caso e árvore:
//  - nivel: vão entre o pai e o filho no EIXO PRINCIPAL da aresta (borda→borda e centro→centro), por
//    profundidade do pai; e se os filhos do mesmo grupo alinham a BORDA próxima ou o CENTRO;
//  - irmaos: vão entre irmãos consecutivos no eixo cruzado, de caixa a caixa (no) e de FAIXA de
//    subárvore a faixa (subarvore; negativo ⇒ uma subárvore entra na faixa da outra = compactação);
//  - centragem: centro do pai − (média do 1.º e do último filho | meio do bloco das caixas dos filhos |
//    meio da faixa das subárvores dos filhos), por grupo com ≥ 2 filhos;
//  - equilibrio: o lado de cada filho da raiz, pela ordem dos dados;
//  - recolher (T5 × T2): deslocamento dos nós que sobram quando A recolhe;
//  - texto (T4): caixa de A (curto), B (~80 caracteres) e C (duas linhas).
// O eixo principal é o do LAYOUT (EIXO abaixo); nos layouts «misto» decide-se aresta a aresta e os
// números valem só como descrição.
// Uso: import { destilaLeis } from "./leis.mjs"; ou `node leis.mjs <pasta_saidas>` (reescreve leis.json).
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const r3 = (v) => Math.round(v * 1000) / 1000 || 0;
const stats = (xs) => {
  if (!xs.length) return null;
  const s = [...xs].sort((a, b) => a - b);
  const med = s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2;
  return { n: s.length, min: r3(s[0]), med: r3(med), max: r3(s[s.length - 1]) };
};
const agrupa = (pares) => {
  const m = {};
  for (const [k, v] of pares) (m[k] ??= []).push(v);
  return Object.fromEntries(Object.entries(m).sort().map(([k, v]) => [k, stats(v)]));
};

function analisaArvore(nos, eixo) {
  const N = new Map(nos.map((n) => [n[0], { id: n[0], pai: n[2], prof: n[3], vis: n[4] !== null, x: n[4], y: n[5], w: n[6], h: n[7], lado: n[8], filhos: [] }]));
  for (const n of N.values()) if (n.pai !== null && N.has(n.pai)) N.get(n.pai).filhos.push(n);
  const cx = (n) => n.x + n.w / 2, cy = (n) => n.y + n.h / 2;
  const bbox = (n) => {
    let b = [n.x, n.y, n.x + n.w, n.y + n.h];
    for (const f of n.filhos) if (f.vis) { const c = bbox(f); b = [Math.min(b[0], c[0]), Math.min(b[1], c[1]), Math.max(b[2], c[2]), Math.max(b[3], c[3])]; }
    return b;
  };
  const nivelBorda = [], nivelCentro = [], alinhaBorda = [], alinhaCentro = [];
  const irmNo = [], irmSub = [], cent1 = [], cent2 = [], cent3 = [];
  const compacta = [];
  for (const p of N.values()) {
    if (!p.vis) continue;
    const grupos = {};
    for (const f of p.filhos) {
      if (!f.vis) continue;
      const dx = cx(f) - cx(p), dy = cy(f) - cy(p);
      // eixo principal do LAYOUT (x: os filhos ficam à esquerda/direita; y: em cima/em baixo); só nos
      // layouts mistos (timeline, fishbone, catálogo) se decide aresta a aresta.
      const horiz = eixo === "x" ? true : eixo === "y" ? false : Math.abs(dx) >= Math.abs(dy);
      const g = horiz ? (dx >= 0 ? "dir" : "esq") : dy >= 0 ? "baixo" : "cima";
      (grupos[g] ??= []).push(f);
      const borda = g === "dir" ? f.x - (p.x + p.w) : g === "esq" ? p.x - (f.x + f.w) : g === "baixo" ? f.y - (p.y + p.h) : p.y - (f.y + f.h);
      nivelBorda.push([`${p.prof}→${p.prof + 1}`, borda]);
      nivelCentro.push([`${p.prof}→${p.prof + 1}`, horiz ? Math.abs(dx) : Math.abs(dy)]);
    }
    for (const [g, fs_] of Object.entries(grupos)) {
      const horiz = g === "dir" || g === "esq";
      // eixo cruzado: y para arestas horizontais, x para verticais
      const a0 = (n) => (horiz ? n.y : n.x), a1 = (n) => (horiz ? n.y + n.h : n.x + n.w);
      const b0 = (b) => (horiz ? b[1] : b[0]), b1 = (b) => (horiz ? b[3] : b[2]);
      const ord = [...fs_].sort((u, v) => a0(u) - a0(v));
      const near = (f) => (g === "dir" ? f.x : g === "esq" ? f.x + f.w : g === "baixo" ? f.y : f.y + f.h);
      const ctr = (f) => (horiz ? cx(f) : cy(f));
      if (ord.length >= 2) {
        alinhaBorda.push([`${p.prof + 1}`, Math.max(...ord.map(near)) - Math.min(...ord.map(near))]);
        alinhaCentro.push([`${p.prof + 1}`, Math.max(...ord.map(ctr)) - Math.min(...ord.map(ctr))]);
      }
      for (let i = 1; i < ord.length; i++) {
        const u = ord[i - 1], v = ord[i];
        irmNo.push([`${v.prof}`, a0(v) - a1(u)]);
        const gs = b0(bbox(v)) - b1(bbox(u));
        irmSub.push([`${v.prof}`, gs]);
        if (gs < -0.5) compacta.push(`${u.id}|${v.id}:${r3(gs)}`);
      }
      if (ord.length >= 2) {
        const pc = horiz ? cy(p) : cx(p);
        const c = (n) => (horiz ? cy(n) : cx(n));
        const first = ord[0], last = ord[ord.length - 1];
        cent1.push([`${p.prof}`, pc - (c(first) + c(last)) / 2]);
        cent2.push([`${p.prof}`, pc - (a0(first) + a1(last)) / 2]);
        const bs = ord.map(bbox);
        cent3.push([`${p.prof}`, pc - (Math.min(...bs.map(b0)) + Math.max(...bs.map(b1))) / 2]);
      }
    }
  }
  return {
    nivel: { borda: agrupa(nivelBorda), centro: agrupa(nivelCentro), espalhamentoBordaProxima: agrupa(alinhaBorda), espalhamentoCentro: agrupa(alinhaCentro) },
    irmaos: { no: agrupa(irmNo), subarvore: agrupa(irmSub), compactados: compacta },
    centragem: { mediaPrimeiroUltimo: agrupa(cent1), meioDoBlocoDeCaixas: agrupa(cent2), meioDaFaixaDasSubarvores: agrupa(cent3) },
  };
}

function equilibrio(nos) {
  const filhos = nos.filter((n) => n[3] === 1);
  const lados = filhos.map((n) => n[8] ?? "?").join("");
  const desc = (id) => nos.filter((n) => n[2] === id).reduce((s, n) => s + 1 + desc(n[0]), 0);
  return { ordemDosLados: lados, porLado: Object.fromEntries([...new Set(filhos.map((n) => n[8]))].map((l) => [l, filhos.filter((n) => n[8] === l).map((n) => `${n[0]}(${1 + desc(n[0])})`).join(" ")])) };
}

// Eixo principal por caso: o dos filhos em relação ao pai em TODOS os níveis.
const EIXO = { me_LEFT: "x", me_RIGHT: "x", me_SIDE: "x", me_DOWN: "y", smm_logicalStructure: "x", smm_logicalStructureLeft: "x", smm_mindMap: "x", smm_organizationStructure: "y", mm_padrao: "x" };
const eixoDe = (caso) => EIXO[caso.replace(/^layout_/, "")] ?? "misto";

export function destilaLeis(layouts) {
  const out = {};
  for (const [caso, res] of Object.entries(layouts).sort()) {
    const A = res.arvores;
    const T2 = new Map(A.T2.nos.map((n) => [n[0], n]));
    const recolher = {};
    for (const n of A.T5.nos) {
      const m = T2.get(n[0]);
      if (n[4] === null || !m || m[4] === null) continue;
      if (n[3] <= 1) recolher[n[0]] = [r3(n[4] - m[4]), r3(n[5] - m[5])];
    }
    const t4 = Object.fromEntries(A.T4.nos.filter((n) => n[3] === 1).map((n) => [n[0], [n[6], n[7]]]));
    const altura = {};
    for (const n of A.T3.nos) if (n[4] !== null) (altura[n[3]] ??= new Set()).add(n[7]);
    const eixo = eixoDe(caso);
    out[caso] = {
      eixo,
      T2: analisaArvore(A.T2.nos, eixo),
      T3: analisaArvore(A.T3.nos, eixo),
      equilibrio: { T1: equilibrio(A.T1.nos), T2: equilibrio(A.T2.nos), T3: equilibrio(A.T3.nos), T6: equilibrio(A.T6.nos) },
      recolherA_T5menosT2: recolher,
      texto_T4_wh: t4,
      alturaDaCaixaPorProfundidade_T3: Object.fromEntries(Object.entries(altura).map(([k, v]) => [k, [...v].sort((a, b) => a - b)])),
    };
  }
  return { leis: out };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const dir = process.argv[2];
  const layouts = {};
  for (const f of fs.readdirSync(dir)) if (/^layout_.*\.json$/.test(f) && !f.endsWith(".meta.json")) layouts[f.slice(0, -5)] = JSON.parse(fs.readFileSync(path.join(dir, f), "utf8"));
  console.log(JSON.stringify(destilaLeis(layouts), null, 1));
}
