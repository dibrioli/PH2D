"""P10 M4/M5: .ase nosso com camada tilemap no Aseprite Trial 1.3.18.6 (Xvfb :79, 1600x1000, screen_scale 2).
uso: python3 -I trial_tiles.py gerar <dir>        escreve <dir>/flips.ase (8 combinacoes X/Y/D) + entradas.json
     python3 -I trial_tiles.py abrir <dir> <f.ase> abre no Trial (HOME isolado) e fotografa <dir>/abrir.png
     python3 -I trial_tiles.py flips <dir>        le a tela de flips.ase e compara as ordens de flip
Formato: spec publica do .ase (chunk 0x2023, layer type 2, cel type 3), sem ler codigo do Aseprite.
O Trial nao grava: tudo se le pela tela."""
import json, os, shutil, struct, subprocess, sys, time, zlib

TW = TH = 4                      # tile 4x4
COLS, ROWS = 4, 2                # mapa 4x2 = 8 celulas = 8 combinacoes
W, H = COLS * TW, ROWS * TH
XF, YF, DF = 0x80000000, 0x40000000, 0x20000000

def tile_pixels():
    # 16 cores todas diferentes, assimetricas: nenhuma simetria do quadrado deixa o tile igual a si mesmo
    return [[(16 * i + 8, 255 - 15 * i, (97 * i + 31) % 256, 255) for x in range(TW) for i in [y * TW + x]] for y in range(TH)]

def string(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data

def gerar(D):
    os.makedirs(D, exist_ok=True)
    tp = tile_pixels()
    empty = bytes(TW * TH * 4)
    t1 = bytes(v for row in tp for p in row for v in p)
    ts = chunk(0x2023, struct.pack('<IIIHHh14x', 0, 2 | 4, 2, TW, TH, 1) + string('ts')
               + struct.pack('<I', len(zlib.compress(empty + t1))) + zlib.compress(empty + t1))
    ly = chunk(0x2004, struct.pack('<HHHHHHB3x', 3, 2, 0, 0, 0, 0, 255) + string('tm') + struct.pack('<I', 0))
    combos = [(fx, fy, fd) for fd in (0, 1) for fy in (0, 1) for fx in (0, 1)]
    tiles = b''
    for fx, fy, fd in combos:
        tiles += struct.pack('<I', 1 | (XF if fx else 0) | (YF if fy else 0) | (DF if fd else 0))
    cel = chunk(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 3, 0)
                + struct.pack('<HHHIIII10x', COLS, ROWS, 32, 0x1fffffff, XF, YF, DF) + zlib.compress(tiles))
    prof = chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0))
    chunks = [ts, ly, cel, prof]   # o tileset ANTES da camada: ao contrario o Trial diz «tileset 0 not found» (medido)
    fh = struct.pack('<IHHH2xI', 16 + sum(map(len, chunks)), 0xF1FA, len(chunks), 100, len(chunks)) + b''.join(chunks)
    hdr = struct.pack('<IHHHHHIHII', 128 + len(fh), 0xA5E0, 1, W, H, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(os.path.join(D, 'flips.ase'), 'wb').write(hdr + fh)
    json.dump({'tw': TW, 'th': TH, 'cols': COLS, 'rows': ROWS, 'combos': combos, 'tile': tp}, open(os.path.join(D, 'entradas.json'), 'w'))

APP = os.path.expanduser('~/.local/opt/aseprite-trial/Aseprite-trial.AppImage')
def env(D):
    e = dict(os.environ, DISPLAY=':79', HOME=os.path.join(D, 'home')); e.pop('WAYLAND_DISPLAY', None)
    return e
def abrir(D, f):
    h = os.path.join(D, 'home', '.config', 'aseprite'); os.makedirs(h, exist_ok=True)
    shutil.copy(os.path.expanduser('~/.config/aseprite/aseprite.ini'), h)
    xv = subprocess.Popen(['Xvfb', ':79', '-screen', '0', '1600x1000x24', '-nolisten', 'tcp'])
    time.sleep(2)
    ap = subprocess.Popen(['timeout', '300', APP, '--appimage-extract-and-run', os.path.join(D, f)], env=env(D),
                          stdout=open(os.path.join(D, 'app.log'), 'w'), stderr=subprocess.STDOUT)
    time.sleep(8)
    subprocess.run(['import', '-window', 'root', os.path.join(D, 'abrir.png')], env=env(D), check=True)
    return xv, ap

if __name__ == '__main__':
    cmd, D = sys.argv[1], os.path.abspath(sys.argv[2])
    if cmd == 'gerar': gerar(D)
    elif cmd == 'abrir':
        xv, ap = abrir(D, sys.argv[3])
        print('foto', os.path.join(D, 'abrir.png'))
        ap.terminate(); xv.terminate()
