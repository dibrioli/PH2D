// Extrai os vetores de SELECAO e TRANSFORMACAO do MVP web (docs/Pixel Art/app, nao versionado;
// mora no primario) para a fixtura de paridade da onda P6 do ph2d-pixel-core. O caminho do MVP e o
// 1.o argumento:
//   node docs/PixelArt/ferramentas/extrai_selecao_mvp.mjs \
//     "/home/enio/Documentos/Projetos/PH2D/docs/Pixel Art/app" \
//     > crates/ph2d-pixel-core/tests/fixtures/mvp_selection.json
// Familias (de selection/morphology/transform/affine/distort.test.js e das funcoes que exercitam):
//   mask_seq     — rectInto/wandInto (replace/add/subtract, 4/8) e invertMask em sequencia, a mascara
//                  depois de cada passo; no fim maskBounds e extractRegion (data + keep);
//   morph        — distanceTo/growMask/shrinkMask/borderMask, 4 e 8, n = 0..3;
//   perm         — flipH/flipV/rot90cw/rot90ccw/rot180 e scaleNearest n = 1..4;
//   rotated_size — rotatedSize com angulos inteiros e fraccionarios;
//   rotate       — rotateNearest e rotateSprite (a variante do CANTO ×8, transform.js);
//   xform        — transformIndices (nearest e rotsprite = a variante do CENTRO, distort.js) sobre
//                  molduras compostas e assentadas, com e sem clip; transformFields de cada uma;
//   affine       — decompose/compose ida e volta, withTransformFields e dragTransform por alca;
//   offset_wrap  — offsetWrap (o Shift com volta).
// O MVP nao tem laco, laco poligonal, varinha com tolerancia, Color Range, elipse como selecao,
// Canvas/Sprite Size, Crop/Trim, Copy Merged nem Rotate Canvas: o oraculo deles e o Trial.
// Mascara = linhas de '0'/'1' (uma string por linha). Indices = arrays de inteiros.
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const app = process.argv[2];
if (!app) throw new Error('uso: extrai_selecao_mvp.mjs <caminho de docs/Pixel Art/app>');
const mod = (f) => import(pathToFileURL(path.join(app, 'engine', f)).href);
const SEL = await mod('selection.js');
const TR = await mod('transform.js');
const AF = await mod('affine.js');
const FX = await mod('effects.js');

// splitmix32, o mesmo de extrai_quadros_mvp.mjs (outra semente).
let seed = 0x5e1ec7ed >>> 0;
const rnd = () => {
  seed = (seed + 0x9e3779b9) >>> 0;
  let z = seed;
  z = Math.imul(z ^ (z >>> 16), 0x85ebca6b) >>> 0;
  z = Math.imul(z ^ (z >>> 13), 0xc2b2ae35) >>> 0;
  return (z ^ (z >>> 16)) >>> 0;
};
const pick = (a) => a[rnd() % a.length];
const range = (lo, hi) => lo + (rnd() % (hi - lo + 1));

const rows = (m, w, h) => {
  const out = [];
  for (let y = 0; y < h; y++) {
    let s = '';
    for (let x = 0; x < w; x++) s += m[y * w + x] ? '1' : '0';
    out.push(s);
  }
  return out;
};
const arr = (a) => Array.from(a);
// Uma grelha de indices com MANCHAS (vizinhos repetem), para a varinha e o EPX terem o que ver.
const grid = (w, h, k) => {
  const g = new Uint8Array(w * h);
  for (let i = 0; i < g.length; i++) {
    const r = rnd() % 10;
    if (r < 4 && i > 0) g[i] = g[i - 1];
    else if (r < 7 && i >= w) g[i] = g[i - w];
    else g[i] = rnd() % k;
  }
  return g;
};
const randMask = (w, h) => {
  const m = SEL.newMask(w, h);
  const n = range(0, 3);
  for (let i = 0; i < n; i++) {
    SEL.rectInto(m, w, h, { x: range(-1, w), y: range(-1, h), w: range(1, w), h: range(1, h) }, i === 0 ? 'replace' : pick(['add', 'subtract']));
  }
  return m;
};

