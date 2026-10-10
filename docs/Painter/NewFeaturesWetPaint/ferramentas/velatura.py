#!/usr/bin/env python3
"""A régua da VELATURA (W1e, F6): onde fica o cruzamento azul-sobre-vermelho entre «o azul POR CIMA» (γ = 0)
e «MULTIPLICAR» as duas tintas (γ = 1, a lei de Beer–Lambert de um pigmento transparente), em luz LINEAR:

    γ = (cruz − azul) / (vermelho·azul − azul)          por canal, cada cor como aparece sobre o branco

e a SANGRIA: quanto o vermelho AO LADO do cruzamento (15 px) se moveu para o azul (Δ do canal azul,
níveis sRGB), depois de tudo assentar. O pincel sujo: o rasto do azul depois do cruzamento − o azul limpo.

As caixas são as da sonda `wetpaint/sonda_mistura.rs`; `--rebelle` desloca-as (+200, +170) para as fotos de
`mistura_papel.py`. Uso: velatura.py [--rebelle] durante.png|ppm depois.png|ppm [antes.png|ppm]
"""
import sys

import numpy as np
from PIL import Image


def lin(c):
    c = np.asarray(c, dtype=float) / 255.0
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def caixa(a, x0, x1, y0, y1, dx, dy):
    return a[y0 + dy:y1 + dy, x0 + dx:x1 + dx].reshape(-1, 3).mean(0)


def gama(cruz, verm, azul):
    c, v, b = lin(cruz), lin(verm), lin(azul)
    den = v * b - b
    g = np.where(np.abs(den) > 1e-4, (c - b) / np.where(np.abs(den) > 1e-4, den, 1), np.nan)
    return g


def main():
    args = sys.argv[1:]
    dx, dy = (200, 170) if args and args[0] == "--rebelle" else (0, 0)
    if dx:
        args = args[1:]
    imgs = [np.asarray(Image.open(p).convert("RGB"), dtype=float) for p in args]
    for nome, a in zip(["durante", "depois", "antes"], imgs):
        cruz = caixa(a, 295, 306, 145, 156, dx, dy)
        verm = caixa(a, 195, 206, 145, 156, dx, dy)
        azul = caixa(a, 295, 306, 60, 100, dx, dy)
        rasto = caixa(a, 295, 306, 185, 225, dx, dy)
        perto = caixa(a, 315, 330, 145, 156, dx, dy)
        g = gama(cruz, verm, azul)
        print(f"{nome:8} γ R {g[0]:+.2f} G {g[1]:+.2f} B {g[2]:+.2f} média {np.nanmean(g):+.2f} | "
              f"cruz {tuple(int(round(v)) for v in cruz)} verm {tuple(int(round(v)) for v in verm)} "
              f"azul {tuple(int(round(v)) for v in azul)} | rasto−azul {tuple(int(round(v)) for v in rasto - azul)} "
              f"| verm a 15 px {tuple(int(round(v)) for v in perto)} (ΔB vs longe {perto[2] - verm[2]:+.0f})")


if __name__ == "__main__":
    main()
