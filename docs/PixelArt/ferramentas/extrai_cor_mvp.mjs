// Extrai os vetores de COR do MVP web (docs/Pixel Art/app, nao versionado; mora no primario)
// para a fixtura de paridade da onda P2 do ph2d-pixel-core. O caminho do MVP e o 1.o argumento:
//   node docs/PixelArt/ferramentas/extrai_cor_mvp.mjs \
//     "/home/enio/Documentos/Projetos/PH2D/docs/Pixel Art/app" \
//     > crates/ph2d-pixel-core/tests/fixtures/mvp_color.json
// Familias: color (OKLab/OKLCH, transferencia sRGB, gamutFit, rampas), palette (rampOf/shade,
// permutacoes, duplicadas, hex, I/O), dither, quantize, ramp_quantize, overlay_color, cycling.
// Cada bloco nomeia a funcao do MVP de onde saiu. Floats em f64 cheio; o gate afirma o u8.
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const app = process.argv[2];
if (!app) throw new Error('uso: extrai_cor_mvp.mjs <caminho de docs/Pixel Art/app>');
const mod = (f) => import(pathToFileURL(path.join(app, 'engine', f)).href);
const K = await mod('oklch.js');
const RP = await mod('ramp.js');
const P = await mod('palette.js');
const IO = await mod('palette_io.js');
const D = await mod('dither.js');
const Q = await mod('quantize.js');
const RQ = await mod('ramp_quantize.js');
const OC = await mod('overlay_color.js');
const CY = await mod('cycling.js');
const CIO = await mod('cycling_io.js');
const C = await mod('compose.js');
const DOC = await mod('doc.js');

const arr = (v) => (ArrayBuffer.isView(v) ? Array.from(v) : v);
const err = (fn) => { try { return { ok: fn() }; } catch (e) { return { error: String(e.message) }; } };

// ── color ─────────────────────────────────────────────────────────────────────
// Entradas: as 9 do golden do MVP (Python/Ottosson), o cubo 6×6×6 e a escada de cinza inteira.
const amostras = [
  [255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255], [0, 0, 0], [128, 128, 128],
  [177, 62, 83], [65, 166, 246], [255, 205, 117],
];
for (let r = 0; r < 6; r++) for (let g = 0; g < 6; g++) for (let b = 0; b < 6; b++) {
  amostras.push([r * 51, g * 51, b * 51]);
}
const oklab = amostras.map((rgb) => {
  const lab = K.rgbToOklab(rgb);
  const lch = K.rgbToOklch(rgb);
  return { rgb, lab: [lab.L, lab.a, lab.b], lch: [lch.L, lch.C, lch.H], back: K.oklabToRgb(lab) };
});
const gray_roundtrip = [];
for (let v = 0; v < 256; v++) gray_roundtrip.push(K.oklabToRgb(K.rgbToOklab([v, v, v]))[0]);
const srgb_to_linear = Array.from({ length: 256 }, (_, v) => K.srgbToLinear(v));
const linear_to_srgb = [-5, -0.001, 0, 0.0001, 0.001, 0.0031308, 0.01, 0.05, 0.1, 0.2, 0.214, 0.5, 0.75, 0.99, 1, 1.5, 9]
  .map((x) => ({ x, v: K.linearToSrgb(x) }));
const lchs = [
  { L: 0.55, C: 0.9, H: 25 }, { L: 0.5, C: 0.05, H: 120 }, { L: 0.95, C: 0.3, H: 100 },
  { L: 0.1, C: 0.2, H: 270 }, { L: 0, C: 0.1, H: 0 }, { L: 1, C: 0.1, H: 0 }, { L: 0.7, C: 0.37, H: 330 },
  { L: 0.62, C: 0.25, H: 29 }, { L: 0.45, C: 0.31, H: 264 }, { L: 0.86, C: 0.29, H: 142 },
];
const gamut_fit = lchs.map((lch) => { const f = K.gamutFit(lch); return { lch: [lch.L, lch.C, lch.H], fit: [f.L, f.C, f.H], rgb: K.oklchToRgb(lch) }; });
const rampas = [];
for (const base of [[177, 62, 83], [65, 166, 246], [255, 205, 117], [128, 128, 128], [20, 12, 28]]) {
  for (const o of [{}, { steps: 2 }, { steps: 3 }, { steps: 8 }, { steps: 12 }, { steps: 5, hueShift: 60, chromaCurve: 0 },
    { steps: 5, hueShift: 0, chromaCurve: 0 }, { steps: 5, contrast: 0.7 }, { steps: 5, contrast: 0.2 },
    { steps: 5, chromaCurve: 0.8 }, { steps: 6, basePos: 0 }, { steps: 6, basePos: 5 }, { steps: 1 }, { steps: 99 }]) {
    rampas.push({ base, opts: o, ramp: RP.makeRamp(base, o) });
  }
}

