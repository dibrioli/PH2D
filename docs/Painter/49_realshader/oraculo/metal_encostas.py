"""Régua do METAL NAS ENCOSTAS (K4): o nosso metal × o Principled Metallic do Cycles sobre a MESMA tinta.

    python3 metal_encostas.py <entrada_k4> <saida_k4> [metal1,metal05]

Máscaras do ORÁCULO: a tinta (cobertura > 0,9) e a inclinação da malha que o Cycles rende (`h_fina.f32`, gradiente
central em células): plano |∇h| < 0,15 · encosta 0,4..1,5 · parede > 1,5. Números em LINEAR presos a [0, 1].
"""
import sys

import numpy as np
import OpenImageIO as oiio

ent, sai = sys.argv[1], sys.argv[2]
MATS = sys.argv[3].split(",") if len(sys.argv) > 3 else ["metal1", "metal05"]


def pnm(c):
    b = open(c, "rb").read()
    p = b.split(maxsplit=4)
    return np.frombuffer(p[4], dtype=np.uint8).reshape(int(p[2]), int(p[1]), -1).squeeze().astype(np.float64)


def lin(s):
    s = s / 255.0
    return np.where(s <= 0.04045, s / 12.92, ((s + 0.055) / 1.055) ** 2.4)


tinta = pnm(f"{ent}/neutro/cobertura.pgm") / 255 > 0.9
gh, gw = tinta.shape
h = np.fromfile(f"{ent}/neutro/h_fina.f32", "<f4").reshape(gh, gw).astype(np.float64)
gy, gx = np.gradient(h)
g = np.hypot(gx, gy)
MASC = {"plano": tinta & (g < 0.15), "encosta": tinta & (g >= 0.4) & (g < 1.5), "parede": tinta & (g >= 1.5)}


def q(v, m):
    return " ".join(f"{np.percentile(v[m], p):.3f}" for p in (10, 50, 90))


for mat in MATS:
    n = np.clip(lin(pnm(f"{ent}/{mat}/oleo.ppm")), 0, 1)
    c = np.clip(oiio.ImageBuf(f"{sai}/mat_{mat}_cor.exr").get_pixels(oiio.FLOAT)[..., :3].astype(np.float64), 0, 1)
    print(f"## {mat} (R p10 p50 p90 · lum<0,01 %)")
    for nome, m in MASC.items():
        ln, lc = n @ [0.2126, 0.7152, 0.0722], c @ [0.2126, 0.7152, 0.0722]
        print(f"  {nome:8s} n={m.sum():6d} | nós {q(n[..., 0], m)} · {100 * (ln[m] < 0.01).mean():4.1f} % | "
              f"Cycles {q(c[..., 0], m)} · {100 * (lc[m] < 0.01).mean():4.1f} % | corr R {np.corrcoef(n[..., 0][m], c[..., 0][m])[0, 1]:+.2f}")
