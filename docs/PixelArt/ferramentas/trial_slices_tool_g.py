"""P11 rodada 2 (G1-G9): uma sessao por caso, le o dialogo Slice Properties e fotografa. Caixa preta; o Trial nao grava.
uso: python3 -I trial_slices_tool_g.py <work> <dir .ase> fix          fixturas extra em <work>/fx
     python3 -I trial_slices_tool_g.py <work> <dir .ase> <caso...>    grava <work>/G_<caso>_*.png
Display e copia extraida: os de trial_slices_tool.py (up/down a parte)."""
import os, struct, subprocess, sys, zlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_slices as g
import trial_slices_tool as t
from PIL import Image

W, ASE = os.path.abspath(sys.argv[1]), sys.argv[2]
FX = os.path.join(W, 'fx')
SPRITE = 'click 121 10; wait .3'
ROT = SPRITE + '; click 170 164; wait .35'
ALL, NONE, TRANSFORM, PROPS, XBTN = 'click 178 70', 'click 218 70', 'click 783 71', 'click 917 70', 'click 942 70'
OK, CANCEL = 'click 869 615; wait .4', 'click 997 615; wait .25'
OK_X1 = 'click 869 720; wait .4'  # expandido na 1.a abertura
CANCEL_X2 = 'click 997 666; wait .25'  # expandido nas seguintes


def do(s):
    t.do(W, s)


def sess(path, tool='key shift+c; setup'):
    os.environ['BOOT'] = '4'
    subprocess.run(['timeout', '40', sys.executable, '-I', t.__file__, W, 'open', path], check=True)
    do(tool)


def fx(n):
    return os.path.join(FX, n + '.ase')


def ase(n):
    return os.path.join(ASE, n + '.ase')


def stack(out, tags, cols=2):
    os.environ['COLS'] = str(cols)
    do('stack ' + out + ' ' + ' '.join(tags))


def rd(tag, x, y, f=None):
    pre = f'frame {f}; ' if f is not None else ''
    do(f'{pre}pdbl {x} {y}; dl3 {tag}; {CANCEL}')
    return tag


def rds(pre, pts):
    """pts: (nome, quadro|None, x, y)"""
    return [rd(f'{pre}_{n}', x, y, f) for n, f, x, y in pts]


def thumbs(pre, frames, cols=None):
    tags = []
    for f in frames:
        do(f'frame {f}; shot {pre}_t{f} 596 112 512 512')
        tags.append(f'{pre}_t{f}')
    out = os.path.join(W, pre + '_thumbs.png')
    cmd = ['montage'] + [os.path.join(W, n + '.png') for n in tags] + ['-tile', f'{cols or len(tags)}x', '-geometry', '256x256+2+2', out]
    subprocess.run(cmd, check=True)


def lista(tag):
    do(f'click 749 70; wait .3; shot {tag} keep 250 55 500 120; key Escape; wait .2')


def corpo(tag):
    do(f'shot {tag} 160 780 400 50')


def bbox():
    p = os.path.join(W, '_bb.png')
    subprocess.run(['import', '-window', 'root', p], env=t.env(W), check=True)
    im = Image.open(p).convert('RGB')
    bg = im.getpixel((172, 104))
    x0, y0 = 10**6, 10**6
    px = im.load()
    for y in range(100, 722):
        for x in range(170, 1540):
            if px[x, y] != bg:
                x0, y0 = min(x0, x), min(y0, y)
    return x0 + 1, y0 + 1


def zoom100():
    do('mv 856 400; key 1; wait .4; mv 1590 700')
    return bbox()


def rd100(org, tag, cx, cy, f=None):
    pre = f'frame {f}; ' if f is not None else ''
    do(f'{pre}dbl {org[0] + 2 * int(cx) + 1} {org[1] + 2 * int(cy) + 1}; dl3 {tag}; {CANCEL}')
    return tag


