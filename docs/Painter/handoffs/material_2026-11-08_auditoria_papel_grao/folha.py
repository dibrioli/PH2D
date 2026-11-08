"""Folha da auditoria Paper/Grain/Shape Color/Grain Colors × Wet blend (08/11).
Uso: python3 -I folha.py <pasta rgba> <saida.png> [variante ...]  — uma linha por pincel, uma coluna por variante,
cada célula o RGBA 600×300 sobre xadrez, a metade."""
import sys, os
from PIL import Image, ImageDraw

W, H = 600, 300
src, out = sys.argv[1], sys.argv[2]
vars_ = sys.argv[3:] or ["off", "base", "grao_off", "grao", "grao_cor_off", "grao_cor", "shape_cor_off", "shape_cor", "papel_off", "papel"]
nomes = sorted({f.split("__")[0] for f in os.listdir(src) if f.endswith(".rgba")})
s = 2
cw, ch = W // s, H // s
folha = Image.new("RGB", (140 + cw * len(vars_), 20 + ch * len(nomes)), (40, 40, 40))
d = ImageDraw.Draw(folha)
xad = Image.new("RGB", (W, H), (200, 200, 200))
dx = ImageDraw.Draw(xad)
for y in range(0, H, 10):
    for x in range(0, W, 10):
        if (x // 10 + y // 10) % 2:
            dx.rectangle([x, y, x + 9, y + 9], fill=(160, 160, 160))
for j, v in enumerate(vars_):
    d.text((140 + j * cw + 4, 4), v, fill=(255, 255, 255))
for i, n in enumerate(nomes):
    d.text((4, 20 + i * ch + ch // 2), n, fill=(255, 255, 255))
    for j, v in enumerate(vars_):
        p = os.path.join(src, f"{n}__{v}.rgba")
        if not os.path.exists(p):
            continue
        im = Image.frombytes("RGBA", (W, H), open(p, "rb").read())
        c = xad.copy()
        c.paste(im, (0, 0), im)
        folha.paste(c.resize((cw, ch)), (140 + j * cw, 20 + i * ch))
folha.save(out)
