import sys, json, time; sys.path.insert(0, sys.argv[1]); run=sys.argv[3]; files=sys.argv[4].split(','); sys.argv=[sys.argv[0], sys.argv[2]]
import trial_efeitos_menus as m
out = json.load(open(f'conv2_{run}.json')) if __import__('os').path.exists(f'conv2_{run}.json') else {}
for f in files:
    m.drv.fecha(); m.setup(15, 15, 16, False); m.drv.abre(f + '.ase', '4')
    print('origin', m.drv.X0, m.drv.Y0, flush=True)
    base = m.grid(m.shot('cbase.png'))
    for k in range(39):
        if f'{f}:{k}' in out: continue
        g = m.conv(k, f'cv_{f}_{k}_{run}')
        assert m.grid(m.shot('cbase.png')) == base, ('undo falhou', f, k)
        out[f'{f}:{k}'] = g
        print(f, k, m.KERNELS[k], [g[7][i][0] for i in range(15)], flush=True)
        json.dump(out, open(f'conv2_{run}.json', 'w'))
