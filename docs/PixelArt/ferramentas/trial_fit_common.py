import json
import numpy as np
from PIL import Image
WD = '/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c142c8eb-76c4-4200-a92d-b3bc73e244a0/scratchpad/oraculo/trial'
def load_gpl(p):
    out = []
    for l in open(p):
        t = l.split()
        if len(t) >= 3 and all(x.isdigit() for x in t[:3]): out.append([int(x) for x in t[:3]])
    return out
PALS = {n: load_gpl(f'{WD}/pal_{n}.gpl') for n in ('gb4', 'bw2', 'db32')}
SIZES = {'cubo': (36, 6), 'grade': (16, 16), 'rampas': (32, 8), 'probe': (72, 40)}
IMGS = {n: np.asarray(Image.open(f'{WD}/rgb_{n}.png').convert('RGB')).astype(np.int64) for n in SIZES}
def records(path):
    for l in open(path):
        r = json.loads(l)
        pal = PALS[r['palette']]
        idx = []
        for c in r['cells']:
            idx.append(pal.index(c) if c in pal else -1)
        r['indices'] = idx
        yield r
def nearest(pal, c):
    P = np.asarray(pal, dtype=np.float64)
    d = ((P - np.asarray(c, dtype=np.float64)) ** 2).sum(1)
    return int(np.argmin(d))