# ---------- fixturas
ODD3 = {0: [
    {'name': 'e', 'flags': 3, 'color': (0, 200, 0, 255), 'keys': [g.k(0, 4, 6, 15, 9, center=(3, 1, 9, 7), pivot=(7, 3))]},
    {'name': 'f', 'flags': 3, 'color': (0, 0, 255, 255), 'keys': [g.k(0, 35, 7, 16, 11, center=(2, 3, 11, 5), pivot=(5, -3))]},
    {'name': 'g', 'flags': 3, 'color': (255, 0, 0, 255), 'keys': [g.k(0, 7, 33, 17, 20, center=(5, 2, 7, 13), pivot=(9, 19))]},
    {'name': 'h', 'flags': 3, 'color': (255, 128, 0, 255), 'keys': [g.k(0, 40, 34, 19, 13, center=(5, 3, 9, 7), pivot=(-5, 11))]},
]}
PV = {0: [{'name': 'pv', 'flags': 2, 'color': (255, 0, 0, 255), 'keys': [
    g.k(0, 8, 8, 24, 24, pivot=(12, 12)), g.k(2, 30, 30, 24, 24, pivot=(6, 18)), g.k(4, 8, 30, 24, 24, pivot=(12, 12))]}]}


def arte_top():
    px = bytearray()
    for y in range(g.H):
        for x in range(g.W):
            if 16 <= x < 32 and 16 <= y < 24:
                px += bytes((250, 220, 40, 255))
            elif 24 <= x < 40 and 24 <= y < 32:
                px += bytes((220, 40, 220, 255))
            else:
                px += bytes(4)
    return bytes(px)


def tr_ase(path):
    """2 camadas (base 0 cheia, top 1 com 2 blocos), 3 quadros; top sem cel no quadro 2; slice s com tecla so no 0."""
    def cel(layer, art):
        return g.chunk(0x2005, struct.pack('<HhhBHh5x', layer, 0, 0, 255, 2, 0) + struct.pack('<HH', g.W, g.H) + zlib.compress(art))
    corpo_ = b''
    top = arte_top()
    for f in range(3):
        cs = []
        if f == 0:
            cs.append(g.chunk(0x2007, struct.pack('<HHI8x', 1, 0, 0)))
            cs.append(g.chunk(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + g.string('base')))
            cs.append(g.chunk(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + g.string('top')))
        cs.append(cel(0, g.arte(f)))
        if f < 2:
            cs.append(cel(1, top))
        if f == 0:
            cs += g.slice_chunk({'name': 's', 'color': (255, 0, 0, 255), 'keys': [g.k(0, 12, 12, 32, 24)]})
        corpo_ += struct.pack('<IHHH2xI', 16 + sum(map(len, cs)), 0xF1FA, len(cs), 100, len(cs)) + b''.join(cs)
    hdr = struct.pack('<IHHHHHIHII', 128 + len(corpo_), 0xA5E0, 3, g.W, g.H, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + corpo_)


def fixturas():
    os.makedirs(FX, exist_ok=True)
    g.escreve(fx('odd3'), 2, ODD3)
    g.escreve(fx('pv'), 6, PV)
    tr_ase(fx('tr'))


# ---------- G1
def g1(pct):
    sess(fx('odd3'))
    do(f'{SPRITE}; click 165 120; wait .5; fill 760 505 {pct}; fill 760 543 {pct}; shot G1_{pct}_dlg keep 500 300 600 400; click 765 650; wait .8')
    org = zoom100()
    do(f'shot G1_{pct}_bar 0 975 400 25; shot G1_{pct}_tela')
    tags = []
    for n, x, y, w, h in [('e', 4, 6, 15, 9), ('f', 35, 7, 16, 11), ('g', 7, 33, 17, 20), ('h', 40, 34, 19, 13)]:
        cx, cy = (x + w / 2) * pct / 100, (y + h / 2) * pct / 100
        tags.append(rd100(org, f'G1_{pct}_{n}', cx, cy))
    stack(f'G_G1_{pct}', tags)