// ── mask_seq ──────────────────────────────────────────────────────────────────
const mask_seq = [];
for (let k = 0; k < 60; k++) {
  const w = range(1, 9), h = range(1, 9);
  const idx = grid(w, h, range(2, 4));
  const m = SEL.newMask(w, h);
  const steps = [];
  const n = range(1, 7);
  for (let s = 0; s < n; s++) {
    const r = rnd() % 10;
    const mode = pick(['replace', 'add', 'subtract']);
    let step;
    if (r < 5) {
      const rect = { x: range(-2, w), y: range(-2, h), w: range(0, w + 2), h: range(0, h + 2) };
      SEL.rectInto(m, w, h, rect, mode);
      step = { op: 'rect', rect, mode };
    } else if (r < 9) {
      const sx = range(-1, w), sy = range(-1, h), conn = pick([4, 8]);
      SEL.wandInto(m, (x, y) => idx[y * w + x], w, h, sx, sy, conn, mode);
      step = { op: 'wand', x: sx, y: sy, conn, mode };
    } else {
      SEL.invertMask(m);
      step = { op: 'invert' };
    }
    steps.push({ ...step, mask: rows(m, w, h), any: SEL.maskAny(m) });
  }
  const ex = SEL.extractRegion(idx, w, h, m, 0);
  mask_seq.push({
    w, h, indices: arr(idx), steps,
    bounds: SEL.maskBounds(m, w, h),
    extract: ex && { x: ex.x, y: ex.y, w: ex.w, h: ex.h, data: arr(ex.data), keep: arr(ex.keep) },
  });
}

// ── morph ─────────────────────────────────────────────────────────────────────
const morph = [];
for (let k = 0; k < 40; k++) {
  const w = range(1, 10), h = range(1, 10);
  const m = randMask(w, h);
  const out = [];
  for (const conn of [4, 8]) {
    const dist = arr(SEL.distanceTo(m, w, h, conn));
    for (let n = 0; n <= 3; n++) {
      out.push({
        conn, n,
        grow: rows(SEL.growMask(m, w, h, n, conn), w, h),
        shrink: rows(SEL.shrinkMask(m, w, h, n, conn), w, h),
        border: rows(SEL.borderMask(m, w, h, n, conn), w, h),
      });
    }
    out.push({ conn, distance: dist });
  }
  morph.push({ w, h, mask: rows(m, w, h), out });
}

// ── perm ──────────────────────────────────────────────────────────────────────
const perm = [];
for (let k = 0; k < 20; k++) {
  const w = range(1, 7), h = range(1, 7);
  const d = grid(w, h, 6);
  const o = (b) => ({ w: b.w, h: b.h, data: arr(b.data) });
  perm.push({
    w, h, data: arr(d),
    flip_h: arr(TR.flipH(d, w, h).data ?? TR.flipH(d, w, h)),
    flip_v: arr(TR.flipV(d, w, h).data ?? TR.flipV(d, w, h)),
    rot90cw: o(TR.rot90cw(d, w, h)), rot90ccw: o(TR.rot90ccw(d, w, h)), rot180: o(TR.rot180(d, w, h)),
    scale: [1, 2, 3, 4].map((n) => ({ n, ...o(TR.scaleNearest(d, w, h, n)) })),
  });
}

// ── rotated_size ──────────────────────────────────────────────────────────────
const DEGS = [0, 1, 15, 22.5, 30, 33, 37, 45, 60, 89, 90, 91, 120, 135, 180, 200, 225, 270, 300, 330, 359, 359.5, 360, -30, -90, 450];
const rotated_size = [];
for (let k = 0; k < 60; k++) {
  const w = range(1, 64), h = range(1, 64), deg = pick(DEGS);
  rotated_size.push({ w, h, deg, out: TR.rotatedSize(w, h, deg) });
}

// ── rotate ────────────────────────────────────────────────────────────────────
const rotate = [];
for (let k = 0; k < 40; k++) {
  const w = range(1, 12), h = range(1, 12), deg = pick(DEGS);
  const d = grid(w, h, 4);
  const n = TR.rotateNearest(d, w, h, deg, 0), r = TR.rotateSprite(d, w, h, deg, 0);
  rotate.push({
    w, h, deg, data: arr(d),
    nearest: { w: n.w, h: n.h, data: arr(n.data) },
    rotsprite: { w: r.w, h: r.h, data: arr(r.data) },
  });
}

