#!/usr/bin/env python3
"""Ajuste OFFLINE das leis de Brightness/Contrast, Color Curve e Hue/Saturation do Aseprite 1.3.18
contra a fixtura medida `crates/ph2d-pixel-core/tests/fixtures/oracle_trial_adjust.json`.

Uso:  python3 -I docs/PixelArt/ferramentas/fit_ajustes.py [caminho/da/fixtura.json]

Imprime as células erradas por caso de cada lei vencedora. Só stdlib.

Vencedores (2026-11-02):
  * Brightness/Contrast — 0 erros em todos os casos:
        y = v/255 ; y = (y - 0.5)*(1 + c/100) + 0.5 ; y = y*(1 + b/100)   (contraste ANTES do brilho)
        y = clamp(y, 0, 1) ; out = floor(y * 255.5)                        (nem round nem floor(y*255))
    Rodada 2 (43 casos cinzentos, `round2_brightness_contrast.cases_gray_lut`): também 0 em 11 008 —
    o «1 613 a ±1» anotado na fixtura era o floor(y*255) rival, não esta lei.
  * Color Curve — 0 erros: interpolação LINEAR por troços entre os pontos (inteiros), out = floor.
  * Hue/Saturation — a lei em aritmética EXACTA (racional) acerta toda célula que não é empate;
    os erros restantes são todos empates exactos k+0.5 desempatados pelo ruído do float do Aseprite.
"""
import json
import math
import sys
from fractions import Fraction as Fr

FIX = sys.argv[1] if len(sys.argv) > 1 else 'crates/ph2d-pixel-core/tests/fixtures/oracle_trial_adjust.json'
D = json.load(open(FIX))


# ---------------------------------------------------------------- Brightness / Contrast
def bc_lut(b, c):
    out = []
    for v in range(256):
        y = v / 255.0
        y = (y - 0.5) * (1.0 + c / 100.0) + 0.5
        y = y * (1.0 + b / 100.0)
        y = min(1.0, max(0.0, y))
        out.append(int(math.floor(y * 255.5)))
    return out


def bc_report():
    print('== Brightness/Contrast  (contraste -> brilho, floor(y*255.5))')
    tot = 0
    for k, lut in D['brightness_contrast']['cases'].items():
        b, c = map(int, k.split(','))
        n = sum(1 for a, e in zip(bc_lut(b, c), lut) if a != e)
        tot += n
        print(f'  b,c={k:>9}: {n} erradas')
    print(f'  TOTAL {tot}')
    # rivais, para registo
    for name, fn in [
        ('floor(y*255) contraste->brilho', lambda y: math.floor(y * 255)),
        ('floor(y*255+0.5)', lambda y: math.floor(y * 255 + 0.5)),
    ]:
        t = 0
        for k, lut in D['brightness_contrast']['cases'].items():
            b, c = map(int, k.split(','))
            for v in range(256):
                y = min(1.0, max(0.0, ((v / 255 - 0.5) * (1 + c / 100) + 0.5) * (1 + b / 100)))
                t += fn(y) != lut[v]
        print(f'  rival {name}: {t}')


# ---------------------------------------------------------------- Color Curve
def curve_lut(px, py):
    xs, ys = [0, px, 255], [0, py, 255]
    out = []
    for x in range(256):
        i = 0 if x <= px else 1
        x0, x1, y0, y1 = xs[i], xs[i + 1], ys[i], ys[i + 1]
        y = y0 + (y1 - y0) * (x - x0) / (x1 - x0)
        out.append(max(0, min(255, int(math.floor(y)))))
    return out


def curve_report():
    print('== Color Curve  (linear por troços, floor; ponto do meio procurado em inteiros)')
    cc = D['color_curve']
    for key in [k for k in cc if k.startswith('lut_')]:
        lut = cc[key]
        ok = [(px, py) for px in range(110, 146) for py in range(100, 220) if curve_lut(px, py) == lut]
        best = min(((sum(a != b for a, b in zip(curve_lut(px, py), lut)), px, py)
                    for px in range(110, 146) for py in range(100, 220)))
        print(f'  {key}: pontos inteiros que dão 0 erros = {ok or "nenhum"}; melhor = {best}')


# ---------------------------------------------------------------- Hue / Saturation
def px_row(s):
    return [tuple(int(s[i * 6 + k * 2:i * 6 + k * 2 + 2], 16) for k in range(3)) for i in range(len(s) // 6)]


def _cl(x):
    return min(Fr(1), max(Fr(0), x))


def hs_exact(mode, H, S, L, c):
    """HSL/HSV do livro em racionais exactos; devolve r,g,b em [0,1] (Fraction)."""
    r, g, b = [Fr(v, 255) for v in c]
    mx, mn = max(r, g, b), min(r, g, b)
    C = mx - mn
    if C == 0:
        h = Fr(0)
    elif mx == r:
        h = ((g - b) / C) % 6 * 60
    elif mx == g:
        h = ((b - r) / C + 2) * 60
    else:
        h = ((r - g) / C + 4) * 60
    if mode.startswith('HSL'):
        l = (mx + mn) / 2
        s = Fr(0) if C == 0 else C / (1 - abs(2 * l - 1))
    else:
        l = mx
        s = Fr(0) if mx == 0 else C / mx
    h = (h + H) % 360
    if mode.endswith('+'):
        s, l = _cl(s + Fr(S, 100)), _cl(l + Fr(L, 100))
    else:
        s, l = _cl(s * (1 + Fr(S, 100))), _cl(l * (1 + Fr(L, 100)))
    if mode.startswith('HSL'):
        C = (1 - abs(2 * l - 1)) * s
        m = l - C / 2
    else:
        C = l * s
        m = l - C
    hp = h / 60
    X = C * (1 - abs(hp % 2 - 1))
    i = int(math.floor(hp)) % 6
    rr, gg, bb = [(C, X, 0), (X, C, 0), (0, C, X), (0, X, C), (X, 0, C), (C, 0, X)][i]
    return rr + m, gg + m, bb + m


def hs_report():
    print('== Hue/Saturation  (lei exacta + floor(x*255+0.5); erros por caso; empates = x*255 exactamente k+0.5)')
    IN = [c for r in D['input']['in'] for c in px_row(r)]
    tot = 0
    tot_nontie = 0
    for k, rows in D['hue_saturation']['cases'].items():
        out = [c for r in rows for c in px_row(r)]
        mode, vals = k.split(':')
        H, S, L = map(int, vals.split(','))
        bad = ties = up = nontie = 0
        for c, o in zip(IN, out):
            p = hs_exact(mode, H, S, L, c)
            got = tuple(max(0, min(255, math.floor(x * 255 + Fr(1, 2)))) for x in p)
            bad += got != o
            for ch in range(3):
                v = p[ch] * 255
                if v.denominator == 2:
                    ties += 1
                    up += o[ch] == math.ceil(v)
                elif o[ch] != math.floor(v + Fr(1, 2)):
                    nontie += 1
        tot += bad
        tot_nontie += nontie
        print(f'  {k:>16}: {bad:4d} células erradas | canais empatados {ties:4d} (sobem {up}, descem {ties - up}) | erros fora de empate {nontie}')
    print(f'  TOTAL células erradas {tot}; erros fora de empate {tot_nontie}')


if __name__ == '__main__':
    bc_report()
    curve_report()
    hs_report()
