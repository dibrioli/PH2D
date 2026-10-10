#!/usr/bin/env python3
"""A régua dos 4 PINCÉIS de fábrica (conferência do dono 10/10): a bancada de `gotas.py` (traços horizontais
de 150 px a 40 px, x0 = 60 + 240·i, y = 150) lida coluna a coluna, igual nos dois lados (as fotos do
Rebelle de `fixtures/rebelle_presets/` e as telas da sonda `wetpaint/sonda_presets.rs`).

Por coluna, com `escuro = 255 − min(rgb)` e o miolo = x0+40 … x0+110:
  tinta     a mediana do escuro no centro do traço (0 = papel, ~230 = o azul puro) — quão carregada
  largura   as linhas com escuro > 25 em torno de y = 150 (mediana por x), contra o tamanho 40
  orla      o escuro das 3 linhas da borda ÷ o do centro (> 1 = orla escura, a aguarela molhada)
  buracos   a fracção do miolo da faixa (60 % central) com escuro < ¼ do centro — o papel a aparecer
  escorre   a maior descida (px) abaixo da faixa com escuro > 25
uso: presets_regua.py <tela.png|ppm>…   (nomes à frente; colunas na ordem da bancada)
"""
import sys

import numpy as np
from PIL import Image


def coluna(e, i):
    x0 = 60 + 240 * i
    xs = range(x0 + 40, x0 + 111)
    larg, orlas, buracos, centro, fundo = [], [], [], [], 150
    for x in xs:
        c = e[:, x]
        ys = np.where(c[100:260] > 25)[0] + 100
        perto = ys[np.abs(ys - 150) < 60] if len(ys) else ys
        if len(perto) == 0:
            continue
        top, bot = perto.min(), perto.max()
        larg.append(bot - top + 1)
        meio = c[top:bot + 1]
        n = len(meio)
        cen = meio[n // 5: n - n // 5] if n >= 5 else meio
        centro.append(np.median(cen))
        orlas.append((np.mean(c[top:top + 3]) + np.mean(c[max(top, bot - 2):bot + 1])) / 2)
        buracos.append(np.mean(cen < np.median(cen) / 4) if len(cen) else 0)
        fundo = max(fundo, bot)
    # escorre: a descida mais longa abaixo da faixa (mediana do fundo) em toda a largura do traço
    fb = int(np.median([150 + l // 2 for l in larg])) if larg else 150
    desc = 0
    for x in range(x0 - 10, x0 + 161):
        c = e[fb:, x] > 25
        k = 0
        while k < len(c) and c[k]:
            k += 1
        desc = max(desc, k)
    t = float(np.median(centro)) if centro else 0.0
    return t, float(np.median(larg)) if larg else 0.0, (float(np.median(orlas)) / t if t else 0.0), \
        float(np.mean(buracos)) if buracos else 0.0, desc


def main():
    for f in sys.argv[1:]:
        a = np.asarray(Image.open(f).convert("RGB")).astype(int)
        e = 255 - a.min(axis=2)
        print(f.rsplit("/", 1)[-1])
        for i in range(4):
            t, l, o, b, d = coluna(e, i)
            print(f"  col {i + 1}: tinta {t:5.0f}  largura {l:5.1f}  orla {o:4.2f}  buracos {b:4.2f}  escorre {d:4d}")


if __name__ == "__main__":
    main()
