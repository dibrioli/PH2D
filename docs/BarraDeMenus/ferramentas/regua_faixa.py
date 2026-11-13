#!/usr/bin/env python3
"""Régua de pixels da faixa (onda 7): mede numa FOTO as bandas do topo e, em cada banda, cada item.

Banda = linhas seguidas com a mesma cor de fundo (moda da linha). Item = colunas com «tinta» (pixel que
difere do fundo da banda) separadas por um vão >= --vao px. Por item: caixa x0..x1, topo/base da tinta,
desvio do centro do item ao centro da banda (dc, px; + = abaixo) e o contraste de pico (dL máximo em
luminância relativa 0..1 — um traço nítido chega perto do contraste do texto; um traço sub-pixel não).

Uso: regua_faixa.py FOTO [--y0 0] [--y1 120] [--x0 0] [--x1 W] [--vao 5] [--min-banda 12]
"""
import argparse
import sys

import numpy as np
from PIL import Image


def lum(rgb):
    c = rgb / 255.0
    c = np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
    return 0.2126 * c[..., 0] + 0.7152 * c[..., 1] + 0.0722 * c[..., 2]


def moda(linha):
    q = (linha // 4).astype(np.int32)
    k = q[:, 0] * 65536 + q[:, 1] * 256 + q[:, 2]
    v, n = np.unique(k, return_counts=True)
    m = v[n.argmax()]
    return linha[k == m].mean(axis=0)


def bandas(img, min_banda):
    fundos = [moda(img[y]) for y in range(img.shape[0])]
    out, ini = [], 0
    for y in range(1, len(fundos) + 1):
        if y == len(fundos) or np.abs(fundos[y] - fundos[ini]).max() > 6:
            out.append((ini, y, fundos[ini]))
            ini = y
    # bandas finas (filetes de 1–2 px) contam como separadores, não como bandas
    return [b for b in out if b[1] - b[0] >= min_banda], [b for b in out if b[1] - b[0] < min_banda]


def itens(img, y0, y1, fundo, vao):
    sub = img[y0:y1].astype(np.float64)
    d = np.abs(sub - fundo).max(axis=2)
    tinta = d > 24
    cols = tinta.any(axis=0)
    L = lum(sub)
    Lf = lum(fundo[None, None, :])[0, 0]
    res, x, W = [], 0, len(cols)
    while x < W:
        if not cols[x]:
            x += 1
            continue
        a, ultimo = x, x
        while x < W and (cols[x] or x - ultimo <= vao):
            if cols[x]:
                ultimo = x
            x += 1
        b = ultimo + 1
        rows = np.where(tinta[:, a:b].any(axis=1))[0]
        top, bot = rows[0], rows[-1] + 1
        pico = float(np.abs(L[:, a:b] - Lf).max())
        res.append(dict(x0=a, x1=b, w=b - a, top=int(top), h=int(bot - top),
                        dc=(top + bot) / 2 - (y1 - y0) / 2, pico=round(pico, 3)))
    return res


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("foto")
    ap.add_argument("--y0", type=int, default=0)
    ap.add_argument("--y1", type=int, default=120)
    ap.add_argument("--x0", type=int, default=0)
    ap.add_argument("--x1", type=int, default=None)
    ap.add_argument("--vao", type=int, default=5)
    ap.add_argument("--min-banda", type=int, default=12)
    a = ap.parse_args()
    img = np.asarray(Image.open(a.foto).convert("RGB")).astype(np.float64)
    img = img[a.y0:a.y1, a.x0:a.x1]
    bs, filetes = bandas(img, a.min_banda)
    print(f"# {a.foto}  (y {a.y0}..{a.y1}, x {a.x0}..{a.x1 or 'fim'})")
    print("filetes:", [(f[0] + a.y0, f[1] - f[0]) for f in filetes])
    for (y0, y1, f) in bs:
        its = itens(img, y0, y1, f, a.vao)
        dcs = [i["dc"] for i in its]
        print(f"\nBANDA y={y0 + a.y0}..{y1 + a.y0} altura={y1 - y0} fundo=#{int(f[0]):02x}{int(f[1]):02x}{int(f[2]):02x}"
              f"  itens={len(its)}  dc min/max={min(dcs, default=0):+.1f}/{max(dcs, default=0):+.1f}")
        for i in its:
            print(f"  x {i['x0'] + a.x0:5d}..{i['x1'] + a.x0:5d} w{i['w']:4d}  tinta y+{i['top']:2d} h{i['h']:3d}"
                  f"  dc {i['dc']:+5.1f}  pico {i['pico']:.3f}")


if __name__ == "__main__":
    sys.exit(main())
