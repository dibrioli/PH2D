"""Calibra a oclusão do céu contra o Cycles (W1c): V(ocl) e o multi-ricochete por albedo (Jimenez 2016).

    python3 docs/Painter/49_realshader/oraculo/calibra_ao.py <entrada> <saida>
"""
import sys
import numpy as np
import OpenImageIO as oiio
ent, sai = sys.argv[1], sys.argv[2]
def pnm(n):
    b = open(f"{ent}/{n}", "rb").read(); p = b.split(maxsplit=4)
    return np.frombuffer(p[4], np.uint8).reshape(int(p[2]), int(p[1]), -1).squeeze().astype(float)
exr = lambda n: oiio.ImageBuf(f"{sai}/{n}.exr").get_pixels(oiio.FLOAT)[..., :3].astype(float)
lin = lambda s: np.where(s / 255 <= 0.04045, s / 255 / 12.92, ((s / 255 + 0.055) / 1.055) ** 2.4)
ceu = 0.6
tinta = pnm("cobertura.pgm") > 250
ocl = (pnm("t_ocl.pgm") - 1) / 254
V = exr("ceu_cinza_direto").mean(-1) / (0.5 * ceu)
G = exr("ceu_cinza").mean(-1) / (0.5 * ceu)
alb = lin(pnm("albedo.ppm"))
Ec = exr("ceu_cor") / np.maximum(alb, 1e-4) / ceu
def jimenez(v, a):
    A = 2.0404 * a - 0.3324; B = -4.7951 * a + 0.6417; C = 2.7552 * a + 0.6903
    return np.maximum(v, ((v * A + B) * v + C) * v)
print("ocl      n     V(cycles) p25 p50 p75 | G(cinza .5)/V p50 | Jimenez(V,.5)/V")
bins = [0, .02, .05, .08, .12, .16, .2, .25, .3, .35, .45, 1]
for a, b in zip(bins, bins[1:]):
    m = tinta & (ocl >= a) & (ocl < b)
    if m.sum() < 50: continue
    v = V[m]
    print(f"{a:.2f}-{b:.2f} {m.sum():6d}  {np.percentile(v,25):.3f} {np.percentile(v,50):.3f} {np.percentile(v,75):.3f} | {np.median(G[m]/np.maximum(v,1e-3)):.3f} | {np.median(jimenez(v,.5)/np.maximum(v,1e-3)):.3f}")
# Vermelho saturado: o ganho do multi-ricochete por canal
verm = tinta & (alb[..., 0] > 0.4) & (alb[..., 1] < 0.05)
for a, b in [(0, .1), (.1, .2), (.2, .35), (.35, 1)]:
    m = verm & (ocl >= a) & (ocl < b)
    if m.sum() < 50: continue
    r = np.median(Ec[..., 0][m] / np.maximum(V[m], 1e-3)); j = np.median(jimenez(V[m], alb[..., 0][m]) / np.maximum(V[m], 1e-3))
    print(f"vermelho ocl {a:.2f}-{b:.2f} n {m.sum():6d}: V p50 {np.median(V[m]):.3f} · ganho R Cycles {r:.3f} · Jimenez {j:.3f}")
