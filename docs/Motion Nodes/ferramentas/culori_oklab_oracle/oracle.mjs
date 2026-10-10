// O ORÁCULO da cor (ciclo 18) — corre o culori (MIT) sobre as NOSSAS cores (RGB LINEAR, a coluna
// `tint` da casa) e imprime: a conversão para OKLab e OKLCH, e a mistura a t = k/8 em quatro
// espaços (RGB linear · OKLab · OKLCH matiz curta · OKLCH matiz longa), de volta em RGB linear.
// Recebe o caminho do `culori` (pasta do pacote).
const c = await import(process.argv[2] + '/bundled/culori.mjs');
const f = (x) => (Number.isFinite(x) ? x : 0).toFixed(7);
const cores = [
  ['vermelho', [1, 0, 0]], ['verde', [0, 1, 0]], ['azul', [0, 0, 1]], ['amarelo', [1, 1, 0]],
  ['ciano', [0, 1, 1]], ['magenta', [1, 0, 1]], ['branco', [1, 1, 1]], ['preto', [0, 0, 0]],
  ['laranja', [1, 0.25, 0.02]], ['cinza', [0.18, 0.18, 0.18]], ['roxo', [0.2, 0.02, 0.6]],
];
const lrgb = (v) => ({ mode: 'lrgb', r: v[0], g: v[1], b: v[2] });
for (const [n, v] of cores) {
  const o = c.oklab(lrgb(v));
  const h = c.oklch(lrgb(v));
  console.log(`CONV ${n} | ${f(o.l)} ${f(o.a)} ${f(o.b)} | ${f(h.l)} ${f(h.c)} ${f(h.h ?? 0)}`);
}
const pares = [['vermelho', 'azul'], ['azul', 'amarelo'], ['verde', 'magenta'], ['preto', 'branco'],
  ['laranja', 'roxo'], ['ciano', 'vermelho'], ['cinza', 'laranja']];
const por = Object.fromEntries(cores);
const espacos = [
  ['lrgb', (a, b) => c.interpolate([a, b], 'lrgb')],
  ['oklab', (a, b) => c.interpolate([a, b], 'oklab')],
  ['oklch_curta', (a, b) => c.interpolate([a, b], 'oklch', { h: { fixup: c.fixupHueShorter } })],
  ['oklch_longa', (a, b) => c.interpolate([a, b], 'oklch', { h: { fixup: c.fixupHueLonger } })],
];
for (const [a, b] of pares) {
  for (const [e, mk] of espacos) {
    const fn = mk(lrgb(por[a]), lrgb(por[b]));
    const vals = [];
    for (let k = 0; k <= 8; k++) {
      const r = c.converter('lrgb')(fn(k / 8));
      vals.push(`${f(r.r)},${f(r.g)},${f(r.b)}`);
    }
    console.log(`MIX ${a}-${b} ${e} | ${vals.join(' ')}`);
    if (e !== 'lrgb') {
      // O mapeamento de gama do CSS Color 4 (culori `toGamut`, os defaults: OKLCH, JND 0,02).
      const gama = c.toGamut('rgb', 'oklch');
      const mapped = [];
      for (let k = 0; k <= 8; k++) {
        const r = c.converter('lrgb')(gama(fn(k / 8)));
        mapped.push(`${f(r.r)},${f(r.g)},${f(r.b)}`);
      }
      console.log(`GAMUT ${a}-${b} ${e} | ${mapped.join(' ')}`);
    }
  }
}
