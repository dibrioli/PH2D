"""Rodada 3 no Aseprite Trial 1.3.18.6 (caixa preta, so corrido): Convolution Matrix sobre as entradas
de gen3() (17-largos em 40x40, saturados em baixo contraste, Tiled). Grava <dir>/r3.json a cada caso.
uso: PH2D_TRIAL_DISPLAY=:96 python3 -I trial_efeitos_r3.py <dir> conv
"""
import json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_efeitos_menus as m   # noqa: E402

D = m.D
OUT = os.path.join(D, 'r3.json')
JOBS = ([(f, 40, k, False) for f in ('impg40', 'imp040', 'stepg40', 'imp40') for k in (8, 10, 11, 34, 35, 36)]
        + [(f, 15, k, False) for f in ('impl', 'imp0l', 'stepl') for k in (21, 22, 24, 17, 18, 19)]
        + [(f, 15, k, False) for f in ('ramp15', 'vramp15') for k in (17, 18, 19)]
        + [('stepg', 15, k, True) for k in (3, 14)])


def load():
    return json.load(open(OUT)) if os.path.exists(OUT) else {}


def save(out):
    json.dump(out, open(OUT + '.tmp', 'w')); os.rename(OUT + '.tmp', OUT)


def run_conv():
    out = load(); conv = out.setdefault('conv', {})
    cur = None
    for f, n, k, tiled in JOBS:
        key = f'{f}:{k}' + (':tiled' if tiled else '')
        if key in conv:
            continue
        if cur != f:
            m.drv.fecha(); m.setup(n, n, 16, False); m.drv.abre(f + '.ase', '4'); cur = f
            base = m.grid(m.shot('cbase.png'))
            conv.setdefault(f'{f}:input', base)
        g = m.conv(k, f'cv3_{f}_{k}' + ('_t' if tiled else ''), tiled=tiled)
        ok = m.grid(m.shot('cbase.png')) == base
        conv[key] = g
        if not ok:
            conv[key + ':undo_failed'] = True
        save(out)
        print(key, m.KERNELS[k], 'undo_ok' if ok else 'UNDO FALHOU', [g[n // 2][i][0] for i in range(n)], flush=True)
        if not ok:
            m.drv.fecha(); cur = None


CLIP = """import sys
from PyQt6.QtWidgets import QApplication
app = QApplication(sys.argv[:1]); md = app.clipboard().mimeData()
open(sys.argv[1], 'wb').write(bytes(md.data('image/png')))
"""


def clip_rgba(tag):
    """Ctrl+A, Ctrl+C, o PNG do clipboard X (alvo image/png que o Trial exporta) lido pelo Qt; Ctrl+D.
    O Trial NAO grava (medido: «Save operation is not supported in trial version»); o clipboard da RGBA exacto."""
    import subprocess
    from PIL import Image
    m.xdo('mousemove', 856, 700); m.key('ctrl+a', t=0.3); m.key('ctrl+c', t=0.6)
    p = os.path.join(D, tag + '_clip.png')
    open(os.path.join(D, 'clip.py'), 'w').write(CLIP)
    subprocess.run(['python3', '-I', os.path.join(D, 'clip.py'), p], env=dict(m.drv.ENV, QT_QPA_PLATFORM='xcb'),
                   check=True, timeout=20)
    m.key('ctrl+d', t=0.3)
    im = Image.open(p).convert('RGBA')
    w, h = im.size
    return [[list(im.getpixel((x, y))) for x in range(w)] for y in range(h)]


def sentinel():
    """copia stepg.ase (64|192 opaco) para o clipboard: um Ctrl+C de imagem toda transparente NAO substitui
    o clipboard (medido: H/S alpha -100 devolveu a copia anterior); resultado == SENTINEL => ilegivel"""
    m.drv.fecha(); m.setup(15, 15, 16, False); m.drv.abre('stepg.ase', '4')
    return clip_rgba('sentinel')


A_BTN = (956, 525)   # botao do canal A no dialogo Convolution Matrix (foto cv3_*_dlg.png)


def run_alpha():
    out = load(); al = out.setdefault('alpha', {})
    if '--redo' in sys.argv:
        out['alpha_pass2_without_sentinel'] = dict(al); al.clear()
    # cada caso num documento ACABADO DE ABRIR: o Ctrl+A/Ctrl+D da leitura entra na fila de undo e o Ctrl+Z
    # desfazia a seleccao, nao o filtro (medido: 1.a passada encadeou os 8 casos)
    def fresh():
        sentinel()
        m.drv.fecha(); m.setup(15, 15, 16, False); m.drv.abre('alfa15.ase', '4')
    fresh()
    al['alfa15:input'] = clip_rgba('a3_input')
    for a_off in (False, True):
        for k in (2, 3, 37, 38):
            key = f'alfa15:{k}' + (':A_off' if a_off else '')
            if key in al:
                continue
            m.open_fx(m.CONV_ITEM)
            m.xdo('mousemove', 700, 424, 'click', 1); m.time.sleep(0.2); m.key('Home', t=0.15)
            for _ in range(k):
                m.key('Down', t=0.06)
            if a_off:
                m.menu(A_BTN, t=0.3)
            m.subprocess_shot(f'a3_{k}_{int(a_off)}')
            m.menu((933, 404), t=0.8)            # OK
            al[key] = clip_rgba(f'a3_{k}_{int(a_off)}')
            if a_off:   # o estado dos canais fica lembrado: religa e cancela
                m.open_fx(m.CONV_ITEM); m.menu(A_BTN, t=0.3); m.subprocess_shot(f'a3_{k}_restore')
                m.menu((933, 488), t=0.4)
            save(out)
            fresh()
            print(key, m.KERNELS[k], [c[3] for c in al[key][7]], [c[0] for c in al[key][7]], flush=True)


def run_hs_alpha():
    """Hue/Saturation (modo por omissao) so com o campo Alpha (881,572), sobre alfa15, lido pelo clipboard"""
    out = load(); hs = out.setdefault('hs_alpha', {})
    if '--redo' in sys.argv:
        out['hs_alpha_without_sentinel'] = dict(hs); hs.clear()
    for a in (-50, 50, -100, 100):
        if f'alfa15:A{a}' in hs:
            continue
        out['sentinel'] = sentinel()
        m.drv.fecha(); m.setup(15, 15, 16, False); m.drv.abre('alfa15.ase', '4')
        m.open_adj('hs'); m.field((881, 572), a)
        m.subprocess_shot(f'hs3_{a}')
        m.menu((987, 420), t=0.8)
        hs[f'alfa15:A{a}'] = clip_rgba(f'hs3_{a}')
        save(out)
        print(a, [c for c in hs[f'alfa15:A{a}'][7][5:12]], flush=True)



def write_ase_multi(path, w, h, frames):
    """RGB, 2 camadas (L0 = fundo cheio, L1 = cel parcial); frames = [[(x, y, pix) por camada], ...]"""
    import struct, zlib
    _s, _c = m.drv._string, m.drv._chunk
    out = b''
    for fi, cels in enumerate(frames):
        chunks = []
        if fi == 0:
            chunks.append(m.drv._chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0)))
            for name in ('L0', 'L1'):
                chunks.append(_c(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + _s(name)))
        for li, (x, y, pix) in enumerate(cels):
            raw = b''.join(bytes([*p, 255]) for row in pix for p in row)
            chunks.append(_c(0x2005, struct.pack('<HhhBHh5x', li, x, y, 255, 2, 0)
                             + struct.pack('<HH', len(pix[0]), len(pix)) + zlib.compress(raw)))
        fr = b''.join(chunks)
        out += struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), 100, len(chunks)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(out), 0xA5E0, len(frames), w, h, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + out)


