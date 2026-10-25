"""Ajusta a lei de cada modo de mistura ao que o Trial mostrou (lido.json de trial_camadas.py).
uso: python3 -I trial_fit_camadas.py <dir>
Candidatas: as formulas publicas (W3C Compositing, PDF 1.7 §11.3.5) em inteiros u8, com as
variantes de arredondamento; a composicao final e o source-over com o alfa da fonte x opacidade.
Imprime, por ficheiro, quantas celulas cada candidata acerta EXACTAMENTE."""
import json, os, sys
import numpy as np

D = sys.argv[1]
E = json.load(open(os.path.join(D, 'entradas.json')))
L = json.load(open(os.path.join(D, 'lido.json')))
B = np.array(E['back'], dtype=np.int64)[:, :3]
S = np.array(E['src'], dtype=np.int64)
M = np.array(E['mid'], dtype=np.int64)

def mul(a, b):  # a*b/255 arredondado (o MUL_UN8 classico)
    t = a * b + 0x80
    return ((t >> 8) + t) >> 8
def tdiv(n, d):  # divisao C (trunca para zero)
    q = np.abs(n) // np.maximum(d, 1)
    return np.where(n < 0, -q, q)

# ── separaveis: f(b, s) -> u8 ─────────────────────────────────────────────────
def screen(b, s): return b + s - mul(b, s)
def hard(b, s):
    s2 = s * 2
    return np.where(s < 128, mul(b, s2), screen(b, s2 - 255))
def soft_f(b, s):
    bf, sf = b / 255.0, s / 255.0
    d = np.where(bf <= 0.25, ((16 * bf - 12) * bf + 4) * bf, np.sqrt(bf))
    r = np.where(sf <= 0.5, bf - (1 - 2 * sf) * bf * (1 - bf), bf + (2 * sf - 1) * (d - bf))
    return r
SEP = {
    1: {'mul': lambda b, s: mul(b, s), 'trunc': lambda b, s: b * s // 255},
    2: {'screen': screen},
    3: {'overlay=hard(s,b)': lambda b, s: hard(s, b)},
    4: {'min': np.minimum}, 5: {'max': np.maximum},
    6: {'dodge': lambda b, s: np.where(b == 0, 0, np.where(s == 255, 255, np.minimum(255, b * 255 // np.maximum(255 - s, 1)))),
        'dodge_r': lambda b, s: np.where(b == 0, 0, np.where(s == 255, 255, np.minimum(255, (b * 255 + (255 - s) // 2) // np.maximum(255 - s, 1))))},
    7: {'burn': lambda b, s: np.where(b == 255, 255, np.where(s == 0, 0, 255 - np.minimum(255, (255 - b) * 255 // np.maximum(s, 1)))),
        'burn_r': lambda b, s: np.where(b == 255, 255, np.where(s == 0, 0, 255 - np.minimum(255, ((255 - b) * 255 + s // 2) // np.maximum(s, 1))))},
    8: {'hard': hard},
    9: {'soft_round': lambda b, s: np.floor(soft_f(b, s) * 255 + 0.5).astype(np.int64),
        'soft_trunc': lambda b, s: (soft_f(b, s) * 255).astype(np.int64)},
    10: {'diff': lambda b, s: np.abs(b - s)},
    11: {'excl': lambda b, s: b + s - 2 * mul(b, s)},
    16: {'add': lambda b, s: np.minimum(b + s, 255)},
    17: {'sub': lambda b, s: np.maximum(b - s, 0)},
    18: {'div': lambda b, s: np.where(b == 0, 0, np.where(b >= s, 255, b * 255 // np.maximum(s, 1))),
         'div_r': lambda b, s: np.where(b == 0, 0, np.where(b >= s, 255, (b * 255 + s // 2) // np.maximum(s, 1)))},
}

# ── nao-separaveis (W3C), em double ───────────────────────────────────────────
def lum(c): return 0.3 * c[:, 0] + 0.59 * c[:, 1] + 0.11 * c[:, 2]
def clip(c):
    l = lum(c)[:, None]; n = c.min(1)[:, None]; x = c.max(1)[:, None]
    c = np.where(n < 0, l + (c - l) * l / np.where(l - n == 0, 1, l - n), c)
    c = np.where(x > 1, l + (c - l) * (1 - l) / np.where(x - l == 0, 1, x - l), c)
    return c
def setlum(c, l): return clip(c + (l - lum(c))[:, None])
def sat(c): return c.max(1) - c.min(1)
def setsat(c, s):
    mx, mn = c.max(1, keepdims=True), c.min(1, keepdims=True)
    rng = mx - mn
    return np.where(rng > 0, (c - mn) * s[:, None] / np.where(rng == 0, 1, rng), 0.0)
NONSEP = {
    12: lambda b, s: setlum(setsat(s, sat(b)), lum(b)),
    13: lambda b, s: setlum(setsat(b, sat(s)), lum(b)),
    14: lambda b, s: setlum(s, lum(b)),
    15: lambda b, s: setlum(b, lum(s)),
}
def nonsep(m, rnd):
    def f(b, s):
        r = NONSEP[m](b / 255.0, s / 255.0) * 255
        return np.floor(r + 0.5).astype(np.int64) if rnd else r.astype(np.int64)
    return f

# ── composicao final ──────────────────────────────────────────────────────────
def norm_t(b, c, a): return b + tdiv((c - b) * a, 255)        # trunca para zero
def norm_r(b, c, a): return b + np.floor((c - b) * a / 255 + 0.5).astype(np.int64)
def norm_f(b, c, a): return b + ((c - b) * a) // 255           # floor
FINAL = {'trunc0': norm_t, 'round': norm_r, 'floor': norm_f}
ALPHA = {'mul': lambda sa, op: mul(sa, op), 'trunc': lambda sa, op: sa * op // 255}

def cands(m):
    if m == 0:
        return {'normal': lambda b, s: s}
    if m in SEP:
        return {k: (lambda f: lambda b, s: f(b, s))(f) for k, f in SEP[m].items()}
    return {'round': nonsep(m, True), 'trunc': nonsep(m, False)}

def blend_vec(f, b, s):
    if f.__code__.co_argcount == 2 and m_cur in NONSEP:
        return f(b, s)
    return np.stack([f(b[:, k], s[:, k]) for k in range(3)], 1)

res = {}
for n, cells in sorted(L.items()):
    if not n.startswith('m'):
        continue
    m, op = int(n[1:3]), int(n[5:8])
    m_cur = m
    got = np.array(cells, dtype=np.int64)
    best = []
    for cn, f in cands(m).items():
        bl = f(B, S[:, :3]) if m in NONSEP else np.stack([f(B[:, k], S[:, k]) for k in range(3)], 1)
        bl = np.clip(bl, 0, 255)
        for an, af in ALPHA.items():
            a = af(S[:, 3], op)[:, None]
            for fn, ff in FINAL.items():
                r = ff(B, bl, a)
                ok = int(np.all(r == got, axis=1).sum())
                near = int(np.all(np.abs(r - got) <= 1, axis=1).sum())
                best.append((ok, near, f'{cn}/{an}/{fn}'))
    best.sort(reverse=True)
    res[n] = best[:3]
    print(n, len(got), best[:3], flush=True)
json.dump(res, open(os.path.join(D, 'fit.json'), 'w'), indent=1)
