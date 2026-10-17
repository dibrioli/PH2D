#!/usr/bin/env python3
"""A prancha do prompt 42: o CRUZAMENTO vermelho × azul de cada pincel do bloco A, Rebelle × nosso.

Linhas: Ink Bullet · Dip Pen · Ink Brush · Marker Bullet · Marker Liner. Colunas: Rebelle molhado (11/10) · Rebelle
seco (16/10, `*seco.png`) · nosso hoje sem Glaze (a lei da obra) · nosso com o Glaze do pincel de fábrica. Recorte
x 200..400, y 380..580 da bancada (o cruzamento no meio), 2×, e o número da régua do 18 §2 por baixo de cada um.
As cenas nossas saem do portão `cada_preset_parece_a_sua_midia` (`PH2D_SONDA_OUT`, `midia_<preset>.ppm`) e da
sonda das mídias (`<receita>@g0.ppm`).

uso: prancha_lei_da_tinta.py <dir fixtures/rebelle> <dir das cenas nossas> <saida.png>
"""
import os
import sys

from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from midias_regua import cruzamento  # noqa: E402

ESCALA = 2
CAIXA = (200, 380, 400, 580)
ROT = 140
LINHAS = [("Ink Bullet", "ink"), ("Dip Pen", "dippen"), ("Ink Brush", "inkbrush"),
          ("Marker Bullet", "markerbullet"), ("Marker Liner", "marker")]
COLUNAS = ["Rebelle molhado", "Rebelle seco", "nosso sem Glaze", "nosso (de fábrica)"]


def png(p):
    if p.endswith(".ppm"):
        q = p[:-4] + ".png"
        Image.open(p).save(q)
        return q
    return p


def main():
    reb, nos, saida = sys.argv[1:4]
    t = (CAIXA[2] - CAIXA[0]) * ESCALA
    out = Image.new("RGB", (ROT + len(COLUNAS) * t, 30 + len(LINHAS) * (t + 22)), "white")
    d = ImageDraw.Draw(out)
    for j, c in enumerate(COLUNAS):
        d.text((ROT + j * t + 6, 8), c, fill="black")
    for i, (nome, foto) in enumerate(LINHAS):
        y = 30 + i * (t + 22)
        d.text((6, y + t // 2), nome, fill="black")
        fontes = [f"{reb}/{foto}.png", f"{reb}/{foto}seco.png",
                  f"{nos}/sem_glaze_{nome}.ppm", f"{nos}/midia_{nome}.ppm"]
        for j, f in enumerate(fontes):
            if not os.path.exists(f):
                d.text((ROT + j * t + 6, y + t // 2), "(falta)", fill="red")
                continue
            f = png(f)
            im = Image.open(f).convert("RGB").crop(CAIXA).resize((t, t), Image.NEAREST)
            out.paste(im, (ROT + j * t, y))
            d.text((ROT + j * t + 6, y + t + 4), f"cruzamento {cruzamento(f):.3f}", fill="black")
    out.save(saida)
    print(saida)


if __name__ == "__main__":
    main()