# ---------- G2
KEYS = [(0, 16, 16), (1, 16, 16), (2, 32, 16), (3, 32, 16), (4, 40, 32), (5, 40, 32)]


def keyreads(pre, keys=KEYS):
    tags = [rd(f'{pre}_f{f}', x, y, f) for f, x, y in keys]
    stack(f'G_{pre}', tags)


def g2(modo, q=3):
    sess(ase('keys'))
    pt = dict((f, (x, y)) for f, x, y in KEYS)[q]
    do(f'frame {q}; pclick {pt[0]} {pt[1]}; wait .2')
    if modo == 'x':
        do(XBTN + '; wait .4')
    elif modo == 'del':
        do(f'pmv {pt[0]} {pt[1]}; key Delete; wait .4')
    else:
        p = t.P(*pt)
        do(f'pclick {pt[0]} {pt[1]} 3; wait .4; click {p[0] + 67} {p[1] + 60}; wait .4')
    name = f'G2_{modo}{q}'
    corpo(name + '_tl')
    lista(name + '_lista')
    thumbs(name, range(6), 3)
    keyreads(name)
    do('key ctrl+z; wait .5')
    lista(name + '_undo_lista')
    keyreads(name + '_undo')


# ---------- G3
U = [('nine', 0, 24, 16), ('pivot', 0, 38, 16), ('both', 0, 24, 46)]
D = [('nine', 0, 24, 16), ('pivot', 0, 45, 16), ('both', 0, 24, 46)]


def g3open():
    sess(ase('full'))
    do(f'{ALL}; wait .2; {PROPS}; wait .5; shot G3_open_a keep 520 350 560 300; click 1045 403; wait .3; shot G3_open_b keep 520 350 560 410; click 997 720; wait .3; shot G3_open_tela')


def g3set(nome, edit, pts=U, frame3=False):
    sess(ase('full'))
    do(f'{ALL}; wait .2; {PROPS}; wait .5; {edit}; dlg G3_{nome}_typed; {OK}; shot G3_{nome}_tela')
    do(f'{NONE}; wait .2; shot G3_{nome}_none')
    tags = rds(f'G3_{nome}', pts)
    if frame3:
        tags.append(rd(f'G3_{nome}_both3', 40, 46, 3))
    stack(f'G_G3_{nome}', [f'G3_{nome}_typed'] + tags)


def g3gest(nome, gest, pts):
    sess(ase('full'))
    do(f'{ALL}; wait .2; {gest}; wait .3; shot G3_{nome}_tela')
    lista(f'G3_{nome}_lista')
    do(f'{NONE}; wait .2; shot G3_{nome}_none')
    stack(f'G_G3_{nome}', rds(f'G3_{nome}', pts))
    do('key ctrl+z; wait .4')
    lista(f'G3_{nome}_undo_lista')
    stack(f'G_G3_{nome}_undo', rds(f'G3_{nome}_undo', [('nine', 0, 16, 16), ('pivot', 0, 48, 16), ('both', 0, 16, 46)]))


def g3single():
    """Width pelo dialogo com uma so slice com 9-slice: o centro acompanha como no arrasto?"""
    sess(ase('full'))
    do(f'pclick 16 16; wait .2; {PROPS}; wait .5; fill 905 485 30; dlg G3_single_typed; {OK}')
    stack('G_G3_single', ['G3_single_typed', rd('G3_single_r', 16, 16)])


def g3cor():
    sess(ase('full'))
    do(f'{ALL}; wait .2; {PROPS}; wait .5; click 1045 403; wait .3; click 853 441; wait .5; shot G3_cor_pop keep; fill 1013 521 ff00ff; key Return; wait .3; shot G3_cor_pop2 keep; click 700 372; wait .3; shot G3_cor_antes keep 520 350 560 410; {OK_X1}; {NONE}; wait .2; shot G3_cor_tela')


