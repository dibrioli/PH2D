#!/usr/bin/env python3
"""W16 — entradas de IMAGEM para a sonda do recorte/espelho/GIF no editor do Excalidraw 0.18.1.

Gera, na pasta dada (por omissão a deste script):
- `w16_quadrantes.png` 400×300 RGB, quatro quadrantes 200×150 de cores que se lêem ao píxel mesmo
  esbatidas sobre branco: TL vermelho (255,0,0) · TR verde (0,160,0) · BL azul (0,0,255) · BR preto.
  O canal a 0 de cada cor dá o alfa: sobre branco, v = (1−α)·255.
- `w16_anim.gif` 120×120, 3 quadros lisos de 200 ms cada, em ciclo: vermelho · verde · azul.

Uso: python3 -I gera_w16_imagens.py [pasta]
"""
import sys
from pathlib import Path

from PIL import Image

out = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent

q = Image.new("RGB", (400, 300))
for (x0, y0), cor in {(0, 0): (255, 0, 0), (200, 0): (0, 160, 0), (0, 150): (0, 0, 255), (200, 150): (0, 0, 0)}.items():
    q.paste(cor, (x0, y0, x0 + 200, y0 + 150))
q.save(out / "w16_quadrantes.png", optimize=False)

quadros = [Image.new("RGB", (120, 120), c).convert("P", palette=Image.Palette.ADAPTIVE, colors=2)
           for c in [(255, 0, 0), (0, 160, 0), (0, 0, 255)]]
quadros[0].save(out / "w16_anim.gif", save_all=True, append_images=quadros[1:], duration=200, loop=0,
                disposal=1, optimize=False)
print("ok", out / "w16_quadrantes.png", out / "w16_anim.gif")
