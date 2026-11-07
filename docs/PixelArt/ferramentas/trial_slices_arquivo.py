"""P11 F1/F3/F5: corre o Trial 1.3.18.6 em batch sobre .ase NOSSOS e guarda o que ele escreve. O Trial recusa PNG/.ase, mas escreve o JSON de --data.
uso: python3 -I trial_slices_arquivo.py gerar <dir_ase>    fixtures extra (ud, tags, transp, zeros, realorder) ao lado das de trial_slices.py
     python3 -I trial_slices_arquivo.py correr <dir_ase> <dir_saida> [filtro]   grava <dir_saida>/batch.json
Nao usa --script (Lua) nem tenta contornar a recusa de gravar imagem."""
import json, os, struct, subprocess, sys, time, zlib
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_slices as T

APP = os.path.expanduser('~/.local/opt/aseprite-trial/Aseprite-trial.AppImage')
W, H = 64, 64
RECUSA = 'Save operation is not supported in trial version.\nGo to http://www.aseprite.org/download/ and get the full-version.'


def slice_ud(s):
    flags = s.get('flags', 0)
    out = struct.pack('<III', len(s['keys']), flags, 0) + T.string(s['name'])
    for k in s['keys']:
        out += struct.pack('<IiiII', k['frame'], k['x'], k['y'], k['w'], k['h'])
        if flags & 1:
            out += struct.pack('<iiII', *k['center'])
        if flags & 2:
            out += struct.pack('<ii', *k['pivot'])
    parts = [T.chunk(0x2022, out)]
    fl = (1 if 'text' in s else 0) | (2 if 'color' in s else 0)
    if fl:
        d = struct.pack('<I', fl)
        if 'text' in s:
            d += T.string(s['text'])
        if 'color' in s:
            d += bytes(s['color'])
        parts.append(T.chunk(0x2020, d))
    return parts


def bloco(x0, y0, w, h):
    px = bytearray(W * H * 4)
    for y in range(y0, y0 + h):
        for x in range(x0, x0 + w):
            px[(y * W + x) * 4:(y * W + x) * 4 + 4] = bytes((230, 60, 40, 255))
    return bytes(px)


def cel(layer, art):
    return T.chunk(0x2005, struct.pack('<HhhBHh5x', layer, 0, 0, 255, 2, 0) + struct.pack('<HH', W, H) + zlib.compress(art))


def escreve(path, frames, layers, arte, slices_antes=(), tags=()):
    corpo = b''
    for f in range(frames):
        ch = []
        if f == 0:
            ch.append(T.chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0)))
            if tags:
                d = struct.pack('<H8x', len(tags))
                for nome, a, b in tags:
                    d += struct.pack('<HHB8xBBBB', a, b, 0, 0, 0, 0, 0) + T.string(nome)
                ch.append(T.chunk(0x2018, d))
            for n in layers:
                ch.append(T.chunk(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + T.string(n)))
            for s in slices_antes:          # ordem real: slices DEPOIS das camadas, ANTES dos cels
                ch += slice_ud(s)
        for i, _ in enumerate(layers):
            a = arte(i, f)
            if a is not None:
                ch.append(cel(i, a))
        corpo += struct.pack('<IHHH2xI', 16 + sum(map(len, ch)), 0xF1FA, len(ch), 100, len(ch)) + b''.join(ch)
    hdr = struct.pack('<IHHHHHIHII', 128 + len(corpo), 0xA5E0, frames, W, H, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + corpo)


