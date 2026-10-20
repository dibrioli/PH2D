#!/usr/bin/env python3
"""Prancha de um oráculo: uma linha por pincel, as 3 cenas da bateria (gestos · tamanhos · sobre) lado a lado.

uso: python3 -I prancha_oraculo.py <saida.png> <escala> <rotulo>=<prefixo> [<rotulo>=<prefixo> ...]
     <prefixo> é o caminho sem `_<cena>.png` (ex.: fixtures/krita_bateria/c__Pencil-2); escala 0.5 = metade.
     Uma PNG RGBA é composta sobre branco. O rótulo vai numa faixa à esquerda.
"""
import sys

from PIL import Image, ImageDraw, ImageFont

CENAS = ("gestos", "tamanhos", "sobre")


def main():
    saida, escala, pares = sys.argv[1], float(sys.argv[2]), [a.split("=", 1) for a in sys.argv[3:]]
    w, h = int(1000 * escala), int(640 * escala)
    faixa, sep = 28, 6
    out = Image.new("RGB", (faixa + 3 * w + 2 * sep, len(pares) * (h + sep) - sep), (255, 255, 255))
    d = ImageDraw.Draw(out)
    try:
        fonte = ImageFont.truetype("DejaVuSans.ttf", 16)
    except OSError:
        fonte = ImageFont.load_default()
    for i, (rotulo, pref) in enumerate(pares):
        y = i * (h + sep)
        for j, cena in enumerate(CENAS):
            im = Image.open(f"{pref}_{cena}.png").convert("RGBA")
            fundo = Image.new("RGBA", im.size, (255, 255, 255, 255))
            im = Image.alpha_composite(fundo, im).convert("RGB").resize((w, h), Image.LANCZOS)
            x = faixa + j * (w + sep)
            out.paste(im, (x, y))
            d.rectangle([x, y, x + w - 1, y + h - 1], outline=(200, 200, 200))
        txt = Image.new("RGB", (h, faixa), (240, 240, 240))
        ImageDraw.Draw(txt).text((6, 5), rotulo, fill=(0, 0, 0), font=fonte)
        out.paste(txt.rotate(90, expand=True), (0, y))
    out.save(saida, optimize=True)
    print(saida, out.size)


if __name__ == "__main__":
    main()
