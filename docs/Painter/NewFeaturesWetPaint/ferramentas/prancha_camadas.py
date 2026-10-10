#!/usr/bin/env python3
"""As pranchas da mistura em CAMADAS (prompt 15): 1:1 (a mancha azul inteira) e 8× (o miolo), passada a passada.

Linhas: o Rebelle (fixtures/rebelle_camadas/c_mol_*) e, do motor (`sonda_camadas` com `PH2D_SONDA_OUT`), cada
«condição_estado_variante» pedida. Colunas: o azul seco, as passadas 1 · 2 · 3 · 5 · 10 e o fim seco.

uso: prancha_camadas.py <dir_ppm> <saida_1x.png> <saida_8x.png> <linha>…   (linha = prefixo dos PPM, ex.
     «K–M_tudo_molhado_antes»; «rebelle» = as fotos do oráculo)
"""
import os
import sys

from PIL import Image, ImageDraw

AQUI = os.path.dirname(os.path.abspath(__file__))
REB = os.path.join(AQUI, "..", "fixtures", "rebelle_camadas")
COLS = [("azul", "00", "p00"), ("1", "01", "p01"), ("2", "02", "p02"), ("3", "03", "p03"), ("5", "05", "p05"),
        ("10", "10", "p10"), ("seco", "11", "seco")]
CAIXA = (140, 100, 460, 300)   # a mancha azul (coordenadas da sonda)
MIOLO = (285, 185, 315, 215)   # o miolo da régua, a 8×


def tile(dirp, linha, col, caixa):
    if linha == "rebelle":
        im = Image.open(os.path.join(REB, f"c_mol_{col[2]}.png")).convert("RGB")
        x0, y0, x1, y1 = caixa
        return im.crop((x0 + 200, y0 + 120, x1 + 200, y1 + 120))
    return Image.open(os.path.join(dirp, f"{linha}_{col[1]}.ppm")).convert("RGB").crop(caixa)


def prancha(dirp, linhas, caixa, escala, saida):
    w = (caixa[2] - caixa[0]) * escala
    h = (caixa[3] - caixa[1]) * escala
    rot = 260
    out = Image.new("RGB", (rot + w * len(COLS), 24 + h * len(linhas)), "white")
    d = ImageDraw.Draw(out)
    for j, c in enumerate(COLS):
        d.text((rot + j * w + 6, 6), c[0], fill="black")
    for i, ln in enumerate(linhas):
        d.text((6, 24 + i * h + h // 2), ln.replace("–", "-"), fill="black")
        for j, c in enumerate(COLS):
            t = tile(dirp, ln, c, caixa).resize((w, h), Image.NEAREST)
            out.paste(t, (rot + j * w, 24 + i * h))
    out.save(saida)


def main():
    dirp, s1, s8, *linhas = sys.argv[1:]
    prancha(dirp, linhas, CAIXA, 1, s1)
    prancha(dirp, linhas, MIOLO, 8, s8)


if __name__ == "__main__":
    main()
