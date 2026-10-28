"""A marquee ELIPTICA do Aseprite Trial para TODA caixa w x h (1..24), lida do ecra.

uso: PH2D_TRIAL_DISPLAY=:84 python3 -I trial_elipse_grelha.py <dir> [dir_drag ...] [--caixas lista.json]
     dir_drag em {tl, br, tr, bl} = o canto onde se PRIME (o oposto e onde se solta); omissao: tl.
     -> <dir>/grelha.json {"<dir_drag>": {"<w>x<h>": {"at": [x, y], "mask": [linhas '0/1' de h x w]}}}
     (acumulativo; "at" = a celula de cima-esquerda da caixa no canvas)

O Trial ja corre (o lancador esta no fim desta docstring). Mesmos quirks de trial_selecao.py (o rato
perde o ultimo movimento: tres movimentos dentro da celula de destino antes de soltar; Fill magenta = a
mascara). Diferencas: o canvas e GRANDE (red_big.ase, 160 x 100 vermelho) e a zoom 400 % (tecla 3 =
8 px de tela por celula com screen_scale 2), e as elipses vao TODAS no modo Add (botao da barra, nunca o
Shift, que no arrasto restringe a circulo) empacotadas em prateleiras com 4 celulas de folga: cada pagina
e um unico Fill + uma fotografia. O que cai na caixa de cada elipse e so dela porque a marquee eliptica
nunca sai da caixa press..release (medido: Trial contido em Zingl, que esta contido na caixa).

Lancador (Xvfb proprio, HOME isolado; aseprite.ini com [GfxMode] Frame = 0 0 1600 1000, screen_scale 2):
  Xvfb :84 -screen 0 1600x1000x24 -nolisten tcp &
  env -u WAYLAND_DISPLAY DISPLAY=:84 SDL_VIDEODRIVER=x11 HOME=<scratch>/home \
      ~/.local/opt/aseprite-trial/Aseprite-trial.AppImage &
"""
import json, math, os, struct, subprocess, sys, time, zlib
import numpy as np
from PIL import Image

D = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else '.')
ENV = dict(os.environ, DISPLAY=os.environ.get('PH2D_TRIAL_DISPLAY', ':84')); ENV.pop('WAYLAND_DISPLAY', None)
CW, CH = 160, 100          # o canvas
CELL = 8
X0, Y0, W, H = 0, 0, CW, CH
MAG = [255, 0, 255]
MAXS = 24
# folga 4: em volta das alcas da transformacao (cantos e meios da caixa da selecao INTEIRA) ha uma
# zona que ESCALA/RODA/INCLINA a selecao; premir a 15 px dela rodou a pagina (medido 27/10, folgas 1 e 2)
GAP = 4


# ── a entrada (.ase pela spec publica; o escritor de trial_ase_selecao.py) ──────────────────────────
def _string(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def _chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data
def write_red(path, w, h):
    raw = bytes([255, 0, 0, 255]) * (w * h)
    chunks = [_chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0)),
              _chunk(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + _string('src')),
              _chunk(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', w, h)
                     + zlib.compress(raw))]
    fr = b''.join(chunks)
    body = struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), 100, len(chunks)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, 1, w, h, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + body)


# ── a tela ──────────────────────────────────────────────────────────────────────────────────────────
def xdo(*a): subprocess.run(['xdotool', *map(str, a)], env=ENV, check=True)
def key(*k, t=0.25):
    for x in k:
        xdo('key', x); time.sleep(t)
def shot(p='z.png', park=True):
    if park:
        xdo('mousemove', 1590, 990); time.sleep(0.3)
    p = os.path.join(D, p)
    subprocess.run(['import', '-window', 'root', p], env=ENV, check=True)
    return np.asarray(Image.open(p).convert('RGB')).astype(int)

def bbox(a):
    reg = a[100:948, 168:1545]
    bg = a[120, 200]
    m = np.any(reg != bg, axis=2)
    ys, xs = np.nonzero(m)
    return xs.min() + 168, ys.min() + 100, xs.max() + 168, ys.max() + 100

def rebbox(name='bb.png'):
    global X0, Y0, W, H
    x0, y0, x1, y1 = bbox(shot(name))
    X0, Y0 = x0 + 2, y0 + 2
    wpx, hpx = x1 - 2 - X0 + 1, y1 - 4 - Y0 + 1
    W, H = round(wpx / CELL), round(hpx / CELL)
    assert (W, H) == (CW, CH), (x0, y0, x1, y1, W, H)
    return W, H

def cc(i, j):
    return int(X0 + (i + 0.5) * CELL), int(Y0 + (j + 0.5) * CELL)

def _settle(b):
    for d in (-2, 1, 0):
        time.sleep(0.05); xdo('mousemove', b[0] + d, b[1] + d)
    time.sleep(0.15)

