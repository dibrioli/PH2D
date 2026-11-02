#!/usr/bin/env python3
"""A lei da cor/alfa da Convolution Matrix (offline) contra os casos RGBA do Trial: `A4b_conv_alpha`
(estado dos botoes lido por pixel) e os de `A4_conv_alpha` com o rotulo CORRIGIDO (o «A_off» de A4 era
o A clicado DEPOIS do kernel, logo o estado inverso do que o kernel traz). Kernels: os de
`crates/ph2d-pixel-core/src/filter/kernels.rs` (lidos do .rs). So bytes VISIVEIS + o alfa contam (o RGB
sob alfa 0 nao e determinado pelo clipboard: A4b `observed`).

Uso: python3 -I docs/PixelArt/ferramentas/fit_convolucao_alfa.py   (cwd = raiz do repo)
Imprime as celulas fora por candidata e, para a vencedora, onde.
"""
import json, re

FIX = 'crates/ph2d-pixel-core/tests/fixtures/oracle_trial_round3.json'
d = json.load(open(FIX))
KN = ['brightness', 'contrast', 'negative', 'blur-3x3']   # indice na lista do Trial = indice nos nossos ate 36
TRIAL = {2: 'negative', 3: 'blur-3x3', 14: 'sharpen-3x3', 20: 'misc-contour', 23: 'misc-marmolize',
         25: 'misc-rock-edges'}


def kernels():
    src = open('crates/ph2d-pixel-core/src/filter/kernels.rs').read()
    out = {}
    for m in re.finditer(r'name: "((?:[^"\\]|\\.)*)", w: (\d+), h: (\d+), cx: (\d+), cy: (\d+), div: (-?\d+), '
                         r'bias: (-?\d+), (?:channels: \[[^\]]*\], )?weights: &\[([^\]]*)\]', src):
        name = m.group(1).replace('\\\\', '\\')
        w, h, cx, cy, div, bias = map(int, m.groups()[1:7])
        ws = [int(v) for v in m.group(8).split(',')]
        out[name] = {(i - cx, j - cy): ws[j * w + i] for j in range(h) for i in range(w) if ws[j * w + i]}, div, bias
    return out


K = kernels()


def t0(a, b):
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b > 0) else -q


def cl(v):
    return max(0, min(255, v))


def cases():
    """(entrada, saida, kernel, A ligado, rotulo)"""
    out = []
    b = d['A4b_conv_alpha']
    c, st = b['cases_rgba'], b['button_state']['per_case']
    for key, g in c.items():
        if key.endswith(':input'):
            continue
        f, k, mode = key.split(':')
        out.append((c[f + ':input'], g, TRIAL[int(k)], st[key]['at_ok']['A'], 'A4b ' + key))
    a = d['A4_conv_alpha']['cases']
    default_a = {'negative': False, 'blur-3x3': True}
    for key, g in a.items():
        p = key.split(':')
        if p[1] == 'input' or int(p[1]) not in TRIAL:
            continue   # 37/38 (outline-transparent-layer) nao estao na nossa lista
        kern = TRIAL[int(p[1])]
        on = default_a[kern] != (len(p) == 3)   # «A_off» de A4 = o A clicado = o estado inverso
        out.append((a[p[0] + ':input'], g, kern, on, 'A4 ' + key))
    return out


def apply(law, src, kern, a_on):
    w, div, bias = K[kern]
    H, W = len(src), len(src[0])
    px = lambda x, y: src[min(max(y, 0), H - 1)][min(max(x, 0), W - 1)]
    res = []
    for y in range(H):
        row = []
        for x in range(W):
            old = src[y][x]
            nb = [(k, px(x - dx, y - dy)) for (dx, dy), k in w.items()]
            rgb, al = law(nb, div, bias)
            row.append(list(rgb) + [al if a_on else old[3]])
        res.append(row)
    return res


def alpha_lin(nb, div, bias):
    return cl(t0(sum(k * p[3] for k, p in nb), div) + bias)


def law_pure(nb, div, bias):            # hoje: RGB = formula pura, alfa linear
    return [cl(t0(sum(k * p[c] for k, p in nb), div) + bias) for c in range(3)], alpha_lin(nb, div, bias)


def law_zero(nb, div, bias):            # vizinho transparente = cor 0
    return [cl(t0(sum(k * p[c] * (p[3] > 0) for k, p in nb), div) + bias) for c in range(3)], alpha_lin(nb, div, bias)


def law_premul(nb, div, bias):          # cor pre-multiplicada, des-multiplicada pelo alfa de saida
    al = alpha_lin(nb, div, bias)
    pm = [t0(sum(k * p[c] * p[3] for k, p in nb), div * 255) + bias for c in range(3)]
    return [cl(v * 255 // al) if al else 0 for v in pm], al


def law_wa(nb, div, bias):              # cor = Σ w·a·c / Σ w·a × Σw/div + bias
    sw = sum(k for k, _ in nb); swa = sum(k * p[3] for k, p in nb)
    return [cl(t0(t0(sum(k * p[3] * p[c] for k, p in nb), swa) * sw, div) + bias) if swa else cl(bias)
            for c in range(3)], alpha_lin(nb, div, bias)


def law_skip(nb, div, bias):            # o transparente sai da soma E o seu peso sai do divisor
    dv = div - sum(k for k, p in nb if p[3] == 0)
    dv = dv or 1
    return [cl(t0(sum(k * p[c] for k, p in nb if p[3]), dv) + bias) for c in range(3)], alpha_lin(nb, div, bias)


LAWS = {'pura': law_pure, 'transparente=0': law_zero, 'pre-multiplicada': law_premul,
        'ponderada pelo alfa': law_wa, 'salta o transparente (peso sai do div)': law_skip}


def misses(law, verbose=False):
    n = 0; where = []
    for src, out, kern, a_on, tag in cases():
        r = apply(law, src, kern, a_on)
        for y in range(len(out)):
            for x in range(len(out[0])):
                o, p = out[y][x], r[y][x]
                vis = range(4) if o[3] else (3,)
                bad = [c for c in vis if o[c] != p[c]]
                if bad:
                    n += 1; where.append((tag, x, y, o, p))
    return n, where


if __name__ == '__main__':
    print('casos:', len(cases()))
    best = None
    for name, law in LAWS.items():
        n, where = misses(law)
        print(f'{name:42s} {n:4d} celulas fora')
        if best is None or n < best[1]:
            best = (name, n, where)
    print('vencedora:', best[0], best[1])
    for w in best[2][:20]:
        print('  ', w)
