#!/usr/bin/env python3
"""A prancha do smoke do 53: por linha, a foto do Rebelle · o preset novo · o vizinho de fábrica (a cena inteira, seca,
à largura da foto — os `id_*` e `v_*` do lote do `avalia_53.py`).

uso: prancha_53.py <dir do lote (.ppm/.png)> <saída.png>
"""
import os
import sys

from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from avalia_53 import OS_11, VIZINHO, slug  # noqa: E402

d, saida = sys.argv[1], sys.argv[2]
F = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "fixtures", "rebelle")
W, H, TOPO = 500, 320, 18


def abre(caminho):
    for c in (caminho, caminho[:-4] + ".ppm"):
        if os.path.exists(c):
            return Image.open(c).convert("RGB").resize((W, H), Image.LANCZOS)
    return Image.new("RGB", (W, H), "magenta")


folha = Image.new("RGB", (3 * W, len(OS_11) * (H + TOPO)), "white")
desenho = ImageDraw.Draw(folha)
for i, (p, foto, _, _, _) in enumerate(OS_11):
    y = i * (H + TOPO)
    for j, (titulo, img) in enumerate([
        (f"Rebelle · {foto}", abre(f"{F}/{foto}.png")),
        (f"PH2D · {p}", abre(f"{d}/id_{slug(p)}.png")),
        (f"vizinho · {VIZINHO[p]}", abre(f"{d}/v_{slug(p)}.png")),
    ]):
        folha.paste(img, (j * W, y + TOPO))
        desenho.text((j * W + 4, y + 3), titulo, fill="black")
folha.save(saida)
print(saida)
