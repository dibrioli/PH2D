"""Candidatas de visibilidade do céu (GTAO, Jimenez 2016, vista = +Z) contra o Cycles, numa rodada só.

    python3 docs/Painter/49_realshader/oraculo/candidatas_ao.py <entrada> <saida>
"""
import sys, math
import numpy as np
import OpenImageIO as oiio
ent, sai = sys.argv[1], sys.argv[2]
txt = dict(l.strip().split("=", 1) for l in open(f"{ent}/oraculo.txt") if "=" in l and not l.startswith("#"))
gw, gh = int(txt["gw"]), int(txt["gh"])
def pnm(n):
    b = open(f"{ent}/{n}", "rb").read(); p = b.split(maxsplit=4)
    return np.frombuffer(p[4], np.uint8).reshape(int(p[2]), int(p[1]), -1).squeeze().astype(float)
H = {n: np.fromfile(f"{ent}/{n}.f32", "<f4").reshape(gh, gw).astype(float) for n in ("h_forma", "h_fina", "h_sombra")}
tinta = pnm("cobertura.pgm") > 250
V = oiio.ImageBuf(f"{sai}/ceu_cinza_direto.exr").get_pixels(oiio.FLOAT)[..., :3].mean(-1) / 0.3
P = 32
def at(h, dx, dy):
    hp = np.pad(h, P, mode="edge")
    return hp[P + dy:P + dy + gh, P + dx:P + dx + gw]
def sobel(h):
    gx = (at(h,1,-1)+2*at(h,1,0)+at(h,1,1)-at(h,-1,-1)-2*at(h,-1,0)-at(h,-1,1))/8
    gy = (at(h,-1,1)+2*at(h,0,1)+at(h,1,1)-at(h,-1,-1)-2*at(h,0,-1)-at(h,1,-1))/8
    return gx, gy
def gtao(hh, hn, N, K, passo=1.0):
    gx, gy = sobel(hn)
    inv = 1 / np.sqrt(gx*gx + gy*gy + 1)
    n = np.stack([-gx*inv, -gy*inv, inv], -1)
    acc = np.zeros((gh, gw))
    for d in range(N):
        phi = math.pi * d / N
        dx, dy = math.cos(phi), math.sin(phi)
        # Horizonte (seno máximo da elevação) nos dois sentidos da fatia.
        s1 = np.full((gh, gw), -1.0); s2 = np.full((gh, gw), -1.0)
        for t in range(1, K + 1):
            r = t * passo
            ox, oy = int(round(dx * r)), int(round(dy * r))
            if ox == 0 and oy == 0: continue
            dist = math.hypot(ox, oy)
            for sgn, s in ((1, s1), (-1, s2)):
                dh = at(hh, sgn*ox, sgn*oy) - hh
                np.maximum(s, dh / np.sqrt(dh*dh + dist*dist), out=s)
        # Ângulos a partir do zénite (vista); a normal projectada na fatia.
        ax = np.array([dx, dy, 0.0]); orto = np.array([-dy, dx, 0.0])
        npj = n - (n @ orto)[..., None] * orto
        L = np.linalg.norm(npj, axis=-1)
        cg = np.clip(npj[..., 2] / np.maximum(L, 1e-9), -1, 1)
        g = np.sign(npj @ ax) * np.arccos(cg)
        h1 = -np.arccos(np.clip(s2, -1, 1)) * 0 + (np.arcsin(np.clip(s1, -1, 1)))  # elevação
        t1 = math.pi/2 - np.arcsin(np.clip(s1, -1, 1))  # do zénite, lado +
        t2 = math.pi/2 - np.arcsin(np.clip(s2, -1, 1))  # lado −
        a1 = g + np.minimum(t1, math.pi/2 - g)   # horizonte + , preso ao hemisfério da normal
        a2 = g - np.minimum(t2, math.pi/2 + g)   # horizonte −
        v = 0.25*(-np.cos(2*a1 - g) + np.cos(g) + 2*a1*np.sin(g)) + 0.25*(-np.cos(2*a2 - g) + np.cos(g) + 2*a2*np.sin(g))
        acc += L * v
    return acc / N
print(f"Cycles V na tinta: p10 {np.percentile(V[tinta],10):.3f} p50 {np.median(V[tinta]):.3f} p90 {np.percentile(V[tinta],90):.3f}")
print("candidata (horizonte · normal · N×K)        viés p50   |erro| p50  p90   corr")
for hh in ("h_forma", "h_sombra", "h_fina"):
    for hn in ("h_fina",):
        for N, K, passo in ((4, 8, 1), (8, 8, 1), (8, 16, 1), (8, 12, 2)):
            v = gtao(H[hh], H[hn], N, K, passo)
            e = (v - V)[tinta]
            print(f"{hh:<8} · {hn:<7} · {N}×{K} passo {passo}   {np.median(e):+.3f}   {np.median(abs(e)):.3f} {np.percentile(abs(e),90):.3f}  {np.corrcoef(v[tinta], V[tinta])[0,1]:.3f}")
