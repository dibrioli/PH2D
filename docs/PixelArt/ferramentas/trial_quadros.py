"""Driver da P4: abre cada .ase de trial_ase_quadros.py no Aseprite Trial (Xvfb :78) e le a tela.
uso: python3 -I trial_quadros.py <dir> [ficheiro ...]
  co_*, z_*  o composto do quadro 0 (centro de cada celula a 400 % = 8 px de tela);
  on_*       o quadro 2 com o papel vegetal desligado (o xadrez) e ligado (F3), mais o .ini do teste;
  tg_*       Enter (tocar) a partir do quadro 1 e um video de 14 s do centro do canvas (ffmpeg
             x11grab, 50 q/s) -> a sequencia de cores com a duracao de cada uma.
Grava <dir>/lido.json."""
import json, os, subprocess, sys, time
import numpy as np
from PIL import Image

D = sys.argv[1]
ENV = dict(os.environ, DISPLAY=':78'); ENV.pop('WAYLAND_DISPLAY', None)
E = json.load(open(os.path.join(D, 'entradas.json')))
W, H = E['w'], E['h']
FILES = sys.argv[2:] or E['files']

def xdo(*a): subprocess.run(['xdotool', *map(str, a)], env=ENV, check=True)
def shot(p):
    xdo('mousemove', 1590, 990); time.sleep(0.3)
    subprocess.run(['import', '-window', 'root', p], env=ENV, check=True)
    return np.asarray(Image.open(p).convert('RGB')).astype(int)

def bbox(a):
    reg = a[100:948, 168:1545]
    bg = a[120, 200]
    m = np.any(reg != bg, axis=2)
    ys, xs = np.nonzero(m)
    return xs.min() + 168, ys.min() + 100, xs.max() + 168, ys.max() + 100

def abre(nome):
    xdo('key', 'ctrl+o'); time.sleep(1.2)
    xdo('type', '--delay', 12, os.path.join(D, nome)); time.sleep(0.3)
    xdo('key', 'Return'); time.sleep(1.8)
    if tuple(shot(os.path.join(D, 'z0.png'))[723, 1300]) == (124, 145, 157):
        xdo('mousemove', 1537, 723, 'click', 1); time.sleep(0.5)   # fecha a janela Preview
    xdo('mousemove', 856, 523); xdo('key', '1'); time.sleep(0.6)
    x0, y0, x1, y1 = bbox(shot(os.path.join(D, 'z1.png')))
    xdo('mousemove', (x0 + x1) // 2, (y0 + y1) // 2); xdo('key', '3'); time.sleep(0.8)

def celulas(a, x0y0):
    x0, y0 = x0y0
    return [a[int(y0 + (j + 0.5) * 8), int(x0 + (i + 0.5) * 8)].tolist() for j in range(H) for i in range(W)]

def origem(a, nome):
    x0, y0, x1, y1 = bbox(a)
    x0, y0, x1, y1 = x0 + 2, y0 + 2, x1 - 2, y1 - 2
    sx, sy = (x1 - x0 + 1) / W, (y1 - y0 + 1) / H
    assert abs(sx - 8) < 0.01 and 8 <= sy < 8.1, (nome, x0, y0, x1, y1, sx, sy)
    return x0, y0

def fecha():
    xdo('key', 'ctrl+w'); time.sleep(0.7)
    c = shot(os.path.join(D, 'z2.png'))
    if tuple(c[437, 640]) == (124, 144, 159) and tuple(c[500, 700]) == (211, 203, 190):
        xdo('mousemove', 799, 546, 'click', 1); time.sleep(0.6)   # «Don't Save»

def ler(nome):
    abre(nome)
    a = shot(os.path.join(D, nome.replace('.ase', '.png')))
    out = celulas(a, origem(a, nome))
    fecha()
    return out

def ler_onion(nome):
    abre(nome)
    a = shot(os.path.join(D, nome.replace('.ase', '_f0.png')))
    o = origem(a, nome)
    xdo('key', 'Home'); xdo('key', 'Right'); xdo('key', 'Right'); time.sleep(0.6)
    sem = celulas(shot(os.path.join(D, nome.replace('.ase', '_off.png'))), o)
    xdo('key', 'F3'); time.sleep(0.8)
    com = celulas(shot(os.path.join(D, nome.replace('.ase', '_on.png'))), o)
    xdo('key', 'F3'); time.sleep(0.3)
    fecha()
    return {'off': sem, 'on': com}

def ler_tocar(nome):
    abre(nome)
    a = shot(os.path.join(D, nome.replace('.ase', '_f0.png')))
    x0, y0 = origem(a, nome)
    xdo('key', 'Home'); time.sleep(0.3)
    cx, cy = x0 + W * 4, y0 + H * 4
    if nome != 'tg_dur.ase':
        xdo('key', 'Home'); xdo('key', 'Right'); time.sleep(0.5)
    xdo('mousemove', 1590, 990)
    mp4 = os.path.join(D, nome.replace('.ase', '.mkv'))
    rec = subprocess.Popen(['ffmpeg', '-loglevel', 'error', '-y', '-f', 'x11grab', '-framerate', '50',
                            '-video_size', '8x8', '-i', f':78+{cx},{cy}', '-t', '14', '-c:v', 'ffv1', mp4], env=ENV)
    time.sleep(1.0)
    xdo('key', 'Return')
    rec.wait()
    xdo('key', 'Return'); time.sleep(0.4)
    fecha()
    raw = subprocess.run(['ffmpeg', '-loglevel', 'error', '-i', mp4, '-vf', 'crop=1:1:4:4', '-f', 'rawvideo',
                          '-pix_fmt', 'rgb24', '-'], capture_output=True, check=True).stdout
    px = [list(raw[i:i + 3]) for i in range(0, len(raw), 3)]
    runs = []
    for p in px:
        if runs and runs[-1][0] == p:
            runs[-1][1] += 1
        else:
            runs.append([p, 1])
    return {'fps': 50, 'runs': runs}

out_p = os.path.join(D, 'lido.json')
out = json.load(open(out_p)) if os.path.exists(out_p) else {}
for n in FILES:
    if n.startswith('on_'):
        out[n] = ler_onion(n)
    elif n.startswith('tg_'):
        out[n] = ler_tocar(n)
    else:
        out[n] = ler(n)
    print(n, flush=True)
    json.dump(out, open(out_p, 'w'))
print('FIM')
