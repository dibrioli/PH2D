import sys
sys.path.insert(0, '/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c142c8eb-76c4-4200-a92d-b3bc73e244a0/scratchpad/oraculo/tools')
import numpy as np
from trial_fit_common import *
def lin(c):
    c = np.asarray(c, float) / 255
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
def xyz(c):
    M = np.array([[0.4124, 0.3576, 0.1805], [0.2126, 0.7152, 0.0722], [0.0193, 0.1192, 0.9505]])
    return lin(c) @ M.T
def lab(c):
    X = xyz(c) / np.array([0.95047, 1.0, 1.08883])
    f = np.where(X > 216 / 24389, np.cbrt(X), (24389 / 27 * X + 16) / 116)
    return np.stack([116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]), 200 * (f[..., 1] - f[..., 2])], -1)
SP = {'rgb': lambda c: np.asarray(c, float), 'lin': lin, 'xyz': xyz, 'lab': lab,
      'w2421': lambda c: np.asarray(c, float) * np.sqrt([2, 4, 3]),
      'w3_4_2': lambda c: np.asarray(c, float) * np.sqrt([3, 4, 2])}
def near(pal, px, sp):
    P = SP[sp](pal); X = SP[sp](px)
    d = ((X[:, None, :] - P[None]) ** 2).sum(-1)
    return d.argmin(1)
path = sys.argv[1]
for r in records(path):
    if r['dither'] != 0: continue
    px = IMGS[r['image']].reshape(-1, 3); pal = PALS[r['palette']]
    got = np.array(r['indices'])
    res = {sp: int((near(pal, px, sp) != got).sum()) for sp in SP}
    print(r['image'], r['palette'], 'map', r['mapping'], 'fit', r['best_fit'], 'unk', int((got < 0).sum()), res)
