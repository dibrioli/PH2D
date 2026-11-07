"""P10 M9b no Trial 1.3.18.6 (Auto): A=1 B=2 C=3; ordem da lista e quem guarda o original em traco horizontal, vertical e diagonal.
Sessao nova por caso. uso: python3 -I trial_tiles_m9b.py <dir>   grava <dir>/m9b.json"""
import json, os, sys, threading, queue, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_tiles_m9 as m
OUT = os.path.abspath(sys.argv[1]); X0, Y0 = 820, 464
P = lambda p: (X0 + p[0] * 8 + 3, Y0 + p[1] * 8 + 3)
A = m.tile_row(2, 2, 4, 'R'); B = m.tile_row(3, 2, 5, 'G'); C = m.tile_row(6, 2, 5, 'G')
CASOS = [
 ('c1_iguais', [(0, 0), (1, 0)], (0, 2), (1, 2), (0, 1), (15, 1)),
 ('c2_diferentes', [(0, 0), (1, 0)], (0, 2), (1, 2), (6, 1), (9, 1)),
 ('c2_diferentes_inv', [(0, 0), (1, 0)], (0, 2), (1, 2), (9, 1), (6, 1)),
 ('c3_vert_desce', [(0, 0), (0, 1)], (2, 0), (3, 0), (1, 6), (1, 9)),
 ('c3_vert_sobe', [(0, 0), (0, 1)], (2, 0), (3, 0), (1, 9), (1, 6)),
 ('c3_vert_desce_3x', [(0, 0), (0, 1), (3, 3)], (2, 0), (3, 0), (1, 6), (1, 9)),
 ('c3_vert_sobe_3x', [(0, 0), (0, 1), (3, 3)], (2, 0), (3, 0), (1, 9), (1, 6)),
 ('c4_diag_1_0_para_0_1', [(0, 1), (1, 0)], (3, 3), (2, 3), (10, 5), (5, 10)),
 ('c4_diag_0_1_para_1_0', [(0, 1), (1, 0)], (3, 3), (2, 3), (5, 10), (10, 5)),
 ('c4_diag_1_0_para_0_1_3x', [(0, 1), (1, 0), (3, 0)], (3, 3), (2, 3), (10, 5), (5, 10)),
 ('c4_diag_0_1_para_1_0_3x', [(0, 1), (1, 0), (3, 0)], (3, 3), (2, 3), (5, 10), (10, 5)),
]
res = {}
def corre(c, disp):
    nome, ca, cb, cc, a, b = c
    w = os.path.join(OUT, nome); os.makedirs(w, exist_ok=True); f = w + '/in.ase'
    g = {x: 1 for x in ca}; g[cb] = 2; g[cc] = 3
    m.write_ase(f, [A, B, C], g)
    s = m.S(w, disp, f)
    try:
        s.key('3'); s.x('mousemove', *m.BOT['auto'], 'click', 1); time.sleep(.3); s.key('b')
        a0 = s.shot('antes.png')
        s.drag(P(a), P(b))
        a1 = s.shot('depois.png')
        res[nome] = {'antes_tiles': m.tiles_panel(a0), 'depois_mapa': m.canvas(a1, X0, Y0), 'depois_tiles': m.tiles_panel(a1)}
    except Exception as e:
        res[nome] = {'erro': repr(e)}
    finally:
        s.fim()
Q = queue.Queue(); [Q.put(c) for c in CASOS]
def op(d):
    while True:
        try: c = Q.get_nowait()
        except queue.Empty: return
        corre(c, d)
ts = [threading.Thread(target=op, args=(150 + i,)) for i in range(5)]
[t.start() for t in ts]; [t.join() for t in ts]
json.dump(res, open(OUT + '/m9b.json', 'w'))
def rows(b): return [(i, l) for i, l in enumerate(b) if l.strip('.')]
for c in CASOS:
    r = res[c[0]]
    if 'erro' in r: print(c[0], r); continue
    print(c[0], 'n', len(r['antes_tiles']), '->', len(r['depois_tiles']), m.cells_of(r['depois_mapa'], r['depois_tiles']))
    print('   ', [rows(t) for t in r['depois_tiles']])