// ── palette ───────────────────────────────────────────────────────────────────
const RAMPS = [{ name: 'r', indices: [3, 4, 5, 6, 7], wrap: false }, { name: 'b', indices: [5, 20, 21], wrap: false }];
const shade = [];
for (const wrap of [false, true]) {
  const pal = { colors: Array.from({ length: 32 }, () => [0, 0, 0]), transparentIndex: 0, ramps: RAMPS.map((r, i) => ({ ...r, indices: r.indices.slice(), wrap: i === 0 ? wrap : false })) };
  for (const i of [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 20, 21, 200]) {
    const ro = P.rampOf(pal, i);
    shade.push({ ramps: pal.ramps, index: i, ramp_of: ro ? { ramp: ro.rampIndex, pos: ro.pos } : null, up: P.shadeIndex(pal, i, +1), down: P.shadeIndex(pal, i, -1) });
  }
}
const move_permutation = [];
for (const [n, f, t] of [[5, 0, 3], [5, 4, 1], [5, 2, 2], [8, 1, 6], [8, 7, 0], [1, 0, 0], [33, 2, 11], [256, 0, 255], [256, 255, 0]]) {
  const p = P.movePermutation(n, f, t);
  move_permutation.push({ size: n, from: f, to: t, perm: arr(p), inverse: arr(P.invertPermutation(p)) });
}
const permute_palette = [];
for (const [perm, ti, ramps] of [[[3, 0, 1, 2], 0, [[1, 2, 3]]], [[1, 2, 3, 0], 2, [[0, 3], [1, 2]]], [[0, 1, 2, 3], 3, []]]) {
  const pal = { colors: [[0, 0, 0], [1, 1, 1], [2, 2, 2], [3, 3, 3]], transparentIndex: ti, ramps: ramps.map((ix, k) => ({ name: 'r' + k, indices: ix.slice(), wrap: false })) };
  P.permutePalette(pal, perm);
  permute_palette.push({ perm, transparent_before: ti, ramps_before: ramps, colors: pal.colors, transparent: pal.transparentIndex, ramps: pal.ramps.map((r) => r.indices) });
}
const duplicates = [];
for (const [colors, ti] of [
  [[[0, 0, 0], [9, 9, 9], [1, 2, 3], [9, 9, 9], [1, 2, 3]], 0],
  [[[9, 9, 9], [9, 9, 9], [9, 9, 9]], 0],
  [[[9, 9, 9], [9, 9, 9], [9, 9, 9]], 1],
  [[[1, 1, 1], [2, 2, 2], [3, 3, 3]], 0],
  [[[0, 0, 0], [0, 0, 0], [5, 5, 5], [0, 0, 0]], 2],
]) {
  duplicates.push({ colors, transparent: ti, dup: [...P.findDuplicates({ colors, transparentIndex: ti }).entries()] });
}
const hex = ['#ffcd75', 'ffcd75', '#f0a', 'F0A', '#FFCD75', 'nope', '#ffcd7', '', '#', '  #1a1c2c  ', '#12345678', 'zzzzzz', '#0000ff']
  .map((t) => ({ text: t, rgb: P.parseHex(t) }));
