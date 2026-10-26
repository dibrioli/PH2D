"""Escreve os .ase de ENTRADA da onda P6 (selecao e transformacao) para o Aseprite Trial.

uso: python3 -I trial_ase_selecao.py <dir>
Spec publica do .ase (docs/ase-file-specs.md do Aseprite), sem ler codigo do Aseprite; o escritor e o
de trial_ase_quadros.py (header, layer, cel 0x2005 comprimido, perfil; o campo antigo de contagem de
chunks leva a contagem real).
  red.ase     24x16 vermelho opaco puro: as mascaras (lidas por Edit > Fill magenta, ver trial_selecao.py)
  redb.ase    o mesmo com a camada Background (flag 8) — nao usado no lote final
  wand.ase    24x16 RGBA com cores a distancias CONTROLADAS do preto de referencia (ver WAND abaixo)
  conn.ase    24x16: vermelho com um anel fechado e uma diagonal de 1 px azuis (conectividade 4 x 8)
  xf.ase      24x16, fundo transparente, uma forma 7x5 assimetrica (19 px) com cores distintas (ids 1..19)
  xfo.ase     o mesmo com o fundo opaco (id 0 = cinza escuro 40,40,40) — nao usado no lote final
  sz.ase      24x16 com uma cor distinta POR celula (384 ids) — Sprite Size / Canvas Size / interpolacao
  szb.ase     o sz.ase como camada Background (Canvas Size: o que enche a area nova)
  trim.ase    24x16, transparente com um bloco 5x3 verde opaco em (9,6); trimc.ase o mesmo sobre azul opaco
              (camada normal) e trimcb.ase sobre azul na camada Background
  pm.ase      2 camadas: o sz em baixo (a activa ao abrir), azul so na metade esquerda em cima (Copy Merged)
O estado grava-se em <dir>/entradas.json."""
import json, os, struct, sys, zlib

D = sys.argv[1]
os.makedirs(D, exist_ok=True)
W, H = 24, 16


def string(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data
def layer(name, flags=3, mode=0, opacity=255):
    return chunk(0x2004, struct.pack('<HHHHHHB3x', flags, 0, 0, 0, 0, mode, opacity) + string(name))
def cel(index, px, w, h, opacity=255, z=0, x=0, y=0):
    raw = bytes(v for p in px for v in p[:4])
    return chunk(0x2005, struct.pack('<HhhBHh5x', index, x, y, opacity, 2, z) + struct.pack('<HH', w, h) + zlib.compress(raw))
def profile():
    return chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0))
def write(name, px, w=W, h=H, bg=False, top=None):
    """top = pixels de uma 2.a camada por cima (a sonda da varinha)"""
    chunks = [profile(), layer('bg' if bg else 'src', 3 | (8 if bg else 0))]
    if top is not None:
        chunks.append(layer('probe'))
    chunks.append(cel(0, px, w, h))
    if top is not None:
        chunks.append(cel(1, top, w, h))
    fr = b''.join(chunks)
    body = struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), 100, len(chunks)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, 1, w, h, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    with open(os.path.join(D, name), 'wb') as f:
        f.write(hdr + body)


RED = [255, 0, 0, 255]
T = [0, 0, 0, 0]
write('red.ase', [RED] * (W * H))
write('redb.ase', [RED] * (W * H), bg=True)

# ── varinha: o clique cai em (0,0) = referencia REF (preto opaco); cada coluna x=1..23 de cada linha
#    y=0..6 tem UMA cor a uma distancia d = DS[x-1] desenhada: y=0 R+d, y=1 G+d, y=2 B+d, y=3 A-d,
#    y=4 R+d e G+d, y=5 R+d G+d B+d, y=6 R+d e A-d. A linha y=8 e transparente (A=0) com RGB=(d,d,d)
#    e (0,8) = (0,0,0,0) — o clique da varinha sobre o transparente. O resto e REF.
#    Leitura: Edit > Fill (F) com o fg MAGENTA opaco (255,0,255), cor ausente da grelha.
REF = [0, 0, 0, 255]
DS = [0, 1, 2, 4, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 100, 127, 128, 129, 200, 254, 255]
def wand_cell(y, d):
    return {0: [d, 0, 0, 255], 1: [0, d, 0, 255], 2: [0, 0, d, 255], 3: [0, 0, 0, 255 - d],
            4: [d, d, 0, 255], 5: [d, d, d, 255], 6: [d, 0, 0, 255 - d], 8: [d, d, d, 0]}[y]
