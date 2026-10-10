#!/usr/bin/env python3
"""A régua das CAMADAS (prompt 15): a cor do MIOLO passada a passada, nas fotos da bancada (`camadas_tela.sh`, com
`--rebelle`: caixas deslocadas de (+200, +120)) ou nas telas PPM/PNG da sonda do motor (sem deslocamento).

Por foto imprime o miolo (x 285–315, y 185–215) em sRGB, a luminância relativa (luz linear, Rec. 709) e duas
leituras da barra: `abaixo_do_azul` = a luminância do miolo menos a do azul sozinho (negativo = mais escuro que a
mais escura das duas tintas, o PRETO) e `para_o_amarelo` = a fracção do caminho azul → amarelo em luz linear,
média do R e do G (0 = o azul, 1 = o amarelo da referência).

uso: camadas_regua.py [--rebelle] <foto|ppm>…   (a 1.ª é a do azul seco, antes da 1.ª passada)
"""
import sys

import numpy as np
from PIL import Image


def lin(c):
    c = np.asarray(c, dtype=float) / 255.0
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def lum(c):
    lc = lin(c)
    return float(0.2126 * lc[0] + 0.7152 * lc[1] + 0.0722 * lc[2])


def caixa(a, x0, x1, y0, y1, d):
    return a[y0 + d[1]:y1 + d[1], x0 + d[0]:x1 + d[0]].reshape(-1, 3).mean(0)


def main():
    args = sys.argv[1:]
    d = (0, 0)
    if args and args[0] == "--rebelle":
        d = (200, 120)
        args = args[1:]
    fotos = [np.asarray(Image.open(f).convert("RGB")).astype(float) for f in args]
    azul = caixa(fotos[0], 160, 200, 190, 210, d)
    amarelo = caixa(fotos[0], 280, 320, 355, 366, d)
    print(f"azul {np.round(azul).astype(int).tolist()} lum {lum(azul):.4f} · amarelo {np.round(amarelo).astype(int).tolist()} lum {lum(amarelo):.4f}")
    la, ly = lin(azul), lin(amarelo)
    for nome, a in zip(args, fotos):
        m = caixa(a, 285, 315, 185, 215, d)
        lm = lin(m)
        frac = np.mean([(lm[c] - la[c]) / (ly[c] - la[c]) for c in (0, 1)])
        print(f"{nome.rsplit('/', 1)[-1]:<28} miolo {np.round(m).astype(int).tolist()!s:<16} lum {lum(m):.4f}"
              f"  abaixo_do_azul {lum(m) - lum(azul):+.4f}  para_o_amarelo {frac:+.2f}")


if __name__ == "__main__":
    main()
