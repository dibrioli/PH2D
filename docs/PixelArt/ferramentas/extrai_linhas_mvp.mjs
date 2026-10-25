// Extrai os vetores de linha do MVP web (docs/Pixel Art/app, nao versionado) para a
// fixtura de paridade do ph2d-pixel-core. Correr da raiz do repo primario:
//   node docs/PixelArt/ferramentas/extrai_linhas_mvp.mjs > crates/ph2d-pixel-core/tests/fixtures/mvp_line_cells.json
import { lineCells } from '../../Pixel Art/app/engine/raster.js';

const casos = [
  [0, 0, 9, 4], [3, 7, 3, 1], [0, 0, 7, 7], [2, 9, 11, 3], [5, 5, 5, 5], [-3, 2, 6, -4],
  [0, 0, 1, 0], [0, 0, 0, 1], [0, 0, 2, 1], [0, 0, 1, 2], [0, 0, 3, 1], [7, 2, 0, 5],
  [0, 0, 31, 13], [31, 0, 0, 31], [10, 10, -10, 3], [4, 0, 0, 4], [0, 4, 4, 0], [1, 1, 6, 2],
];
const out = casos.map(([x0, y0, x1, y1]) => {
  const cells = [];
  lineCells(x0, y0, x1, y1, (x, y) => cells.push([x, y]));
  return { from: [x0, y0], to: [x1, y1], cells };
});
process.stdout.write(JSON.stringify({
  source: 'docs/Pixel Art/app/engine/raster.js lineCells (MVP web, 25/10)',
  law: 'canonicalised Bresenham: endpoints ordered (min x, then min y); cells in canonical order',
  cases: out,
}) + '\n');
