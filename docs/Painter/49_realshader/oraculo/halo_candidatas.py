"""Candidatas da sombra no PAPEL (K4) sobre os termos do papel da sonda × o Cycles (cor), por faixa e lado.

    python3 halo_candidatas.py <entrada> <saida_cycles> [<variante estudio> <variante classico>]

`<entrada>/cobertura.pgm` (ou `<entrada>/<variante>/`) e `<entrada>/<variante>/t_papel.f32` da sonda; os `*_cor.exr`
do Cycles. A cena dos traços lisos: `entrada_halo saida_halo estudio classico`; a do dono: `entrada_k4 saida_k4
neutro classico`.
"""
import sys

import numpy as np
from PIL import Image

import halo_regua as H

ent, sai = sys.argv[1], sys.argv[2]
VAR = {"estudio": sys.argv[3] if len(sys.argv) > 3 else "estudio",
       "classico": sys.argv[4] if len(sys.argv) > 4 else "classico"}
cob = f"{ent}/cobertura.pgm"
try:
    open(cob).close()
except FileNotFoundError:
    cob = f"{ent}/{VAR['estudio']}/cobertura.pgm"
tinta = np.asarray(Image.open(cob)) > 25
gh, gw = tinta.shape
d = H.distancia(tinta)
lado = H.lado_da_sombra(tinta)
FX = [(3, 6), (6, 12), (12, 24)]


def mr(v, a):
    A, B, C = 2.0404 * a - 0.3324, -4.7951 * a + 0.6417, 2.7552 * a + 0.6903
    return np.maximum(v, ((v * A + B) * v + C) * v)


def bandas(x):
    out = []
    for rot, m0 in [("luz", ~lado), ("sombra", lado)]:
        out.append(rot + " " + " ".join(f"{np.percentile(x[(d >= a) & (d < b) & m0], 50):+.3f}" for a, b in FX))
    return " | ".join(out)


def enc(x):
    x = np.clip(x, 0, 1)
    return np.where(x <= 0.0031308, 12.92 * x, 1.055 * x ** (1 / 2.4) - 0.055)


def dec(s):
    return np.where(s <= 0.04045, s / 12.92, ((s + 0.055) / 1.055) ** 2.4)


F = 0.6
for amb, cy in [("classico", "sol10_cor"), ("estudio", "studio_sol10_cor")]:
    v_dir = f"{ent}/v/{VAR[amb]}" if VAR[amb] in ("estudio", "classico") and "halo" in ent else f"{ent}/{VAR[amb]}"
    t = np.fromfile(f"{v_dir}/t_papel.f32", "<f4").reshape(gh, gw, 3).astype(np.float64)
    luz, v, g = t[..., 0], t[..., 1], t[..., 2]
    lum = H.luminancia(f"{sai}/{cy}.exr")[0]
    ref = np.percentile(lum[d >= 41], 50)
    print(f"## {amb}\n  Cycles            {bandas(1 - lum / ref)}")
    directa = (1 - F) * (1 - luz)
    s = np.clip(directa + F * (1 - v * g), 0, 1)
    # ⚠️ os `t_papel` são de HOJE (a lei K4 não muda luz, V nem o ganho): as candidatas refazem a sombra deles.
    print(f"  hoje (sRGB)       {bandas(1 - dec(1 - s))}")
    print(f"  linear            {bandas(s)}")
    for a in (0.5, 0.8, 1.0):
        rec = v * g + (mr(v, a) - v)
        s2 = np.clip(directa + F * (1 - rec), 0, 1)
        print(f"  linear+mr(a={a})  {bandas(s2)}")