k = T.k
EXTRA = {
    'ud': (1, ['art'], lambda i, f: T.arte(0), [
        {'name': 'color_text', 'color': (10, 20, 30, 40), 'text': 'hello', 'keys': [k(0, 0, 0, 8, 8)]},
        {'name': 'text_only', 'text': 'só texto', 'keys': [k(0, 8, 0, 8, 8)]},
        {'name': 'no_ud', 'keys': [k(0, 16, 0, 8, 8)]},
        {'name': 'alpha', 'color': (255, 128, 0, 128), 'keys': [k(0, 24, 0, 8, 8)]},
        {'name': 'açã ο/x', 'color': (1, 2, 3, 255), 'keys': [k(0, 32, 0, 8, 8)]},
        {'name': '', 'color': (4, 5, 6, 255), 'keys': [k(0, 40, 0, 8, 8)]},
        {'name': 'quo"te\\', 'text': 'a"b\\c', 'keys': [k(0, 48, 0, 8, 8)]},
    ], []),
    'tags': (6, ['bottom', 'top'], lambda i, f: T.arte(f) if i == 0 else (bloco(0, 0, 8, 8) if f < 3 else None), [
        {'name': 'moves', 'keys': [k(0, 8, 8, 16, 16), k(2, 24, 8, 16, 16), k(4, 24, 24, 32, 16)]},
        {'name': 'still', 'keys': [k(0, 40, 40, 8, 8)]},
    ], [('idle', 0, 2), ('walk', 3, 5)]),
    'transp': (3, ['art'], lambda i, f: bloco(10, 10, 8, 8) if f == 0 else (bloco(42, 42, 4, 4) if f == 1 else None), [
        {'name': 'a', 'keys': [k(0, 8, 8, 16, 16)]},
        {'name': 'b', 'keys': [k(0, 40, 40, 16, 16)]},
    ], []),
    'zeros': (6, ['art'], lambda i, f: T.arte(f), [
        {'name': 'w0_first', 'keys': [k(0, 0, 0, 0, 0), k(2, 8, 8, 16, 16)]},
        {'name': 'w0_only', 'keys': [k(0, 8, 8, 0, 16)]},
        {'name': 'h0_only', 'keys': [k(0, 8, 8, 16, 0)]},
        {'name': 'w0_xy', 'keys': [k(0, 8, 8, 16, 16), k(3, 30, 30, 0, 16)]},
        {'name': 'no_keys', 'keys': []},
    ], []),
    'realorder': (6, ['art'], lambda i, f: T.arte(f), [
        {'name': 'first', 'color': (0, 0, 255, 255), 'keys': [k(0, 8, 8, 16, 16)]},
        {'name': 'pivonly', 'flags': 2, 'color': (0, 0, 255, 255), 'keys': [k(0, 30, 8, 16, 16, pivot=(0, 0))]},
        {'name': 'cen0', 'flags': 1, 'color': (0, 0, 255, 255), 'keys': [k(0, 8, 30, 16, 16, center=(0, 0, 16, 16))]},
    ], []),
}


# Sem user data a slice fica sem cor e o Trial nao a desenha (medido): as copias c_* trazem cor para a foto dizer o quadro.
for _n in ('late', 'hidden', 'tarde', 'weird'):
    _fr, _sq = T.FIXTURAS[_n]
    _sl = [dict(s, color=(255, 0, 255, 255)) for v in _sq.values() for s in v]
    EXTRA['c_' + _n] = (_fr, ['art'], (lambda i, f: T.arte(f)), _sl, [])
EXTRA['c_zeros'] = (6, ['art'], EXTRA['zeros'][2], [dict(s, color=(255, 0, 255, 255)) for s in EXTRA['zeros'][3]], [])


EXTRA['c_zero2'] = (6, ['art'], EXTRA['zeros'][2], [dict(s, color=(255, 0, 255, 255)) for s in [
    {'name': 'w0_only', 'keys': [k(0, 40, 8, 0, 16)]},
    {'name': 'h0_only', 'keys': [k(0, 8, 40, 16, 0)]},
    {'name': 'w0_later', 'keys': [k(0, 40, 40, 16, 16), k(3, 50, 50, 0, 8)]},
    {'name': 'one_px', 'keys': [k(0, 8, 8, 1, 1)]},
    {'name': 'center_zero', 'flags': 1, 'keys': [k(0, 16, 16, 16, 16, center=(8, 8, 0, 0))]},
]], [])


def gerar(d):
    os.makedirs(d, exist_ok=True)
    meta = {}
    for n, (fr, ly, art, sl, tg) in EXTRA.items():
        escreve(os.path.join(d, n + '.ase'), fr, ly, art, sl, tg)
        meta[n] = {'frames': fr, 'layers': ly, 'slices': sl, 'tags': tg}
    json.dump(meta, open(os.path.join(d, 'entradas_extra.json'), 'w'), indent=1, ensure_ascii=False)


REAL = {'real_props': '/home/enio/Downloads/ase-para-testar/file-tests-props.aseprite',
        'real_app': '/home/enio/Área de trabalho/Tiling/research/tilemap/vendor/ldtk/res/atlas/appElements.aseprite',
        'real_icons': '/home/enio/Área de trabalho/Tiling/research/tilemap/vendor/ldtk/res/atlas/icons.aseprite'}
