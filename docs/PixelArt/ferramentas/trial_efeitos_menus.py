"""Ajustes, Convolution Matrix, Despeckle e Text do Aseprite Trial 1.3.18.6 (R-A, R-C, R-D, R-T do
P8_plano §5). Caixa preta, so corrido (02/11). Reusa trial_elipse_grelha.py (abre/fecha/shot/cc).

uso: PH2D_TRIAL_DISPLAY=:94 python3 -I trial_efeitos_menus.py <dir> <modo> [args]
     modos: gen (escreve as entradas .ase) · read <ficheiro.ase> <zoom> <cell> <tag> (abre, le, fecha)
     o resto (diálogos) corre-se por funcoes deste modulo importadas num driver curto (ver fixturas).

Entradas (.ase pela spec publica, como trial_elipse_grelha.write_red):
  adj.ase   256 x 8 RGB: linha 0 rampa R (G=B=0), 1 G, 2 B, 3 cinza; 4 matizes (ver ramp_rows);
            5..7 cores semeadas (random.Random(8)).
  idx.ase   16 x 1 Indexed, paleta de 16 cores semeadas, celula i = indice i.
  imp.ase   15 x 15 preto com 1 branco em (7,7) e outro no canto (0,0); edge.ase 15 x 15 metade esquerda
            (x<7) branca. spk.ase 24 x 24 cinza 128 com sal-e-pimenta semeado (random.Random(24), p=0.2).
Zoom: tecla 2 = 200 % = 4 px de tela por celula (screen_scale 2); tecla 4 = 800 % = 16 px.
Lancador: Xvfb :94 -screen 0 1600x1000x24 -nolisten tcp & ; env -u WAYLAND_DISPLAY DISPLAY=:94
SDL_VIDEODRIVER=x11 HOME=<scratch>/trial94/home ~/.local/opt/aseprite-trial/Aseprite-trial.AppImage &
"""
import colorsys, json, os, random, struct, sys, time, zlib
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_elipse_grelha as drv   # noqa: E402

D = drv.D
xdo, key, shot = drv.xdo, drv.key, drv.shot


def write_ase(path, pix, palette=None):
    """pix = lista de linhas; RGB: (r,g,b) por celula; Indexed: int por celula + palette [(r,g,b)]"""
    h, w = len(pix), len(pix[0])
    _s, _c = drv._string, drv._chunk
    if palette is None:
        raw = b''.join(bytes([*p, 255]) for row in pix for p in row); depth = 32
        chunks = []
    else:
        raw = bytes(v for row in pix for v in row); depth = 8
        pal = struct.pack('<III8x', len(palette), 0, len(palette) - 1) + b''.join(
            struct.pack('<HBBBB', 0, *c, 255) for c in palette)
        chunks = [_c(0x2019, pal)]
    chunks += [_c(0x2007, struct.pack('<HHI8x', 1, 0, 0)),
               _c(0x2004, struct.pack('<HHHHHHB3x', 3 if palette is None else 1, 0, 0, 0, 0, 0, 255) + _s('src')),
               _c(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', w, h)
                  + zlib.compress(raw))]
    fr = b''.join(chunks)
    body = struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), 100, len(chunks)) + fr
    ncol = 0 if palette is None else len(palette)
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, 1, w, h, depth, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0 if palette is None else 255, ncol, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + body)