# ---------- G4
def g4(nome, f, sel, drag, pt, extra=None):
    sess(fx('pv'))
    do(f'frame {f}; pclick {sel[0]} {sel[1]}; pdrag {drag[0]} {drag[1]} {drag[2]} {drag[3]}; shot G4_{nome}_tela')
    tags = [rd(f'G4_{nome}_r', pt[0], pt[1])]
    if extra:
        tags += [rd(f'G4_{nome}_f{ef}', x, y, ef) for ef, x, y in extra]
    do(f'frame {f}; key ctrl+z; wait .4')
    tags.append(rd(f'G4_{nome}_undo', sel[0], sel[1]))
    stack(f'G_G4_{nome}', tags)


# ---------- G5
def g5(nome, layer_y, frame, ops):
    sess(fx('tr'))
    do(f'frame {frame}; click 290 {layer_y}; wait .3; {TRANSFORM}; wait .3; pclick 28 20; wait .2; shot G5_{nome}_antes; {ops}; wait .4; shot G5_{nome}_depois 160 100 1400 640')
    thumbs(f'G5_{nome}', range(3))
    rd(f'G5_{nome}_r', 28, 24, frame)
    do('key ctrl+z; wait .5')
    thumbs(f'G5_{nome}_undo', range(3))


def g5ctl(nome, tf):
    """Mover no quadro 1 (tecla so no 0): com e sem Transform, ler o dialogo nos 3 quadros num ponto dentro das 2 posicoes."""
    sess(fx('tr'))
    do(f'frame 1; click 290 818; wait .3; {TRANSFORM + "; wait .3; " if tf else ""}pclick 28 20; wait .2; pdrag 28 20 36 28; wait .4')
    stack(f'G_G5_{nome}', [rd(f'G5_{nome}_f{f}', 30, 25, f) for f in range(3)], 3)


def g5pv():
    """Transform, quadro 3 com tecla herdada do 2 e teclas 0,2,4."""
    sess(fx('pv'))
    do(f'frame 3; {TRANSFORM}; wait .3; pclick 40 40; wait .2; pdrag 40 40 44 44; wait .4')
    stack('G_G5_pv', [rd(f'G5_pv_f{f}', x, y, f) for f, x, y in [(0, 20, 20), (1, 20, 20), (2, 40, 40), (3, 44, 44), (4, 20, 40), (5, 20, 40)]])


# ---------- G6
def g6num():
    sess(fx('pv'))
    do(f'frame 0; pclick 20 20; wait .2')
    tags = []
    for n, fld, val in [('piv', '697 560', '-5'), ('x', '697 485', '-10'), ('w', '905 485', '99999'), ('h', '1009 485', '-7'), ('wa', '905 485', 'abc'), ('wb', '905 485', '12abc'), ('xc', '697 485', '1.5'), ('xe', '697 485', '')]:
        arg = f'"{val}"' if val else "''"
        if val == '':
            do(f'{PROPS}; wait .4; click {fld}; wait .15; key ctrl+a; key BackSpace; wait .2; dl3 G6_{n}_t; {OK}; {PROPS}; wait .4; dl3 G6_{n}_r; {CANCEL}')
        else:
            do(f'{PROPS}; wait .4; fill {fld} {arg}; dl3 G6_{n}_t; {OK}; {PROPS}; wait .4; dl3 G6_{n}_r; {CANCEL}')
        tags += [f'G6_{n}_t', f'G6_{n}_r']
    stack('G_G6_num', tags)
    do('shot G6_num_tela')


def g6cor():
    sess(fx('pv'))
    do(f'frame 0; pclick 20 20; wait .2; pdbl 20 20; wait .4; click 1045 403; wait .3; click 853 441; wait .5; shot G6_cor_pop keep; fill 1013 521 ff00ff; key Return; wait .3; shot G6_cor_pop2 keep; click 700 372; wait .3; click 853 495; wait .2; type "hello"; wait .2; fill 836 404 renamed; wait .2; shot G6_cor_antes keep 520 350 560 410; {OK_X1}; shot G6_cor_tela0')
    for f, x, y in [(0, 20, 20), (1, 20, 20), (2, 40, 40), (3, 40, 40), (4, 20, 40), (5, 20, 40)]:
        do(f'frame {f}; pdbl {x} {y}; wait .4; shot G6_cor_f{f} keep 520 300 560 400; {CANCEL_X2}; shot G6_cor_tela{f}')
    stack('G_G6_cor', [f'G6_cor_f{f}' for f in range(6)], 3)
    do('key ctrl+z; wait .4; frame 3; pdbl 40 40; wait .4; shot G6_cor_undo keep 520 300 560 400; ' + CANCEL_X2)