def gen_multi():
    """multi.ase 15x15, 2 quadros x 2 camadas: L0 cheio (q1: 40, q2: 80), L1 cel 7x15 em x=0 (q1: 160, q2: 200)"""
    full = lambda v: [[(v, v, v)] * 15 for _ in range(15)]
    half = lambda v: [[(v, v, v)] * 7 for _ in range(15)]
    write_ase_multi(os.path.join(D, 'multi.ase'), 15, 15, [[(0, 0, full(40)), (0, 0, half(160))],
                                                         [(0, 0, full(80)), (0, 0, half(200))]])


TEXT_CELL = (10, 10)


def run_text():
    """D: Text (T) «Ag1|» DejaVu Sans sobre white160.ase (160x100 branco) a 400 % (8 px/celula), FG preto;
    um documento novo por caso; grelha inteira como linhas hex de R (2 caracteres por celula)"""
    out = load(); tx = out.setdefault('text', {})
    m.write_ase(os.path.join(D, 'white160.ase'), [[(255, 255, 255)] * 160 for _ in range(100)])
    cases = [(s, False) for s in (9, 10, 11, 13, 14, 15, 17, 18, 20, 24, 32)] + [(s, True) for s in (12, 16, 24, 32)]
    # UM documento; entre casos Ctrl+Z e conferencia do branco (fechar com o texto feito deixou o aviso
    # «Saving changes» por fechar e o Ctrl+O seguinte abriu lixo, medido 02/11)
    m.setup(160, 100, 8, False); m.drv.abre('white160.ase', '3')
    if m.shot('t_fg.png')[915, 20].tolist() != [0, 0, 0]:   # FG preto pelo dialogo de cor (como drv.ensure_fg)
        m.xdo('mousemove', 64, 915, 'click', '--repeat', 2, '--delay', 80, 1); m.time.sleep(0.8)
        m.xdo('mousemove', 316, 748, 'click', '--repeat', 2, '--delay', 80, 1); m.time.sleep(0.3)
        m.key('ctrl+a'); m.xdo('type', '000000'); m.key('Return')
        m.xdo('mousemove', 588, 740, 'click', 1); m.time.sleep(0.5)
    assert m.shot('t_fg.png')[915, 20].tolist() == [0, 0, 0], 'FG nao ficou preto'
    m.xdo('mousemove', 856, 700); m.key('t', t=0.5)
    for size, aa in cases:
        key = f'DejaVu Sans:{size}:{"aa" if aa else "noaa"}'
        if key in tx:
            continue
        m.text_case('DejaVu Sans', size, aa, TEXT_CELL, 'Ag1|')
        m.key('Escape', t=0.4)
        g = m.grid(m.shot(f't3_{size}_{int(aa)}.png'))
        tx[key] = {'anchor_cell': list(TEXT_CELL),
                   'rows_hexR': [''.join('%02x' % c[0] for c in row) for row in g],
                   'non_gray_cells': sum(1 for row in g for c in row if not (c[0] == c[1] == c[2]))}
        m.xdo('mousemove', 856, 700); m.key('ctrl+z', t=0.6)
        if any(c != [255, 255, 255] for row in m.grid(m.shot('t_back.png')) for c in row):
            tx[key]['undo_failed'] = True
        save(out)
        ink = [(x, y) for y, row in enumerate(g) for x, c in enumerate(row) if c[0] < 255]
        print(key, 'bbox', (min(p[0] for p in ink), min(p[1] for p in ink), max(p[0] for p in ink),
                            max(p[1] for p in ink)) if ink else None, flush=True)


