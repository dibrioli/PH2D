"""Ajusta candidatos de dither aos dados observados no Aseprite Trial (exact nearest RGB)."""
import sys, itertools
sys.path.insert(0, '/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c142c8eb-76c4-4200-a92d-b3bc73e244a0/scratchpad/oraculo/tools')
from trial_fit_common import *
def bayer(n):
    M = np.array([[0, 2], [3, 1]])
    while M.shape[0] < n:
        M = np.block([[4 * M, 4 * M + 2], [4 * M + 3, 4 * M + 1]])
    return M
NS = {1: 2, 2: 4, 3: 8, 4: 2, 5: 4, 6: 8}
def nn(P, X):
    d = ((X[..., None, :] - P) ** 2).sum(-1); return d.argmin(-1)
def ordA(img, P, n, S, off=0.5, tr=False):
    M = bayer(n); H, W, _ = img.shape
    yy, xx = np.mgrid[0:H, 0:W]
    t = (M[xx % n, yy % n] if tr else M[yy % n, xx % n]) / (n * n) + (0.5 / (n * n) if off == 'c' else 0) - (0.5 if off != 'c' else 0.5)
    X = np.clip(img + t[..., None] * S, 0, 255)
    return nn(P, X).ravel()
def ordB(img, P, n, pen=0.0, tr=False):
    """Yliluoma alg.1: par (i,j) e razão k/n² que minimiza |c - mix|² + pen*|ci-cj|²."""
    M = bayer(n); H, W, _ = img.shape; N = n * n
    out = np.zeros((H, W), int); K = len(P)
    cache = {}
    for y in range(H):
        for x in range(W):
            c = tuple(img[y, x])
            if c not in cache:
                best = None
                for i in range(K):
                    for j in range(i, K):
                        for k in range(N + 1):
                            mix = P[i] + (P[j] - P[i]) * k / N
                            e = ((np.array(c) - mix) ** 2).sum() + pen * ((P[i] - P[j]) ** 2).sum()
                            if best is None or e < best[0]: best = (e, i, j, k)
                cache[c] = best
            _, i, j, k = cache[c]
            m = M[x % n, y % n] if tr else M[y % n, x % n]
            out[y, x] = j if m < k else i
    return out.ravel()
def old(img, P, n, mult=1.0, var='ratio', tr=False):
    M = bayer(n); H, W, _ = img.shape; N = n * n
    out = np.zeros((H, W), int)
    for y in range(H):
        for x in range(W):
            c = img[y, x]; i = int(nn(P, c[None])[0]); e = c - P[i]
            j = int(nn(P, np.clip(c + mult * e, 0, 255)[None])[0])
            m = (M[x % n, y % n] if tr else M[y % n, x % n])
            if j == i: out[y, x] = i; continue
            if var == 'ratio':
                r = np.sqrt((e ** 2).sum()) / np.sqrt(((P[j] - P[i]) ** 2).sum())
            else:  # projecção
                v = P[j] - P[i]; r = (e @ v) / (v @ v)
            out[y, x] = j if m / N < r else i
    return out.ravel()
def fs(img, P, serp=False, integer=False):
    H, W, _ = img.shape; E = img.astype(float).copy(); out = np.zeros((H, W), int)
    for y in range(H):
        rng = range(W) if not (serp and y % 2) else range(W - 1, -1, -1); s = 1 if not (serp and y % 2) else -1
        for x in rng:
            c = np.clip(E[y, x], 0, 255)
            if integer: c = np.floor(c)
            i = int(nn(P, c[None])[0]); out[y, x] = i; e = E[y, x] - P[i]
            if integer: e = np.trunc(e)
            for dx, dy, w in ((s, 0, 7), (-s, 1, 3), (0, 1, 5), (s, 1, 1)):
                if 0 <= x + dx < W and y + dy < H:
                    E[y + dy, x + dx] += (np.trunc(e * w / 16) if integer else e * w / 16)
    return out.ravel()
def cands(d, img, P):
    K = len(P); out = {}
    if d == 0:
        out['nearest'] = nn(P, img.astype(float)).ravel()
    elif d in (1, 2, 3):
        n = NS[d]
        for S in (127.5, 255 / (K - 1), 64, 128, 255):
            for tr in (False, True):
                out[f'A S={S:.1f} tr={tr}'] = ordA(img, P, n, S, tr=tr)
        for pen in (0.0, 0.1, 0.5):
            out[f'B pen={pen}'] = ordB(img, P, n, pen)
    elif d in (4, 5, 6):
        n = NS[d]
        for mult, var in itertools.product((1.0, 2.0), ('ratio', 'proj')):
            out[f'old mult={mult} {var}'] = old(img, P, n, mult, var)
    else:
        for serp, integer in itertools.product((False, True), (False, True)):
            out[f'fs serp={serp} int={integer}'] = fs(img, P, serp, integer)
    return out
def luma(c): return 299 * c[0] + 587 * c[1] + 114 * c[2]
def knoll(img, P, n, mult=1.0, tr=False, sortkey='luma'):
    M = bayer(n); H, W, _ = img.shape; N = n * n; out = np.zeros((H, W), int); cache = {}
    for y in range(H):
        for x in range(W):
            c = img[y, x]; key = tuple(c)
            if key not in cache:
                err = np.zeros(3); cl = []
                for _ in range(N):
                    i = int(nn(P, np.clip(c + err * mult, 0, 255)[None])[0]); cl.append(i); err += c - P[i]
                cl.sort(key=(lambda i: luma(P[i])) if sortkey == 'luma' else (lambda i: i))
                cache[key] = cl
            m = M[x % n, y % n] if tr else M[y % n, x % n]
            out[y, x] = cache[key][m]
    return out.ravel()
def yl2(img, P, n, tr=False):
    M = bayer(n); H, W, _ = img.shape; N = n * n; out = np.zeros((H, W), int); cache = {}
    for y in range(H):
        for x in range(W):
            c = img[y, x]; key = tuple(c)
            if key not in cache:
                cl = []; s = np.zeros(3)
                for k in range(N):
                    best = min(range(len(P)), key=lambda i: (((s + P[i]) / (k + 1) - c) ** 2).sum())
                    cl.append(best); s += P[best]
                cl.sort(key=lambda i: luma(P[i])); cache[key] = cl
            m = M[x % n, y % n] if tr else M[y % n, x % n]
            out[y, x] = cache[key][m]
    return out.ravel()
_old_cands = cands
def cands(d, img, P):
    out = _old_cands(d, img, P)
    if d in (1, 2, 3, 4, 5, 6):
        n = NS[d]
        for mult in (0.5, 1.0):
            for tr in (False, True):
                out[f'knoll m={mult} tr={tr}'] = knoll(img, P, n, mult, tr)
        out['yl2'] = yl2(img, P, n); out['yl2 tr'] = yl2(img, P, n, True)
    return out

if __name__ == '__main__':
    path = sys.argv[1]; want = sys.argv[2:] and [int(a) for a in sys.argv[2:]]
    for r in records(path):
        if r['mapping'] != 0 or r['best_fit'] != 1: continue
        if want and r['dither'] not in want: continue
        img = IMGS[r['image']].astype(float); P = np.array(PALS[r['palette']], float)
        got = np.array(r['indices'])
        res = sorted(((c != got).sum(), k) for k, c in cands(r['dither'], img, P).items())
        print(r['image'], r['palette'], 'd', r['dither'], 'unk', int((got < 0).sum()), 'N', len(got), res[:4])
