import sys, os
sys.path.insert(0, os.environ['M'])
import numpy as np
from PIL import Image, ImageDraw
from carrega import ph2d
S = sys.argv[1]; out = sys.argv[2]; vs = sys.argv[3:]
# recorte: a rampa (traço 1, y 60..160) entre p ~0,3 e ~0,8, coluna a (papel vazio)
y0, y1, x0, x1 = 70, 150, 230, 570
tiles = []
for v in vs:
    row = []
    for n in range(1, 6):
        x = '' if n == 1 else f'__x{n}'
        rgb, _ = ph2d(S + '/cena', f'__{v}__rampa{x}')
        row.append(np.clip(rgb[y0:y1, x0:x1], 0, 255).astype(np.uint8))
    tiles.append(np.concatenate([np.pad(t, ((4, 4), (4, 4), (0, 0)), constant_values=255) for t in row], 1))
im = Image.fromarray(np.concatenate(tiles, 0))
d = ImageDraw.Draw(im)
for i, v in enumerate(vs):
    d.text((8, i * (y1 - y0 + 8) + 6), v, fill=(200, 0, 0))
for n in range(5):
    d.text((n * (x1 - x0 + 8) + (x1 - x0) // 2, 0), f'{n + 1}x', fill=(0, 0, 200))
im = im.resize((im.width * 2, im.height * 2), Image.NEAREST)
im.save(out)
print(im.size)
