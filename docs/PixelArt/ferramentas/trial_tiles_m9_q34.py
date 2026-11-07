"""P10 M9 Q3/Q4 e o seguimento de Q1 no Trial 1.3.18.6: tamanho do canvas, Crop, Trim; vida do tileset; a celula esvaziada em Manual.
Uma sessao por cenario (Xvfb, HOME limpo). Le o mapa e o tileset pela foto. uso: python3 -I trial_tiles_m9_q34.py <dir> [filtro]"""
import json, os, sys, threading, queue, time
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_tiles_m9 as m
OUT = os.path.abspath(sys.argv[1]); FILT = sys.argv[2] if len(sys.argv) > 2 else ''
R = m.tile_row(1, 1, 6, 'R'); G = m.tile_row(3, 1, 6, 'G'); B = m.tile_row(5, 1, 6, 'B')
GRID = {(0, 0): 1, (1, 1): 2, (3, 3): 3, (2, 2): 1}
W_F, H_F = (1315, 170), (1315, 208); ANCHOR_TL = (1397, 175); TRIM = (1205, 405); OK = (1363, 456)
LEFT, TOP = (1305, 316), (1497, 316)
def box(a):
    bg = a[110, 300]
    reg = a[98:948, 170:1150]
    d = np.any(reg != bg, axis=2)
    ys, xs = np.nonzero(d)
    return xs.min() + 170, ys.min() + 98, xs.max() + 170, ys.max() + 98