const to_hex = [[255, 205, 117], [0, 0, 0], [255, 255, 255], [1, 2, 3], [16, 160, 10]].map((rgb) => ({ rgb, hex: P.toHex(rgb) }));
const textos = [
  'GIMP Palette\r\nName: Sujo\r\nColumns: 4\r\n#comentário\r\n\r\n  10\t20   30\tuma cor\r\n255 0 0\r\n',
  'GIMP Palette\nName: x\n1 2 3\n',
  'GIMP Palette\n#\n0 0 0 preto\n255 255 255\tbranco\n300 0 0\n12 34\n',
  'não é gpl',
  '#1a1c2c\n5d275d\n\n  b13e53  \nlixo\n#f0a',
  '#ff0000\n#00ff00',
  '{"name":"Greyt-bit","author":"Sam","colors":["574368","8488d3","cfd3c1"]}',
  '{"colors":["ff0000"]}',
  '{"nome":"sem cores"}',
  '{isso não é json',
  '',
  'nada aqui é cor',
  'JASC-PAL\r\n0100\r\n2\r\n255 0 0\r\n0 255 0\r\n',
];
const parse = textos.map((t) => ({
  text: t,
  palette: err(() => IO.parsePalette(t)),
  gpl: err(() => IO.parseGpl(t)),
  hex_list: err(() => IO.parseHexList(t)),
  lospec: err(() => IO.parseLospec(t)),
}));
const sweetie = DOC.createDoc({ w: 2, h: 2 }).palette;
const export_ = {
  sweetie16: { colors: sweetie.colors, transparent: sweetie.transparentIndex },
  gpl: IO.toGpl(sweetie, 'Teste'),
  gpl_default_name: IO.toGpl(sweetie),
  hex_list: IO.toHexList(sweetie),
};
const lut = [];
for (const ti of [0, 5, 16, 99]) {
  lut.push({ colors: sweetie.colors, transparent: ti, lut: arr(C.buildLut({ colors: sweetie.colors, transparentIndex: ti })) });
}

// ── dither ────────────────────────────────────────────────────────────────────
const custom = new Uint8Array(64); for (let i = 0; i < 64; i++) custom[i] = (i * 7 + (i >> 3)) % 3 === 0 ? 1 : 0;
const pads = [...D.PATTERNS, D.makeCustomPattern(custom)];
const patterns = pads.map((p) => {
  const t = [];
  for (let y = -2; y < 14; y++) for (let x = -2; x < 14; x++) t.push(D.patternThreshold(p, x, y));
  return { id: p.id, size: p.size ?? null, levels: p.levels ?? null, mat: p.mat ? arr(p.mat) : null, seed: p.seed ?? null, origin: [-2, -2], side: 16, thresholds: t };
});
const bayer4 = []; for (let y = 0; y < 8; y++) for (let x = 0; x < 8; x++) bayer4.push(D.bayerThreshold(x, y));
const soft_coverage = []; for (let i = 0; i <= 30; i++) soft_coverage.push({ t: i / 20, c: D.softCoverage(i / 20) });
const soft_step = [];
for (const t of [0, 0.1, 0.45, 0.5, 0.6, 0.75, 0.9, 0.99, 1, 1.2]) {
  const on = []; for (let y = 0; y < 8; y++) for (let x = 0; x < 8; x++) on.push(D.softStep(t, x, y) ? 1 : 0);
  soft_step.push({ t, on });
}
const opacity_covers = [];
for (const op of [0, 1, 3, 4, 5, 63, 64, 127, 128, 129, 191, 200, 251, 252, 254, 255]) {
  const on = []; for (let y = 0; y < 8; y++) for (let x = 0; x < 8; x++) on.push(D.opacityCovers(op, x, y) ? 1 : 0);
  opacity_covers.push({ op, on });
}

