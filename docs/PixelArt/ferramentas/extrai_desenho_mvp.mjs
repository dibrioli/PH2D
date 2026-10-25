// Extrai os vetores de DESENHO do MVP web (docs/Pixel Art/app, nao versionado; mora no primario)
// para a fixtura de paridade da onda P1 do ph2d-pixel-core. O caminho do MVP e o 1.o argumento:
//   node docs/PixelArt/ferramentas/extrai_desenho_mvp.mjs \
//     "/home/enio/Documentos/Projetos/PH2D/docs/Pixel Art/app" \
//     > crates/ph2d-pixel-core/tests/fixtures/mvp_draw.json
// Cada bloco nomeia a funcao do MVP de onde saiu. A ordem das celulas e a do MVP.
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const app = process.argv[2];
if (!app) throw new Error('uso: extrai_desenho_mvp.mjs <caminho de docs/Pixel Art/app>');
const mod = (f) => import(pathToFileURL(path.join(app, 'engine', f)).href);
const R = await mod('raster.js');
const S = await mod('symmetry.js');
const G = await mod('rng.js');

const collect = (fn) => { const out = []; fn((x, y) => out.push([x, y])); return out; };

// pixel-perfect em lote e ao vivo (ppPush: os eventos +pinta / -retrai, na ordem).
const caminhos = [
  [[0, 0], [1, 0], [1, 1], [2, 1], [2, 2]],
  [[0, 0], [1, 0], [2, 0], [3, 0]],
  [[0, 0], [1, 1], [2, 2], [3, 3]],
  [[5, 5]],
  [[0, 0], [1, 0]],
  [[0, 0], [1, 0], [1, 1]],
  [[0, 0], [0, 1], [1, 1], [1, 2], [2, 2], [3, 2], [3, 3]],
  [[4, 4], [5, 4], [5, 5], [5, 5], [6, 5], [6, 6], [7, 6], [8, 6], [8, 7]],
  [[0, 0], [1, 0], [2, 0], [2, 1], [2, 2], [1, 2], [0, 2], [0, 1], [0, 0]],
];
const pixel_perfect = caminhos.map((p) => {
  const f = R.createPixelPerfect();
  const events = [];
  for (const [x, y] of p) {
    R.ppPush(f, x, y, (a, b) => events.push(['+', a, b]), (a, b) => events.push(['-', a, b]));
  }
  return { path: p, batch: R.pixelPerfect(p), live: events };
});

// curva quadratica (controle no meio, na ponta, alem da ponta, em S largo, degenerada).
const curvas = [
  [0, 0, 5, 0, 10, 0], [0, 0, 0, 0, 10, 4], [0, 0, 10, 4, 10, 4], [0, 0, 15, 6, 10, 4],
  [0, 0, 8, -6, 16, 0], [0, 10, 0, 0, 10, 0], [3, 3, 3, 3, 3, 3], [0, 0, 20, 20, 0, 12],
  [2, 9, 11, -4, 20, 9], [0, 0, 4, 8, 1, 1],
];
const curve = curvas.map((c) => ({ args: c, cells: R.curvePath(...c) }));

// retangulos e elipses (bbox, cantos em qualquer ordem; pares, impares, achatadas, 1 celula).
const caixas = [
  [0, 0, 0, 0], [0, 0, 1, 0], [0, 0, 1, 1], [2, 3, 6, 8], [6, 8, 2, 3], [0, 0, 6, 6], [0, 0, 7, 7],
  [0, 0, 9, 3], [0, 0, 3, 9], [0, 0, 12, 1], [0, 0, 1, 12], [-4, -2, 5, 7], [0, 0, 15, 10],
  [0, 0, 2, 2], [0, 0, 4, 2], [0, 0, 31, 31],
];
const rect = caixas.map((b) => ({
  box: b,
  outline: collect((cb) => R.rectCells(...b, false, cb)),
  filled: collect((cb) => R.rectCells(...b, true, cb)),
}));
const ellipse = caixas.map((b) => ({
  box: b,
  outline: collect((cb) => R.ellipseCells(...b, cb)),
  filled: collect((cb) => R.ellipseFilledCells(...b, cb)),
}));

