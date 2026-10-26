"""Ajusta as leis da P4 ao que o Trial mostrou (trial_quadros.py) e imprime as sequencias tocadas.
uso: python3 -I trial_fit_quadros.py <dir>
  co_*  opacidade efectiva do cel: candidatas mul(L, C) e trunc(L*C/255)
  z_*   ordem dos cels: chave (indice + z) com desempate por z, por indice ou por -z
  on_*  (onion_<variante>.json) a tinta vermelha/azul: luma Rec.709, Rec.601 e a de inteiros
  tg_*  a sequencia de quadros tocados (indice da cor solida) e o tempo de cada um em ms
A lei de mistura Normal sobre destino opaco e a medida na P3: d + trunc((s - d) * mul(sa, op) / 255)."""
import json, os, sys

D = sys.argv[1]
E = json.load(open(os.path.join(D, 'entradas.json')))
L = json.load(open(os.path.join(D, 'lido.json')))
W, H = E['w'], E['h']

def mul(a, b):
    t = a * b + 0x80
    return ((t >> 8) + t) >> 8
def trunc_div(n, d):
    q = abs(n) // d
    return q if n >= 0 else -q
def normal(d, s, op):
    a = mul(s[3], op)
    return [d[k] + trunc_div((s[k] - d[k]) * a, 255) for k in range(3)]
def exact(got, want):
    return sum(g == w for g, w in zip(got, want))

print('== opacidade do cel x camada')
for l, c in E['co']:
    got = L[f'co_l{l:03d}_c{c:03d}.ase']
    res = {}
    for nome, op in (('mul', mul(l, c)), ('trunc', l * c // 255)):
        res[nome] = exact(got, [normal(b, s, op) for b, s in zip(E['back'], E['src']['A'])])
    print(f'  L={l} C={c}', res, '/', W * H)

print('== z-index')
REGRAS = {'z': lambda i, z: (i + z, z), 'indice': lambda i, z: (i + z, i), '-z': lambda i, z: (i + z, -z)}
for nome, zs in E['z'].items():
    got = L[f'z_{nome}.ase']
    res = {}
    for rn, key in REGRAS.items():
        # a de baixo (indice 0, z 0) entra na ordem; opaca, tapa tudo o que fica antes dela
        ordem = sorted(range(4), key=lambda k: key(k, 0 if k == 0 else zs[k - 1]))
        ordem = ordem[ordem.index(0) + 1:]
        want = []
        for p in range(W * H):
            d = E['back'][p][:3]
            for k in ordem:
                d = normal(d, E['src']['ABC'[k - 1]][p], 255)
            want.append(d)
        res[rn] = exact(got, want)
    print(f'  {nome:8s}', res, '/', W * H)

print('== papel vegetal: tinta')
LUMAS = {'709': lambda c: (c[0] * 2126 + c[1] * 7152 + c[2] * 722) // 10000,
         '601': lambda c: (c[0] * 299 + c[1] * 587 + c[2] * 114) // 1000,
         'int': lambda c: (c[0] * 54 + c[1] * 183 + c[2] * 19) >> 8}
QUARTO = {0: 0, 1: 1, 3: 2, 4: 3}
oni = os.path.join(D, 'onion_tint.json')
if os.path.exists(oni):
    o = json.load(open(oni))['on_vazio.ase']
    for ln, luma in LUMAS.items():
        want = []
        for p in range(W * H):
            x, y = p % W, p // W
            f = [k for k, q in QUARTO.items() if x // (W // 4) == q][0]
            dist = abs(f - 2)
            op = max(0, 68 - 28 * (dist - 1))
            c = E['onion'][str(f)]
            v = luma(c)
            t = [(255 + v) // 2, v // 2, v // 2] if f < 2 else [v // 2, v // 2, (255 + v) // 2]
            want.append(normal(o['off'][p], t + [255 if y < H // 2 else 128], op))
        print(f'  luma {ln}:', exact(o['on'], want), '/', W * H)

print('== tocar (indice da cor, ms)')
SOL = [tuple(c) for c in E['solidas']]
for n in sorted(k for k in L if k.startswith('tg_')):
    runs = L[n]['runs']
    seq = [(SOL.index(tuple(c)) if tuple(c) in SOL else '?', round(k * 20)) for c, k in runs]
    print(f'  {n:14s}', ' '.join(f'{i}:{ms}' for i, ms in seq))