// ── xform ─────────────────────────────────────────────────────────────────────
const xform = [];
for (let k = 0; k < 50; k++) {
  const w = range(1, 10), h = range(1, 10);
  const d = grid(w, h, 4);
  const lin = {
    rot: pick([0, 0, 90, 180, -90, 15, 30, 33, 45, -60, 120]),
    sx: pick([1, 1, 2, 0.5, 1.5, 3]),
    sy: pick([1, 1, 2, 0.5, 1.25]),
    skew: pick([0, 0, 0, 10, -20, 45]),
  };
  const t = AF.composeAffine(lin, range(-4, 8) + pick([0, 0, 0.5, 0.25]), range(-4, 8) + pick([0, 0, 0.5]));
  const clip = rnd() % 3 === 0 ? { x: 0, y: 0, w: range(1, 10), h: range(1, 10) } : null;
  const one = (method) => {
    const r = AF.transformIndices(d, w, h, t, { method, transparent: 0, clip });
    return r && { x: r.x, y: r.y, w: r.w, h: r.h, data: arr(r.data) };
  };
  xform.push({ w, h, data: arr(d), lin, t, clip, nearest: one('nearest'), rotsprite: one('rotsprite'), fields: AF.transformFields(t, w, h) });
}

// ── affine ────────────────────────────────────────────────────────────────────
const HANDLES = ['move', 'pivot', 'rot-ne', 'rot-sw', 'nw', 'ne', 'se', 'sw', 'n', 'e', 's', 'w', 'skew-n', 'skew-s', 'skew-e', 'skew-w'];
const affine = [];
for (let k = 0; k < 80; k++) {
  const w = range(1, 16), h = range(1, 16);
  const lin = {
    rot: pick([0, 0, 30, -45, 90, 135, 12.5]),
    sx: pick([1, 1, 2, 0.75]), sy: pick([1, 1, 1.5, 3]), skew: pick([0, 0, 15, -30]),
  };
  const t = AF.composeAffine(lin, range(-5, 10), range(-5, 10));
  const pivot = pick([{ u: w / 2, v: h / 2 }, { u: 0, v: 0 }, { u: w, v: h }, { u: Math.round(w / 3), v: 0.5 }]);
  const handle = pick(HANDLES);
  const from = { x: range(-5, 20) + 0.5, y: range(-5, 20) + 0.5 };
  const to = { x: from.x + range(-6, 6) + pick([0, 0.3, -0.4]), y: from.y + range(-6, 6) + pick([0, 0.2]) };
  const opts = { keepRatio: rnd() % 3 === 0, fromCenter: rnd() % 3 === 0, snap: rnd() % 3 === 0 };
  const patch = pick([{ w: range(1, 30) }, { h: range(1, 30) }, { rot: pick([0, 45, -170, 200]) }, { skew: pick([10, 95, -120]) }, { x: range(-5, 20), y: range(-5, 20) }, { w: range(1, 20), h: range(1, 20) }]);
  const keepRatio = rnd() % 2 === 0;
  affine.push({
    w, h, lin, t,
    decompose: AF.decomposeAffine(t),
    compose: AF.composeAffine(AF.decomposeAffine(t), t.e, t.f),
    settle: AF.settleAffine(t),
    pivot, handle, from, to, opts,
    drag: AF.dragTransform(t, pivot, w, h, handle, from, to, opts),
    patch, keep_ratio: keepRatio,
    with_fields: AF.withTransformFields(t, pivot, w, h, patch, { keepRatio }),
  });
}

// ── offset_wrap ───────────────────────────────────────────────────────────────
const offset_wrap = [];
for (let k = 0; k < 30; k++) {
  const w = range(1, 9), h = range(1, 9);
  const d = grid(w, h, 5);
  const dx = range(-20, 20), dy = range(-20, 20);
  offset_wrap.push({ w, h, dx, dy, data: arr(d), out: arr(FX.offsetWrap(d, w, h, dx, dy)) });
}

process.stdout.write(JSON.stringify({
  source: 'docs/Pixel Art/app/engine/{selection,transform,affine,distort,effects}.js — extrai_selecao_mvp.mjs',
  constants: {
    rotsprite_scale: TR.ROTSPRITE_SCALE, rot_snap_deg: AF.XFORM_ROT_SNAP_DEG, max_skew_deg: AF.XFORM_MAX_SKEW_DEG,
  },
  mask_seq, morph, perm, rotated_size, rotate, xform, affine, offset_wrap,
}) + '\n');
