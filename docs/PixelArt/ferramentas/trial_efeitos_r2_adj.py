import sys, json, os, time; sys.path.insert(0, sys.argv[1]); run=sys.argv[3]; sys.argv=[sys.argv[0], sys.argv[2]]
import trial_efeitos_menus as m
m.drv.fecha(); m.setup(256, 8, 4, False); print(m.drv.abre('adj.ase', '2'), m.drv.X0, m.drv.Y0, flush=True)
ref = m.ramp_rows()
def chk():
    g = m.grid(m.shot('base.png')); n = sum(tuple(g[j][i]) != ref[j][i] for j in range(8) for i in range(256)); assert n == 0, n
chk()
p = f'adj2_{run}.json'
out = json.load(open(p)) if os.path.exists(p) else {'bc': {}, 'hs': {}}
BC = [(b, 0) for b in range(-100, 101, 10)] + [(0, c) for c in range(-100, 101, 10)] + [(30, -40), (-70, 60)]
for b, c in BC:
    k = f'{b},{c}'
    if k in out['bc'] or (b, c) == (0, 0) and '0,0' in out['bc']: continue
    out['bc'][k] = m.bc(b, c, f'bc2_{b}_{c}'); chk()
    json.dump(out, open(p, 'w')); print(k, [out['bc'][k][3][i][0] for i in range(0, 256, 32)], flush=True)
for mode in ('HSL', 'HSV', 'HSL+', 'HSV+'):
    for h, s, l in [(1,0,0),(10,0,0),(30,0,0),(45,0,0),(0,10,0),(0,-10,0),(0,0,10),(0,0,-10)]:
        k = f'{mode}:{h},{s},{l}'
        if k in out['hs']: continue
        out['hs'][k] = m.hs(mode, h, s, l, f'hs2_{mode}_{h}_{s}_{l}'); chk()
        json.dump(out, open(p, 'w'))
    print(mode, flush=True)
