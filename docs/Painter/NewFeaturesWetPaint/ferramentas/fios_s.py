#!/usr/bin/env python3
"""A régua dos FIOS no S (W1g, O3): quantos fios descem da orla de baixo do S, quão LARGOS e quão
COMPRIDOS. A orla de cada coluna é a da régua do recto (`recto.py`: a tinta mais baixa cuja corrida
horizontal passa de 8 px, ligada à de cima sem buraco de mais de 3 linhas); a base = a mediana da orla
em ±40 colunas. Um FIO = colunas seguidas cuja tinta mais baixa passa a base por ≥ 20 px; largura = as
colunas dele, comprimento = a descida máxima. Tinta = croma (max − min do rgb) > 12.

Rebelle (s_inclinado*.png, tela no máximo): ~10 fios de 3–7 px e 250–460 px.

Uso: fios_s.py tela.(png|ppm) [...]
"""
import sys

import numpy as np
from PIL import Image

LARGO = 8
DESCE = 20


def le(caminho):
    a = np.asarray(Image.open(caminho).convert("RGB")).astype(int)
    t = (a.max(axis=2) - a.min(axis=2)) > 12
    h, w = t.shape
    corrida = np.zeros_like(t, dtype=int)
    for y in range(h):
        row = t[y]
        x = 0
        while x < w:
            if not row[x]:
                x += 1
                continue
            x0 = x
            while x + 1 < w and row[x + 1]:
                x += 1
            corrida[y, x0:x + 1] = x - x0 + 1
            x += 1
    larga = t & (corrida > LARGO)
    cols = np.where(larga.any(axis=0))[0]
    a0, b0 = cols[0], cols[-1]
    m = (b0 - a0) * 8 // 100
    xs = range(a0 + m, b0 - m + 1)
    orla, baixo = [], []
    for x in xs:
        ys = np.where(larga[:, x])[0]
        if ys.size == 0:
            orla.append(None)
        else:
            yb, buraco = ys[0], 0
            for y in range(ys[0], h):
                if larga[y, x]:
                    yb, buraco = y, 0
                else:
                    buraco += 1
                    if buraco > 3:
                        break
            orla.append(yb)
        yt = np.where(t[:, x])[0]
        baixo.append(yt[-1] if yt.size else None)
    base = []
    for k in range(len(orla)):
        j = sorted(v for v in orla[max(0, k - 40):k + 41] if v is not None)
        base.append(j[len(j) // 2] if j else None)
    fio = [b is not None and m_ is not None and b >= m_ + DESCE for b, m_ in zip(baixo, base)]
    fios = []
    k = 0
    while k < len(fio):
        if not fio[k]:
            k += 1
            continue
        j = k
        while j + 1 < len(fio) and fio[j + 1]:
            j += 1
        fios.append((j - k + 1, max(baixo[i] - base[i] for i in range(k, j + 1))))
        k = j + 1
    return fios


def main():
    for c in sys.argv[1:]:
        f = le(c)
        larg = sorted(w for w, _ in f)
        comp = sorted(d for _, d in f)
        finos_longos = sum(1 for w, d in f if w <= 7 and d >= 100)
        print(
            f"{c}: {len(f)} fios · largura mediana {larg[len(larg) // 2] if larg else 0} "
            f"(máx {larg[-1] if larg else 0}) · comprimento mediano {comp[len(comp) // 2] if comp else 0} "
            f"(máx {comp[-1] if comp else 0}) · finos (≤ 7 px) e longos (≥ 100 px): {finos_longos}"
        )


if __name__ == "__main__":
    main()
