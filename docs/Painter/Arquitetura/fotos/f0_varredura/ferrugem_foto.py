"""ferrugem_foto.py <saida.png> <rotulo>=<png> ... — por célula: o recorte, o mapa da ferrugem (vermelho) e o crop 4x da maior concentração."""
import sys, colorsys
from PIL import Image, ImageDraw
out = sys.argv[1]; itens = [a.split('=', 1) for a in sys.argv[2:]]
def fer(c):
    r, g, b = [v / 255 for v in c]
    h, s, v = colorsys.rgb_to_hsv(r, g, b); h *= 360
    return s > 0.2 and (h < 40 or h > 250)
linhas = []
for rot, f in itens:
    im = Image.open(f).convert('RGB'); w, h = im.size; px = im.load()
    m = im.copy(); mp = m.load(); pts = []
    for y in range(h):
        for x in range(w):
            if fer(px[x, y]): mp[x, y] = (255, 0, 0); pts.append((x, y))
    if pts:
        cx = sorted(p[0] for p in pts)[len(pts)//2]; cy = sorted(p[1] for p in pts)[len(pts)//2]
    else:
        cx, cy = w // 2, h // 2
    x0, y0 = max(0, min(w - 40, cx - 20)), max(0, min(h - 30, cy - 15))
    crop = im.crop((x0, y0, x0 + 40, y0 + 30)).resize((160, 120), Image.NEAREST)
    linhas.append((rot, im, m, crop, len(pts)))
W = 300 * 2 + 160 + 20; H = 135
img = Image.new('RGB', (W, H * len(linhas)), 'white'); d = ImageDraw.Draw(img)
for i, (rot, im, m, crop, n) in enumerate(linhas):
    y = i * H
    d.text((2, y), f"{rot}  | recorte | ferrugem em vermelho ({n} px) | crop 4x", fill='black')
    img.paste(im.resize((300, 130 * 300 // 300 - 10)) if False else im, (0, y + 12)) if im.size[0] == 300 else img.paste(im.resize((300, 118)), (0, y + 12))
    img.paste(m.resize((300, 118)), (305, y + 12)); img.paste(crop.resize((160, 118)), (610, y + 12))
img.save(out)
