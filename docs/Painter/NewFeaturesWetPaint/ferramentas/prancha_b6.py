#!/usr/bin/env python3
"""A prancha do B6 (prompt 39): a ponta de CERDAS de fábrica ao lado da ponta LISA (Shape None), no Wet Paint.

Linhas: os presets da sonda `sonda_prancha_b6_lisa_e_cerdas` (PPM em `PH2D_SONDA_OUT`) e, por baixo, o Round do
Rebelle (fixtures/rebelle/ponta_textura_round_*, célula «molhado, água 10») como referência do oráculo.

uso: prancha_b6.py <dir_ppm> <saida.png> <preset>=<razão lisa÷cerdas>…   (ex. «Wet_Round=4.7»)
"""
import os
import sys

from PIL import Image, ImageDraw

AQUI = os.path.dirname(os.path.abspath(__file__))
REB = os.path.join(AQUI, "..", "fixtures", "rebelle")
ESCALA = 2
W, H = 320, 96
ROT = 230


def main():
    dirp, saida, *linhas = sys.argv[1:]
    presets = [ln.split("=") for ln in linhas]
    tw, th = W * ESCALA, H * ESCALA
    n = len(presets) + 1
    out = Image.new("RGB", (ROT + 2 * tw, 30 + n * th), "white")
    d = ImageDraw.Draw(out)
    d.text((ROT + 8, 8), "CERDAS de fabrica (como o pincel vem)", fill="black")
    d.text((ROT + tw + 8, 8), "LISA (Shape = None)", fill="black")
    for i, (p, razao) in enumerate(presets):
        y = 30 + i * th
        d.text((8, y + th // 2 - 14), p.replace("_", " "), fill="black")
        d.text((8, y + th // 2 + 2), f"lisa = {razao}x a tinta", fill="black")
        for j, ponta in enumerate(["cerdas", "lisa"]):
            im = Image.open(os.path.join(dirp, f"b6_{p}_{ponta}.ppm")).convert("RGB")
            out.paste(im.resize((tw, th), Image.NEAREST), (ROT + j * tw, y))
        d.line([(0, y), (ROT + 2 * tw, y)], fill=(200, 200, 200))
    y = 30 + len(presets) * th
    d.line([(0, y), (ROT + 2 * tw, y)], fill=(120, 120, 120), width=2)
    d.text((8, y + th // 2 - 14), "Rebelle Round", fill="black")
    d.text((8, y + th // 2 + 2), "(oraculo; tamanho 20 e 40)", fill="black")
    for j, tam in enumerate(["20", "40"]):
        im = Image.open(os.path.join(REB, f"ponta_textura_round_{tam}.png")).convert("RGB")
        cel = im.crop((0, 0, 320, 100)).resize((tw, th), Image.NEAREST)
        out.paste(cel, (ROT + j * tw, y))
        d.text((ROT + j * tw + 8, y + 6), f"Rebelle Watercolor Round, tamanho {tam} (nao e cerdas/lisa)", fill="black")
    out.save(saida)


if __name__ == "__main__":
    main()
