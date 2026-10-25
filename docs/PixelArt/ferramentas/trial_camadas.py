"""Driver: abre cada .ase de trial_ase_camadas.py no Aseprite Trial (Xvfb :78, janela 1600x1000), a
800 %, e le o composto da tela (o centro de cada celula). Grava <dir>/lido.json {ficheiro: [[r,g,b]...]}.
uso: python3 -I trial_camadas.py <dir> [ficheiro ...]
Controlo: o ficheiro m00_o255 (Normal, 255) tem de bater com o source-over exacto calculado aqui
das entradas; o driver para se nao bater (a tela estaria a mentir)."""
import json, os, subprocess, sys, time
import numpy as np
from PIL import Image

D = sys.argv[1]
ENV = dict(os.environ, DISPLAY=':78'); ENV.pop('WAYLAND_DISPLAY', None)
E = json.load(open(os.path.join(D, 'entradas.json')))
W, H = E['w'], E['h']
FILES = sys.argv[2:] or E['files']
BG = None

def xdo(*a): subprocess.run(['xdotool', *map(str, a)], env=ENV, check=True)
def shot(p):
    xdo('mousemove', 1590, 990); time.sleep(0.3)
    subprocess.run(['import', '-window', 'root', p], env=ENV, check=True)
    return np.asarray(Image.open(p).convert('RGB')).astype(int)

def bbox(a):
    # area do canvas: dentro da moldura do editor; o fundo do editor e a cor de (200, 120)
    reg = a[100:948, 168:1545]
    bg = a[120, 200]
    m = np.any(reg != bg, axis=2)
    ys, xs = np.nonzero(m)
    return xs.min() + 168, ys.min() + 100, xs.max() + 168, ys.max() + 100

def ler(nome):
    xdo('key', 'ctrl+o'); time.sleep(1.2)
    xdo('type', '--delay', 12, os.path.join(D, nome)); time.sleep(0.3)
    xdo('key', 'Return'); time.sleep(1.8)
    xdo('mousemove', 856, 523); xdo('key', '1'); time.sleep(0.6)
    x0, y0, x1, y1 = bbox(shot(os.path.join(D, 'z1.png')))
    # escala 2 da interface: 400 % = 8 px de tela por celula
    xdo('mousemove', (x0 + x1) // 2, (y0 + y1) // 2); xdo('key', '3'); time.sleep(0.8)
    p = os.path.join(D, nome.replace('.ase', '.png'))
    a = shot(p)
    x0, y0, x1, y1 = bbox(a)
    x0, y0, x1, y1 = x0 + 2, y0 + 2, x1 - 2, y1 - 2   # a borda escura de 1 px (2 de tela)
    sx, sy = (x1 - x0 + 1) / W, (y1 - y0 + 1) / H
    # em baixo a sombra da moldura soma mais 2 px: a escala le-se na largura
    assert abs(sx - 8) < 0.01 and 8 <= sy < 8.1, (nome, x0, y0, x1, y1, sx, sy)
    cells = [a[int(y0 + (j + 0.5) * 8), int(x0 + (i + 0.5) * 8)].tolist() for j in range(H) for i in range(W)]
    xdo('key', 'ctrl+w'); time.sleep(0.7)
    c = shot(os.path.join(D, 'z2.png'))
    if tuple(c[437, 640]) == (124, 144, 159) and tuple(c[500, 700]) == (211, 203, 190):
        xdo('mousemove', 799, 546, 'click', 1); time.sleep(0.6)   # «Don't Save»
    return cells

def over(d, s):
    sa = s[3]
    if sa == 255: return s[:3]
    return [(s[k] * sa + d[k] * (255 - sa) + 127) // 255 for k in range(3)]

out_p = os.path.join(D, 'lido.json')
out = json.load(open(out_p)) if os.path.exists(out_p) else {}
ctl = ler('m00_o255.ase')
want = [over(b, s) for b, s in zip(E['back'], E['src'])]
bad = sum(any(abs(x - y) > 1 for x, y in zip(c, w)) for c, w in zip(ctl, want))
exact = sum(c == w for c, w in zip(ctl, want))
print(f'controlo Normal: {exact}/{len(want)} exactos, {bad} com |d|>1', flush=True)
assert bad == 0, 'a tela nao mostra o composto: parar'
for n in FILES:
    out[n] = ler(n)
    print(n, flush=True)
    json.dump(out, open(out_p, 'w'))
print('FIM')
