#!/usr/bin/env python3
"""prancha_manchas.py <saida.png> <rótulo=arquivo.ppm> ...: cada célula = o traço inteiro + o miolo a 3× + o miolo a 3× com contraste ×4."""
import sys
import numpy as np
from PIL import Image, ImageDraw
def f(n):
    from PIL import ImageFont
    import glob
    p = glob.glob("/usr/share/fonts/**/DejaVuSans.ttf", recursive=True)
    return ImageFont.truetype(p[0], n) if p else ImageFont.load_default()
BOX = (180, 110, 340, 230)
Z = 2
saida, itens = sys.argv[1], [a.split("=", 1) for a in sys.argv[2:]]
CW, CH = 520 + (BOX[2] - BOX[0]) * Z + 10, 360 + 22
o = Image.new("RGB", (2 * CW + 20, (len(itens) + 1) // 2 * CH), (245, 245, 245))
D = ImageDraw.Draw(o)
for i, (rot, arq) in enumerate(itens):
    x, y = (i % 2) * (CW + 10) + 4, i // 2 * CH
    im = Image.open(arq).convert("RGB")
    D.text((x, y + 2), rot, fill=(20, 20, 120), font=f(14))
    o.paste(im, (x, y + 20))
    c = im.crop(BOX)
    a = np.asarray(c, float)
    t = a.sum(2) < 560
    m = np.median(a[t], axis=0) if t.any() else np.median(a, axis=(0, 1))
    b = Image.fromarray(np.clip((a - m) * 4 + 128, 0, 255).astype("uint8"))
    tam = ((BOX[2] - BOX[0]) * Z, (BOX[3] - BOX[1]) * Z // 1)
    x2 = x + 526
    o.paste(c.resize((tam[0], 170), Image.NEAREST), (x2, y + 20))
    o.paste(b.resize((tam[0], 170), Image.NEAREST), (x2, y + 194))
o.save(saida)
print(o.size)