if __name__ == '__main__':
    {'conv': run_conv, 'alpha': run_alpha, 'hs_alpha': run_hs_alpha, 'text': run_text}[sys.argv[2]]()


def assemble(d96, d97, dst):
    """junta os JSON brutos das duas sessoes (:96 conv/C/text, :97 alfa/B) na fixtura; grelhas cinza -> R"""
    j = lambda *p: json.load(open(os.path.join(*p)))
    r96, r97 = j(d96, 'r3.json'), j(d97, 'r3.json')
    conv = r96['conv']
    assert all(p[0] == p[1] == p[2] for g in conv.values() if isinstance(g, list) for row in g for p in row)
    R = lambda g: [[p[0] for p in row] for row in g]
    sel = lambda pred: {k: R(g) for k, g in conv.items() if pred(k)}
    return {
        'conv_17wide_40x40': sel(lambda k: k.split(':')[0].endswith('40')),
        'conv_low_contrast_15x15': sel(lambda k: k.split(':')[0] in ('impl', 'imp0l', 'stepl', 'ramp15', 'vramp15')),
        'conv_tiled_15x15': sel(lambda k: k.split(':')[0] == 'stepg'),
        'alpha': r97['alpha'], 'alpha_repeat_without_sentinel': r97['alpha_pass2_without_sentinel'],
        'sentinel_stepg_rgba': r97['sentinel'], 'hs_alpha': r97['hs_alpha'],
        'hs_alpha_first_pass': r97['hs_alpha_without_sentinel'],
        'invert_indexed': {'first_ok_only': j(d97, 'b1.json'), 'apply_then_ok': j(d97, 'b1b.json')},
        'gradient_indexed': {'fg_bg_both_idx2': j(d97, 'b2.json'), 'fg2_bg9': j(d97, 'b2b.json'),
                             'rgb_control_stepg': j(d97, 'b2_rgbctl.json')},
        'outline_bg_color': j(d97, 'b3.json'), 'targets': j(d96, 'c.json'), 'text': r96['text'],
    }