def drag(a, b, steps=6):
    # o premir tambem perde o ultimo movimento: premir no fim da elipse ANTERIOR (dentro da selecao)
    # MOVE a selecao (medido 27/10, pagina 'bl'); assenta-se na celula de partida antes do mousedown
    xdo('mousemove', *a); _settle(a); xdo('mousedown', 1); time.sleep(0.08)
    n = max(steps, int(math.hypot(b[0] - a[0], b[1] - a[1]) / 6))
    for s in range(1, n + 1):
        xdo('mousemove', a[0] + (b[0] - a[0]) * s // n, a[1] + (b[1] - a[1]) * s // n); time.sleep(0.01)
    _settle(b)
    xdo('mouseup', 1); time.sleep(0.15)

def cells(a):
    return [[a[int(Y0 + (j + 0.5) * CELL), int(X0 + (i + 0.5) * CELL)].tolist() for i in range(W)]
            for j in range(H)]

def ensure_fg():
    a = shot('fg.png')
    if a[915, 20].tolist() != MAG:
        xdo('mousemove', 64, 915, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(0.8)
        xdo('mousemove', 316, 748, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(0.3)
        key('ctrl+a'); xdo('type', 'ff00ff'); key('Return')
        xdo('mousemove', 588, 740, 'click', 1); time.sleep(0.5)
        assert shot('fg.png')[915, 20].tolist() == MAG

def mask_now(name):
    """Fill magenta, Esc (as alcas da transformacao e as formigas caem sobre centros de celula a 8 px),
    fotografia; o documento fecha-se sem gravar a seguir"""
    key('f', t=0.6); key('Escape', t=0.4)
    a = shot(name + '.png')
    return [[p == MAG for p in row] for row in cells(a)]

def fecha():
    xdo('key', 'ctrl+w'); time.sleep(0.8)
    c = shot('z2.png')
    if tuple(c[437, 640]) == (124, 144, 159) and tuple(c[500, 700]) == (211, 203, 190):
        xdo('mousemove', 799, 546, 'click', 1); time.sleep(0.6)   # «Don't Save»

def abre(nome, zoom='3'):
    xdo('key', 'ctrl+o'); time.sleep(1.2)
    xdo('type', '--delay', 12, os.path.join(D, nome)); time.sleep(0.3)
    xdo('key', 'Return'); time.sleep(1.8)
    if tuple(shot('z0.png')[723, 1300]) == (124, 145, 157):
        xdo('mousemove', 1537, 723, 'click', 1); time.sleep(0.5)   # fecha a janela Preview
    xdo('mousemove', 856, 523); xdo('key', '1'); time.sleep(0.6)
    x0, y0, x1, y1 = bbox(shot('z1.png'))
    xdo('mousemove', (x0 + x1) // 2, (y0 + y1) // 2); xdo('key', zoom); time.sleep(0.8)
    return rebbox('z1b.png')

ADD, REPLACE = (206, 72), (178, 72)


# ── o empacotamento ─────────────────────────────────────────────────────────────────────────────────
def pages(boxes):
    """prateleiras: altura decrescente; GAP celulas de folga; devolve [[(w, h, x, y)]]"""
    todo = sorted(boxes, key=lambda b: (-b[1], -b[0]))
    out, cur, x, y, sh = [], [], 0, 0, 0
    for w, h in todo:
        if x + w > CW:
            x, y, sh = 0, y + sh + GAP, 0
        if y + h > CH:
            out.append(cur); cur, x, y, sh = [], 0, 0, 0
        cur.append((w, h, x, y)); x += w + GAP; sh = max(sh, h)
    if cur:
        out.append(cur)
    return out

CORNER = {'tl': lambda x, y, w, h: ((x, y), (x + w - 1, y + h - 1)),
          'br': lambda x, y, w, h: ((x + w - 1, y + h - 1), (x, y)),
          'tr': lambda x, y, w, h: ((x + w - 1, y), (x, y + h - 1)),
          'bl': lambda x, y, w, h: ((x, y + h - 1), (x + w - 1, y))}


def run_page(page, corner, tag):
    abre('red_big.ase')
    key('Escape'); key('shift+m'); xdo('mousemove', *ADD, 'click', 1); time.sleep(0.3)
    for w, h, x, y in page:
        a, b = CORNER[corner](x, y, w, h)
        drag(cc(*a), cc(*b))
    xdo('mousemove', *REPLACE, 'click', 1); time.sleep(0.3)
    m = mask_now(tag)
    fecha()
    res = [[''.join('1' if m[y + j][x + i] else '0' for i in range(w)) for j in range(h)]
           for w, h, x, y in page]
    # nada fora das caixas (a folga) pode estar seleccionado
    inside = np.zeros((CH, CW), bool)
    for w, h, x, y in page:
        inside[y:y + h, x:x + w] = True
    stray = int((np.array(m) & ~inside).sum())
    assert stray == 0, f'{stray} celulas seleccionadas fora das caixas na pagina {tag}'
    return res


def main():
    args = sys.argv[2:]
    extra = None
    if '--caixas' in args:   # lista JSON [[w, h], ...] no lugar da grelha 1..MAXS (as caixas maiores)
        i = args.index('--caixas'); extra = json.load(open(args[i + 1])); del args[i:i + 2]
    corners = args or ['tl']
    write_red(os.path.join(D, 'red_big.ase'), CW, CH)
    out_p = os.path.join(D, 'grelha.json')
    out = json.load(open(out_p)) if os.path.exists(out_p) else {}
    abre('red_big.ase'); key('m'); ensure_fg(); fecha()   # o fecho do popup pode pintar: documento novo
    boxes = [tuple(b) for b in extra] if extra else [(w, h) for w in range(1, MAXS + 1) for h in range(1, MAXS + 1)]
    for corner in corners:
        got = out.setdefault(corner, {})
        todo = [b for b in boxes if f'{b[0]}x{b[1]}' not in got]
        for k, page in enumerate(pages(todo)):
            t = time.time()
            for (w, h, x, y), m in zip(page, run_page(page, corner, f'p_{corner}_{k}')):
                got[f'{w}x{h}'] = {'at': [x, y], 'mask': m}
            json.dump(out, open(out_p, 'w'))
            print(f'{corner} pagina {k}: {len(page)} caixas, {time.time() - t:.0f}s', flush=True)
    print('FIM')


if __name__ == '__main__':
    main()