// ── quantize ──────────────────────────────────────────────────────────────────
const img = (w, h, at) => {
  const rgba = new Uint8Array(w * h * 4);
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const [r, g, b, a = 255] = at(x, y); const o = (y * w + x) * 4;
    rgba[o] = r; rgba[o + 1] = g; rgba[o + 2] = b; rgba[o + 3] = a;
  }
  return rgba;
};
const imagens = {
  tres_cores: [4, 4, (x, y) => [[200, 30, 40], [30, 200, 40], [40, 30, 200]][(x + y) % 3]],
  transparente: [2, 2, (x) => (x === 0 ? [10, 20, 30, 255] : [0, 0, 0, 0])],
  alfa_corte: [4, 2, (x, y) => [x * 60, y * 90, 7, [0, 127, 128, 255][x]]],
  distintas_256: [16, 16, (x, y) => { const i = y * 16 + x; return [i, (i * 3) & 255, (i * 7) & 255]; }],
  grelha_12: [12, 12, (x, y) => [(x * 21) & 255, (y * 21) & 255, ((x + y) * 13) & 255]],
  grelha_16: [16, 16, (x, y) => [(x * 16) & 255, (y * 16) & 255, ((x * y) * 3) & 255]],
  meio_transparente: [8, 8, (x, y) => (y < 4 ? [x * 30, 0, 0, 255] : [0, 0, 0, 0])],
  uma_cor: [6, 6, () => [120, 60, 180]],
  degrade_cinza: [32, 1, (x) => [x * 8, x * 8, x * 8]],
};
const quantize = [];
for (const [nome, [w, h, at]] of Object.entries(imagens)) {
  const rgba = img(w, h, at);
  const ex = Q.exactPalette(rgba);
  const mc = {};
  for (const m of [1, 2, 4, 8, 16, 64]) { const q = Q.medianCut(rgba, m); mc[m] = { colors: q.colors, transparent: q.transparentIndex, indices: arr(q.indices) }; }
  quantize.push({ name: nome, w, h, rgba: arr(rgba), exact: ex ? { colors: ex.colors, transparent: ex.transparentIndex, indices: arr(ex.indices) } : null, median_cut: mc });
}

// ── ramp_quantize ─────────────────────────────────────────────────────────────
function material(hue, n, { L0 = 0.22, L1 = 0.86, shift = 26, C: c = 0.13 } = {}) {
  const out = [];
  for (let i = 0; i < n; i++) {
    const t = n === 1 ? 0 : i / (n - 1);
    out.push(K.oklchToRgb({ L: L0 + t * (L1 - L0), C: c * (0.55 + 0.45 * Math.sin(Math.PI * t)), H: hue + t * shift }));
  }
  return out;
}
const faixas = (bands, rows = 4) => {
  const w = bands.reduce((a, b) => a + b.length, 0);
  const rgba = new Uint8Array(w * rows * 4);
  for (let y = 0; y < rows; y++) { let x = 0; for (const band of bands) for (const c of band) { const o = (y * w + x) * 4; rgba[o] = c[0]; rgba[o + 1] = c[1]; rgba[o + 2] = c[2]; rgba[o + 3] = 255; x++; } }
  return { rgba, w, h: rows };
};
const RED = material(29, 40), BLUE = material(250, 40), GREEN = material(150, 40);
const greys = []; for (let i = 0; i < 24; i++) { const v = 10 + i * 9; greys.push([v, v, v]); }
const flat = Array.from({ length: 30 }, () => [200, 40, 40]);
const oito = []; for (let i = 0; i < 8; i++) oito.push(material(i * 45, 20));
const cenas = {
  rgb3: [RED, BLUE, GREEN], vermelho: [RED], vermelho_azul: [RED, BLUE], vermelho_cinzas: [RED, greys],
  mesma_tinta: [material(29, 30, { L0: 0.20, L1: 0.50 }), material(31, 30, { L0: 0.55, L1: 0.90 })],
  hue_shift_40: [material(29, 60, { shift: 40 })], chapado: [flat], oito_materiais: oito,
};
const ramp_quantize = [];
for (const [nome, bands] of Object.entries(cenas)) {
  const { rgba, w, h } = faixas(bands);
  const hist = RQ.histogram(rgba);
  const fams = RQ.clusterMaterials(hist);
  const runs = {};
  for (const m of [6, 12, 16, 24, 26, 30, 40]) {
    const q = RQ.rampQuantize(rgba, { maxColors: m });
    runs[m] = { colors: q.colors, transparent: q.transparentIndex, indices: arr(q.indices), ramps: q.ramps, reduced_bits: q.reducedBits };
  }
  ramp_quantize.push({
    name: nome, w, h, rgba: arr(rgba),
    histogram: { n: hist.n, shift: hist.shift, mass: arr(hist.mass).slice(0, hist.n) },
    families: fams.map((f) => ({ members: arr(f.members), neutral: !!f.neutral })),
    ramp_from_family: fams.map((f) => [2, 6, 8].map((k) => RQ.rampFromFamily(f, hist, k).map((s) => ({ L: s.L, rgb: s.rgb })))),
    runs,
  });
}
const allocate = [];
for (const [fams, budgets] of [
  [[{ Lmin: 0.1, Lmax: 0.9 }, { Lmin: 0.4, Lmax: 0.42 }], [4, 6, 12, 20]],
  [[{ Lmin: 0.1, Lmax: 0.9 }, { Lmin: 0.2, Lmax: 0.7 }, { Lmin: 0.3, Lmax: 0.35 }], [6, 7, 12, 18, 30]],
  [[{ Lmin: 0.5, Lmax: 0.5 }], [2, 20]],
]) for (const b of budgets) allocate.push({ families: fams, budget: b, steps: arr(RQ.allocateSteps(fams, b)) });
const hue_name = Array.from({ length: 360 }, (_, d) => RQ.hueName(d));