FIX = ['plain', 'keys', 'full', 'hidden', 'late', 'weird', 'tarde', 'ud', 'tags', 'transp', 'zeros', 'realorder', 'c_late', 'c_hidden', 'c_tarde', 'c_weird', 'c_zeros', 'c_zero2']
D = 'd.json'
JA = ['--data', D, '--format', 'json-array', '--sheet', 's.png']


def casos(a):
    """A ordem dos flags importa ao Aseprite: «next given sprite»; por isso a variante antes/depois do ficheiro."""
    p = lambda n: os.path.join(a, n + '.ase')
    c = []
    for n in FIX:
        c.append((f'meta_{n}', ['-b', '--list-slices', p(n)] + JA))
        c.append((f'split_{n}', ['-b', '--split-slices', p(n)] + JA))
    for n, f in REAL.items():
        c.append((f'meta_{n}', ['-b', '--list-slices', f] + JA))
    K = p('keys')
    c += [
        ('ordem_lista_depois', ['-b', K, '--list-slices', '--data', D, '--sheet', 's.png']),
        ('ordem_lista_antes_sem_data', ['-b', '--list-slices', K]),
        ('ordem_lista_depois_sem_data', ['-b', K, '--list-slices']),
        ('lista_sem_list', ['-b', K, '--data', D, '--sheet', 's.png']),
        ('hash_default', ['-b', '--split-slices', K, '--data', D, '--sheet', 's.png']),
        ('split_depois', ['-b', K, '--split-slices', '--data', D, '--sheet', 's.png']),
        ('saveas_split_v', ['-b', '-v', '--split-slices', K, '--save-as', 'x_{slice}.png']),
        ('saveas_doc_v', ['-b', '-v', K, '--save-as', 'x-{slice}.png']),
        ('saveas_slice_v', ['-b', '-v', '--slice', 'moves', K, '--save-as', 'x.png']),
        ('split_data_saveas', ['-b', '--split-slices', K, '--data', D, '--format', 'json-array', '--save-as', 'x_{slice}.png']),
        ('slice_moves', ['-b', '--slice', 'moves', K] + JA),
        ('slice_moves_list', ['-b', '--list-slices', '--slice', 'moves', K] + JA),
        ('slice_inexistente', ['-b', '--slice', 'nope', K] + JA),
        ('slice_gemeo', ['-b', '--slice', 'twin', p('weird')] + JA),
        ('slice_escondido', ['-b', '--slice', 'blinks', p('hidden')] + JA),
        ('slice_late', ['-b', '--slice', 'late', p('late')] + JA),
        ('slice_fora', ['-b', '--slice', 'out', p('weird')] + JA),
        ('split_fmt_slice', ['-b', '--split-slices', '--filename-format', '{title}|{slice}|{frame}|{tag}|{layer}|{extension}', p('tags')] + JA),
        ('fmt_slice_sem_split', ['-b', '--filename-format', '{title}|{slice}|{frame}', K] + JA),
        ('ss_split_tags', ['-b', '--split-slices', '--split-tags', p('tags')] + JA),
        ('split_layers', ['-b', '--split-slices', '--split-layers', p('tags')] + JA),
        ('split_tudo', ['-b', '--split-slices', '--split-tags', '--split-layers', p('tags')] + JA),
        ('split_tag_idle', ['-b', '--split-slices', '--tag', 'idle', p('tags')] + JA),
        ('split_range', ['-b', '--split-slices', '--frame-range', '1,3', K] + JA),
        ('meta_range', ['-b', '--list-slices', '--frame-range', '1,3', K] + JA),
        ('split_trim', ['-b', '--split-slices', '--trim', p('transp')] + JA),
        ('split_trimsprite', ['-b', '--split-slices', '--trim-sprite', p('transp')] + JA),
        ('split_ignore', ['-b', '--split-slices', '--ignore-empty', p('transp')] + JA),
        ('split_dup', ['-b', '--split-slices', '--merge-duplicates', p('transp')] + JA),
        ('split_grid_junto', ['-b', '--split-slices', '--split-grid', K] + JA),
        ('split_scale', ['-b', '--split-slices', '--scale', '2', K] + JA),
        ('meta_scale', ['-b', '--list-slices', '--scale', '2', K] + JA),
        ('sem_sheet', ['-b', '--list-slices', K, '--data', D]),
        ('meta_tags_layers', ['-b', '--list-slices', '--list-tags', '--list-layers', p('tags')] + JA),
        ('x_lista_weird', ['-b', '--list-slices', p('weird')]),
        ('x_lista_hidden', ['-b', '--list-slices', p('hidden')]),
        ('x_lista_ud', ['-b', '--list-slices', p('ud')]),
        ('x_lista_zeros', ['-b', '--list-slices', p('zeros')]),
        ('x_slice_depois', ['-b', K, '--slice', 'moves'] + JA),
        ('x_slice_data_saveas', ['-b', '--slice', 'moves', K, '--data', D, '--format', 'json-array', '--save-as', 'x.png']),
        ('x_split_depois_saveas', ['-b', K, '--split-slices', '--save-as', 'x_{slice}.png']),
        ('x_meta_trim', ['-b', '--list-slices', '--trim', p('transp')] + JA),
        ('x_meta_tag', ['-b', '--list-slices', '--tag', 'walk', p('tags')] + JA),
        ('x_meta_split_tags', ['-b', '--list-slices', '--split-tags', p('tags')] + JA),
        ('x_meta_split_layers', ['-b', '--list-slices', '--split-layers', p('tags')] + JA),
        ('x_meta_split_slices', ['-b', '--list-slices', '--split-slices', K] + JA),
        ('x_meta_pack', ['-b', '--list-slices', '--sheet-pack', K] + JA),
        ('x_meta_hash', ['-b', '--list-slices', K, '--data', D, '--format', 'json-hash', '--sheet', 's.png']),
        ('x_meta_dois_ficheiros', ['-b', '--list-slices', K, '--list-slices', p('late')] + JA),
    ]
    return c


