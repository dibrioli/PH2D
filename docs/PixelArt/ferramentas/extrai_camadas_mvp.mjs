// Extrai os vetores de CAMADAS do MVP web (docs/Pixel Art/app, nao versionado; mora no primario)
// para a fixtura de paridade da onda P3 do ph2d-pixel-core. O caminho do MVP e o 1.o argumento:
//   node docs/PixelArt/ferramentas/extrai_camadas_mvp.mjs \
//     "/home/enio/Documentos/Projetos/PH2D/docs/Pixel Art/app" \
//     > crates/ph2d-pixel-core/tests/fixtures/mvp_layers.json
// Familias (de compose.test.js, layers.test.js e da funcao que eles exercitam):
//   compose  — composeIndices + plano de dono sobre pilhas sorteadas (visivel, opacidade, clip);
//   lut      — buildLut (o indice transparente tem alfa 0, fora da paleta alfa 0);
//   coverage — opacityCovers(op, x, y), o padrao 8x8 de cada opacidade 0..255 (Indexed);
//   ops      — sequencias layer.insert/delete/move/setProps com a ordem e as props depois de
//              cada passo, de cada undo e de cada redo;
//   limits   — MAX_LAYERS e as recusas (apagar a ultima camada, passar do maximo).
// O MVP e so de indices: os 19 modos de mistura NAO existem nele (o oraculo deles e o Trial).
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const app = process.argv[2];
if (!app) throw new Error('uso: extrai_camadas_mvp.mjs <caminho de docs/Pixel Art/app>');
const mod = (f) => import(pathToFileURL(path.join(app, 'engine', f)).href);
const DOC = await mod('doc.js');
const C = await mod('compose.js');
const OPS = await mod('ops.js');
const D = await mod('dither.js');

const err = (fn) => { try { fn(); return null; } catch (e) { return String(e.message); } };

// splitmix32: o RNG da casa do MVP, para as pilhas sorteadas serem reprodutiveis.
let seed = 0x9e3779b9 >>> 0;
const rnd = () => {
  seed = (seed + 0x9e3779b9) >>> 0;
  let z = seed;
  z = Math.imul(z ^ (z >>> 16), 0x85ebca6b) >>> 0;
  z = Math.imul(z ^ (z >>> 13), 0xc2b2ae35) >>> 0;
  return (z ^ (z >>> 16)) >>> 0;
};
const pick = (a) => a[rnd() % a.length];

// ── compose ───────────────────────────────────────────────────────────────────
const OPACITIES = [0, 1, 63, 64, 127, 128, 191, 200, 254, 255, 255, 255];
const compose = [];
// as 160 primeiras sorteiam tudo; as 80 seguintes sao opacas sem clip (onde a lei e a do Aseprite)
for (let k = 0; k < 240; k++) {
  const w = pick([1, 3, 4, 8, 9]);
  const h = pick([1, 4, 8, 5]);
  const doc = DOC.createDoc({ w, h });
  const nl = 1 + (rnd() % 5);
  for (let i = 1; i < nl; i++) doc.layers.push(DOC.normalizeLayer({ id: i + 1, name: `L${i + 1}` }));
  for (const l of doc.layers) {
    l.visible = rnd() % 6 !== 0;
    l.opacity = k < 160 ? pick(OPACITIES) : 255;
    l.clip = k < 160 && rnd() % 4 === 0;
    if (rnd() % 7 === 0) continue;               // camada sem cel
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
      // ~40 % transparente (indice 0), o resto em indices 1..15
      const v = rnd() % 10 < 4 ? 0 : 1 + (rnd() % 15);
      if (v) DOC.setCell(doc, l.id, 1, x, y, v);
    }
  }
  if (rnd() % 5 === 0) doc.palette.transparentIndex = 1 + (rnd() % 3);
  const owner = new Uint8Array(w * h);
  const out = C.composeIndices(doc, 1, new Uint8Array(w * h), owner);
  compose.push({
    w, h,
    transparent: doc.palette.transparentIndex,
    layers: doc.layers.map((l) => {
      const cel = DOC.getCel(doc, l.id, 1);
      const cells = [];
      for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) cells.push(DOC.getCell(doc, l.id, 1, x, y));
      return { visible: l.visible, opacity: l.opacity, clip: l.clip, cells: cel ? cells : null };
    }),
    out: Array.from(out),
    owner: Array.from(owner),
  });
}