// ── overlay_color ─────────────────────────────────────────────────────────────
const avoid = OC.avoidOf(sweetie);
const ranked = OC.rankedOverlayColors(avoid);
const picks = []; { const taken = []; for (let i = 0; i < 5; i++) { const h = OC.pickForDoc(sweetie, taken); picks.push(h); taken.push(h); } }
const cheia = { colors: Array.from({ length: 256 }, (_, i) => [(i * 37) & 255, (i * 91) & 255, (i * 173) & 255]), transparentIndex: 0 };
const overlay_color = {
  candidates: OC.candidates(), checker: OC.CHECKER_RGB,
  lab_distance: [[[0, 0, 255], [0, 0, 200]], [[0, 255, 0], [0, 200, 0]], [[0, 0, 0], [255, 255, 255]], [[177, 62, 83], [65, 166, 246]], [[10, 10, 10], [10, 10, 10]]]
    .map(([a, b]) => ({ a, b, d: OC.labDistance(a, b) })),
  avoid_sweetie: avoid, avoid_transparent_5: OC.avoidOf({ ...sweetie, transparentIndex: 5 }),
  ranked_sweetie: ranked.map((r) => ({ hex: r.hex, ...Object.fromEntries(Object.entries(r).filter(([k]) => k !== 'hex' && typeof r[k] === 'number')) })),
  picks_sweetie: picks, pick_full: OC.pickForDoc(cheia, []),
  pick_junk_taken: OC.pickForDoc(sweetie, [null, undefined, 'azul', '#zzz', 12]),
};