# ---------- G7
F7 = [('nine', 16, 16), ('pivot', 48, 16), ('both', 16, 46)]


def g7(nome, setup_op, tf=None, fix='full'):
    sess(ase(fix))
    do(setup_op)
    org = zoom100()
    do(f'shot G7_{nome}_tela; shot G7_{nome}_bar 0 975 400 25')
    tags = [rd100(org, f'G7_{nome}_{n}', x, y, 0) for n, x, y in F7]
    if tf:
        tags += [rd100(org, f'G7_{nome}_{n}_tf', *tf(x, y), 0) for n, x, y in F7]
    tags.append(rd100(org, f'G7_{nome}_both3', 44, 46, 3))
    stack(f'G_G7_{nome}', tags)


def g7crop():
    sess(ase('full'), 'key m; setup; pdrag 8 8 48 40; wait .3')
    do(f'shot G7_crop_sel; {SPRITE}; click 133 194; wait .8; key shift+c; wait .3')
    org = zoom100()
    do('shot G7_crop_tela; shot G7_crop_bar 0 975 400 25')
    lista('G7_crop_lista')
    # traduzido (x-8,y-8) e nao traduzido
    tags = [rd100(org, f'G7_crop_{n}', x, y, 0) for n, x, y in [('nine_tf', 8, 8), ('pivot_tf', 30, 8), ('both_tf', 4, 30), ('nine_igual', 16, 16), ('pivot_igual', 38, 16)]]
    stack('G_G7_crop', tags)


# ---------- G8
def g8(nome, ops, reads_op=None, frames_op=6, n_undo=1):
    sess(ase('keys'))
    do(ops)
    corpo(f'G8_{nome}_tl')
    thumbs(f'G8_{nome}', range(frames_op), 7)
    do('frame 0; wait .2')
    for _ in range(n_undo):
        do('key ctrl+z; wait .5')
    corpo(f'G8_{nome}_undo_tl')
    thumbs(f'G8_{nome}_undo', range(6), 6)
    keyreads(f'G8_{nome}_undo')


# ---------- G9
def g9pencil(tool, nome):
    sess(ase('full'))
    do(f'pclick 16 16; wait .2; key {tool}; wait .3; shot G9_{nome}_a; pclick 16 16; wait .3; shot G9_{nome}_b; pdrag 12 12 20 20; wait .3; shot G9_{nome}_c; pdbl 48 16; wait .4; shot G9_{nome}_d; key Escape; wait .2; key shift+c; wait .3; shot G9_{nome}_e')
    stack(f'G_G9_{nome}', rds(f'G9_{nome}', [('nine', 0, 16, 16), ('pivot', 0, 48, 16)]))


def g9show():
    sess(ase('full'))
    menu = 'click 379 10; wait .4; click 400 134; wait .5; click 685 222; wait .5; key Escape; wait .2'
    do(f'pclick 16 16; wait .2; {menu}; shot G9_show_off; pdrag 40 40 50 50; wait .3; shot G9_show_b; pdbl 16 16; wait .4; shot G9_show_dbl; key Escape; wait .2; pdrag 48 16 52 20; wait .3; shot G9_show_drag; click 379 10; wait .4; click 400 134; wait .5; shot G9_show_menu keep 300 100 500 300; click 685 222; wait .5; key Escape; wait .2; shot G9_show_on')
    lista('G9_show_lista')


