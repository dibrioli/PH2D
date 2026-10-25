"""Escreve os .ase de ENTRADA da medicao dos modos de mistura no Aseprite Trial (P3).

uso: python3 -I trial_ase_camadas.py <dir> [W H]
Cada ficheiro: uma camada de baixo OPACA de cores sorteadas e, por cima, a fonte (RGBA sorteado,
alfas 0..255 com peso nos extremos) com o modo e a opacidade de camada do nome. O composto fica
opaco, portanto a tela do Trial MOSTRA o resultado (sem o xadrez a misturar-se).
  m{MM}_o{OOO}.ase      fonte com o modo MM (0..18, ids do formato) e opacidade OOO
  g{MM}.ase             grupo (Normal, 255) com [meio semi-transparente Normal, fonte modo MM] sobre
                        a de baixo: mede o grupo ISOLADO vs pass-through e o modo sobre destino
                        nao-opaco
  gm{MM}_o{OOO}.ase     grupo com modo MM e opacidade OOO contendo a fonte Normal
  ix_o{OOO}.ase         Indexed (paleta de 64), fonte Normal com opacidade OOO
  cel_o{OOO}.ase        opacidade do CEL (camada a 255) — a outra porta da opacidade
O estado sorteado grava-se em <dir>/entradas.json (as mesmas cores, lidas pela fixtura).
Formato: spec publica do .ase (docs/ase-file-specs.md do Aseprite), sem ler codigo do Aseprite."""
import json, os, struct, sys, zlib

D = sys.argv[1]
W, H = (int(sys.argv[2]), int(sys.argv[3])) if len(sys.argv) > 3 else (96, 64)
os.makedirs(D, exist_ok=True)

s = 0x2545F491
def rnd():
    global s
    s ^= (s << 13) & 0xFFFFFFFF; s ^= s >> 17; s ^= (s << 5) & 0xFFFFFFFF
    return s
ALFAS = [0, 1, 32, 64, 100, 127, 128, 160, 200, 254, 255, 255, 255, 255]
def cor():
    # metade dos canais em valores de fronteira (0, 128, 255), metade livre
    return [rnd() % 256 if rnd() % 2 else [0, 1, 64, 127, 128, 129, 192, 254, 255][rnd() % 9] for _ in range(3)]
N = W * H
BACK = [cor() + [255] for _ in range(N)]
SRC = [cor() + [ALFAS[rnd() % len(ALFAS)] if rnd() % 3 else rnd() % 256] for _ in range(N)]
MID = [cor() + [[0, 64, 128, 200, 255][rnd() % 5]] for _ in range(N)]

def string(t):
    b = t.encode(); return struct.pack('<H', len(b)) + b
def chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data
def layer(name, mode=0, opacity=255, kind=0, level=0, flags=3):
    return chunk(0x2004, struct.pack('<HHHHHHB3x', flags, kind, level, 0, 0, mode, opacity) + string(name))
def cel(index, px, opacity=255, bpp=4):
    raw = bytes(v for p in px for v in p[:bpp]) if bpp > 1 else bytes(px)
    return chunk(0x2005, struct.pack('<HhhBHh5x', index, 0, 0, opacity, 2, 0) + struct.pack('<HH', W, H) + zlib.compress(raw))
def profile():
    return chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0))
def palette(cols):
    body = struct.pack('<III8x', len(cols), 0, len(cols) - 1)
    for c in cols:
        body += struct.pack('<HBBBB', 0, *c)
    return chunk(0x2019, body)
def write(name, chunks, depth=32, ncolors=0, transparent=0):
    frame = b''.join(chunks)
    # o campo velho da contagem leva o numero (com 0xFFFF o Trial 1.3.18 desalinha: medido)
    fh = struct.pack('<IHHH2xI', 16 + len(frame), 0xF1FA, len(chunks), 100, len(chunks)) + frame
    hdr = struct.pack('<IHHHHHIHII', 128 + len(fh), 0xA5E0, 1, W, H, depth, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', transparent, ncolors, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    with open(os.path.join(D, name), 'wb') as f:
        f.write(hdr + fh)

ARQS = []
for m in range(19):
    for o in (255, 128, 77):
        n = f'm{m:02d}_o{o:03d}.ase'
        write(n, [profile(), layer('back'), layer('src', m, o), cel(0, BACK), cel(1, SRC)]); ARQS.append(n)
    n = f'g{m:02d}.ase'
    write(n, [profile(), layer('back'), layer('grp', kind=1), layer('mid', level=1), layer('src', m, level=1),
              cel(0, BACK), cel(2, MID), cel(3, SRC)]); ARQS.append(n)
    if m:
        n = f'gm{m:02d}_o255.ase'
        write(n, [profile(), layer('back'), layer('grp', m, kind=1), layer('src', level=1),
                  cel(0, BACK), cel(2, SRC)]); ARQS.append(n)
for o in (255, 128, 77):
    n = f'gm00_o{o:03d}.ase'
    write(n, [profile(), layer('back'), layer('grp', 0, o, kind=1), layer('src', level=1), cel(0, BACK), cel(2, SRC)]); ARQS.append(n)
    n = f'cel_o{o:03d}.ase'
    write(n, [profile(), layer('back'), layer('src'), cel(0, BACK), cel(1, SRC, o)]); ARQS.append(n)
# Indexed: 64 cores sorteadas, indice 0 transparente; a de baixo usa 1..63, a fonte 0..63.
PAL = [[0, 0, 0, 0]] + [cor() + [255] for _ in range(63)]
IB = [1 + rnd() % 63 for _ in range(N)]
IS = [rnd() % 64 for _ in range(N)]
for o in (255, 128, 77):
    n = f'ix_o{o:03d}.ase'
    write(n, [profile(), palette(PAL), layer('back'), layer('src', 0, o), cel(0, IB, bpp=1), cel(1, IS, bpp=1)],
          depth=8, ncolors=64); ARQS.append(n)
json.dump({'w': W, 'h': H, 'back': BACK, 'src': SRC, 'mid': MID, 'pal': PAL, 'ib': IB, 'is': IS, 'files': ARQS},
          open(os.path.join(D, 'entradas.json'), 'w'))
print(len(ARQS), 'ficheiros')