// ── cycling ───────────────────────────────────────────────────────────────────
const cyc_amount = [];
for (const mode of CY.CYCLE_MODES) for (const size of [1, 2, 6]) for (let s = 0; s <= 14; s++) {
  cyc_amount.push({ mode, steps: s, size, k: CY.cycleAmount(mode, s, size) });
}
const cycPal = {
  colors: [[10, 10, 10], [40, 0, 0], [80, 0, 0], [120, 0, 0], [160, 0, 0], [200, 0, 0], [240, 0, 0], [128, 128, 128]],
  transparentIndex: 0,
};
const palette_at_time = [];
for (const mode of CY.CYCLE_MODES) for (const blend of [false, true]) for (const sps of [10, 3, 0]) {
  const cycles = [{ low: 1, high: 6, stepsPerSecond: sps, mode, blend, enabled: true }];
  for (const t of [0, 50, 100, 137, 150, 250, 300, 500, 600, 700, 1200, 1999]) {
    palette_at_time.push({ cycles, t, lut: arr(CY.paletteAtTime(cycPal, cycles, t)).slice(0, cycPal.colors.length * 4) });
  }
}
for (const cycles of [
  [{ low: 1, high: 6, stepsPerSecond: 10, mode: 'forward', blend: false, enabled: false }],
  [{ low: 1, high: 99, stepsPerSecond: 10, mode: 'forward', blend: false, enabled: true }],
  [{ low: 1, high: 3, stepsPerSecond: 10, mode: 'forward', blend: false, enabled: true }, { low: 4, high: 6, stepsPerSecond: 5, mode: 'reverse', blend: true, enabled: true }],
]) for (const t of [0, 100, 250]) palette_at_time.push({ cycles, t, lut: arr(CY.paletteAtTime(cycPal, cycles, t)).slice(0, cycPal.colors.length * 4) });
const HUE_WHEEL = [[200, 60, 60], [200, 140, 60], [120, 180, 60], [60, 180, 140], [60, 120, 200], [140, 60, 200]];
const rampasCiclo = {
  hue_wheel: HUE_WHEEL,
  monotonica: [[20, 20, 20], [80, 80, 80], [140, 140, 140], [200, 200, 200]],
  ja_fechada: [[70, 40, 50], [110, 60, 70], [150, 90, 90], [150, 110, 100], [110, 80, 90], [95, 55, 70]],
  reta: [[30, 20, 60], [90, 40, 90], [150, 70, 110], [210, 120, 140]],
  ciclo: [[100, 60, 60], [140, 80, 60], [140, 100, 80], [110, 70, 65]],
};
const cycling = {
  modes: CY.CYCLE_MODES, amount: cyc_amount, palette: cycPal, palette_at_time,
  has_live: [[], [{ enabled: false, stepsPerSecond: 10, low: 1, high: 6 }], [{ enabled: true, stepsPerSecond: 0, low: 1, high: 6 }],
    [{ enabled: true, stepsPerSecond: 5, low: 1, high: 1 }], [{ enabled: true, stepsPerSecond: 5, low: 1, high: 6 }]]
    .map((cycles) => ({ cycles, live: CY.hasLiveCycle(cycles) })),
  make_cycle: [[6, 2, 'x'], [1, 4], [3, 3, 'y']].map((a) => ({ args: a, cycle: CY.makeCycle(...a) })),
  ramps: Object.fromEntries(Object.entries(rampasCiclo).map(([k, r]) => [k, { colors: r, seam: CIO.seamRatio(r), closed: CIO.closeRampCycle(r), closed_seam: CIO.seamRatio(CIO.closeRampCycle(r)) }])),
  canvas_cycle: ['forward', 'reverse', 'pingpong', 'sine'].map((mode) => {
    const s = CIO.toCanvasCycle(cycPal, [{ low: 1, high: 6, stepsPerSecond: 10, mode }], 2, 2, new Uint8Array([1, 2, 3, 4]));
    return { mode, scene: { ...s, pixels: arr(s.pixels) } };
  }),
};

process.stdout.write(JSON.stringify({
  source: 'docs/Pixel Art/app/engine/{oklch,ramp,palette,palette_io,dither,quantize,ramp_quantize,overlay_color,cycling,cycling_io,compose}.js (MVP web, 25/10) — extrai_cor_mvp.mjs',
  laws: {
    color: 'oklch.js rgbToOklab/oklabToRgb/rgbToOklch/gamutFit/oklchToRgb, srgbToLinear/linearToSrgb; ramp.js makeRamp',
    palette: 'palette.js rampOf/shadeIndex/movePermutation/invertPermutation/permutePalette/findDuplicates/parseHex/toHex; palette_io.js parse*/to*; compose.js buildLut',
    dither: 'dither.js patternThreshold (PATTERNS + makeCustomPattern), bayerThreshold, softCoverage, softStep, opacityCovers',
    quantize: 'quantize.js exactPalette / medianCut (indice 0 = transparente, alfa < 128 corta)',
    ramp_quantize: 'ramp_quantize.js histogram/clusterMaterials/rampFromFamily/allocateSteps/rampQuantize/hueName',
    overlay_color: 'overlay_color.js candidates/labDistance/avoidOf/rankedOverlayColors/pickForDoc',
    cycling: 'cycling.js cycleAmount/paletteAtTime/hasLiveCycle/makeCycle; cycling_io.js seamRatio/closeRampCycle/toCanvasCycle',
  },
  color: { oklab, gray_roundtrip, srgb_to_linear, linear_to_srgb, gamut_fit, ramps: rampas },
  palette: { shade, move_permutation, permute_palette, duplicates, hex, to_hex, parse, export: export_, lut },
  dither: { patterns, bayer4, soft_core: D.SOFT_CORE, soft_coverage, soft_step, opacity_covers },
  quantize, ramp_quantize: { scenes: ramp_quantize, allocate, hue_name }, overlay_color, cycling,
}) + '\n');