CASES = {
    'G1': lambda: [g1(p) for p in (50, 150, 33)],
    'G2': lambda: [g2('x'), g2('del'), g2('menu'), g2('del', 2)],
    'G3a': lambda: (g3open(), g3set('noop', 'wait .1', frame3=True), g3set('name', 'fill 836 404 zzz', frame3=True), g3set('x', 'fill 697 485 20'), g3set('w', 'fill 905 485 30'), g3cor()),
    'G3b': lambda: (
        g3set('ninechk', 'click 553 521; wait .2'), g3set('ninef', 'fill 697 522 3'),
        g3set('pivchk', 'click 553 559; wait .2'), g3set('pivf', 'fill 697 560 3'),
        g3gest('drag', 'pdrag 24 16 28 20', D), g3gest('edge', 'pdrag 27 16 23 16', U),
        g3gest('del', 'pmv 24 16; key Delete', U), g3gest('x', XBTN, U)),
    'G3c': lambda: (
        g3set('nine4', 'click 553 521; wait .2; fill 697 522 3; fill 801 522 4; fill 905 522 5; fill 1009 522 6'),
        g3set('piv2', 'click 553 559; wait .2; fill 697 560 3; fill 801 560 4'),
        g3set('piv1', 'click 553 559; wait .2; fill 697 560 3'),
        g3single()),
    'G5c': lambda: (g5ctl('ctl', False), g5ctl('tf', True), g5pv()),
    'G4': lambda: (
        g4('r16', 0, (20, 20), (31, 20, 15, 20), (12, 12)),
        g4('l4', 0, (20, 20), (8, 20, 12, 20), (20, 20)), g4('l16', 0, (20, 20), (8, 20, 24, 20), (28, 20)),
        g4('t16', 0, (20, 20), (20, 8, 20, 24), (20, 28)),
        g4('tl4', 0, (20, 20), (8, 8, 12, 12), (20, 20)),
        g4('k3', 3, (40, 40), (53, 40, 49, 40), (40, 40), [(2, 40, 40), (4, 20, 40), (5, 20, 40), (0, 20, 20)])),
    'G5': lambda: (
        g5('move_top', 818, 0, 'pdrag 28 20 36 28'), g5('move_base', 841, 0, 'pdrag 28 20 36 28'),
        g5('grow', 818, 0, 'pdrag 43 35 51 43'), g5('shrink', 818, 0, 'pdrag 43 35 35 27'),
        g5('move_f1', 818, 1, 'pdrag 28 20 36 28')),
    'G6': lambda: (g6num(), g6cor()),
    'G7': lambda: (
        g7('R180', f'{ROT}; click 357 164; wait .6', lambda x, y: (63 - x, 63 - y)),
        g7('R270', f'{ROT}; click 377 208; wait .6', lambda x, y: (y, 63 - x)),
        g7('flipsel', 'key m; pdrag 0 0 32 32; wait .3; key shift+h; wait .5; shot G7_flipsel_a; key shift+c; wait .3'),
        g7('flipcel', 'shot G7_flipcel_a; key shift+h; wait .5; shot G7_flipcel_b'),
        g7crop()),
    'G8': lambda: (
        g8('df', 'frame 2; key alt+c; wait .5', frames_op=5), g8('nf', 'frame 1; key alt+n; wait .5', frames_op=7),
        g8('df2', 'frame 5; key alt+c; wait .5; frame 4; key alt+c; wait .5', frames_op=4, n_undo=2),
        g8('mv', 'click 397 795; wait .3; drag 386 795 470 795; wait .5', frames_op=6),
        g8('rev', 'drag 397 795 445 795; wait .3; key alt+i; wait .5', frames_op=6)),
    'G9': lambda: (g9pencil('b', 'pencil'), g9pencil('v', 'move'), g9show()),
}

if __name__ == '__main__':
    if sys.argv[3] == 'fix':
        fixturas()
    else:
        for c in sys.argv[3:]:
            print('==', c, flush=True)
            CASES[c]()
