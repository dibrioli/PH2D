#!/usr/bin/env python3
"""Os CARRIS de um traço horizontal (W2, prompt 48; doc 48_medicao): nas linhas do miolo (a máscara de
`mede_traco.py` erodida 3 px) que cruzam ≥ 80 % do corpo do traço (fora dos 10 % de cada ponta), a mediana do
desvio da cobertura AO LONGO da linha, em % da cobertura do miolo. `0` = cada linha constante pelo traço
inteiro — o tapete de cerdas arrastado pela lane. A régua Rust é `regua_midia::Medida::carril`.

uso: carril.py <png>…   (recorte do recto da bancada, 20,60,580,120)
"""
import sys
import numpy as np
from PIL import Image
def carril(path, crop=(20, 60, 580, 120)):
    x, y, w, h = crop
    a = np.asarray(Image.open(path).convert('RGB'), dtype=np.float32)[y:y+h, x:x+w]
    borda = np.concatenate([a[:2].reshape(-1, 3), a[-2:].reshape(-1, 3)])
    bg = np.median(borda, axis=0)
    c = np.abs(a - bg).max(axis=2) / 255.0
    mask = c > 0.08
    inner = mask.copy()
    for _ in range(3):
        inner = inner & np.roll(inner, 1, 0) & np.roll(inner, -1, 0) & np.roll(inner, 1, 1) & np.roll(inner, -1, 1)
    cols = np.where(mask.any(0))[0]; x0, x1 = cols[0], cols[-1]; n = x1 - x0 + 1
    xa, xb = x0 + int(0.1 * n), x0 + int(0.9 * n)
    cob = c[inner].mean()
    sds = [c[r, xa:xb][inner[r, xa:xb]].std() for r in range(h) if inner[r, xa:xb].sum() > 0.8 * (xb - xa)]
    return 100 * float(np.median(sds)) / cob, len(sds)
for p in sys.argv[1:]:
    v, k = carril(p); print(f'{v:6.2f}  ({k} linhas)  {p.split("/")[-1]}')
