"""Ajusta a lei do GRUPO e do destino nao-opaco aos g*.ase / gm*.ase lidos no Trial.
uso: python3 -I trial_fit_grupos.py <dir>
g{MM}: back opaca; grupo Normal 255 com [mid Normal, src modo MM].
  iso  = o grupo compoe-se sobre transparente e o resultado vai Normal sobre a back;
  pass = mid e src compoem-se directamente sobre a back.
Para o destino com alfa < 255 (so no iso) testam-se duas leis da cor misturada:
  A    = a cor B(d, s) entra por source-over como se o destino fosse opaco;
  W3C  = s' = (1 - da)·s + da·B(d, s), depois source-over (Compositing §5.8);
  W3Ci = idem em inteiros, s' = s + mul(da, B - s) (trunca para zero).
gm{MM}: grupo com modo MM contendo src Normal: o grupo mistura o seu composto com o seu modo."""
import json, os, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import importlib.util
spec = importlib.util.spec_from_file_location('f', os.path.join(os.path.dirname(os.path.abspath(__file__)), 'trial_fit_camadas.py'))

D = sys.argv[1]
E = json.load(open(os.path.join(D, 'entradas.json')))
L = json.load(open(os.path.join(D, 'lido.json')))
sys.argv = [sys.argv[0], D]
F = importlib.util.module_from_spec(spec); spec.loader.exec_module(F)
B = np.array(E['back'], dtype=np.int64)
S = np.array(E['src'], dtype=np.int64)
M = np.array(E['mid'], dtype=np.int64)
mul, tdiv = F.mul, F.tdiv

def best_blend(m):
    # a melhor candidata medida em m{MM}_o255
    n = f'm{m:02d}_o255.ase'
    name = F.res[n][0][2].split('/')[0] if n in F.res else None
    c = F.cands(m)
    return c[name] if name in c else list(c.values())[0]

def mix(m, d3, s3):
    f = best_blend(m)
    r = f(d3, s3) if m in F.NONSEP else np.stack([f(d3[:, k], s3[:, k]) for k in range(3)], 1)
    return np.clip(r, 0, 255)

def comp(m, dst, src, op, law):
    sa = mul(src[:, 3], op)
    da = dst[:, 3]
    bm = mix(m, dst[:, :3], src[:, :3])
    if law == 'A':
        c = bm
    elif law == 'W3C':
        c = np.floor(((255 - da)[:, None] * src[:, :3] + da[:, None] * bm) / 255 + 0.5).astype(np.int64)
    else:
        c = src[:, :3] + tdiv((bm - src[:, :3]) * da[:, None], 255)
    ra = sa + da - mul(da, sa)
    ra_s = np.maximum(ra, 1)[:, None]
    o = dst[:, :3] + tdiv((c - dst[:, :3]) * sa[:, None], ra_s)
    o = np.where((da == 0)[:, None], src[:, :3], o)
    a = np.where(da == 0, sa, ra)
    o = np.where((sa == 0)[:, None], dst[:, :3], o)
    a = np.where(sa == 0, da, a)
    return np.concatenate([o, a[:, None]], 1)

Z = np.zeros_like(B)
for n, cells in sorted(L.items()):
    got = np.array(cells, dtype=np.int64)
    rows = []
    if n.startswith('g') and not n.startswith('gm'):
        m = int(n[1:3])
        for law in ('A', 'W3C', 'W3Ci'):
            inner = comp(m, comp(0, Z, M, 255, law), S, 255, law)
            iso = comp(0, B, inner, 255, law)[:, :3]
            rows.append((int(np.all(iso == got, 1).sum()), f'iso/{law}'))
        pas = comp(m, comp(0, B, M, 255, 'A'), S, 255, 'A')[:, :3]
        rows.append((int(np.all(pas == got, 1).sum()), 'pass'))
    elif n.startswith('gm'):
        m, op = int(n[2:4]), int(n[6:9])
        iso = comp(m, B, comp(0, Z, S, 255, 'A'), op, 'A')[:, :3]
        rows.append((int(np.all(iso == got, 1).sum()), 'grupo-mistura-com-o-modo'))
        pas = comp(0, B, S, 255, 'A')[:, :3]
        rows.append((int(np.all(pas == got, 1).sum()), 'modo-do-grupo-ignorado'))
    else:
        continue
    rows.sort(reverse=True)
    print(n, len(got), rows, flush=True)
