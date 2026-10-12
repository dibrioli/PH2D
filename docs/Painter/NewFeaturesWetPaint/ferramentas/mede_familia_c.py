#!/usr/bin/env python3
"""A régua da família C (prompt 23): seis números por imagem 1000×640 da cena da bancada (grelha A–J × 1–6).

- `lado`: a largura (px) do traço VERTICAL vermelho em D4 e D6 (fora do cruzamento) — o chato fino de lado.
- `alto`: a largura (px) do traço HORIZONTAL azul A2–F2 — a referência do mesmo pincel de frente.
- `tinta`: a tinta mediana no miolo do A2–F2 (0 = papel; ABSOLUTA, para comparar a claridade das duas).
- `falhas`: a fracção das linhas DENTRO da largura do A2–F2 cuja tinta fica abaixo de metade da do miolo
  (o papel entre os tufos).
- `estrias`: o desvio-padrão / média da tinta entre as linhas do miolo do A2–F2 (o papel claro entre tufos
  que não chega a metade: as riscas claras do Express Oil do Rebelle).
- `grao`: a diferença média de tinta entre píxeis vizinhos AO LONGO do A2–F2, nas linhas do miolo (o
  granulado: tinta lisa ≈ 0, sal e pimenta alto).

Tinta de um píxel = 1 − min(R,G,B)/255 para o vermelho e 1 − R/255 para o azul (o canal que a tinta mais
apaga). uso: mede_familia_c.py <img>…
"""
import sys

import numpy as np
from PIL import Image


def tinta(img, canal):
    a = np.asarray(Image.open(img).convert("RGB"), dtype=np.float64)
    if canal == "azul":
        t = 1.0 - a[..., 0] / 255.0  # o R cai no azul
        t[a[..., 2] < a[..., 0]] = 0.0  # o vermelho não conta
    else:
        t = 1.0 - a[..., 1] / 255.0  # o G cai no vermelho
        t[a[..., 0] < a[..., 2]] = 0.0  # o azul não conta
    return t


def largura(perfil):
    """Px acima de metade do pico (o pico = p95 do perfil, para não ser um píxel)."""
    pico = np.percentile(perfil, 95)
    return int((perfil > 0.5 * pico).sum()), pico


def mede(img):
    azul = tinta(img, "azul")
    verm = tinta(img, "verm")
    # A2–F2 (o traço em y ≈ 98–140 da imagem): linhas 80–160, colunas 150–500 (longe das pontas).
    perfil_h = azul[80:160, 150:500].mean(axis=1)
    alto, pico = largura(perfil_h)
    dentro = perfil_h > 0.15 * pico
    miolo = perfil_h[dentro]
    nucleo = perfil_h[perfil_h > 0.5 * pico]
    linhas = azul[80:160, 150:500][perfil_h > 0.5 * pico]
    grao = float(np.abs(np.diff(linhas, axis=1)).mean())
    estrias = float(nucleo.std() / nucleo.mean())
    falhas = float((miolo < 0.5 * np.median(miolo)).mean()) if miolo.size else 0.0
    # D4–D6 (o traço em y ≈ 380–577), fora do cruzamento com o A5 (y ≈ 458–500): colunas 240–360.
    lados = [largura(verm[y0:y1, 240:360].mean(axis=0))[0] for y0, y1 in ((395, 445), (515, 565))]
    return dict(lado=float(np.mean(lados)), alto=alto, tinta=round(float(np.median(miolo)), 3), falhas=round(falhas, 3), estrias=round(estrias, 3), grao=round(grao, 3))


if __name__ == "__main__":
    for f in sys.argv[1:]:
        print(f"{f.rsplit('/', 1)[-1]:40s} {mede(f)}")
