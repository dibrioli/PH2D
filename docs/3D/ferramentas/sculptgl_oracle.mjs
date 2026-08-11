#!/usr/bin/env node
// O ORACULO DO SCULPTGL -- o JS da referencia EXECUTANDO, nao uma leitura dele.
//
// Uso:
//   node docs/3D/ferramentas/sculptgl_oracle.mjs <dir-do-SculptGL> <arquivo-de-saida>
//
// O que ele faz, e por que assim:
//
// Ele NAO reimplementa nenhum kernel. Ele ABRE os arquivos de
// `src/editing/tools/` e EXTRAI o texto do corpo de cada metodo por
// casamento de chaves, monta uma `Function` com aquele texto e a chama contra
// um `this` de mentira. O que roda e' o codigo que o SculptGL shipa, byte a
// byte -- e e' essa propriedade que separa "eu transliterei certo" de "a
// referencia produziu este numero".
//
// A alternativa (transcrever os kernels a mao para dentro deste arquivo) tem o
// mesmo modo de falha do gate que espelha o produto em vez de o interrogar: ela
// so' pode confirmar a minha leitura.
//
// A saida e' um arquivo de texto com os bits EXATOS (hex) de cada f32 de
// entrada e de saida. Nada e' re-derivado do outro lado: o Rust le' as mesmas
// posicoes, as mesmas normais, a mesma lista de vertices e compara os bits do
// resultado. Um formato decimal deixaria a igualdade depender do parser.

import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const SRC = process.argv[2];
const OUT = process.argv[3];
if (!SRC || !OUT) {
  console.error('uso: sculptgl_oracle.mjs <dir-do-SculptGL> <saida>');
  process.exit(2);
}

// ---------------------------------------------------------------------------
// A EXTRACAO -- o corpo de um metodo, tirado do arquivo que shipa.
// ---------------------------------------------------------------------------

function readTool(name) {
  return readFileSync(join(SRC, 'src/editing/tools', name + '.js'), 'utf8');
}

/// Devolve `{ args, body }` do metodo `name` declarado com dois espacos de
/// indentacao (a forma que todas as classes de tool usam).
function extractMethod(src, name) {
  const re = new RegExp('^  ' + name + '\\(([^)]*)\\) \\{$', 'm');
  const m = re.exec(src);
  if (!m) throw new Error('metodo nao encontrado: ' + name);
  const args = m[1].split(',').map((s) => s.trim()).filter(Boolean);
  // Casamento de chaves a partir da abertura. As tools nao tem string nem
  // regex com chave desbalanceada nos corpos que nos interessam -- e se um dia
  // tiverem, isto quebra ALTO (o `Function` nao compila) em vez de extrair
  // metade de um metodo em silencio.
  let i = m.index + m[0].length - 1;
  let depth = 0;
  let start = i + 1;
  for (; i < src.length; i++) {
    if (src[i] === '{') depth++;
    else if (src[i] === '}') {
      depth--;
      if (depth === 0) break;
    }
  }
  if (depth !== 0) throw new Error('chaves desbalanceadas em ' + name);
  return { args, body: src.slice(start, i) };
}

function compile(src, name) {
  const { args, body } = extractMethod(src, name);
  // eslint-disable-next-line no-new-func
  return new Function(...args, body);
}

// ---------------------------------------------------------------------------
// OS STUBS -- o minimo que os corpos tocam.
// ---------------------------------------------------------------------------

// `picking.getAlpha` devolve 1.0 sem textura de alpha (Picking.js:60-62), que
// e' o `_idAlpha = 0` de fabrica das dez tools.
const picking = { getAlpha: () => 1.0 };

function makeSelf(mesh, opts) {
  return {
    getMesh: () => mesh,
    _negative: !!opts.negative,
    _accumulate: opts.accumulate !== false,
    _lockPosition: !!opts.lockPosition,
    _clay: opts.clay !== false,
    _culling: !!opts.culling,
    _intensity: opts.intensity,
    _radius: opts.radius
  };
}

