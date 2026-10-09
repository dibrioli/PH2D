// O ORÁCULO da cascata (ciclo 17, doc 126 §2) — corre o `stagger()` e o `easing()` do anime.js
// (MIT) sobre as NOSSAS grelhas e imprime uma linha por caso. Recebe o caminho do `anime.js`.
// Formato: `STAGGER <caso> <n> <cols> <rows> <from> <axis> <ease> | v0 v1 ...` (valor com stagger(1))
//          `EASE <nome> | e(t0) e(t1) ...` sobre t = k/64, k = 0..64.
const anime = require(process.argv[2]);
const fmt = (x) => (+x).toFixed(6);
const casos = [
  // [nome, n, grid|null, from, axis|null, easing|null]
  ['fila9_first', 9, null, 'first', null, null],
  ['fila10_center', 10, null, 'center', null, null],
  ['fila9_last', 9, null, 'last', null, null],
  ['fila10_idx3', 10, null, 3, null, null],
  ['g7x5_first', 35, [7, 5], 'first', null, null],
  ['g7x5_center', 35, [7, 5], 'center', null, null],
  ['g7x5_last', 35, [7, 5], 'last', null, null],
  ['g7x5_idx12', 35, [7, 5], 12, null, null],
  ['g7x5_center_x', 35, [7, 5], 'center', 'x', null],
  ['g7x5_center_y', 35, [7, 5], 'center', 'y', null],
  ['g7x5_first_x', 35, [7, 5], 'first', 'x', null],
  ['g10x10_center', 100, [10, 10], 'center', null, null],
  ['g8x3_parcial_first', 20, [8, 3], 'first', null, null],
  ['g8x3_parcial_center', 20, [8, 3], 'center', null, null],
  ['g8x3_parcial_last', 20, [8, 3], 'last', null, null],
  ['g8x3_parcial_idx13_y', 20, [8, 3], 13, 'y', null],
  ['g7x5_center_inquad', 35, [7, 5], 'center', null, 'easeInQuad'],
  ['g7x5_first_outcubic', 35, [7, 5], 'first', null, 'easeOutCubic'],
  ['g8x3_parcial_first_inquad', 20, [8, 3], 'first', null, 'easeInQuad'],
  ['fila10_center_inquad', 10, null, 'center', null, 'easeInQuad'],
];
for (const [nome, n, grid, from, axis, ease] of casos) {
  const opt = { from };
  if (grid) opt.grid = grid;
  if (axis) opt.axis = axis;
  if (ease) opt.easing = ease;
  const f = anime.stagger(1, opt);
  const v = Array.from({ length: n }, (_, i) => fmt(f({}, i, n)));
  const g = grid || [n, 1];
  console.log(`STAGGER ${nome} ${n} ${g[0]} ${g[1]} ${from} ${axis || '-'} ${ease || '-'} | ${v.join(' ')}`);
}
const ts = Array.from({ length: 65 }, (_, k) => k / 64);
for (const nome of ['easeInSine', 'easeOutSine', 'easeInOutSine', 'easeInExpo', 'easeOutExpo',
  'easeInOutExpo', 'easeOutElastic(1, .3)', 'easeInElastic(1, .3)', 'easeInOutElastic(1, .3)',
  'cubicBezier(.25,.1,.25,1)', 'cubicBezier(.68,-0.55,.27,1.55)', 'cubicBezier(.42,0,.58,1)']) {
  const e = anime.easing(nome);
  console.log(`EASE ${nome.replace(/\s+/g, '')} | ${ts.map((t) => fmt(e(t))).join(' ')}`);
}
