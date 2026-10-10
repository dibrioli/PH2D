"""A régua do TILT (09/10): a bancada do doc 06 §2 — quatro traços horizontais — sobre as fotos do Rebelle
(`fixtures/rebelle_gotas/*.png`, reduzidas, com o rótulo em cima) e sobre os PPM da sonda
`tests/it/sonda_tilt_do_preset.rs`. Acha os traços sozinha (a escala sai do comprimento: 150 px + 40 de
diâmetro = 190 px de tela) e, por traço:

- `orla` = escuro médio das 3 linhas de baixo ÷ as 3 de cima do perfil vertical do miolo (a régua do doc 06:
  o Rebelle a tela no máximo sem gotas dá 3,1 · 2,3 nas águas 100 · 70);
- `desce` = quanto a tinta (> 25) passa abaixo da orla de baixo do miolo, em px de TELA.

uso: python3 -I tilt_rebelle.py img [img ...]
"""
import sys

import numpy as np
from PIL import Image

TINTA = 25


def escuro(path):
    a = np.asarray(Image.open(path).convert("RGB"), dtype=np.int32)
    return 255 - a.min(axis=2)


def tracos(d):
    h, w = d.shape
    tinta = d > TINTA
    linhas = tinta.sum(axis=1)
    # a banda dos traços: a linha com mais tinta e as vizinhas com tinta (ignora o rótulo de cima)
    y0 = int(np.argmax(linhas))
    cols = tinta[max(0, y0 - 3) : y0 + 4].any(axis=0)
    segs, x = [], 0
    while x < w:
        if cols[x]:
            a = x
            while x < w and cols[x]:
                x += 1
            if x - a > w * 0.05:
                segs.append((a, x))
        x += 1
    return y0, segs


def mede(path):
    d = escuro(path).astype(float)
    y0, segs = tracos(d)
    out = []
    for a, b in segs[:4]:
        esc = (b - a) / 190.0  # px da imagem por px de tela
        m0, m1 = int(a + 0.25 * (b - a)), int(b - 0.25 * (b - a))
        perfil = d[:, m0:m1].mean(axis=1)
        ys = [y for y in range(max(0, y0 - int(60 * esc)), min(d.shape[0], y0 + int(60 * esc))) if perfil[y] > TINTA]
        if not ys:
            out.append(None)
            continue
        t, bb = ys[0], ys[-1]
        n = max(1, round(3 * esc))
        cima = perfil[t : t + n].mean()
        baixo = perfil[bb - n + 1 : bb + 1].mean()
        fundo = bb
        for y in range(bb, d.shape[0]):
            if (d[y, a:b] > TINTA).any():
                fundo = y
        out.append((round(baixo / max(cima, 1.0), 2), round((fundo - bb) / esc), round(esc, 3)))
    return out


if __name__ == "__main__":
    for p in sys.argv[1:]:
        r = mede(p)
        print(f"{p.rsplit('/', 1)[-1]:<28}", " | ".join("—" if x is None else f"orla {x[0]:4.2f} desce {x[1]:4d}" for x in r), f"(esc {r[0][2] if r and r[0] else '?'})")