def ramp_rows():
    rows = [[(v, 0, 0) for v in range(256)], [(0, v, 0) for v in range(256)],
            [(0, 0, v) for v in range(256)], [(v, v, v) for v in range(256)]]
    hue = []
    for i in range(256):
        h = (i % 36) / 36.0
        s, v = [(1, 1), (0.5, 0.6), (1, 0.5), (0.3, 0.8), (0.7, 0.3), (1, 0.75), (0.2, 0.4), (0.6, 1)][(i // 36) % 8]
        hue.append(tuple(round(c * 255) for c in colorsys.hsv_to_rgb(h, s, v)))
    rows.append(hue)
    rnd = random.Random(8)
    rows += [[tuple(rnd.randrange(256) for _ in range(3)) for _ in range(256)] for _ in range(3)]
    return rows


_r16 = random.Random(16)
IDX_PAL = [tuple(_r16.randrange(256) for _ in range(3)) for k in range(16)]


def speckle():
    rnd = random.Random(24)
    out = []
    for _y in range(24):
        row = []
        for _x in range(24):
            r = rnd.random()
            row.append((0, 0, 0) if r < 0.1 else (255, 255, 255) if r < 0.2 else (128, 128, 128))
        out.append(row)
    return out


def gen():
    write_ase(os.path.join(D, 'adj.ase'), ramp_rows())
    write_ase(os.path.join(D, 'idx.ase'), [list(range(16))], IDX_PAL)
    imp = [[(255, 255, 255) if (x, y) in ((7, 7), (0, 0)) else (0, 0, 0) for x in range(15)] for y in range(15)]
    write_ase(os.path.join(D, 'imp.ase'), imp)
    write_ase(os.path.join(D, 'edge.ase'), [[(255, 255, 255) if x < 7 else (0, 0, 0) for x in range(15)] for _ in range(15)])
    write_ase(os.path.join(D, 'spk.ase'), speckle())


def setup(cw, ch, cell, locate=True):
    """dimensoes do sprite aberto; locate=True acha o canvas na tela (sem dialogo por cima)"""
    drv.CW, drv.CH, drv.CELL = cw, ch, cell
    drv.W, drv.H = cw, ch
    if locate:
        drv.rebbox('bb.png')


def grid(a):
    """celulas lidas como [[r,g,b]] linha a linha"""
    return drv.cells(a)


def menu(*clicks, t=0.5):
    for c in clicks:
        xdo('mousemove', *c, 'click', 1); time.sleep(t)


# ── os dialogos (coordenadas de tela 1600x1000, screen_scale 2; dialogo centrado) ─────────────────────
EDIT = (65, 10)
ADJ = (110, 518)        # Edit › Adjustments ▸
ADJ_ITEMS = {'bc': (510, 518), 'hs': (490, 540), 'curve': (480, 562)}
INVERT = (95, 496)
FXM = (78, 540)         # Edit › FX ▸


def field(xy, v):
    xdo('mousemove', *xy, 'click', '--repeat', 2, '--delay', 60, 1); time.sleep(0.2)
    key('ctrl+a', t=0.1); xdo('type', '--', str(v)); key('Tab', t=0.4)


def ok_read(ok_xy, tag, undo=True):
    """OK do dialogo, fotografia, leitura das celulas; desfaz (Ctrl+Z) para o caso seguinte"""
    xdo('mousemove', *ok_xy, 'click', 1); time.sleep(0.7)
    g = grid(shot(tag + '.png'))
    if undo:   # o Ctrl+Z so pega com o rato sobre o editor (medido: estacionado no canto nao desfez)
        xdo('mousemove', 856, 700); key('ctrl+z', t=0.6)
    return g


def open_adj(item):
    menu(EDIT, ADJ, ADJ_ITEMS[item])


def bc(b, c, tag):
    open_adj('bc'); field((877, 446), b); field((877, 510), c)
    return ok_read((971, 420), tag)


HS_MODE = {'HSV': (655, 417), 'HSL': (837, 417), 'HSV+': (655, 442), 'HSL+': (837, 442)}


def hs(mode, h, s, l, tag):
    """Hue/Saturation: botoes de modo no topo; campos H S L(V) A em x=881, y=482/512/542/572"""
    open_adj('hs'); menu(HS_MODE[mode], t=0.3)
    for y, v in ((482, h), (512, s), (542, l)):
        field((881, y), v)
    return ok_read((987, 420), tag)


# Color Curve: caixa (0,0) em tela (627,607), (255,255) em (866,418); arrastar a meio da recta poe um ponto
CURVE0, CURVE1 = (627, 607), (866, 418)


def curve_xy(x, y):
    return (round(CURVE0[0] + x / 255 * (CURVE1[0] - CURVE0[0])),
            round(CURVE0[1] + y / 255 * (CURVE1[1] - CURVE0[1])))


def curve(src, dst, tag):
    open_adj('curve')
    a, b = curve_xy(*src), curve_xy(*dst)
    xdo('mousemove', *a); time.sleep(0.2); xdo('mousedown', 1); time.sleep(0.1)
    for k in range(1, 7):
        xdo('mousemove', a[0] + (b[0] - a[0]) * k // 6, a[1] + (b[1] - a[1]) * k // 6); time.sleep(0.03)
    time.sleep(0.2); xdo('mouseup', 1); time.sleep(0.3)
    shot(tag + '_dlg.png', park=False)
    return ok_read((933, 420), tag)


def palette_panel(a, n):
    """as n primeiras cores do painel de paleta (5 colunas de 24 px a partir de (22,110))"""
    return [a[110 + 24 * (k // 5), 22 + 24 * (k % 5)].tolist() for k in range(n)]


def open_fx(item_xy):
    menu(EDIT, FXM, item_xy)


CONV_ITEM = (510, 562)  # Edit › FX › Convolution Matrix... (F9)
DESP_ITEM = (535, 584)  # Edit › FX › Despeckle (Median Filter)...
# lista de kernels (39, ordem da lista; os dois ultimos tem o nome cortado na lista)
KERNELS = ['brightness', 'contrast', 'negative', 'blur-3x3', 'blur-3x3-hard', 'blur-5x5', 'blur-7x7',
           'blur-9x9', 'blur-17x17', 'blur-5x3-left', 'blur-17x3-left', 'blur-3x17-top',
           'blur-5x5-diagonal(\\)', 'blur-5x5-diagonal(/)', 'sharpen-3x3', 'sharpen-5x5', 'sharpen-7x7',
           'edges-find', 'edges-find-horizontal', 'edges-find-vertical', 'misc-contour', 'misc-texturize',
           'misc-emboss', 'misc-marmolize', 'misc-rock', 'misc-rock-edges', 'drunk-3x3_x', 'drunk-3x3_+',
           'drunk-5x5_x', 'drunk-5x5_+', 'drunk-7x7_x', 'drunk-7x7_+', 'drunk-9x9_x', 'drunk-9x9_+',
           'drunk-17x17_x', 'drunk-17x17_+', 'drunk-17x17_o', 'outline-transparent-layer-(1)',
           'outline-transparent-layer-(2)']


def conv(k, tag, tiled=False):
    """dialogo Convolution Matrix: clique na lista, Home, Down k vezes; OK (933,404); Tiled (895,621)"""
    open_fx(CONV_ITEM)
    xdo('mousemove', 700, 424, 'click', 1); time.sleep(0.2); key('Home', t=0.15)
    for _ in range(k):
        key('Down', t=0.06)
    time.sleep(0.4)
    if tiled:
        menu((895, 621), t=0.3)
    subprocess_shot(tag)
    g = ok_read((933, 404), tag)
    if tiled:   # o Tiled fica lembrado: desliga
        open_fx(CONV_ITEM); menu((895, 621), t=0.3); menu((933, 488), t=0.4)
    return g


def subprocess_shot(tag):
    import subprocess
    subprocess.run(['import', '-window', 'root', '-crop', '380x290+610+357', os.path.join(D, tag + '_dlg.png')],
                   env=drv.ENV, check=True)


DESP_W, DESP_H, DESP_OK, DESP_TILED = (779, 404), (779, 442), (885, 404), (845, 621)


def despeckle(w, h, tag, tiled=False):
    """Despeckle: campos W e H (coordenadas medidas na foto 86_)"""
    open_fx(DESP_ITEM)
    field(DESP_W, w); field(DESP_H, h)
    if tiled:
        menu(DESP_TILED, t=0.3)
    subprocess_shot(tag)
    return ok_read(DESP_OK, tag)


def gen2():
    """2.a rodada: entradas CINZA para os pesos negativos (impg = 255 em (7,7) sobre 128; stepg = 64|192
    com x<7 = 64; imp0 = 0 em (7,7) sobre 128) e bc2.ase = a rampa cinza 256x1"""
    G = (128, 128, 128)
    write_ase(os.path.join(D, 'impg.ase'), [[(255,) * 3 if (x, y) == (7, 7) else G for x in range(15)] for y in range(15)])
    write_ase(os.path.join(D, 'imp0.ase'), [[(0,) * 3 if (x, y) == (7, 7) else G for x in range(15)] for y in range(15)])
    write_ase(os.path.join(D, 'stepg.ase'), [[(64,) * 3 if x < 7 else (192,) * 3 for x in range(15)] for _ in range(15)])


if __name__ == '__main__' and len(sys.argv) > 2:
    if sys.argv[2] == 'gen':
        gen()
    elif sys.argv[2] == 'gen2':
        gen2()


# ── Text (ferramenta T): barra = fonte (290,71) · tamanho (458,71) · B I · «...» (587,71) com
# Antialias (desligado por omissao) / Hinting (ligado) / Ligatures · Fill/Stroke · 0pt (fotos 85_*) ──
FONT_Y = {'DejaVu Sans': 497, 'DejaVu Sans Mono': 575, 'DejaVu Serif': 601}   # na lista de fontes
_AA = [False]   # o Trial LEMBRA o Antialias entre corridas: comece com o HOME limpo (medido 02/11)


def text_case(font, size, aa, cell):
    xdo('mousemove', 290, 71, 'click', 1); time.sleep(0.7)
    xdo('mousemove', 300, FONT_Y[font], 'click', 1); time.sleep(0.5)
    xdo('mousemove', 458, 71, 'click', '--repeat', 2, '--delay', 60, 1); time.sleep(0.2)
    key('ctrl+a', t=0.1); xdo('type', str(size)); key('Return', t=0.3)
    if aa != _AA[0]:
        xdo('mousemove', 587, 71, 'click', 1); time.sleep(0.4)
        xdo('mousemove', 590, 97, 'click', 1); time.sleep(0.3); key('Escape', t=0.3)
        _AA[0] = aa
    xdo('mousemove', *drv.cc(*cell), 'click', 1); time.sleep(0.6)
    xdo('type', 'Ag1'); time.sleep(0.3); key('Return', t=0.6)
