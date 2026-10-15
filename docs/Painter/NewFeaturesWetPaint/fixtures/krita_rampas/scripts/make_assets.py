# Gera os recursos de ENTRADA (nossos) para a corrida do oráculo Krita.
import sys, numpy as np
from PIL import Image
d = sys.argv[1]
# Ponta: 64x64 RGBA, alfa = disco duro r=30, RGB = rampa VERTICAL (escuro em cima -> claro em baixo),
# com leve tinta azul para o Krita a tratar como ponta «colorida».
n = 64; y, x = np.mgrid[0:n, 0:n]
v = (y / (n - 1) * 235).astype(np.uint8)
a = (((x - 31.5) ** 2 + (y - 31.5) ** 2) <= 30 ** 2).astype(np.uint8) * 255
rgba = np.stack([v, v, np.minimum(v.astype(int) + 20, 255).astype(np.uint8), a], -1)
Image.fromarray(rgba, 'RGBA').save(f'{d}/brushes/rampa_vertical_tip.png')
# Padrão (textura): 64x64 cinzento, rampa HORIZONTAL 0..255 (período 64 px em x, constante em y).
p = np.tile((np.arange(64) * 255 / 63).astype(np.uint8), (64, 1))
Image.fromarray(np.stack([p, p, p], -1), 'RGB').save(f'{d}/patterns/rampa_horizontal_64.png')
# Gradiente: vermelho puro -> azul puro, interpolação linear RGB.
open(f'{d}/gradients/RedBlue_oracle.ggr', 'w').write(
    'GIMP Gradient\nName: RedBlue_oracle\n1\n'
    '0.000000 0.500000 1.000000 1.000000 0.000000 0.000000 1.000000 0.000000 0.000000 1.000000 1.000000 0 0 0 0\n')
print('ok')
