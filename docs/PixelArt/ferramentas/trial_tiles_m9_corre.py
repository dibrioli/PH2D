"""P10 M9 Q1/Q2/Q5 no Trial: cada caso = uma sessao nova (Xvfb, HOME limpo) sobre um .ase nosso; 1 traco; le mapa e tileset pela foto.
uso: python3 -I trial_tiles_m9_corre.py <dir> [filtro]   grava <dir>/q125.json; fotos em <dir>/<caso>/"""
import json, os, sys, threading, queue
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_tiles_m9 as m
OUT = os.path.abspath(sys.argv[1]); FILT = sys.argv[2] if len(sys.argv) > 2 else ''
X0, Y0 = 820, 464   # canvas a 400 %, calibrado na foto
if os.environ.get('M9_X0'): X0, Y0 = int(os.environ['M9_X0']), int(os.environ['M9_Y0'])
def P(p): return (X0 + p[0] * 8 + 3, Y0 + p[1] * 8 + 3)
R3 = m.tile_row(3, 2, 5, 'R'); G5 = m.tile_row(5, 2, 5, 'G'); R3u = m.tile_row(3, 2, 2, 'R'); G5u = m.tile_row(5, 2, 2, 'G')
Q1G = {(0, 1): 1, (0, 0): 2, (1, 0): 2, (2, 0): 2}
Q2G2 = {(0, 0): 1, (1, 0): 1}; Q2G3 = {(0, 0): 1, (1, 0): 1, (3, 0): 1}; R5 = m.tile_row(5, 2, 5, 'R')
CASOS = {}
for mode in ('manual', 'auto', 'stack'):
    for nome, ts, st in (('a_1x_um', [R3, G5], ((3, 11), (3, 11))), ('b_1x_todos', [R3, G5], ((2, 11), (5, 11))),
                         ('c_3x_um', [R3, G5], ((11, 5), (11, 5))), ('d_3x_todos', [R3, G5], ((10, 5), (13, 5))),
                         ('b1_1x_pixelunico', [R3u, G5u], ((2, 11), (2, 11))), ('d1_3x_pixelunico', [R3u, G5u], ((10, 5), (10, 5)))):
        CASOS[f'q1_{mode}_{nome}'] = dict(mode=mode, tool='e', ts=ts, grid=Q1G, st=st)
    CASOS[f'q1_{mode}_ctrl_lapis'] = dict(mode=mode, tool='b', ts=[R3, G5], grid=Q1G, st=((11, 1), (11, 1)))
    for nome, g, st in (('a_2x_diferente', Q2G2, ((6, 1), (9, 1))), ('a_rev', Q2G2, ((9, 1), (6, 1))), ('b_2x_igual', Q2G2, ((0, 1), (15, 1))),
                        ('c_3x_diferente', Q2G3, ((6, 1), (9, 1))), ('c2_3x_igual', Q2G3, ((0, 1), (15, 1))), ('e_2x_so_um', Q2G2, ((2, 1), (4, 1)))):
        CASOS[f'q2_{mode}_{nome}'] = dict(mode=mode, tool='b', ts=[R5], grid=g, st=st)
    CASOS[f'q5_{mode}'] = dict(mode=mode, tool='b', ts=[R5], grid={(0, 0): 1}, grid2={(1, 1): 1}, st=((2, 1), (2, 1)))
CASOS['ctrl_raster_borracha'] = dict(mode='auto', tool='e', ts=[R3, G5], grid=Q1G, st=None)
res = {}
def corre(nome, c, disp):
    w = os.path.join(OUT, nome); os.makedirs(w, exist_ok=True)
    f = os.path.join(w, 'in.ase')
    m.write_ase(f, c['ts'], c['grid'], grid2=c.get('grid2'))
    s = m.S(w, disp, f)
    try:
        s.key('3')                       # 400 %
        s.x('mousemove', *m.BOT[c['mode']], 'click', 1); import time; time.sleep(.4)
        s.key(c['tool'])
        s.x('mousemove', 230, 72, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(.3)
        s.key('ctrl+a'); s.key('1'); s.key('Return')
        a0 = s.shot('antes.png')
        r = {'antes_mapa': m.canvas(a0, X0, Y0), 'antes_tiles': m.tiles_panel(a0)}
        if c['st']:
            s.drag(P(c['st'][0]), P(c['st'][1]))
            a1 = s.shot('depois.png')
            r['depois_mapa'] = m.canvas(a1, X0, Y0); r['depois_tiles'] = m.tiles_panel(a1)
            s.key('ctrl+z'); a2 = s.shot('undo1.png')
            r['undo1_igual_antes'] = m.canvas(a2, X0, Y0) == r['antes_mapa'] and m.tiles_panel(a2) == r['antes_tiles']
        res[nome] = r
    finally:
        s.fim()
Q = queue.Queue()
for k, v in CASOS.items():
    if FILT in k: Q.put((k, v))
def op(disp):
    while True:
        try: k, v = Q.get_nowait()
        except queue.Empty: return
        try: corre(k, v, disp)
        except Exception as e: res[k] = {'erro': repr(e)}
ts = [threading.Thread(target=op, args=(100 + i,)) for i in range(int(os.environ.get('M9_PAR', '5')))]
[t.start() for t in ts]; [t.join() for t in ts]
json.dump(res, open(os.path.join(OUT, f'q125{FILT}.json'), 'w'))
for k in sorted(res):
    r = res[k]
    if 'erro' in r: print(k, r); continue
    if 'depois_tiles' not in r: continue
    print(k, 'tiles', len(r['antes_tiles']), '->', len(r['depois_tiles']), 'mapa', m.cells_of(r['depois_mapa'], r['depois_tiles']), 'undo1ok', r['undo1_igual_antes'])