// ---------------------------------------------------------------------------
// A FIXTURE -- uma esfera UV, gerada aqui e DESPEJADA (o Rust nunca a re-deriva).
// ---------------------------------------------------------------------------

function uvSphere(rings, segs, radius) {
  const pos = [];
  const nrm = [];
  for (let i = 0; i <= rings; i++) {
    const v = i / rings;
    const phi = v * Math.PI;
    for (let j = 0; j < segs; j++) {
      const u = j / segs;
      const th = u * Math.PI * 2;
      const x = Math.sin(phi) * Math.cos(th);
      const y = Math.cos(phi);
      const z = Math.sin(phi) * Math.sin(th);
      // ⚠️ **As normais do SculptGL NAO sao unitarias, e a fixture tem de
      // conter isso.** O `Mesh.updateVerticesNormal` guarda a MEDIA das normais
      // das faces do anel (`nAr = (sum faceNormals) / count`) -- sem
      // normalizar. Numa superficie lisa o comprimento chega perto de 1; numa
      // quina ele despenca. E' exatamente por isso que o `Inflate` divide pelo
      // comprimento da normal na hora de usar (`Inflate.js:66`), e uma fixture
      // com normais unitarias analiticas faz dessa divisao um no-op -- ela
      // aprovaria um kernel que a apagou.
      //
      // O fator varia por vertice para o divisor nao poder ser dobrado numa
      // constante.
      const k = 0.62 + 0.38 * Math.abs(Math.sin(i * 2.3 + j * 1.7));
      nrm.push(x * k, y * k, z * k);
      pos.push(x * radius, y * radius, z * radius);
    }
  }
  return {
    pos: new Float32Array(pos),
    nrm: new Float32Array(nrm),
    count: pos.length / 3
  };
}

function makeMesh(fx) {
  // `mAr` e' o array de MATERIAIS do SculptGL: `[roughness, metalness, mask]`
  // por vertice, e os kernels leem `mAr[ind + 2]`, a MASCARA.
  const mats = new Float32Array(fx.count * 3);
  for (let i = 0; i < fx.count; i++) {
    mats[i * 3] = 0;
    mats[i * 3 + 1] = 0;
    mats[i * 3 + 2] = fx.mask[i];
  }
  return {
    getVertices: () => fx.pos,
    getNormals: () => fx.nrm,
    getMaterials: () => mats,
    getVerticesProxy: () => fx.proxy,
    isDynamic: false,
    updateGeometry: () => {},
    getFacesFromVertices: () => []
  };
}

// ---------------------------------------------------------------------------
// OS CASOS
// ---------------------------------------------------------------------------

const Brush = readTool('Brush');
const Flatten = readTool('Flatten');
const Inflate = readTool('Inflate');
const Crease = readTool('Crease');
const Pinch = readTool('Pinch');
const Drag = readTool('Drag');
const Move = readTool('Move');
const LocalScale = readTool('LocalScale');

const kernels = {
  brush: compile(Brush, 'brush'),
  flatten: compile(Flatten, 'flatten'),
  inflate: compile(Inflate, 'inflate'),
  crease: compile(Crease, 'crease'),
  pinch: compile(Pinch, 'pinch'),
  drag: compile(Drag, 'drag'),
  move: compile(Move, 'move'),
  scale: compile(LocalScale, 'scale')
};

const RINGS = 64;
const SEGS = 128;
const SPHERE_R = 1.0;

/// Constroi a fixture do zero para cada caso -- um caso NUNCA ve o resultado do
/// anterior, senao a ordem dos casos viraria parte do oraculo.
function freshFixture() {
  const s = uvSphere(RINGS, SEGS, SPHERE_R);
  const mask = new Float32Array(s.count);
  for (let i = 0; i < s.count; i++) {
    // Tres regimes na mesma fixture: livre, meio-mascarado e travado.
    //
    // ⚠️ **A polaridade e a da REFERENCIA: `1` e LIVRE, `0` e TRAVADO** -- o
    // oposto da nossa (`DEFAULT_MASK = 0` significa totalmente esculpivel). A
    // maioria tem de ser LIVRE, senao a fixture exercita o caminho mascarado e
    // quase nao exercita o normal -- e a `areaNormal`, que pesa por esta mesma
    // mascara, sairia decidida por uma minoria.
    mask[i] = i % 7 === 0 ? 0.0 : i % 7 === 3 ? 0.5 : 1.0;
  }
  return { pos: s.pos, nrm: s.nrm, count: s.count, mask, proxy: s.pos.slice() };
}

