"""P10 M4: le z4.png (flips.ase a 800 %, 16 px de tela por pixel) e diz que ordem de flips o Trial aplica.
uso: python3 -I trial_tiles_le.py <dir> <x0> <y0>   (x0,y0 = canto sup. esq. do canvas na foto)"""
import json, sys
import numpy as np
from PIL import Image
D, X0, Y0 = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
E = json.load(open(D + '/entradas.json')); TW = E['tw']; TH = E['th']
a = np.asarray(Image.open(D + '/z4.png').convert('RGB')).astype(int)
tile = [[tuple(E['tile'][y][x][:3]) for x in range(TW)] for y in range(TH)]
idx = {tile[y][x]: (x, y) for y in range(TH) for x in range(TW)}
def pix(cx, cy, i, j):
    return tuple(a[Y0 + (cy * TH + j) * 16 + 8, X0 + (cx * TW + i) * 16 + 8])
def modelo(ordem, fx, fy, fd):
    # devolve para cada (x,y) de destino a coordenada de origem
    out = {}
    for y in range(TH):
        for x in range(TW):
            sx, sy = x, y
            if ordem == 'D,X,Y':            # D troca primeiro (no destino->origem a ordem inverte-se: ver abaixo)
                pass
            out[(x, y)] = None
    return out
def aplicar(ordem, fx, fy, fd, x, y):
    """Coordenada de ORIGEM do pixel de destino (x,y): percorre a cadeia de operacoes sobre o tile."""
    # imagem(destino) = op_n(...op_1(tile)); calculamos para cada pixel de origem onde cai e invertemos
    for sx in range(TW):
        for sy in range(TH):
            px, py = sx, sy
            for op in ordem:
                if op == 'D' and fd: px, py = py, px
                if op == 'X' and fx: px = TW - 1 - px
                if op == 'Y' and fy: py = TH - 1 - py
            if (px, py) == (x, y): return (sx, sy)
ordens = ['DXY', 'DYX', 'XYD', 'YXD', 'XDY', 'YDX']
res = {}
for n, (fx, fy, fd) in enumerate(E['combos']):
    cx, cy = n % E['cols'], n // E['cols']
    lido = {}
    for j in range(TH):
        for i in range(TW):
            c = pix(cx, cy, i, j)
            lido[(i, j)] = idx.get(c)
    ok = [o for o in ordens if all(aplicar(o, fx, fy, fd, i, j) == lido[(i, j)] for i in range(TW) for j in range(TH))]
    nul = sum(v is None for v in lido.values())
    res[n] = {'fx': fx, 'fy': fy, 'd': fd, 'ordens_que_batem': ok, 'pixels_fora_da_paleta': nul,
              'lido_origem_por_destino': {f'{i},{j}': lido[(i, j)] for j in range(TH) for i in range(TW)}}
    print(n, (fx, fy, fd), 'bate com', ok, 'fora da paleta', nul)
json.dump(res, open(D + '/flips_lido.json', 'w'))