def read(s, name):
    a = s.shot(name)
    x0, y0, x1, y1 = box(a)
    w, h = int((x1 - x0 - 1) // 8), int((y1 - y0 - 1) // 8)
    return {'w': w, 'h': h, 'mapa': m.canvas(a, x0 + 1, y0 + 1, w, h), 'tiles': m.tiles_panel(a)}
def field(s, p, v):
    s.x('mousemove', *p, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(.2)
    s.key('ctrl+a', t=.1); s.x('type', '--', str(v)); time.sleep(.1); s.key('Tab', t=.2)
def canvas_size(s, w=None, h=None, left=None, top=None, trim=False):
    s.key('c'); time.sleep(.5)
    if w is not None:
        s.x('mousemove', *ANCHOR_TL, 'click', 1); time.sleep(.2)
        field(s, W_F, w); field(s, H_F, h)
    if left is not None:
        field(s, LEFT, left); field(s, TOP, top)
    if trim:
        s.x('mousemove', *TRIM, 'click', 1); time.sleep(.2)
    return s
def ok(s):
    s.shot('dlg.png', park=False)
    s.x('mousemove', *OK, 'click', 1); time.sleep(.7)
def menu(s, mx, ix, iy):
    s.x('mousemove', mx, 10, 'click', 1); time.sleep(.4); s.x('mousemove', ix, iy, 'click', 1); time.sleep(.6)
def sel(s, p0, p1, b):
    """moldura de pixel p0 ate p1 inclusive, sobre o canvas em box b"""
    s.key('m')
    X, Y = b[0] + 1, b[1] + 1
    A = (X + p0[0] * 8 + 1, Y + p0[1] * 8 + 1); B_ = (X + (p1[0] + 1) * 8 - 2, Y + (p1[1] + 1) * 8 - 2)
    s.drag(A, B_)
def c_shrink_grow(s, r, trim):
    r['antes'] = read(s, 'antes.png')
    canvas_size(s, 16, 16, trim=trim); ok(s); r['16'] = read(s, '16.png')
    canvas_size(s, 32, 32); ok(s); r['volta32'] = read(s, 'v32.png')
    s.key('ctrl+z'); s.key('ctrl+z'); time.sleep(.3); r['undo2'] = read(s, 'undo2.png')
def c_nm(s, r):
    canvas_size(s, 20, 20); ok(s); r['20'] = read(s, '20.png')
    canvas_size(s, 32, 32); ok(s); r['volta32'] = read(s, 'v32.png')
def c_of8(s, r):
    canvas_size(s, left=8, top=8); ok(s); r['of8'] = read(s, 'of8.png')
def c_of3(s, r):
    canvas_size(s, left=3, top=3); ok(s); r['of3'] = read(s, 'of3.png')
    # e pintar depois do deslocamento: onde cai a grelha?
def c_neg(s, r):
    canvas_size(s, left=-8, top=-8); ok(s); r['neg8'] = read(s, 'neg8.png')
    canvas_size(s, left=8, top=8); ok(s); r['volta'] = read(s, 'volta.png')
def c_crop(s, r, p0, p1):
    a = s.shot('p.png'); b = box(a)
    sel(s, p0, p1, b); s.shot('sel.png')
    menu(s, 120, 132, 195); r['crop'] = read(s, 'crop.png')
    s.key('ctrl+d'); time.sleep(.2)
    canvas_size(s, left=p0[0], top=p0[1]); ok(s)
    q = read(s, 'cresce1.png'); r['cresce_esq'] = q
    canvas_size(s, 32, 32); ok(s); r['cresce32'] = read(s, 'cresce2.png')
def c_trim(s, r):
    menu(s, 120, 133, 216); r['trim'] = read(s, 'trim.png')
def newdlg(s, name, dropdown=True):
    s.space('n'); time.sleep(.6)
    if dropdown:
        s.x('mousemove', 1025, 455, 'click', 1); time.sleep(.5)
    s.shot(name, park=False)
def q4(s, r, op):
    s.key('Tab'); time.sleep(.4); s.key('Up'); time.sleep(.2)
    s.shot('antes.png')
    if op == 'ctrl': pass
    elif op == 'delete': menu(s, 184, 230, 156)
    elif op == 'flatten': menu(s, 184, 205, 252)
    elif op == 'convert':
        s.x('mousemove', 184, 10, 'click', 1); time.sleep(.4); s.x('mousemove', 230, 178, 'click', 1); time.sleep(.5)
        s.x('mousemove', 410, 200, 'click', 1); time.sleep(.6)
    elif op == 'merge': menu(s, 184, 228, 230)
    s.shot('depois.png')
    newdlg(s, 'dialogo.png')
    # 2.a entrada da lista
    s.x('mousemove', 830, 500, 'click', 1); time.sleep(.5); s.shot('escolhe.png', park=False)
    s.x('mousemove', 851, 603, 'click', 1); time.sleep(.8); s.shot('criada.png')
    r['tiles_apos_criar'] = m.tiles_panel(s.shot('criada2.png'))
def qfollow(s, r, cell_px, st):
    s.key('3'); s.x('mousemove', *m.BOT['manual'], 'click', 1); time.sleep(.3)
    s.key('e'); s.x('mousemove', 230, 72, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(.2); s.key('ctrl+a'); s.key('1'); s.key('Return')
    X, Y = 820, 464; P = lambda p: (X + p[0] * 8 + 3, Y + p[1] * 8 + 3)
    s.drag(P(st[0]), P(st[1]))
    r['esvaziado'] = read(s, 'e.png')
    s.key('b'); s.drag(P(cell_px), P(cell_px))
    r['lapis_na_mesma'] = read(s, 'l.png')
R3 = m.tile_row(3, 2, 5, 'R'); G5 = m.tile_row(5, 2, 5, 'G')
Q1G = {(0, 1): 1, (0, 0): 2, (1, 0): 2, (2, 0): 2}
CASOS = {
 'q3_16_sem_trim': (lambda s, r: c_shrink_grow(s, r, False), False),
 'q3_16_com_trim': (lambda s, r: c_shrink_grow(s, r, True), False),
 'q3_20': (c_nm, False), 'q3_of8': (c_of8, False), 'q3_of3': (c_of3, False), 'q3_neg8': (c_neg, False),
 'q3_crop8': (lambda s, r: c_crop(s, r, (8, 8), (23, 23)), False),
 'q3_crop3': (lambda s, r: c_crop(s, r, (3, 3), (18, 18)), False),
 'q3_trim': (c_trim, False),
 'q4_ctrl': (lambda s, r: q4(s, r, 'ctrl'), True), 'q4_delete': (lambda s, r: q4(s, r, 'delete'), True),
 'q4_flatten': (lambda s, r: q4(s, r, 'flatten'), True), 'q4_convert': (lambda s, r: q4(s, r, 'convert'), True),
 'q4_merge': (lambda s, r: q4(s, r, 'merge'), True),
 'qf_manual_b': (lambda s, r: qfollow(s, r, (3, 9), ((2, 11), (5, 11))), 'q1'),
 'qf_manual_d': (lambda s, r: qfollow(s, r, (11, 1), ((10, 5), (13, 5))), 'q1'),
}
res = {}
def corre(nome, fn, kind, disp):
    w = os.path.join(OUT, nome); os.makedirs(w, exist_ok=True); f = os.path.join(w, 'in.ase')
    if kind == 'q1': m.write_ase(f, [R3, G5], Q1G)
    else: m.write_ase(f, [R, G, B], GRID, base_layer=bool(kind))
    s = m.S(w, disp, f)
    r = {}
    try:
        if kind != 'q1': s.key('3')
        fn(s, r)
    except Exception as e:
        r['erro'] = repr(e)
    finally:
        s.fim()
    res[nome] = r
Q = queue.Queue()
for k, v in CASOS.items():
    if FILT in k: Q.put((k, v))
def op(disp):
    while True:
        try: k, v = Q.get_nowait()
        except queue.Empty: return
        corre(k, v[0], v[1], disp)
ts = [threading.Thread(target=op, args=(120 + i,)) for i in range(5)]
[t.start() for t in ts]; [t.join() for t in ts]
json.dump(res, open(os.path.join(OUT, f'q34{FILT}.json'), 'w'))
def rows(b): return [(i, l) for i, l in enumerate(b) if l.strip('.')]
for k in sorted(res):
    print(k)
    for st, v in res[k].items():
        if isinstance(v, dict): print(' ', st, v['w'], 'x', v['h'], 'ntiles', len(v['tiles']), rows(v['mapa']))
        else: print(' ', st, str(v)[:200])