// ── lut ───────────────────────────────────────────────────────────────────────
const lut = [0, 5].map((t) => {
  const pal = DOC.defaultPalette();
  pal.transparentIndex = t;
  return { colors: pal.colors, transparent: t, lut: Array.from(C.buildLut(pal)) };
});

// ── coverage ──────────────────────────────────────────────────────────────────
// mask[op] = os 64 bits de y*8+x, em 8 bytes (linha y = byte y, bit x).
const coverage = [];
for (let op = 0; op < 256; op++) {
  const rows = [];
  for (let y = 0; y < 8; y++) {
    let b = 0;
    for (let x = 0; x < 8; x++) if (D.opacityCovers(op, x, y)) b |= 1 << x;
    rows.push(b);
  }
  coverage.push(rows);
}

// ── ops ───────────────────────────────────────────────────────────────────────
const snap = (doc) => doc.layers.map((l) => ({
  id: l.id, name: l.name, visible: l.visible, opacity: l.opacity, locked: l.locked,
  cell00: DOC.getCell(doc, l.id, 1, 0, 0),
}));
const ops = [];
for (let k = 0; k < 40; k++) {
  const doc = DOC.createDoc({ w: 2, h: 2 });
  const hist = OPS.createHistory();
  DOC.setCell(doc, 1, 1, 0, 0, 1);
  const steps = [];
  const n = 3 + (rnd() % 8);
  for (let s = 0; s < n; s++) {
    const len = doc.layers.length;
    const r = rnd() % 10;
    let op;
    if (r < 4 || len < 2) {
      const id = doc.meta.nextId; doc.meta.nextId = id + 1;
      const cels = {};
      op = { type: 'layer.insert', index: rnd() % (len + 1), layer: { id, name: `L${id}` }, cels };
      steps.push({ op: 'insert', index: op.index, id });
      OPS.commit(doc, hist, op);
      DOC.setCell(doc, id, 1, 0, 0, 1 + (id % 15));
      steps[steps.length - 1].after = snap(doc);
      continue;
    } else if (r < 6) {
      const id = pick(doc.layers).id;
      op = { type: 'layer.delete', id };
      steps.push({ op: 'delete', id });
    } else if (r < 8) {
      const from = rnd() % len, to = rnd() % len;
      op = { type: 'layer.move', from, to };
      steps.push({ op: 'move', from, to });
    } else {
      const id = pick(doc.layers).id;
      const props = pick([{ opacity: pick(OPACITIES) }, { visible: false }, { name: `n${s}` }, { locked: true, opacity: 9 }]);
      op = { type: 'layer.setProps', id, props };
      steps.push({ op: 'props', id, props });
    }
    OPS.commit(doc, hist, op);
    steps[steps.length - 1].after = snap(doc);
  }
  const undos = [];
  while (OPS.canUndo(hist)) { OPS.undo(doc, hist); undos.push(snap(doc)); }
  const redos = [];
  while (OPS.canRedo(hist)) { OPS.redo(doc, hist); redos.push(snap(doc)); }
  ops.push({ steps, undos, redos });
}

// ── limits ────────────────────────────────────────────────────────────────────
const one = DOC.createDoc({ w: 1, h: 1 });
const limits = {
  max_layers: DOC.MAX_LAYERS,
  delete_last: err(() => OPS.applyOp(one, { type: 'layer.delete', id: 1 })),
  setprops_keeps_id: (() => {
    const d = DOC.createDoc({ w: 1, h: 1 });
    OPS.applyOp(d, { type: 'layer.setProps', id: 1, props: { id: 99, name: 'x' } });
    return d.layers[0].id;
  })(),
};

process.stdout.write(JSON.stringify({
  source: 'docs/Pixel Art/app (MVP web), extrai_camadas_mvp.mjs',
  note: 'MVP so de indices: opacidade de camada = cobertura Bayer 8x8 ancorada no canvas; clip = mascara da camada de baixo (nao existe no Aseprite).',
  compose, lut, coverage, ops, limits,
}) + '\n');
