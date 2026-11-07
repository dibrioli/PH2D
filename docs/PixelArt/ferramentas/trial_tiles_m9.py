"""P10 M9: casos de borda de tilemap no Aseprite Trial 1.3.18.6 (caixa preta, so corrido; Xvfb proprio, HOME limpo).
Escritor .ase pela spec publica (como trial_tiles.py); leitura por fotografia a 800 % (16 px de tela por pixel).
Biblioteca usada por trial_tiles_m9_corre.py. O Trial nao grava: tudo se le pela tela. Nunca o ecra do dono."""
import os, shutil, struct, subprocess, time, zlib
import numpy as np
from PIL import Image

APP = os.path.expanduser('~/.local/opt/aseprite-trial/Aseprite-trial.AppImage')
T = 8
COL = {'.': (0, 0, 0, 0), 'R': (255, 0, 0, 255), 'G': (0, 255, 0, 255), 'B': (0, 0, 255, 255)}

def _s(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def _c(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data

def tile_row(y, x0, x1, ch):
    return [''.join(ch if (j == y and x0 <= i <= x1) else '.' for i in range(T)) for j in range(T)]

def write_ase(path, tiles, grid, w=32, h=32, base_layer=False, grid2=None):
    """tiles = lista de bitmaps T x T (sem o tile 0, que e o vazio); grid = {(cx,cy): id}. Camadas: [pixels?] + tilemap."""
    raw = bytes(T * T * 4)
    for im in tiles:
        raw += bytes(v for row in im for ch in row for v in COL[ch])
    z = zlib.compress(raw)
    ts = _c(0x2023, struct.pack('<IIIHHh14x', 0, 2 | 4, len(tiles) + 1, T, T, 1) + _s('ts') + struct.pack('<I', len(z)) + z)
    chunks = [_c(0x2007, struct.pack('<HHI8x', 1, 0, 0)), ts]
    li = 0
    if base_layer:
        chunks.append(_c(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + _s('base')))
        rawb = bytes(4) * (w * h)
        chunks.append(_c(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', w, h) + zlib.compress(rawb)))
        li = 1
    chunks.append(_c(0x2004, struct.pack('<HHHHHHB3x', 3, 2, 0, 0, 0, 0, 255) + _s('tm') + struct.pack('<I', 0)))
    cols, rows = w // T, h // T
    data = b''.join(struct.pack('<I', grid.get((x, y), 0)) for y in range(rows) for x in range(cols))
    chunks.append(_c(0x2005, struct.pack('<HhhBHh5x', li, 0, 0, 255, 3, 0)
                     + struct.pack('<HHHIIII10x', cols, rows, 32, 0x1fffffff, 0x80000000, 0x40000000, 0x20000000) + zlib.compress(data)))
    if grid2 is not None:
        chunks.append(_c(0x2004, struct.pack('<HHHHHHB3x', 3, 2, 0, 0, 0, 0, 255) + _s('tm2') + struct.pack('<I', 0)))
        data2 = b''.join(struct.pack('<I', grid2.get((x, y), 0)) for y in range(rows) for x in range(cols))
        chunks.append(_c(0x2005, struct.pack('<HhhBHh5x', li + 1, 0, 0, 255, 3, 0)
                         + struct.pack('<HHHIIII10x', cols, rows, 32, 0x1fffffff, 0x80000000, 0x40000000, 0x20000000) + zlib.compress(data2)))
    fr = b''.join(chunks)
    body = struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), 100, len(chunks)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, 1, w, h, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + body)

class S:
    """Uma sessao: Xvfb + HOME limpo + Trial com um ficheiro."""
    def __init__(self, work, disp, f):
        self.w = work; self.d = f':{disp}'
        os.makedirs(work, exist_ok=True)
        self.home = os.path.join(work, 'home')
        shutil.rmtree(self.home, ignore_errors=True)
        os.makedirs(self.home + '/.config/aseprite')
        open(self.home + '/.config/aseprite/aseprite.ini', 'w').write('[GfxMode]\nFrame = 0 0 1600 1000\n[general]\nscreen_scale = 2\nui_scale = 1\n')
        self.env = dict(os.environ, DISPLAY=self.d, HOME=self.home, SDL_VIDEODRIVER='x11'); self.env.pop('WAYLAND_DISPLAY', None)
        self.xv = subprocess.Popen(['Xvfb', self.d, '-screen', '0', '1600x1000x24', '-nolisten', 'tcp'], stderr=subprocess.DEVNULL)
        time.sleep(1.5)
        self.ap = subprocess.Popen(['timeout', '280', APP, f], env=self.env, stdout=open(work + '/app.log', 'w'), stderr=subprocess.STDOUT)
        time.sleep(float(os.environ.get('M9_BOOT', '7')))
    def x(self, *a):
        subprocess.run(['xdotool', *map(str, a)], env=self.env, check=True)
    def key(self, *k, t=0.3):
        for q in k:
            self.x('key', q); time.sleep(t)
    def space(self, k):
        self.x('keydown', 'space'); time.sleep(.1); self.x('key', k); time.sleep(.1); self.x('keyup', 'space'); time.sleep(.3)
    def shot(self, name, park=True):
        if park:
            self.x('mousemove', 1590, 990); time.sleep(.3)
        p = os.path.join(self.w, name)
        subprocess.run(['import', '-window', 'root', p], env=self.env, check=True)
        return np.asarray(Image.open(p).convert('RGB')).astype(int)
    def drag(self, a, b, step=8):
        self.x('mousemove', *a); time.sleep(.15); self.x('mousemove', a[0] + 1, a[1]); time.sleep(.1); self.x('mousemove', *a); time.sleep(.15)
        cmd = ['mousedown', '1']
        n = max(1, max(abs(b[0] - a[0]), abs(b[1] - a[1])) // step)
        for s in range(1, n + 1):
            cmd += ['sleep', '0.02', 'mousemove', a[0] + (b[0] - a[0]) * s // n, a[1] + (b[1] - a[1]) * s // n]
        # o Trial perde o ultimo movimento antes do largar (medido: ficava 1 pixel por apagar): assenta 2x dentro do pixel
        cmd += ['sleep', '0.08', 'mousemove', b[0] + 1, b[1], 'sleep', '0.08', 'mousemove', b[0], b[1], 'sleep', '0.08', 'mouseup', '1']
        self.x(*cmd); time.sleep(.35)
    def fim(self):
        self.ap.terminate(); time.sleep(.2); self.xv.terminate()

# leitura (1600x1000, screen_scale 2, zoom 400 % = 8 px/pixel)
BOT = {'manual': (56, 107), 'auto': (85, 107), 'stack': (113, 107)}
def cls(c):
    r, g, b = c
    if r > 200 and g < 80 and b < 80: return 'R'
    if g > 200 and r < 80 and b < 80: return 'G'
    if b > 200 and r < 80 and g < 80: return 'B'
    if r > 240 and g > 240 and b > 240: return 'W'
    return '.'
def canvas(a, x0, y0, w=32, h=32, px=8):
    return [''.join(cls(a[y0 + j * px + px // 2 - 1, x0 + i * px + px // 2 - 1]) for i in range(w)) for j in range(h)]
TX, TY, TS = 12, 444, 34        # o painel de tiles em baixo: 3 por linha, 32 px cada, 4 px por pixel (foto)
def tiles_panel(a, per_row=3, maxn=12):
    bg = a[600, 80]
    out = []
    for n in range(maxn):
        tx = TX + TS * (n % per_row); ty = TY + TS * (n // per_row)
        reg = a[ty + 2:ty + 30, tx + 2:tx + 30]
        if (reg == bg).all(axis=2).mean() > .6: break
        if ty + 32 > 730: break
        # o «II» do fim da lista
        if (reg == bg).all(axis=2).mean() > .3: break
        out.append([''.join(cls(a[ty + j * 4 + 1, tx + i * 4 + 1]) for i in range(T)) for j in range(T)])
    return out
def cells_of(cv, tset):
    res = {}
    for cy in range(len(cv) // T):
        for cx in range(len(cv[0]) // T):
            bm = [r[cx * T:cx * T + T] for r in cv[cy * T:cy * T + T]]
            ids = [i for i, t in enumerate(tset) if t == bm]
            if not any(c != '.' for r in bm for c in r) and (0 in ids or not ids):
                continue
            res[f'{cx},{cy}'] = ids if ids else ['?'] + [''.join(bm)]
    return res
