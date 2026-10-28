"""Giro fora de 90 graus e inclinacao no Aseprite Trial (caixa preta): entradas, corrida e fixtura.

uso: python3 -I trial_rotacao.py ase <dir>                  -> <dir>/r86.ase r75.ase r58.ase + entradas_rot.json
     python3 -I trial_rotacao.py run <dir> [:display] [prefixo ...] -> <dir>/lido.json (acumulativo)
     python3 -I trial_rotacao.py fixture <dir> <saida.json>

Entradas NOSSAS (spec publica do .ase, o escritor de trial_ase_selecao.py): canvas 32x24 transparente com
UM bloco cheio de cores distintas (id 1..w*h, linha a linha) — cada celula da saida diz de que celula da
fonte veio. r86 = 8x6 em (12,9) (centro inteiro), r75 = 7x5 em (12,9) (centro em meia celula), r58 = 5x8 em
(13,8) (misto). A corrida e a de trial_selecao.py (os mesmos quirks e coordenadas da barra; Xvfb proprio,
1600x1000, screen_scale 2, zoom 800 %): Ctrl+A, Ctrl+T, os campos Angle/Skew e Enter. Nunca o ecra do dono.
"""
import json, os, struct, sys, zlib

CW, CH = 32, 24
BLOCKS = {'r86': (8, 6, 12, 9), 'r75': (7, 5, 12, 9), 'r58': (5, 8, 13, 8)}
CHECKER = {'808080', 'c0c0c0'}


def idcolor(i):
    return [(37 * i + 60) % 256, (91 * i + 20) % 256, (53 * i + 140) % 256, 255]


def write_ase(path, px, w, h):
    def string(t):
        b = t.encode(); return struct.pack('<H', len(b)) + b
    def chunk(kind, data):
        return struct.pack('<IH', len(data) + 6, kind) + data
    raw = bytes(v for p in px for v in p[:4])
    chunks = [chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0)),
              chunk(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + string('src')),
              chunk(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', w, h)
                    + zlib.compress(raw))]
    fr = b''.join(chunks)
    body = struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), 100, len(chunks)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, 1, w, h, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    with open(path, 'wb') as f:
        f.write(hdr + body)


def ase(d):
    os.makedirs(d, exist_ok=True)
    out = {'w': CW, 'h': CH, 'blocks': {}}
    for name, (bw, bh, bx, by) in BLOCKS.items():
        ids = [[0] * CW for _ in range(CH)]
        px = []
        for y in range(CH):
            for x in range(CW):
                if bx <= x < bx + bw and by <= y < by + bh:
                    i = (y - by) * bw + (x - bx) + 1
                    ids[y][x] = i
                    px.append(idcolor(i))
                else:
                    px.append([0, 0, 0, 0])
        cols = {i: '%02x%02x%02x' % tuple(idcolor(i)[:3]) for i in range(1, bw * bh + 1)}
        assert len(set(cols.values())) == len(cols) and not CHECKER & set(cols.values())
        write_ase(os.path.join(d, name + '.ase'), px, CW, CH)
        out['blocks'][name] = {'box': [bx, by, bw, bh], 'ids': ids, 'colors': cols}
    json.dump(out, open(os.path.join(d, 'entradas_rot.json'), 'w'))
    print('ok')


FAST_ANGLES = [3, 8, 15, 26, 45, 53, 60, 72, 100, 117, 150, 170, 180, -10, -45, -60, -135]
SKEWS = [10, -10, 25, -40, 45]
ROTSPRITE_ANGLES = [8, 15, 26, 45, 60, 100, 150, -30]


def cases():
    out = []
    for blk in BLOCKS:
        for a in FAST_ANGLES:
            out.append((f'rot_fast_{blk}_a{a}', blk, 'fast', {'angle': a}))
        for k in SKEWS:
            out.append((f'rot_fast_{blk}_k{k}', blk, 'fast', {'skew': k}))
        out.append((f'rot_fast_{blk}_a30k15', blk, 'fast', {'angle': 30, 'skew': 15}))
    # cantos a menos de 1e-4 de um inteiro (o raio 5 do r86 perto do extremo): o arredondamento do canto
    for a in [-127, -37, 143]:
        out.append((f'rot_fast_r86_a{a}', 'r86', 'fast', {'angle': a}))
    for blk in ('r86', 'r75'):
        for a in ROTSPRITE_ANGLES:
            out.append((f'rot_rotsprite_{blk}_a{a}', blk, 'rotsprite', {'angle': a}))
        out.append((f'rot_rotsprite_{blk}_k20', blk, 'rotsprite', {'skew': 20}))
    return out


