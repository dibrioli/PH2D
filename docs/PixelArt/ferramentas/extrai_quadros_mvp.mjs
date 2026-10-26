// Extrai os vetores de QUADROS, TOCAR e TAGS do MVP web (docs/Pixel Art/app, nao versionado; mora
// no primario) para a fixtura de paridade da onda P4 do ph2d-pixel-core. O caminho do MVP e o 1.o
// argumento:
//   node docs/PixelArt/ferramentas/extrai_quadros_mvp.mjs \
//     "/home/enio/Documentos/Projetos/PH2D/docs/Pixel Art/app" \
//     > crates/ph2d-pixel-core/tests/fixtures/mvp_frames.json
// Familias (de frames.test.js, playback.test.js, tags.test.js e das funcoes que eles exercitam):
//   frame_ops — sequencias frame.insert (em branco/duplicado)/delete/move/setDuration com a ordem,
//               as duracoes e a marca do cel depois de cada passo, de cada undo e de cada redo;
//   frame_limits — apagar o ultimo quadro recusa; o id nunca recicla;
//   playback  — frameAtTime sobre duracoes sorteadas (com e sem loop, t negativo) e o total;
//   play_order — tagPlayOrder de cada direcao sobre intervalos de 1..6 quadros;
//   tag_ops   — tags depois de frame.insert/delete/move e o undo (crescer, encolher, sumir, voltar).
// O MVP nao tem cel ligado, opacidade/z-index de cel nem repeticao no tocar: o oraculo deles e o Trial.
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const app = process.argv[2];
if (!app) throw new Error('uso: extrai_quadros_mvp.mjs <caminho de docs/Pixel Art/app>');
const mod = (f) => import(pathToFileURL(path.join(app, 'engine', f)).href);
const DOC = await mod('doc.js');
const OPS = await mod('ops.js');
const PB = await mod('playback.js');
const TG = await mod('tags.js');

const err = (fn) => { try { fn(); return null; } catch (e) { return String(e.message); } };

// splitmix32, o mesmo de extrai_camadas_mvp.mjs (outra semente).
let seed = 0x51ed2701 >>> 0;
const rnd = () => {
  seed = (seed + 0x9e3779b9) >>> 0;
  let z = seed;
  z = Math.imul(z ^ (z >>> 16), 0x85ebca6b) >>> 0;
  z = Math.imul(z ^ (z >>> 13), 0xc2b2ae35) >>> 0;
  return (z ^ (z >>> 16)) >>> 0;
};
const pick = (a) => a[rnd() % a.length];

// ── frame_ops ─────────────────────────────────────────────────────────────────
// Cada quadro leva uma marca na celula (0,0) da camada 1 (= o id do quadro), para se ver que os
// pixels seguem o ID e que o duplicado os copia.
const snap = (doc) => doc.frames.map((f) => ({
  id: f.id, ms: f.durationMs, cell00: DOC.getCell(doc, 1, f.id, 0, 0),
}));
const frame_ops = [];
for (let k = 0; k < 40; k++) {
  const doc = DOC.createDoc({ w: 2, h: 2 });
  const hist = OPS.createHistory();
  DOC.setCell(doc, 1, 1, 0, 0, 1);
  const steps = [];
  const n = 3 + (rnd() % 8);
  for (let s = 0; s < n; s++) {
    const len = doc.frames.length;
    const r = rnd() % 10;
    let op, step;
    if (r < 4 || len < 2) {
      const id = doc.meta.nextId; doc.meta.nextId = id + 1;
      const from = pick(doc.frames);
      const dup = rnd() % 2 === 0;
      const cels = {};
      const src = DOC.getCel(doc, 1, from.id);
      if (dup && src) cels[1] = new Uint8Array(src.data);
      const index = rnd() % (len + 1);
      op = { type: 'frame.insert', index, frame: { id, durationMs: pick([40, 100, 100, 250]) }, cels };
      step = { op: 'insert', index, id, ms: op.frame.durationMs, copy_of: dup && src ? from.id : null };
    } else if (r < 6) {
      const id = pick(doc.frames).id;
      op = { type: 'frame.delete', id };
      step = { op: 'delete', id };
    } else if (r < 8) {
      const from = rnd() % len, to = rnd() % len;
      op = { type: 'frame.move', from, to };
      step = { op: 'move', from, to };
    } else {
      const id = pick(doc.frames).id;
      const ms = pick([1, 16, 33, 100, 500, 65535]);
      op = { type: 'frame.setDuration', id, ms };
      step = { op: 'duration', id, ms };
    }
    OPS.commit(doc, hist, op);
    step.after = snap(doc);
    steps.push(step);
  }
  const undos = [];
  while (OPS.canUndo(hist)) { OPS.undo(doc, hist); undos.push(snap(doc)); }
  const redos = [];
  while (OPS.canRedo(hist)) { OPS.redo(doc, hist); redos.push(snap(doc)); }
  frame_ops.push({ steps, undos, redos });
}

