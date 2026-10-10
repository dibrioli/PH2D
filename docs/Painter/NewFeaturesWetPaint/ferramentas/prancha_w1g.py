#!/usr/bin/env python3
"""As pranchas da W1g: cada opção desligada · ligada · Rebelle, a 1:1 e a 8× (o recorte da orla / dos
fios). As telas são as da sonda `sonda_opcoes` (`PH2D_SONDA_OUT`); o Rebelle, os fixtures de
`fixtures/rebelle_gotas` (o `meio.png` está a 40 %: amplia-se ao tamanho da tela).

Uso: prancha_w1g.py <pasta dos .ppm> <saída sem extensão>
"""
import sys

from PIL import Image, ImageDraw

W, H = 1000, 640


def abre(c):
    i = Image.open(c).convert("RGB")
    return i if i.size == (W, H) else i.resize((W, H), Image.LANCZOS)


def main():
    d, saida = sys.argv[1], sys.argv[2]
    reb = "fixtures/rebelle_gotas/"
    linhas = [
        ("Heavy Edge - bancada, tilt a meia", f"{d}/bancada_v0_k0_meia.ppm", f"{d}/bancada_v0_k5_meia.ppm",
         reb + "meio.png", (70, 140, 190, 200)),
        ("Heavy Edge - S, tilt no maximo", f"{d}/s_v0_k0_máxima.ppm", f"{d}/s_v0_k5_máxima.ppm",
         reb + "s_inclinado.png", (430, 180, 560, 300)),
        ("Live Water - S no Up, tilt a meia", f"{d}/up_v0_meia.ppm", f"{d}/up_v1_meia.ppm",
         reb + "s_inclinado.png", (430, 180, 560, 300)),
        ("Live Water - S no Up, tilt no maximo", f"{d}/up_v0_máxima.ppm", f"{d}/up_v1_máxima.ppm",
         reb + "s_inclinado.png", (430, 180, 560, 300)),
    ]
    rot = 28
    um = Image.new("RGB", (3 * W, len(linhas) * (H + rot)), "white")
    oito = Image.new("RGB", (3 * 1040, len(linhas) * (1000 + rot)), "white")
    for k, (nome, a, b, r, caixa) in enumerate(linhas):
        y = k * (H + rot)
        y8 = k * (1000 + rot)
        dr = ImageDraw.Draw(um)
        d8 = ImageDraw.Draw(oito)
        reb_rot = "Rebelle (1 min depois; sem foto no Up)" if "Up" in nome else "Rebelle"
        for j, (c, etiqueta) in enumerate([(a, "desligada"), (b, "ligada"), (r, reb_rot)]):
            im = abre(c)
            um.paste(im, (j * W, y + rot))
            dr.text((j * W + 8, y + 6), f"{nome} - {etiqueta}", fill="black")
            x0, y0, x1, y1 = caixa
            corte = im.crop(caixa).resize(((x1 - x0) * 8, (y1 - y0) * 8), Image.NEAREST)
            oito.paste(corte, (j * 1040, y8 + rot))
            d8.text((j * 1040 + 8, y8 + 6), f"{nome} - {etiqueta} (8x, x {x0}-{x1}, y {y0}-{y1})", fill="black")
    um.save(saida + ".png", optimize=True)
    oito.save(saida + "_8x.png", optimize=True)


if __name__ == "__main__":
    main()