WAND = []
for y in range(H):
    for x in range(W):
        if y == 8:
            WAND.append(wand_cell(8, DS[x - 1]) if 1 <= x <= len(DS) else [0, 0, 0, 0])
        elif y < 7 and 1 <= x <= len(DS):
            WAND.append(wand_cell(y, DS[x - 1]))
        else:
            WAND.append(list(REF))
write('wand.ase', WAND)

# ── conectividade: vermelho; um anel azul FECHADO (2..8 x 2..8) em volta de uma ilha vermelha; uma
#    diagonal azul de UM pixel, x - 12 == y (y = 0..11), do topo a borda direita: corta o vermelho em
#    4-conectividade, nao em 8
CONN = [list(RED) for _ in range(W * H)]
for y in range(H):
    for x in range(W):
        if (x in (2, 8) and 2 <= y <= 8) or (y in (2, 8) and 2 <= x <= 8):
            CONN[y * W + x] = [0, 0, 255, 255]
        if x - 12 == y:
            CONN[y * W + x] = [0, 0, 255, 255]
write('conn.ase', CONN)

# ── forma assimetrica 7x5 com 35 cores distintas, encostada em (9,5)
SHAPE = ['1111111',
         '1100000',
         '1111100',
         '1100000',
         '1100001']
def idcolor(i):
    # cores distintas, longe do cinza/branco do xadrez e umas das outras
    return [(37 * i + 60) % 256, (91 * i + 20) % 256, (53 * i + 140) % 256, 255]
XF, IDS = [], []
k = 0
for y in range(H):
    for x in range(W):
        sx, sy = x - 9, y - 5
        if 0 <= sx < 7 and 0 <= sy < 5 and SHAPE[sy][sx] == '1':
            k += 1; XF.append(idcolor(k)); IDS.append(k)
        else:
            XF.append(list(T)); IDS.append(0)
write('xf.ase', XF)
GREY = [40, 40, 40, 255]
write('xfo.ase', [p if p[3] else GREY for p in XF])

# ── uma cor por celula (para Sprite/Canvas Size)
SZ = [[(x * 10 + 5) % 256, (y * 15 + 8) % 256, (x * 7 + y * 13 + 100) % 256, 255] for y in range(H) for x in range(W)]
write('sz.ase', SZ)

TRIM = [[0, 200, 0, 255] if 9 <= x < 14 and 6 <= y < 9 else list(T) for y in range(H) for x in range(W)]
write('trim.ase', TRIM)
write('trimc.ase', [p if p[3] else [0, 0, 255, 255] for p in TRIM])
write('trimcb.ase', [p if p[3] else [0, 0, 255, 255] for p in TRIM], bg=True)
write('szb.ase', SZ, bg=True)
# ── copy merged: em baixo o sz (uma cor por celula); em cima (activa) azul na metade esquerda, transparente na direita
PMB = SZ
PMT = [[0, 0, 255, 255] if x < W // 2 else list(T) for y in range(H) for x in range(W)]
write('pm.ase', PMB, top=PMT)

json.dump({'w': W, 'h': H, 'red': RED, 'wand_ref': REF, 'wand_ds': DS, 'wand': WAND, 'conn': CONN,
           'shape': SHAPE, 'shape_at': [9, 5], 'xf': XF, 'xf_ids': IDS, 'grey': GREY, 'sz': SZ, 'trim': TRIM},
          open(os.path.join(D, 'entradas.json'), 'w'))
print('ok')
