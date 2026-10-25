"""Lê os pixels de um sprite fotografado no Trial: le_tela.py foto.png W H [ref.png] -> JSON."""
import json, sys
from PIL import Image
foto, W, H = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
im = Image.open(foto).convert('RGB')
bg = im.getpixel((300, 200))
xs, ys = [], []
for y in range(165, 880):
    for x in range(232, 1480):
        if x > 1115 and y > 675:
            continue
        if im.getpixel((x, y)) != bg:
            xs.append(x); ys.append(y)
x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
# a borda escura de 1 px à volta do sprite
s = (x1 - x0 - 1) / W
cells = []
for j in range(H):
    for i in range(W):
        cx = int(x0 + 1 + (i + 0.5) * s); cy = int(y0 + 1 + (j + 0.5) * s)
        cells.append(list(im.getpixel((cx, cy))))
out = {'bbox': [x0, y0, x1, y1], 'scale': s, 'cells': cells}
if len(sys.argv) > 4:
    ref = [list(p[:3]) for p in Image.open(sys.argv[4]).convert('RGBA').get_flattened_data()]
    out['control_mismatch'] = sum(a != b for a, b in zip(ref, cells))
print(json.dumps(out))