def correr(a, s, filtro=None):
    os.makedirs(s, exist_ok=True)
    home = os.path.join(s, 'home', '.config', 'aseprite')
    os.makedirs(home, exist_ok=True)
    ini = os.path.expanduser('~/.config/aseprite/aseprite.ini')
    if os.path.exists(ini):
        open(os.path.join(home, 'aseprite.ini'), 'wb').write(open(ini, 'rb').read())
    env = {x: y for x, y in os.environ.items() if x not in ('DISPLAY', 'WAYLAND_DISPLAY')}
    env['HOME'] = os.path.join(s, 'home')
    saida = os.path.join(s, 'batch.json')
    res = json.load(open(saida)) if os.path.exists(saida) else {}
    for cid, argv in casos(a):
        if filtro and filtro not in cid:
            continue
        wd = os.path.join(s, 'c', cid)
        os.makedirs(wd, exist_ok=True)
        for f in os.listdir(wd):
            os.remove(os.path.join(wd, f))
        t0 = time.time()
        for _ in range(6):   # 127 = outro processo a extrair o AppImage em /tmp ao mesmo tempo
            r = subprocess.run(['timeout', '30', APP, '--appimage-extract-and-run'] + argv, cwd=wd, env=env, capture_output=True, text=True, errors='replace')
            if r.returncode != 127:
                break
            time.sleep(1.0)
        err = '\n'.join(l for l in r.stderr.splitlines() if 'File exists and file size' not in l)
        pedacos = r.stdout.split(RECUSA)
        item = {'argv': [x[len(a) + 1:] if x.startswith(a + '/') else x for x in argv], 'exit': r.returncode, 'recusas': len(pedacos) - 1,
                'stdout': ''.join(pedacos).strip(), 'stderr': err[:2000], 'ficheiros': sorted(os.listdir(wd)), 'seg': round(time.time() - t0, 2)}
        dj = os.path.join(wd, D)
        if os.path.exists(dj):
            txt = open(dj, encoding='utf8', errors='replace').read()
            try:
                j = json.loads(txt)
                fr = j['frames']
                if isinstance(fr, dict):
                    fr = [dict(v, filename=n2) for n2, v in fr.items()]
                item['frames'] = [{'nome': f['filename'], 'frame': f['frame'], 'sss': f['spriteSourceSize'], 'src': f['sourceSize'], 'trimmed': f['trimmed'], 'dur': f['duration']} for f in fr]
                item['meta'] = {x: y for x, y in j['meta'].items() if x != 'app'}
            except Exception as e:
                item['json_erro'] = str(e)
                item['json_cru'] = txt[:2000]
            i = txt.find('"slices"')
            item['slices_cru'] = txt[i:i + 3000] if i >= 0 else None
        res[cid] = item
    json.dump(res, open(saida, 'w'), indent=0, ensure_ascii=False)
    return res


if __name__ == '__main__':
    if sys.argv[1] == 'gerar':
        gerar(sys.argv[2])
    else:
        correr(sys.argv[2], sys.argv[3], sys.argv[4] if len(sys.argv) > 4 else None)