function selectSphere(fx, center, radius) {
  const sel = [];
  const r2 = radius * radius;
  for (let i = 0; i < fx.count; i++) {
    const dx = fx.pos[i * 3] - center[0];
    const dy = fx.pos[i * 3 + 1] - center[1];
    const dz = fx.pos[i * 3 + 2] - center[2];
    if (dx * dx + dy * dy + dz * dz < r2) sel.push(i);
  }
  return new Uint32Array(sel);
}

/// Uma permutacao deterministica -- o embaralhador de Fisher-Yates com um LCG.
///
/// ⚠️ **Ela existe por um buraco MEDIDO, nao por capricho.** O proxy do `move`
/// e' EMPACOTADO: ele e' indexado pela posicao na lista (`j = i * 3`), nao pelo
/// id do vertice (`ind = iVerts[i] * 3`). Com a selecao sendo a identidade
/// (`sel = [0..n)`) os dois indices COINCIDEM, e a mutacao que troca um pelo
/// outro passa verde -- foi o que aconteceu. Embaralhada, `i !== v` para quase
/// todo vertice e a distincao vira observavel.
function shuffled(arr, seed) {
  const a = Array.from(arr);
  let s = BigInt(seed);
  const M = (1n << 64n) - 1n;
  for (let i = a.length - 1; i > 0; i--) {
    s = (s * 6364136223846793005n + 1442695040888963407n) & M;
    const j = Number((s >> 33n) % BigInt(i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return new Uint32Array(a);
}

// O centro fica sobre a superficie, no equador -- e' onde um pick de verdade
// pousa, e a normal ali nao e' um eixo do mundo (o polo esconderia erro de eixo).
const CENTER = [
  Math.sin(Math.PI * 0.5) * Math.cos(0.7),
  Math.cos(Math.PI * 0.5),
  Math.sin(Math.PI * 0.5) * Math.sin(0.7)
];
const RADIUS = 0.45;
const R2 = RADIUS * RADIUS;

/// A normal e o centro de AREA, do jeito que o `SculptBase` os calcula
/// (`areaNormal`/`areaCenter`) -- extraidos tambem, nao reescritos.
const areaNormal = compile(readTool('SculptBase'), 'areaNormal');
const areaCenter = compile(readTool('SculptBase'), 'areaCenter');
const getFrontVertices = compile(readTool('SculptBase'), 'getFrontVertices');

// `getFrontVertices` usa `Utils.getMemory`; o corpo o referencia por nome
// livre, entao ele tem de existir no escopo da `Function`. Um `Function` so'
// enxerga o escopo GLOBAL, e e' por isso que o stub mora ali.
globalThis.Utils = {
  getMemory: (n) => new ArrayBuffer(n)
};

const EYE = (() => {
  // Do olho para a superficie: a camera olha para a origem a partir de fora,
  // entao o raio que acerta o equador aponta para DENTRO.
  const l = Math.hypot(CENTER[0], CENTER[1], CENTER[2]);
  return [-CENTER[0] / l, -CENTER[1] / l, -CENTER[2] / l];
})();

const cases = [];

/// A fixture COMPACTA: so' os vertices da pegada, re-indexados de 0 a n.
///
/// ⚠️ **Isto e' EXATO, nao uma amostragem.** Todo kernel indexa a malha
/// *atraves* de `iVerts` -- nenhum deles varre `vAr` inteiro, nenhum deles le'
/// um vizinho que nao esteja na lista (o unico que leria e' o `smooth`, pelo
/// anel, e ele nao esta' aqui). Entao a malha compacta com `iVerts = [0..n)`
/// produz os MESMOS bits que a esfera inteira, e o arquivo cabe no repo em vez
/// de despejar 4704 vertices dos quais 99% nao sao tocados.
function compact(fx, sel) {
  const n = sel.length;
  const pos = new Float32Array(n * 3);
  const nrm = new Float32Array(n * 3);
  const mask = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const s = sel[i] * 3;
    pos[i * 3] = fx.pos[s];
    pos[i * 3 + 1] = fx.pos[s + 1];
    pos[i * 3 + 2] = fx.pos[s + 2];
    nrm[i * 3] = fx.nrm[s];
    nrm[i * 3 + 1] = fx.nrm[s + 1];
    nrm[i * 3 + 2] = fx.nrm[s + 2];
    mask[i] = fx.mask[sel[i]];
  }
  const out = { pos, nrm, mask, count: n, proxy: pos.slice() };
  return out;
}

/// `over` deixa um caso escolher OUTRO centro/olho -- e existe por um buraco
/// medido, nao por generalidade: com o centro sobre a superficie e o olho
/// apontando direto para ele, TODO vertice da pegada e frontal, e o
/// `getFrontVertices` nunca e' exercitado. O `eye` so' e' observavel onde a
/// pegada atravessa a TERMINADORA -- a mesma frase que o doc do nosso
/// `Dab::eye` ja' carregava, e que esta fixture nao continha.
function record(name, params, run, over = {}) {
  const center = over.center || CENTER;
  const eye = over.eye || EYE;
  const full = freshFixture();
  // ⚠️ A malha compacta leva uma MARGEM (ate' 1,35x o raio), entao a selecao e'
  // um subconjunto PROPRIO dela: quem esta' fora tem de sair byte-identico, e um
  // kernel que escrevesse alem da lista seria pego pela comparacao.
  const fx = compact(full, selectSphere(full, center, RADIUS * 1.35));
  const mesh = makeMesh(fx);
  const sel = shuffled(selectSphere(fx, center, RADIUS), 0x5c01);
  const inPos = fx.pos.slice();
  run(fx, mesh, sel, center, eye);
  cases.push({
    name, params, fx, sel, inPos, outPos: fx.pos.slice(),
    over: over.center || over.eye ? { center, eye } : null
  });
}

const INTENSITY = 0.5;

// --- Brush com Clay DESLIGADO: o `brush()` puro (empurra pela normal de area).
record('brush', { intensity: INTENSITY, negative: false }, (fx, mesh, sel) => {
  const self = makeSelf(mesh, { intensity: INTENSITY, negative: false, clay: false });
  const front = getFrontVertices.call(self, sel, EYE);
  const aNormal = areaNormal.call(self, front);
  kernels.brush.call(self, sel, aNormal, CENTER, R2, INTENSITY, picking);
});

// --- O MESMO Brush, mas com a pegada ATRAVESSANDO A TERMINADORA.
//
// ⚠️ E' o unico caso em que `getFrontVertices` filtra alguma coisa. Nos outros
// o olho aponta direto para o centro da pegada, todo vertice e' frontal, e o
// filtro devolve a lista inteira -- um gate que so' olhasse para eles
// aprovaria um `front_vertices` que devolve tudo. O centro fica sobre o
// equador visto DE LADO: metade da pegada olha para a camera, metade some.
const TERM_CENTER = [Math.cos(0.7), 0.0, Math.sin(0.7)];
const TERM_EYE = (() => {
  // A camera olha na direcao +X a partir de longe: o raio aponta para +X, e o
  // ponto do equador em `x = cos(0.7)` fica exatamente na silhueta.
  const t = [Math.sin(0.7), 0.0, -Math.cos(0.7)];
  return t;
})();
record(
  'brush_terminator',
  { intensity: INTENSITY, negative: false },
  (fx, mesh, sel, center, eye) => {
    const self = makeSelf(mesh, { intensity: INTENSITY, negative: false, clay: false });
    const front = getFrontVertices.call(self, sel, eye);
    if (front.length === sel.length || front.length === 0) {
      throw new Error(
        'o caso da terminadora nao filtra nada (' + front.length + ' de ' + sel.length + ')'
      );
    }
    const aNormal = areaNormal.call(self, front);
    kernels.brush.call(self, sel, aNormal, center, R2, INTENSITY, picking);
  },
  { center: TERM_CENTER, eye: TERM_EYE }
);

// --- Brush com Clay LIGADO: o default de fabrica (`Brush.js:12`). Ele NAO
// chama `brush()` -- ele achata contra um plano deslocado por `0.1 * raio`.
record('clay', { intensity: INTENSITY, negative: false }, (fx, mesh, sel) => {
  const self = makeSelf(mesh, { intensity: INTENSITY, negative: false, clay: true });
  const front = getFrontVertices.call(self, sel, EYE);
  const aNormal = areaNormal.call(self, front);
  const aCenter = areaCenter.call(self, front);
  const off = Math.sqrt(R2) * 0.1;
  aCenter[0] += aNormal[0] * off;
  aCenter[1] += aNormal[1] * off;
  aCenter[2] += aNormal[2] * off;
  kernels.flatten.call(self, sel, aNormal, aCenter, CENTER, R2, INTENSITY, picking);
});

// --- Flatten puro: `_negative = true` de fabrica (`Flatten.js:12`).
record('flatten', { intensity: 0.75, negative: true }, (fx, mesh, sel) => {
  const self = makeSelf(mesh, { intensity: 0.75, negative: true });
  const front = getFrontVertices.call(self, sel, EYE);
  const aNormal = areaNormal.call(self, front);
  const aCenter = areaCenter.call(self, front);
  kernels.flatten.call(self, sel, aNormal, aCenter, CENTER, R2, 0.75, picking);
});

// --- Inflate: empurra cada vertice pela PROPRIA normal.
record('inflate', { intensity: 0.3, negative: false }, (fx, mesh, sel) => {
  const self = makeSelf(mesh, { intensity: 0.3, negative: false });
  kernels.inflate.call(self, sel, CENTER, R2, 0.3, picking);
});

// --- Crease: pinch + brush, com o `pow(fallOff, 5)` no termo normal.
record('crease', { intensity: 0.75, negative: true }, (fx, mesh, sel) => {
  const self = makeSelf(mesh, { intensity: 0.75, negative: true });
  const front = getFrontVertices.call(self, sel, EYE);
  const aNormal = areaNormal.call(self, front);
  kernels.crease.call(self, sel, aNormal, CENTER, R2, 0.75, picking);
});

// --- Pinch: junta em volta do ponto de intersecao, em 3D (sem projetar).
record('pinch', { intensity: 0.75, negative: false }, (fx, mesh, sel) => {
  const self = makeSelf(mesh, { intensity: 0.75, negative: false });
  kernels.pinch.call(self, sel, CENTER, R2, 0.75, picking);
});

// --- Drag (o Snake Hook): soma um deslocamento sobre a posicao VIVA.
const DRAG_DIR = [0.08, 0.03, -0.02];
record('drag', { dir: DRAG_DIR }, (fx, mesh, sel) => {
  const self = makeSelf(mesh, {});
  self._dragDir = DRAG_DIR;
  self._dragDirSym = DRAG_DIR;
  kernels.drag.call(self, sel, CENTER, R2, false, picking);
});

// --- Move (o Grab): le' de um proxy congelado e soma `dir * fallOff`.
const MOVE_DIR = [0.12, -0.05, 0.04];
record('move', { dir: MOVE_DIR }, (fx, mesh, sel) => {
  const self = makeSelf(mesh, {});
  const vProxy = new Float32Array(sel.length * 3);
  for (let i = 0; i < sel.length; i++) {
    const ind = sel[i] * 3;
    vProxy[i * 3] = fx.pos[ind];
    vProxy[i * 3 + 1] = fx.pos[ind + 1];
    vProxy[i * 3 + 2] = fx.pos[ind + 2];
  }
  kernels.move.call(self, sel, CENTER, R2, { vProxy, dir: MOVE_DIR }, picking);
});

// --- LocalScale: afasta/aproxima da ancora. O `delta` e' em PIXELS de gesto.
const SCALE_DELTA = 20.0;
record('local_scale', { delta: SCALE_DELTA }, (fx, mesh, sel) => {
  const self = makeSelf(mesh, {});
  kernels.scale.call(self, sel, CENTER, R2, SCALE_DELTA, picking);
});

// ---------------------------------------------------------------------------
// O DESPEJO -- bits, nunca decimais.
// ---------------------------------------------------------------------------

const f32buf = new DataView(new ArrayBuffer(4));
function f32bits(x) {
  f32buf.setFloat32(0, x);
  return f32buf.getUint32(0).toString(16).padStart(8, '0');
}
const f64buf = new DataView(new ArrayBuffer(8));
function f64bits(x) {
  f64buf.setFloat64(0, x);
  return f64buf.getBigUint64(0).toString(16).padStart(16, '0');
}

const lines = [];
lines.push('# sculptgl-oracle v1');
lines.push('# gerado por docs/3D/ferramentas/sculptgl_oracle.mjs -- NAO EDITE A MAO');
lines.push('# cada f32 e um u32 hex (bits), cada f64 e um u64 hex (bits).');
lines.push('sphere ' + RINGS + ' ' + SEGS + ' ' + f64bits(SPHERE_R));
lines.push('center ' + CENTER.map(f64bits).join(' '));
lines.push('radius2 ' + f64bits(R2));
lines.push('eye ' + EYE.map(f64bits).join(' '));

for (const c of cases) {
  lines.push('case ' + c.name);
  for (const [k, v] of Object.entries(c.params)) {
    if (Array.isArray(v)) lines.push('param ' + k + ' ' + v.map(f64bits).join(' '));
    else if (typeof v === 'number') lines.push('param ' + k + ' ' + f64bits(v));
    else lines.push('param ' + k + ' ' + (v ? 1 : 0));
  }
  if (c.over) {
    lines.push('param center ' + c.over.center.map(f64bits).join(' '));
    lines.push('param eye ' + c.over.eye.map(f64bits).join(' '));
  }
  lines.push('verts ' + c.fx.count);
  lines.push('in.pos ' + Array.from(c.inPos, f32bits).join(' '));
  lines.push('in.nrm ' + Array.from(c.fx.nrm, f32bits).join(' '));
  lines.push('in.mask ' + Array.from(c.fx.mask, f32bits).join(' '));
  lines.push('sel ' + Array.from(c.sel).join(' '));
  lines.push('out.pos ' + Array.from(c.outPos, f32bits).join(' '));
  const moved = [];
  for (let i = 0; i < c.inPos.length; i++) if (c.inPos[i] !== c.outPos[i]) moved.push(i);
  lines.push('# ' + c.name + ': ' + c.sel.length + ' selecionados, ' + moved.length + ' componentes mexidos');
  if (moved.length === 0) {
    // Um caso que nao move nada e' um oraculo que nao pode falhar: ele
    // aprovaria um kernel deletado. Falha ALTO em vez de despejar um zero.
    throw new Error('caso ' + c.name + ' nao moveu nenhum componente -- fixture nao contem o fenomeno');
  }
}

writeFileSync(OUT, lines.join('\n') + '\n');
console.log('escrito: ' + OUT);
for (const c of cases) {
  let maxd = 0;
  for (let i = 0; i < c.inPos.length; i++) maxd = Math.max(maxd, Math.abs(c.inPos[i] - c.outPos[i]));
  console.log(
    '  ' + c.name.padEnd(12) + ' sel=' + String(c.sel.length).padStart(4) +
    '  maior deslocamento por componente = ' + maxd.toFixed(6)
  );
}
