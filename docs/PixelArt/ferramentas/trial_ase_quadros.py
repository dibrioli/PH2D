"""Escreve os .ase de ENTRADA da onda P4 (quadros, cels, papel vegetal, tags) para o Aseprite Trial.

uso: python3 -I trial_ase_quadros.py <dir>
Spec publica do .ase (docs/ase-file-specs.md do Aseprite), sem ler codigo do Aseprite.
  co_l{LLL}_c{CCC}.ase   opacidade do CEL x a da CAMADA: de baixo opaca, fonte com camada LLL e cel CCC
  z_{nome}.ase           z-index do cel: de baixo opaca + A, B, C (RGBA sorteado, Normal) com os z do nome
  on_{nome}.ase          papel vegetal: 5 quadros, o activo (2) vazio ou com faixa cinza opaca no topo;
                         os quadros 0, 1, 3, 4 pintam cada um a sua quarta parte (metade de cima alfa 255,
                         de baixo alfa 128)
  tg_{dir}_r{R}.ase      tocar: 6 quadros de cor solida (250 ms), tag 1..4 com a direccao e a repeticao
  tg_dur.ase             tocar sem tag: duracoes 100/300/600/200 ms
O estado sorteado grava-se em <dir>/entradas.json."""
import json, os, struct, sys, zlib

D = sys.argv[1]
os.makedirs(D, exist_ok=True)
W, H = 96, 64
N = W * H

s = 0x6A09E667
def rnd():
    global s
    s ^= (s << 13) & 0xFFFFFFFF; s ^= s >> 17; s ^= (s << 5) & 0xFFFFFFFF
    return s
ALFAS = [0, 1, 32, 64, 100, 127, 128, 160, 200, 254, 255, 255, 255, 255]
def cor():
    return [rnd() % 256 if rnd() % 2 else [0, 1, 64, 127, 128, 129, 192, 254, 255][rnd() % 9] for _ in range(3)]
BACK = [cor() + [255] for _ in range(N)]
SRC = {k: [cor() + [ALFAS[rnd() % len(ALFAS)] if rnd() % 3 else rnd() % 256] for _ in range(N)] for k in 'ABC'}

def string(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data
def layer(name, mode=0, opacity=255):
    return chunk(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, mode, opacity) + string(name))
def cel(index, px, opacity=255, z=0, w=W, h=H, x=0, y=0):
    raw = bytes(v for p in px for v in p[:4])
    return chunk(0x2005, struct.pack('<HhhBHh5x', index, x, y, opacity, 2, z) + struct.pack('<HH', w, h) + zlib.compress(raw))
def profile():
    return chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0))
def tags(lst):
    body = struct.pack('<H8x', len(lst))
    for a, b, d, r, name in lst:
        body += struct.pack('<HHBH6x3Bx', a, b, d, r, 0, 0, 0) + string(name)
    return chunk(0x2018, body)
def write(name, frames):
    """frames = [(duration_ms, [chunks])]"""
    body = b''
    for ms, chunks in frames:
        fr = b''.join(chunks)
        body += struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), ms, len(chunks)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, len(frames), W, H, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    with open(os.path.join(D, name), 'wb') as f:
        f.write(hdr + body)

ARQS = []
# ── opacidade do cel x da camada ──────────────────────────────────────────────
CO = [(128, 128), (77, 200), (200, 77), (254, 254), (10, 250), (255, 128), (128, 255)]
for l, c in CO:
    n = f'co_l{l:03d}_c{c:03d}.ase'
    write(n, [(100, [profile(), layer('back'), layer('src', 0, l), cel(0, BACK), cel(1, SRC['A'], c)])])
    ARQS.append(n)
# ── z-index ───────────────────────────────────────────────────────────────────
ZS = {'b-1': (0, -1, 0), 'b-2': (0, -2, 0), 'b-3': (0, -3, 0), 'b+1': (0, 1, 0), 'b+2': (0, 2, 0),
      'a+1': (1, 0, 0), 'a+2': (2, 0, 0), 'c-1': (0, 0, -1), 'a+1c-1': (1, 0, -1), 'a+2b+1': (2, 1, 0)}
for nome, (za, zb, zc) in ZS.items():
    n = f'z_{nome}.ase'
    write(n, [(100, [profile(), layer('back'), layer('A'), layer('B'), layer('C'),
                     cel(0, BACK), cel(1, SRC['A'], z=za), cel(2, SRC['B'], z=zb), cel(3, SRC['C'], z=zc)])])
    ARQS.append(n)
# ── papel vegetal ─────────────────────────────────────────────────────────────
ONION = {0: [255, 0, 0], 1: [0, 200, 0], 3: [0, 0, 255], 4: [255, 160, 0]}
QUARTO = {0: 0, 1: 1, 3: 2, 4: 3}
def faixa(f):
    px = []
    for y in range(H):
        for x in range(W):
            mine = x // (W // 4) == QUARTO[f]
            px.append(ONION[f] + [(255 if y < H // 2 else 128) if mine else 0])
    return px
CINZA = [[128, 128, 128, 255] if y < 8 else [0, 0, 0, 0] for y in range(H) for x in range(W)]
for nome, activo in (('vazio', None), ('faixa', CINZA)):
    frames = []
    for f in range(5):
        ch = [profile(), layer('src')] if f == 0 else []
        if f == 2:
            if activo:
                ch.append(cel(0, activo))
        else:
            ch.append(cel(0, faixa(f)))
        frames.append((100, ch))
    n = f'on_{nome}.ase'
    write(n, frames); ARQS.append(n)
# ── tocar ─────────────────────────────────────────────────────────────────────
SOLIDAS = [[230, 25, 75], [60, 180, 75], [255, 225, 25], [0, 130, 200], [245, 130, 48], [145, 30, 180]]
def solida(c):
    return [c + [255]] * N
DIRS = ['fwd', 'rev', 'pp', 'ppr']
for d, dn in enumerate(DIRS):
    for r in (0, 1, 2, 3):
        frames = []
        for f in range(6):
            ch = [profile(), layer('src'), tags([(1, 4, d, r, 'walk')])] if f == 0 else []
            ch.append(cel(0, solida(SOLIDAS[f])))
            frames.append((250, ch))
        n = f'tg_{dn}_r{r}.ase'
        write(n, frames); ARQS.append(n)
frames = []
for f, ms in enumerate((100, 300, 600, 200)):
    ch = [profile(), layer('src')] if f == 0 else []
    ch.append(cel(0, solida(SOLIDAS[f])))
    frames.append((ms, ch))
write('tg_dur.ase', frames); ARQS.append('tg_dur.ase')

json.dump({'w': W, 'h': H, 'back': BACK, 'src': SRC, 'co': CO, 'z': ZS, 'onion': ONION,
           'solidas': SOLIDAS, 'files': ARQS}, open(os.path.join(D, 'entradas.json'), 'w'))
print(len(ARQS), 'ficheiros')
