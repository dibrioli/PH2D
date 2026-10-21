#!/usr/bin/env python3
"""mistura_foto.py <foto>... — a identidade de uma ferramenta da família `mistura` na cena «sobre» (as contas do
`Identidade::de` de `crates/ph2d-tool-painter/tests/it/familias/mistura.rs`): o cruzamento 2 (x = 500) sobre a faixa
de cima (y 120..180) e o da de baixo (y 440..500).

  resto  — o que fica da faixa no eixo do cruzamento (1 = intacta, 0 = papel), pelo verde: (255 − g) ÷ (255 − g_faixa)
  largura — as colunas (x 420..580, na linha do meio da faixa) que diferem da faixa intacta > 15 de 255
  borda  — as colunas dessa linha com o resto entre 0,1 e 0,9, ÷ 2 (a orla de cada lado)
  fundo  — quantos px abaixo da faixa (y > 180) o vermelho foi ARRASTADO (verde < 128 em x 480..520)
  orla_v — os px da orla de CIMA da faixa (resto entre 0,1 e 0,9, y0 − 50..y0) no eixo (x 500) menos os da intacta (x 200)
"""
import sys

import numpy as np
from PIL import Image


def identidade(f):
    a = np.asarray(Image.open(f).convert("RGB")).astype(float)
    g = a[:, :, 1]
    out = []
    for y0 in (150, 470):
        ref = float(np.median(g[y0, 60:200]))
        linha = g[y0, 420:581]
        resto_x = np.clip((255 - linha) / max(1.0, 255 - ref), 0, 1)
        resto = float(resto_x[78:83].mean())
        largura = int((np.abs(a[y0, 420:581] - np.median(a[y0, 60:200], axis=0)).max(axis=1) > 15).sum())
        borda = float(((resto_x > 0.1) & (resto_x < 0.9)).sum()) / 2
        fundo = max([y - (y0 + 30) for y in range(y0 + 31, y0 + 150) if y < g.shape[0] and g[y, 480:521].min() < 128] or [0])
        # a orla de CIMA da faixa (y0 − 30) no eixo do cruzamento ÷ a da faixa intacta (x = 200): o desfoque a abre
        def orla(x):
            col = np.clip((255 - g[y0 - 50:y0 + 1, x]) / max(1.0, 255 - ref), 0, 1)
            return float(((col > 0.1) & (col < 0.9)).sum())
        orla_v = orla(500) - orla(200)
        out.append(dict(resto=round(resto, 3), largura=largura, borda=borda, fundo=fundo, orla_v=orla_v))
    return out


for f in sys.argv[1:]:
    print(f.split("fixtures/")[-1] if "fixtures/" in f else f, identidade(f))