def run(d, display, prefixes):
    import time
    sys.argv = [sys.argv[0], d]
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import trial_selecao as T
    T.ENV['DISPLAY'] = display
    out_p = os.path.join(d, 'lido.json')
    out = json.load(open(out_p)) if os.path.exists(out_p) else {}
    todo = [c for c in cases() if (not prefixes or any(c[0].startswith(p) for p in prefixes)) and c[0] not in out]
    for name, blk, alg, vals in todo:
        T.abre(blk + '.ase')   # sem ensure_fg: a transformacao nao le mascara
        t = time.time()
        try:
            out[name] = T.c_xform(alg, **vals)()
        except Exception as e:
            out[name] = {'error': repr(e)}
            T.key('Escape'); T.key('Escape')
        T.key('Escape'); T.fecha()
        print(f'{name} {time.time() - t:.1f}s', flush=True)
        json.dump(out, open(out_p, 'w'))
    print('FIM', len(todo))


def fixture(d, dest):
    import datetime
    E = json.load(open(os.path.join(d, 'entradas_rot.json')))
    L = json.load(open(os.path.join(d, 'lido.json')))
    cs = {}
    for name, blk, alg, vals in cases():
        v = L.get(name)
        if v is None:
            continue
        c = {'block': blk, 'algorithm': alg, 'fields': vals}
        if 'error' in v:
            c['error'] = v['error']
        else:
            table = {h: int(i) for i, h in E['blocks'][blk]['colors'].items()}
            ids = [[table.get(h, 0 if h in CHECKER else -1) for h in row] for row in v['rgb']]
            assert (v['w'], v['h']) == (CW, CH), name
            c['ids'] = ids
            if any(-1 in r for r in ids):
                c['rgb'] = [''.join(r) for r in v['rgb']]
        cs[name] = c
    fx = {
        'source': 'Aseprite Trial 1.3.18.6 (AppImage, observed black box) under its own Xvfb 1600x1000, '
                  'screen_scale 2, 800 % zoom (16 screen px per cell), centre of each cell; driven by '
                  'docs/PixelArt/ferramentas/trial_rotacao.py over trial_selecao.py (xdotool; quirks in its '
                  'docstring). Aseprite/LibreSprite source never read.',
        'method': 'each input = 32x24 transparent canvas with ONE full block of distinct colours (id 1..w*h, '
                  'row-major); Ctrl+A, Ctrl+T (box = the block), algorithm combo, fields Angle then Skew '
                  '(positive angle = CCW on screen), Enter. Output ids: id of the source cell, 0 = '
                  'transparent (checker), -1 = colour of no input (then rgb rows).',
        'date': datetime.date.today().isoformat(),
        'command': 'python3 -I docs/PixelArt/ferramentas/trial_rotacao.py ase <dir> && '
                   'python3 -I docs/PixelArt/ferramentas/trial_rotacao.py run <dir> :83 && '
                   'python3 -I docs/PixelArt/ferramentas/trial_rotacao.py fixture <dir> <this file>',
        'inputs': {k: {'box': b['box']} for k, b in E['blocks'].items()},
        'cases': cs,
    }
    with open(dest, 'w') as f:
        json.dump(fx, f, separators=(',', ':'))
        f.write('\n')
    print(len(cs), 'casos')


if __name__ == '__main__':
    cmd = sys.argv[1]
    if cmd == 'ase':
        ase(sys.argv[2])
    elif cmd == 'run':
        run(os.path.abspath(sys.argv[2]), sys.argv[3], sys.argv[4:])
    else:
        fixture(sys.argv[2], sys.argv[3])
