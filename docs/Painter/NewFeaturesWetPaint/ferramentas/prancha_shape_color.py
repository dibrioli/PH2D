#!/usr/bin/env python3
"""A prancha do prompt 41: a Shape Color no Wet Paint, ao lado do Digital.

Linhas: sem rampa · Texture Color · Falloff Color · as duas (Shape Voronoi, rampa verde -> vermelho, pincel
azul). Colunas: Digital · Wet Paint (estado cru) no soltar e a 10 s · Wet Round de fabrica no soltar e a 10 s ·
um carimbo de raio 40 no Digital e no Wet Paint.
Os PPM saem da sonda `sonda_prancha_shape_color` (`PH2D_SONDA_OUT`).

uso: prancha_shape_color.py <dir_ppm> <saida.png>
"""
import os
import sys

from PIL import Image, ImageDraw

ESCALA = 2
L = 128
ROT = 150
LINHAS = [("desligada", "sem rampa"), ("texture", "Texture Color"), ("falloff", "Falloff Color"),
          ("duas", "as duas camadas")]
COLUNAS = [("digital", "Digital"), ("wet", "Wet Paint (soltar)"), ("wet10", "Wet Paint (10 s)"),
           ("round", "Wet Round (soltar)"), ("round10", "Wet Round (10 s)"),
           ("cdigital", "carimbo r40: Digital"), ("cwet", "carimbo r40: Wet Paint")]


def main():
    dirp, saida = sys.argv[1:3]
    t = L * ESCALA
    out = Image.new("RGB", (ROT + len(COLUNAS) * t, 30 + len(LINHAS) * t), "white")
    d = ImageDraw.Draw(out)
    for j, (_, rotulo) in enumerate(COLUNAS):
        d.text((ROT + j * t + 8, 8), rotulo, fill="black")
    for i, (chave, rotulo) in enumerate(LINHAS):
        y = 30 + i * t
        d.text((8, y + t // 2 - 6), rotulo, fill="black")
        for j, (col, _) in enumerate(COLUNAS):
            im = Image.open(os.path.join(dirp, f"sc_{chave}_{col}.ppm")).convert("RGB")
            out.paste(im.resize((t, t), Image.NEAREST), (ROT + j * t, y))
        d.line([(0, y), (ROT + len(COLUNAS) * t, y)], fill=(200, 200, 200))
    out.save(saida)


if __name__ == "__main__":
    main()
