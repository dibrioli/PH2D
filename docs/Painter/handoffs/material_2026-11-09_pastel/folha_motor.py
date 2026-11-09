# Rodada 3 do Pastel (09/11): a folha lado a lado — nosso hoje · Krita Chalk Soft · as variantes pedidas.
# Uso: python3 -I folha_motor.py <scratch> <saída.png> <variante>[=rótulo] ...
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from carrega import COL, base_rgb, compoe_camada, ph2d  # noqa: E402
from krita_rampa import k_krita  # noqa: E402

S, OUT, PEDIDAS = sys.argv[1], sys.argv[2], sys.argv[3:]
Y0, Y1 = 60, 160
fonte = ImageFont.truetype('/usr/share/fonts/TTF/DejaVuSans.ttf', 13)
fonte_g = ImageFont.truetype('/usr/share/fonts/TTF/DejaVuSans-Bold.ttf', 15)


def cel(rgb):
    return Image.fromarray(rgb.clip(0, 255).astype(np.uint8)).resize((400, 440), Image.BICUBIC)


def sobre_branco(k):
    return compoe_camada(k, np.full(k.shape + (3,), 255, np.float32))


def linha_ph2d(suf):
    rgb, _ = ph2d(S + '/cena', suf)
    _, a = ph2d(S + '/cena', suf + '__rampa')
    _, b = ph2d(S + '/cena', suf + '__rampa__x2')
    return [rgb[:, :COL], rgb[:, COL:2 * COL], rgb[:, 2 * COL:], sobre_branco(a), sobre_branco(b), rgb[Y0:Y1, 250:550]]


base = base_rgb(S + '/cena')
kb = k_krita(f'{S}/krita_cru/h__Chalk_Soft_v3.png')
krgb = compoe_camada(np.concatenate([kb] * 3, 1), base)
L = [('PH2D hoje', 'o Pastel de fábrica (spray)', linha_ph2d('__hoje')),
     ('Krita Chalk Soft', 'o mais realista dos oráculos (camada própria)',
      [krgb[:, :COL], krgb[:, COL:2 * COL], krgb[:, 2 * COL:],
       sobre_branco(k_krita(f'{S}/krita_rampa/h__Chalk_Soft_rampa.png')),
       sobre_branco(k_krita(f'{S}/krita_rampa_x2/h__Chalk_Soft_rampa_x2.png')), krgb[Y0:Y1, 250:550]])]
for p in PEDIDAS:
    v, _, rot = p.partition('=')
    L.append((v, rot or v, linha_ph2d(f'__{v}')))
COLS = ('camada vazia · p 1', 'branco · p 1', 'blocos · p 1', 'pressão 0,1→1', '2 passadas', 'detalhe 200 % (p 1)')
LW, CW, DW, TOPO = 230, 400, 600, 22
lin_h = 440 + TOPO + 14
fol = Image.new('RGB', (LW + 5 * (CW + 8) + DW + 10, 40 + lin_h * len(L)), (36, 36, 40))
d = ImageDraw.Draw(fol)
d.text((10, 10), 'Pastel — rodadas 3–5 (09/11): grão médio de borda macia · borda quebrada pelo papel (motor novo)', font=fonte_g,
       fill=(255, 255, 255))
for i, (rot, sub, cels) in enumerate(L):
    y = 40 + i * lin_h
    d.text((10, y + TOPO + 10), rot, font=fonte_g, fill=(255, 255, 255))
    yy = y + TOPO + 34
    for j in range(0, len(sub), 28):
        d.text((10, yy), sub[j:j + 28], font=fonte, fill=(200, 200, 200))
        yy += 17
    for c, im in enumerate(cels):
        x = LW + c * (CW + 8)
        if c < 5:
            fol.paste(cel(im), (x, y + TOPO))
        else:
            fol.paste(Image.fromarray(im.clip(0, 255).astype(np.uint8)).resize((DW, 200), Image.NEAREST), (x, y + TOPO))
        d.text((x, y + 4), COLS[c], font=fonte, fill=(170, 200, 255))
fol.save(OUT, optimize=True)
print('folha ok', OUT)