// ── frame_limits ──────────────────────────────────────────────────────────────
const frame_limits = {
  delete_last: err(() => OPS.applyOp(DOC.createDoc({ w: 1, h: 1 }), { type: 'frame.delete', id: 1 })),
  default_ms: DOC.createDoc({ w: 1, h: 1 }).frames[0].durationMs,
};

// ── playback ──────────────────────────────────────────────────────────────────
const playback = [];
for (let k = 0; k < 60; k++) {
  const n = 1 + (rnd() % 6);
  const ms = Array.from({ length: n }, () => pick([0, 1, 16, 50, 100, 100, 250]));
  const frames = ms.map((d, i) => ({ id: i + 1, durationMs: d }));
  const total = PB.totalDurationMs(frames);
  const times = [0, -1, -total - 3, total, total - 1, total * 3 + 7];
  for (let i = 0; i < 8; i++) times.push(rnd() % (total * 2 + 1));
  playback.push({
    ms, total,
    loop: times.map((t) => [t, PB.frameAtTime(frames, t, true)]),
    once: times.filter((t) => t >= 0).map((t) => [t, PB.frameAtTime(frames, t, false)]),
  });
}

// ── play_order ────────────────────────────────────────────────────────────────
const play_order = [];
for (const direction of TG.DIRECTIONS) {
  for (let from = 0; from < 3; from++) {
    for (let len = 1; len <= 6; len++) {
      const tag = TG.normalizeTag({ id: 1, from, to: from + len - 1, direction });
      play_order.push({ direction, from, to: tag.to, order: TG.tagPlayOrder(tag) });
    }
  }
}

// ── tag_ops ───────────────────────────────────────────────────────────────────
const tagsOf = (doc) => doc.tags.map((t) => ({ id: t.id, from: t.from, to: t.to }));
const tag_ops = [];
for (let k = 0; k < 30; k++) {
  const doc = DOC.createDoc({ w: 1, h: 1 });
  const nf = 3 + (rnd() % 6);
  for (let i = 1; i < nf; i++) {
    OPS.applyOp(doc, { type: 'frame.insert', index: i, frame: { id: 100 + i, durationMs: 100 }, cels: {} });
  }
  const ntag = 1 + (rnd() % 3);
  for (let i = 0; i < ntag; i++) {
    const from = rnd() % nf;
    const to = from + (rnd() % (nf - from));
    OPS.applyOp(doc, { type: 'tag.insert', index: i, tag: { id: 500 + i, name: `t${i}`, from, to } });
  }
  const tags0 = tagsOf(doc);
  const hist = OPS.createHistory();
  const steps = [];
  for (let s = 0; s < 4; s++) {
    const len = doc.frames.length;
    const r = rnd() % 3;
    let op, step;
    if (r === 0 || len < 2) {
      const index = rnd() % (len + 1);
      op = { type: 'frame.insert', index, frame: { id: 900 + k * 10 + s, durationMs: 100 }, cels: {} };
      step = { op: 'insert', index };
    } else if (r === 1) {
      const index = rnd() % len;
      op = { type: 'frame.delete', id: doc.frames[index].id };
      step = { op: 'delete', index };
    } else {
      const from = rnd() % len, to = rnd() % len;
      op = { type: 'frame.move', from, to };
      step = { op: 'move', from, to };
    }
    OPS.commit(doc, hist, op);
    step.after = tagsOf(doc);
    steps.push(step);
  }
  const undos = [];
  while (OPS.canUndo(hist)) { OPS.undo(doc, hist); undos.push(tagsOf(doc)); }
  tag_ops.push({ frames: nf, tags: tags0, steps, undos });
}

process.stdout.write(JSON.stringify({
  source: 'docs/Pixel Art/app (MVP web), extrai_quadros_mvp.mjs',
  note: 'MVP: frame.move troca so a ordem (cels por id); a fronteira de tempo pertence ao quadro seguinte; duracao com piso de 1 ms; ping-pong sem repetir pontas. Sem cel ligado, opacidade/z-index de cel nem repeticao.',
  frame_ops, frame_limits, playback, play_order, tag_ops,
}) + '\n');
