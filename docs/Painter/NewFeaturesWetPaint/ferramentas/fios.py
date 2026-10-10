#!/usr/bin/env python3
"""A régua dos FIOS (W1e, F1): a bancada do doc 06 §2 — quatro traços horizontais de 150 px em y = 150,
x0 = 60 + 240·i — numa tela 1000×640 (as sondas `sonda_gota` / `sonda_tilt_do_preset`). Por traço:

- `orla` = escuro médio das 3 linhas de cima da orla de BAIXO do miolo ÷ as 3 de baixo da orla de CIMA
  (> 1 = a tinta desceu para a orla de baixo; Rebelle a meio, água 100: 1,6);
- `fios` = quantas corridas de tinta atravessam a linha 15 px abaixo da orla de baixo E chegam a 20 px;
- `desce` = o fio mais comprido abaixo da orla, px; `largura` = a largura mediana dos fios a meia altura.

Escuro = 255 − min(rgb), tinta = escuro > 25. Uso: fios.py tela.ppm [...]
"""
import sys

import numpy as np
from PIL import Image

TINTA = 25


def traco(d, i):
    x0 = 60 + 240 * i
    xa, xb = x0 - 25, x0 + 175
    miolo = d[:, x0 + 25:x0 + 125]
    linhas = (miolo > TINTA).mean(axis=1)
    y = 150
    while y + 1 < d.shape[0] and linhas[y + 1] >= 0.5:
        y += 1
    yb = y
    y = 150
    while y - 1 >= 0 and linhas[y - 1] >= 0.5:
        y -= 1
    yt = y
    orla = miolo[yb - 2:yb + 1].mean() / max(miolo[yt:yt + 3].mean(), 1e-9)
    tinta = d[:, xa:xb] > TINTA
    r = yb + 15
    fios = []
    if r < d.shape[0]:
        linha = tinta[r]
        x = 0
        while x < linha.size:
            if not linha[x]:
                x += 1
                continue
            x1 = x
            while x1 + 1 < linha.size and linha[x1 + 1]:
                x1 += 1
            # segue a corrida para baixo, pela janela das colunas dela (±2)
            c0, c1 = max(x - 2, 0), min(x1 + 2, linha.size - 1)
            y = r
            while y + 1 < d.shape[0] and tinta[y + 1, c0:c1 + 1].any():
                y += 1
                cols = np.nonzero(tinta[y, c0:c1 + 1])[0]
                c0, c1 = max(c0 + cols[0] - 2, 0), min(c0 + cols[-1] + 2, linha.size - 1)
            comp = y - yb
            if comp >= 20:
                meio = yb + comp // 2
                larg = int(tinta[meio, max(x - 6, 0):min(x1 + 7, linha.size)].sum())
                fios.append((comp, larg))
            x = x1 + 1
    desce = max((f[0] for f in fios), default=0)
    larg = int(np.median([f[1] for f in fios])) if fios else 0
    return orla, len(fios), desce, larg


def main():
    for path in sys.argv[1:]:
        d = 255 - np.asarray(Image.open(path).convert("RGB"), dtype=np.int32).min(axis=2)
        cols = []
        for i in range(4):
            o, n, ds, lg = traco(d, i)
            cols.append(f"orla {o:4.2f} fios {n:2d} desce {ds:3d} larg {lg:2d}")
        print(f"{path.split('/')[-1]:28} " + " | ".join(cols))


if __name__ == "__main__":
    main()