// carimbos 1..16, redondo e quadrado: os offsets na ordem do MVP.
const stamps = [];
for (const shape of ['circle', 'square']) {
  for (let d = 1; d <= 16; d++) stamps.push({ shape, d, offsets: Array.from(R.stamp(shape, d)) });
}

// balde (indices): grelhas ASCII, sementes, o resultado contiguo 4-conectado e o global.
const grelhas = [
  ['#####', '#...#', '##..#', '#.###', '#####'],
  ['#######', '#.....#', '#.###.#', '#.....#', '#######'],
  ['#.#', '...', '#.#'],
  ['........', '.######.', '.#....#.', '.#.##.#.', '.#.##.#.', '.#....#.', '.######.', '........'],
];
const fill = [];
for (const art of grelhas) {
  const h = art.length, w = art[0].length;
  const get = (x, y) => (art[y][x] === '#' ? 1 : 0);
  for (const [sx, sy] of [[0, 0], [1, 1], [2, 1], [3, 3], [-1, 0], [w, 0]]) {
    fill.push({
      art, seed: [sx, sy],
      contiguous: collect((cb) => R.floodFill(get, w, h, sx, sy, cb)),
      global: (sx < 0 || sy < 0 || sx >= w || sy >= h) ? []
        : collect((cb) => R.replaceIndex(get, w, h, get(sx, sy), cb)),
    });
  }
}

// simetria: imagens de uma celula, e o funil com um carimbo PAR e um L (W24: o espelho reflete o carimbo).
const symmetry = [];
for (const [w, h] of [[8, 8], [7, 7], [11, 9], [13, 7]]) {
  for (const mode of S.SYM_MODES) {
    for (const [x, y] of [[0, 0], [2, 3], [3, 3], [w - 1, h - 1], [(w - 1) >> 1, 1]]) {
      const out = [];
      S.symmetryImages(x, y, mode, w, h, (a, b, fx, fy) => out.push([a, b, fx, fy]));
      symmetry.push({ size: [w, h], mode, cell: [x, y], images: out });
    }
  }
}
const funil = [];
const carimbos = { par2: Array.from(R.stamp('square', 2)), L: [0, 0, 1, 0, 0, 1, 0, 2] };
for (const [nome, offs] of Object.entries(carimbos)) {
  for (const mode of S.SYM_MODES) {
    for (const tiling of [false, true]) {
      for (const [cx, cy] of [[1, 1], [0, 0], [6, 5]]) {
        const out = [];
        S.expandStamp(cx, cy, offs, mode, 8, 7, tiling, null, (x, y, dx, dy, k) => out.push([x, y, k]));
        funil.push({ stamp: nome, offsets: offs, mode, tiling, size: [8, 7], cell: [cx, cy], out });
      }
    }
  }
}

// splitmix32: os primeiros 8 de algumas sementes (o spray sorteia por ele).
const rng = [0, 1, 42, -1, 0x7fffffff].map((seed) => {
  const next = G.splitmix32(seed);
  return { seed, first: Array.from({ length: 8 }, next) };
});

process.stdout.write(JSON.stringify({
  source: 'docs/Pixel Art/app/engine/{raster,symmetry,rng}.js (MVP web, 25/10) — extrai_desenho_mvp.mjs',
  laws: {
    pixel_perfect: 'raster.js pixelPerfect / ppPush (Ricky Han; pinta ao vivo, retrai o cotovelo)',
    curve: 'raster.js curvePath (quadratica; colinear dentro da corda = lineCells; pixel-perfect)',
    rect: 'raster.js rectCells', ellipse: 'raster.js ellipseCells / ellipseFilledCells (Zingl, pela bbox)',
    stamps: 'raster.js stamp (folga 0.1, ancora floor((d-1)/2))',
    fill: 'raster.js floodFill (scanline, 4-conectado) / replaceIndex',
    symmetry: 'symmetry.js symmetryImages / expandStamp (o espelho reflete o carimbo; tiling dobra)',
    rng: 'rng.js splitmix32',
  },
  pixel_perfect, curve, rect, ellipse, stamps, fill, symmetry, funil, rng,
}) + '\n');
