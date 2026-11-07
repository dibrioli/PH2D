"""P11: gera .ase NOSSOS com slices (spec publica do .ase, chunk 0x2022 + user data 0x2020), sem ler codigo do Aseprite.
uso: python3 -I trial_slices.py gerar <dir>     escreve <dir>/{plain,keys,full,hidden,late,weird,tarde}.ase + entradas.json
Cada quadro tem arte propria: grade de 8 px + uma barra cuja posicao diz o numero do quadro, para a foto dizer em que quadro esta."""
import json, os, struct, sys, zlib

W, H = 64, 64
FLAG_NINE, FLAG_PIVOT = 1, 2


def string(t):
    b = t.encode()
    return struct.pack('<H', len(b)) + b


def chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data


def arte(frame):
    px = bytearray()
    for y in range(H):
        for x in range(W):
            if y >= 56 and frame * 8 <= x < frame * 8 + 8:
                px += bytes((230, 60, 40, 255))
            elif (x // 8 + y // 8) % 2:
                px += bytes((70, 110, 160, 255))
            else:
                px += bytes((200, 210, 170, 255))
    return bytes(px)


def cel(frame):
    return chunk(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', W, H) + zlib.compress(arte(frame)))


def slice_chunk(s):
    flags = s.get('flags', 0)
    out = struct.pack('<III', len(s['keys']), flags, 0) + string(s['name'])
    for k in s['keys']:
        out += struct.pack('<IiiII', k['frame'], k['x'], k['y'], k['w'], k['h'])
        if flags & FLAG_NINE:
            out += struct.pack('<iiII', *k['center'])
        if flags & FLAG_PIVOT:
            out += struct.pack('<ii', *k['pivot'])
    parts = [chunk(0x2022, out)]
    if 'color' in s:
        parts.append(chunk(0x2020, struct.pack('<I', 2) + bytes(s['color'])))
    return parts


def escreve(path, frames, slices_por_quadro):
    corpo = b''
    for f in range(frames):
        chunks = []
        if f == 0:
            chunks.append(chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0)))
            chunks.append(chunk(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + string('art')))
        chunks.append(cel(f))
        for s in slices_por_quadro.get(f, []):
            chunks += slice_chunk(s)
        corpo += struct.pack('<IHHH2xI', 16 + sum(map(len, chunks)), 0xF1FA, len(chunks), 100, len(chunks)) + b''.join(chunks)
    hdr = struct.pack('<IHHHHHIHII', 128 + len(corpo), 0xA5E0, frames, W, H, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    with open(path, 'wb') as f:
        f.write(hdr + corpo)


def k(frame, x, y, w, h, center=None, pivot=None):
    d = {'frame': frame, 'x': x, 'y': y, 'w': w, 'h': h}
    if center:
        d['center'] = center
    if pivot:
        d['pivot'] = pivot
    return d


FIXTURAS = {
    'plain': (6, {}),
    'keys': (6, {0: [{'name': 'moves', 'color': (255, 0, 0, 255), 'keys': [k(0, 8, 8, 16, 16), k(2, 24, 8, 16, 16), k(4, 24, 24, 32, 16)]}]}),
    'full': (6, {0: [
        {'name': 'nine', 'flags': FLAG_NINE, 'color': (0, 200, 0, 255), 'keys': [k(0, 4, 4, 24, 24, center=(8, 8, 8, 8))]},
        {'name': 'pivot', 'flags': FLAG_PIVOT, 'color': (0, 0, 255, 255), 'keys': [k(0, 36, 4, 24, 24, pivot=(12, 24))]},
        {'name': 'both', 'flags': FLAG_NINE | FLAG_PIVOT, 'color': (255, 128, 0, 128), 'keys': [
            k(0, 4, 36, 24, 20, center=(4, 4, 16, 12), pivot=(0, 0)),
            k(3, 36, 36, 16, 20, center=(2, 6, 12, 8), pivot=(8, 10))]},
    ]}),
    'hidden': (6, {0: [{'name': 'blinks', 'keys': [k(0, 8, 8, 16, 16), k(2, 0, 0, 0, 0), k(4, 32, 32, 16, 16)]}]}),
    'late': (6, {0: [{'name': 'late', 'keys': [k(2, 8, 8, 16, 16)]}]}),
    'weird': (6, {0: [
        {'name': 'out', 'keys': [k(0, -8, -8, 24, 24)]},
        {'name': 'past', 'keys': [k(0, 56, 56, 24, 24)]},
        {'name': 'twin', 'keys': [k(0, 20, 20, 8, 8)]},
        {'name': 'twin', 'keys': [k(0, 30, 20, 8, 8)]},
        {'name': 'badcenter', 'flags': FLAG_NINE, 'keys': [k(0, 8, 32, 16, 16, center=(12, 12, 20, 20))]},
        {'name': 'badpivot', 'flags': FLAG_PIVOT, 'keys': [k(0, 32, 32, 16, 16, pivot=(-5, 99))]},
        {'name': 'farframe', 'keys': [k(99, 40, 8, 8, 8)]},
        {'name': 'unsorted', 'keys': [k(3, 44, 44, 8, 8), k(1, 4, 44, 8, 8)]},
    ]}),
    'tarde': (6, {3: [{'name': 'chunk_in_frame_3', 'keys': [k(0, 8, 8, 16, 16), k(3, 32, 8, 16, 16)]}]}),
}


def gerar(d):
    os.makedirs(d, exist_ok=True)
    entradas = {}
    for nome, (frames, slices) in FIXTURAS.items():
        escreve(os.path.join(d, nome + '.ase'), frames, slices)
        entradas[nome] = {'frames': frames, 'canvas': [W, H], 'slices_por_quadro': {str(f): v for f, v in slices.items()}}
    with open(os.path.join(d, 'entradas.json'), 'w') as f:
        json.dump(entradas, f, indent=1)


if __name__ == '__main__':
    assert sys.argv[1] == 'gerar', __doc__
    gerar(sys.argv[2])
