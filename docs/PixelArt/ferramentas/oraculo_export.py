#!/usr/bin/env python3
"""Oraculo do Export/Import (P9): o LibreSprite 1.3 (GPL-2; so CORRIDO, nunca lido) em `-b` sobre `.ase`
NOSSOS, escritos aqui pela spec publica do formato. A saida vira a fixtura
`crates/ph2d-pixel-io/tests/fixtures/oracle_export.json` (entradas em hex, para os testes as abrirem).

    python3 -I docs/PixelArt/ferramentas/oraculo_export.py <pasta de trabalho> \
        > crates/ph2d-pixel-io/tests/fixtures/oracle_export.json

Com `--verifica a.ase b.ase ...`: o LibreSprite abre ficheiros NOSSOS (gate M2) e devolve o composto e o
que relê de cada um, em JSON. Precisa de: libresprite, Pillow. HOME isolado; sem janela.
"""
import datetime
import hashlib
import json
import os
import struct
import subprocess
import sys
import zlib

from PIL import Image

W, H = 24, 16


def string(t):
    b = t.encode()
    return struct.pack('<H', len(b)) + b


def chunk(kind, data):
    return struct.pack('<IH', len(data) + 6, kind) + data


def layer(name, mode=0, opacity=255, kind=0, level=0, flags=3):
    return chunk(0x2004, struct.pack('<HHHHHHB3x', flags, kind, level, 0, 0, mode, opacity) + string(name))


def cel(index, x, y, w, h, raw, opacity=255):
    return chunk(0x2005, struct.pack('<HhhBHh5x', index, x, y, opacity, 2, 0) + struct.pack('<HH', w, h) + zlib.compress(raw))


def linked(index, frame, x=0, y=0, opacity=255):
    return chunk(0x2005, struct.pack('<HhhBHh5x', index, x, y, opacity, 1, 0) + struct.pack('<H', frame))


def tags(items):
    body = struct.pack('<H8x', len(items))
    for (name, a, b, d, rep, rgb) in items:
        body += struct.pack('<HHBH6x3Bx', a, b, d, rep, *rgb) + string(name)
    return chunk(0x2018, body)


def palette(cols):
    body = struct.pack('<III8x', len(cols), 0, len(cols) - 1)
    for c in cols:
        body += struct.pack('<HBBBB', 0, *c)
    return chunk(0x2019, body)


def ase(frames, depth=32, ncolors=0, transparent=0, w=W, h=H):
    """`frames` = [(duracao, [chunks])]."""
    out = b''
    for (ms, chunks) in frames:
        body = b''.join(chunks)
        out += struct.pack('<IHHH2xI', 16 + len(body), 0xF1FA, len(chunks), ms, len(chunks)) + body
    hdr = struct.pack('<IHHHHHIHII', 128 + len(out), 0xA5E0, len(frames), w, h, depth, 1, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', transparent, ncolors, 1, 1, 0, 0, 16, 16)
    return hdr + b'\0' * (128 - len(hdr)) + out


def box(w, h, c):
    return bytes(c) * (w * h)


def ramp(w, h, seed):
    """Celulas todas diferentes, com alfa parcial nos cantos: nada que um erro de uma celula esconda."""
    out = bytearray()
    for y in range(h):
        for x in range(w):
            a = 255
            if (x, y) in ((0, 0), (w - 1, h - 1)):
                a = 128
            out += bytes(((x * 37 + seed * 11) % 256, (y * 53 + seed * 29) % 256, ((x + y) * 19 + seed) % 256, a))
    return bytes(out)


def completo():
    """Grupo, blend, opacidades de camada e de cel, oculta, ligado, quadro vazio, quadro duplicado, tags."""
    head = [
        layer('fundo'),
        layer('grp', kind=1, opacity=200),
        layer('mult', mode=1, opacity=128, level=1),
        layer('tela', mode=2, level=1),
        layer('oculta', flags=2),
        tags([('idle', 0, 1, 0, 0, (255, 0, 0)), ('walk', 2, 4, 2, 2, (0, 255, 0)), ('back', 1, 3, 1, 3, (0, 0, 255))]),
    ]
    f0 = head + [cel(0, 2, 3, 10, 8, ramp(10, 8, 1)), cel(2, 5, 4, 6, 6, ramp(6, 6, 2)), cel(3, 8, 5, 7, 7, ramp(7, 7, 3), opacity=128),
                 cel(4, 0, 0, 3, 3, box(3, 3, (9, 9, 9, 255)))]
    f1 = [cel(0, 3, 3, 12, 9, ramp(12, 9, 4))]
    f2 = [linked(0, 1, 3, 3), cel(2, 14, 8, 8, 6, ramp(8, 6, 5))]
    f3 = [linked(0, 1, 3, 3), linked(2, 2, 14, 8)]  # igual ao anterior: o duplicado do merge
    f4 = []  # vazio: o do ignore-empty
    return ase([(100, f0), (150, f1), (70, f2), (333, f3), (20, f4)])


def folha():
    """O `completo` so com Normal: as folhas comparam-se por hash, e a mistura sobre destino transparente
    do LibreSprite e a recusa medida da P9."""
    head = [
        layer('fundo'),
        layer('grp', kind=1, opacity=200),
        layer('mult', opacity=128, level=1),
        layer('tela', level=1),
        layer('oculta', flags=2),
        tags([('idle', 0, 1, 0, 0, (255, 0, 0)), ('walk', 2, 4, 2, 2, (0, 255, 0)), ('back', 1, 3, 1, 3, (0, 0, 255))]),
    ]
    f0 = head + [cel(0, 2, 3, 10, 8, ramp(10, 8, 1)), cel(2, 5, 4, 6, 6, ramp(6, 6, 2)), cel(3, 8, 5, 7, 7, ramp(7, 7, 3), opacity=128),
                 cel(4, 0, 0, 3, 3, box(3, 3, (9, 9, 9, 255)))]
    f1 = [cel(0, 3, 3, 12, 9, ramp(12, 9, 4))]
    f2 = [linked(0, 1, 3, 3), cel(2, 14, 8, 8, 6, ramp(8, 6, 5))]
    f3 = [linked(0, 1, 3, 3), linked(2, 2, 14, 8)]
    f4 = []
    return ase([(100, f0), (150, f1), (70, f2), (333, f3), (20, f4)])


def plano():
    """O mesmo sem grupo: o LibreSprite ACHATA grupos ao ler (medido), logo o composto dele so se compara aqui."""
    head = [
        layer('fundo'),
        layer('mult', mode=1, opacity=128),
        layer('tela', mode=2),
        layer('oculta', flags=2),
        tags([('idle', 0, 1, 0, 0, (255, 0, 0)), ('walk', 2, 4, 2, 2, (0, 255, 0)), ('back', 1, 3, 1, 3, (0, 0, 255))]),
    ]
    f0 = head + [cel(0, 2, 3, 10, 8, ramp(10, 8, 1)), cel(1, 5, 4, 6, 6, ramp(6, 6, 2)), cel(2, 8, 5, 7, 7, ramp(7, 7, 3), opacity=128),
                 cel(3, 0, 0, 3, 3, box(3, 3, (9, 9, 9, 255)))]
    f1 = [cel(0, 3, 3, 12, 9, ramp(12, 9, 4))]
    f2 = [linked(0, 1, 3, 3), cel(1, 14, 8, 8, 6, ramp(8, 6, 5))]
    f3 = [linked(0, 1, 3, 3), linked(1, 2, 14, 8)]
    f4 = []
    return ase([(100, f0), (150, f1), (70, f2), (333, f3), (20, f4)])


INDEXED_PAL = [(0, 0, 0, 0), (255, 0, 0, 255), (0, 255, 0, 255), (0, 0, 255, 255), (255, 255, 0, 128), (17, 34, 51, 255), (200, 100, 50, 255), (255, 255, 255, 255)]


def indexado(transparent=0):
    idx = bytes((x + y) % len(INDEXED_PAL) for y in range(8) for x in range(10))
    return ase([(100, [palette(INDEXED_PAL), layer('a'), cel(0, 3, 2, 10, 8, idx)]),
                (200, [cel(0, 6, 4, 10, 8, idx)])], depth=8, ncolors=len(INDEXED_PAL), transparent=transparent)


def um_quadro_indexado():
    idx = bytes((x + y) % len(INDEXED_PAL) for y in range(8) for x in range(10))
    return ase([(100, [palette(INDEXED_PAL), layer('a'), cel(0, 3, 2, 10, 8, idx)])], depth=8, ncolors=len(INDEXED_PAL))


def cinza():
    raw = bytes(v for y in range(8) for x in range(10) for v in ((x * 25) % 256, 255 if y else 128))
    return ase([(100, [layer('a'), cel(0, 1, 1, 10, 8, raw)])], depth=16)


def muitas_cores():
    raw = bytes(v for y in range(32) for x in range(32) for v in (x * 8, y * 8, (x ^ y) * 8, 255))
    return ase([(100, [layer('a'), cel(0, 0, 0, 32, 32, raw)])], w=32, h=32)


def alfa_parcial():
    raw = bytes(v for y in range(4) for x in range(16) for v in (200, 100, 50, x * 17))
    return ase([(100, [layer('a'), cel(0, 0, 0, 16, 4, raw)])], w=16, h=4)


def duracoes():
    """Duracoes que nao sao multiplo de 10 ms, e abaixo do minimo de muitos leitores."""
    a = cel(0, 0, 0, 4, 4, box(4, 4, (255, 0, 0, 255)))
    b = cel(0, 0, 0, 4, 4, box(4, 4, (0, 255, 0, 255)))
    return ase([(ms, ([layer('a'), c] if i == 0 else [c])) for i, (ms, c) in enumerate(((7, a), (15, b), (333, a), (1004, b)))], w=4, h=4)


def modos():
    """Os 19 modos da spec, para o nome que cada um tem no JSON."""
    return ase([(100, [layer(f'm{m:02d}', mode=m) for m in range(19)])], w=4, h=4)


ENTRADAS = {
    'modos': modos,
    'folha': folha,
    'completo': completo, 'plano': plano, 'indexado': indexado, 'um_quadro_indexado': um_quadro_indexado, 'cinza': cinza,
    'muitas_cores': muitas_cores, 'alfa_parcial': alfa_parcial, 'duracoes': duracoes,
}

# A matriz das folhas (M3): (nome, opcoes ANTES do ficheiro, opcoes DEPOIS).
FOLHAS = [
    ('horizontal', [], ['--sheet-type', 'horizontal']),
    ('vertical', [], ['--sheet-type', 'vertical']),
    ('rows_w50', [], ['--sheet-type', 'rows', '--sheet-width', '50']),
    ('rows_w49', [], ['--sheet-type', 'rows', '--sheet-width', '49']),
    ('rows_w48', [], ['--sheet-type', 'rows', '--sheet-width', '48']),
    ('rows_w47', [], ['--sheet-type', 'rows', '--sheet-width', '47']),
    ('rows_w10', [], ['--sheet-type', 'rows', '--sheet-width', '10']),
    ('columns_h32', [], ['--sheet-type', 'columns', '--sheet-height', '32']),
    ('columns_h31', [], ['--sheet-type', 'columns', '--sheet-height', '31']),
    ('os_tres_w59', [], ['--border-padding', '2', '--shape-padding', '3', '--inner-padding', '1', '--sheet-type', 'rows', '--sheet-width', '59']),
    ('os_tres_w58', [], ['--border-padding', '2', '--shape-padding', '3', '--inner-padding', '1', '--sheet-type', 'rows', '--sheet-width', '58']),
    ('os_tres_w57', [], ['--border-padding', '2', '--shape-padding', '3', '--inner-padding', '1', '--sheet-type', 'rows', '--sheet-width', '57']),
    ('os_tres_w55', [], ['--border-padding', '2', '--shape-padding', '3', '--inner-padding', '1', '--sheet-type', 'rows', '--sheet-width', '55']),
    ('split_rows_w50', ['--split-layers'], ['--sheet-type', 'rows', '--sheet-width', '50']),
    ('split_columns', ['--split-layers'], ['--sheet-type', 'columns']),
    ('tag_split', ['--split-layers', '--frame-tag', 'walk'], []),
    ('rows_h32', [], ['--sheet-type', 'rows', '--sheet-height', '32']),
    ('columns_h40', [], ['--sheet-type', 'columns', '--sheet-height', '40']),
    ('columns_w48', [], ['--sheet-type', 'columns', '--sheet-width', '48']),
    ('rows_livre', [], ['--sheet-type', 'rows']),
    ('packed', [], ['--sheet-pack']),
    ('trim', [], ['--trim']),
    ('trim_rows', [], ['--trim', '--sheet-type', 'rows', '--sheet-width', '50']),
    ('border2', [], ['--border-padding', '2']),
    ('shape3', [], ['--shape-padding', '3']),
    ('inner1', [], ['--inner-padding', '1']),
    ('os_tres', [], ['--border-padding', '2', '--shape-padding', '3', '--inner-padding', '1', '--sheet-type', 'rows', '--sheet-width', '70']),
    ('trim_os_tres', [], ['--trim', '--border-padding', '2', '--shape-padding', '3', '--inner-padding', '1']),
    ('ignore_empty', [], ['--ignore-empty']),
    ('split_layers', ['--split-layers'], []),
    ('split_layers_empty', ['--split-layers', '--ignore-empty'], []),
    ('all_layers', ['--all-layers'], []),
    ('layer_tela', ['--layer', 'grp-tela'], []),
    ('layer_oculta', ['--layer', 'oculta'], []),
    ('tag_walk', ['--frame-tag', 'walk'], []),
    ('range_1_3', ['--frame-range', '1,3'], []),
    ('crop', ['--crop', '4,4,10,8'], []),
    ('trim_inner_rows', [], ['--trim', '--inner-padding', '2', '--sheet-type', 'columns', '--sheet-height', '40']),
    ('nome', [], ['--filename-format', '{title}-{layer}-{tag}-{frame}-{frame01}.{extension}']),
    ('nome_split', ['--split-layers'], ['--filename-format', '{layer}_{frame}']),
    ('list', ['--list-layers', '--list-tags'], []),
]


def corre(work, *args):
    env = {k: v for k, v in os.environ.items() if k not in ('DISPLAY', 'WAYLAND_DISPLAY')}
    env.update(HOME=os.path.join(work, 'home'), SDL_VIDEODRIVER='dummy')
    r = subprocess.run(['libresprite', '-b', *args], cwd=work, env=env, capture_output=True, text=True, timeout=60)
    avisos = [l for l in r.stdout.splitlines() + r.stderr.splitlines()
              if any(k in l.lower() for k in ('error', 'warning', 'unsupported', 'cannot'))]
    return r.returncode, avisos


def rgba_hash(path):
    im = Image.open(path).convert('RGBA')
    return {'w': im.width, 'h': im.height, 'rgba_sha256': hashlib.sha256(im.tobytes()).hexdigest()}


def rgba_hex(path):
    im = Image.open(path).convert('RGBA')
    return {'w': im.width, 'h': im.height, 'rgba_hex': im.tobytes().hex()}


def chunks_ase(path):
    """M1: o que o `.ase` DELE diz, por chunk."""
    b = open(path, 'rb').read()
    n, w, h, d = struct.unpack_from('<HHHH', b, 6)
    flags, speed = struct.unpack_from('<IH', b, 14)
    pos, out = 128, []
    trans, ncol = struct.unpack_from('<B3xH', b, 28)
    for f in range(n):
        size, _, old, ms, new = struct.unpack_from('<IHHH2xI', b, pos)
        at, cs = pos + 16, []
        for _ in range(new or old):
            cz, ct = struct.unpack_from('<IH', b, at)
            body = b[at + 6:at + cz]
            e = {'type': hex(ct), 'size': cz}
            if ct == 0x2004:
                fl, k, lv, _, _, bl, op = struct.unpack_from('<HHHHHHB', body)
                nl = struct.unpack_from('<H', body, 16)[0]
                e.update(flags=fl, kind=k, level=lv, blend=bl, opacity=op, name=body[18:18 + nl].decode())
            elif ct == 0x2005:
                ix, x, y, op, kind = struct.unpack_from('<HhhBH', body)
                e.update(layer=ix, x=x, y=y, opacity=op, kind=kind)
                if kind == 1:
                    e['link'] = struct.unpack_from('<H', body, 16)[0]
                else:
                    e.update(w=struct.unpack_from('<H', body, 16)[0], h=struct.unpack_from('<H', body, 18)[0])
            elif ct == 0x2018:
                e['hex'] = body.hex()
            else:
                e['hex'] = body[:64].hex()
            cs.append(e)
            at += cz
        out.append({'duration_ms': ms, 'chunks': cs})
        pos += size
    return {'frames': n, 'w': w, 'h': h, 'depth': d, 'flags': flags, 'speed': speed, 'transparent': trans, 'ncolors': ncol, 'by_frame': out}


def gif_info(path):
    im = Image.open(path)
    fs = []
    for i in range(getattr(im, 'n_frames', 1)):
        im.seek(i)
        c = im.convert('RGBA')
        fs.append({'duration': im.info.get('duration'), 'rgba_sha256': hashlib.sha256(c.tobytes()).hexdigest()})
    im.seek(0)
    return {'n': len(fs), 'loop': im.info.get('loop'), 'frames': fs, 'colors_first': len(set(im.convert('RGBA').tobytes()[i:i + 4] for i in range(0, im.width * im.height * 4, 4))),
            'first_rgba_hex': im.convert('RGBA').tobytes().hex() if im.width * im.height < 128 else None,
            'hex': open(path, 'rb').read().hex() if os.path.getsize(path) < 4096 else None}


def png_info(path):
    b = open(path, 'rb').read()
    pos, out = 8, []
    while pos < len(b):
        n, t = struct.unpack_from('>I4s', b, pos)
        d = b[pos + 8:pos + 8 + n]
        e = {'type': t.decode(), 'len': n}
        if t == b'IHDR':
            e['hex'] = d.hex()
        if t in (b'PLTE', b'tRNS'):
            e['hex'] = d.hex()
        out.append(e)
        pos += 12 + n
    return out


def sheet_case(work, src, nome, antes, depois):
    png, js = f'{nome}.png', f'{nome}.json'
    fmts = {}
    for fmt in ('json-hash', 'json-array'):
        code, avisos = corre(work, *antes, src, '--sheet', png, '--data', js, '--format', fmt, *depois)
        p = os.path.join(work, js)
        fmts[fmt] = {'exit': code, 'avisos': avisos, 'json': open(p).read() if os.path.exists(p) else None}
        if os.path.exists(p):
            os.remove(p)
    pp = os.path.join(work, png)
    return {'name': nome, 'antes': antes, 'depois': depois, 'sheet': rgba_hash(pp) if os.path.exists(pp) else None, **fmts}


def frames_of(work, src, tag):
    """Cada quadro composto, em RGBA: uma folha por quadro."""
    out = []
    n = struct.unpack_from('<H', open(os.path.join(work, src), 'rb').read(), 6)[0]
    for i in range(n):
        png = f'{tag}_q{i}.png'
        corre(work, '--frame-range', f'{i},{i}', src, '--sheet', png)
        out.append(rgba_hex(os.path.join(work, png)))
    return out


def verifica(work, paths):
    out = []
    for p in paths:
        base = os.path.basename(p)
        with open(p, 'rb') as a, open(os.path.join(work, base), 'wb') as b:
            b.write(a.read())
        code, avisos = corre(work, base, '--save-as', 'relido.ase')
        rel = os.path.join(work, 'relido.ase')
        out.append({'file': base, 'exit': code, 'avisos': avisos, 'relido': chunks_ase(rel) if os.path.exists(rel) else None,
                    'frames': frames_of(work, base, base)})
    json.dump(out, sys.stdout, indent=1)
    sys.stdout.write('\n')


def main():
    work = os.path.abspath(sys.argv[1])
    os.makedirs(os.path.join(work, 'home'), exist_ok=True)
    if len(sys.argv) > 2 and sys.argv[2] == '--verifica':
        return verifica(work, [os.path.abspath(p) for p in sys.argv[3:]])
    entradas = {}
    for n, f in ENTRADAS.items():
        b = f()
        with open(os.path.join(work, f'{n}.ase'), 'wb') as o:
            o.write(b)
        entradas[n] = b.hex()

    resaved, frames = {}, {}
    for n in ENTRADAS:  # M1
        code, avisos = corre(work, f'{n}.ase', '--save-as', f'{n}_rt.ase')
        p = os.path.join(work, f'{n}_rt.ase')
        resaved[n] = {'exit': code, 'avisos': avisos, 'chunks': chunks_ase(p) if os.path.exists(p) else None}
        frames[n] = frames_of(work, f'{n}.ase', n)

    folhas = [sheet_case(work, 'folha.ase', *c) for c in FOLHAS]  # M3, M4
    folhas += [sheet_case(work, 'um_quadro_indexado.ase', 'um_quadro', [], [])]
    folhas += [sheet_case(work, 'modos.ase', 'modos', ['--list-layers'], [])]
    folhas += [sheet_case(work, 'indexado.ase', 'ix_' + c[0], *c[1:]) for c in FOLHAS if c[0] in ('rows_w50', 'trim')]

    saidas = {}
    for n in ENTRADAS:  # M5, M6, M7
        d = {}
        for ext, info in (('gif', gif_info), ('webp', gif_info), ('png', png_info)):
            o = f'{n}_out.{ext}'
            code, avisos = corre(work, f'{n}.ase', '--save-as', o)
            p = os.path.join(work, o)
            d[ext] = {'exit': code, 'avisos': avisos, 'info': info(p) if os.path.exists(p) else None}
        d['seq'] = sorted(f for f in os.listdir(work) if f.startswith(f'{n}_out') and f.endswith('.png'))
        saidas[n] = d

    escalas = {}
    for nome, args in (('scale2', ['--scale', '2']), ('scale05', ['--scale', '0.5']), ('shrink', ['--shrink-to', '10,10'])):  # M8
        code, avisos = corre(work, 'folha.ase', *args, '--sheet', f'{nome}.png', '--data', f'{nome}.json')
        escalas[nome] = {'exit': code, 'avisos': avisos, 'sheet': rgba_hash(os.path.join(work, f'{nome}.png')),
                         'json': open(os.path.join(work, f'{nome}.json')).read()}

    lv = subprocess.run(['pacman', '-Q', 'libresprite'], capture_output=True, text=True).stdout.strip()
    json.dump({
        'source': f'{lv} (GPL-2, corrido como oraculo; nunca lido; o rodape do app diz «1.2-dev (local build)»)',
        'method': 'libresprite -b sobre .ase NOSSOS (spec publica), HOME isolado, sem janela',
        'date': datetime.date.today().isoformat(),
        'command': 'python3 -I docs/PixelArt/ferramentas/oraculo_export.py <pasta> > crates/ph2d-pixel-io/tests/fixtures/oracle_export.json',
        'entradas_hex': entradas,
        'relido': resaved,
        'quadros': frames,
        'folhas': folhas,
        'saidas': saidas,
        'escalas': escalas,
    }, sys.stdout, indent=0)
    sys.stdout.write('\n')


if __name__ == '__main__':
    main()
